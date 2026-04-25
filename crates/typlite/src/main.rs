//! Command line entry point for typlite conversion.

use std::io::Write;
use std::path::PathBuf;

use clap::{ArgAction, Parser, builder::ValueParser};
use tinymist_project::{
    CompileFontArgs, CompileOnceArgs, CompilePackageArgs, Feature, WorldProvider, parse_input_pair,
    parse_source_date_epoch,
    world::{DiagnosticFormat, system::print_diagnostics},
};
use tinymist_std::error::prelude::*;
use typlite::{Format, Typlite};

#[derive(Debug, Clone, Parser)]
#[clap(
    name = "typlite",
    author,
    version,
    about = "Convert Typst documents with typlite"
)]
struct Args {
    /// Typst compilation inputs.
    #[clap(flatten)]
    compile: CompileArgs,

    /// Output format.
    #[clap(long = "format", short = 'f', default_value_t, value_enum)]
    format: OutputFormat,

    /// Write output to a file. Use `-` or omit this option to write to stdout.
    #[clap(long = "output", short = 'o', value_name = "PATH")]
    output: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Parser)]
struct CompileArgs {
    /// Specify the path to input Typst file. If the path is relative, it will
    /// be resolved relative to the current working directory.
    #[clap(value_name = "INPUT")]
    input: Option<String>,

    /// Configure the project root.
    #[clap(long = "root", value_name = "DIR")]
    root: Option<PathBuf>,

    /// Specify font related arguments.
    #[clap(flatten)]
    font: CompileFontArgs,

    /// Specify package related arguments.
    #[clap(flatten)]
    package: CompilePackageArgs,

    /// Enable in-development Typst features.
    #[arg(long = "features", value_delimiter = ',', env = "TYPST_FEATURES")]
    features: Vec<Feature>,

    /// Add a string key-value pair visible through `sys.inputs`.
    #[clap(
        long = "input",
        value_name = "key=value",
        action = ArgAction::Append,
        value_parser = ValueParser::new(parse_input_pair),
    )]
    inputs: Vec<(String, String)>,

    /// Configure the document's creation date formatted as a UNIX timestamp.
    #[clap(
        long = "creation-timestamp",
        env = "SOURCE_DATE_EPOCH",
        value_name = "UNIX_TIMESTAMP",
        value_parser = parse_source_date_epoch,
        hide(true),
    )]
    creation_timestamp: Option<i64>,

    /// Specify the path to CA certificate file for network access.
    #[clap(long = "cert", env = "TYPST_CERT", value_name = "CERT_PATH")]
    cert: Option<PathBuf>,
}

impl From<CompileArgs> for CompileOnceArgs {
    fn from(args: CompileArgs) -> Self {
        Self {
            input: args.input,
            root: args.root,
            font: args.font,
            package: args.package,
            pdf: Default::default(),
            png: Default::default(),
            features: args.features,
            inputs: args.inputs,
            creation_timestamp: args.creation_timestamp,
            cert: args.cert,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum OutputFormat {
    /// Markdown.
    #[default]
    Md,
    /// LaTeX.
    Latex,
    /// Plain text.
    Text,
    /// DOCX.
    #[cfg(feature = "docx")]
    Docx,
}

impl From<OutputFormat> for Format {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Md => Self::Md,
            OutputFormat::Latex => Self::LaTeX,
            OutputFormat::Text => Self::Text,
            #[cfg(feature = "docx")]
            OutputFormat::Docx => Self::Docx,
        }
    }
}

fn main() -> typlite::Result<()> {
    let args = Args::parse();
    let compile = CompileOnceArgs::from(args.compile);
    let universe = compile.resolve()?;
    let world = universe.snapshot();
    let output = Typlite::new(world.clone().into())
        .with_format(args.format.into())
        .convert_with_diagnostics()?;

    if !output.warnings.is_empty() {
        print_diagnostics(&world, output.warnings.iter(), DiagnosticFormat::Human)
            .context_ut("failed to print typlite warnings")?;
    }

    match args.output.as_deref() {
        None => write_stdout(output.output.as_bytes()),
        Some(path) if path == std::path::Path::new("-") => write_stdout(output.output.as_bytes()),
        Some(path) => std::fs::write(path, output.output.as_bytes())
            .context_ut("failed to write typlite output"),
    }
}

fn write_stdout(bytes: &[u8]) -> typlite::Result<()> {
    std::io::stdout()
        .write_all(bytes)
        .context_ut("failed to write typlite output")?;
    Ok(())
}
