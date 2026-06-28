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

    let mut grammar = grammar::Grammar::default();
    let mut lexer = grammar::rustylr::LexerSpec::default();
    for entry in WalkDir::new(LANGUAGE_DIR).sort_by_file_name() {
        let path = entry.unwrap().into_path();
        if !(path.is_file() && path.to_string_lossy().ends_with(".md.rs")) {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(&path).unwrap();
        let markdown = literate_markdown(&content);
        let relative_path = path.strip_prefix("src/book").unwrap().to_owned();
        grammar::rustylr::parse_lexer_blocks(&markdown, &mut lexer, &relative_path).unwrap();
        grammar::parse_grammar_blocks(&markdown, &mut grammar, "syntax", relative_path).unwrap();
    }

    let declarations = lexer.rustylr_declarations().unwrap();
    let productions = grammar::rustylr::render_rustylr(&grammar, &lexer).unwrap();
    format!("use crate::language::*;\n\n%%\n\n{declarations}\n{productions}")
}

fn literate_markdown(content: &str) -> String {
    let mut markdown = String::new();
    for line in content.lines().filter_map(|line| line.strip_prefix("//@")) {
        let line = line.strip_prefix(' ').unwrap_or(line);
        markdown.push_str(line);
        markdown.push('\n');
    }
    markdown
}
