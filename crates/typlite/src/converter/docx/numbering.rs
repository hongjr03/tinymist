//! List numbering management for DOCX conversion

use super::config::DocxConfig;
use docx_rs::*;

/// List numbering management for DOCX
#[derive(Clone, Debug)]
pub struct DocxNumbering {
    initialized: bool,
    next_id: usize,
    config: Option<DocxConfig>,
}

impl DocxNumbering {
    /// Create a new numbering manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            next_id: 1,
            config: None,
        }
    }

    /// Create a new numbering manager with custom configuration
    pub fn with_config(config: DocxConfig) -> Self {
        Self {
            initialized: false,
            next_id: 1,
            config: Some(config),
        }
    }

    /// Create a list level with the specified parameters
    pub fn create_list_level(
        id: usize,
        format: &str,
        text: &str,
        is_bullet: bool,
        indent: Option<i32>,
        hanging_indent: Option<i32>,
    ) -> Level {
        let indent_size = indent.unwrap_or(720 * (id + 1) as i32);
        let hanging = hanging_indent.unwrap_or(if is_bullet { 360 } else { 420 });

        Level::new(
            id,
            Start::new(1),
            NumberFormat::new(format),
            LevelText::new(text),
            LevelJc::new("left"),
        )
        .indent(
            Some(indent_size),
            Some(SpecialIndentType::Hanging(hanging)),
            None,
            None,
        )
    }

    /// Initialize the numbering manager
    pub fn initialize_numbering(&mut self, docx: Docx) -> Docx {
        if self.initialized {
            return docx;
        }

        self.initialized = true;
        docx
    }

    /// Create a new ordered list numbering, including a new AbstractNumbering instance
    pub fn create_ordered_numbering(&mut self, docx: Docx) -> (Docx, usize) {
        let abstract_id = self.next_id;
        let numbering_id = self.next_id;
        self.next_id += 1;

        let mut ordered_abstract = AbstractNumbering::new(abstract_id);

        // Check if we have custom ordered list configuration
        let ordered_config = self
            .config
            .as_ref()
            .and_then(|c| c.numbering.as_ref())
            .and_then(|n| n.ordered.as_ref())
            .and_then(|o| o.levels.as_ref());

        for i in 0..9 {
            let level_key = i.to_string();
            
            // Get custom configuration for this level if available
            let level_config = ordered_config.and_then(|levels| levels.get(&level_key));

            // Default values
            let level_text = match i {
                0 => "%1.",
                1 => "%2.",
                2 => "%3.",
                3 => "%4.",
                4 => "%5.",
                5 => "%6.",
                _ => "%7.",
            };

            let number_format = match i {
                0 => "decimal",
                1 => "lowerLetter",
                2 => "lowerRoman",
                3 => "upperRoman",
                4 => "decimal",
                5 => "lowerLetter",
                _ => "decimal",
            };

            // Apply custom config if available
            let format = level_config
                .and_then(|c| c.format.as_ref())
                .map_or(number_format, |v| v);
            let text = level_config
                .and_then(|c| c.text.as_ref())
                .map_or(level_text, |v| v);
            let indent = level_config.and_then(|c| c.indent);
            let hanging_indent = level_config.and_then(|c| c.hanging_indent);

            let mut ordered_level =
                Self::create_list_level(i, format, text, false, indent, hanging_indent);

            if i > 0 {
                ordered_level = ordered_level.level_restart(0_u32);
            }

            ordered_abstract = ordered_abstract.add_level(ordered_level);
        }

        let docx = docx
            .add_abstract_numbering(ordered_abstract)
            .add_numbering(Numbering::new(numbering_id, abstract_id));

        (docx, numbering_id)
    }

    /// Create a new unordered list numbering, including a new AbstractNumbering instance
    pub fn create_unordered_numbering(&mut self, docx: Docx) -> (Docx, usize) {
        let abstract_id = self.next_id;
        let numbering_id = self.next_id;
        self.next_id += 1;

        // Create AbstractNumbering for unordered list
        let mut unordered_abstract = AbstractNumbering::new(abstract_id);

        // Check if we have custom unordered list configuration
        let unordered_config = self
            .config
            .as_ref()
            .and_then(|c| c.numbering.as_ref())
            .and_then(|n| n.unordered.as_ref())
            .and_then(|u| u.levels.as_ref());

        // Add 9 levels of definition
        for i in 0..9 {
            let level_key = i.to_string();
            
            // Get custom configuration for this level if available
            let level_config = unordered_config.and_then(|levels| levels.get(&level_key));

            // Default bullets
            let default_bullet = match i {
                0 => "•",
                1 => "○",
                2 => "▪",
                3 => "▫",
                4 => "◆",
                _ => "◇",
            };

            // Apply custom config if available
            let bullet = level_config
                .and_then(|c| c.bullet.as_ref())
                .map_or(default_bullet, |v| v);
            let indent = level_config.and_then(|c| c.indent);
            let hanging_indent = level_config.and_then(|c| c.hanging_indent);

            let unordered_level =
                Self::create_list_level(i, "bullet", bullet, true, indent, hanging_indent);

            unordered_abstract = unordered_abstract.add_level(unordered_level);
        }

        let docx = docx
            .add_abstract_numbering(unordered_abstract)
            .add_numbering(Numbering::new(numbering_id, abstract_id));

        (docx, numbering_id)
    }
}
