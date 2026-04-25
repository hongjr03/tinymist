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
    /// A generic block element not yet assigned a dedicated IR node.
    Element(BlockElement),
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

/// A block-level Typst element carried by the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockElement {
    /// Element kind from the generated typlite spec.
    pub kind: ElementKind,
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
    /// A generic inline element not yet assigned a dedicated IR node.
    Element(InlineElement),
    /// Raw inline content.
    Raw {
        /// Optional source language.
        lang: Option<EcoString>,
        /// Raw text.
        text: EcoString,
    },
}

/// An inline Typst element carried by the IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineElement {
    /// Element kind from the generated typlite spec.
    pub kind: ElementKind,
    /// Inline body extracted from content-like fields.
    pub body: Vec<Inline>,
}
