use anyhow::{Context, bail};
use mdbook_markdown::pulldown_cmark::{CowStr, Event, Tag, TagEnd};
use mdbook_markdown::{MarkdownOptions, new_cmark_parser};
use pulldown_cmark_to_cmark::cmark;
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::Command;

mod literate_rust;

const REFERENCE_ROOT: &str = "https://doc.rust-lang.org/reference/";
const LITERATE_RUST_TITLE_PREFIX: &str = "▶️ ";

const RULE_PAGE_PREFIXES: &[(&str, &str)] = &[
    ("associated", "items/associated-items.html"),
    ("coerce", "type-coercions.html"),
    ("destructors", "destructors.html"),
    ("expr.call", "expressions/call-expr.html"),
    ("expr.deref", "expressions/operator-expr.html"),
    ("expr.if", "expressions/if-expr.html"),
    ("expr.method", "expressions/method-call-expr.html"),
    ("expr.operator", "expressions/operator-expr.html"),
    ("expr.paren", "expressions/grouped-expr.html"),
    ("expr.place-value", "expressions.html"),
    ("expr.struct", "expressions/struct-expr.html"),
    ("items.associated", "items/associated-items.html"),
    ("items.fn", "items/functions.html"),
    ("items.union", "items/unions.html"),
    ("macro.decl", "macros-by-example.html"),
    ("names", "names.html"),
    ("type.closure", "types/closure.html"),
    ("type.fn-item", "types/function-item.html"),
];

pub fn handle_preprocessing() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut values: Vec<Value> = serde_json::from_str(&input)?;
    if values.len() != 2 {
        bail!("mdBook passes [context, book], got {} values", values.len());
    }

    let mut book = values.pop().context("mdBook input missing book")?;
    let context = values.pop().context("mdBook input missing context")?;
    let literate_chapters = literate_rust::collect_chapters(&book["sections"]);
    ensure_interactive_wasm_is_current(&context)?;
    let rustdoc_links = literate_rust::rustdoc_link_map(&context, &literate_chapters);

    render_literate_sections(&mut book["sections"], &rustdoc_links)?;
    let grammar = collect_grammar(&book["sections"])?;
    render_grammar_sections(&mut book["sections"], &grammar);

    let mut missing_rules = Vec::new();
    render_reference_sections(&mut book["sections"], &mut missing_rules)?;
    missing_rules.sort();
    missing_rules.dedup();
    if !missing_rules.is_empty() {
        bail!(
            "missing Reference page mapping for rule id(s): {}",
            missing_rules.join(", ")
        );
    }
    serde_json::to_writer(io::stdout(), &book)?;
    Ok(())
}

fn ensure_interactive_wasm_is_current(context: &Value) -> anyhow::Result<()> {
    let repo_root = literate_rust::repo_root(context)?;
    let profile = if env::var_os("CI").is_some() {
        "release"
    } else {
        "debug"
    };

    eprintln!("mdbook-spec: ensuring interactive desugaring wasm assets ({profile})");
    let output = Command::new("just")
        .args(["build-interactive-wasm", profile])
        .current_dir(&repo_root)
        .output()
        .with_context(|| {
            format!(
                "failed to run `just build-interactive-wasm {profile}` from {}",
                repo_root.display()
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    bail!(
        "`just build-interactive-wasm {profile}` failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn render_literate_sections(
    sections: &mut Value,
    rustdoc_links: &BTreeMap<String, BTreeMap<usize, Vec<literate_rust::LinkRange>>>,
) -> anyhow::Result<()> {
    let Some(sections) = sections.as_array_mut() else {
        return Ok(());
    };

    for section in sections {
        if let Some(chapter) = section.get_mut("Chapter") {
            add_literate_rust_title_prefix(chapter);
            if let Some(content) = chapter.get("content").and_then(Value::as_str) {
                if let Some(source_path) = literate_rust::chapter_source_path(chapter)
                    && source_path.ends_with(".md.rs")
                {
                    chapter["content"] = Value::String(literate_rust::render_chapter(
                        content,
                        &source_path,
                        rustdoc_links.get(&source_path),
                    )?);
                }
            }
            render_literate_sections(&mut chapter["sub_items"], rustdoc_links)?;
        } else if let Some(part) = section.get_mut("PartTitle") {
            render_literate_sections(&mut part["sub_items"], rustdoc_links)?;
        }
    }

    Ok(())
}

fn collect_grammar(sections: &Value) -> anyhow::Result<grammar::Grammar> {
    let mut chapters = Vec::new();
    collect_chapters(sections, &mut chapters);

    let mut grammar = grammar::Grammar::default();
    for (content, path) in chapters {
        grammar::parse_grammar_blocks(&content, &mut grammar, "syntax", path)?;
    }
    Ok(grammar)
}

fn collect_chapters(sections: &Value, chapters: &mut Vec<(String, PathBuf)>) {
    let Some(sections) = sections.as_array() else {
        return;
    };

    for section in sections {
        if let Some(chapter) = section.get("Chapter") {
            let content = chapter
                .get("content")
                .and_then(Value::as_str)
                .unwrap_or_default();
            chapters.push((content.to_owned(), chapter_path(chapter)));
            collect_chapters(&chapter["sub_items"], chapters);
        } else if let Some(part) = section.get("PartTitle") {
            collect_chapters(&part["sub_items"], chapters);
        }
    }
}

fn render_grammar_sections(sections: &mut Value, grammar: &grammar::Grammar) {
    let Some(sections) = sections.as_array_mut() else {
        return;
    };

    for section in sections {
        if let Some(chapter) = section.get_mut("Chapter") {
            let path = chapter_path(chapter);
            if let Some(content) = chapter.get("content").and_then(Value::as_str) {
                chapter["content"] =
                    Value::String(grammar::render::render_chapter(grammar, content, &path));
            }
            render_grammar_sections(&mut chapter["sub_items"], grammar);
        } else if let Some(part) = section.get_mut("PartTitle") {
            render_grammar_sections(&mut part["sub_items"], grammar);
        }
    }
}

fn chapter_path(chapter: &Value) -> PathBuf {
    chapter
        .get("source_path")
        .or_else(|| chapter.get("path"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("chapter.md"))
}

fn render_reference_sections(
    sections: &mut Value,
    missing_rules: &mut Vec<String>,
) -> anyhow::Result<()> {
    let Some(sections) = sections.as_array_mut() else {
        return Ok(());
    };

    for section in sections {
        if let Some(chapter) = section.get_mut("Chapter") {
            if let Some(content) = chapter.get("content").and_then(Value::as_str) {
                chapter["content"] =
                    Value::String(render_reference_links_in_chapter(content, missing_rules)?);
            }
            render_reference_sections(&mut chapter["sub_items"], missing_rules)?;
        } else if let Some(part) = section.get_mut("PartTitle") {
            render_reference_sections(&mut part["sub_items"], missing_rules)?;
        }
    }

    Ok(())
}

fn add_literate_rust_title_prefix(chapter: &mut Value) {
    if !literate_rust::is_literate_chapter(chapter) {
        return;
    }

    let Some(name) = chapter.get("name").and_then(Value::as_str) else {
        return;
    };
    if name.starts_with(LITERATE_RUST_TITLE_PREFIX) {
        return;
    }
    let name = name.to_owned();

    chapter["name"] = Value::String(format!("{LITERATE_RUST_TITLE_PREFIX}{name}"));
}

pub fn render_reference_links(content: &str) -> anyhow::Result<String> {
    let mut missing_rules = Vec::new();
    let rendered = render_reference_links_in_chapter(content, &mut missing_rules)?;
    if !missing_rules.is_empty() {
        missing_rules.sort();
        missing_rules.dedup();
        bail!(
            "missing Reference page mapping for rule id(s): {}",
            missing_rules.join(", ")
        );
    }
    Ok(rendered)
}

fn render_reference_links_in_chapter(
    content: &str,
    missing_rules: &mut Vec<String>,
) -> anyhow::Result<String> {
    let parser = new_cmark_parser(content, &MarkdownOptions::default());
    let events = rewrite_reference_events(parser.map(Event::into_static), missing_rules);

    let mut rendered = String::with_capacity(content.len());
    cmark(events.into_iter(), &mut rendered)?;
    Ok(rendered)
}

fn rewrite_reference_events<I>(events: I, missing_rules: &mut Vec<String>) -> Vec<Event<'static>>
where
    I: IntoIterator<Item = Event<'static>>,
{
    let mut rewritten = Vec::new();
    let mut text = String::new();
    let mut in_code_block = false;
    let mut in_link = false;

    for event in events {
        match event {
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_text(&mut text, &mut rewritten, missing_rules);
                in_code_block = true;
                rewritten.push(Event::Start(Tag::CodeBlock(kind)));
            }
            Event::End(TagEnd::CodeBlock) => {
                rewritten.push(Event::End(TagEnd::CodeBlock));
                in_code_block = false;
            }
            Event::Start(tag @ Tag::Link { .. }) => {
                flush_text(&mut text, &mut rewritten, missing_rules);
                in_link = true;
                rewritten.push(Event::Start(tag));
            }
            Event::End(TagEnd::Link) => {
                rewritten.push(Event::End(TagEnd::Link));
                in_link = false;
            }
            Event::Text(event_text) if !in_code_block && !in_link => {
                text.push_str(&event_text);
            }
            event => {
                flush_text(&mut text, &mut rewritten, missing_rules);
                rewritten.push(event);
            }
        }
    }

    flush_text(&mut text, &mut rewritten, missing_rules);
    rewritten
}

fn flush_text(
    text: &mut String,
    events: &mut Vec<Event<'static>>,
    missing_rules: &mut Vec<String>,
) {
    if text.is_empty() {
        return;
    }

    events.extend(rewrite_reference_text(text, missing_rules));
    text.clear();
}

fn rewrite_reference_text(text: &str, missing_rules: &mut Vec<String>) -> Vec<Event<'static>> {
    let mut events = Vec::new();
    let mut plain = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            match parse_reference_marker(&mut chars) {
                ReferenceParse::Marker(rule_id) => {
                    flush_plain_text(&mut plain, &mut events);
                    match render_reference_link(&rule_id) {
                        Some(link) => events.push(Event::InlineHtml(CowStr::from(link))),
                        None => {
                            missing_rules.push(rule_id.clone());
                            plain.push_str("[ref:");
                            plain.push_str(&rule_id);
                            plain.push(']');
                        }
                    }
                }
                ReferenceParse::NotMarker(consumed) => {
                    plain.push(ch);
                    plain.push_str(&consumed);
                }
            }
        } else {
            plain.push(ch);
        }
    }

    flush_plain_text(&mut plain, &mut events);
    events
}

enum ReferenceParse {
    Marker(String),
    NotMarker(String),
}

fn parse_reference_marker<I>(chars: &mut std::iter::Peekable<I>) -> ReferenceParse
where
    I: Iterator<Item = char>,
{
    let mut consumed = String::new();

    for expected in ['r', 'e', 'f', ':'] {
        let Some(ch) = chars.next() else {
            return ReferenceParse::NotMarker(consumed);
        };
        consumed.push(ch);
        if ch != expected {
            return ReferenceParse::NotMarker(consumed);
        }
    }

    let mut rule_id = String::new();
    while let Some(ch) = chars.next() {
        consumed.push(ch);
        if ch == ']' {
            return if rule_id.is_empty() {
                ReferenceParse::NotMarker(consumed)
            } else {
                ReferenceParse::Marker(rule_id)
            };
        }

        if is_rule_id_char(ch) {
            rule_id.push(ch);
        } else {
            return ReferenceParse::NotMarker(consumed);
        }
    }

    ReferenceParse::NotMarker(consumed)
}

fn flush_plain_text(plain: &mut String, events: &mut Vec<Event<'static>>) {
    if !plain.is_empty() {
        events.push(Event::Text(CowStr::from(std::mem::take(plain))));
    }
}

fn is_rule_id_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')
}

fn render_reference_link(rule_id: &str) -> Option<String> {
    let page = reference_page(rule_id)?;
    let href = format!("{REFERENCE_ROOT}{page}#r-{rule_id}");
    let title = format!("Rust Reference: {rule_id}");
    let label = break_rule_id(rule_id);

    Some(format!(
        r#"<a class="reference-link" href="{}" title="{}"><span>&#91;{}&#93;</span></a>"#,
        html_escape(&href),
        html_escape(&title),
        label,
    ))
}

fn reference_page(rule_id: &str) -> Option<&'static str> {
    RULE_PAGE_PREFIXES
        .iter()
        .find_map(|(prefix, page)| rule_matches_prefix(rule_id, prefix).then_some(*page))
}

fn rule_matches_prefix(rule_id: &str, prefix: &str) -> bool {
    rule_id == prefix
        || rule_id
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn break_rule_id(rule_id: &str) -> String {
    let mut label = String::new();

    for (index, part) in rule_id.split('.').enumerate() {
        if index > 0 {
            label.push_str("<wbr>.");
        }
        label.push_str(&html_escape(part));
    }

    label
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
    fn renders_reference_link() {
        let rendered =
            render_reference_links("See [ref:associated.fn.method.self-pat-shorthands].").unwrap();

        assert_eq!(
            rendered,
            "See <a class=\"reference-link\" href=\"https://doc.rust-lang.org/reference/items/associated-items.html#r-associated.fn.method.self-pat-shorthands\" title=\"Rust Reference: associated.fn.method.self-pat-shorthands\"><span>&#91;associated<wbr>.fn<wbr>.method<wbr>.self-pat-shorthands&#93;</span></a>."
        );
    }

    #[test]
    fn skips_fenced_code_blocks() {
        let input = "```markdown\n[ref:items.fn.syntax]\n```\n[ref:items.fn.syntax]\n";
        let rendered = render_reference_links(input).unwrap();

        assert!(rendered.contains("markdown\n[ref:items.fn.syntax]\n"));
        assert_eq!(rendered.matches("reference-link").count(), 1);
    }

    #[test]
    fn skips_inline_code_spans() {
        let input = "`[ref:items.fn.syntax]` [ref:items.fn.syntax]";
        let rendered = render_reference_links(input).unwrap();

        assert!(rendered.starts_with("`[ref:items.fn.syntax]` "));
        assert!(rendered.ends_with("</a>"));
    }

    #[test]
    fn renders_after_multiline_inline_code_span() {
        let input = "A `self:\n&Self`. [ref:associated.fn.method.self-pat-shorthands]";
        let rendered = render_reference_links(input).unwrap();

        assert!(rendered.starts_with("A `self: &Self`. <a class="));
        assert_eq!(rendered.matches("reference-link").count(), 1);
        assert!(rendered.contains(
            "href=\"https://doc.rust-lang.org/reference/items/associated-items.html#r-associated.fn.method.self-pat-shorthands\""
        ));
    }

    #[test]
    fn reports_missing_rule_page_mapping() {
        let err = render_reference_links("[ref:unknown.rule]").unwrap_err();

        assert!(
            err.to_string()
                .contains("missing Reference page mapping for rule id(s): unknown.rule")
        );
    }
}
