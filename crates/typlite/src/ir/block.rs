use ecow::EcoString;

use super::{Inline, MathNode, TableAlign, TableRow};

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
    Heading(HeadingBlock),
    /// A paragraph.
    Paragraph(ParagraphBlock),
    /// A block quote.
    Quote(QuoteBlock),
    /// A figure.
    Figure(FigureBlock),
    /// An aligned block.
    Align(AlignBlock),
    /// A math equation.
    Math(MathBlock),
    /// A table-like block.
    Table(TableBlock),
    /// Bibliography block.
    Bibliography(BibliographyBlock),
    /// Typst `block` element.
    Block(BlockBlock),
    /// Column break.
    Colbreak(ColbreakBlock),
    /// Columns block.
    Columns(ColumnsBlock),
    /// Moved block.
    Move(MoveBlock),
    /// Outline block.
    Outline(OutlineBlock),
    /// Padded block.
    Pad(PadBlock),
    /// Page break.
    Pagebreak(PagebreakBlock),
    /// Paragraph break.
    Parbreak(ParbreakBlock),
    /// Rotated block.
    Rotate(RotateBlock),
    /// Scaled block.
    Scale(ScaleBlock),
    /// Skewed block.
    Skew(SkewBlock),
    /// Stack block.
    Stack(StackBlock),
    /// Terms list.
    Terms(TermsBlock),
    /// Document title.
    Title(TitleBlock),
    /// Vertical spacing.
    V(VBlock),
    /// A backend-specific raw block.
    Raw(RawBlock),
    /// A bullet or numbered list.
    List(ListBlock),
}

/// A section heading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadingBlock {
    /// Optional HTML anchor id emitted by Typst.
    pub id: Option<EcoString>,
    /// Heading depth.
    pub level: u8,
    /// Heading body.
    pub body: Vec<Inline>,
}

/// A paragraph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphBlock {
    /// Paragraph body.
    pub body: Vec<Inline>,
}

/// A block quote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteBlock {
    /// Quoted blocks.
    pub body: Vec<Block>,
}

/// A figure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureBlock {
    /// Figure body.
    pub body: Vec<Block>,
    /// Optional caption.
    pub caption: Vec<Inline>,
    /// Optional alternative text.
    pub alt: Option<EcoString>,
}

/// An aligned block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignBlock {
    /// Alignment value.
    pub alignment: Option<EcoString>,
    /// Aligned body.
    pub body: Vec<Block>,
}

/// A math equation block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathBlock {
    /// Math body.
    pub body: MathNode,
}

/// A table-like block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableBlock {
    /// Table rows.
    pub rows: Vec<TableRow>,
    /// Column alignments.
    pub alignments: Vec<TableAlign>,
}

/// Terms list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermsBlock {
    /// Terms list items.
    pub items: Vec<TermItem>,
}

/// A backend-specific raw block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawBlock {
    /// Optional source language.
    pub lang: Option<EcoString>,
    /// Raw text.
    pub text: EcoString,
}

/// A bullet or numbered list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListBlock {
    /// Whether this is a numbered list.
    pub ordered: bool,
    /// Whether list spacing is tight.
    pub tight: bool,
    /// Numbering pattern for numbered lists.
    pub numbering: Option<EcoString>,
    /// Start number for numbered lists.
    pub start: Option<i64>,
    /// Whether numbered list order is reversed.
    pub reversed: bool,
    /// Whether full numbers are shown for nested numbered lists.
    pub full: bool,
    /// List items.
    pub items: Vec<ListItem>,
}

/// A list item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    /// Optional explicit number for numbered lists.
    pub number: Option<EcoString>,
    /// Item body blocks.
    pub body: Vec<Block>,
}

/// A terms list item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermItem {
    /// Term body.
    pub term: Vec<Inline>,
    /// Term description blocks.
    pub description: Vec<Block>,
}

/// Bibliography block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BibliographyBlock {
    /// Bibliography sources encoded by the Typst-side library.
    pub sources: Option<EcoString>,
    /// Optional rendered title.
    pub title: Vec<Inline>,
    /// Whether all entries should be rendered.
    pub full: bool,
    /// Optional bibliography style name.
    pub style: Option<EcoString>,
}

/// Typst `block` element.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockBlock {
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Whether the block can break.
    pub breakable: Option<EcoString>,
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
    /// Inner spacing.
    pub spacing: Option<EcoString>,
    /// Spacing above.
    pub above: Option<EcoString>,
    /// Spacing below.
    pub below: Option<EcoString>,
    /// Clip behavior.
    pub clip: Option<EcoString>,
    /// Sticky behavior.
    pub sticky: Option<EcoString>,
    /// Block body.
    pub body: Vec<Block>,
}

/// Column break.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColbreakBlock {
    /// Whether the break is weak.
    pub weak: bool,
}

/// Columns block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColumnsBlock {
    /// Column count.
    pub count: Option<EcoString>,
    /// Column gutter.
    pub gutter: Option<EcoString>,
    /// Column body.
    pub body: Vec<Block>,
}

/// Moved block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MoveBlock {
    /// Horizontal offset.
    pub dx: Option<EcoString>,
    /// Vertical offset.
    pub dy: Option<EcoString>,
    /// Moved body.
    pub body: Vec<Block>,
}

/// Outline block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutlineBlock {
    /// Optional outline title.
    pub title: Vec<Inline>,
    /// Outline target.
    pub target: Option<EcoString>,
    /// Outline depth.
    pub depth: Option<EcoString>,
    /// Outline indent.
    pub indent: Option<EcoString>,
}

/// Padded block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PadBlock {
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
    /// Padded body.
    pub body: Vec<Block>,
}

/// Page break.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PagebreakBlock {
    /// Whether the break is weak.
    pub weak: bool,
    /// Target page parity.
    pub to: Option<EcoString>,
}

/// Paragraph break.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParbreakBlock {}

/// Rotated block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RotateBlock {
    /// Rotation angle.
    pub angle: Option<EcoString>,
    /// Transform origin.
    pub origin: Option<EcoString>,
    /// Whether layout reflows.
    pub reflow: bool,
    /// Rotated body.
    pub body: Vec<Block>,
}

/// Scaled block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScaleBlock {
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
    /// Scaled body.
    pub body: Vec<Block>,
}

/// Skewed block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkewBlock {
    /// Horizontal skew angle.
    pub ax: Option<EcoString>,
    /// Vertical skew angle.
    pub ay: Option<EcoString>,
    /// Transform origin.
    pub origin: Option<EcoString>,
    /// Whether layout reflows.
    pub reflow: bool,
    /// Skewed body.
    pub body: Vec<Block>,
}

/// Stack block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StackBlock {
    /// Stack direction.
    pub dir: Option<EcoString>,
    /// Stack spacing.
    pub spacing: Option<EcoString>,
    /// Stack children.
    pub children: Vec<Block>,
}

/// Document title block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TitleBlock {
    /// Title body.
    pub body: Vec<Block>,
}

/// Vertical spacing block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VBlock {
    /// Spacing amount.
    pub amount: Option<EcoString>,
    /// Whether the spacing is weak.
    pub weak: bool,
}
