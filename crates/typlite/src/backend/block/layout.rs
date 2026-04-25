use crate::Result;
use crate::ir::*;

use super::{
    BibliographyContext, push_css_length, push_css_scale, push_html_escaped,
    push_optional_css_length_value, render_blocks_html_into, render_blocks_into,
};

pub(super) fn render_align(
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

pub(super) fn render_columns(
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

pub(super) fn render_stack(
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

pub(super) fn render_vertical_space(data: &VBlock, indent: usize, out: &mut String) -> Result<()> {
    let Some(amount) = data.amount.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"height: ");
    push_css_length(amount, out);
    out.push_str("\"></div>");

    Ok(())
}

pub(super) fn render_pad_block(
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

pub(super) fn render_move_block(
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

pub(super) fn render_rotate_block(
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

pub(super) fn render_scale_block(
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

pub(super) fn render_skew_block(
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
