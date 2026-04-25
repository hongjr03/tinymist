use crate::Result;
use crate::ir::*;
use tinymist_std::error::prelude::*;
use typst::diag::SourceDiagnostic;
use typst_syntax::Span;

use super::{BibliographyContext, push_html_comment_escaped};

pub(in crate::backend) fn render_unimplemented_inline(node: &Inline) -> Result<()> {
    render_unimplemented(inline_kind(node)?)
}

pub(in crate::backend) fn render_curve_component_warning(
    node: &Inline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let kind = inline_kind(node)?;
    if let Some(span) = curve_component_span(node) {
        bibliography.warn(SourceDiagnostic::warning(
            span,
            format!(
                "typlite markdown rendering for `{kind}` requires wrapping the parent curve in html.frame"
            ),
        ));
    }
    out.push_str("<!-- typlite-warning: ");
    push_html_comment_escaped(kind, out);
    out.push_str(" requires wrapping the parent curve in html.frame -->");
    Ok(())
}

pub(in crate::backend) fn curve_component_span(node: &Inline) -> Option<Span> {
    match node {
        Inline::CurveClose(data) => data.span,
        Inline::CurveCubic(data) => data.span,
        Inline::CurveLine(data) => data.span,
        Inline::CurveMove(data) => data.span,
        Inline::CurveQuad(data) => data.span,
        _ => None,
    }
}

pub(in crate::backend) fn render_unimplemented(feature: &str) -> Result<()> {
    bail!("typlite markdown rendering for `{feature}` is not implemented")
}

pub(in crate::backend) fn inline_kind(node: &Inline) -> Result<&'static str> {
    let kind = match node {
        Inline::Box(_) => "box",
        Inline::Circle(_) => "circle",
        Inline::Cite(_) => "cite",
        Inline::Curve(_) => "curve",
        Inline::CurveClose(_) => "curve.close",
        Inline::CurveCubic(_) => "curve.cubic",
        Inline::CurveLine(_) => "curve.line",
        Inline::CurveMove(_) => "curve.move",
        Inline::CurveQuad(_) => "curve.quad",
        Inline::Document(_) => "document",
        Inline::Ellipse(_) => "ellipse",
        Inline::FigureCaption(_) => "figure.caption",
        Inline::Footnote(_) => "footnote",
        Inline::FootnoteEntry(_) => "footnote.entry",
        Inline::GridCell(_) => "grid.cell",
        Inline::GridFooter(_) => "grid.footer",
        Inline::GridHeader(_) => "grid.header",
        Inline::GridHline(_) => "grid.hline",
        Inline::GridVline(_) => "grid.vline",
        Inline::H(_) => "h",
        Inline::Hide(_) => "hide",
        Inline::Highlight(_) => "highlight",
        Inline::Image(_) => "image",
        Inline::Line(_) => "line",
        Inline::MathAccent(_) => "math.accent",
        Inline::MathAttach(_) => "math.attach",
        Inline::MathBinom(_) => "math.binom",
        Inline::MathCancel(_) => "math.cancel",
        Inline::MathCases(_) => "math.cases",
        Inline::MathClass(_) => "math.class",
        Inline::MathFrac(_) => "math.frac",
        Inline::MathLimits(_) => "math.limits",
        Inline::MathLr(_) => "math.lr",
        Inline::MathMat(_) => "math.mat",
        Inline::MathMid(_) => "math.mid",
        Inline::MathOp(_) => "math.op",
        Inline::MathOverbrace(_) => "math.overbrace",
        Inline::MathOverbracket(_) => "math.overbracket",
        Inline::MathOverline(_) => "math.overline",
        Inline::MathOverparen(_) => "math.overparen",
        Inline::MathOvershell(_) => "math.overshell",
        Inline::MathPrimes(_) => "math.primes",
        Inline::MathRoot(_) => "math.root",
        Inline::MathScripts(_) => "math.scripts",
        Inline::MathStretch(_) => "math.stretch",
        Inline::MathUnderbrace(_) => "math.underbrace",
        Inline::MathUnderbracket(_) => "math.underbracket",
        Inline::MathUnderline(_) => "math.underline",
        Inline::MathUnderparen(_) => "math.underparen",
        Inline::MathUndershell(_) => "math.undershell",
        Inline::MathVec(_) => "math.vec",
        Inline::Metadata(_) => "metadata",
        Inline::Move(_) => "move",
        Inline::OutlineEntry(_) => "outline.entry",
        Inline::Overline(_) => "overline",
        Inline::Pad(_) => "pad",
        Inline::Page(_) => "page",
        Inline::ParLine(_) => "par.line",
        Inline::Path(_) => "path",
        Inline::PdfArtifact(_) => "pdf.artifact",
        Inline::PdfAttach(_) => "pdf.attach",
        Inline::PdfEmbed(_) => "pdf.embed",
        Inline::Place(_) => "place",
        Inline::PlaceFlush(_) => "place.flush",
        Inline::Polygon(_) => "polygon",
        Inline::Quote(_) => "quote",
        Inline::RawLine(_) => "raw.line",
        Inline::Rect(_) => "rect",
        Inline::Ref(_) => "ref",
        Inline::Repeat(_) => "repeat",
        Inline::Rotate(_) => "rotate",
        Inline::Scale(_) => "scale",
        Inline::Skew(_) => "skew",
        Inline::Smallcaps(_) => "smallcaps",
        Inline::Smartquote(_) => "smartquote",
        Inline::Square(_) => "square",
        Inline::TableCell(_) => "table.cell",
        Inline::TableFooter(_) => "table.footer",
        Inline::TableHeader(_) => "table.header",
        Inline::TableHline(_) => "table.hline",
        Inline::TableVline(_) => "table.vline",
        Inline::Underline(_) => "underline",
        Inline::Text(_)
        | Inline::Emph(_)
        | Inline::Strong(_)
        | Inline::Link(_)
        | Inline::Strike(_)
        | Inline::Sub(_)
        | Inline::Super(_)
        | Inline::Math(_)
        | Inline::Linebreak(_)
        | Inline::Frame(_)
        | Inline::Raw(_) => bail!("not a generated inline element"),
    };

    Ok(kind)
}
