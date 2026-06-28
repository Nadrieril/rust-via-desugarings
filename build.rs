use std::{env, fs, path::PathBuf};
use walkdir::WalkDir;

const LANGUAGE_DIR: &str = "src/book/language";

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let grammar_path = out_dir.join("parser.rustylr");
    fs::write(&grammar_path, collect_grammar()).unwrap();

    let parser_path = out_dir.join("parser.rs");
    rusty_lr::build::Builder::new()
        .file(grammar_path.to_str().unwrap())
        .glr(true)
        .note_conflicts(false)
        .note_conflicts_resolving(false)
        .note_optimization(false)
        .build(parser_path.to_str().unwrap());
}

fn collect_grammar() -> String {
    println!("cargo:rerun-if-changed={LANGUAGE_DIR}");

    let mut declarations = String::new();
    let mut productions = String::new();
    for entry in WalkDir::new(LANGUAGE_DIR).sort_by_file_name() {
        let path = entry.unwrap().into_path();
        if !(path.is_file() && path.to_string_lossy().ends_with(".md.rs")) {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(&path).unwrap();
        let mut within_rustylr_fences = None;
        for markdown in content.lines().filter_map(|line| line.strip_prefix("//@")) {
            let markdown = markdown.strip_prefix(' ').unwrap_or(markdown);
            let markdown = markdown.trim();

            if within_rustylr_fences.is_none() && markdown.starts_with("```rustylr") {
                within_rustylr_fences = Some(if markdown.contains("declarations") {
                    &mut declarations
                } else {
                    &mut productions
                });
                continue;
            } else if within_rustylr_fences.is_some() && markdown == "```" {
                within_rustylr_fences = None;
            }

            if let Some(output) = within_rustylr_fences.as_deref_mut() {
                output.push_str(markdown);
                output.push('\n');
            }
        }
    }

    format!("use crate::language::*;\n\n%%\n\n{declarations}\n{productions}")
}
