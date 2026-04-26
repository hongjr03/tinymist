use std::path::Path;

use hayagriva::archive::{ArchivedStyle, locales};
use hayagriva::citationberg::{IndependentStyle, Style};
use hayagriva::{
    BibliographyDriver, BibliographyRequest, BufWriteFormat, CitationItem, CitationRequest, Library,
};
use tinymist_std::error::prelude::*;
use typst::World;
use typst_syntax::FileId;

use crate::Result;
use crate::backend::BibliographyContext;

pub(super) fn render_bibliography_entries(
    library: &Library,
    cited: &[String],
    style: &str,
    world: &dyn World,
    entry: FileId,
) -> Result<BibliographyContext> {
    let style = load_style(world, entry, style)?;
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

fn load_style(world: &dyn World, entry: FileId, style: &str) -> Result<IndependentStyle> {
    if Path::new(style).extension().is_none() {
        let Some(archived_style) = ArchivedStyle::by_name(style) else {
            bail!("unsupported bibliography style `{style}`");
        };
        let Style::Independent(style) = archived_style.get() else {
            bail!("bibliography style `{style}` must be independent");
        };
        return Ok(style);
    }

    let style_id = entry.join(style);
    let bytes = world
        .file(style_id)
        .context_ut("cannot fetch bibliography CSL style")?;
    let text = bytes
        .as_str()
        .context_ut("bibliography CSL style must be UTF-8")?;
    IndependentStyle::from_xml(text).context_ut("cannot parse bibliography CSL style")
}
