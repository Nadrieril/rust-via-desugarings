use std::{env, fs, path::PathBuf};
use walkdir::WalkDir;

const LANGUAGE_DIR: &str = "src/book/language";

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let lalrpop = collect_grammar();
    let grammar_path = out_dir.join("parser.lalrpop");
    fs::write(&grammar_path, lalrpop).unwrap();

    lalrpop::Configuration::new()
        .set_out_dir(&out_dir)
        .process_file(&grammar_path)
        .unwrap();
}

fn collect_grammar() -> String {
    println!("cargo:rerun-if-changed={LANGUAGE_DIR}");

    let mut lalrpop_body = format!("grammar;\n\nuse crate::language::*;\n\n");
    for entry in WalkDir::new(LANGUAGE_DIR).sort_by_file_name() {
        let path = entry.unwrap().into_path();
        if !(path.is_file() && path.to_string_lossy().ends_with(".md.rs")) {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(&path).unwrap();
        let mut within_larlpop_fences = false;
        for markdown in content.lines().filter_map(|line| line.strip_prefix("//@")) {
            let markdown = markdown.strip_prefix(' ').unwrap_or(markdown);
            let markdown = markdown.trim();

            if !within_larlpop_fences && markdown.starts_with("```lalrpop") {
                within_larlpop_fences = true;
                continue;
            } else if within_larlpop_fences && markdown == "```" {
                within_larlpop_fences = false;
            }

            if within_larlpop_fences {
                lalrpop_body.push_str(markdown);
                lalrpop_body.push('\n');
            }
        }
    }
    lalrpop_body
}
