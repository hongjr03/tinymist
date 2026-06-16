#![doc = include_str!("../README.md")]

use std::path::PathBuf;

use clap::Parser;
use tinymist_std::error::prelude::*;
use typlite::CompileOnceArgs;

/// Common arguments of compile, watch, and query.
#[derive(Debug, Clone, Parser, Default)]
pub struct CompileArgs {
    /// Shared compile-once arguments.
    #[clap(flatten)]
    pub compile: CompileOnceArgs,

    /// Path to output file.
    #[clap(value_name = "OUTPUT", default_value = None)]
    pub output: Option<String>,

    /// Configures the path of assets directory.
    #[clap(long, default_value = None, value_name = "ASSETS_PATH")]
    pub assets_path: Option<PathBuf>,

    /// Specifies the package to process markup.
    #[clap(long = "processor", default_value = None, value_name = "PACKAGE_SPEC")]
    pub processor: Option<String>,
}

fn main() -> Result<()> {
    let _ = env_logger::try_init();
    let _args = CompileArgs::parse();
    bail!("typlite conversion is unavailable while the crate is being rewritten")
}
