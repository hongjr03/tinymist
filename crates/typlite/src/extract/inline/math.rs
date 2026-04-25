use typst::introspection::Introspector;
use typst_html::HtmlElement;

use crate::Result;
use crate::element_spec::ElementKind;
use crate::ir::*;

use super::super::{bool_field, inline_field, scalar_field};

pub(super) fn inline_from_element_kind_tail(
    element: &HtmlElement,
    kind: ElementKind,
    introspector: &Introspector,
) -> Result<Inline> {
    Ok(match kind {
        ElementKind::Page => Inline::Page(PageInline {
            paper: scalar_field(element, "paper"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            flipped: bool_field(element, "flipped"),
            margin: scalar_field(element, "margin"),
            binding: scalar_field(element, "binding"),
            columns: scalar_field(element, "columns"),
            fill: scalar_field(element, "fill"),
            numbering: scalar_field(element, "numbering"),
            supplement: inline_field(element, "supplement", introspector)?,
            number_align: scalar_field(element, "number-align"),
            header: inline_field(element, "header", introspector)?,
            header_ascent: scalar_field(element, "header-ascent"),
            footer: inline_field(element, "footer", introspector)?,
            footer_descent: scalar_field(element, "footer-descent"),
            background: inline_field(element, "background", introspector)?,
            foreground: inline_field(element, "foreground", introspector)?,
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::ParLine => Inline::ParLine(ParLineInline {
            numbering: scalar_field(element, "numbering"),
            number_align: scalar_field(element, "number-align"),
            number_margin: scalar_field(element, "number-margin"),
            number_clearance: scalar_field(element, "number-clearance"),
            numbering_scope: scalar_field(element, "numbering-scope"),
        }),
        ElementKind::MathAccent => Inline::MathAccent(MathAccentInline {
            base: scalar_field(element, "base"),
            accent: scalar_field(element, "accent"),
            size: scalar_field(element, "size"),
            dotless: bool_field(element, "dotless"),
        }),
        ElementKind::MathAttach => Inline::MathAttach(MathAttachInline {
            base: scalar_field(element, "base"),
            t: scalar_field(element, "t"),
            b: scalar_field(element, "b"),
            tl: scalar_field(element, "tl"),
            bl: scalar_field(element, "bl"),
            tr: scalar_field(element, "tr"),
            br: scalar_field(element, "br"),
        }),
        ElementKind::MathBinom => Inline::MathBinom(MathBinomInline {
            upper: scalar_field(element, "upper"),
            lower: scalar_field(element, "lower"),
        }),
        ElementKind::MathCancel => Inline::MathCancel(MathCancelInline {
            body: scalar_field(element, "body"),
            length: scalar_field(element, "length"),
            inverted: bool_field(element, "inverted"),
            cross: bool_field(element, "cross"),
            angle: scalar_field(element, "angle"),
            stroke: scalar_field(element, "stroke"),
        }),
        ElementKind::MathCases => Inline::MathCases(MathCasesInline {
            delim: scalar_field(element, "delim"),
            reverse: bool_field(element, "reverse"),
            gap: scalar_field(element, "gap"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::MathClass => Inline::MathClass(MathClassInline {
            class: scalar_field(element, "class"),
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathFrac => Inline::MathFrac(MathFracInline {
            num: scalar_field(element, "num"),
            denom: scalar_field(element, "denom"),
            style: scalar_field(element, "style"),
        }),
        ElementKind::MathLimits => Inline::MathLimits(MathLimitsInline {
            body: scalar_field(element, "body"),
            inline: bool_field(element, "inline"),
        }),
        ElementKind::MathLr => Inline::MathLr(MathLrInline {
            size: scalar_field(element, "size"),
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathMat => Inline::MathMat(MathMatInline {
            delim: scalar_field(element, "delim"),
            align: scalar_field(element, "align"),
            augment: scalar_field(element, "augment"),
            gap: scalar_field(element, "gap"),
            row_gap: scalar_field(element, "row-gap"),
            column_gap: scalar_field(element, "column-gap"),
            rows: scalar_field(element, "rows"),
        }),
        ElementKind::MathMid => Inline::MathMid(MathMidInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathOp => Inline::MathOp(MathOpInline {
            text: scalar_field(element, "text"),
            limits: bool_field(element, "limits"),
        }),
        ElementKind::MathOverbrace => Inline::MathOverbrace(MathOverbraceInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathOverbracket => Inline::MathOverbracket(MathOverbracketInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathOverline => Inline::MathOverline(MathOverlineInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathOverparen => Inline::MathOverparen(MathOverparenInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathOvershell => Inline::MathOvershell(MathOvershellInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathPrimes => Inline::MathPrimes(MathPrimesInline {
            count: scalar_field(element, "count"),
        }),
        ElementKind::MathRoot => Inline::MathRoot(MathRootInline {
            index: scalar_field(element, "index"),
            radicand: scalar_field(element, "radicand"),
        }),
        ElementKind::MathScripts => Inline::MathScripts(MathScriptsInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathStretch => Inline::MathStretch(MathStretchInline {
            body: scalar_field(element, "body"),
            size: scalar_field(element, "size"),
        }),
        ElementKind::MathUnderbrace => Inline::MathUnderbrace(MathUnderbraceInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathUnderbracket => Inline::MathUnderbracket(MathUnderbracketInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathUnderline => Inline::MathUnderline(MathUnderlineInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathUnderparen => Inline::MathUnderparen(MathUnderparenInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathUndershell => Inline::MathUndershell(MathUndershellInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathVec => Inline::MathVec(MathVecInline {
            delim: scalar_field(element, "delim"),
            align: scalar_field(element, "align"),
            gap: scalar_field(element, "gap"),
            children: inline_field(element, "children", introspector)?,
        }),
        _ => unreachable!("tail only receives covered inline element kinds"),
    })
}
