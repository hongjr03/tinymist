use typst::World;
use typst_syntax::FileId;

use crate::Result;
use crate::backend::BibliographyContext;
use crate::ir::Document;

mod cite;
mod render;
mod source;
use self::cite::collect_cite_keys;
use self::render::render_bibliography_entries;
use self::source::{bibliography_sources, collect_bibliography_blocks, load_bibliography_sources};

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

fn is_auto_or_none(value: &str) -> bool {
    value.is_empty() || value == "auto" || value == "none"
}
