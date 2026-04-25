use hayagriva::archive::{ArchivedStyle, locales};
use hayagriva::citationberg::Style;
use hayagriva::io::{from_biblatex_str, from_yaml_str};
use hayagriva::{
    BibliographyDriver, BibliographyRequest, BufWriteFormat, CitationItem, CitationRequest, Library,
};
use tinymist_std::error::prelude::*;
use typst::World;
use typst_syntax::FileId;

use crate::Result;
use crate::backend::BibliographyContext;
use crate::ir::{BibliographyBlock, Block, Document, Inline};

impl BibliographyContext {
    pub(super) fn from_document(doc: &Document, world: &dyn World, entry: FileId) -> Result<Self> {
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
