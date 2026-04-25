//! Experimental intermediate representation for typlite.

use ecow::EcoString;

use crate::element_spec::ElementKind;

/// A converted Typst document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Top-level blocks.
    pub blocks: Vec<Block>,
}

/// Block-level semantic nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// A section heading.
    Heading {
        /// Heading depth.
        level: u8,
        /// Heading body.
        body: Vec<Inline>,
    },
    /// A paragraph.
    Paragraph(Vec<Inline>),
    /// A block quote.
    Quote(Vec<Block>),
    /// A figure.
    Figure {
        /// Figure body.
        body: Vec<Block>,
        /// Optional caption.
        caption: Vec<Inline>,
    },
    /// An aligned block.
    Align(Vec<Block>),
    /// A math equation.
    Math(Vec<Inline>),
    /// A table-like block.
    Table {
        /// Table rows.
        rows: Vec<TableRow>,
    },
    /// Bibliography block.
    Bibliography(BlockElementData),
    /// Generic block element.
    Block(BlockElementData),
    /// Column break.
    Colbreak(BlockElementData),
    /// Columns block.
    Columns(BlockElementData),
    /// Outline block.
    Outline(BlockElementData),
    /// Page break.
    Pagebreak(BlockElementData),
    /// Paragraph break.
    Parbreak(BlockElementData),
    /// Stack block.
    Stack(BlockElementData),
    /// Terms list.
    Terms(BlockElementData),
    /// Document title.
    Title(BlockElementData),
    /// Vertical spacing.
    V(BlockElementData),
    /// A backend-specific raw block.
    Raw {
        /// Optional source language.
        lang: Option<EcoString>,
        /// Raw text.
        text: EcoString,
    },
    /// A bullet or numbered list.
    List {
        /// Whether this is a numbered list.
        ordered: bool,
        /// Whether list spacing is tight.
        tight: bool,
        /// Numbering pattern for numbered lists.
        numbering: Option<EcoString>,
        /// Start number for numbered lists.
        start: Option<i64>,
        /// Whether numbered list order is reversed.
        reversed: bool,
        /// Whether full numbers are shown for nested numbered lists.
        full: bool,
        /// List items.
        items: Vec<ListItem>,
    },
}

/// A list item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    /// Optional explicit number for numbered lists.
    pub number: Option<EcoString>,
    /// Item body blocks.
    pub body: Vec<Block>,
}

/// A table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    /// Table cells.
    pub cells: Vec<TableCell>,
}

/// A table cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableCell {
    /// Cell inline body.
    pub body: Vec<Inline>,
}

/// A generated block element payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockElementData {
    /// Extracted fields.
    pub fields: Vec<ElementField>,
    /// Block body extracted from content-like fields.
    pub body: Vec<Block>,
}

/// Inline semantic nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    /// Plain text.
    Text(EcoString),
    /// Emphasized inline content.
    Emph(Vec<Inline>),
    /// Strong inline content.
    Strong(Vec<Inline>),
    /// Hyperlink inline content.
    Link {
        /// Link destination.
        dest: EcoString,
        /// Link body.
        body: Vec<Inline>,
    },
    /// Struck-through inline content.
    Strike(Vec<Inline>),
    /// Subscript inline content.
    Sub(Vec<Inline>),
    /// Superscript inline content.
    Super(Vec<Inline>),
    /// Inline math content.
    Math(Vec<Inline>),
    /// A line break.
    Linebreak,
    /// Boxed content.
    Box(InlineElementData),
    /// Circle shape.
    Circle(InlineElementData),
    /// Citation.
    Cite(InlineElementData),
    /// Curve shape.
    Curve(InlineElementData),
    /// Curve close component.
    CurveClose(InlineElementData),
    /// Curve cubic component.
    CurveCubic(InlineElementData),
    /// Curve line component.
    CurveLine(InlineElementData),
    /// Curve move component.
    CurveMove(InlineElementData),
    /// Curve quad component.
    CurveQuad(InlineElementData),
    /// Document metadata.
    Document(InlineElementData),
    /// Ellipse shape.
    Ellipse(InlineElementData),
    /// Figure caption.
    FigureCaption(InlineElementData),
    /// Footnote.
    Footnote(InlineElementData),
    /// Footnote entry.
    FootnoteEntry(InlineElementData),
    /// Grid cell.
    GridCell(InlineElementData),
    /// Grid footer.
    GridFooter(InlineElementData),
    /// Grid header.
    GridHeader(InlineElementData),
    /// Grid horizontal line.
    GridHline(InlineElementData),
    /// Grid vertical line.
    GridVline(InlineElementData),
    /// Horizontal spacing.
    H(InlineElementData),
    /// Hidden content.
    Hide(InlineElementData),
    /// Highlighted content.
    Highlight(InlineElementData),
    /// Image.
    Image(InlineElementData),
    /// Line shape.
    Line(InlineElementData),
    /// Math accent.
    MathAccent(InlineElementData),
    /// Math attach.
    MathAttach(InlineElementData),
    /// Math binomial.
    MathBinom(InlineElementData),
    /// Math cancel.
    MathCancel(InlineElementData),
    /// Math cases.
    MathCases(InlineElementData),
    /// Math class.
    MathClass(InlineElementData),
    /// Math fraction.
    MathFrac(InlineElementData),
    /// Math limits.
    MathLimits(InlineElementData),
    /// Math left-right.
    MathLr(InlineElementData),
    /// Math matrix.
    MathMat(InlineElementData),
    /// Math middle delimiter.
    MathMid(InlineElementData),
    /// Math operator.
    MathOp(InlineElementData),
    /// Math overbrace.
    MathOverbrace(InlineElementData),
    /// Math overbracket.
    MathOverbracket(InlineElementData),
    /// Math overline.
    MathOverline(InlineElementData),
    /// Math overparen.
    MathOverparen(InlineElementData),
    /// Math overshell.
    MathOvershell(InlineElementData),
    /// Math primes.
    MathPrimes(InlineElementData),
    /// Math root.
    MathRoot(InlineElementData),
    /// Math scripts.
    MathScripts(InlineElementData),
    /// Math stretch.
    MathStretch(InlineElementData),
    /// Math underbrace.
    MathUnderbrace(InlineElementData),
    /// Math underbracket.
    MathUnderbracket(InlineElementData),
    /// Math underline.
    MathUnderline(InlineElementData),
    /// Math underparen.
    MathUnderparen(InlineElementData),
    /// Math undershell.
    MathUndershell(InlineElementData),
    /// Math vector.
    MathVec(InlineElementData),
    /// Metadata.
    Metadata(InlineElementData),
    /// Moved content.
    Move(InlineElementData),
    /// Outline entry.
    OutlineEntry(InlineElementData),
    /// Overlined content.
    Overline(InlineElementData),
    /// Padded content.
    Pad(InlineElementData),
    /// Page settings.
    Page(InlineElementData),
    /// Paragraph line.
    ParLine(InlineElementData),
    /// Path shape.
    Path(InlineElementData),
    /// PDF artifact.
    PdfArtifact(InlineElementData),
    /// PDF attachment.
    PdfAttach(InlineElementData),
    /// PDF embed.
    PdfEmbed(InlineElementData),
    /// Placed content.
    Place(InlineElementData),
    /// Place flush marker.
    PlaceFlush(InlineElementData),
    /// Polygon shape.
    Polygon(InlineElementData),
    /// Inline quote.
    Quote(InlineElementData),
    /// Raw line.
    RawLine(InlineElementData),
    /// Rectangle shape.
    Rect(InlineElementData),
    /// Reference.
    Ref(InlineElementData),
    /// Repeated content.
    Repeat(InlineElementData),
    /// Rotated content.
    Rotate(InlineElementData),
    /// Scaled content.
    Scale(InlineElementData),
    /// Skewed content.
    Skew(InlineElementData),
    /// Small caps content.
    Smallcaps(InlineElementData),
    /// Smart quote settings.
    Smartquote(InlineElementData),
    /// Square shape.
    Square(InlineElementData),
    /// Table cell.
    TableCell(InlineElementData),
    /// Table footer.
    TableFooter(InlineElementData),
    /// Table header.
    TableHeader(InlineElementData),
    /// Table horizontal line.
    TableHline(InlineElementData),
    /// Table vertical line.
    TableVline(InlineElementData),
    /// Underlined content.
    Underline(InlineElementData),
    /// Raw inline content.
    Raw {
        /// Optional source language.
        lang: Option<EcoString>,
        /// Raw text.
        text: EcoString,
    },
}

/// A generated inline element payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineElementData {
    /// Extracted fields.
    pub fields: Vec<ElementField>,
    /// Inline body extracted from content-like fields.
    pub body: Vec<Inline>,
}

/// An extracted element field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementField {
    /// Field name from the generated spec.
    pub name: &'static str,
    /// Field value.
    pub value: ElementFieldValue,
}

/// A field value extracted from a Typst element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementFieldValue {
    /// Scalar value rendered by the Typst-side IR library.
    Scalar(EcoString),
    /// Inline content field.
    Inlines(Vec<Inline>),
    /// Block content field.
    Blocks(Vec<Block>),
}

impl Block {
    /// Returns the body for variants generated from Typst elements.
    pub fn generated_body(&self) -> Option<&[Block]> {
        match self {
            Self::Bibliography(data)
            | Self::Block(data)
            | Self::Colbreak(data)
            | Self::Columns(data)
            | Self::Outline(data)
            | Self::Pagebreak(data)
            | Self::Parbreak(data)
            | Self::Stack(data)
            | Self::Terms(data)
            | Self::Title(data)
            | Self::V(data) => Some(&data.body),
            _ => None,
        }
    }
}

impl Inline {
    /// Returns the body for variants generated from Typst elements.
    pub fn generated_body(&self) -> Option<&[Inline]> {
        match self {
            Self::Box(data)
            | Self::Circle(data)
            | Self::Cite(data)
            | Self::Curve(data)
            | Self::CurveClose(data)
            | Self::CurveCubic(data)
            | Self::CurveLine(data)
            | Self::CurveMove(data)
            | Self::CurveQuad(data)
            | Self::Document(data)
            | Self::Ellipse(data)
            | Self::FigureCaption(data)
            | Self::Footnote(data)
            | Self::FootnoteEntry(data)
            | Self::GridCell(data)
            | Self::GridFooter(data)
            | Self::GridHeader(data)
            | Self::GridHline(data)
            | Self::GridVline(data)
            | Self::H(data)
            | Self::Hide(data)
            | Self::Highlight(data)
            | Self::Image(data)
            | Self::Line(data)
            | Self::MathAccent(data)
            | Self::MathAttach(data)
            | Self::MathBinom(data)
            | Self::MathCancel(data)
            | Self::MathCases(data)
            | Self::MathClass(data)
            | Self::MathFrac(data)
            | Self::MathLimits(data)
            | Self::MathLr(data)
            | Self::MathMat(data)
            | Self::MathMid(data)
            | Self::MathOp(data)
            | Self::MathOverbrace(data)
            | Self::MathOverbracket(data)
            | Self::MathOverline(data)
            | Self::MathOverparen(data)
            | Self::MathOvershell(data)
            | Self::MathPrimes(data)
            | Self::MathRoot(data)
            | Self::MathScripts(data)
            | Self::MathStretch(data)
            | Self::MathUnderbrace(data)
            | Self::MathUnderbracket(data)
            | Self::MathUnderline(data)
            | Self::MathUnderparen(data)
            | Self::MathUndershell(data)
            | Self::MathVec(data)
            | Self::Metadata(data)
            | Self::Move(data)
            | Self::OutlineEntry(data)
            | Self::Overline(data)
            | Self::Pad(data)
            | Self::Page(data)
            | Self::ParLine(data)
            | Self::Path(data)
            | Self::PdfArtifact(data)
            | Self::PdfAttach(data)
            | Self::PdfEmbed(data)
            | Self::Place(data)
            | Self::PlaceFlush(data)
            | Self::Polygon(data)
            | Self::Quote(data)
            | Self::RawLine(data)
            | Self::Rect(data)
            | Self::Ref(data)
            | Self::Repeat(data)
            | Self::Rotate(data)
            | Self::Scale(data)
            | Self::Skew(data)
            | Self::Smallcaps(data)
            | Self::Smartquote(data)
            | Self::Square(data)
            | Self::TableCell(data)
            | Self::TableFooter(data)
            | Self::TableHeader(data)
            | Self::TableHline(data)
            | Self::TableVline(data)
            | Self::Underline(data) => Some(&data.body),
            _ => None,
        }
    }
}

/// Creates a block IR variant from an element kind.
pub fn block_from_element_kind(kind: ElementKind, data: BlockElementData) -> Option<Block> {
    match kind {
        ElementKind::Bibliography => Some(Block::Bibliography(data)),
        ElementKind::Block => Some(Block::Block(data)),
        ElementKind::Colbreak => Some(Block::Colbreak(data)),
        ElementKind::Columns => Some(Block::Columns(data)),
        ElementKind::Outline => Some(Block::Outline(data)),
        ElementKind::Pagebreak => Some(Block::Pagebreak(data)),
        ElementKind::Parbreak => Some(Block::Parbreak(data)),
        ElementKind::Stack => Some(Block::Stack(data)),
        ElementKind::Terms => Some(Block::Terms(data)),
        ElementKind::Title => Some(Block::Title(data)),
        ElementKind::V => Some(Block::V(data)),
        _ => None,
    }
}

/// Creates an inline IR variant from an element kind.
pub fn inline_from_element_kind(kind: ElementKind, data: InlineElementData) -> Option<Inline> {
    match kind {
        ElementKind::Box => Some(Inline::Box(data)),
        ElementKind::Circle => Some(Inline::Circle(data)),
        ElementKind::Cite => Some(Inline::Cite(data)),
        ElementKind::Curve => Some(Inline::Curve(data)),
        ElementKind::CurveClose => Some(Inline::CurveClose(data)),
        ElementKind::CurveCubic => Some(Inline::CurveCubic(data)),
        ElementKind::CurveLine => Some(Inline::CurveLine(data)),
        ElementKind::CurveMove => Some(Inline::CurveMove(data)),
        ElementKind::CurveQuad => Some(Inline::CurveQuad(data)),
        ElementKind::Document => Some(Inline::Document(data)),
        ElementKind::Ellipse => Some(Inline::Ellipse(data)),
        ElementKind::FigureCaption => Some(Inline::FigureCaption(data)),
        ElementKind::Footnote => Some(Inline::Footnote(data)),
        ElementKind::FootnoteEntry => Some(Inline::FootnoteEntry(data)),
        ElementKind::GridCell => Some(Inline::GridCell(data)),
        ElementKind::GridFooter => Some(Inline::GridFooter(data)),
        ElementKind::GridHeader => Some(Inline::GridHeader(data)),
        ElementKind::GridHline => Some(Inline::GridHline(data)),
        ElementKind::GridVline => Some(Inline::GridVline(data)),
        ElementKind::H => Some(Inline::H(data)),
        ElementKind::Hide => Some(Inline::Hide(data)),
        ElementKind::Highlight => Some(Inline::Highlight(data)),
        ElementKind::Image => Some(Inline::Image(data)),
        ElementKind::Line => Some(Inline::Line(data)),
        ElementKind::MathAccent => Some(Inline::MathAccent(data)),
        ElementKind::MathAttach => Some(Inline::MathAttach(data)),
        ElementKind::MathBinom => Some(Inline::MathBinom(data)),
        ElementKind::MathCancel => Some(Inline::MathCancel(data)),
        ElementKind::MathCases => Some(Inline::MathCases(data)),
        ElementKind::MathClass => Some(Inline::MathClass(data)),
        ElementKind::MathFrac => Some(Inline::MathFrac(data)),
        ElementKind::MathLimits => Some(Inline::MathLimits(data)),
        ElementKind::MathLr => Some(Inline::MathLr(data)),
        ElementKind::MathMat => Some(Inline::MathMat(data)),
        ElementKind::MathMid => Some(Inline::MathMid(data)),
        ElementKind::MathOp => Some(Inline::MathOp(data)),
        ElementKind::MathOverbrace => Some(Inline::MathOverbrace(data)),
        ElementKind::MathOverbracket => Some(Inline::MathOverbracket(data)),
        ElementKind::MathOverline => Some(Inline::MathOverline(data)),
        ElementKind::MathOverparen => Some(Inline::MathOverparen(data)),
        ElementKind::MathOvershell => Some(Inline::MathOvershell(data)),
        ElementKind::MathPrimes => Some(Inline::MathPrimes(data)),
        ElementKind::MathRoot => Some(Inline::MathRoot(data)),
        ElementKind::MathScripts => Some(Inline::MathScripts(data)),
        ElementKind::MathStretch => Some(Inline::MathStretch(data)),
        ElementKind::MathUnderbrace => Some(Inline::MathUnderbrace(data)),
        ElementKind::MathUnderbracket => Some(Inline::MathUnderbracket(data)),
        ElementKind::MathUnderline => Some(Inline::MathUnderline(data)),
        ElementKind::MathUnderparen => Some(Inline::MathUnderparen(data)),
        ElementKind::MathUndershell => Some(Inline::MathUndershell(data)),
        ElementKind::MathVec => Some(Inline::MathVec(data)),
        ElementKind::Metadata => Some(Inline::Metadata(data)),
        ElementKind::Move => Some(Inline::Move(data)),
        ElementKind::OutlineEntry => Some(Inline::OutlineEntry(data)),
        ElementKind::Overline => Some(Inline::Overline(data)),
        ElementKind::Pad => Some(Inline::Pad(data)),
        ElementKind::Page => Some(Inline::Page(data)),
        ElementKind::ParLine => Some(Inline::ParLine(data)),
        ElementKind::Path => Some(Inline::Path(data)),
        ElementKind::PdfArtifact => Some(Inline::PdfArtifact(data)),
        ElementKind::PdfAttach => Some(Inline::PdfAttach(data)),
        ElementKind::PdfEmbed => Some(Inline::PdfEmbed(data)),
        ElementKind::Place => Some(Inline::Place(data)),
        ElementKind::PlaceFlush => Some(Inline::PlaceFlush(data)),
        ElementKind::Polygon => Some(Inline::Polygon(data)),
        ElementKind::Quote => Some(Inline::Quote(data)),
        ElementKind::RawLine => Some(Inline::RawLine(data)),
        ElementKind::Rect => Some(Inline::Rect(data)),
        ElementKind::Ref => Some(Inline::Ref(data)),
        ElementKind::Repeat => Some(Inline::Repeat(data)),
        ElementKind::Rotate => Some(Inline::Rotate(data)),
        ElementKind::Scale => Some(Inline::Scale(data)),
        ElementKind::Skew => Some(Inline::Skew(data)),
        ElementKind::Smallcaps => Some(Inline::Smallcaps(data)),
        ElementKind::Smartquote => Some(Inline::Smartquote(data)),
        ElementKind::Square => Some(Inline::Square(data)),
        ElementKind::TableCell => Some(Inline::TableCell(data)),
        ElementKind::TableFooter => Some(Inline::TableFooter(data)),
        ElementKind::TableHeader => Some(Inline::TableHeader(data)),
        ElementKind::TableHline => Some(Inline::TableHline(data)),
        ElementKind::TableVline => Some(Inline::TableVline(data)),
        ElementKind::Underline => Some(Inline::Underline(data)),
        _ => None,
    }
}
