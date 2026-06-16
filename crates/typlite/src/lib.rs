//! Minimal typlite rewrite shell.
//!
//! The old converter implementation has been removed as the first step of the
//! rewrite. This crate intentionally keeps a small compatibility surface for
//! workspace callers while the new implementation is built.

pub mod common;

use std::path::PathBuf;
use std::sync::Arc;

use ecow::EcoString;
use tinymist_project::LspWorld;
use tinymist_std::error::prelude::*;
use typst::diag::SourceDiagnostic;
use typst::syntax::FileId;

pub use crate::common::Format;
pub use tinymist_project::CompileOnceArgs;

/// The result type for typlite.
pub type Result<T> = tinymist_std::Result<T>;

const UNAVAILABLE: &str = "typlite conversion is unavailable while the crate is being rewritten";

fn unavailable<T>() -> Result<T> {
    bail!("{}", UNAVAILABLE)
}

/// A placeholder conversion document.
///
/// This type remains so existing callers can keep compiling while the new
/// converter pipeline is introduced.
#[derive(Debug, Default, Clone)]
pub struct MarkdownDocument;

impl MarkdownDocument {
    /// Get collected conversion warnings.
    pub fn warnings(&self) -> Vec<SourceDiagnostic> {
        Vec::new()
    }

    /// Convert the placeholder document to Markdown.
    pub fn to_md_string(&self) -> Result<EcoString> {
        unavailable()
    }

    /// Convert the placeholder document to plain text.
    pub fn to_text_string(&self) -> Result<EcoString> {
        unavailable()
    }

    /// Convert the placeholder document to LaTeX.
    pub fn to_tex_string(&self) -> Result<EcoString> {
        unavailable()
    }

    /// Convert the placeholder document to DOCX.
    #[cfg(feature = "docx")]
    pub fn to_docx(&self) -> Result<Vec<u8>> {
        unavailable()
    }
}

/// A color theme for rendering conversion assets.
#[derive(Debug, Default, Clone, Copy)]
pub enum ColorTheme {
    /// Light color theme.
    #[default]
    Light,
    /// Dark color theme.
    Dark,
}

/// Source mapping information for a wrapped typlite compilation.
#[derive(Debug, Clone)]
pub struct WrapInfo {
    /// The synthetic wrapper file that hosts the original Typst source.
    pub wrap_file_id: FileId,
    /// The user's actual Typst source file.
    pub original_file_id: FileId,
    /// Number of UTF-8 bytes injected ahead of the original source.
    pub prefix_len_bytes: usize,
}

/// Conversion feature flags accepted by the typlite shell.
#[derive(Debug, Default, Clone)]
pub struct TypliteFeat {
    /// The preferred color theme.
    pub color_theme: Option<ColorTheme>,
    /// The path of external assets directory.
    pub assets_path: Option<PathBuf>,
    /// Allows GFM (GitHub Flavored Markdown) markups.
    pub gfm: bool,
    /// Annotate the elements for identification.
    pub annotate_elem: bool,
    /// Embed errors in the output instead of yielding them.
    pub soft_error: bool,
    /// Remove HTML tags from the output.
    pub remove_html: bool,
    /// The target to convert.
    pub target: Format,
    /// Import context for code examples.
    pub import_context: Option<String>,
    /// Specifies the package to process markup.
    pub processor: Option<String>,
    /// Optional mapping from the wrapper file back to the original source.
    pub wrap_info: Option<WrapInfo>,
}

impl TypliteFeat {
    /// Prepare a world for conversion.
    ///
    /// The rewrite shell leaves the world unchanged and returns no wrap
    /// information. A later implementation will rebuild the old preparation
    /// behavior on top of the new pipeline.
    pub fn prepare_world(
        &self,
        world: &LspWorld,
        _format: Format,
    ) -> Result<(LspWorld, Option<WrapInfo>)> {
        Ok((world.clone(), None))
    }
}

/// Task builder for converting a Typst document.
pub struct Typlite {
    _world: Arc<LspWorld>,
    _feat: TypliteFeat,
    _format: Format,
}

impl Typlite {
    /// Creates a new typlite conversion task.
    pub fn new(world: Arc<LspWorld>) -> Self {
        Self {
            _world: world,
            _feat: Default::default(),
            _format: Format::Md,
        }
    }

    /// Sets conversion features.
    pub fn with_feature(mut self, feat: TypliteFeat) -> Self {
        self._feat = feat;
        self
    }

    /// Sets the output format.
    pub fn with_format(mut self, format: Format) -> Self {
        self._format = format;
        self
    }

    /// Convert the content to a string.
    pub fn convert(self) -> Result<EcoString> {
        unavailable()
    }

    /// Convert the content to a DOCX document.
    #[cfg(feature = "docx")]
    pub fn to_docx(self) -> Result<Vec<u8>> {
        unavailable()
    }

    /// Convert the content to a placeholder document.
    pub fn convert_doc(self, _format: Format) -> Result<MarkdownDocument> {
        unavailable()
    }

    /// Convert a prepared world to a placeholder document.
    pub fn convert_doc_prepared(
        _feat: TypliteFeat,
        _format: Format,
        _world: Arc<LspWorld>,
    ) -> Result<MarkdownDocument> {
        unavailable()
    }
}
