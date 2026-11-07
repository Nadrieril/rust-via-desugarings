//! The UI test harness. See `tests/README.md` for how the tests and their directives work.

use libtest_mimic::{Failed, Trial};
use rust_via_desugarings::{ProgramRejection, RejectionKind, Stage};
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
    run: bool,
    expect_rejection: Option<ExpectedRejection>,
    known_bug: Option<String>,
}

/// The rejection described by an `expect-rejection` directive.
///
/// At least one of the fields is present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExpectedRejection {
    kind: Option<RejectionKind>,
    stage: Option<Stage>,
}

impl ExpectedRejection {
    /// Parses a directive value of the form `<Kind>`, `<Kind> @ <Stage>`, or `@ <Stage>`.
    fn parse(value: &str) -> Result<Self, String> {
        let (kind, stage) = match value.split_once('@') {
            Some((kind, stage)) => (kind.trim(), Some(stage.trim())),
            None => (value, None),
        };
        let kind = match kind {
            "" if stage.is_some() => None,
            kind => Some(
                kind_from_name(kind).ok_or_else(|| format!("unknown rejection kind `{kind}`"))?,
            ),
        };
        let stage = stage
            .map(|stage| stage_from_name(stage).ok_or_else(|| format!("unknown stage `{stage}`")))
            .transpose()?;
        Ok(Self { kind, stage })
    }

    fn matches(&self, rejection: &ProgramRejection) -> bool {
        self.kind.is_none_or(|kind| kind == rejection.kind)
            && self.stage.is_none_or(|stage| stage == rejection.stage)
    }
}

impl std::fmt::Display for ExpectedRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.kind, self.stage) {
            (Some(kind), Some(stage)) => write!(f, "{kind:?} @ {stage:?}"),
            (Some(kind), None) => write!(f, "{kind:?}"),
            (None, Some(stage)) => write!(f, "@ {stage:?}"),
            (None, None) => write!(f, "any"),
        }
    }
}

fn kind_from_name(name: &str) -> Option<RejectionKind> {
    Some(match name {
        "Invalid" => RejectionKind::Invalid,
        "Unimplemented" => RejectionKind::Unimplemented,
        "InternalError" => RejectionKind::InternalError,
        "RuntimeError" => RejectionKind::RuntimeError,
        "Unclassified" => RejectionKind::Unclassified,
        _ => return None,
    })
}

fn stage_from_name(name: &str) -> Option<Stage> {
    Some(match name {
        "Parse" => Stage::Parse,
        "Desugaring" => Stage::Desugaring,
        "FormalityTranslation" => Stage::FormalityTranslation,
        "FormalityCheck" => Stage::FormalityCheck,
        "MiniRustTranslation" => Stage::MiniRustTranslation,
        "MiniRustExecution" => Stage::MiniRustExecution,
        _ => return None,
    })
}

/// Describes an actual rejection in the same form as an `expect-rejection` directive.
fn describe(rejection: &ProgramRejection) -> String {
    format!("{:?} @ {:?}", rejection.kind, rejection.stage)
}

fn parse_directives(input: &str) -> Result<Directives, String> {
    let mut directives = Directives {
        run: false,
        expect_rejection: None,
        known_bug: None,
    };

    for (index, line) in input.lines().enumerate() {
        let Some(directive) = line.trim_start().strip_prefix("//@") else {
            continue;
        };
        apply_directive(&mut directives, directive.trim())
            .map_err(|message| format!("line {}: {message}", index + 1))?;
    }

    Ok(directives)
}

/// Records a single directive (the text following `//@`) in `directives`.
fn apply_directive(directives: &mut Directives, directive: &str) -> Result<(), String> {
    let (name, value) = match directive.split_once(':') {
        Some((name, value)) => (name.trim(), Some(value.trim())),
        None => (directive, None),
    };
    let duplicate = || format!("duplicate `{name}` directive");
    let no_value = || match value {
        None => Ok(()),
        Some(_) => Err(format!("`{name}` directive doesn't take a value")),
    };
    let required_value = || match value {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(format!(
            "`{name}` directive needs a value (`{name}: <value>`)"
        )),
    };
    match name {
        "run" => {
            no_value()?;
            if directives.run {
                Err(duplicate())?;
            }
            directives.run = true;
        }
        "expect-rejection" => {
            let value = required_value()?;
            if directives.expect_rejection.is_some() {
                Err(duplicate())?;
            }
            let expected = ExpectedRejection::parse(value)
                .map_err(|message| format!("`expect-rejection` directive: {message}"))?;
            directives.expect_rejection = Some(expected);
        }
        "known-bug" => {
            let value = required_value()?;
            if directives.known_bug.is_some() {
                Err(duplicate())?;
            }
            directives.known_bug = Some(value.to_owned());
        }
        _ => Err(format!("unknown directive: `{directive}`"))?,
    }
    Ok(())
}

fn run_case(input_path: &Path) -> Result<(), Failed> {
    let input = fs::read_to_string(input_path)
        .map_err(|error| format!("failed to read input file: {error}"))?;
    let directives = parse_directives(&input)?;
    let desugared_path = input_path.with_extension(DESUGARED_SUFFIX);
    let stdout_path = input_path.with_extension("out");
    let stderr_path = input_path.with_extension("stderr");

    let result = rust_via_desugarings::parse(&input);
    if let Ok(program) = &result {
        // Roundtrip the printer.
        let roundtrip = rust_via_desugarings::print_program(program);
        let reparsed = rust_via_desugarings::parse(&roundtrip)
            .map_err(|error| format!("failed to re-parse printed source:\n{error}"))?;
        if &reparsed != program {
            Err(format!(
                "printed source did not round-trip:\nprinted source:\n{roundtrip}\noriginal AST:\n{program:#?}\nreparsed AST:\n{reparsed:#?}"
            ))?
        }
    }

    let result = result.and_then(rust_via_desugarings::desugar_program);

    let _ = fs::remove_file(&desugared_path);
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);
    let outcome = 'outcome: {
        let program = match result {
            Ok(program) => program,
            Err(error) => break 'outcome Err(error),
        };
        write_output(
            &desugared_path,
            rust_via_desugarings::print_program(&program),
        )?;
        if directives.run {
            let stdout = match rust_via_desugarings::check_and_run(&program) {
                Ok(stdout) => stdout,
                Err(error) => break 'outcome Err(error),
            };
            write_output(&stdout_path, stdout)?;
        }
        Ok(())
    };
    if let Err(error) = &outcome {
        write_output(&stderr_path, format!("{error}\n"))?;
    }

    let mismatch = match (&outcome, directives.expect_rejection) {
        (Ok(()), None) => None,
        (Err(error), Some(expected)) if expected.matches(error) => None,
        (Err(error), None) => Some(format!(
            "expected success, got {} rejection:\n{error}",
            describe(error)
        )),
        (Ok(()), Some(expected)) => Some(format!(
            "expected {expected} rejection, but processing succeeded"
        )),
        (Err(error), Some(expected)) => Some(format!(
            "expected {expected} rejection, got {} rejection:\n{error}",
            describe(error)
        )),
    };
    match mismatch {
        None => Ok(()),
        Some(mut message) => {
            if let Some(known_bug) = &directives.known_bug {
                message.push_str(&format!(
                    "\n\nThis test is marked `known-bug: {known_bug}`. \
                    If the bug has been fixed, update the directives and remove `known-bug`."
                ));
            }
            Err(message.into())
        }
    }
}

fn write_output(path: &Path, output: impl AsRef<str>) -> Result<(), String> {
    fs::write(path, output.as_ref())
        .map_err(|error| format!("failed to write output file {}: {error}", path.display()))
}
