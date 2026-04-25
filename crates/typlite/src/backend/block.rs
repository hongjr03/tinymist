use crate::Result;
use crate::ir::*;
use ecow::EcoString;

use super::{
    BibliographyContext, is_auto_inlines, is_none_inlines, push_css_length, push_css_scale,
    push_html_escaped, push_optional_css_length_value, render_inlines, render_inlines_html,
    render_list, render_math, render_table, render_terms,
};

pub(super) fn render_blocks(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
) -> Result<String> {
    let mut out = String::new();

    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, bibliography, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(rendered.trim_end_matches([' ', '\t']));
    }

    Ok(out)
}

pub(super) fn render_blocks_compact_into(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, bibliography, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push('\n');
        }
        rendered_count += 1;
        out.push_str(rendered.trim_end_matches([' ', '\t']));
    }

    Ok(())
}

fn render_block(
    block: &Block,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_reference_anchors(bibliography.take_reference_anchors(block), indent, out);
    match block {
        Block::Heading(data) => {
            if let Some(id) = &data.id {
                out.push_str(&" ".repeat(indent));
                out.push_str("<a id=\"");
                push_html_escaped(id, out);
                out.push_str("\"></a>\n");
            }
            out.push_str(&" ".repeat(indent));
            out.push_str(&"#".repeat(data.level as usize));
            out.push(' ');
            render_inlines(&data.body, bibliography, out)?;
        }
        Block::Paragraph(data) => {
            out.push_str(&" ".repeat(indent));
            render_inlines(&data.body, bibliography, out)?;
        }
        Block::Quote(data) => render_quote(&data.body, indent, bibliography, out)?,
        Block::Figure(data) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<figure");
            if let Some(alt) = &data.alt {
                out.push_str(" aria-label=\"");
                push_html_escaped(alt, out);
                out.push('"');
            }
            out.push_str(">\n");
            render_blocks_html_into(&data.body, indent, bibliography, out)?;
            if !data.caption.is_empty() {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
                out.push_str("<figcaption>");
                render_inlines_html(&data.caption, bibliography, out)?;
                out.push_str("</figcaption>");
            }
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            out.push_str("</figure>");
        }
        Block::Align(data) => render_align(
            data.alignment.as_deref(),
            &data.body,
            indent,
            bibliography,
            out,
        )?,
        Block::Math(data) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("$$");
            render_math(&data.body, out)?;
            out.push_str("$$");
        }
        Block::Table(data) => {
            render_table(&data.rows, &data.alignments, indent, bibliography, out)?
        }
        Block::Raw(data) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
            if let Some(lang) = &data.lang {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(&data.text);
            if !data.text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
        }
        Block::List(data) => render_list(
            data.ordered,
            data.start,
            data.reversed,
            &data.items,
            indent,
            bibliography,
            out,
        )?,
        Block::Columns(data) => render_columns(data, indent, bibliography, out)?,
        Block::Move(data) => render_move_block(data, indent, bibliography, out)?,
        Block::Pad(data) => render_pad_block(data, indent, bibliography, out)?,
        Block::Rotate(data) => render_rotate_block(data, indent, bibliography, out)?,
        Block::Scale(data) => render_scale_block(data, indent, bibliography, out)?,
        Block::Skew(data) => render_skew_block(data, indent, bibliography, out)?,
        Block::Stack(data) => render_stack(data, indent, bibliography, out)?,
        Block::Block(data) => render_blocks_compact_into(&data.body, indent, bibliography, out)?,
        Block::Title(data) => render_blocks_into(&data.body, indent, bibliography, out)?,
        Block::Terms(data) => render_terms(&data.items, indent, bibliography, out)?,
        Block::Colbreak(_) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<div style=\"break-after: column\"></div>");
        }
        Block::V(data) => render_vertical_space(data, indent, out)?,
        Block::Parbreak(_) => {}
        Block::Outline(data) => {
            if !is_auto_inlines(&data.title)
                && !is_none_inlines(&data.title)
                && !data.title.is_empty()
            {
                out.push_str(&" ".repeat(indent));
                render_inlines(&data.title, bibliography, out)?;
            }
        }
        Block::Pagebreak(_) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<div style=\"break-after: page\"></div>");
        }
        Block::Bibliography(data) => render_bibliography(data, bibliography, indent, out)?,
    }

    Ok(())
}

pub(super) fn render_blocks_into(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, bibliography, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(rendered.trim_end_matches([' ', '\t']));
    }

    Ok(())
}

fn render_reference_anchors(anchors: Vec<EcoString>, indent: usize, out: &mut String) {
    for anchor in anchors {
        out.push_str(&" ".repeat(indent));
        out.push_str("<a id=\"");
        push_html_escaped(&anchor, out);
        out.push_str("\"></a>\n");
    }
}

pub(super) fn render_blocks_html_into(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block_html(block, indent, bibliography, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push('\n');
        }
        rendered_count += 1;
        out.push_str(&rendered);
    }

    Ok(())
}

fn render_block_html(
    block: &Block,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    match block {
        Block::Paragraph(data) => {
            out.push_str(&" ".repeat(indent));
            render_inlines_html(&data.body, bibliography, out)
        }
        Block::Raw(data) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<pre><code");
            if let Some(lang) = &data.lang {
                out.push_str(" class=\"language-");
                push_html_escaped(lang, out);
                out.push('"');
            }
            out.push('>');
            push_html_escaped(&data.text, out);
            out.push_str("</code></pre>");
            Ok(())
        }
        _ => render_block(block, indent, bibliography, out),
    }
}

fn render_align(
    alignment: Option<&str>,
    body: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(text_align) = alignment.and_then(css_text_align) else {
        return render_blocks_into(body, indent, bibliography, out);
    };

    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"text-align: ");
    out.push_str(text_align);
    out.push_str("\">\n");
    render_blocks_html_into(body, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_columns(
    data: &ColumnsBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"");
    let mut has_style = false;
    if let Some(count) = data.count.as_deref().filter(|value| !value.is_empty()) {
        has_style = true;
        out.push_str("column-count: ");
        push_html_escaped(count, out);
    }
    if let Some(gutter) = data.gutter.as_deref().filter(|value| !value.is_empty()) {
        if has_style {
            out.push_str("; ");
        }
        out.push_str("column-gap: ");
        push_css_length(gutter, out);
    }
    out.push_str("\">\n");
    render_blocks_html_into(&data.body, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_stack(
    data: &StackBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"display: flex");
    if let Some(direction) = data.dir.as_deref().and_then(css_stack_direction) {
        out.push_str("; flex-direction: ");
        out.push_str(direction);
    }
    if let Some(spacing) = data.spacing.as_deref().filter(|value| !value.is_empty()) {
        out.push_str("; gap: ");
        push_css_length(spacing, out);
    }
    out.push_str("\">\n");
    render_blocks_html_into(&data.children, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_vertical_space(data: &VBlock, indent: usize, out: &mut String) -> Result<()> {
    let Some(amount) = data.amount.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"height: ");
    push_css_length(amount, out);
    out.push_str("\"></div>");

    Ok(())
}

fn render_bibliography(
    data: &BibliographyBlock,
    bibliography: &BibliographyContext,
    indent: usize,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<section data-typlite-bibliography=\"true\">");
    if let Some(title) = bibliography_title(data, bibliography)? {
        out.push('\n');
        out.push_str(&" ".repeat(indent + 2));
        out.push_str("<h2>");
        push_html_escaped(&title, out);
        out.push_str("</h2>");
    }
    if bibliography.is_empty() {
        out.push('\n');
        out.push_str(&" ".repeat(indent + 2));
        out.push_str("<!-- typlite-bibliography: no rendered entries -->");
    } else {
        for (key, entry) in bibliography.ordered_entries() {
            out.push('\n');
            out.push_str(&" ".repeat(indent + 2));
            out.push_str("<div id=\"ref-");
            push_html_escaped(key, out);
            out.push_str("\" class=\"csl-entry\">");
            push_html_escaped(entry, out);
            for citation_index in 1..=bibliography.citation_count(key) {
                out.push_str(" <a href=\"#cite-");
                push_html_escaped(key, out);
                out.push('-');
                out.push_str(&citation_index.to_string());
                out.push_str(
                    "\" class=\"csl-backref\" aria-label=\"Back to citation\">&#8617;</a>",
                );
            }
            out.push_str("</div>");
        }
    }
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</section>");
    Ok(())
}

fn bibliography_title(
    data: &BibliographyBlock,
    bibliography: &BibliographyContext,
) -> Result<Option<String>> {
    if data.title.is_empty() {
        return Ok(None);
    }

    let mut rendered = String::new();
    render_inlines(&data.title, bibliography, &mut rendered)?;
    if rendered.is_empty() || rendered == "auto" || rendered == "none" {
        Ok(None)
    } else {
        Ok(Some(rendered))
    }
}

fn render_pad_block(
    data: &PadBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block");
        push_optional_css_length_value(data.left.as_deref(), "padding-left", out);
        push_optional_css_length_value(data.top.as_deref(), "padding-top", out);
        push_optional_css_length_value(data.right.as_deref(), "padding-right", out);
        push_optional_css_length_value(data.bottom.as_deref(), "padding-bottom", out);
    })
}

fn render_move_block(
    data: &MoveBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: translate(");
        push_css_length(data.dx.as_deref().unwrap_or("0pt"), out);
        out.push_str(", ");
        push_css_length(data.dy.as_deref().unwrap_or("0pt"), out);
        out.push(')');
    })
}

fn render_rotate_block(
    data: &RotateBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: rotate(");
        push_html_escaped(data.angle.as_deref().unwrap_or("0deg"), out);
        out.push(')');
    })
}

fn render_scale_block(
    data: &ScaleBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: scale(");
        push_css_scale(
            data.x
                .as_deref()
                .or(data.factor.as_deref())
                .unwrap_or("100%"),
            out,
        );
        out.push_str(", ");
        push_css_scale(
            data.y
                .as_deref()
                .or(data.factor.as_deref())
                .unwrap_or("100%"),
            out,
        );
        out.push(')');
    })
}

fn render_skew_block(
    data: &SkewBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: skew(");
        push_html_escaped(data.ax.as_deref().unwrap_or("0deg"), out);
        out.push_str(", ");
        push_html_escaped(data.ay.as_deref().unwrap_or("0deg"), out);
        out.push(')');
    })
}

fn render_layout_div(
    body: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
    push_style: impl FnOnce(&mut String),
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"");
    push_style(out);
    out.push_str("\">\n");
    render_blocks_html_into(body, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");
    Ok(())
}

fn css_stack_direction(value: &str) -> Option<&'static str> {
    match value.trim() {
        "ttb" => Some("column"),
        "btt" => Some("column-reverse"),
        "ltr" => Some("row"),
        "rtl" => Some("row-reverse"),
        _ => None,
    }
}

fn css_text_align(value: &str) -> Option<&'static str> {
    let value = value.trim();
    if value.contains("center") || value.contains("horizon") {
        Some("center")
    } else if value.contains("right") || value.contains("end") {
        Some("right")
    } else if value.contains("left") || value.contains("start") {
        Some("left")
    } else {
        None
    }
}

fn render_quote(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let body = render_blocks(blocks, 0, bibliography)?;
    let prefix = format!("{}> ", " ".repeat(indent));

    for (index, line) in body.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&prefix);
        out.push_str(line);
    }

    Ok(())
}
