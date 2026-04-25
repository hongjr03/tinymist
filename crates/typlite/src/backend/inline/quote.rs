use crate::Result;
use crate::ir::*;

use super::{BibliographyContext, push_html_comment_escaped, render_inlines, render_inlines_html};

pub(in crate::backend) fn render_inline_quote(
    data: &QuoteInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let quoted = match data.quotes.as_deref().unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        _ => true,
    };

    if quoted {
        out.push('"');
    }
    render_inlines(&data.body, bibliography, out)?;
    if quoted {
        out.push('"');
    }

    if has_semantic_inlines(&data.attribution) {
        out.push_str(" (");
        render_inlines(&data.attribution, bibliography, out)?;
        out.push(')');
    }

    Ok(())
}

pub(in crate::backend) fn render_inline_quote_html(
    data: &QuoteInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let quoted = match data.quotes.as_deref().unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        _ => true,
    };

    if quoted {
        out.push_str("<q>");
    }
    render_inlines_html(&data.body, bibliography, out)?;
    if quoted {
        out.push_str("</q>");
    }

    if has_semantic_inlines(&data.attribution) {
        out.push_str(" <cite>");
        render_inlines_html(&data.attribution, bibliography, out)?;
        out.push_str("</cite>");
    }

    Ok(())
}

pub(in crate::backend) fn render_metadata(data: &MetadataInline, out: &mut String) {
    let Some(value) = data.value.as_deref().filter(|value| !value.is_empty()) else {
        return;
    };

    out.push_str("<!-- typlite-metadata: ");
    push_html_comment_escaped(value, out);
    out.push_str(" -->");
}

pub(in crate::backend) fn render_smartquote(
    data: &SmartquoteInline,
    out: &mut String,
) -> Result<()> {
    out.push(smartquote_char(data)?);
    Ok(())
}

pub(in crate::backend) fn render_smartquote_html(
    data: &SmartquoteInline,
    out: &mut String,
) -> Result<()> {
    match smartquote_char(data)? {
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        _ => unreachable!(),
    }
    Ok(())
}

pub(in crate::backend) fn smartquote_char(data: &SmartquoteInline) -> Result<char> {
    match data.double.as_deref().unwrap_or("true") {
        "true" => Ok('"'),
        "false" => Ok('\''),
        _ => Ok('"'),
    }
}

pub(in crate::backend) fn has_semantic_inlines(value: &[Inline]) -> bool {
    !value.is_empty() && !is_auto_inlines(value) && !is_none_inlines(value)
}

pub(in crate::backend) fn is_auto_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(data)] if data.text.as_str() == "auto")
}

pub(in crate::backend) fn is_none_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(data)] if data.text.as_str() == "none")
}
