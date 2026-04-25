use super::Inline;

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
    /// Number of columns spanned by this cell.
    pub colspan: usize,
    /// Number of rows spanned by this cell.
    pub rowspan: usize,
    /// Cell-level alignment.
    pub align: TableAlign,
}

/// Markdown-compatible table column alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableAlign {
    /// Default alignment.
    Default,
    /// Left alignment.
    Left,
    /// Center alignment.
    Center,
    /// Right alignment.
    Right,
}
