use libtest_mimic::{Failed, Trial};
use std::{error::Error, fs, path::Path};
use walkdir::WalkDir;

const TESTS_DIR: &str = "tests/ui";
const DESUGARED_SUFFIX: &str = "desugared.rs";

fn main() -> Result<(), Box<dyn Error>> {
    let mut tests = Vec::new();
    for entry in WalkDir::new(TESTS_DIR).sort_by_file_name() {
        let path = entry?.into_path();
        let is_input_file = path.extension().is_some_and(|extension| extension == "rs")
            && path
                .file_name()
                .is_some_and(|name| !name.to_string_lossy().ends_with(DESUGARED_SUFFIX));
        if is_input_file {
            let name = path
                .strip_prefix(TESTS_DIR)
                .expect("ui test path should be under the tests/ui directory")
                .display()
                .to_string();
            tests.push(Trial::test(name, move || run_case(&path)));
        }
    }

    let args = libtest_mimic::Arguments::from_args();
    libtest_mimic::run(&args, tests).exit();
}

struct Directives {
    known_failure: bool,
    run: bool,
}

fn parse_directives(input: &str) -> Directives {
    let mut directives = Directives {
        known_failure: false,
        run: false,
    };

    for line in input.lines() {
        let Some(directive) = line.strip_prefix("//@") else {
            break;
        };
        match directive.trim() {
            "known-failure" => directives.known_failure = true,
            "run" => directives.run = true,
            _ => {}
        }
    }

    directives
}

fn run_case(input_path: &Path) -> Result<(), Failed> {
    let input = fs::read_to_string(input_path)
        .map_err(|error| format!("failed to read input file: {error}"))?;
    let directives = parse_directives(&input);
    let desugared_path = input_path.with_extension(DESUGARED_SUFFIX);
    let stdout_path = input_path.with_extension("out");
    let stderr_path = input_path.with_extension("stderr");

    let result = rust_via_desugarings::parser::parse_program(&input);
    if let Ok(program) = &result {
        // Roundtrip the printer.
        let roundtrip = rust_via_desugarings::print_program(program);
        let reparsed = rust_via_desugarings::parser::parse_program(&roundtrip)
            .map_err(|error| format!("failed to re-parse printed source:\n{error}"))?;
        if &reparsed != program {
            Err(format!(
                "printed source did not round-trip:\nprinted source:\n{roundtrip}\noriginal AST:\n{program:#?}\nreparsed AST:\n{reparsed:#?}"
            ))?
        }
    }

    let result = result.and_then(rust_via_desugarings::desugar);

    let _ = fs::remove_file(&desugared_path);
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);
    match result {
        Ok(program) => {
            let output = rust_via_desugarings::print_program(&program);
            write_output(&desugared_path, output)?;
            if directives.run {
                match rust_via_desugarings::run_in_minirust(&program) {
                    Ok(stdout) => write_output(&stdout_path, stdout)?,
                    Err(error) => {
                        write_output(&stderr_path, format!("{error}\n"))?;
                        if !directives.known_failure {
                            Err(format!("expected run success, got error:\n{error}"))?
                        }
                        return Ok(());
                    }
                }
            }
            if directives.known_failure {
                Err("expected failure, but parsing and printing succeeded".to_owned())?
            }
        }
        Err(error) => {
            write_output(&stderr_path, format!("{error}\n"))?;
            if !directives.known_failure {
                Err(format!("expected success, got parse error:\n{error}"))?
            }
        }
    }
    Ok(())
}

fn write_output(path: &Path, output: impl AsRef<str>) -> Result<(), String> {
    fs::write(path, output.as_ref())
        .map_err(|error| format!("failed to write output file {}: {error}", path.display()))
}
