mod base;
mod content;
mod frame;
mod graphics;
mod layout;
mod math;
mod table;

pub use self::base::*;
pub use self::content::*;
pub use self::frame::*;
pub use self::graphics::*;
pub use self::layout::*;
pub use self::math::*;
pub use self::table::*;

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
