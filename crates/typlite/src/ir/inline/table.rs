use ecow::EcoString;

use super::Inline;

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
