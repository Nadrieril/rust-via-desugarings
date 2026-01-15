#![feature(exit_status_error)]
//! End-to-end check against rustc's UI test suite.
//! For each UI test we can understand, we desugar it and assert that compilation
//! succeeds or fails the same way as the original source.

use std::{
    env::{self, current_dir},
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Component, Path, PathBuf},
    process::{Command, ExitStatus},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow, bail};
use libtest_mimic::Trial;
use walkdir::WalkDir;

mod util;

/// How many of the rustc tests to run, for now.
static HOW_MANY_TESTS: usize = 1000;
/// The page number for tests, where each page has `HOW_MANY_TESTS` tests.
static PAGE: usize = 2;

/// Tests that are failing because the hir printer is incomplere.
static INCOMPLETE_HIR_PRETTY: &[&str] = &[
    // Missing `impl Trait` printing.
    "reachable/expr_cast.rs",
    "rust-2021/inherent-method-collision.rs",
    "traits/next-solver/alias-relate/tait-eq-tait.rs",
    "generic-associated-types/issue-80433-reduced.rs",
    // Uses relative paths for `#[path = "..."]` attre
    "simd/intrinsic/generic-bswap-byte.rs",
    // Doesn't print #[may_dangle]
    "nll/drop-may-dangle.rs",
    // Depends on non-public paths
    "try-trait/",
    // Bad attr printing
    "cfg/conditional-compile-arch.rs",
    // Bad region `+` printing
    "regions/regions-bound-lists-feature-gate-2.rs",
    // Bad dyn parenthisation
    "traits/mut-trait-in-struct-8249.rs",
    // Bad handling of empty clause lists
    "traits/alias/empty.rs",
];

/// Tests that are failing for nontrivial-to-fix reasons.
static UNSUPPORTED_HARD: &[&str] = &[
    // Printing short paths can cause shadowing and visibility issues.
    "self/self-impl-2.rs",
    "symbol-names/issue-53912.rs",
    "higher-ranked/trait-bounds/normalize-under-binder/issue-56556.rs",
    "imports/export-glob-imports-target.rs",
    // Two-phase mut
    "borrowck/two-phase-bin-ops.rs",
    // Print higher-kinded bound variables
    "implied-bounds/implied-bounds-on-nested-references-plus-variance-2.rs",
    "const_prop/dont-propagate-generic-instance.rs",
    "associated-types/associated-types-normalize-in-bounds.rs",
    "binding/underscore-prefixed-function-argument.rs",
    // Const-to-pat exposes private fields
    "consts/tuple-struct-constructors.rs",
];

/// Tests that we won't fix.
static WONTFIX: &[&str] = &[
    // Lints against unnecessarily precise paths.
    "lint/unused-qualifications-global-paths.rs",
    // Unused parens lint
    "lint/unused/no-unused-parens-return-block.rs",
    // Delegation feature results in weird-ass types
    "delegation/",
    // Loop match feature
    "loop-match/",
    // Labeled break, try blocks
    "let-else/issue-100103.rs",
    // Hygiene shenanigans
    "hygiene/",
];

#[derive(Debug, Clone)]
struct Config {
    workdir: PathBuf,
    rustc_src: PathBuf,
    ui_src: PathBuf,
    test_root: PathBuf,
    logs_root: PathBuf,
    build_root: PathBuf,
    rustc_bin: String,
    cargo_desugar_bin: PathBuf,
    rustc_commit: String,
}

#[derive(Debug, Default, Clone)]
struct ParsedDirectives {
    rustc_args: Vec<String>,
    skip_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct TestCase {
    config: Arc<Config>,
    input_path: PathBuf,
    rel_path: PathBuf,
    directives: ParsedDirectives,
}

fn env_or_default(key: &str, default: impl Into<String>) -> String {
    std::env::var(key).unwrap_or_else(|_| default.into())
}

fn rustc_commit(rustc_bin: &str) -> Result<String> {
    let output = Command::new(rustc_bin)
        .arg("-vV")
        .output()
        .context("running rustc -vV")?;
    if !output.status.success() {
        bail!("rustc -vV failed with status {}", output.status);
    }
    let stdout = String::from_utf8(output.stdout)?;
    let commit = stdout
        .lines()
        .find_map(|line| line.strip_prefix("commit-hash: "))
        .ok_or_else(|| anyhow!("could not find commit-hash in rustc -vV output"))?;
    Ok(commit.to_string())
}

fn configure() -> Result<Config> {
    // Find a parent folder that contains a `Cargo.lock`, otherwise the current folder.
    let cargo_target_dir = {
        let mut root_path = PathBuf::new();
        let mut found = false;
        for path in env::current_dir()?.ancestors() {
            if path.join("Cargo.lock").exists() {
                found = true;
                break;
            }
            root_path = root_path.join(Component::ParentDir);
        }
        if !found {
            root_path = PathBuf::new();
        }
        root_path.join("target")
    };

    let rustc_bin = "rustc".to_string();
    let cargo_desugar_bin = assert_cmd::cargo::cargo_bin!("cargo-desugar").to_path_buf();
    let rustc_commit = rustc_commit(&rustc_bin).context("extracting rustc commit")?;

    let workdir = cargo_target_dir.join("rustc-test-suite");
    let rustc_src_default = workdir.join(format!("rustc-{rustc_commit}"));
    let test_root = workdir.join("desugared");
    let logs_root = workdir.join("logs");
    let build_root = workdir.join("build");

    let rustc_src = PathBuf::from(env_or_default(
        "RUSTC_SRC",
        rustc_src_default.display().to_string(),
    ));
    let ui_src = rustc_src.join("tests/ui");

    Ok(Config {
        workdir,
        rustc_src,
        ui_src,
        test_root,
        logs_root,
        build_root,
        rustc_bin,
        cargo_desugar_bin,
        rustc_commit,
    })
}

fn download_rustc_sources(cfg: &Config) -> Result<()> {
    if cfg.ui_src.exists() {
        return Ok(());
    }

    fs::create_dir_all(&cfg.workdir)?;
    let tarball = env_or_default(
        "RUSTC_TARBALL",
        format!(
            "https://github.com/rust-lang/rust/archive/{}.tar.gz",
            cfg.rustc_commit
        ),
    );
    let archive = cfg
        .workdir
        .join(format!("rust-{}.tar.gz", cfg.rustc_commit));

    Command::new("curl")
        .arg("-L")
        .arg(&tarball)
        .arg("-o")
        .arg(&archive)
        .status()?
        .exit_ok()?;
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&cfg.workdir)
        .status()
        .context("extracting rustc tarball")?;
    if !status.success() {
        bail!("failed to extract rustc tarball: {status}");
    }
    let unpacked = cfg.workdir.join(format!("rust-{}", cfg.rustc_commit));
    fs::rename(unpacked, &cfg.rustc_src)?;
    Ok(())
}

fn parse_directives(path: &Path) -> Result<ParsedDirectives> {
    let mut out = ParsedDirectives::default();

    if path.with_extension("stderr").exists() {
        out.skip_reason = Some("this test is expected to fail".into());
    }
    if out.skip_reason.is_none() {
        for line in BufReader::new(File::open(path)?).lines() {
            let line = line?;
            let Some(rest) = line.strip_prefix("//@") else {
                continue;
            };
            let directive = rest.trim();
            if let Some(flags) = directive.strip_prefix("compile-flags:") {
                let flags = flags.trim();
                out.rustc_args
                    .extend(flags.split_whitespace().map(|s| s.to_string()));
            } else if let Some(edition) = directive.strip_prefix("edition:") {
                let edition = edition.trim();
                if !edition.is_empty() {
                    out.rustc_args.push("--edition".into());
                    out.rustc_args.push(edition.to_string());
                }
            } else if let Some(kinds) = directive.strip_prefix("crate-type:") {
                let kinds = kinds.trim();
                out.rustc_args.extend(
                    kinds
                        .split_whitespace()
                        .map(|k| format!("--crate-type={k}")),
                );
            } else if directive == "no-prefer-dynamic" {
                out.rustc_args.push("-C".into());
                out.rustc_args.push("prefer-dynamic=no".into());
            } else if directive.starts_with("revisions:") {
                out.skip_reason = Some("multiple revisions not supported".into());
            } else if directive.starts_with("aux-build:")
                || directive.starts_with("aux-crate:")
                || directive.contains("auxiliary")
            {
                out.skip_reason = Some("auxiliary crates unsupported".into());
            } else if directive.starts_with("ignore-") || directive.starts_with("only-") {
                out.skip_reason = Some("target-filtered test".into());
            } else if directive.starts_with("needs-") {
                out.skip_reason = Some("needs-* requirement".into());
            }
        }
    }

    Ok(out)
}

fn write_log(path: &Path, stdout: &[u8], stderr: &[u8]) -> io::Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    let mut contents = Vec::new();
    contents.extend_from_slice(stdout);
    if !stdout.is_empty() && !stderr.is_empty() {
        contents.extend_from_slice(b"\n--- stderr ---\n");
    }
    contents.extend_from_slice(stderr);
    fs::write(path, contents)
}

fn run_rustc(
    cfg: &Config,
    out_dir: &Path,
    args: &[String],
    input: &Path,
) -> Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    fs::create_dir_all(out_dir)?;
    let mut cmd = Command::new(&cfg.rustc_bin);
    cmd.arg("--emit=metadata")
        .arg("--out-dir")
        .arg(out_dir)
        .args(args)
        .arg(input);
    let output = cmd
        .output()
        .with_context(|| format!("error running: {cmd:?}"))?;
    Ok((output.status, output.stdout, output.stderr))
}

fn run_desugar(
    cfg: &Config,
    out_dir: &Path,
    args: &[String],
    input: &Path,
    desugared: &Path,
) -> Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let mut cmd = Command::new(&cfg.cargo_desugar_bin);
    cmd.arg("rustc")
        .arg("--")
        .arg("--out-dir")
        .arg(out_dir)
        .args(args)
        .arg(input);
    let output = cmd
        .output()
        .with_context(|| format!("error running: {cmd:?}"))?;
    fs::write(desugared, &output.stdout)?;
    Ok((output.status, output.stdout, output.stderr))
}

fn setup_test(cfg: Arc<Config>, input_path: PathBuf) -> Result<Trial> {
    let rel_path = input_path.strip_prefix(&cfg.ui_src).unwrap().to_path_buf();

    let path_str = rel_path.to_string_lossy();
    let skip_reason = if INCOMPLETE_HIR_PRETTY
        .iter()
        .copied()
        .chain(UNSUPPORTED_HARD.iter().copied())
        .chain(WONTFIX.iter().copied())
        // Broken `#[attr = ..]` pretty-printing for attrs
        .chain(include_str!("./failing-due-to-attrs").lines())
        // Uses private `core::fmt::rt::Argument` type.
        .chain(include_str!("./failing-due-to-fmt-internals").lines())
        // Missing `impl Trait` printing.
        .chain(include_str!("./failing-due-to-impl-trait").lines())
        .any(|p| path_str.starts_with(p))
    {
        Some("incomplete hir pretty-printer")
    } else if rel_path.components().any(|c| c.as_os_str() == "auxiliary") {
        Some("auxiliary crate")
    } else if rel_path
        .ancestors()
        .any(|dir| dir.join("compiletest-ignore-dir").exists())
    {
        Some("compiletest-ignore-dir")
    } else {
        None
    };

    let directives = if skip_reason.is_some() {
        // Don't bother reading the file
        ParsedDirectives {
            skip_reason: skip_reason.map(String::from),
            ..Default::default()
        }
    } else {
        parse_directives(&input_path)?
    };

    // Skip helper crates and directories compiletest marks as ignored.
    let name = rel_path.display().to_string();
    let ignore = directives.skip_reason.is_some();

    let case = TestCase {
        config: cfg,
        input_path,
        rel_path,
        directives,
    };

    let trial = Trial::test(name, move || perform_test(&case).map_err(|e| e.into()))
        .with_ignored_flag(ignore);
    Ok(trial)
}

fn perform_test(case: &TestCase) -> Result<()> {
    if let Some(reason) = &case.directives.skip_reason {
        bail!("skipped: {reason}");
    }
    let cfg = &case.config;
    let rel_dir = case.rel_path.parent().unwrap();
    let desugared = cfg.test_root.join(&case.rel_path);
    fs::create_dir_all(desugared.parent().unwrap())?;
    if desugared.exists() {
        fs::remove_file(&desugared)?;
    }

    let build_orig = cfg.build_root.join(rel_dir).join("orig");
    let build_desugar = cfg.build_root.join(rel_dir).join("desugar");
    let build_desugared = cfg.build_root.join(rel_dir).join("desugared");

    let logs_base = cfg.logs_root.join(&case.rel_path);
    let orig_log = logs_base.with_extension("orig.log");
    let desugar_log = logs_base.with_extension("desugar.log");
    let desugared_log = logs_base.with_extension("desugared.log");

    let (orig_status, orig_stdout, orig_stderr) = run_rustc(
        cfg,
        &build_orig,
        &case.directives.rustc_args,
        &case.input_path,
    )?;
    write_log(&orig_log, &orig_stdout, &orig_stderr)?;

    // Symlink the original file next to the desugared one.
    let orig_symlink = desugared.with_added_extension("orig");
    if !orig_symlink.exists() {
        let input_path_abs = current_dir().unwrap().join(&case.input_path);
        std::os::unix::fs::symlink(input_path_abs, &orig_symlink)?;
    }

    let (desugar_status, _desugar_stdout, desugar_stderr) = run_desugar(
        cfg,
        &build_desugar,
        &case.directives.rustc_args,
        &case.input_path,
        &desugared,
    )?;
    write_log(&desugar_log, b"", &desugar_stderr)?;
    if !desugar_status.success() {
        // Copy the file as-is.
        fs::copy(&case.input_path, &desugared).with_context(|| {
            format!(
                "copying {} to {}",
                case.input_path.display(),
                desugared.display()
            )
        })?;
    }

    let (desugared_status, desugared_stdout, desugared_stderr) = run_rustc(
        cfg,
        &build_desugared,
        &case.directives.rustc_args,
        &desugared,
    )?;
    write_log(&desugared_log, &desugared_stdout, &desugared_stderr)?;

    if orig_status.success() == desugared_status.success() {
        Ok(())
    } else {
        let selected_log = if !desugared_status.success() {
            desugared_stderr
        } else {
            orig_stderr
        };
        bail!(
            "status mismatch:\n \
            original:  {orig_status}\n \
            desugared: {desugared_status}\n\
            files:\n \
            original:  {}\n \
            desugared: {}\n \
            orig build log:      {}\n \
            desugaring log:      {}\n \
            desugared build log: {}\n\n\
            {}",
            orig_symlink.display(),
            desugared.display(),
            orig_log.display(),
            desugar_log.display(),
            desugared_log.display(),
            String::from_utf8_lossy(&selected_log),
        )
    }
}

fn collect_tests(cfg: Arc<Config>) -> Result<Vec<Trial>> {
    download_rustc_sources(&cfg)?;
    fs::create_dir_all(&cfg.test_root)?;
    fs::create_dir_all(&cfg.logs_root)?;
    fs::create_dir_all(&cfg.build_root)?;
    let mut tests = Vec::new();
    let total_num_of_tests = 20000;
    for path in WalkDir::new(&cfg.ui_src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|e| e.extension().is_some_and(|ext| ext == "rs"))
        .enumerate()
        // Uniformly sample `HOW_MANY_TESTS` tests across all tests
        .filter(|(i, _)| (i + PAGE) % (total_num_of_tests / HOW_MANY_TESTS) == 0)
        .map(|(_, path)| path)
    {
        tests.push(setup_test(cfg.clone(), path)?);
    }
    Ok(tests)
}

fn main() -> Result<()> {
    let cfg = Arc::new(configure()?);
    let tests = collect_tests(cfg)?;
    let args = libtest_mimic::Arguments::from_args();
    libtest_mimic::run(&args, tests).exit()
}
