use typst::introspection::Introspector;
use typst_html::HtmlElement;

use crate::Result;
use crate::element_spec::ElementKind;
use crate::ir::*;

use super::{
    block_field, bool_field, collect_inlines, collect_item_blocks, frame_field, grid_cell_inline,
    grid_hline_inline, grid_vline_inline, inline_field, inline_has_content, plain_text_blocks,
    scalar_field, source_field, source_span, table_cell_inline, table_hline_inline,
    table_vline_inline,
};

pub(super) fn inline_from_element_kind(
    element: &HtmlElement,
    kind: ElementKind,
    introspector: &Introspector,
) -> Result<Option<Inline>> {
    let inline = match kind {
        ElementKind::Box => Inline::Box(BoxInline {
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            baseline: scalar_field(element, "baseline"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            radius: scalar_field(element, "radius"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            clip: scalar_field(element, "clip"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Circle => Inline::Circle(CircleInline {
            radius: scalar_field(element, "radius"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Cite => Inline::Cite(CiteInline {
            key: scalar_field(element, "key"),
            supplement: inline_field(element, "supplement", introspector)?,
            form: scalar_field(element, "form"),
            style: scalar_field(element, "style"),
        }),
        ElementKind::Curve => Inline::Curve(CurveInline {
            fill: scalar_field(element, "fill"),
            fill_rule: scalar_field(element, "fill-rule"),
            stroke: scalar_field(element, "stroke"),
            components: inline_field(element, "components", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::CurveClose => Inline::CurveClose(CurveCloseInline {
            mode: scalar_field(element, "mode"),
            span: source_span(element),
        }),
        ElementKind::CurveCubic => Inline::CurveCubic(CurveCubicInline {
            control_start: scalar_field(element, "control-start"),
            control_end: scalar_field(element, "control-end"),
            end: scalar_field(element, "end"),
            relative: bool_field(element, "relative"),
            span: source_span(element),
        }),
        ElementKind::CurveLine => Inline::CurveLine(CurveLineInline {
            end: scalar_field(element, "end"),
            relative: bool_field(element, "relative"),
            span: source_span(element),
        }),
        ElementKind::CurveMove => Inline::CurveMove(CurveMoveInline {
            start: scalar_field(element, "start"),
            relative: bool_field(element, "relative"),
            span: source_span(element),
        }),
        ElementKind::CurveQuad => Inline::CurveQuad(CurveQuadInline {
            control: scalar_field(element, "control"),
            end: scalar_field(element, "end"),
            relative: bool_field(element, "relative"),
            span: source_span(element),
        }),
        ElementKind::Document => Inline::Document(DocumentInline {
            title: scalar_field(element, "title"),
            author: scalar_field(element, "author"),
            description: scalar_field(element, "description"),
            keywords: scalar_field(element, "keywords"),
            date: scalar_field(element, "date"),
        }),
        ElementKind::Ellipse => Inline::Ellipse(EllipseInline {
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::FigureCaption => Inline::FigureCaption(FigureCaptionInline {
            position: scalar_field(element, "position"),
            separator: scalar_field(element, "separator"),
            body: inline_field(element, "body", introspector)?,
            kind: scalar_field(element, "kind"),
            supplement: inline_field(element, "supplement", introspector)?,
            numbering: scalar_field(element, "numbering"),
            counter: scalar_field(element, "counter"),
        }),
        ElementKind::Footnote => Inline::Footnote(FootnoteInline {
            numbering: scalar_field(element, "numbering"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::FootnoteEntry => Inline::FootnoteEntry(FootnoteEntryInline {
            note: inline_field(element, "note", introspector)?,
            separator: inline_field(element, "separator", introspector)?,
            clearance: scalar_field(element, "clearance"),
            gap: scalar_field(element, "gap"),
            indent: scalar_field(element, "indent"),
        }),
        ElementKind::GridCell => Inline::GridCell(grid_cell_inline(element, introspector)?),
        ElementKind::GridFooter => Inline::GridFooter(GridFooterInline {
            repeat: bool_field(element, "repeat"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::GridHeader => Inline::GridHeader(GridHeaderInline {
            repeat: bool_field(element, "repeat"),
            level: scalar_field(element, "level"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::GridHline => Inline::GridHline(grid_hline_inline(element)),
        ElementKind::GridVline => Inline::GridVline(grid_vline_inline(element)),
        ElementKind::H => Inline::H(HInline {
            amount: scalar_field(element, "amount"),
            weak: bool_field(element, "weak"),
        }),
        ElementKind::Hide => Inline::Hide(HideInline {
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Highlight => Inline::Highlight(HighlightInline {
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            top_edge: scalar_field(element, "top-edge"),
            bottom_edge: scalar_field(element, "bottom-edge"),
            extent: scalar_field(element, "extent"),
            radius: scalar_field(element, "radius"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Image => Inline::Image(ImageInline {
            source: source_field(element, "source"),
            format: scalar_field(element, "format"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            alt: scalar_field(element, "alt"),
            page: scalar_field(element, "page"),
            fit: scalar_field(element, "fit"),
            scaling: scalar_field(element, "scaling"),
            icc: scalar_field(element, "icc"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Line => Inline::Line(LineInline {
            start: scalar_field(element, "start"),
            end: scalar_field(element, "end"),
            length: scalar_field(element, "length"),
            angle: scalar_field(element, "angle"),
            stroke: scalar_field(element, "stroke"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Metadata => Inline::Metadata(MetadataInline {
            value: scalar_field(element, "value"),
        }),
        ElementKind::Move => Inline::Move(MoveInline {
            dx: scalar_field(element, "dx"),
            dy: scalar_field(element, "dy"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::OutlineEntry => Inline::OutlineEntry(OutlineEntryInline {
            level: scalar_field(element, "level"),
            element: block_field(element, "element", introspector)?,
            fill: scalar_field(element, "fill"),
        }),
        ElementKind::Overline => Inline::Overline(OverlineInline {
            stroke: scalar_field(element, "stroke"),
            offset: scalar_field(element, "offset"),
            extent: scalar_field(element, "extent"),
            evade: scalar_field(element, "evade"),
            background: scalar_field(element, "background"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Pad => Inline::Pad(PadInline {
            left: scalar_field(element, "left"),
            top: scalar_field(element, "top"),
            right: scalar_field(element, "right"),
            bottom: scalar_field(element, "bottom"),
            x: scalar_field(element, "x"),
            y: scalar_field(element, "y"),
            rest: scalar_field(element, "rest"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Path => Inline::Path(PathInline {
            fill: scalar_field(element, "fill"),
            fill_rule: scalar_field(element, "fill-rule"),
            stroke: scalar_field(element, "stroke"),
            closed: bool_field(element, "closed"),
            vertices: scalar_field(element, "vertices"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::PdfArtifact => Inline::PdfArtifact(PdfArtifactInline {
            kind: scalar_field(element, "kind"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::PdfAttach => Inline::PdfAttach(PdfAttachInline {
            path: scalar_field(element, "path"),
            data: scalar_field(element, "data"),
            relationship: scalar_field(element, "relationship"),
            mime_type: scalar_field(element, "mime-type"),
            description: scalar_field(element, "description"),
        }),
        ElementKind::PdfEmbed => Inline::PdfEmbed(PdfEmbedInline {
            path: scalar_field(element, "path"),
            data: scalar_field(element, "data"),
            relationship: scalar_field(element, "relationship"),
            mime_type: scalar_field(element, "mime-type"),
            description: scalar_field(element, "description"),
        }),
        ElementKind::Place => Inline::Place(PlaceInline {
            alignment: scalar_field(element, "alignment"),
            scope: scalar_field(element, "scope"),
            float: scalar_field(element, "float"),
            clearance: scalar_field(element, "clearance"),
            dx: scalar_field(element, "dx"),
            dy: scalar_field(element, "dy"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::PlaceFlush => Inline::PlaceFlush(PlaceFlushInline {}),
        ElementKind::Polygon => Inline::Polygon(PolygonInline {
            fill: scalar_field(element, "fill"),
            fill_rule: scalar_field(element, "fill-rule"),
            stroke: scalar_field(element, "stroke"),
            vertices: scalar_field(element, "vertices"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Quote => Inline::Quote(QuoteInline {
            block: bool_field(element, "block"),
            quotes: scalar_field(element, "quotes"),
            attribution: inline_field(element, "attribution", introspector)?,
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::RawLine => Inline::RawLine(RawLineInline {
            number: scalar_field(element, "number"),
            count: scalar_field(element, "count"),
            text: scalar_field(element, "text"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Rect => Inline::Rect(RectInline {
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            radius: scalar_field(element, "radius"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Ref => Inline::Ref(RefInline {
            target: scalar_field(element, "target"),
            supplement: inline_field(element, "supplement", introspector)?,
            form: scalar_field(element, "form"),
            citation: inline_field(element, "citation", introspector)?,
            element: block_field(element, "element", introspector)?,
        }),
        ElementKind::Repeat => Inline::Repeat(RepeatInline {
            body: inline_field(element, "body", introspector)?,
            gap: scalar_field(element, "gap"),
            justify: scalar_field(element, "justify"),
        }),
        ElementKind::Rotate => Inline::Rotate(RotateInline {
            angle: scalar_field(element, "angle"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Scale => Inline::Scale(ScaleInline {
            factor: scalar_field(element, "factor"),
            x: scalar_field(element, "x"),
            y: scalar_field(element, "y"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Skew => Inline::Skew(SkewInline {
            ax: scalar_field(element, "ax"),
            ay: scalar_field(element, "ay"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Smallcaps => Inline::Smallcaps(SmallcapsInline {
            all: bool_field(element, "all"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Smartquote => Inline::Smartquote(SmartquoteInline {
            double: scalar_field(element, "double"),
            enabled: scalar_field(element, "enabled"),
            alternative: scalar_field(element, "alternative"),
            quotes: scalar_field(element, "quotes"),
        }),
        ElementKind::Square => Inline::Square(SquareInline {
            size: scalar_field(element, "size"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            radius: scalar_field(element, "radius"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::TableCell => Inline::TableCell(table_cell_inline(element, introspector)?),
        ElementKind::TableFooter => Inline::TableFooter(TableFooterInline {
            repeat: bool_field(element, "repeat"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::TableHeader => Inline::TableHeader(TableHeaderInline {
            repeat: bool_field(element, "repeat"),
            level: scalar_field(element, "level"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::TableHline => Inline::TableHline(table_hline_inline(element)),
        ElementKind::TableVline => Inline::TableVline(table_vline_inline(element)),
        ElementKind::Underline => Inline::Underline(UnderlineInline {
            stroke: scalar_field(element, "stroke"),
            offset: scalar_field(element, "offset"),
            extent: scalar_field(element, "extent"),
            evade: scalar_field(element, "evade"),
            background: scalar_field(element, "background"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Page
        | ElementKind::ParLine
        | ElementKind::MathAccent
        | ElementKind::MathAttach
        | ElementKind::MathBinom
        | ElementKind::MathCancel
        | ElementKind::MathCases
        | ElementKind::MathClass
        | ElementKind::MathFrac
        | ElementKind::MathLimits
        | ElementKind::MathLr
        | ElementKind::MathMat
        | ElementKind::MathMid
        | ElementKind::MathOp
        | ElementKind::MathOverbrace
        | ElementKind::MathOverbracket
        | ElementKind::MathOverline
        | ElementKind::MathOverparen
        | ElementKind::MathOvershell
        | ElementKind::MathPrimes
        | ElementKind::MathRoot
        | ElementKind::MathScripts
        | ElementKind::MathStretch
        | ElementKind::MathUnderbrace
        | ElementKind::MathUnderbracket
        | ElementKind::MathUnderline
        | ElementKind::MathUnderparen
        | ElementKind::MathUndershell
        | ElementKind::MathVec => inline_from_element_kind_tail(element, kind, introspector)?,
        _ => return Ok(None),
    };
    Ok(Some(inline))
}

fn inline_from_element_kind_tail(
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

pub(super) fn coalesce_raw_inlines(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut out = Vec::with_capacity(inlines.len());

    for inline in inlines {
        match (out.last_mut(), inline) {
            (Some(Inline::Raw(prev)), Inline::Raw(raw)) if prev.lang == raw.lang => {
                prev.text.push_str(&raw.text);
            }
            (_, inline) => out.push(inline),
        }
    }

    out
}

pub(super) fn collect_link_body(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    let body = collect_inlines(&element.children, introspector)?;
    if body.iter().any(inline_has_content) {
        return Ok(body);
    }

    let blocks = collect_item_blocks(&element.children, introspector)?;
    let text = plain_text_blocks(&blocks);
    if text.trim().is_empty() {
        Ok(body)
    } else {
        Ok(vec![Inline::Text(TextInline {
            text: text.trim().into(),
        })])
    }
}
