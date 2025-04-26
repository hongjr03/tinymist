//! Config handling for DOCX converter

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Style configuration for a document
#[derive(Debug, Clone, Deserialize, Default)]
pub struct StyleConfig {
    /// Document heading styles
    pub headings: Option<HeadingStylesConfig>,
    /// Document paragraph styles
    pub paragraphs: Option<ParagraphStylesConfig>,
    /// Document character styles
    pub characters: Option<CharacterStylesConfig>,
    /// Document table styles
    pub tables: Option<TableStyleConfig>,
}

/// Configuration for heading styles
#[derive(Debug, Clone, Deserialize, Default)]
pub struct HeadingStylesConfig {
    /// Heading 1 style
    pub heading1: Option<HeadingStyleConfig>,
    /// Heading 2 style
    pub heading2: Option<HeadingStyleConfig>,
    /// Heading 3 style
    pub heading3: Option<HeadingStyleConfig>,
    /// Heading 4 style
    pub heading4: Option<HeadingStyleConfig>,
    /// Heading 5 style
    pub heading5: Option<HeadingStyleConfig>,
    /// Heading 6 style
    pub heading6: Option<HeadingStyleConfig>,
}

/// Line spacing configuration
#[derive(Debug, Clone, Deserialize)]
pub struct LineSpacingConfig {
    /// Line spacing rule ("auto", "exact", "atLeast")
    pub rule: Option<String>,
    /// Line spacing value (for auto, represents multiplier; for exact or atLeast, represents twips value)
    pub value: Option<i32>,
    /// Space before paragraph (twips)
    pub before: Option<u32>,
    /// Space after paragraph (twips)
    pub after: Option<u32>,
}

/// Indent configuration
#[derive(Debug, Clone, Deserialize)]
pub struct IndentConfig {
    /// Left indent (twips)
    pub left: Option<i32>,
    /// Right indent (twips)
    pub right: Option<i32>,
    /// Special indent type ("firstLine", "hanging")
    pub special: Option<String>,
    /// Special indent value (twips)
    pub special_value: Option<i32>,
    /// First line indent character count
    pub first_line_chars: Option<i32>,
    /// Hanging indent character count
    pub hanging_chars: Option<i32>,
}

/// Configuration for heading style
#[derive(Debug, Clone, Deserialize)]
pub struct HeadingStyleConfig {
    /// Font size in half-points (e.g. 24 = 12pt)
    pub size: Option<usize>,
    /// Font name
    pub font: Option<String>,
    /// Whether the heading is bold
    pub bold: Option<bool>,
    /// Whether the heading is italic
    pub italic: Option<bool>,
    /// Heading color in hex format (e.g. "FF0000" for red)
    pub color: Option<String>,
    /// Line spacing settings
    pub line_spacing: Option<LineSpacingConfig>,
    /// Indent settings
    pub indent: Option<IndentConfig>,
}

/// Configuration for paragraph styles
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ParagraphStylesConfig {
    /// Code block style
    pub code_block: Option<ParagraphStyleConfig>,
    /// Block quote style
    pub blockquote: Option<ParagraphStyleConfig>,
    /// Caption style
    pub caption: Option<ParagraphStyleConfig>,
    /// Math block style
    pub math_block: Option<ParagraphStyleConfig>,
}

/// Configuration for paragraph style
#[derive(Debug, Clone, Deserialize)]
pub struct ParagraphStyleConfig {
    /// Font size in half-points (e.g. 24 = 12pt)
    pub size: Option<usize>,
    /// Font name
    pub font: Option<String>,
    /// Whether the text is bold
    pub bold: Option<bool>,
    /// Whether the text is italic
    pub italic: Option<bool>,
    /// Text color in hex format (e.g. "FF0000" for red)
    pub color: Option<String>,
    /// Text alignment ("left", "center", "right", "justify")
    pub alignment: Option<String>,
    /// Indent settings
    pub indent: Option<IndentConfig>,
    /// Line spacing settings
    pub line_spacing: Option<LineSpacingConfig>,
}

/// Configuration for character styles
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CharacterStylesConfig {
    /// Code inline style
    pub code_inline: Option<CharacterStyleConfig>,
    /// Emphasis style
    pub emphasis: Option<CharacterStyleConfig>,
    /// Strong style
    pub strong: Option<CharacterStyleConfig>,
    /// Highlight style
    pub highlight: Option<CharacterStyleConfig>,
    /// Hyperlink style
    pub hyperlink: Option<CharacterStyleConfig>,
}

/// Configuration for character style
#[derive(Debug, Clone, Deserialize)]
pub struct CharacterStyleConfig {
    /// Font size in half-points (e.g. 24 = 12pt)
    pub size: Option<usize>,
    /// Font name
    pub font: Option<String>,
    /// Whether the text is bold
    pub bold: Option<bool>,
    /// Whether the text is italic
    pub italic: Option<bool>,
    /// Text color in hex format (e.g. "FF0000" for red)
    pub color: Option<String>,
    /// Underline style ("single", "double", "thick", "dotted", "dash", etc.)
    pub underline: Option<String>,
    /// Highlight color name ("yellow", "green", etc.)
    pub highlight_color: Option<String>,
}

/// Configuration for table style
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TableStyleConfig {
    /// Table alignment ("left", "center", "right")
    pub alignment: Option<String>,
}

/// Numbering configuration for documents
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NumberingConfig {
    /// Ordered (numbered) list configuration
    pub ordered: Option<OrderedListConfig>,
    /// Unordered (bullet) list configuration
    pub unordered: Option<UnorderedListConfig>,
}

/// Configuration for ordered lists
#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderedListConfig {
    /// Configuration for each level of ordered lists
    pub levels: Option<HashMap<String, OrderedLevelConfig>>,
}

/// Configuration for a specific ordered list level
#[derive(Debug, Clone, Deserialize)]
pub struct OrderedLevelConfig {
    /// Number format ("decimal", "lowerLetter", "upperLetter", "lowerRoman", "upperRoman")
    pub format: Option<String>,
    /// Text format for the number (e.g. "%1.")
    pub text: Option<String>,
    /// Indent size in twips (1/20 of a point)
    pub indent: Option<i32>,
    /// Hanging indent size in twips
    pub hanging_indent: Option<i32>,
}

/// Configuration for unordered lists
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UnorderedListConfig {
    /// Configuration for each level of unordered lists
    pub levels: Option<HashMap<String, UnorderedLevelConfig>>,
}

/// Configuration for a specific unordered list level
#[derive(Debug, Clone, Deserialize)]
pub struct UnorderedLevelConfig {
    /// Bullet character (e.g. "•", "○", "▪")
    pub bullet: Option<String>,
    /// Indent size in twips (1/20 of a point)
    pub indent: Option<i32>,
    /// Hanging indent size in twips
    pub hanging_indent: Option<i32>,
}

/// Document configuration for DOCX conversion
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DocxConfig {
    /// Style configuration
    pub styles: Option<StyleConfig>,
    /// Numbering configuration
    pub numbering: Option<NumberingConfig>,
}

impl DocxConfig {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: DocxConfig =
            toml::from_str(&content).map_err(|e| format!("Failed to parse TOML config: {}", e))?;

        Ok(config)
    }
}
