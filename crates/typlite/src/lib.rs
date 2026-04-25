//! Placeholder API for the next typlite implementation.

#![allow(missing_docs)]

pub mod backend;
pub mod element_spec {
    include!(concat!(env!("OUT_DIR"), "/typlite-elements.rs"));
}
mod extract;
pub mod ir;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use ecow::EcoString;
use hayagriva::archive::{ArchivedStyle, locales};
use hayagriva::citationberg::Style;
use hayagriva::io::{from_biblatex_str, from_yaml_str};
use hayagriva::{
    BibliographyDriver, BibliographyRequest, BufWriteFormat, CitationItem, CitationRequest, Library,
};
use tinymist_project::{EntryReader, LspWorld, TaskInputs, base::ShadowApi};
use tinymist_std::error::prelude::*;
use typst::World;
use typst::WorldExt;
use typst::foundations::Bytes;
use typst_syntax::VirtualPath;
use typst_syntax::package::PackageSpec;
use typst_syntax::{FileId, Span};

use crate::backend::{BibliographyContext, render_markdown_with_bibliography};
use crate::extract::extract_document;
use crate::ir::{Block, Document, ElementFieldValue, Inline};

pub use tinymist_project::CompileOnceArgs;

/// Result type used by the typlite placeholder API.
pub type Result<T, Err = tinymist_std::Error> = std::result::Result<T, Err>;

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

impl BibliographyContext {
    fn from_document(doc: &Document, world: &dyn World, entry: FileId) -> Result<Self> {
        let mut bibliographies = Vec::new();
        collect_bibliography_blocks(&doc.blocks, &mut bibliographies);
        if bibliographies.is_empty() {
            return Ok(Self::default());
        }

        let mut sources = Vec::new();
        let mut style = None;
        let mut full = false;
        for bibliography in &bibliographies {
            sources.extend(bibliography_sources(bibliography)?);
            if style.is_none() {
                style = bibliography
                    .scalar("style")
                    .filter(|value| !is_auto_or_none(value))
                    .map(str::to_owned);
            }
            full |= bibliography.scalar("full") == Some("true");
        }

        let library = load_bibliography_sources(world, entry, &sources)?;
        if library.is_empty() {
            return Ok(Self::default());
        }

        let cited = if full {
            library.keys().map(ToOwned::to_owned).collect::<Vec<_>>()
        } else {
            let mut cited = Vec::new();
            collect_cite_keys(&doc.blocks, &mut cited);
            cited
        };

        render_bibliography_entries(&library, &cited, style.as_deref().unwrap_or("ieee"))
    }
}

fn collect_bibliography_blocks<'a>(
    blocks: &'a [Block],
    out: &mut Vec<&'a crate::ir::BlockElementData>,
) {
    for block in blocks {
        match block {
            Block::Bibliography(data) => out.push(data),
            Block::Quote(blocks) => collect_bibliography_blocks(blocks, out),
            Block::Figure { body, .. }
            | Block::Align { body, .. }
            | Block::Block(crate::ir::BlockElementData { body, .. })
            | Block::Columns(crate::ir::BlockElementData { body, .. })
            | Block::Move(crate::ir::BlockElementData { body, .. })
            | Block::Pad(crate::ir::BlockElementData { body, .. })
            | Block::Rotate(crate::ir::BlockElementData { body, .. })
            | Block::Scale(crate::ir::BlockElementData { body, .. })
            | Block::Skew(crate::ir::BlockElementData { body, .. })
            | Block::Stack(crate::ir::BlockElementData { body, .. })
            | Block::Title(crate::ir::BlockElementData { body, .. }) => {
                collect_bibliography_blocks(body, out);
            }
            Block::List { items, .. } => {
                for item in items {
                    collect_bibliography_blocks(&item.body, out);
                }
            }
            Block::Terms { items } => {
                for item in items {
                    collect_bibliography_blocks(&item.description, out);
                }
            }
            _ => {}
        }
    }
}

fn bibliography_sources(data: &crate::ir::BlockElementData) -> Result<Vec<String>> {
    let Some(raw) = data.scalar("sources").filter(|value| !value.is_empty()) else {
        return Ok(Vec::new());
    };

    match serde_json::from_str::<serde_json::Value>(raw)
        .context_ut("cannot parse bibliography sources")?
    {
        serde_json::Value::String(source) => Ok(vec![source]),
        serde_json::Value::Array(sources) => sources
            .into_iter()
            .map(|source| match source {
                serde_json::Value::String(source) => Ok(source),
                other => bail!("bibliography source must be a string, got {other}"),
            })
            .collect(),
        other => bail!("bibliography sources must be a string or array, got {other}"),
    }
}

fn load_bibliography_sources(
    world: &dyn World,
    entry: FileId,
    sources: &[String],
) -> Result<Library> {
    let mut merged = Library::new();

    for source in sources {
        let source_id = entry.join(source.as_str());
        let text = world
            .source(source_id)
            .context_ut("cannot fetch bibliography source")?
            .text()
            .to_owned();
        let library = if source.ends_with(".bib") {
            from_biblatex_str(&text).map_err(|errors| {
                error_once!("cannot parse BibLaTeX bibliography", errors: format!("{errors:?}"))
            })?
        } else {
            from_yaml_str(&text).context_ut("cannot parse Hayagriva YAML bibliography")?
        };

        for entry in library.iter() {
            merged.push(entry);
        }
    }

    Ok(merged)
}

fn collect_cite_keys(blocks: &[Block], out: &mut Vec<String>) {
    for block in blocks {
        match block {
            Block::Heading { body, .. } | Block::Paragraph(body) => {
                collect_cite_keys_in_inlines(body, out);
            }
            Block::Quote(blocks) => collect_cite_keys(blocks, out),
            Block::Figure { body, caption, .. } => {
                collect_cite_keys(body, out);
                collect_cite_keys_in_inlines(caption, out);
            }
            Block::Align { body, .. }
            | Block::Block(crate::ir::BlockElementData { body, .. })
            | Block::Columns(crate::ir::BlockElementData { body, .. })
            | Block::Move(crate::ir::BlockElementData { body, .. })
            | Block::Pad(crate::ir::BlockElementData { body, .. })
            | Block::Rotate(crate::ir::BlockElementData { body, .. })
            | Block::Scale(crate::ir::BlockElementData { body, .. })
            | Block::Skew(crate::ir::BlockElementData { body, .. })
            | Block::Stack(crate::ir::BlockElementData { body, .. })
            | Block::Title(crate::ir::BlockElementData { body, .. }) => {
                collect_cite_keys(body, out)
            }
            Block::List { items, .. } => {
                for item in items {
                    collect_cite_keys(&item.body, out);
                }
            }
            Block::Terms { items } => {
                for item in items {
                    collect_cite_keys_in_inlines(&item.term, out);
                    collect_cite_keys(&item.description, out);
                }
            }
            _ => {}
        }
    }
}

fn collect_cite_keys_in_inlines(inlines: &[Inline], out: &mut Vec<String>) {
    for inline in inlines {
        if let Inline::Cite(data) = inline {
            if let Some(key) = data.scalar("key").or_else(|| data.scalar("label")) {
                let key = key.trim_start_matches('<').trim_end_matches('>').to_owned();
                if !out.contains(&key) {
                    out.push(key);
                }
            }
        }

        match inline {
            Inline::Emph(children)
            | Inline::Strong(children)
            | Inline::Strike(children)
            | Inline::Sub(children)
            | Inline::Super(children) => collect_cite_keys_in_inlines(children, out),
            Inline::Math(_)
            | Inline::Text(_)
            | Inline::Linebreak
            | Inline::Frame(_)
            | Inline::Raw { .. } => {}
            Inline::Link { body, .. } => collect_cite_keys_in_inlines(body, out),
            _ => {
                if let Some(children) = inline.generated_body() {
                    collect_cite_keys_in_inlines(children, out);
                }
                for field in inline_fields(inline) {
                    collect_cite_keys_in_field(field, out);
                }
            }
        }
    }
}

fn inline_fields(inline: &Inline) -> &[crate::ir::ElementField] {
    match inline {
        Inline::Box(data)
        | Inline::Circle(data)
        | Inline::Cite(data)
        | Inline::Curve(data)
        | Inline::CurveClose(data)
        | Inline::CurveCubic(data)
        | Inline::CurveLine(data)
        | Inline::CurveMove(data)
        | Inline::CurveQuad(data)
        | Inline::Document(data)
        | Inline::Ellipse(data)
        | Inline::FigureCaption(data)
        | Inline::Footnote(data)
        | Inline::FootnoteEntry(data)
        | Inline::GridCell(data)
        | Inline::GridFooter(data)
        | Inline::GridHeader(data)
        | Inline::GridHline(data)
        | Inline::GridVline(data)
        | Inline::H(data)
        | Inline::Hide(data)
        | Inline::Highlight(data)
        | Inline::Image(data)
        | Inline::Line(data)
        | Inline::MathAccent(data)
        | Inline::MathAttach(data)
        | Inline::MathBinom(data)
        | Inline::MathCancel(data)
        | Inline::MathCases(data)
        | Inline::MathClass(data)
        | Inline::MathFrac(data)
        | Inline::MathLimits(data)
        | Inline::MathLr(data)
        | Inline::MathMat(data)
        | Inline::MathMid(data)
        | Inline::MathOp(data)
        | Inline::MathOverbrace(data)
        | Inline::MathOverbracket(data)
        | Inline::MathOverline(data)
        | Inline::MathOverparen(data)
        | Inline::MathOvershell(data)
        | Inline::MathPrimes(data)
        | Inline::MathRoot(data)
        | Inline::MathScripts(data)
        | Inline::MathStretch(data)
        | Inline::MathUnderbrace(data)
        | Inline::MathUnderbracket(data)
        | Inline::MathUnderline(data)
        | Inline::MathUnderparen(data)
        | Inline::MathUndershell(data)
        | Inline::MathVec(data)
        | Inline::Metadata(data)
        | Inline::Move(data)
        | Inline::OutlineEntry(data)
        | Inline::Overline(data)
        | Inline::Pad(data)
        | Inline::Page(data)
        | Inline::ParLine(data)
        | Inline::Path(data)
        | Inline::PdfArtifact(data)
        | Inline::PdfAttach(data)
        | Inline::PdfEmbed(data)
        | Inline::Place(data)
        | Inline::PlaceFlush(data)
        | Inline::Polygon(data)
        | Inline::Quote(data)
        | Inline::RawLine(data)
        | Inline::Rect(data)
        | Inline::Ref(data)
        | Inline::Repeat(data)
        | Inline::Rotate(data)
        | Inline::Scale(data)
        | Inline::Skew(data)
        | Inline::Smallcaps(data)
        | Inline::Smartquote(data)
        | Inline::Square(data)
        | Inline::TableCell(data)
        | Inline::TableFooter(data)
        | Inline::TableHeader(data)
        | Inline::TableHline(data)
        | Inline::TableVline(data)
        | Inline::Underline(data) => &data.fields,
        _ => &[],
    }
}

fn collect_cite_keys_in_field(field: &crate::ir::ElementField, out: &mut Vec<String>) {
    match &field.value {
        ElementFieldValue::Inlines(inlines) => collect_cite_keys_in_inlines(inlines, out),
        ElementFieldValue::Blocks(blocks) => collect_cite_keys(blocks, out),
        ElementFieldValue::Scalar(_) => {}
    }
}

fn render_bibliography_entries(
    library: &Library,
    cited: &[String],
    style: &str,
) -> Result<BibliographyContext> {
    let Some(archived_style) = ArchivedStyle::by_name(style) else {
        bail!("unsupported bibliography style `{style}`");
    };
    let Style::Independent(style) = archived_style.get() else {
        bail!("bibliography style `{style}` must be independent");
    };
    let locales = locales();

    let mut driver = BibliographyDriver::new();
    for key in cited {
        let Some(entry) = library.get(key) else {
            bail!("bibliography entry `{key}` was cited but not found in bibliography sources");
        };
        driver.citation(CitationRequest::from_items(
            vec![CitationItem::with_entry(entry)],
            &style,
            &locales,
        ));
    }

    let rendered = driver.finish(BibliographyRequest::new(&style, None, &locales));
    let mut citations = Vec::new();
    for (key, citation) in cited.iter().zip(&rendered.citations) {
        let mut rendered = String::new();
        citation
            .citation
            .write_buf(&mut rendered, BufWriteFormat::Plain)
            .context_ut("cannot render bibliography citation")?;
        citations.push((key.as_str().into(), rendered.into()));
    }

    let Some(bibliography) = rendered.bibliography else {
        return Ok(BibliographyContext::default());
    };

    let mut entries = Vec::new();
    for item in bibliography.items {
        let mut rendered = String::new();
        item.content
            .write_buf(&mut rendered, BufWriteFormat::Plain)
            .context_ut("cannot render bibliography entry")?;
        entries.push((item.key.into(), rendered.into()));
    }

    Ok(BibliographyContext::new(entries, citations))
}

fn is_auto_or_none(value: &str) -> bool {
    value.is_empty() || value == "auto" || value == "none"
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
        let entry = self
            .world
            .entry_state()
            .main()
            .context("no main file in workspace")?;
        let (world, _) = self.feat.prepare_world(&self.world, self.format)?;
        let compiled = typst::compile::<typst_html::HtmlDocument>(&world);
        let html = compiled.output?;
        let ir = extract_document(&html)?;
        let bibliography = BibliographyContext::from_document(&ir, &world, entry)?;

        match self.format {
            Format::Md => Ok(render_markdown_with_bibliography(&ir, &bibliography)?.into()),
            Format::LaTeX => bail!("typlite LaTeX conversion is not implemented"),
            Format::Text => bail!("typlite text conversion is not implemented"),
            #[cfg(feature = "docx")]
            Format::Docx => bail!("typlite DOCX conversion is not implemented"),
        }
    }

    #[cfg(feature = "docx")]
    pub fn to_docx(self) -> Result<Vec<u8>> {
        let _ = (self.world, self.feat, self.format);
        bail!("typlite DOCX conversion is not implemented in the placeholder crate")
    }
}
