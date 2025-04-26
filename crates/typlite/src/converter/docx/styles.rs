//! Document style management for DOCX conversion

use super::config::{
    CharacterStyleConfig, DocxConfig, HeadingStyleConfig, IndentConfig, LineSpacingConfig,
    ParagraphStyleConfig,
};
use docx_rs::*;

/// 定义一个简单的 trait，为 style builder 提供公共字体设置能力
trait StyleBuilderProperties {
    fn fonts(self, fonts: RunFonts) -> Self;
}

// 为 Style 结构体实现这个 trait
impl StyleBuilderProperties for Style {
    fn fonts(self, fonts: RunFonts) -> Self {
        self.fonts(fonts)
    }
}

/// Document style management
#[derive(Clone, Debug)]
pub struct DocxStyles {
    initialized: bool,
    config: Option<DocxConfig>,
}

impl DocxStyles {
    /// Create a new style manager
    pub fn new() -> Self {
        Self {
            initialized: false,
            config: None,
        }
    }

    /// Create a new style manager with custom configuration
    pub fn with_config(config: DocxConfig) -> Self {
        Self {
            initialized: false,
            config: Some(config),
        }
    }

    /// Apply font settings to a style from configuration
    fn apply_font_settings<T: StyleBuilderProperties>(style_builder: T, font: Option<&str>) -> T {
        if let Some(font_name) = font {
            let fonts = RunFonts::new()
                .ascii(font_name)
                .hi_ansi(font_name)
                .east_asia(font_name)
                .cs(font_name);

            style_builder.fonts(fonts)
        } else {
            style_builder
        }
    }

    /// 应用行距设置到样式
    fn apply_line_spacing(style: Style, spacing_config: &LineSpacingConfig) -> Style {
        let mut spacing = LineSpacing::new();

        // 设置行距规则
        if let Some(rule) = &spacing_config.rule {
            let line_rule = match rule.to_lowercase().as_str() {
                "auto" => LineSpacingType::Auto,
                "exact" => LineSpacingType::Exact,
                "atleast" => LineSpacingType::AtLeast,
                _ => LineSpacingType::Auto,
            };
            spacing = spacing.line_rule(line_rule);
        }

        // 设置行距值
        if let Some(value) = spacing_config.value {
            spacing = spacing.line(value);
        }

        // 设置段前距离
        if let Some(before) = spacing_config.before {
            spacing = spacing.before(before);
        }

        // 设置段后距离
        if let Some(after) = spacing_config.after {
            spacing = spacing.after(after);
        }

        // 应用行距设置到样式
        style.line_spacing(spacing)
    }

    /// 应用缩进设置到样式
    fn apply_indent(style: Style, indent_config: &IndentConfig) -> Style {
        let mut result = style;

        // 应用左右缩进
        let left_indent = indent_config.left;
        let right_indent = indent_config.right;

        // 检查特殊缩进类型
        let mut special_indent = None;
        if let Some(special) = &indent_config.special {
            if let Some(value) = indent_config.special_value {
                special_indent = match special.to_lowercase().as_str() {
                    "firstline" => Some(SpecialIndentType::FirstLine(value)),
                    "hanging" => Some(SpecialIndentType::Hanging(value)),
                    _ => None,
                };
            }
        }

        // 获取首行缩进字符数
        let start_chars = None;

        // 应用缩进设置
        if left_indent.is_some()
            || right_indent.is_some()
            || special_indent.is_some()
            || start_chars.is_some()
        {
            result = result.indent(left_indent, special_indent, right_indent, start_chars);
        }

        // 应用首行缩进字符数
        if let Some(chars) = indent_config.first_line_chars {
            result = result.first_line_chars(chars);
        }

        // 应用悬挂缩进字符数
        if let Some(chars) = indent_config.hanging_chars {
            result = result.hanging_chars(chars);
        }

        result
    }

    /// Create a heading style with the specified parameters
    fn create_heading_style(
        name: &str,
        display_name: &str,
        size: usize,
        config: Option<&HeadingStyleConfig>,
    ) -> Style {
        let mut style = Style::new(name, StyleType::Paragraph).name(display_name);

        // Apply configuration if provided
        if let Some(cfg) = config {
            // Apply font size
            style = style.size(cfg.size.unwrap_or(size));

            // Apply font if specified
            if let Some(font) = &cfg.font {
                style = Self::apply_font_settings(style, Some(font));
            }

            // Apply bold if specified
            if cfg.bold.unwrap_or(true) {
                style = style.bold();
            }

            // Apply italic if specified
            if cfg.italic.unwrap_or(false) {
                style = style.italic();
            }

            // Apply color if specified
            if let Some(color) = &cfg.color {
                style = style.color(color);
            }

            // 应用缩进
            if let Some(indent) = &cfg.indent {
                style = Self::apply_indent(style, indent);
            }

            // 应用行距
            if let Some(line_spacing) = &cfg.line_spacing {
                style = Self::apply_line_spacing(style, line_spacing);
            }
        } else {
            // Default behavior
            style = style.size(size).bold();
        }

        style
    }

    /// Create paragraph style with configuration
    fn create_paragraph_style(
        name: &str,
        display_name: &str,
        config: Option<&ParagraphStyleConfig>,
        default_size: Option<usize>,
    ) -> Style {
        let mut style = Style::new(name, StyleType::Paragraph).name(display_name);

        // Apply configuration if provided
        if let Some(cfg) = config {
            // Apply font size
            if let Some(size) = cfg.size.or(default_size) {
                style = style.size(size);
            }

            // Apply font if specified
            if let Some(font) = &cfg.font {
                style = Self::apply_font_settings(style, Some(font));
            }

            // Apply bold if specified
            if cfg.bold.unwrap_or(false) {
                style = style.bold();
            }

            // Apply italic if specified
            if cfg.italic.unwrap_or(false) {
                style = style.italic();
            }

            // Apply color if specified
            if let Some(color) = &cfg.color {
                style = style.color(color);
            }

            // Apply alignment if specified
            if let Some(alignment) = &cfg.alignment {
                let align_type = match alignment.to_lowercase().as_str() {
                    "center" => AlignmentType::Center,
                    "right" => AlignmentType::Right,
                    "justify" => AlignmentType::Justified,
                    _ => AlignmentType::Left,
                };
                style = style.align(align_type);
            }

            // Apply indent if specified
            if let Some(indent_config) = &cfg.indent {
                style = Self::apply_indent(style, indent_config);
            }

            // 应用行距
            if let Some(line_spacing) = &cfg.line_spacing {
                style = Self::apply_line_spacing(style, line_spacing);
            }
        } else if let Some(size) = default_size {
            // Apply default size if provided
            style = style.size(size);
        }

        style
    }

    /// Create character style with configuration
    fn create_character_style(
        name: &str,
        display_name: &str,
        config: Option<&CharacterStyleConfig>,
        default_size: Option<usize>,
    ) -> Style {
        let mut style = Style::new(name, StyleType::Character).name(display_name);

        // Apply configuration if provided
        if let Some(cfg) = config {
            // Apply font size
            if let Some(size) = cfg.size.or(default_size) {
                style = style.size(size);
            }

            // Apply font if specified
            if let Some(font) = &cfg.font {
                style = Self::apply_font_settings(style, Some(font));
            }

            // Apply bold if specified
            if cfg.bold.unwrap_or(false) {
                style = style.bold();
            }

            // Apply italic if specified
            if cfg.italic.unwrap_or(false) {
                style = style.italic();
            }

            // Apply color if specified
            if let Some(color) = &cfg.color {
                style = style.color(color);
            }

            // Apply underline if specified
            if let Some(underline) = &cfg.underline {
                style = style.underline(underline);
            }

            // Apply highlight if specified
            if let Some(highlight) = &cfg.highlight_color {
                style = style.highlight(highlight);
            }
        } else if let Some(size) = default_size {
            // Apply default size if provided
            style = style.size(size);
        }

        style
    }

    /// Initialize all document styles
    pub fn initialize_styles(&self, docx: Docx) -> Docx {
        if self.initialized {
            return docx;
        }

        let config = self.config.as_ref();
        let styles_config = config.and_then(|c| c.styles.as_ref());

        // Heading styles
        let headings_config = styles_config.and_then(|s| s.headings.as_ref());
        let heading1 = Self::create_heading_style(
            "Heading1",
            "Heading 1",
            32,
            headings_config.and_then(|h| h.heading1.as_ref()),
        );
        let heading2 = Self::create_heading_style(
            "Heading2",
            "Heading 2",
            28,
            headings_config.and_then(|h| h.heading2.as_ref()),
        );
        let heading3 = Self::create_heading_style(
            "Heading3",
            "Heading 3",
            26,
            headings_config.and_then(|h| h.heading3.as_ref()),
        );
        let heading4 = Self::create_heading_style(
            "Heading4",
            "Heading 4",
            24,
            headings_config.and_then(|h| h.heading4.as_ref()),
        );
        let heading5 = Self::create_heading_style(
            "Heading5",
            "Heading 5",
            22,
            headings_config.and_then(|h| h.heading5.as_ref()),
        );
        let heading6 = Self::create_heading_style(
            "Heading6",
            "Heading 6",
            20,
            headings_config.and_then(|h| h.heading6.as_ref()),
        );

        // Paragraph styles configuration
        let paragraphs_config = styles_config.and_then(|s| s.paragraphs.as_ref());

        // Default courier font for code
        let default_courier_fonts = RunFonts::new()
            .ascii("Courier New")
            .hi_ansi("Courier New")
            .east_asia("Courier New")
            .cs("Courier New");

        // Code block style
        let code_block_config = paragraphs_config.and_then(|p| p.code_block.as_ref());
        let mut code_block =
            Self::create_paragraph_style("CodeBlock", "Code Block", code_block_config, Some(18));
        if code_block_config.and_then(|c| c.font.as_ref()).is_none() {
            // Apply default courier font if no font specified
            code_block = code_block.fonts(default_courier_fonts.clone());
        }

        // Character styles configuration
        let characters_config = styles_config.and_then(|s| s.characters.as_ref());

        // Code inline style
        let code_inline_config = characters_config.and_then(|c| c.code_inline.as_ref());
        let mut code_inline =
            Self::create_character_style("CodeInline", "Code Inline", code_inline_config, Some(18));
        if code_inline_config.and_then(|c| c.font.as_ref()).is_none() {
            // Apply default courier font if no font specified
            code_inline = code_inline.fonts(default_courier_fonts);
        }

        // Math block style
        let math_block_config = paragraphs_config.and_then(|p| p.math_block.as_ref());
        let math_block =
            Self::create_paragraph_style("MathBlock", "Math Block", math_block_config, None);

        // Apply default center alignment if not specified in config
        let math_block = if math_block_config
            .and_then(|c| c.alignment.as_ref())
            .is_none()
        {
            math_block.align(AlignmentType::Center)
        } else {
            math_block
        };

        // Emphasis style
        let emphasis_config = characters_config.and_then(|c| c.emphasis.as_ref());
        let emphasis = Self::create_character_style("Emphasis", "Emphasis", emphasis_config, None);
        let emphasis = if emphasis_config.is_none()
            || emphasis_config.and_then(|c| c.italic).unwrap_or(true)
        {
            emphasis.italic()
        } else {
            emphasis
        };

        // Strong style
        let strong_config = characters_config.and_then(|c| c.strong.as_ref());
        let strong = Self::create_character_style("Strong", "Strong", strong_config, None);
        let strong =
            if strong_config.is_none() || strong_config.and_then(|c| c.bold).unwrap_or(true) {
                strong.bold()
            } else {
                strong
            };

        // Highlight style
        let highlight_config = characters_config.and_then(|c| c.highlight.as_ref());
        let highlight =
            Self::create_character_style("Highlight", "Highlight", highlight_config, None);
        let highlight = if highlight_config
            .and_then(|c| c.highlight_color.as_ref())
            .is_none()
        {
            highlight.highlight("yellow")
        } else {
            highlight
        };

        // Hyperlink style
        let hyperlink_config = characters_config.and_then(|c| c.hyperlink.as_ref());
        let hyperlink =
            Self::create_character_style("Hyperlink", "Hyperlink", hyperlink_config, None);
        let hyperlink = if hyperlink_config.is_none()
            || (hyperlink_config.and_then(|c| c.color.as_ref()).is_none()
                && hyperlink_config
                    .and_then(|c| c.underline.as_ref())
                    .is_none())
        {
            hyperlink.color("0000FF").underline("single")
        } else {
            hyperlink
        };

        // Blockquote style
        let blockquote_config = paragraphs_config.and_then(|p| p.blockquote.as_ref());
        let blockquote =
            Self::create_paragraph_style("Blockquote", "Block Quote", blockquote_config, None);
        let blockquote = if blockquote_config.and_then(|c| c.indent.as_ref()).is_none() {
            blockquote.indent(Some(720), None, None, None)
        } else {
            blockquote
        };
        let blockquote = if blockquote_config.is_none()
            || blockquote_config.and_then(|c| c.italic).unwrap_or(true)
        {
            blockquote.italic()
        } else {
            blockquote
        };

        // Caption style
        let caption_config = paragraphs_config.and_then(|p| p.caption.as_ref());
        let caption = Self::create_paragraph_style("Caption", "Caption", caption_config, Some(16));
        let caption = if caption_config.and_then(|c| c.alignment.as_ref()).is_none() {
            caption.align(AlignmentType::Center)
        } else {
            caption
        };
        let caption =
            if caption_config.is_none() || caption_config.and_then(|c| c.italic).unwrap_or(true) {
                caption.italic()
            } else {
                caption
            };

        // Table style
        let tables_config = styles_config.and_then(|s| s.tables.as_ref());
        let mut table = Style::new("Table", StyleType::Table).name("Table");

        if let Some(table_config) = tables_config {
            if let Some(alignment) = &table_config.alignment {
                let table_align_type = match alignment.to_lowercase().as_str() {
                    "center" => TableAlignmentType::Center,
                    "right" => TableAlignmentType::Right,
                    _ => TableAlignmentType::Left,
                };
                table = table.table_align(table_align_type);
            } else {
                table = table.table_align(TableAlignmentType::Center);
            }
        } else {
            table = table.table_align(TableAlignmentType::Center);
        }

        docx.add_style(heading1)
            .add_style(heading2)
            .add_style(heading3)
            .add_style(heading4)
            .add_style(heading5)
            .add_style(heading6)
            .add_style(code_block)
            .add_style(code_inline)
            .add_style(math_block)
            .add_style(emphasis)
            .add_style(strong)
            .add_style(highlight)
            .add_style(hyperlink)
            .add_style(blockquote)
            .add_style(caption)
            .add_style(table)
    }
}
