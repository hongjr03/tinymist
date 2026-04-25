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
use typst::Library as TypstLibrary;
use typst::World;
use typst::WorldExt;
use typst::comemo::Tracked;
use typst::diag::SourceDiagnostic;
use typst::foundations::{
    Binding, Bytes, Content, Context, Func, NativeFunc, Repr, SequenceElem, Str, StyleChain,
    StyledElem, SymbolElem, Target, TargetElem, Value, func,
};
use typst::math::EquationElem;
use typst::text::TextElem;
use typst::utils::LazyHash;
use typst_syntax::VirtualPath;
use typst_syntax::package::PackageSpec;
use typst_syntax::{FileId, Span};

use crate::backend::{BibliographyContext, render_markdown_with_diagnostics};
use crate::extract::extract_document;
use crate::ir::{BibliographyBlock, Block, Document, Inline};

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

fn typlite_library(library: &Arc<LazyHash<TypstLibrary>>) -> Arc<LazyHash<TypstLibrary>> {
    let mut library = library.as_ref().clone();
    let scope = library.global.scope_mut();
    scope.define_func::<__typlite_encode_content>();
    scope.bind(
        "target".into(),
        Binding::detached(Func::from(__typlite_target::data())),
    );
    if let Some(Value::Module(mut std)) = scope.get("std").map(|binding| binding.read().clone()) {
        std.scope_mut().bind(
            "target".into(),
            Binding::detached(Func::from(__typlite_target::data())),
        );
        scope.bind("std".into(), Binding::detached(std));
    }
    if let Value::Module(mut std) = library.std.read().clone() {
        std.scope_mut().bind(
            "target".into(),
            Binding::detached(Func::from(__typlite_target::data())),
        );
        library.std = Binding::detached(std);
    }
    Arc::new(library)
}

#[func(name = "__typlite_encode_content", title = "Typlite content encoder")]
fn __typlite_encode_content(body: Content) -> Str {
    Str::from(serde_json::to_string(&encode_content(&body)).unwrap_or_else(|_| "{}".to_owned()))
}

#[func(contextual, name = "target", title = "Typlite target")]
fn __typlite_target(context: Tracked<Context>) -> typst::diag::HintedStrResult<Str> {
    let target = context.styles()?.get(TargetElem::target);
    Ok(match target {
        Target::Html => Str::from("typlite"),
        Target::Paged => Str::from("paged"),
    })
}

fn encode_content(body: &Content) -> serde_json::Value {
    if let Some(styled) = body.to_packed::<StyledElem>() {
        let styles = StyleChain::new(&styled.styles);
        let mut object = serde_json::Map::new();
        object.insert("func".into(), "styled".into());
        object.insert("child".into(), encode_content(&styled.child));
        object.insert("bold".into(), styles.get(EquationElem::bold).into());
        object.insert("cramped".into(), styles.get(EquationElem::cramped).into());
        object.insert(
            "italic".into(),
            encode_optional_bool(styles.get(EquationElem::italic)),
        );
        object.insert(
            "size".into(),
            encode_math_size(styles.get(EquationElem::size)),
        );
        object.insert(
            "variant".into(),
            encode_math_variant(styles.get(EquationElem::variant)),
        );
        object.insert(
            "text_fill".into(),
            styles.get_cloned(TextElem::fill).repr().as_str().into(),
        );
        object.insert(
            "text_size".into(),
            format!("{:?}", styles.get(TextElem::size)).into(),
        );
        object.insert(
            "text_style".into(),
            format!("{:?}", styles.get(TextElem::style)).into(),
        );
        object.insert(
            "text_weight".into(),
            format!("{:?}", styles.get(TextElem::weight)).into(),
        );
        return serde_json::Value::Object(object);
    }

    if let Some(sequence) = body.to_packed::<SequenceElem>() {
        return serde_json::json!({
            "func": "sequence",
            "children": sequence.children.iter().map(encode_content).collect::<Vec<_>>(),
        });
    }

    if let Some(equation) = body.to_packed::<EquationElem>() {
        return serde_json::json!({
            "func": "equation",
            "block": equation.block.get(StyleChain::default()),
            "body": encode_content(&equation.body),
        });
    }

    if let Some(text) = body.to_packed::<TextElem>() {
        return serde_json::json!({
            "func": "text",
            "text": text.text.as_str(),
        });
    }

    if let Some(symbol) = body.to_packed::<SymbolElem>() {
        return serde_json::json!({
            "func": "symbol",
            "text": symbol.text.as_str(),
        });
    }

    encode_content_fields(body)
}

fn encode_content_fields(body: &Content) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert("func".into(), body.elem().name().into());
    for (name, value) in body.fields().iter() {
        object.insert(name.as_str().into(), encode_value(value));
    }
    serde_json::Value::Object(object)
}

fn encode_value(value: &Value) -> serde_json::Value {
    match value {
        Value::None => serde_json::Value::Null,
        Value::Auto => "auto".into(),
        Value::Bool(value) => (*value).into(),
        Value::Int(value) => (*value).into(),
        Value::Float(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| value.repr().as_str().into()),
        Value::Str(value) => value.as_str().into(),
        Value::Content(value) => encode_content(value),
        Value::Array(value) => value.iter().map(encode_value).collect(),
        Value::Dict(value) => value
            .iter()
            .map(|(key, value)| (key.as_str().to_owned(), encode_value(value)))
            .collect(),
        _ => value.repr().as_str().into(),
    }
}

fn encode_optional_bool(value: Option<bool>) -> serde_json::Value {
    match value {
        Some(value) => value.to_string().into(),
        None => serde_json::Value::Null,
    }
}

fn encode_math_size(value: impl std::fmt::Debug) -> serde_json::Value {
    match format!("{value:?}").as_str() {
        "Display" => "display".into(),
        "Text" => "text".into(),
        "Script" => "script".into(),
        "ScriptScript" => "script-script".into(),
        _ => serde_json::Value::Null,
    }
}

fn encode_math_variant(value: impl std::fmt::Debug) -> serde_json::Value {
    match format!("{value:?}").as_str() {
        "Some(Plain)" => "plain".into(),
        "Some(SansSerif)" => "sans-serif".into(),
        "Some(Chancery)" => "chancery".into(),
        "Some(Roundhand)" => "roundhand".into(),
        "Some(Fraktur)" => "fraktur".into(),
        "Some(Monospace)" => "monospace".into(),
        "Some(DoubleStruck)" => "double-struck".into(),
        _ => serde_json::Value::Null,
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
                    .style
                    .as_deref()
                    .filter(|value| !is_auto_or_none(value))
                    .map(str::to_owned);
            }
            full |= bibliography.full;
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

enum BibliographySource {
    Path(String),
    Text(String),
    String(String),
}

fn collect_bibliography_blocks<'a>(blocks: &'a [Block], out: &mut Vec<&'a BibliographyBlock>) {
    for block in blocks {
        match block {
            Block::Bibliography(data) => out.push(data),
            Block::Quote(data) => collect_bibliography_blocks(&data.body, out),
            Block::Figure(data) => collect_bibliography_blocks(&data.body, out),
            Block::Align(data) => collect_bibliography_blocks(&data.body, out),
            Block::Block(data) => collect_bibliography_blocks(&data.body, out),
            Block::Columns(data) => collect_bibliography_blocks(&data.body, out),
            Block::Move(data) => collect_bibliography_blocks(&data.body, out),
            Block::Pad(data) => collect_bibliography_blocks(&data.body, out),
            Block::Rotate(data) => collect_bibliography_blocks(&data.body, out),
            Block::Scale(data) => collect_bibliography_blocks(&data.body, out),
            Block::Skew(data) => collect_bibliography_blocks(&data.body, out),
            Block::Stack(data) => collect_bibliography_blocks(&data.children, out),
            Block::Title(data) => collect_bibliography_blocks(&data.body, out),
            Block::List(data) => {
                for item in &data.items {
                    collect_bibliography_blocks(&item.body, out);
                }
            }
            Block::Terms(data) => {
                for item in &data.items {
                    collect_bibliography_blocks(&item.description, out);
                }
            }
            _ => {}
        }
    }
}

fn bibliography_sources(data: &BibliographyBlock) -> Result<Vec<BibliographySource>> {
    let Some(raw) = data.sources.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(Vec::new());
    };

    match serde_json::from_str::<serde_json::Value>(raw)
        .context_ut("cannot parse bibliography sources")?
    {
        serde_json::Value::String(source) => Ok(vec![BibliographySource::String(source)]),
        serde_json::Value::Array(sources) => sources.into_iter().map(bibliography_source).collect(),
        serde_json::Value::Object(_) => Ok(vec![bibliography_source(
            serde_json::from_str(raw).context_ut("cannot parse bibliography source")?,
        )?]),
        other => bail!("bibliography sources must be a string or array, got {other}"),
    }
}

fn bibliography_source(source: serde_json::Value) -> Result<BibliographySource> {
    match source {
        serde_json::Value::String(source) => Ok(BibliographySource::String(source)),
        serde_json::Value::Object(mut source) => match source.remove("kind") {
            Some(serde_json::Value::String(kind)) if kind == "string" => {
                let Some(serde_json::Value::String(value)) = source.remove("value") else {
                    bail!("bibliography string source must contain string field `value`");
                };
                Ok(BibliographySource::String(value))
            }
            Some(serde_json::Value::String(kind)) if kind == "path" => {
                let Some(serde_json::Value::String(path)) = source.remove("path") else {
                    bail!("bibliography path source must contain string field `path`");
                };
                Ok(BibliographySource::Path(path))
            }
            Some(serde_json::Value::String(kind)) if kind == "bytes" => {
                let text = if let Some(serde_json::Value::String(text)) = source.remove("text") {
                    text
                } else if let Some(bytes) = source.remove("bytes") {
                    decode_bibliography_bytes(bytes)?
                } else {
                    bail!("bibliography bytes source must contain field `bytes`");
                };
                Ok(BibliographySource::Text(text))
            }
            Some(kind) => bail!("unsupported bibliography source kind {kind}"),
            None => bail!("bibliography source object must contain field `kind`"),
        },
        other => bail!("bibliography source must be a string or object, got {other}"),
    }
}

fn decode_bibliography_bytes(bytes: serde_json::Value) -> Result<String> {
    let serde_json::Value::Array(values) = bytes else {
        bail!("bibliography bytes source field `bytes` must be an array");
    };
    let mut bytes = Vec::with_capacity(values.len());
    for value in values {
        let Some(byte) = value.as_u64().and_then(|value| u8::try_from(value).ok()) else {
            bail!("bibliography bytes source contains non-byte value {value}");
        };
        bytes.push(byte);
    }
    String::from_utf8(bytes).context_ut("bibliography bytes source must be UTF-8")
}

fn load_bibliography_sources(
    world: &dyn World,
    entry: FileId,
    sources: &[BibliographySource],
) -> Result<Library> {
    let mut merged = Library::new();

    for source in sources {
        let library = match source {
            BibliographySource::Path(source) => {
                load_bibliography_path_source(world, entry, source)?
            }
            BibliographySource::Text(text) => parse_bibliography_source(text, None)?,
            BibliographySource::String(source) => {
                load_ambiguous_bibliography_source(world, entry, source)?
            }
        };

        for entry in library.iter() {
            merged.push(entry);
        }
    }

    Ok(merged)
}

fn load_bibliography_path_source(
    world: &dyn World,
    entry: FileId,
    source: &str,
) -> Result<Library> {
    let source_id = entry.join(source);
    let text = world
        .source(source_id)
        .context_ut("cannot fetch bibliography source")?
        .text()
        .to_owned();
    parse_bibliography_source(&text, Some(source))
}

fn load_ambiguous_bibliography_source(
    world: &dyn World,
    entry: FileId,
    source: &str,
) -> Result<Library> {
    // Typst's bibliography accepts both path-like strings and bibliography
    // payload strings. They are indistinguishable after extraction, so this is
    // the one intentional fallback in this path: try the source as a path first,
    // then as inline bibliography content.
    match load_bibliography_path_source(world, entry, source) {
        Ok(library) => Ok(library),
        Err(path_error) => parse_bibliography_source(source, None).map_err(|inline_error| {
            error_once!(
                "cannot parse bibliography source as path or inline content",
                path_error: path_error.to_string(),
                inline_error: inline_error.to_string(),
            )
        }),
    }
}

fn parse_bibliography_source(text: &str, path: Option<&str>) -> Result<Library> {
    let is_biblatex =
        path.is_some_and(|path| path.ends_with(".bib")) || text.trim_start().starts_with('@');
    if is_biblatex {
        from_biblatex_str(text).map_err(|errors| {
            error_once!("cannot parse BibLaTeX bibliography", errors: format!("{errors:?}"))
        })
    } else {
        from_yaml_str(text).context_ut("cannot parse Hayagriva YAML bibliography")
    }
}

fn collect_cite_keys(blocks: &[Block], out: &mut Vec<String>) {
    for block in blocks {
        match block {
            Block::Heading(data) => collect_cite_keys_in_inlines(&data.body, out),
            Block::Paragraph(data) => collect_cite_keys_in_inlines(&data.body, out),
            Block::Quote(data) => collect_cite_keys(&data.body, out),
            Block::Figure(data) => {
                collect_cite_keys(&data.body, out);
                collect_cite_keys_in_inlines(&data.caption, out);
            }
            Block::Align(data) => collect_cite_keys(&data.body, out),
            Block::Block(data) => collect_cite_keys(&data.body, out),
            Block::Columns(data) => collect_cite_keys(&data.body, out),
            Block::Move(data) => collect_cite_keys(&data.body, out),
            Block::Pad(data) => collect_cite_keys(&data.body, out),
            Block::Rotate(data) => collect_cite_keys(&data.body, out),
            Block::Scale(data) => collect_cite_keys(&data.body, out),
            Block::Skew(data) => collect_cite_keys(&data.body, out),
            Block::Stack(data) => collect_cite_keys(&data.children, out),
            Block::Title(data) => collect_cite_keys(&data.body, out),
            Block::List(data) => {
                for item in &data.items {
                    collect_cite_keys(&item.body, out);
                }
            }
            Block::Terms(data) => {
                for item in &data.items {
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
            if let Some(key) = data.key.as_deref() {
                let key = key.trim_start_matches('<').trim_end_matches('>').to_owned();
                if !out.contains(&key) {
                    out.push(key);
                }
            }
        }

        match inline {
            Inline::Emph(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Strong(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Strike(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Sub(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Super(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Math(_)
            | Inline::Text(_)
            | Inline::Linebreak(_)
            | Inline::Frame(_)
            | Inline::Raw(_) => {}
            Inline::Link(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Box(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Circle(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Curve(data) => collect_cite_keys_in_inlines(&data.components, out),
            Inline::Ellipse(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::FigureCaption(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Footnote(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::GridCell(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::GridFooter(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::GridHeader(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::Hide(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Highlight(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::MathCases(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::MathVec(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::Move(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Overline(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Pad(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Page(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::PdfArtifact(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Place(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Quote(data) => {
                collect_cite_keys_in_inlines(&data.attribution, out);
                collect_cite_keys_in_inlines(&data.body, out);
            }
            Inline::RawLine(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Ref(data) => {
                collect_cite_keys_in_inlines(&data.supplement, out);
                collect_cite_keys_in_inlines(&data.citation, out);
                collect_cite_keys(&data.element, out);
            }
            Inline::Repeat(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Rotate(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Scale(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Skew(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Smallcaps(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::TableCell(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::TableFooter(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::TableHeader(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::Underline(data) => collect_cite_keys_in_inlines(&data.body, out),
            _ => {}
        }
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
