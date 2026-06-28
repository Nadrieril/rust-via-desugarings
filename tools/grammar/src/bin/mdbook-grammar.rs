use grammar::{Grammar, parse_grammar_blocks};
use serde_json::Value;
use std::io::{self, Read};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("supports") {
        return Ok(());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let mut values: Vec<Value> = serde_json::from_str(&input)?;
    let mut book = values.pop().expect("mdBook passes [context, book]");

    let mut chapters = Vec::new();
    collect_chapters(&book["sections"], &mut chapters);

    let mut grammar = Grammar::default();
    for (content, path) in &chapters {
        parse_grammar_blocks(content, &mut grammar, "syntax", path.clone())?;
    }

    render_sections(&mut book["sections"], &grammar);
    serde_json::to_writer(io::stdout(), &book)?;
    Ok(())
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
            let path = chapter_path(chapter);
            chapters.push((content.to_string(), path));
            collect_chapters(&chapter["sub_items"], chapters);
        } else if let Some(part) = section.get("PartTitle") {
            collect_chapters(&part["sub_items"], chapters);
        }
    }
}

fn render_sections(sections: &mut Value, grammar: &Grammar) {
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
            render_sections(&mut chapter["sub_items"], grammar);
        } else if let Some(part) = section.get_mut("PartTitle") {
            render_sections(&mut part["sub_items"], grammar);
        }
    }
}

fn chapter_path(chapter: &Value) -> PathBuf {
    chapter
        .get("path")
        .or_else(|| chapter.get("source_path"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("chapter.md"))
}
