//! Experimental backends for typlite IR.

use crate::Result;
use crate::ir::*;
mod block;
mod context;
mod inline;
mod list;
mod math;
mod media;
mod table;
use self::block::render_blocks;
pub use self::context::{BibliographyContext, RenderedMarkdown};
use self::inline::{
    is_auto_inlines, is_auto_or_none, is_none_inlines, push_css_scale, push_html_comment_escaped,
    push_html_escaped, push_markdown_link_text_escaped, push_markdown_url,
    push_optional_css_length_value, push_url_escaped, render_inlines, render_inlines_html,
    render_unimplemented, render_unimplemented_inline,
};
use self::list::{render_list, render_terms};
use self::math::{render_math, render_math_inline};
use self::media::{
    render_box, render_element_frame, render_frame_image, render_image, render_image_html,
    render_move, render_pad, render_pdf_artifact, render_pdf_embedding, render_place,
    render_repeat, render_rotate, render_scale, render_skew,
};
use self::table::render_table;

/// Renders a document IR as Markdown.
pub fn render_markdown(doc: &Document) -> Result<String> {
    render_markdown_with_bibliography(doc, &BibliographyContext::default())
}

/// Renders a document IR as Markdown with a bibliography context.
pub fn render_markdown_with_bibliography(
    doc: &Document,
    bibliography: &BibliographyContext,
) -> Result<String> {
    Ok(render_markdown_with_diagnostics(doc, bibliography)?.output)
}

/// Renders a document IR as Markdown with diagnostics.
pub fn render_markdown_with_diagnostics(
    doc: &Document,
    bibliography: &BibliographyContext,
) -> Result<RenderedMarkdown> {
    bibliography.reset_render_state(doc);
    let output = render_blocks(&doc.blocks, 0, bibliography)?;
    let warnings = bibliography.take_warnings();
    Ok(RenderedMarkdown { output, warnings })
}

pub(super) fn push_css_length(value: &str, out: &mut String) {
    if value.contains('+') || value.contains('-') {
        out.push_str("calc(");
        push_html_escaped(value, out);
        out.push(')');
    } else {
        push_html_escaped(value, out);
    }
}
