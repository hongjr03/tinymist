//! Experimental intermediate representation for typlite.

use ecow::EcoString;

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
    /// A backend-specific raw block.
    Raw {
        /// Optional source language.
        lang: Option<EcoString>,
        /// Raw text.
        text: EcoString,
    },
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
    /// Raw inline content.
    Raw {
        /// Optional source language.
        lang: Option<EcoString>,
        /// Raw text.
        text: EcoString,
    },
}
