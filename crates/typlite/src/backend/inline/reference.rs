use crate::Result;
use crate::ir::*;

use super::{
    BibliographyContext, has_semantic_inlines, push_html_escaped, push_markdown_link_text_escaped,
    render_inlines, render_unimplemented,
};

pub(super) fn render_cite(
    data: &CiteInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(key) = data.key.as_deref() else {
        return render_unimplemented("cite without key");
    };

    let key = key.trim_start_matches('<').trim_end_matches('>');
    if let Some(citation) = bibliography.citation(key) {
        let id = bibliography.next_citation_id(key);
        render_citation_link(&id, key, citation, out);
        return Ok(());
    }

    out.push_str("[@");
    push_markdown_link_text_escaped(key, out);
    out.push_str("](#ref-");
    out.push_str(key);
    out.push(')');
    Ok(())
}

pub(super) fn render_ref(
    data: &RefInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(target) = data.target.as_deref() else {
        return render_unimplemented("ref without target");
    };

    let target = target.trim_start_matches('<').trim_end_matches('>');
    if let Some(citation) = bibliography.citation(target) {
        let id = bibliography.next_citation_id(target);
        render_citation_link(&id, target, citation, out);
        return Ok(());
    }

    if has_semantic_inlines(&data.supplement) {
        let supplement = &data.supplement;
        render_ref_link(target, supplement, bibliography, out)?;
        return Ok(());
    }

    if !data.element.is_empty() {
        render_ref_element_link(target, &data.element, bibliography, out)?;
        return Ok(());
    }

    render_ref_text_link(target, target, out)
}

pub(super) fn render_citation_link(id: &str, key: &str, citation: &str, out: &mut String) {
    out.push_str("<a id=\"");
    push_html_escaped(id, out);
    out.push_str("\" href=\"#ref-");
    push_html_escaped(key, out);
    out.push_str("\">");
    push_html_escaped(citation, out);
    out.push_str("</a>");
}

pub(super) fn render_ref_link(
    target: &str,
    body: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push('[');
    render_inlines(body, bibliography, out)?;
    out.push_str("](#");
    out.push_str(target);
    out.push(')');
    Ok(())
}

pub(super) fn render_ref_element_link(
    target: &str,
    blocks: &[Block],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    match blocks {
        [Block::Heading(data)] if has_semantic_inlines(&data.body) => {
            render_ref_link(target, &data.body, bibliography, out)
        }
        [Block::Figure(data)] if has_semantic_inlines(&data.caption) => {
            render_ref_link(target, &data.caption, bibliography, out)
        }
        [_] | [_, ..] => render_ref_text_link(target, target, out),
        [] => render_ref_text_link(target, target, out),
    }
}

pub(super) fn render_ref_text_link(target: &str, text: &str, out: &mut String) -> Result<()> {
    out.push('[');
    push_markdown_link_text_escaped(text, out);
    out.push_str("](#");
    out.push_str(target);
    out.push(')');
    Ok(())
}
