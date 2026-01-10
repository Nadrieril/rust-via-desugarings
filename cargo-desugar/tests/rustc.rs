#![feature(exit_status_error)]
//! End-to-end check against rustc's UI test suite.
//! For each UI test we can understand, we desugar it and assert that compilation
//! succeeds or fails the same way as the original source.

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow, bail};
use libtest_mimic::Trial;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
struct Config {
    workdir: PathBuf,
    rustc_src: PathBuf,
    ui_src: PathBuf,
    test_root: PathBuf,
    results_root: PathBuf,
    logs_root: PathBuf,
    build_root: PathBuf,
    rustc_bin: String,
    cargo_desugar_bin: PathBuf,
    rustc_commit: String,
}

#[derive(Debug, Clone)]
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
    let rustc_bin = "rustc".to_string();
    let cargo_desugar_bin = assert_cmd::cargo::cargo_bin!("cargo-desugar").to_path_buf();
    let rustc_commit = rustc_commit(&rustc_bin).context("extracting rustc commit")?;

    let workdir = PathBuf::from("target/rustc-ui-desugar");
    let rustc_src_default = workdir.join(format!("rustc-{rustc_commit}"));
    let test_root = workdir.join("ui");
    let results_root = workdir.join("results");
    let logs_root = results_root.join("logs");
    let build_root = results_root.join("build");

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
        results_root,
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
    let mut rustc_args = Vec::new();
    let mut skip_reason = None;

    for line in fs::read_to_string(path)?.lines() {
        let Some(rest) = line.strip_prefix("//@") else {
            continue;
        };
        let directive = rest.trim();
        if directive.starts_with("compile-flags:") {
            let flags = directive.trim_start_matches("compile-flags:").trim();
            rustc_args.extend(flags.split_whitespace().map(|s| s.to_string()));
        } else if directive.starts_with("edition:") {
            let edition = directive.trim_start_matches("edition:").trim();
            if !edition.is_empty() {
                rustc_args.push("--edition".into());
                rustc_args.push(edition.to_string());
            }
        } else if directive.starts_with("crate-type:") {
            let kinds = directive.trim_start_matches("crate-type:").trim();
            rustc_args.extend(
                kinds
                    .split_whitespace()
                    .map(|k| format!("--crate-type={k}")),
            );
        } else if directive == "no-prefer-dynamic" {
            rustc_args.push("-C".into());
            rustc_args.push("prefer-dynamic=no".into());
        } else if directive.starts_with("run-") {
            skip_reason.get_or_insert_with(|| "requires runtime execution".into());
        } else if directive.starts_with("revisions:") || directive.contains('[') {
            skip_reason.get_or_insert_with(|| "multiple revisions not supported".into());
        } else if directive.starts_with("aux-build:")
            || directive.starts_with("aux-crate:")
            || directive.contains("auxiliary")
        {
            skip_reason.get_or_insert_with(|| "auxiliary crates unsupported".into());
        } else if directive.starts_with("ignore-") || directive.starts_with("only-") {
            skip_reason.get_or_insert_with(|| "target-filtered test".into());
        } else if directive.starts_with("needs-") {
            skip_reason.get_or_insert_with(|| "needs-* requirement".into());
        } else if directive.starts_with("gate-test-")
            || directive == "check-pass"
            || directive == "build-pass"
            || directive == "run-pass"
            || directive == "run-pass-valgrind"
            || directive == "rustfix-only-machine-applicable"
            || directive == "run-rustfix"
            || directive == "normalize-stdout-test"
            || directive == "normalize-stderr-test"
            || directive == "must-compile-successfully"
        {
            // Markers we can safely ignore.
        } else {
            skip_reason.get_or_insert_with(|| format!("unsupported directive: {directive}"));
        }
    }

    Ok(ParsedDirectives {
        rustc_args,
        skip_reason,
    })
}

fn write_log(path: &Path, stdout: &[u8], stderr: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
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
    args: &[String],
    input: &Path,
    desugared: &Path,
) -> Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    let mut cmd = Command::new(&cfg.cargo_desugar_bin);
    cmd.arg("rustc").arg("--").args(args).arg(input);
    let output = cmd
        .output()
        .with_context(|| format!("error running: {cmd:?}"))?;
    fs::write(desugared, &output.stdout)?;
    Ok((output.status, output.stdout, output.stderr))
}

fn setup_test(cfg: Arc<Config>, input_path: PathBuf) -> Result<Trial> {
    let rel_path = input_path.strip_prefix(&cfg.ui_src).unwrap().to_path_buf();
    let mut directives = parse_directives(&input_path)?;

    // Skip helper crates and directories compiletest marks as ignored.
    if rel_path.components().any(|c| c.as_os_str() == "auxiliary") {
        directives
            .skip_reason
            .get_or_insert_with(|| "auxiliary crate".into());
    }
    if rel_path
        .ancestors()
        .any(|dir| dir.join("compiletest-ignore-dir").exists())
    {
        directives
            .skip_reason
            .get_or_insert_with(|| "compiletest-ignore-dir".into());
    }
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
    let stem = case
        .input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("cannot find file stem"))?;
    let rel_dir = case.rel_path.parent().unwrap();
    let desugared = cfg
        .test_root
        .join(rel_dir)
        .join(format!("{stem}-desugared.rs"));
    if desugared.exists() {
        fs::remove_file(&desugared)?;
    }
    let build_orig = cfg.build_root.join(rel_dir).join("orig");
    let build_desugared = cfg.build_root.join(rel_dir).join("desugared");
    let logs_dir = cfg.logs_root.join(rel_dir);
    let orig_log = logs_dir.join(format!("{stem}.orig.log"));
    let desugar_log = logs_dir.join(format!("{stem}.desugar.log"));
    let desugared_log = logs_dir.join(format!("{stem}.desugared.log"));
    let dir = desugared
        .parent()
        .ok_or_else(|| anyhow!("test has no parent directory"))?;
    fs::create_dir_all(&dir)?;
    fs::create_dir_all(&build_orig)?;

    let (orig_status, orig_stdout, orig_stderr) = run_rustc(
        cfg,
        &build_orig,
        &case.directives.rustc_args,
        &case.input_path,
    )?;
    write_log(&orig_log, &orig_stdout, &orig_stderr)?;

    let (desugar_status, _desugar_stdout, desugar_stderr) = run_desugar(
        cfg,
        &case.directives.rustc_args,
        &case.input_path,
        &desugared,
    )?;
    write_log(&desugar_log, b"", &desugar_stderr)?;
    if !desugar_status.success() {
        // Copy the file as-is, to check that rustc also errors on it.
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
        bail!(
            "status mismatch:\n \
            original:  {orig_status}\n \
            desugared: {desugared_status}\n\
            logs:\n \
            {}\n \
            {}\n \
            {}",
            orig_log.display(),
            desugar_log.display(),
            desugared_log.display()
        )
    }
}

fn collect_tests(cfg: Arc<Config>) -> Result<Vec<Trial>> {
    download_rustc_sources(&cfg)?;
    fs::create_dir_all(&cfg.test_root)?;
    fs::create_dir_all(&cfg.results_root)?;
    fs::create_dir_all(&cfg.logs_root)?;
    let mut tests = Vec::new();
    for path in WalkDir::new(&cfg.ui_src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|e| e.extension().is_some_and(|ext| ext == "rs"))
        .take(50)
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
