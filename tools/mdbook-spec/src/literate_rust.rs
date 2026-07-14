use anyhow::{Context, bail};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::parse::{Parse, ParseStream};
use syn::{ItemMacro, PathArguments, Token};

const RUSTDOC_SOURCE_CRATE: &str = "rust_via_desugarings";

#[derive(Debug, Clone)]
pub struct ChapterInfo {
    pub source_path: String,
    html_path: String,
    visible_lines: BTreeSet<usize>,
}

#[derive(Debug, Clone)]
pub struct LinkRange {
    start: usize,
    end: usize,
    text: String,
    href: String,
}

#[derive(Debug, Clone)]
struct RustdocAnchor {
    href: String,
    text: String,
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct ActiveAnchor {
    href: String,
    start: usize,
    text: String,
}

pub fn is_literate_chapter(chapter: &Value) -> bool {
    chapter_source_path(chapter).is_some_and(|path| path.ends_with(".md.rs"))
}

pub fn chapter_source_path(chapter: &Value) -> Option<String> {
    let path = chapter
        .get("source_path")
        .or_else(|| chapter.get("path"))
        .and_then(Value::as_str)?;

    Some(if path.starts_with("book/") {
        path.to_owned()
    } else {
        format!("book/{path}")
    })
}

pub fn collect_chapters(sections: &Value) -> Vec<ChapterInfo> {
    let mut chapters = Vec::new();
    collect_chapters_in_sections(sections, &mut chapters);
    chapters
}

fn collect_chapters_in_sections(sections: &Value, chapters: &mut Vec<ChapterInfo>) {
    let Some(sections) = sections.as_array() else {
        return;
    };

    for section in sections {
        if let Some(chapter) = section.get("Chapter") {
            if let Some(source_path) = chapter_source_path(chapter)
                && source_path.ends_with(".md.rs")
            {
                let content = chapter.get("content").and_then(Value::as_str).unwrap_or("");
                chapters.push(ChapterInfo {
                    html_path: html_path_for_source(&source_path),
                    visible_lines: visible_lines_in_content(content),
                    source_path,
                });
            }
            collect_chapters_in_sections(&chapter["sub_items"], chapters);
        } else if let Some(part) = section.get("PartTitle") {
            collect_chapters_in_sections(&part["sub_items"], chapters);
        }
    }
}

pub fn render_chapter(
    content: &str,
    source_path: &str,
    links_by_line: Option<&BTreeMap<usize, Vec<LinkRange>>>,
) -> anyhow::Result<String> {
    let mut result = Vec::new();
    let mut code_buffer = Vec::new();
    let mut lines = content.lines().enumerate().peekable();

    while let Some((index, line)) = lines.next() {
        let line_no = index + 1;
        if let Some(markdown) = markdown_comment(line) {
            flush_code(&mut result, &mut code_buffer, source_path);
            result.push(markdown.to_owned());
        } else if is_interactive_example_start(line) {
            flush_code(&mut result, &mut code_buffer, source_path);
            let invocation = collect_interactive_example_invocation(line, &mut lines)?;
            let example = parse_interactive_example(&invocation, source_path, line_no)?;
            result.push(render_interactive_example(&example));
        } else if !is_hidden_code_line(line) {
            code_buffer.push((line_no, line.to_owned()));
        }
    }

    flush_code(&mut result, &mut code_buffer, source_path);

    if let Some(links_by_line) = links_by_line
        && !links_by_line.is_empty()
    {
        result.push(format!(
            r#"<script type="application/json" class="literate-rust-links">{}</script>"#,
            serialize_links(links_by_line),
        ));
    }

    let mut output = result.join("\n");
    output.push('\n');
    Ok(output)
}

fn flush_code(result: &mut Vec<String>, code_buffer: &mut Vec<(usize, String)>, source_path: &str) {
    if code_buffer.is_empty() {
        return;
    }

    let source_lines = code_buffer
        .iter()
        .map(|(line_no, _)| line_no.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    result.push(format!(
        r#"<div class="literate-rust-source" hidden data-source-path="{}" data-source-lines="{}"></div>"#,
        html_escape(source_path),
        html_escape(&source_lines),
    ));
    result.push(String::new());
    result.push("```rust,noplayground".to_owned());
    result.extend(code_buffer.iter().map(|(_, line)| line.clone()));
    result.push("```".to_owned());
    code_buffer.clear();
}

fn markdown_comment(line: &str) -> Option<&str> {
    let line = line.trim_start_matches([' ', '\t']);
    let markdown = line.strip_prefix("//@")?;
    Some(markdown.strip_prefix(' ').unwrap_or(markdown))
}

fn is_interactive_example_start(line: &str) -> bool {
    line.trim_start().starts_with("interactive_example!")
}

fn collect_interactive_example_invocation<'a>(
    first_line: &str,
    lines: &mut std::iter::Peekable<impl Iterator<Item = (usize, &'a str)>>,
) -> anyhow::Result<String> {
    let mut invocation = first_line.to_owned();

    loop {
        if let Ok(item_macro) = syn::parse_str::<ItemMacro>(&invocation) {
            if item_macro.mac.path.is_ident("interactive_example") {
                return Ok(invocation);
            }

            bail!(
                "expected `interactive_example!` macro, got `{}`",
                invocation
            );
        }

        let Some((_index, line)) = lines.next() else {
            bail!("unterminated `interactive_example!` macro invocation");
        };
        invocation.push('\n');
        invocation.push_str(line);
    }
}

struct InteractiveExample {
    id: String,
    step: String,
    sample: String,
}

struct InteractiveExampleArgs {
    step: syn::Path,
}

impl Parse for InteractiveExampleArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let step = input.parse()?;
        input.parse::<Token![,]>()?;
        while !input.is_empty() {
            input.parse::<syn::Item>()?;
        }
        Ok(Self { step })
    }
}

fn parse_interactive_example(
    invocation: &str,
    source_path: &str,
    line_no: usize,
) -> anyhow::Result<InteractiveExample> {
    let item_macro: ItemMacro = syn::parse_str(invocation).with_context(|| {
        format!("could not parse interactive example at {source_path}:{line_no}")
    })?;
    let args: InteractiveExampleArgs = syn::parse2(item_macro.mac.tokens).with_context(|| {
        format!("could not parse interactive example at {source_path}:{line_no}")
    })?;

    Ok(InteractiveExample {
        id: format!("{}:{line_no}", rustc_source_path(source_path)),
        step: format_path(&args.step)?,
        sample: interactive_example_body(invocation, source_path, line_no)?,
    })
}

fn rustc_source_path(source_path: &str) -> String {
    if source_path.starts_with("src/") {
        source_path.to_owned()
    } else {
        format!("src/{source_path}")
    }
}

fn format_path(path: &syn::Path) -> anyhow::Result<String> {
    let mut rendered = String::new();
    if path.leading_colon.is_some() {
        rendered.push_str("::");
    }

    for (index, segment) in path.segments.iter().enumerate() {
        if !matches!(segment.arguments, PathArguments::None) {
            bail!("interactive example desugaring path may not contain generic arguments");
        }
        if index > 0 {
            rendered.push_str("::");
        }
        rendered.push_str(&segment.ident.to_string());
    }

    Ok(rendered)
}

fn interactive_example_body(
    invocation: &str,
    source_path: &str,
    line_no: usize,
) -> anyhow::Result<String> {
    let mut lines = invocation.lines();
    let first_line = lines
        .next()
        .context("expected `interactive_example!` macro invocation")?;
    let body_indent = first_line.bytes().take_while(|&byte| byte == b' ').count() + 4;
    let mut body_lines = lines.collect::<Vec<_>>();
    let Some(_closing_brace) = body_lines.pop() else {
        bail!("unterminated `interactive_example!` macro invocation at {source_path}:{line_no}");
    };

    let step_index = body_lines
        .iter()
        .position(|line| !line.trim().is_empty())
        .with_context(|| {
            format!("empty `interactive_example!` macro at {source_path}:{line_no}")
        })?;
    let mut sample = dedent_lines(&body_lines[step_index + 1..], body_indent);
    while sample.starts_with('\n') {
        sample.remove(0);
    }
    let sample = trim_one_final_newline(&sample).to_owned();
    if sample.trim().is_empty() {
        bail!("empty interactive example body at {source_path}:{line_no}");
    }
    Ok(sample)
}

fn dedent_lines(lines: &[&str], indent: usize) -> String {
    let prefix = " ".repeat(indent);
    lines
        .iter()
        .map(|line| line.strip_prefix(&prefix).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_interactive_example(example: &InteractiveExample) -> String {
    format!(
        r#"<div class="interactive-desugar" data-desugar-example="{id}">
<div class="interactive-desugar__panes">
<label class="interactive-desugar__pane">
<span class="interactive-desugar__label">Input (interactive)</span>
<textarea class="interactive-desugar__editor" spellcheck="false">{sample}</textarea>
</label>
<div class="interactive-desugar__pane">
<span class="interactive-desugar__label">Desugared (<code>{step}</code>)</span>
<pre class="interactive-desugar__output"><code></code></pre>
</div>
</div>
</div>"#,
        id = html_escape(&example.id),
        step = html_escape(&example.step),
        sample = html_escape(&example.sample),
    )
}

fn trim_one_final_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or_else(|| input.strip_suffix('\n'))
        .unwrap_or(input)
}

fn is_hidden_code_line(line: &str) -> bool {
    line.trim_end().ends_with("//#")
}

fn visible_lines_in_content(content: &str) -> BTreeSet<usize> {
    content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            (markdown_comment(line).is_none() && !is_hidden_code_line(line)).then_some(index + 1)
        })
        .collect()
}

pub fn rustdoc_link_map(
    context: &Value,
    chapters: &[ChapterInfo],
) -> BTreeMap<String, BTreeMap<usize, Vec<LinkRange>>> {
    if chapters.is_empty() {
        return BTreeMap::new();
    }

    let Ok(repo_root) = repo_root(context) else {
        warn_no_rustdoc_links("could not find repository root from mdBook context");
        return BTreeMap::new();
    };
    if let Err(err) = ensure_rustdoc_links(&repo_root) {
        warn_no_rustdoc_links(&format!("{err:#}"));
        return BTreeMap::new();
    }

    let visible_sources = chapters
        .iter()
        .flat_map(|chapter| {
            chapter.visible_lines.iter().map(|line| {
                (
                    (chapter.source_path.clone(), *line),
                    chapter.html_path.clone(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let source_to_html = chapters
        .iter()
        .map(|chapter| (chapter.source_path.clone(), chapter.html_path.clone()))
        .collect::<BTreeMap<_, _>>();
    let rustdoc_source_root = repo_root.join("target/doc/src").join(RUSTDOC_SOURCE_CRATE);

    chapters
        .iter()
        .map(|chapter| {
            (
                chapter.source_path.clone(),
                rustdoc_links_for_chapter(
                    chapter,
                    &rustdoc_source_root,
                    &visible_sources,
                    &source_to_html,
                ),
            )
        })
        .collect()
}

fn repo_root(context: &Value) -> anyhow::Result<PathBuf> {
    if let Some(root) = context.get("root").and_then(Value::as_str) {
        let root = PathBuf::from(root);
        return Ok(root.parent().unwrap_or(&root).to_path_buf());
    }

    let current_dir = env::current_dir().context("could not read current directory")?;
    Ok(current_dir.parent().unwrap_or(&current_dir).to_path_buf())
}

fn ensure_rustdoc_links(repo_root: &Path) -> anyhow::Result<()> {
    let rustdocflags = env::var("RUSTDOCFLAGS").unwrap_or_default();
    let rustdocflags = format!(
        "{} -Z unstable-options --generate-link-to-definition",
        rustdocflags.trim()
    );

    let output = Command::new("cargo")
        .args(["doc", "--no-deps"])
        .current_dir(repo_root)
        .env("RUSTDOCFLAGS", rustdocflags.trim())
        .output()
        .context("failed to run `cargo doc --no-deps`")?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "`cargo doc --no-deps` failed while generating rustdoc link-to-definition metadata:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn warn_no_rustdoc_links(reason: &str) {
    eprintln!(
        "warning: could not generate rustdoc link-to-definition metadata; \
         literate Rust code will be rendered without go-to-definition links\n{reason}"
    );
}

fn rustdoc_links_for_chapter(
    chapter: &ChapterInfo,
    rustdoc_source_root: &Path,
    visible_sources: &BTreeMap<(String, usize), String>,
    source_to_html: &BTreeMap<String, String>,
) -> BTreeMap<usize, Vec<LinkRange>> {
    let html_file = rustdoc_source_root.join(format!("{}.html", chapter.source_path));
    let Ok(content) = fs::read_to_string(&html_file) else {
        return BTreeMap::new();
    };

    let mut links_by_line: BTreeMap<usize, Vec<LinkRange>> = BTreeMap::new();
    for raw_line in content.lines() {
        let Some((line_no, line_html)) = split_rustdoc_line(raw_line) else {
            continue;
        };
        if !chapter.visible_lines.contains(&line_no) {
            continue;
        }

        let (source_text, anchors) = parse_rustdoc_line(line_html);
        for anchor in anchors {
            if !is_identifier(&anchor.text) {
                continue;
            }
            let Some((target_source, target_line)) =
                resolve_rustdoc_href(&chapter.source_path, &anchor.href)
            else {
                continue;
            };
            if !visible_sources.contains_key(&(target_source.clone(), target_line)) {
                continue;
            }
            if source_text.get(anchor.start..anchor.end) != Some(anchor.text.as_str()) {
                continue;
            }

            let Some(target_html) = source_to_html.get(&target_source) else {
                continue;
            };
            let target_id = line_anchor_id(&target_source, target_line);
            links_by_line.entry(line_no).or_default().push(LinkRange {
                start: anchor.start,
                end: anchor.end,
                text: anchor.text,
                href: relative_href(&chapter.html_path, target_html, &target_id),
            });
        }
    }

    links_by_line
}

fn split_rustdoc_line(line: &str) -> Option<(usize, &str)> {
    let rest = line.strip_prefix("<a href=#")?;
    let digits_len = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits_len == 0 {
        return None;
    }

    let line_no = rest[..digits_len].parse().ok()?;
    let expected = format!(" id={line_no} data-nosnippet>{line_no}</a>");
    let rest = rest[digits_len..].strip_prefix(&expected)?;
    Some((line_no, rest))
}

fn parse_rustdoc_line(line_html: &str) -> (String, Vec<RustdocAnchor>) {
    let mut source_text = String::new();
    let mut active_anchors = Vec::<ActiveAnchor>::new();
    let mut anchors = Vec::<RustdocAnchor>::new();
    let mut remaining = line_html;

    while let Some(tag_start) = remaining.find('<') {
        push_decoded_text(
            &remaining[..tag_start],
            &mut source_text,
            &mut active_anchors,
        );
        remaining = &remaining[tag_start + 1..];

        let Some(tag_end) = remaining.find('>') else {
            push_decoded_text("<", &mut source_text, &mut active_anchors);
            push_decoded_text(remaining, &mut source_text, &mut active_anchors);
            return (source_text, anchors);
        };

        let tag = &remaining[..tag_end];
        handle_tag(tag, source_text.len(), &mut active_anchors, &mut anchors);
        remaining = &remaining[tag_end + 1..];
    }

    push_decoded_text(remaining, &mut source_text, &mut active_anchors);
    (source_text, anchors)
}

fn handle_tag(
    tag: &str,
    offset: usize,
    active_anchors: &mut Vec<ActiveAnchor>,
    anchors: &mut Vec<RustdocAnchor>,
) {
    let tag = tag.trim();
    if tag
        .strip_prefix('/')
        .is_some_and(|tag| tag.starts_with('a'))
    {
        if let Some(anchor) = active_anchors.pop() {
            anchors.push(RustdocAnchor {
                href: anchor.href,
                text: anchor.text,
                start: anchor.start,
                end: offset,
            });
        }
        return;
    }

    if tag_name(tag) == Some("a")
        && let Some(href) = attr_value(tag, "href")
    {
        active_anchors.push(ActiveAnchor {
            href,
            start: offset,
            text: String::new(),
        });
    }
}

fn tag_name(tag: &str) -> Option<&str> {
    tag.split([' ', '\t', '\n', '\r', '/']).next()
}

fn attr_value(tag: &str, attr: &str) -> Option<String> {
    let marker = format!("{attr}=");
    let start = tag.find(&marker)? + marker.len();
    let rest = &tag[start..];
    if let Some(rest) = rest.strip_prefix('"') {
        let end = rest.find('"')?;
        Some(rest[..end].to_owned())
    } else if let Some(rest) = rest.strip_prefix('\'') {
        let end = rest.find('\'')?;
        Some(rest[..end].to_owned())
    } else {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        Some(rest[..end].trim_end_matches('/').to_owned())
    }
}

fn push_decoded_text(encoded: &str, source_text: &mut String, active_anchors: &mut [ActiveAnchor]) {
    let decoded = html_unescape(encoded);
    source_text.push_str(&decoded);
    for anchor in active_anchors {
        anchor.text.push_str(&decoded);
    }
}

fn html_unescape(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(ampersand) = remaining.find('&') {
        output.push_str(&remaining[..ampersand]);
        remaining = &remaining[ampersand + 1..];

        let Some(semicolon) = remaining.find(';') else {
            output.push('&');
            output.push_str(remaining);
            return output;
        };

        let entity = &remaining[..semicolon];
        match entity {
            "amp" => output.push('&'),
            "lt" => output.push('<'),
            "gt" => output.push('>'),
            "quot" => output.push('"'),
            "#39" => output.push('\''),
            entity => {
                if let Some(ch) = decode_numeric_entity(entity) {
                    output.push(ch);
                } else {
                    output.push('&');
                    output.push_str(entity);
                    output.push(';');
                }
            }
        }
        remaining = &remaining[semicolon + 1..];
    }

    output.push_str(remaining);
    output
}

fn decode_numeric_entity(entity: &str) -> Option<char> {
    let value = if let Some(hex) = entity.strip_prefix("#x") {
        u32::from_str_radix(hex, 16).ok()?
    } else if let Some(decimal) = entity.strip_prefix('#') {
        decimal.parse().ok()?
    } else {
        return None;
    };
    char::from_u32(value)
}

fn resolve_rustdoc_href(current_source_path: &str, href: &str) -> Option<(String, usize)> {
    if href.starts_with("http://") || href.starts_with("https://") {
        return None;
    }

    let (path, fragment) = href.split_once('#').unwrap_or((href, ""));
    let line = leading_number(fragment)?;
    let source_path = if path.is_empty() {
        current_source_path.to_owned()
    } else {
        let current_dir = current_source_path
            .rsplit_once('/')
            .map_or("", |(dir, _)| dir);
        normalize_posix_path(&format!("{current_dir}/{path}"))
    };
    let source_path = source_path
        .strip_suffix(".html")
        .unwrap_or(&source_path)
        .to_owned();

    Some((source_path, line))
}

fn leading_number(input: &str) -> Option<usize> {
    let len = input.bytes().take_while(u8::is_ascii_digit).count();
    (len > 0).then(|| input[..len].parse().ok()).flatten()
}

fn normalize_posix_path(path: &str) -> String {
    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            component => components.push(component),
        }
    }
    components.join("/")
}

fn html_path_for_source(source_path: &str) -> String {
    let source_path = source_path.strip_prefix("book/").unwrap_or(source_path);
    if let Some(markdown_source) = source_path.strip_suffix(".rs") {
        format!("{markdown_source}.html")
    } else {
        format!("{source_path}.html")
    }
}

fn line_anchor_id(source_path: &str, line: usize) -> String {
    format!("literate-rust-{}-L{line}", slug_for_source(source_path))
}

fn slug_for_source(source_path: &str) -> String {
    source_path
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

fn relative_href(from_html_path: &str, to_html_path: &str, target_id: &str) -> String {
    if from_html_path == to_html_path {
        return format!("#{target_id}");
    }

    let from_dir = from_html_path.rsplit_once('/').map_or("", |(dir, _)| dir);
    format!(
        "{}#{target_id}",
        relative_posix_path(from_dir, to_html_path)
    )
}

fn relative_posix_path(from_dir: &str, to_path: &str) -> String {
    let from = from_dir
        .split('/')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    let to = to_path
        .split('/')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();

    let mut common = 0;
    while common < from.len() && common < to.len() && from[common] == to[common] {
        common += 1;
    }

    let mut relative = Vec::new();
    relative.extend(std::iter::repeat_n("..", from.len() - common));
    relative.extend(to[common..].iter().copied());
    if relative.is_empty() {
        ".".to_owned()
    } else {
        relative.join("/")
    }
}

fn is_identifier(input: &str) -> bool {
    let mut chars = input.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn serialize_links(links_by_line: &BTreeMap<usize, Vec<LinkRange>>) -> String {
    let value = Value::Object(
        links_by_line
            .iter()
            .map(|(line, links)| {
                (
                    line.to_string(),
                    Value::Array(
                        links
                            .iter()
                            .map(|link| {
                                json!({
                                    "start": link.start,
                                    "end": link.end,
                                    "text": link.text,
                                    "href": link.href,
                                })
                            })
                            .collect(),
                    ),
                )
            })
            .collect(),
    );

    serde_json::to_string(&value)
        .expect("serializing link metadata should not fail")
        .replace('<', "\\u003c")
        .replace('&', "\\u0026")
}

fn html_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_literate_rust_markdown() {
        let rendered = render_chapter(
            "//@ # Title\nuse crate::x; //#\nfn f() {}\n//@ text\n",
            "book/chapter.md.rs",
            None,
        )
        .unwrap();

        assert!(rendered.contains("# Title\n"));
        assert!(rendered.contains("```rust,noplayground\nfn f() {}\n```"));
        assert!(rendered.contains("\ntext\n"));
        assert!(!rendered.contains("use crate::x"));
    }

    #[test]
    fn renders_interactive_example_macro() {
        let rendered = render_chapter(
            r#"fn before() {}
interactive_example! {
    make_place_coercions_explicit,
    fn main() {
        let x: bool = true;
        print(x);
    }
}
fn after() {}
"#,
            "book/pipeline/explicit-value-place.md.rs",
            None,
        )
        .unwrap();

        assert!(rendered.contains("fn before() {}"));
        assert!(rendered.contains("fn after() {}"));
        assert!(rendered.contains("<div class=\"interactive-desugar\""));
        assert!(
            rendered.contains(
                "data-desugar-example=\"src/book/pipeline/explicit-value-place.md.rs:2\""
            )
        );
        assert!(rendered.contains("<code>make_place_coercions_explicit</code>"));
        assert!(rendered.contains("fn main()"));
        assert!(!rendered.contains("interactive_example!"));
    }

    #[test]
    fn parses_rustdoc_link_ranges() {
        let (source_text, anchors) = parse_rustdoc_line(
            r##"</span><span class="kw">fn </span><a href="#20-22">target</a>() { <a class="x" href="../other.md.rs.html#4">call</a>(); }"##,
        );

        assert_eq!(source_text, "fn target() { call(); }");
        assert_eq!(anchors.len(), 2);
        assert_eq!(anchors[0].text, "target");
        assert_eq!(anchors[0].start, 3);
        assert_eq!(anchors[1].text, "call");
        assert_eq!(anchors[1].start, 14);
    }

    #[test]
    fn resolves_relative_rustdoc_href() {
        assert_eq!(
            resolve_rustdoc_href(
                "book/pipeline/current.md.rs",
                "../language/other.md.rs.html#42-45"
            ),
            Some(("book/language/other.md.rs".to_owned(), 42)),
        );
    }

    #[test]
    fn builds_relative_book_href() {
        assert_eq!(
            relative_href(
                "pipeline/current.md.html",
                "language/other.md.html",
                "target",
            ),
            "../language/other.md.html#target",
        );
    }
}
