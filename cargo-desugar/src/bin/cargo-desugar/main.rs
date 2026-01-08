#![feature(rustc_private)]

use std::{env, path::PathBuf, process::Command};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use rust_via_desugarings::options::{CliOpts, DESUGAR_ARGS_ENV};
use rust_via_desugarings::util::arg_value;

extern crate rustc_driver;

#[derive(Debug, Parser)]
#[command(name = "cargo-desugar")]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    /// Runs the driver on a cargo project.
    Cargo(CargoArgs),
    /// Runs the driver on a single rustc invocation.
    Rustc(RustcArgs),
}

/// Usage: `cargo-desugar cargo [desugar options] -- [cargo build options]`
#[derive(Args, Debug)]
struct CargoArgs {
    #[command(flatten)]
    opts: CliOpts,

    /// Args that `cargo build` accepts.
    #[arg(last = true)]
    cargo: Vec<String>,
}

/// Usage: `cargo-desugar rustc [desugar options] -- [rustc options]`
#[derive(Args, Debug)]
struct RustcArgs {
    #[command(flatten)]
    opts: CliOpts,

    /// Args that `rustc` accepts.
    #[arg(last = true)]
    rustc: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let exit_status = match cli.command {
        CommandKind::Cargo(subcmd_cargo) => {
            translate_with_cargo(subcmd_cargo.opts, subcmd_cargo.cargo)?
        }
        CommandKind::Rustc(mut subcmd_rustc) => {
            let mut options = subcmd_rustc.opts;
            options.rustc_args.append(&mut subcmd_rustc.rustc);
            translate_without_cargo(options)?
        }
    };

    handle_exit_status(exit_status)
}

fn translate_with_cargo(
    options: CliOpts,
    cargo_args: Vec<String>,
) -> anyhow::Result<std::process::ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.env("RUSTC_WRAPPER", driver_path()?);
    cmd.env("CARGO_DESUGAR_USING_CARGO", "1");
    cmd.env_remove("CARGO_PRIMARY_PACKAGE");
    cmd.env(DESUGAR_ARGS_ENV, serde_json::to_string(&options).unwrap());
    cmd.arg("build");
    if arg_value(&cargo_args, "--target").is_none() {
        // Make sure the build target is explicitly set. This is needed to detect which crates are
        // proc-macro/build-script in the driver.
        cmd.arg("--target");
        cmd.arg(&host_triple()?);
    }
    cmd.args(cargo_args);
    Ok(cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?"))
}

fn translate_without_cargo(mut options: CliOpts) -> anyhow::Result<std::process::ExitStatus> {
    let mut cmd = driver_cmd()?;
    let is_specified = |arg: &str| {
        options
            .rustc_args
            .iter()
            .any(|input: &String| input.starts_with(arg))
    };
    if !is_specified("--target") {
        // Make sure the build target is explicitly set. This is needed to detect which crates are
        // proc-macro/build-script in the driver.
        cmd.arg("--target");
        cmd.arg(&host_triple()?);
    }
    cmd.args(std::mem::take(&mut options.rustc_args));
    cmd.env(DESUGAR_ARGS_ENV, serde_json::to_string(&options).unwrap());
    Ok(cmd
        .spawn()
        .expect("could not run cargo-desugar-driver")
        .wait()
        .expect("failed to wait for cargo-desugar-driver?"))
}

fn handle_exit_status(exit_status: std::process::ExitStatus) -> Result<()> {
    if exit_status.success() {
        Ok(())
    } else {
        let code = exit_status.code().unwrap_or(-1);
        // Rethrow the exit code
        std::process::exit(code);
    }
}

fn driver_path() -> anyhow::Result<PathBuf> {
    let mut path = env::current_exe()?;
    path.set_file_name("cargo_desugar_driver");
    Ok(path)
}

fn driver_cmd() -> anyhow::Result<Command> {
    let mut cmd = Command::new(driver_path()?);
    // Mimic Cargo's RUSTC_WRAPPER calling convention: the first arg is the path to rustc.
    cmd.arg("rustc");
    Ok(cmd)
}

fn host_triple() -> anyhow::Result<String> {
    Ok(rustc_version::version_meta()?.host)
}
