use crate::Result;
use crate::ir::*;

use super::{
    BibliographyContext, push_css_length, render_box, render_element_frame, render_frame_image,
    render_image, render_image_html, render_math, render_math_inline, render_move, render_pad,
    render_pdf_artifact, render_pdf_embedding, render_place, render_repeat, render_rotate,
    render_scale, render_skew,
};

mod escape;
mod kind;
mod quote;
mod reference;
pub(in crate::backend) use self::escape::{
    push_html_comment_escaped, push_html_escaped, push_markdown_link_text_escaped,
    push_markdown_url, push_url_escaped,
};
pub(in crate::backend) use self::kind::{
    render_curve_component_warning, render_unimplemented, render_unimplemented_inline,
};
pub(in crate::backend) use self::quote::{
    has_semantic_inlines, is_auto_inlines, is_none_inlines, render_inline_quote,
    render_inline_quote_html, render_metadata, render_smartquote, render_smartquote_html,
};
use self::reference::{render_cite, render_ref};

pub(super) fn render_inlines(
    nodes: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    for node in nodes {
        match node {
            Inline::Text(data) => out.push_str(&data.text),
            Inline::Emph(data) => {
                out.push('*');
                render_inlines(&data.body, bibliography, out)?;
                out.push('*');
            }
            Inline::Strong(data) => {
                out.push_str("**");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("**");
            }
            Inline::Link(data) => {
                out.push('[');
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("](");
                out.push_str(&data.dest);
                out.push(')');
            }
            Inline::Strike(data) => {
                out.push_str("~~");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("~~");
            }
            Inline::Sub(data) => {
                out.push_str("<sub>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(data) => {
                out.push_str("<sup>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Math(data) => {
                out.push('$');
                render_math(&data.body, out)?;
                out.push('$');
            }
            Inline::Linebreak(_) => out.push_str("  \n"),
            Inline::Frame(data) => render_frame_image("frame", &data.image, out)?,
            Inline::Raw(data) => {
                out.push('`');
                out.push_str(&data.text);
                out.push('`');
            }
            Inline::Repeat(data) => render_repeat(data, bibliography, out)?,
            Inline::TableCell(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::TableFooter(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::TableHeader(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::GridCell(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::GridFooter(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::GridHeader(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::ParLine(_) => {}
            Inline::RawLine(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::PdfArtifact(data) => render_pdf_artifact(data, bibliography, out)?,
            Inline::Box(data) => render_box(data, bibliography, out)?,
            Inline::Move(data) => render_move(data, bibliography, out)?,
            Inline::Pad(data) => render_pad(data, bibliography, out)?,
            Inline::Place(data) => render_place(data, bibliography, out)?,
            Inline::Rotate(data) => render_rotate(data, bibliography, out)?,
            Inline::Scale(data) => render_scale(data, bibliography, out)?,
            Inline::Skew(data) => render_skew(data, bibliography, out)?,
            Inline::Quote(data) => render_inline_quote(data, bibliography, out)?,
            Inline::Circle(data) => render_element_frame("circle", data.frame.as_ref(), out)?,
            Inline::Curve(data) => render_element_frame("curve", data.frame.as_ref(), out)?,
            Inline::Ellipse(data) => render_element_frame("ellipse", data.frame.as_ref(), out)?,
            Inline::Line(data) => render_element_frame("line", data.frame.as_ref(), out)?,
            Inline::Path(data) => render_element_frame("path", data.frame.as_ref(), out)?,
            Inline::Polygon(data) => render_element_frame("polygon", data.frame.as_ref(), out)?,
            Inline::Rect(data) => render_element_frame("rect", data.frame.as_ref(), out)?,
            Inline::Square(data) => render_element_frame("square", data.frame.as_ref(), out)?,
            Inline::Cite(data) => render_cite(data, bibliography, out)?,
            Inline::Metadata(data) => render_metadata(data, out),
            Inline::Document(_) | Inline::Hide(_) | Inline::Page(_) => {}
            Inline::FigureCaption(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::FootnoteEntry(_) => {}
            Inline::Footnote(data) => {
                out.push_str("^[");
                render_inlines(&data.body, bibliography, out)?;
                out.push(']');
            }
            Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::PlaceFlush(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
            Inline::H(_) => out.push(' '),
            Inline::Highlight(data) => {
                out.push_str("<mark>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</mark>");
            }
            Inline::Image(data) => render_image(data, out)?,
            Inline::MathAccent(_)
            | Inline::MathAttach(_)
            | Inline::MathBinom(_)
            | Inline::MathCancel(_)
            | Inline::MathCases(_)
            | Inline::MathClass(_)
            | Inline::MathFrac(_)
            | Inline::MathLimits(_)
            | Inline::MathLr(_)
            | Inline::MathMat(_)
            | Inline::MathMid(_)
            | Inline::MathOp(_)
            | Inline::MathOverbrace(_)
            | Inline::MathOverbracket(_)
            | Inline::MathOverline(_)
            | Inline::MathOverparen(_)
            | Inline::MathOvershell(_)
            | Inline::MathPrimes(_)
            | Inline::MathRoot(_)
            | Inline::MathScripts(_)
            | Inline::MathStretch(_)
            | Inline::MathUnderbrace(_)
            | Inline::MathUnderbracket(_)
            | Inline::MathUnderline(_)
            | Inline::MathUnderparen(_)
            | Inline::MathUndershell(_)
            | Inline::MathVec(_) => render_math_inline(node, true, out)?,
            Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_) => render_curve_component_warning(node, bibliography, out)?,
            Inline::OutlineEntry(_) => {}
            Inline::Overline(data) => {
                out.push_str("<span style=\"text-decoration: overline\">");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::PdfAttach(data) => render_pdf_embedding(data.path.as_deref(), out),
            Inline::PdfEmbed(data) => render_pdf_embedding(data.path.as_deref(), out),
            Inline::Ref(data) => render_ref(data, bibliography, out)?,
            Inline::Smallcaps(data) => {
                out.push_str("<span style=\"font-variant: small-caps\">");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::Smartquote(data) => render_smartquote(data, out)?,
            Inline::Underline(data) => {
                out.push_str("<u>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</u>");
            }
        }
    }

    Ok(())
}

pub(super) fn render_inlines_html(
    nodes: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    for node in nodes {
        match node {
            Inline::Text(data) => push_html_escaped(&data.text, out),
            Inline::Emph(data) => {
                out.push_str("<em>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</em>");
            }
            Inline::Strong(data) => {
                out.push_str("<strong>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</strong>");
            }
            Inline::Link(data) => {
                out.push_str("<a href=\"");
                push_html_escaped(&data.dest, out);
                out.push_str("\">");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</a>");
            }
            Inline::Strike(data) => {
                out.push_str("<del>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</del>");
            }
            Inline::Sub(data) => {
                out.push_str("<sub>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(data) => {
                out.push_str("<sup>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Raw(data) => {
                out.push_str("<code>");
                push_html_escaped(&data.text, out);
                out.push_str("</code>");
            }
            Inline::Linebreak(_) => out.push_str("<br>"),
            Inline::Math(data) => {
                out.push('$');
                render_math(&data.body, out)?;
                out.push('$');
            }
            Inline::Frame(data) => render_frame_image("frame", &data.image, out)?,
            Inline::Footnote(data) => {
                out.push_str("<sup>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Highlight(data) => {
                out.push_str("<mark>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</mark>");
            }
            Inline::Image(data) => render_image_html(data, out)?,
            Inline::Overline(data) => {
                out.push_str("<span style=\"text-decoration: overline\">");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::Smallcaps(data) => {
                out.push_str("<span style=\"font-variant: small-caps\">");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::Underline(data) => {
                out.push_str("<u>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</u>");
            }
            Inline::Repeat(data) => render_repeat(data, bibliography, out)?,
            Inline::TableCell(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::TableFooter(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::TableHeader(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::GridCell(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::GridFooter(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::GridHeader(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::ParLine(_) => {}
            Inline::RawLine(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::FigureCaption(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::OutlineEntry(_) => {}
            Inline::PdfArtifact(data) => render_pdf_artifact(data, bibliography, out)?,
            Inline::Box(data) => render_box(data, bibliography, out)?,
            Inline::Move(data) => render_move(data, bibliography, out)?,
            Inline::Pad(data) => render_pad(data, bibliography, out)?,
            Inline::Place(data) => render_place(data, bibliography, out)?,
            Inline::Rotate(data) => render_rotate(data, bibliography, out)?,
            Inline::Scale(data) => render_scale(data, bibliography, out)?,
            Inline::Skew(data) => render_skew(data, bibliography, out)?,
            Inline::Quote(data) => render_inline_quote_html(data, bibliography, out)?,
            Inline::FootnoteEntry(_) => {}
            Inline::Circle(data) => render_element_frame("circle", data.frame.as_ref(), out)?,
            Inline::Curve(data) => render_element_frame("curve", data.frame.as_ref(), out)?,
            Inline::Ellipse(data) => render_element_frame("ellipse", data.frame.as_ref(), out)?,
            Inline::Line(data) => render_element_frame("line", data.frame.as_ref(), out)?,
            Inline::Path(data) => render_element_frame("path", data.frame.as_ref(), out)?,
            Inline::Polygon(data) => render_element_frame("polygon", data.frame.as_ref(), out)?,
            Inline::Rect(data) => render_element_frame("rect", data.frame.as_ref(), out)?,
            Inline::Square(data) => render_element_frame("square", data.frame.as_ref(), out)?,
            Inline::Cite(data) => render_cite(data, bibliography, out)?,
            Inline::Metadata(data) => render_metadata(data, out),
            Inline::Document(_) | Inline::Hide(_) | Inline::Page(_) => {}
            Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::PlaceFlush(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
            Inline::H(_) => out.push(' '),
            Inline::PdfAttach(data) => render_pdf_embedding(data.path.as_deref(), out),
            Inline::PdfEmbed(data) => render_pdf_embedding(data.path.as_deref(), out),
            Inline::Ref(data) => render_ref(data, bibliography, out)?,
            Inline::Smartquote(data) => render_smartquote_html(data, out)?,
            Inline::MathAccent(_)
            | Inline::MathAttach(_)
            | Inline::MathBinom(_)
            | Inline::MathCancel(_)
            | Inline::MathCases(_)
            | Inline::MathClass(_)
            | Inline::MathFrac(_)
            | Inline::MathLimits(_)
            | Inline::MathLr(_)
            | Inline::MathMat(_)
            | Inline::MathMid(_)
            | Inline::MathOp(_)
            | Inline::MathOverbrace(_)
            | Inline::MathOverbracket(_)
            | Inline::MathOverline(_)
            | Inline::MathOverparen(_)
            | Inline::MathOvershell(_)
            | Inline::MathPrimes(_)
            | Inline::MathRoot(_)
            | Inline::MathScripts(_)
            | Inline::MathStretch(_)
            | Inline::MathUnderbrace(_)
            | Inline::MathUnderbracket(_)
            | Inline::MathUnderline(_)
            | Inline::MathUnderparen(_)
            | Inline::MathUndershell(_)
            | Inline::MathVec(_) => render_math_inline(node, true, out)?,
            Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_) => render_curve_component_warning(node, bibliography, out)?,
        }
    }

    Ok(())
}

pub(super) fn push_optional_css_length_value(
    value: Option<&str>,
    property: &str,
    out: &mut String,
) {
    if let Some(value) = value.filter(|value| !is_auto_or_none(value)) {
        out.push_str("; ");
        out.push_str(property);
        out.push_str(": ");
        push_css_length(value, out);
    }
}

pub(super) fn push_css_scale(value: &str, out: &mut String) {
    if let Some(percent) = value
        .strip_suffix('%')
        .and_then(|value| value.parse::<f64>().ok())
    {
        out.push_str(&(percent / 100.0).to_string());
    } else {
        push_html_escaped(value, out);
    }
}

pub(super) fn is_auto_or_none(value: &str) -> bool {
    value.is_empty() || value == "auto" || value == "none"
}
