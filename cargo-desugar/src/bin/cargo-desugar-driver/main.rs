#![feature(rustc_private)]
#![feature(if_let_guard)]

use core::fmt;
use std::env;

use anyhow::{Result, bail};
use rust_via_desugarings::{
    desugar::desugar_thir,
    options::{CliOpts, DESUGAR_ARGS_ENV},
    print::print_thir,
    util::arg_value,
};
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::{Config, interface::Compiler};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::{OutputType, OutputTypes};

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_mir_build;
extern crate rustc_session;

/// Custom `DefId` debug routine that doesn't print unstable values like ids and hashes.
fn def_id_debug(def_id: rustc_hir::def_id::DefId, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    rustc_middle::ty::tls::with_opt(|opt_tcx| {
        if let Some(tcx) = opt_tcx {
            let crate_name = if def_id.is_local() {
                tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE)
            } else {
                tcx.cstore_untracked().crate_name(def_id.krate)
            };
            write!(
                f,
                "{}{}",
                crate_name,
                tcx.def_path(def_id).to_string_no_crate_verbose()
            )?;
        } else {
            write!(f, "<can't access `tcx` to print `DefId` path>")?;
        }
        Ok(())
    })
}

/// Dummy callbacks used to run the compiler normally when we shouldn't be analyzing the crate.
pub struct RunCompilerNormallyCallbacks {}
impl Callbacks for RunCompilerNormallyCallbacks {
    fn config(&mut self, _config: &mut Config) {}
}

pub struct DesugarCallbacks {}
impl Callbacks for DesugarCallbacks {
    fn config(&mut self, config: &mut Config) {
        // Set up our own `DefId` debug routine.
        rustc_hir::def_id::DEF_ID_DEBUG
            .swap(&(def_id_debug as fn(_, &mut fmt::Formatter<'_>) -> _));
        config.opts.unstable_opts.no_codegen = true;
        config.opts.output_types = OutputTypes::new(&[(OutputType::Metadata, None)]);
        config.override_queries = Some(|_sess, providers| {
            providers.thir_body = |tcx, def_id| {
                let (body, root) =
                    (rustc_interface::DEFAULT_QUERY_PROVIDERS.thir_body)(tcx, def_id)?;
                let mut body = body.steal();
                desugar_thir(tcx, def_id, &mut body);
                println!("{}", print_thir(tcx, def_id, &body, root));
                Ok((tcx.alloc_steal_thir(body), root))
            };
        });
    }

    fn after_analysis<'tcx>(&mut self, _compiler: &Compiler, tcx: TyCtxt<'tcx>) -> Compilation {
        for ldid in tcx.hir_body_owners() {
            let _ = tcx.thir_body(ldid); // ensure all thir bodies are built
        }
        Compilation::Stop
    }
}

/// Helper that runs the compiler and catches its fatal errors.
fn run_compiler_with_callbacks(
    args: Vec<String>,
    callbacks: &mut (dyn Callbacks + Send),
) -> Result<()> {
    rustc_driver::catch_fatal_errors(|| rustc_driver::run_compiler(&args, callbacks))
        .or_else(|_| bail!("compiler encountered fatal error"))
}

fn run_driver(options: CliOpts) -> Result<()> {
    // Retrieve the command-line arguments passed to our driver. The first arg is the path to
    // the current executable, we skip it to mimic the behavior with `RUSTC_WRAPPER` where the
    // first argument is the actual rustc path.
    let mut compiler_args: Vec<String> = env::args().skip(1).collect();

    // When called using cargo, we tell cargo to use our driver by setting the `RUSTC_WRAPPER` env
    // var. We may however not want to be calling our driver on all crates; `CARGO_PRIMARY_PACKAGE`
    // tells us whether the crate was specifically selected or is a dependency.
    let is_workspace_dependency =
        env::var("CARGO_DESUGAR_USING_CARGO").is_ok() && env::var("CARGO_PRIMARY_PACKAGE").is_err();
    // Determines if we are being invoked to build a crate for the "target" architecture, in
    // contrast to the "host" architecture. Host crates are for build scripts and proc macros and
    // still need to be built like normal; target crates are the ones we want to process.
    //
    // Currently, we detect this by checking for "--target=", which is never set for host crates.
    // This matches what Miri does, which hopefully makes it reliable enough. This relies on us
    // always invoking cargo itself with `--target`, which we'll have to enforce from our `cargo`
    // custom command.
    let is_target = arg_value(&compiler_args, "--target").is_some();
    // Whether this is the crate we want to translate.
    let is_selected_crate = !is_workspace_dependency && is_target;

    if is_selected_crate {
        for extra_flag in options.rustc_args {
            compiler_args.push(extra_flag);
        }

        // Call the Rust compiler with our custom callback.
        let mut callback = DesugarCallbacks {};
        run_compiler_with_callbacks(compiler_args, &mut callback)?;
    } else {
        // Run the compiler normally.
        run_compiler_with_callbacks(compiler_args, &mut RunCompilerNormallyCallbacks {})?;
    }

    Ok(())
}

fn main() -> Result<()> {
    // Retrieve the options by deserializing them from the environment variable
    let options: CliOpts = env::var(DESUGAR_ARGS_ENV)
        .ok()
        .map(|opts| serde_json::from_str(opts.as_str()))
        .transpose()?
        .unwrap_or_default();

    run_driver(options)
}
