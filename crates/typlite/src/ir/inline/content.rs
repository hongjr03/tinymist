use ecow::EcoString;

use super::{FrameImage, Inline};

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
