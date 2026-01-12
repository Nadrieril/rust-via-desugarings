//! UI tests for the cargo-desugar driver. Each `<file>.rs` file in `tests/ui/` will be passed to
//! the driver. The resulting output is stored in `<file>.out`, and CI will ensure these stay
//! up-to-date.
//!
//! Files can start with special comments that affect the test behavior. Supported magic comments:
//! see [`HELP_STRING`].
use anyhow::{anyhow, bail};
use assert_cmd::prelude::OutputAssertExt;
use indoc::indoc as unindent;
use itertools::Itertools;
use libtest_mimic::Trial;
use regex::Regex;
use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};
use walkdir::WalkDir;

use util::compare_or_overwrite;
mod util;

static TESTS_DIR: &str = "tests/ui";

enum TestKind {
    Pass,
    KnownFailure,
    KnownPanic,
    Skip,
}

struct MagicComments {
    test_kind: TestKind,
    /// The options with which to run cargo-desugar.
    desugar_opts: Vec<String>,
    /// The options to pass to rustc.
    rustc_opts: Vec<String>,
    /// Whether we should store the test output in a file and check it.
    check_output: bool,
    /// A list of paths to files that must be compiled as dependencies for this test.
    auxiliary_crates: Vec<PathBuf>,
}

static HELP_STRING: &str = unindent!(
    "Options are:
    - `//@ known-failure`: a test that is expected to fail.
    - `//@ known-panic`: a test that is expected to panic.
    - `//@ skip`: skip the test.

    Other comments can be used to control the behavior of cargo-desugar:
    - `//@ desugar-args=<cargo-desugar cli options>`
    - `//@ desugar-arg=<single cargo-desugar cli option>`
    - `//@ rustc-args=<rustc cli options>`
    - `//@ no-check-output`: don't store the output in a file; useful if the output is unstable or
         differs between debug and release mode.
    - `//@ aux-crate=<file path>`: compile this file as a crate dependency.
    "
);

fn parse_magic_comments(input_path: &std::path::Path) -> anyhow::Result<MagicComments> {
    // Parse the magic comments.
    let mut comments = MagicComments {
        test_kind: TestKind::Pass,
        desugar_opts: Vec::new(),
        rustc_opts: Vec::new(),
        check_output: true,
        auxiliary_crates: Vec::new(),
    };
    for line in BufReader::new(File::open(input_path)?).lines() {
        let line = line?;
        let Some(line) = line.strip_prefix("//@") else {
            break;
        };
        let line = line.trim();
        if line == "known-panic" {
            comments.test_kind = TestKind::KnownPanic;
        } else if line == "known-failure" {
            comments.test_kind = TestKind::KnownFailure;
        } else if line == "skip" {
            comments.test_kind = TestKind::Skip;
        } else if line == "no-check-output" {
            comments.check_output = false;
        } else if let Some(desugar_opts) = line.strip_prefix("desugar-args=") {
            comments
                .desugar_opts
                .extend(desugar_opts.split_whitespace().map(|s| s.to_string()));
        } else if let Some(desugar_opt) = line.strip_prefix("desugar-arg=") {
            comments.desugar_opts.push(desugar_opt.to_string());
        } else if let Some(rustc_opts) = line.strip_prefix("rustc-args=") {
            comments
                .rustc_opts
                .extend(rustc_opts.split_whitespace().map(|s| s.to_string()));
        } else if let Some(crate_path) = line.strip_prefix("aux-crate=") {
            let crate_path: PathBuf = crate_path.into();
            let crate_path = input_path.parent().unwrap().join(crate_path);
            comments.auxiliary_crates.push(crate_path)
        } else {
            return Err(
                anyhow!("Unknown magic comment: `{line}`. {HELP_STRING}").context(format!(
                    "While processing file {}",
                    input_path.to_string_lossy()
                )),
            );
        }
    }
    Ok(comments)
}

struct Case {
    input_path: PathBuf,
    expected: PathBuf,
    desugared: PathBuf,
    magic_comments: MagicComments,
}

fn setup_test(input_path: PathBuf) -> anyhow::Result<Trial> {
    let name = input_path
        .to_str()
        .unwrap()
        .strip_prefix(TESTS_DIR)
        .unwrap()
        .strip_prefix("/")
        .unwrap()
        .to_owned();
    let expected = input_path.with_extension("out");
    let desugared = input_path.with_extension("desugared.rs");
    let magic_comments = parse_magic_comments(&input_path)?;
    let ignore = matches!(magic_comments.test_kind, TestKind::Skip);
    let case = Case {
        input_path,
        expected,
        desugared,
        magic_comments,
    };
    let trial = Trial::test(name, move || perform_test(&case).map_err(|err| err.into()))
        .with_ignored_flag(ignore);
    Ok(trial)
}

fn path_to_crate_name(path: &Path) -> Option<String> {
    Some(
        path.file_name()?
            .to_str()?
            .strip_suffix(".rs")?
            .replace(['-'], "_"),
    )
}

fn compile_desugared(
    test_case: &Case,
    deps: &[(String, PathBuf, String)],
    source_path: &Path,
) -> anyhow::Result<std::process::Output> {
    let mut cmd = Command::new("rustc");
    cmd.arg("--out-dir")
        .arg("target/ui-tests-build")
        .arg("-Zno-codegen")
        .arg("--crate-name=test_crate")
        .arg("--crate-type=rlib")
        .arg("--allow=unused");
    for (crate_name, _, rlib_path) in deps {
        cmd.arg(format!("--extern={crate_name}={rlib_path}"));
    }
    cmd.args(&test_case.magic_comments.rustc_opts);
    cmd.arg(source_path);
    cmd.output().map_err(|e| e.into())
}

fn perform_test(test_case: &Case) -> anyhow::Result<()> {
    // Dependencies
    // Vec of (crate name, path to crate.rs, path to libcrate.rlib).
    let deps: Vec<(String, PathBuf, String)> = test_case
        .magic_comments
        .auxiliary_crates
        .iter()
        .cloned()
        .map(|path| {
            let crate_name = path_to_crate_name(&path).unwrap();
            let rlib_file_name = format!("lib{crate_name}.rlib"); // yep it must start with "lib"
            let rlib_path = path.parent().unwrap().join(rlib_file_name);
            let rlib_path = rlib_path.to_str().unwrap().to_owned();
            (crate_name, path, rlib_path)
        })
        .collect();
    for (crate_name, rs_path, rlib_path) in deps.iter() {
        Command::new("rustc")
            .arg("--crate-type=rlib")
            .arg("-Zalways-encode-mir")
            .arg(format!("--crate-name={crate_name}"))
            .arg("-o")
            .arg(rlib_path)
            .arg(rs_path)
            .output()?
            .assert()
            .try_success()
            .map_err(|e| anyhow!(e.to_string()))?;
    }

    // Run cargo-desugar.
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cargo-desugar"));
    cmd.arg("rustc");

    // cargo-desugar args
    cmd.args(&test_case.magic_comments.desugar_opts);

    // Rustc args
    cmd.arg("--");
    cmd.arg(&test_case.input_path);
    cmd.arg("--crate-name=test_crate");
    cmd.arg("--crate-type=rlib");
    cmd.arg("--allow=unused"); // Removes noise
    for (crate_name, _, rlib_path) in deps.iter() {
        cmd.arg(format!("--extern={crate_name}={rlib_path}"));
    }
    cmd.args(&test_case.magic_comments.rustc_opts);

    let args = cmd
        .get_args()
        .map(OsStr::to_string_lossy)
        .collect::<Vec<_>>()
        .join(" ");
    let cmd_str = format!("cargo-desugar {args}");
    let output = cmd.output()?;
    let stderr = String::from_utf8(output.stderr.clone())?;
    let stdout = String::from_utf8(output.stdout.clone())?;

    // Hide thread id from the output.
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"thread 'rustc' \(\d+\) panicked").unwrap());
    let stderr = RE.replace_all(&stderr, "thread 'rustc' panicked");
    let stderr_owned = stderr.to_string();

    let mut test_output = match test_case.magic_comments.test_kind {
        TestKind::KnownPanic => {
            if output.status.code() != Some(101) {
                let status = if output.status.success() {
                    "succeeded"
                } else {
                    "errored"
                };
                bail!(
                    "Command: `{cmd_str}`\nCompilation was expected to panic but instead {status}:\n{stderr_owned}"
                );
            }
            stderr_owned.clone()
        }
        TestKind::KnownFailure | TestKind::Pass => {
            if !output.status.success()
                && !matches!(test_case.magic_comments.test_kind, TestKind::KnownFailure)
            {
                bail!("Command: `{cmd_str}`\nCompilation failed:\n{stderr_owned}")
            }
            stdout
        }
        TestKind::Skip => unreachable!(),
    };

    test_output = util::rustfmt_output(&test_output);
    compare_or_overwrite(&test_output, &test_case.desugared)?;

    let compile_output = if output.status.success() {
        Some(compile_desugared(test_case, &deps, &test_case.desugared)?)
    } else {
        None
    };

    match test_case.magic_comments.test_kind {
        TestKind::Pass => {
            if let Some(compile) = &compile_output {
                if !compile.status.success() {
                    let stderr = String::from_utf8_lossy(&compile.stderr);
                    bail!("Desugared code failed to compile:\n{stderr}");
                }
            } else {
                bail!("Desugaring failed unexpectedly:\n{stderr_owned}");
            }
        }
        TestKind::KnownFailure => {
            if output.status.success() {
                if let Some(compile) = &compile_output {
                    if compile.status.success() {
                        bail!(
                            "Known failure unexpectedly compiled successfully (desugar + rustc)."
                        );
                    } else {
                        test_output = String::from_utf8_lossy(&compile.stderr).into_owned();
                    }
                } else {
                    bail!("Known failure had successful desugaring but no compile output?");
                }
            } else if output.status.code() == Some(101) {
                test_output = stderr_owned.clone();
            } else {
                test_output = stderr_owned.clone();
            }
        }
        TestKind::KnownPanic | TestKind::Skip => {}
    }

    if test_case.magic_comments.check_output {
        compare_or_overwrite(&test_output, &test_case.expected)?;
    } else if test_case.expected.exists() {
        std::fs::remove_file(&test_case.expected)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let tests: Vec<_> = WalkDir::new(TESTS_DIR)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|e| e.extension().is_some_and(|ext| ext == "rs"))
        .filter(|e| !e.to_str().unwrap().ends_with(".desugared.rs"))
        .map(|path| setup_test(path))
        .try_collect()?;
    let args = libtest_mimic::Arguments::from_args();
    libtest_mimic::run(&args, tests).exit()
}
