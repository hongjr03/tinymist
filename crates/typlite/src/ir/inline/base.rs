use ecow::EcoString;

use super::{FrameImage, Inline};
use crate::ir::MathNode;

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
