//! Reference-style rendering for grammar blocks.

use crate::{GRAMMAR_RE, Grammar};
use anyhow::Result;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

mod render_markdown;
mod render_railroad;

static NAMES_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(?:@root\s+)?([A-Z][A-Za-z0-9_]*)\s*:").unwrap());

#[derive(Debug)]
pub struct RenderCtx {
    md_link_map: HashMap<String, String>,
    rr_link_map: HashMap<String, String>,
    for_summary: bool,
}

pub fn render_chapter(grammar: &Grammar, content: &str, chapter_path: &Path) -> String {
    let link_map = make_relative_link_map(grammar, chapter_path);
    GRAMMAR_RE
        .replace_all(content, |cap: &Captures<'_>| {
            let names: Vec<_> = NAMES_RE
                .captures_iter(cap.get(2).unwrap().as_str())
                .map(|cap| cap.get(1).unwrap().as_str())
                .collect();
            render_names(grammar, &names, &link_map)
                .unwrap_or_else(|err| format!("<!-- failed to render grammar: {err} -->"))
        })
        .to_string()
}

fn make_relative_link_map(grammar: &Grammar, chapter_path: &Path) -> HashMap<String, String> {
    let current_path = chapter_path.parent().unwrap_or_else(|| Path::new(""));
    grammar
        .productions
        .values()
        .map(|p| {
            let relative = pathdiff::diff_paths(&p.path, current_path)
                .unwrap_or_else(|| PathBuf::from(&p.path));
            let relative = relative.display().to_string().replace('\\', "/");
            (p.name.clone(), relative)
        })
        .collect()
}

fn render_names(
    grammar: &Grammar,
    names: &[&str],
    link_map: &HashMap<String, String>,
) -> Result<String> {
    let mut output = String::new();
    output.push_str("<div class=\"grammar-container\">\n\n");
    output.push_str("<div class=\"grammar-heading\"><strong><sup>Syntax</sup></strong></div>\n");

    let update_link_map = |get_id: fn(&str, bool) -> String| -> HashMap<String, String> {
        link_map
            .iter()
            .map(|(name, path)| {
                let id = get_id(name, false);
                (name.clone(), format!("{path}#{id}"))
            })
            .collect()
    };

    let render_ctx = RenderCtx {
        md_link_map: update_link_map(render_markdown::markdown_id),
        rr_link_map: update_link_map(render_railroad::railroad_id),
        for_summary: false,
    };

    render_markdown::render_markdown(grammar, &render_ctx, names, &mut output)?;

    output.push_str(
        "\n\
         <button class=\"grammar-toggle-code\" type=\"button\" \
            title=\"Toggle grammar actions and bindings\" \
            onclick=\"toggle_grammar_code()\">\
            Hide code\
         </button>\n\
         <button class=\"grammar-toggle-railroad\" type=\"button\" \
            title=\"Toggle railroad display\" \
            onclick=\"toggle_railroad()\">\
            Show syntax diagrams\
         </button>\n\
         </div>\n\
         <div class=\"grammar-railroad grammar-hidden\">\n\
         \n",
    );

    render_railroad::render_railroad(grammar, &render_ctx, names, &mut output)?;
    output.push_str("</div>\n");
    Ok(output)
}

fn require_production<'a>(grammar: &'a Grammar, name: &str) -> Result<&'a crate::Production> {
    grammar
        .productions
        .get(name)
        .ok_or_else(|| anyhow::anyhow!("could not find grammar production named `{name}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_colon_productions_with_actions() {
        let source = r#"
```grammar
Rule:
    | IDENTIFIER => Identifier::Named(IDENTIFIER),
    | `&` IDENTIFIER => Identifier::Borrowed { name: IDENTIFIER },
```
"#;
        let mut grammar = Grammar::default();
        crate::parse_grammar_blocks(source, &mut grammar, "syntax", "chapter.md").unwrap();

        let rendered = render_chapter(&grammar, source, Path::new("chapter.md"));

        assert!(rendered.contains("</span>:"));
        assert!(rendered.contains("=&gt; <code class=\"grammar-action-code\">Identifier::Named"));
        assert!(rendered.contains("Identifier::Borrowed { name: IDENTIFIER }"));
        assert!(rendered.contains("class=\"grammar-toggle-code\""));
        assert!(!rendered.contains("</span> -&gt;"));
    }

    #[test]
    fn renders_bindings_and_newline_actions_as_hideable_code() {
        let source = r#"
```grammar
Rule:
    value=IDENTIFIER
    => Identifier::Named(value)
```
"#;
        let mut grammar = Grammar::default();
        crate::parse_grammar_blocks(source, &mut grammar, "syntax", "chapter.md").unwrap();

        let rendered = render_chapter(&grammar, source, Path::new("chapter.md"));

        assert!(rendered.contains("<span class=\"grammar-binding\">value=</span>"));
        assert!(
            rendered.contains("<span class=\"grammar-action\"><br>\n&nbsp;&nbsp;&nbsp;&nbsp;=&gt;")
        );
    }
}
