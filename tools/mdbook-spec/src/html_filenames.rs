//! Makes literate Rust (`.md.rs`) chapters render as `foo.html` rather than
//! `foo.md.html`.
//!
//! mdBook's HTML renderer derives a chapter's output filename (and the links
//! to it from the sidebar, navigation, search index and print page) from the
//! chapter's `path`, by replacing its extension with `.html`. We change the
//! `path` of `foo.md.rs` chapters to `foo.md`, leaving `source_path` alone so
//! that the "edit this page" link still points at the real source file.
//!
//! mdBook's HTML renderer also rewrites links to `foo.md.rs` as `foo.html`, so
//! with this change such links work without further help.
//!
//! This runs as a separate preprocessor which is only enabled for the HTML
//! renderer, after the backlinks preprocessor, so that other preprocessors and
//! renderers still see the real source paths.

use anyhow::{Context, bail};
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::{self, Read};

pub fn handle_preprocessing() -> anyhow::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut values: Vec<Value> = serde_json::from_str(&input)?;
    if values.len() != 2 {
        bail!("mdBook passes [context, book], got {} values", values.len());
    }

    let mut book = values.pop().context("mdBook input missing book")?;
    let mut paths = BTreeSet::new();
    collect_paths(&book["items"], &mut paths);
    rename_sections(&mut book["items"], &paths)?;
    serde_json::to_writer(io::stdout(), &book)?;
    Ok(())
}

fn collect_paths(sections: &Value, paths: &mut BTreeSet<String>) {
    let Some(sections) = sections.as_array() else {
        return;
    };

    for section in sections {
        if let Some(chapter) = section.get("Chapter") {
            if let Some(path) = chapter.get("path").and_then(Value::as_str) {
                paths.insert(path.to_owned());
            }
            collect_paths(&chapter["sub_items"], paths);
        } else if let Some(part) = section.get("PartTitle") {
            collect_paths(&part["sub_items"], paths);
        }
    }
}

fn rename_sections(sections: &mut Value, paths: &BTreeSet<String>) -> anyhow::Result<()> {
    let Some(sections) = sections.as_array_mut() else {
        return Ok(());
    };

    for section in sections {
        if let Some(chapter) = section.get_mut("Chapter") {
            if let Some(path) = chapter.get("path").and_then(Value::as_str)
                && let Some(new_path) = html_source_path(path)
            {
                if paths.contains(&new_path) {
                    bail!("{path} and {new_path} would both be rendered as the same HTML file");
                }
                chapter["path"] = Value::String(new_path);
            }
            rename_sections(&mut chapter["sub_items"], paths)?;
        } else if let Some(part) = section.get_mut("PartTitle") {
            rename_sections(&mut part["sub_items"], paths)?;
        }
    }

    Ok(())
}

/// Returns the path mdBook should treat a literate Rust chapter as having, or
/// `None` if the chapter isn't literate Rust.
fn html_source_path(path: &str) -> Option<String> {
    path.strip_suffix(".rs")
        .filter(|path| path.ends_with(".md"))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn chapter(path: &str, sub_items: Value) -> Value {
        json!({"Chapter": {"path": path, "source_path": path, "sub_items": sub_items}})
    }

    #[test]
    fn renames_literate_chapters() {
        let mut items = json!([
            chapter("intro.md", json!([])),
            {"PartTitle": {"sub_items": [
                chapter("language/foo.md.rs", json!([chapter("language/foo/bar.md.rs", json!([]))])),
            ]}},
        ]);
        let mut paths = BTreeSet::new();
        collect_paths(&items, &mut paths);
        rename_sections(&mut items, &paths).unwrap();

        assert_eq!(
            items,
            json!([
                chapter("intro.md", json!([])),
                {"PartTitle": {"sub_items": [
                    {"Chapter": {
                        "path": "language/foo.md",
                        "source_path": "language/foo.md.rs",
                        "sub_items": [{"Chapter": {
                            "path": "language/foo/bar.md",
                            "source_path": "language/foo/bar.md.rs",
                            "sub_items": [],
                        }}],
                    }},
                ]}},
            ]),
        );
    }

    #[test]
    fn leaves_plain_rust_paths_alone() {
        assert_eq!(html_source_path("foo.rs"), None);
        assert_eq!(html_source_path("foo.md"), None);
        assert_eq!(html_source_path("foo.md.rs"), Some("foo.md".to_owned()));
    }

    #[test]
    fn rejects_clashing_chapters() {
        let mut items = json!([
            chapter("foo.md", json!([])),
            chapter("foo.md.rs", json!([]))
        ]);
        let mut paths = BTreeSet::new();
        collect_paths(&items, &mut paths);
        assert!(rename_sections(&mut items, &paths).is_err());
    }
}
