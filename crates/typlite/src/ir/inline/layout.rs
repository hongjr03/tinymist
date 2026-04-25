use ecow::EcoString;

use super::{FrameImage, Inline};
use crate::ir::Block;

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
