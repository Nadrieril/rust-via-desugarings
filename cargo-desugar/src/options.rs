use clap::Args;
use serde::{Deserialize, Serialize};

/// The name of the environment variable used to pass serialized options to the driver.
pub const DESUGAR_ARGS_ENV: &str = "CARGO_DESUGAR_ARGS";

/// Options understood by the desugaring driver.
#[derive(Clone, Debug, Default, Args, PartialEq, Eq, Serialize, Deserialize)]
#[command(name = "cargo-desugar")]
pub struct CliOpts {
    /// Extra flags to forward directly to rustc.
    #[arg(long = "rustc-arg")]
    #[serde(default)]
    pub rustc_args: Vec<String>,
}
