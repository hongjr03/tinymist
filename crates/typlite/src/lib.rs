//! Placeholder API for the next typlite implementation.

#![allow(missing_docs)]

pub mod backend;
mod bibliography;
mod content;
pub mod element_spec {
    include!(concat!(env!("OUT_DIR"), "/typlite-elements.rs"));
}
mod extract;
pub mod ir;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use ecow::EcoString;
use tinymist_project::{EntryReader, LspWorld, TaskInputs, base::ShadowApi};
use tinymist_std::error::prelude::*;
use typst::World;
use typst::WorldExt;
use typst::diag::SourceDiagnostic;
use typst::foundations::Bytes;
use typst_syntax::VirtualPath;
use typst_syntax::package::PackageSpec;
use typst_syntax::{FileId, Span};

use crate::backend::{BibliographyContext, render_markdown_with_diagnostics};
use crate::content::typlite_library;
use crate::extract::extract_document;

pub use tinymist_project::CompileOnceArgs;

/// Result type used by the typlite placeholder API.
pub type Result<T, Err = tinymist_std::Error> = std::result::Result<T, Err>;

/// Conversion output and non-fatal diagnostics.
#[derive(Debug, Clone)]
pub struct TypliteOutput {
    pub output: EcoString,
    pub warnings: ecow::EcoVec<SourceDiagnostic>,
}

/// A color theme for rendering converted content.
#[derive(Debug, Default, Clone, Copy)]
pub enum ColorTheme {
    #[default]
    Light,
    Dark,
}

/// Valid output formats for conversion.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    #[default]
    Md,
    LaTeX,
    Text,
    #[cfg(feature = "docx")]
    Docx,
}

/// Span mapping metadata for wrapped Typst sources.
#[derive(Debug, Clone)]
pub struct WrapInfo {
    pub wrap_file_id: FileId,
    pub original_file_id: FileId,
    pub prefix_len_bytes: usize,
}

impl WrapInfo {
    pub fn remap_span(&self, world: &dyn typst::World, span: Span) -> Option<Span> {
        if span.id() != Some(self.wrap_file_id) {
            return Some(span);
        }

        let range = world.range(span)?;
        let start = range.start.checked_sub(self.prefix_len_bytes)?;
        let end = range.end.checked_sub(self.prefix_len_bytes)?;
        let original_source = world.source(self.original_file_id).ok()?;
        let original_len = original_source.lines().len_bytes();

        if start >= original_len || end > original_len {
            return None;
        }

        Some(Span::from_range(self.original_file_id, start..end))
    }
}

/// Conversion options accepted by downstream crates.
#[derive(Debug, Default, Clone)]
pub struct TypliteFeat {
    pub color_theme: Option<ColorTheme>,
    pub assets_path: Option<PathBuf>,
    pub gfm: bool,
    pub annotate_elem: bool,
    pub soft_error: bool,
    pub remove_html: bool,
    pub target: Format,
    pub import_context: Option<String>,
    pub processor: Option<String>,
    pub wrap_info: Option<WrapInfo>,
}

impl TypliteFeat {
    pub fn prepare_world(
        &self,
        world: &LspWorld,
        format: Format,
    ) -> Result<(LspWorld, Option<WrapInfo>)> {
        let entry = world.entry_state();
        let current = entry.main().context("no main file in workspace")?;
        let wrap_main_id = current.join("__typlite_main.typ");
        let wrapped_source_id = current.join("__typlite_source.typ");

        let original_source = world
            .source(current)
            .context_ut("cannot fetch main source")?
            .text()
            .to_owned();

        let wrap_content = format!(
            r#"#import "@local/typlite-ir:0.1.0": typlite
#typlite(include "__typlite_source.typ")
"#
        );

        let task_inputs = TaskInputs {
            entry: Some(entry.select_in_workspace(wrap_main_id.vpath().as_rooted_path())),
            inputs: None,
        };
        let mut world = world.task(task_inputs).html_task().into_owned();
        world.library = typlite_library(&world.library);

        let package_id = FileId::new(
            Some(
                PackageSpec::from_str("@local/typlite-ir:0.1.0")
                    .context_ut("failed to create typlite IR package spec")?,
            ),
            VirtualPath::new("lib.typ"),
        );

        world
            .map_shadow_by_id(
                package_id.join("typst.toml"),
                Bytes::from_string(include_str!("typlite-ir.toml")),
            )
            .context_ut("cannot map typlite IR package manifest")?;
        world
            .map_shadow_by_id(
                package_id,
                Bytes::from_string(include_str!(concat!(env!("OUT_DIR"), "/typlite-ir.typ"))),
            )
            .context_ut("cannot map typlite IR package")?;
        world
            .map_shadow_by_id(wrapped_source_id, Bytes::from_string(original_source))
            .context_ut("cannot map wrapped source")?;
        world
            .map_shadow_by_id(wrap_main_id, Bytes::from_string(wrap_content))
            .context_ut("cannot map typlite wrapper")?;

        let wrap_info = if format == Format::Md {
            Some(WrapInfo {
                wrap_file_id: wrapped_source_id,
                original_file_id: current,
                prefix_len_bytes: 0,
            })
        } else {
            None
        };

        Ok((world, wrap_info))
    }
}

/// Placeholder task builder for typlite conversion.
pub struct Typlite {
    world: Arc<LspWorld>,
    feat: TypliteFeat,
    format: Format,
}

impl Typlite {
    pub fn new(world: Arc<LspWorld>) -> Self {
        Self {
            world,
            feat: TypliteFeat::default(),
            format: Format::default(),
        }
    }

    pub fn with_feature(mut self, feat: TypliteFeat) -> Self {
        self.feat = feat;
        self
    }

    pub fn with_format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    pub fn convert(self) -> Result<EcoString> {
        Ok(self.convert_with_diagnostics()?.output)
    }

    pub fn convert_with_diagnostics(self) -> Result<TypliteOutput> {
        let entry = self
            .world
            .entry_state()
            .main()
            .context("no main file in workspace")?;
        let (world, wrap_info) = self.feat.prepare_world(&self.world, self.format)?;
        let compiled = typst::compile::<typst_html::HtmlDocument>(&world);
        let html = compiled.output?;
        let ir = extract_document(&html)?;
        let bibliography = BibliographyContext::from_document(&ir, &world, entry)?;

        let result = match self.format {
            Format::Md => {
                let rendered = render_markdown_with_diagnostics(&ir, &bibliography)?;
                TypliteOutput {
                    output: rendered.output.into(),
                    warnings: remap_warnings(rendered.warnings, &world, wrap_info.as_ref()),
                }
            }
            Format::LaTeX => bail!("typlite LaTeX conversion is not implemented"),
            Format::Text => bail!("typlite text conversion is not implemented"),
            #[cfg(feature = "docx")]
            Format::Docx => bail!("typlite DOCX conversion is not implemented"),
        };

        Ok(result)
    }

    #[cfg(feature = "docx")]
    pub fn to_docx(self) -> Result<Vec<u8>> {
        let _ = (self.world, self.feat, self.format);
        bail!("typlite DOCX conversion is not implemented in the placeholder crate")
    }
}

fn remap_warnings(
    warnings: ecow::EcoVec<SourceDiagnostic>,
    world: &dyn World,
    wrap_info: Option<&WrapInfo>,
) -> ecow::EcoVec<SourceDiagnostic> {
    warnings
        .into_iter()
        .map(|mut warning| {
            if let Some(wrap_info) = wrap_info
                && let Some(span) = wrap_info.remap_span(world, warning.span)
            {
                warning.span = span;
            }
            warning
        })
        .collect()
}
