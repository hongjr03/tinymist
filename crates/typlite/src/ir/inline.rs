use ecow::EcoString;
use typst_syntax::Span;

use super::{Block, MathNode};

/// Inline semantic nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Plain text.
    Text(TextInline),
    /// Emphasized inline content.
    Emph(EmphInline),
    /// Strong inline content.
    Strong(StrongInline),
    /// Hyperlink inline content.
    Link(LinkInline),
    /// Struck-through inline content.
    Strike(StrikeInline),
    /// Subscript inline content.
    Sub(SubInline),
    /// Superscript inline content.
    Super(SuperInline),
    /// Inline math content.
    Math(MathInline),
    /// A line break.
    Linebreak(LinebreakInline),
    /// A laid-out Typst frame rendered as SVG.
    Frame(FrameInline),
    /// Boxed content.
    Box(BoxInline),
    /// Circle shape.
    Circle(CircleInline),
    /// Citation.
    Cite(CiteInline),
    /// Curve shape.
    Curve(CurveInline),
    /// Curve close component.
    CurveClose(CurveCloseInline),
    /// Curve cubic component.
    CurveCubic(CurveCubicInline),
    /// Curve line component.
    CurveLine(CurveLineInline),
    /// Curve move component.
    CurveMove(CurveMoveInline),
    /// Curve quad component.
    CurveQuad(CurveQuadInline),
    /// Document metadata.
    Document(DocumentInline),
    /// Ellipse shape.
    Ellipse(EllipseInline),
    /// Figure caption.
    FigureCaption(FigureCaptionInline),
    /// Footnote.
    Footnote(FootnoteInline),
    /// Footnote entry.
    FootnoteEntry(FootnoteEntryInline),
    /// Grid cell.
    GridCell(GridCellInline),
    /// Grid footer.
    GridFooter(GridFooterInline),
    /// Grid header.
    GridHeader(GridHeaderInline),
    /// Grid horizontal line.
    GridHline(GridHlineInline),
    /// Grid vertical line.
    GridVline(GridVlineInline),
    /// Horizontal spacing.
    H(HInline),
    /// Hidden content.
    Hide(HideInline),
    /// Highlighted content.
    Highlight(HighlightInline),
    /// Image.
    Image(ImageInline),
    /// Line shape.
    Line(LineInline),
    /// Math accent.
    MathAccent(MathAccentInline),
    /// Math attach.
    MathAttach(MathAttachInline),
    /// Math binomial.
    MathBinom(MathBinomInline),
    /// Math cancel.
    MathCancel(MathCancelInline),
    /// Math cases.
    MathCases(MathCasesInline),
    /// Math class.
    MathClass(MathClassInline),
    /// Math fraction.
    MathFrac(MathFracInline),
    /// Math limits.
    MathLimits(MathLimitsInline),
    /// Math left-right.
    MathLr(MathLrInline),
    /// Math matrix.
    MathMat(MathMatInline),
    /// Math middle delimiter.
    MathMid(MathMidInline),
    /// Math operator.
    MathOp(MathOpInline),
    /// Math overbrace.
    MathOverbrace(MathOverbraceInline),
    /// Math overbracket.
    MathOverbracket(MathOverbracketInline),
    /// Math overline.
    MathOverline(MathOverlineInline),
    /// Math overparen.
    MathOverparen(MathOverparenInline),
    /// Math overshell.
    MathOvershell(MathOvershellInline),
    /// Math primes.
    MathPrimes(MathPrimesInline),
    /// Math root.
    MathRoot(MathRootInline),
    /// Math scripts.
    MathScripts(MathScriptsInline),
    /// Math stretch.
    MathStretch(MathStretchInline),
    /// Math underbrace.
    MathUnderbrace(MathUnderbraceInline),
    /// Math underbracket.
    MathUnderbracket(MathUnderbracketInline),
    /// Math underline.
    MathUnderline(MathUnderlineInline),
    /// Math underparen.
    MathUnderparen(MathUnderparenInline),
    /// Math undershell.
    MathUndershell(MathUndershellInline),
    /// Math vector.
    MathVec(MathVecInline),
    /// Metadata.
    Metadata(MetadataInline),
    /// Moved content.
    Move(MoveInline),
    /// Outline entry.
    OutlineEntry(OutlineEntryInline),
    /// Overlined content.
    Overline(OverlineInline),
    /// Padded content.
    Pad(PadInline),
    /// Page settings.
    Page(PageInline),
    /// Paragraph line.
    ParLine(ParLineInline),
    /// Path shape.
    Path(PathInline),
    /// PDF artifact.
    PdfArtifact(PdfArtifactInline),
    /// PDF attachment.
    PdfAttach(PdfAttachInline),
    /// PDF embed.
    PdfEmbed(PdfEmbedInline),
    /// Placed content.
    Place(PlaceInline),
    /// Place flush marker.
    PlaceFlush(PlaceFlushInline),
    /// Polygon shape.
    Polygon(PolygonInline),
    /// Inline quote.
    Quote(QuoteInline),
    /// Raw line.
    RawLine(RawLineInline),
    /// Rectangle shape.
    Rect(RectInline),
    /// Reference.
    Ref(RefInline),
    /// Repeated content.
    Repeat(RepeatInline),
    /// Rotated content.
    Rotate(RotateInline),
    /// Scaled content.
    Scale(ScaleInline),
    /// Skewed content.
    Skew(SkewInline),
    /// Small caps content.
    Smallcaps(SmallcapsInline),
    /// Smart quote settings.
    Smartquote(SmartquoteInline),
    /// Square shape.
    Square(SquareInline),
    /// Table cell.
    TableCell(TableCellInline),
    /// Table footer.
    TableFooter(TableFooterInline),
    /// Table header.
    TableHeader(TableHeaderInline),
    /// Table horizontal line.
    TableHline(TableHlineInline),
    /// Table vertical line.
    TableVline(TableVlineInline),
    /// Underlined content.
    Underline(UnderlineInline),
    /// Raw inline content.
    Raw(RawInline),
}

/// Plain text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInline {
    /// Text body.
    pub text: EcoString,
}

/// Emphasized inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmphInline {
    /// Emphasized body.
    pub body: Vec<Inline>,
}

/// Strong inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrongInline {
    /// Strong body.
    pub body: Vec<Inline>,
}

/// Hyperlink inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkInline {
    /// Link destination.
    pub dest: EcoString,
    /// Link body.
    pub body: Vec<Inline>,
}

/// Struck-through inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrikeInline {
    /// Struck-through body.
    pub body: Vec<Inline>,
}

/// Subscript inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubInline {
    /// Subscript body.
    pub body: Vec<Inline>,
}

/// Superscript inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuperInline {
    /// Superscript body.
    pub body: Vec<Inline>,
}

/// Inline math content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathInline {
    /// Math body.
    pub body: MathNode,
}

/// A line break.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LinebreakInline {}

/// A laid-out Typst frame rendered as SVG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameInline {
    /// Rendered frame image.
    pub image: FrameImage,
}

/// Raw inline content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInline {
    /// Optional source language.
    pub lang: Option<EcoString>,
    /// Raw text.
    pub text: EcoString,
}

/// Boxed content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoxInline {
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Baseline.
    pub baseline: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Corner radius.
    pub radius: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Clip behavior.
    pub clip: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Circle shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CircleInline {
    /// Radius.
    pub radius: Option<EcoString>,
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Citation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CiteInline {
    /// Citation key.
    pub key: Option<EcoString>,
    /// Citation supplement.
    pub supplement: Vec<Inline>,
    /// Citation form.
    pub form: Option<EcoString>,
    /// Citation style.
    pub style: Option<EcoString>,
}

/// Curve shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveInline {
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Fill rule.
    pub fill_rule: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Curve components.
    pub components: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Curve close component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveCloseInline {
    /// Close mode.
    pub mode: Option<EcoString>,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve cubic component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveCubicInline {
    /// Start control point.
    pub control_start: Option<EcoString>,
    /// End control point.
    pub control_end: Option<EcoString>,
    /// End point.
    pub end: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve line component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveLineInline {
    /// End point.
    pub end: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve move component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveMoveInline {
    /// Start point.
    pub start: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve quadratic component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveQuadInline {
    /// Control point.
    pub control: Option<EcoString>,
    /// End point.
    pub end: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Document metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentInline {
    /// Title.
    pub title: Option<EcoString>,
    /// Author.
    pub author: Option<EcoString>,
    /// Description.
    pub description: Option<EcoString>,
    /// Keywords.
    pub keywords: Option<EcoString>,
    /// Date.
    pub date: Option<EcoString>,
}

/// Ellipse shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EllipseInline {
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Figure caption.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FigureCaptionInline {
    /// Caption position.
    pub position: Option<EcoString>,
    /// Caption separator.
    pub separator: Option<EcoString>,
    /// Caption body.
    pub body: Vec<Inline>,
    /// Figure kind.
    pub kind: Option<EcoString>,
    /// Figure supplement.
    pub supplement: Vec<Inline>,
    /// Numbering.
    pub numbering: Option<EcoString>,
    /// Counter.
    pub counter: Option<EcoString>,
}

/// Footnote.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FootnoteInline {
    /// Numbering pattern.
    pub numbering: Option<EcoString>,
    /// Footnote body.
    pub body: Vec<Inline>,
}

/// Footnote entry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FootnoteEntryInline {
    /// Entry note.
    pub note: Vec<Inline>,
    /// Separator.
    pub separator: Vec<Inline>,
    /// Clearance.
    pub clearance: Option<EcoString>,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Indent.
    pub indent: Option<EcoString>,
}

/// Grid cell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridCellInline {
    /// Cell body.
    pub body: Vec<Inline>,
    /// Column index.
    pub x: Option<EcoString>,
    /// Row index.
    pub y: Option<EcoString>,
    /// Column span.
    pub colspan: Option<EcoString>,
    /// Row span.
    pub rowspan: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Alignment.
    pub align: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Whether the cell is breakable.
    pub breakable: Option<EcoString>,
}

/// Grid footer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridFooterInline {
    /// Whether footer repeats.
    pub repeat: bool,
    /// Footer children.
    pub children: Vec<Inline>,
}

/// Grid header.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridHeaderInline {
    /// Whether header repeats.
    pub repeat: bool,
    /// Header level.
    pub level: Option<EcoString>,
    /// Header children.
    pub children: Vec<Inline>,
}

/// Grid horizontal line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridHlineInline {
    /// Row index.
    pub y: Option<EcoString>,
    /// Start column.
    pub start: Option<EcoString>,
    /// End column.
    pub end: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Line position.
    pub position: Option<EcoString>,
}

/// Grid vertical line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridVlineInline {
    /// Column index.
    pub x: Option<EcoString>,
    /// Start row.
    pub start: Option<EcoString>,
    /// End row.
    pub end: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Line position.
    pub position: Option<EcoString>,
}

/// Horizontal spacing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HInline {
    /// Spacing amount.
    pub amount: Option<EcoString>,
    /// Whether the spacing is weak.
    pub weak: bool,
}

/// Hidden content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HideInline {
    /// Hidden body.
    pub body: Vec<Inline>,
}

/// Highlighted content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HighlightInline {
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Top edge.
    pub top_edge: Option<EcoString>,
    /// Bottom edge.
    pub bottom_edge: Option<EcoString>,
    /// Extent.
    pub extent: Option<EcoString>,
    /// Radius.
    pub radius: Option<EcoString>,
    /// Highlighted body.
    pub body: Vec<Inline>,
}

/// Image.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImageInline {
    /// Encoded source.
    pub source: Option<EcoString>,
    /// Explicit image format.
    pub format: Option<EcoString>,
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Alternative text.
    pub alt: Option<EcoString>,
    /// PDF page.
    pub page: Option<EcoString>,
    /// Fit mode.
    pub fit: Option<EcoString>,
    /// Scaling mode.
    pub scaling: Option<EcoString>,
    /// ICC profile.
    pub icc: Option<EcoString>,
    /// Rendered frame for formats that need it.
    pub frame: Option<FrameImage>,
}

/// Line shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LineInline {
    /// Start point.
    pub start: Option<EcoString>,
    /// End point.
    pub end: Option<EcoString>,
    /// Length.
    pub length: Option<EcoString>,
    /// Angle.
    pub angle: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Math accent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathAccentInline {
    /// Base expression.
    pub base: Option<EcoString>,
    /// Accent.
    pub accent: Option<EcoString>,
    /// Accent size.
    pub size: Option<EcoString>,
    /// Whether the base is dotless.
    pub dotless: bool,
}

/// Math attachment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathAttachInline {
    /// Base expression.
    pub base: Option<EcoString>,
    /// Top attachment.
    pub t: Option<EcoString>,
    /// Bottom attachment.
    pub b: Option<EcoString>,
    /// Top-left attachment.
    pub tl: Option<EcoString>,
    /// Bottom-left attachment.
    pub bl: Option<EcoString>,
    /// Top-right attachment.
    pub tr: Option<EcoString>,
    /// Bottom-right attachment.
    pub br: Option<EcoString>,
}

/// Math binomial.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathBinomInline {
    /// Upper expression.
    pub upper: Option<EcoString>,
    /// Lower expression.
    pub lower: Option<EcoString>,
}

/// Math cancel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathCancelInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Stroke length.
    pub length: Option<EcoString>,
    /// Whether cancellation is inverted.
    pub inverted: bool,
    /// Whether cancellation is crossed.
    pub cross: bool,
    /// Cancellation angle.
    pub angle: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
}

/// Math cases.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathCasesInline {
    /// Delimiter.
    pub delim: Option<EcoString>,
    /// Whether cases are reversed.
    pub reverse: bool,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Children.
    pub children: Vec<Inline>,
}

/// Math class.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathClassInline {
    /// Class.
    pub class: Option<EcoString>,
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math fraction.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathFracInline {
    /// Numerator.
    pub num: Option<EcoString>,
    /// Denominator.
    pub denom: Option<EcoString>,
    /// Fraction style.
    pub style: Option<EcoString>,
}

/// Math limits.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathLimitsInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Whether limits are inline.
    pub inline: bool,
}

/// Math left-right group.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathLrInline {
    /// Size.
    pub size: Option<EcoString>,
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math matrix.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathMatInline {
    /// Delimiter.
    pub delim: Option<EcoString>,
    /// Alignment.
    pub align: Option<EcoString>,
    /// Augment.
    pub augment: Option<EcoString>,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Row gap.
    pub row_gap: Option<EcoString>,
    /// Column gap.
    pub column_gap: Option<EcoString>,
    /// Rows.
    pub rows: Option<EcoString>,
}

/// Math middle delimiter.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathMidInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math operator.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOpInline {
    /// Operator text.
    pub text: Option<EcoString>,
    /// Whether limits are used.
    pub limits: bool,
}

/// Math overbrace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverbraceInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math overbracket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverbracketInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math overline.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverlineInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math overparen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverparenInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math overshell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOvershellInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math primes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathPrimesInline {
    /// Prime count.
    pub count: Option<EcoString>,
}

/// Math root.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathRootInline {
    /// Root index.
    pub index: Option<EcoString>,
    /// Radicand.
    pub radicand: Option<EcoString>,
}

/// Math scripts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathScriptsInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math stretch.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathStretchInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Stretch size.
    pub size: Option<EcoString>,
}

/// Math underbrace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderbraceInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math underbracket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderbracketInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math underline.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderlineInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math underparen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderparenInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math undershell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUndershellInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math vector.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathVecInline {
    /// Delimiter.
    pub delim: Option<EcoString>,
    /// Alignment.
    pub align: Option<EcoString>,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Children.
    pub children: Vec<Inline>,
}

/// Metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetadataInline {
    /// Metadata value.
    pub value: Option<EcoString>,
}

/// Moved content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MoveInline {
    /// Horizontal offset.
    pub dx: Option<EcoString>,
    /// Vertical offset.
    pub dy: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Outline entry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutlineEntryInline {
    /// Heading level.
    pub level: Option<EcoString>,
    /// Referenced element.
    pub element: Vec<Block>,
    /// Fill.
    pub fill: Option<EcoString>,
}

/// Overlined content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OverlineInline {
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Offset.
    pub offset: Option<EcoString>,
    /// Extent.
    pub extent: Option<EcoString>,
    /// Whether to evade.
    pub evade: Option<EcoString>,
    /// Background paint.
    pub background: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Padded content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PadInline {
    /// Left padding.
    pub left: Option<EcoString>,
    /// Top padding.
    pub top: Option<EcoString>,
    /// Right padding.
    pub right: Option<EcoString>,
    /// Bottom padding.
    pub bottom: Option<EcoString>,
    /// Horizontal padding shorthand.
    pub x: Option<EcoString>,
    /// Vertical padding shorthand.
    pub y: Option<EcoString>,
    /// Remaining padding value.
    pub rest: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Page settings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageInline {
    /// Paper preset.
    pub paper: Option<EcoString>,
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Whether the page is flipped.
    pub flipped: bool,
    /// Margin.
    pub margin: Option<EcoString>,
    /// Binding.
    pub binding: Option<EcoString>,
    /// Column count.
    pub columns: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Numbering.
    pub numbering: Option<EcoString>,
    /// Supplement.
    pub supplement: Vec<Inline>,
    /// Number alignment.
    pub number_align: Option<EcoString>,
    /// Header.
    pub header: Vec<Inline>,
    /// Header ascent.
    pub header_ascent: Option<EcoString>,
    /// Footer.
    pub footer: Vec<Inline>,
    /// Footer descent.
    pub footer_descent: Option<EcoString>,
    /// Background.
    pub background: Vec<Inline>,
    /// Foreground.
    pub foreground: Vec<Inline>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Paragraph line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParLineInline {
    /// Numbering.
    pub numbering: Option<EcoString>,
    /// Number alignment.
    pub number_align: Option<EcoString>,
    /// Number margin.
    pub number_margin: Option<EcoString>,
    /// Number clearance.
    pub number_clearance: Option<EcoString>,
    /// Numbering scope.
    pub numbering_scope: Option<EcoString>,
}

/// Path shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PathInline {
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Fill rule.
    pub fill_rule: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Whether the path is closed.
    pub closed: bool,
    /// Vertices.
    pub vertices: Option<EcoString>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// PDF artifact.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PdfArtifactInline {
    /// Artifact kind.
    pub kind: Option<EcoString>,
    /// Artifact body.
    pub body: Vec<Inline>,
}

/// PDF attachment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PdfAttachInline {
    /// Attached path.
    pub path: Option<EcoString>,
    /// Attached data.
    pub data: Option<EcoString>,
    /// Relationship.
    pub relationship: Option<EcoString>,
    /// MIME type.
    pub mime_type: Option<EcoString>,
    /// Description.
    pub description: Option<EcoString>,
}

/// PDF embed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PdfEmbedInline {
    /// Embedded path.
    pub path: Option<EcoString>,
    /// Embedded data.
    pub data: Option<EcoString>,
    /// Relationship.
    pub relationship: Option<EcoString>,
    /// MIME type.
    pub mime_type: Option<EcoString>,
    /// Description.
    pub description: Option<EcoString>,
}

/// Placed content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlaceInline {
    /// Alignment.
    pub alignment: Option<EcoString>,
    /// Placement scope.
    pub scope: Option<EcoString>,
    /// Float behavior.
    pub float: Option<EcoString>,
    /// Clearance.
    pub clearance: Option<EcoString>,
    /// Horizontal offset.
    pub dx: Option<EcoString>,
    /// Vertical offset.
    pub dy: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Place flush marker.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlaceFlushInline {}

/// Polygon shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PolygonInline {
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Fill rule.
    pub fill_rule: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Vertices.
    pub vertices: Option<EcoString>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Inline quote.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuoteInline {
    /// Whether quote is block-level.
    pub block: bool,
    /// Whether quotation marks are rendered.
    pub quotes: Option<EcoString>,
    /// Attribution.
    pub attribution: Vec<Inline>,
    /// Quote body.
    pub body: Vec<Inline>,
}

/// Raw line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RawLineInline {
    /// Line number.
    pub number: Option<EcoString>,
    /// Line count.
    pub count: Option<EcoString>,
    /// Raw text.
    pub text: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Rectangle shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RectInline {
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Corner radius.
    pub radius: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Reference.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RefInline {
    /// Reference target.
    pub target: Option<EcoString>,
    /// Reference supplement.
    pub supplement: Vec<Inline>,
    /// Reference form.
    pub form: Option<EcoString>,
    /// Rendered citation content.
    pub citation: Vec<Inline>,
    /// Referenced element.
    pub element: Vec<Block>,
}

/// Repeated content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RepeatInline {
    /// Body.
    pub body: Vec<Inline>,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Whether repeated content is justified.
    pub justify: Option<EcoString>,
}

/// Rotated content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RotateInline {
    /// Rotation angle.
    pub angle: Option<EcoString>,
    /// Transform origin.
    pub origin: Option<EcoString>,
    /// Whether layout reflows.
    pub reflow: bool,
    /// Body.
    pub body: Vec<Inline>,
}

/// Scaled content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScaleInline {
    /// Scale factor.
    pub factor: Option<EcoString>,
    /// Horizontal scale.
    pub x: Option<EcoString>,
    /// Vertical scale.
    pub y: Option<EcoString>,
    /// Transform origin.
    pub origin: Option<EcoString>,
    /// Whether layout reflows.
    pub reflow: bool,
    /// Body.
    pub body: Vec<Inline>,
}

/// Skewed content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkewInline {
    /// Horizontal skew angle.
    pub ax: Option<EcoString>,
    /// Vertical skew angle.
    pub ay: Option<EcoString>,
    /// Transform origin.
    pub origin: Option<EcoString>,
    /// Whether layout reflows.
    pub reflow: bool,
    /// Body.
    pub body: Vec<Inline>,
}

/// Small caps content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SmallcapsInline {
    /// Whether all letters are small caps.
    pub all: bool,
    /// Body.
    pub body: Vec<Inline>,
}

/// Smart quote.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SmartquoteInline {
    /// Whether this is a double quote.
    pub double: Option<EcoString>,
    /// Whether smart quotes are enabled.
    pub enabled: Option<EcoString>,
    /// Whether the alternative quote form is used.
    pub alternative: Option<EcoString>,
    /// Explicit quote pair.
    pub quotes: Option<EcoString>,
}

/// Square shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SquareInline {
    /// Size.
    pub size: Option<EcoString>,
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Corner radius.
    pub radius: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Table cell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableCellInline {
    /// Cell body.
    pub body: Vec<Inline>,
    /// Column index.
    pub x: Option<EcoString>,
    /// Row index.
    pub y: Option<EcoString>,
    /// Column span.
    pub colspan: Option<EcoString>,
    /// Row span.
    pub rowspan: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Alignment.
    pub align: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Whether the cell is breakable.
    pub breakable: Option<EcoString>,
}

/// Table footer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableFooterInline {
    /// Whether footer repeats.
    pub repeat: bool,
    /// Footer children.
    pub children: Vec<Inline>,
}

/// Table header.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableHeaderInline {
    /// Whether header repeats.
    pub repeat: bool,
    /// Header level.
    pub level: Option<EcoString>,
    /// Header children.
    pub children: Vec<Inline>,
}

/// Table horizontal line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableHlineInline {
    /// Row index.
    pub y: Option<EcoString>,
    /// Start column.
    pub start: Option<EcoString>,
    /// End column.
    pub end: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Line position.
    pub position: Option<EcoString>,
}

/// Table vertical line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableVlineInline {
    /// Column index.
    pub x: Option<EcoString>,
    /// Start row.
    pub start: Option<EcoString>,
    /// End row.
    pub end: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Line position.
    pub position: Option<EcoString>,
}

/// Underlined content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnderlineInline {
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Offset.
    pub offset: Option<EcoString>,
    /// Extent.
    pub extent: Option<EcoString>,
    /// Whether to evade.
    pub evade: Option<EcoString>,
    /// Background paint.
    pub background: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// A rendered frame image extracted from `html.frame`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameImage {
    /// SVG payload.
    pub svg: EcoString,
}
