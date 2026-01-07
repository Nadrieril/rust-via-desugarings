#![feature(rustc_private)]
#![feature(if_let_guard)]

use core::fmt;
use std::env;

use anyhow::bail;
use desugar::desugar_thir;
use rustc_driver::{Callbacks, Compilation};
use rustc_interface::{interface::Compiler, Config};
use rustc_middle::ty::TyCtxt;

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_driver;
extern crate rustc_error_messages;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;

mod desugar;

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
        // setup_compiler(config, false);
        config.override_queries = Some(|_sess, providers| {
            providers.thir_body = |tcx, def_id| {
                let (body, expr_id) =
                    (rustc_interface::DEFAULT_QUERY_PROVIDERS.thir_body)(tcx, def_id)?;
                let mut body = body.steal();
                desugar_thir(tcx, &mut body);
                Ok((tcx.alloc_steal_thir(body), expr_id))
            };
        });
    }

    fn after_expansion<'tcx>(&mut self, _compiler: &Compiler, _tcx: TyCtxt<'tcx>) -> Compilation {
        // Set up our own `DefId` debug routine.
        rustc_hir::def_id::DEF_ID_DEBUG
            .swap(&(def_id_debug as fn(_, &mut fmt::Formatter<'_>) -> _));
        Compilation::Continue
    }
}

/// Helper that runs the compiler and catches its fatal errors.
fn run_compiler_with_callbacks(
    args: Vec<String>,
    callbacks: &mut (dyn Callbacks + Send),
) -> anyhow::Result<()> {
    rustc_driver::catch_fatal_errors(|| rustc_driver::run_compiler(&args, callbacks))
        .or_else(|_| bail!("compiler encountered fatal error"))
}

/// Returns the values of the command-line options that match `find_arg`. The options are built-in
/// to be of the form `--arg=value` or `--arg value`.
pub fn arg_values<'a, T: AsRef<str>>(
    args: &'a [T],
    needle: &'a str,
) -> impl Iterator<Item = &'a str> {
    struct ArgFilter<'a, T> {
        args: std::slice::Iter<'a, T>,
        needle: &'a str,
    }
    impl<'a, T: AsRef<str>> Iterator for ArgFilter<'a, T> {
        type Item = &'a str;
        fn next(&mut self) -> Option<Self::Item> {
            while let Some(arg) = self.args.next() {
                let mut split_arg = arg.as_ref().splitn(2, '=');
                if split_arg.next() == Some(self.needle) {
                    return match split_arg.next() {
                        // `--arg=value` form
                        arg @ Some(_) => arg,
                        // `--arg value` form
                        None => self.args.next().map(|x| x.as_ref()),
                    };
                }
            }
            None
        }
    }
    ArgFilter {
        args: args.iter(),
        needle,
    }
}

pub fn arg_value<'a, T: AsRef<str>>(args: &'a [T], needle: &'a str) -> Option<&'a str> {
    arg_values(args, needle).next()
}

fn main() -> anyhow::Result<()> {
    // Retrieve the command-line arguments pased to our driver. The first arg is the path to
    // the current executable, we skip it.
    let compiler_args: Vec<String> = env::args().collect();

    // When called using cargo, we tell cargo to use our driver by setting the `RUSTC_WRAPPER` env
    // var.
    // We may however not want to be calling our driver on all crates; `CARGO_PRIMARY_PACKAGE` tells us
    // whether the crate was specifically selected or is a dependency.
    let is_workspace_dependency =
        env::var("CARGO_DESUGAR_USING_CARGO").is_ok() && !env::var("CARGO_PRIMARY_PACKAGE").is_ok();
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
        // Call the Rust compiler with our custom callback.
        let mut callback = DesugarCallbacks {};
        run_compiler_with_callbacks(compiler_args, &mut callback)?;
    } else {
        // Run the compiler normally.
        run_compiler_with_callbacks(compiler_args, &mut RunCompilerNormallyCallbacks {})?;
    }

    Ok(())
}
