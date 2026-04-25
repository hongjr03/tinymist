use hayagriva::Library;
use hayagriva::io::{from_biblatex_str, from_yaml_str};
use tinymist_std::error::prelude::*;
use typst::World;
use typst_syntax::FileId;

use crate::Result;
use crate::ir::{BibliographyBlock, Block};

pub(super) enum BibliographySource {
    Path(String),
    Text(String),
    String(String),
}

pub(super) fn collect_bibliography_blocks<'a>(
    blocks: &'a [Block],
    out: &mut Vec<&'a BibliographyBlock>,
) {
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

pub(super) fn bibliography_sources(data: &BibliographyBlock) -> Result<Vec<BibliographySource>> {
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

pub(super) fn load_bibliography_sources(
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
