# DOCX Style Customization Guide

## Overview

Typlite now supports customizing DOCX document styles and numbering formats through TOML configuration files. This gives you complete control over the appearance of generated DOCX documents, including fonts, colors, sizes, indents, and list numbering formats.

## Usage

Use the `--docx-style-path` parameter in the command line to specify the path to your custom style configuration file:

```bash
typlite --input your_document.typ --output your_document.docx --docx-style-path path/to/your/style.toml
```

## Configuration File Structure

The configuration file uses TOML format and is divided into two main sections:

1. `styles` - Controls various document styles, including headings, paragraphs, character styles, and table styles
2. `numbering` - Controls the appearance and numbering methods of ordered and unordered lists

### Style Configuration

Style configuration is divided into four subsections:

- `styles.headings` - Heading styles (levels 1-6)
- `styles.paragraphs` - Paragraph styles (code blocks, blockquotes, etc.)
- `styles.characters` - Character styles (inline code, emphasis, etc.)
- `styles.tables` - Table styles

#### Heading Styles

You can define styles for each heading level (heading1 through heading6):

```toml
[styles.headings.heading1]
size = 36            # Font size (36 = 18pt)
font = "Arial"       # Font name
bold = true          # Bold or not
italic = false       # Italic or not
color = "000080"     # Font color (RGB format)
```

#### Paragraph Styles

You can define styles for special paragraph types:

```toml
[styles.paragraphs.code_block]
size = 20
font = "Consolas"
color = "404040"
indent = 720         # Indent size (720 = 0.5 inches)
```

Configurable paragraph types include:

- `code_block` - Code blocks
- `blockquote` - Block quotes
- `caption` - Figure captions
- `math_block` - Math formula blocks

#### Character Styles

You can define styles for inline elements:

```toml
[styles.characters.code_inline]
size = 20
font = "Consolas"
color = "800080"
highlight_color = "F0F0F0"
```

Configurable character styles include:

- `code_inline` - Inline code
- `emphasis` - Emphasized text (typically italic)
- `strong` - Strong text (typically bold)
- `highlight` - Highlighted text
- `hyperlink` - Hyperlinks

#### Table Styles

```toml
[styles.tables]
alignment = "center" # Table alignment
```

### Numbering Configuration

Numbering configuration allows you to customize the appearance of ordered and unordered lists:

#### Ordered Lists

```toml
[numbering.ordered.levels.0]  # First level list
format = "decimal"     # Number format: "decimal", "lowerLetter", "upperLetter", "lowerRoman", "upperRoman"
text = "%1."           # Display format
indent = 720           # Indent size
hanging_indent = 360   # Hanging indent size
```

Available number formats include:

- `decimal` - Numbers (1, 2, 3...)
- `lowerLetter` - Lowercase letters (a, b, c...)
- `upperLetter` - Uppercase letters (A, B, C...)
- `lowerRoman` - Lowercase Roman numerals (i, ii, iii...)
- `upperRoman` - Uppercase Roman numerals (I, II, III...)

#### Unordered Lists

```toml
[numbering.unordered.levels.0]  # First level list
bullet = "•"           # Bullet symbol
indent = 720           # Indent size
hanging_indent = 360   # Hanging indent size
```

## Example Configuration

See `docx_style_example.toml` for a complete example configuration and options.

## Notes

- All configuration options are optional; unspecified options will use default values
- Font sizes are specified in half-points (e.g., 24 = 12pt)
- Colors should be specified in RGB hexadecimal format (e.g., "FF0000" for red)
- Indents are specified in twips (1/20 of a point) (e.g., 720 = 0.5 inches)
