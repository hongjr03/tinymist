use hayagriva::archive::{ArchivedStyle, locales};
use hayagriva::citationberg::Style;
use hayagriva::{
    BibliographyDriver, BibliographyRequest, BufWriteFormat, CitationItem, CitationRequest, Library,
};
use tinymist_std::error::prelude::*;

use crate::Result;
use crate::backend::BibliographyContext;

pub(super) fn render_bibliography_entries(
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
