//! Experimental backends for typlite IR.

use crate::Result;
use crate::ir::{
    Block, Document, ElementFieldValue, FrameImage, Inline, InlineElementData, MathNode, MathValue,
    TableAlign, TableRow, TermItem,
};
use tinymist_std::error::prelude::*;

/// Renders a document IR as Markdown.
pub fn render_markdown(doc: &Document) -> Result<String> {
    render_blocks(&doc.blocks, 0)
}

fn render_blocks(blocks: &[Block], indent: usize) -> Result<String> {
    let mut out = String::new();

    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(&rendered);
    }

    Ok(out)
}

fn render_block(block: &Block, indent: usize, out: &mut String) -> Result<()> {
    match block {
        Block::Heading { level, body } => {
            out.push_str(&" ".repeat(indent));
            out.push_str(&"#".repeat(*level as usize));
            out.push(' ');
            render_inlines(body, out)?;
        }
        Block::Paragraph(body) => {
            out.push_str(&" ".repeat(indent));
            render_inlines(body, out)?;
        }
        Block::Quote(blocks) => render_quote(blocks, indent, out)?,
        Block::Figure { body, caption, alt } => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<figure");
            if let Some(alt) = alt {
                out.push_str(" aria-label=\"");
                push_html_escaped(alt, out);
                out.push('"');
            }
            out.push_str(">\n");
            render_blocks_html_into(body, indent, out)?;
            if !caption.is_empty() {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
                out.push_str("<figcaption>");
                render_inlines_html(caption, out)?;
                out.push_str("</figcaption>");
            }
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            out.push_str("</figure>");
        }
        Block::Align { alignment, body } => render_align(alignment.as_deref(), body, indent, out)?,
        Block::Math(body) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("$$");
            render_math(body, out)?;
            out.push_str("$$");
        }
        Block::Table { rows, alignments } => render_table(rows, alignments, indent, out)?,
        Block::Raw { lang, text } => {
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
            if let Some(lang) = lang {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
        }
        Block::List {
            ordered,
            start,
            reversed,
            items,
            ..
        } => render_list(*ordered, *start, *reversed, items, indent, out)?,
        Block::Columns(data) => render_columns(data, indent, out)?,
        Block::Block(data) | Block::Stack(data) | Block::Title(data) => {
            render_blocks_into(&data.body, indent, out)?
        }
        Block::Terms { items } => render_terms(items, indent, out)?,
        Block::Colbreak(_) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<div style=\"break-after: column\"></div>");
        }
        Block::Parbreak(_) | Block::V(_) => {}
        Block::Outline(data) => {
            if let Some(title) = data.field("title") {
                out.push_str(&" ".repeat(indent));
                render_field_value(title, out)?;
            }
        }
        Block::Pagebreak(_) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<div style=\"break-after: page\"></div>");
        }
        Block::Bibliography(_) => {
            bail!("typlite markdown bibliography rendering is not implemented")
        }
    }

    Ok(())
}

fn render_blocks_into(blocks: &[Block], indent: usize, out: &mut String) -> Result<()> {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(&rendered);
    }

    Ok(())
}

fn render_blocks_html_into(blocks: &[Block], indent: usize, out: &mut String) -> Result<()> {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block_html(block, indent, &mut rendered)?;
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

fn render_block_html(block: &Block, indent: usize, out: &mut String) -> Result<()> {
    match block {
        Block::Paragraph(body) => {
            out.push_str(&" ".repeat(indent));
            render_inlines_html(body, out)
        }
        Block::Raw { lang, text } => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<pre><code");
            if let Some(lang) = lang {
                out.push_str(" class=\"language-");
                push_html_escaped(lang, out);
                out.push('"');
            }
            out.push('>');
            push_html_escaped(text, out);
            out.push_str("</code></pre>");
            Ok(())
        }
        _ => render_block(block, indent, out),
    }
}

fn render_align(
    alignment: Option<&str>,
    body: &[Block],
    indent: usize,
    out: &mut String,
) -> Result<()> {
    let Some(text_align) = alignment.and_then(css_text_align) else {
        return render_blocks_into(body, indent, out);
    };

    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"text-align: ");
    out.push_str(text_align);
    out.push_str("\">\n");
    render_blocks_html_into(body, indent + 2, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_columns(
    data: &crate::ir::BlockElementData,
    indent: usize,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"");
    let mut has_style = false;
    if let Some(count) = data.scalar("count").filter(|value| !value.is_empty()) {
        has_style = true;
        out.push_str("column-count: ");
        push_html_escaped(count, out);
    }
    if let Some(gutter) = data.scalar("gutter").filter(|value| !value.is_empty()) {
        if has_style {
            out.push_str("; ");
        }
        out.push_str("column-gap: ");
        push_css_length(gutter, out);
    }
    out.push_str("\">\n");
    render_blocks_html_into(&data.body, indent + 2, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
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

fn push_css_length(value: &str, out: &mut String) {
    if value.contains('+') || value.contains('-') {
        out.push_str("calc(");
        push_html_escaped(value, out);
        out.push(')');
    } else {
        push_html_escaped(value, out);
    }
}

fn render_quote(blocks: &[Block], indent: usize, out: &mut String) -> Result<()> {
    let body = render_blocks(blocks, 0)?;
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

fn render_table(
    rows: &[TableRow],
    alignments: &[TableAlign],
    indent: usize,
    out: &mut String,
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    let columns = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    if columns == 0 {
        return Ok(());
    }

    if requires_html_table(rows) {
        return render_html_table(rows, indent, out);
    }

    render_table_row(&rows[0], columns, indent, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push('|');
    for index in 0..columns {
        out.push(' ');
        out.push_str(table_align_marker(
            alignments
                .get(index)
                .copied()
                .unwrap_or(TableAlign::Default),
        ));
        out.push_str(" |");
    }

    for row in &rows[1..] {
        out.push('\n');
        render_table_row(row, columns, indent, out)?;
    }

    Ok(())
}

fn requires_html_table(rows: &[TableRow]) -> bool {
    rows.iter().any(|row| {
        row.cells
            .iter()
            .any(|cell| cell.colspan != 1 || cell.rowspan != 1 || cell.align != TableAlign::Default)
    })
}

fn render_html_table(rows: &[TableRow], indent: usize, out: &mut String) -> Result<()> {
    let indentation = " ".repeat(indent);
    out.push_str(&indentation);
    out.push_str("<table>");

    for row in rows {
        out.push('\n');
        out.push_str(&indentation);
        out.push_str("  <tr>");
        for cell in &row.cells {
            out.push('\n');
            out.push_str(&indentation);
            out.push_str("    <td");
            if cell.colspan > 1 {
                out.push_str(" colspan=\"");
                out.push_str(&cell.colspan.to_string());
                out.push('"');
            }
            if cell.rowspan > 1 {
                out.push_str(" rowspan=\"");
                out.push_str(&cell.rowspan.to_string());
                out.push('"');
            }
            if let Some(align) = table_align_style(cell.align) {
                out.push_str(" style=\"text-align: ");
                out.push_str(align);
                out.push('"');
            }
            out.push('>');
            render_inlines_html(&cell.body, out)?;
            out.push_str("</td>");
        }
        out.push('\n');
        out.push_str(&indentation);
        out.push_str("  </tr>");
    }

    out.push('\n');
    out.push_str(&indentation);
    out.push_str("</table>");
    Ok(())
}

fn table_align_style(align: TableAlign) -> Option<&'static str> {
    match align {
        TableAlign::Default => None,
        TableAlign::Left => Some("left"),
        TableAlign::Center => Some("center"),
        TableAlign::Right => Some("right"),
    }
}

fn table_align_marker(align: TableAlign) -> &'static str {
    match align {
        TableAlign::Default => "---",
        TableAlign::Left => ":---",
        TableAlign::Center => ":---:",
        TableAlign::Right => "---:",
    }
}

fn render_table_row(row: &TableRow, columns: usize, indent: usize, out: &mut String) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push('|');
    for index in 0..columns {
        out.push(' ');
        if let Some(cell) = row.cells.get(index) {
            render_inlines(&cell.body, out)?;
        }
        out.push_str(" |");
    }

    Ok(())
}

fn render_list(
    ordered: bool,
    start: Option<i64>,
    reversed: bool,
    items: &[crate::ir::ListItem],
    indent: usize,
    out: &mut String,
) -> Result<()> {
    let mut next_number = if reversed {
        start.unwrap_or(items.len() as i64)
    } else {
        start.unwrap_or(1)
    };

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        let marker = if ordered {
            let number = item
                .number
                .as_ref()
                .and_then(|number| number.parse::<i64>().ok())
                .unwrap_or(next_number);
            next_number = if reversed { number - 1 } else { number + 1 };
            format!("{number}.")
        } else {
            "-".to_owned()
        };
        let prefix = format!("{}{} ", " ".repeat(indent), marker);
        let continuation = indent + marker.len() + 1;
        let body = render_blocks(&item.body, continuation)?;

        if body.is_empty() {
            out.push_str(&prefix);
            continue;
        }

        let mut lines = body.lines();
        if let Some(first) = lines.next() {
            out.push_str(&prefix);
            out.push_str(first.trim_start());
        }

        for line in lines {
            out.push('\n');
            out.push_str(line);
        }
    }

    Ok(())
}

fn render_terms(items: &[TermItem], indent: usize, out: &mut String) -> Result<()> {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
            out.push('\n');
        }

        out.push_str(&" ".repeat(indent));
        render_inlines(&item.term, out)?;

        let description = render_blocks(&item.description, indent + 2)?;
        if !description.is_empty() {
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            out.push_str(": ");
            let mut lines = description.lines();
            if let Some(first) = lines.next() {
                out.push_str(first.trim_start());
            }
            for line in lines {
                out.push('\n');
                out.push_str(line);
            }
        }
    }

    Ok(())
}

fn render_inlines(nodes: &[Inline], out: &mut String) -> Result<()> {
    for node in nodes {
        match node {
            Inline::Text(text) => out.push_str(text),
            Inline::Emph(children) => {
                out.push('*');
                render_inlines(children, out)?;
                out.push('*');
            }
            Inline::Strong(children) => {
                out.push_str("**");
                render_inlines(children, out)?;
                out.push_str("**");
            }
            Inline::Link { dest, body } => {
                out.push('[');
                render_inlines(body, out)?;
                out.push_str("](");
                out.push_str(dest);
                out.push(')');
            }
            Inline::Strike(children) => {
                out.push_str("~~");
                render_inlines(children, out)?;
                out.push_str("~~");
            }
            Inline::Sub(children) => {
                out.push_str("<sub>");
                render_inlines(children, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(children) => {
                out.push_str("<sup>");
                render_inlines(children, out)?;
                out.push_str("</sup>");
            }
            Inline::Math(children) => {
                out.push('$');
                render_math(children, out)?;
                out.push('$');
            }
            Inline::Linebreak => out.push('\n'),
            Inline::Frame(frame) => render_frame_image("frame", frame, out)?,
            Inline::Raw { text, .. } => {
                out.push('`');
                out.push_str(text);
                out.push('`');
            }
            Inline::Box(data)
            | Inline::Move(data)
            | Inline::Pad(data)
            | Inline::Place(data)
            | Inline::Repeat(data)
            | Inline::Rotate(data)
            | Inline::Scale(data)
            | Inline::Skew(data)
            | Inline::TableCell(data)
            | Inline::TableFooter(data)
            | Inline::TableHeader(data)
            | Inline::GridCell(data)
            | Inline::GridFooter(data)
            | Inline::GridHeader(data)
            | Inline::ParLine(data)
            | Inline::PdfArtifact(data)
            | Inline::RawLine(data) => render_inlines(&data.body, out)?,
            Inline::Quote(data) => render_inline_quote(data, out)?,
            Inline::Circle(data)
            | Inline::Curve(data)
            | Inline::Ellipse(data)
            | Inline::Line(data)
            | Inline::Path(data)
            | Inline::Polygon(data)
            | Inline::Rect(data)
            | Inline::Square(data) => render_element_frame(node, data, out)?,
            Inline::Cite(data) => render_cite(data, out)?,
            Inline::Document(_) | Inline::Hide(_) | Inline::Metadata(_) | Inline::Page(_) => {}
            Inline::FigureCaption(data) => render_inlines(&data.body, out)?,
            Inline::FootnoteEntry(_) => {}
            Inline::Footnote(data) => {
                out.push_str("^[");
                render_inlines(&data.body, out)?;
                out.push(']');
            }
            Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::PlaceFlush(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
            Inline::H(_) => out.push(' '),
            Inline::Highlight(data) => {
                out.push_str("<mark>");
                render_inlines(&data.body, out)?;
                out.push_str("</mark>");
            }
            Inline::Image(data) => render_image(data, out)?,
            Inline::MathAccent(data)
            | Inline::MathAttach(data)
            | Inline::MathBinom(data)
            | Inline::MathCancel(data)
            | Inline::MathCases(data)
            | Inline::MathClass(data)
            | Inline::MathFrac(data)
            | Inline::MathLimits(data)
            | Inline::MathLr(data)
            | Inline::MathMat(data)
            | Inline::MathMid(data)
            | Inline::MathOp(data)
            | Inline::MathOverbrace(data)
            | Inline::MathOverbracket(data)
            | Inline::MathOverline(data)
            | Inline::MathOverparen(data)
            | Inline::MathOvershell(data)
            | Inline::MathPrimes(data)
            | Inline::MathRoot(data)
            | Inline::MathScripts(data)
            | Inline::MathStretch(data)
            | Inline::MathUnderbrace(data)
            | Inline::MathUnderbracket(data)
            | Inline::MathUnderline(data)
            | Inline::MathUnderparen(data)
            | Inline::MathUndershell(data)
            | Inline::MathVec(data)
            | Inline::CurveClose(data)
            | Inline::CurveCubic(data)
            | Inline::CurveLine(data)
            | Inline::CurveMove(data)
            | Inline::CurveQuad(data) => render_structured_inline(node, data, out)?,
            Inline::OutlineEntry(data) => render_inlines(&data.body, out)?,
            Inline::Overline(data) => {
                out.push_str("<span style=\"text-decoration: overline\">");
                render_inlines(&data.body, out)?;
                out.push_str("</span>");
            }
            Inline::PdfAttach(_) | Inline::PdfEmbed(_) => {
                bail!("typlite markdown PDF embedding rendering is not implemented")
            }
            Inline::Ref(data) => render_ref(data, out)?,
            Inline::Smallcaps(data) => {
                out.push_str("<span style=\"font-variant: small-caps\">");
                render_inlines(&data.body, out)?;
                out.push_str("</span>");
            }
            Inline::Smartquote(data) => render_smartquote(data, out)?,
            Inline::Underline(data) => {
                out.push_str("<u>");
                render_inlines(&data.body, out)?;
                out.push_str("</u>");
            }
        }
    }

    Ok(())
}

fn render_inlines_html(nodes: &[Inline], out: &mut String) -> Result<()> {
    for node in nodes {
        match node {
            Inline::Text(text) => push_html_escaped(text, out),
            Inline::Emph(children) => {
                out.push_str("<em>");
                render_inlines_html(children, out)?;
                out.push_str("</em>");
            }
            Inline::Strong(children) => {
                out.push_str("<strong>");
                render_inlines_html(children, out)?;
                out.push_str("</strong>");
            }
            Inline::Link { dest, body } => {
                out.push_str("<a href=\"");
                push_html_escaped(dest, out);
                out.push_str("\">");
                render_inlines_html(body, out)?;
                out.push_str("</a>");
            }
            Inline::Strike(children) => {
                out.push_str("<del>");
                render_inlines_html(children, out)?;
                out.push_str("</del>");
            }
            Inline::Sub(children) => {
                out.push_str("<sub>");
                render_inlines_html(children, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(children) => {
                out.push_str("<sup>");
                render_inlines_html(children, out)?;
                out.push_str("</sup>");
            }
            Inline::Raw { text, .. } => {
                out.push_str("<code>");
                push_html_escaped(text, out);
                out.push_str("</code>");
            }
            Inline::Linebreak => out.push_str("<br>"),
            Inline::Math(math) => {
                out.push('$');
                render_math(math, out)?;
                out.push('$');
            }
            Inline::Frame(frame) => render_frame_image("frame", frame, out)?,
            Inline::Footnote(data) => {
                out.push_str("<sup>");
                render_inlines_html(&data.body, out)?;
                out.push_str("</sup>");
            }
            Inline::Highlight(data) => {
                out.push_str("<mark>");
                render_inlines_html(&data.body, out)?;
                out.push_str("</mark>");
            }
            Inline::Image(data) => render_image_html(data, out)?,
            Inline::Overline(data) => {
                out.push_str("<span style=\"text-decoration: overline\">");
                render_inlines_html(&data.body, out)?;
                out.push_str("</span>");
            }
            Inline::Smallcaps(data) => {
                out.push_str("<span style=\"font-variant: small-caps\">");
                render_inlines_html(&data.body, out)?;
                out.push_str("</span>");
            }
            Inline::Underline(data) => {
                out.push_str("<u>");
                render_inlines_html(&data.body, out)?;
                out.push_str("</u>");
            }
            Inline::Box(data)
            | Inline::Move(data)
            | Inline::Pad(data)
            | Inline::Place(data)
            | Inline::Repeat(data)
            | Inline::Rotate(data)
            | Inline::Scale(data)
            | Inline::Skew(data)
            | Inline::TableCell(data)
            | Inline::TableFooter(data)
            | Inline::TableHeader(data)
            | Inline::GridCell(data)
            | Inline::GridFooter(data)
            | Inline::GridHeader(data)
            | Inline::ParLine(data)
            | Inline::PdfArtifact(data)
            | Inline::RawLine(data)
            | Inline::FigureCaption(data)
            | Inline::OutlineEntry(data) => render_inlines_html(&data.body, out)?,
            Inline::Quote(data) => render_inline_quote_html(data, out)?,
            Inline::FootnoteEntry(_) => {}
            Inline::Circle(data)
            | Inline::Curve(data)
            | Inline::Ellipse(data)
            | Inline::Line(data)
            | Inline::Path(data)
            | Inline::Polygon(data)
            | Inline::Rect(data)
            | Inline::Square(data) => render_element_frame(node, data, out)?,
            Inline::Cite(data) => render_cite(data, out)?,
            Inline::Document(_) | Inline::Hide(_) | Inline::Metadata(_) | Inline::Page(_) => {}
            Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::PlaceFlush(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
            Inline::H(_) => out.push(' '),
            Inline::PdfAttach(_) | Inline::PdfEmbed(_) => {
                bail!("typlite markdown PDF embedding rendering is not implemented")
            }
            Inline::Ref(data) => render_ref(data, out)?,
            Inline::Smartquote(data) => render_smartquote_html(data, out)?,
            Inline::MathAccent(_)
            | Inline::MathAttach(_)
            | Inline::MathBinom(_)
            | Inline::MathCancel(_)
            | Inline::MathCases(_)
            | Inline::MathClass(_)
            | Inline::MathFrac(_)
            | Inline::MathLimits(_)
            | Inline::MathLr(_)
            | Inline::MathMat(_)
            | Inline::MathMid(_)
            | Inline::MathOp(_)
            | Inline::MathOverbrace(_)
            | Inline::MathOverbracket(_)
            | Inline::MathOverline(_)
            | Inline::MathOverparen(_)
            | Inline::MathOvershell(_)
            | Inline::MathPrimes(_)
            | Inline::MathRoot(_)
            | Inline::MathScripts(_)
            | Inline::MathStretch(_)
            | Inline::MathUnderbrace(_)
            | Inline::MathUnderbracket(_)
            | Inline::MathUnderline(_)
            | Inline::MathUnderparen(_)
            | Inline::MathUndershell(_)
            | Inline::MathVec(_)
            | Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_) => {
                bail!("typlite HTML table cell rendering is not implemented for generated inline")
            }
        }
    }

    Ok(())
}

fn render_math(node: &MathNode, out: &mut String) -> Result<()> {
    match node.func.as_str() {
        "sequence" => render_math_nodes(math_nodes(node, "children")?, out),
        "text" | "symbol" => {
            out.push_str(math_scalar(node, "text")?);
            Ok(())
        }
        "space" => {
            out.push(' ');
            Ok(())
        }
        "accent" => render_math_accent(node, out),
        "attach" => render_math_attach(node, out),
        "binom" => render_math_two_arg_command(node, "binom", "upper", "lower", out),
        "cancel" => render_math_one_arg_command(node, "cancel", "body", out),
        "cases" => render_math_cases(node, out),
        "class" | "limits" | "lr" | "scripts" | "stretch" | "mid" => {
            render_math(math_child(node, "body")?, out)
        }
        "frac" => render_math_two_arg_command(node, "frac", "num", "denom", out),
        "mat" => render_math_matrix(node, out),
        "op" => render_math_op(node, out),
        "overbrace" => render_math_annotated_command(node, "overbrace", "body", out),
        "overbracket" => render_math_annotated_command(node, "overbracket", "body", out),
        "overline" => render_math_one_arg_command(node, "overline", "body", out),
        "overparen" => render_math_annotated_command(node, "overparen", "body", out),
        "overshell" => render_math_annotation_command(node, "overset", out),
        "primes" => render_math_primes(node, out),
        "root" => render_math_root(node, out),
        "underbrace" => render_math_annotated_command(node, "underbrace", "body", out),
        "underbracket" => render_math_annotated_command(node, "underbracket", "body", out),
        "underline" => render_math_one_arg_command(node, "underline", "body", out),
        "underparen" => render_math_annotated_command(node, "underparen", "body", out),
        "undershell" => render_math_annotation_command(node, "underset", out),
        "vec" => render_math_one_arg_nodes_command(node, "vec", "children", out),
        func => bail!("typlite markdown math rendering is not implemented for `{func}`"),
    }
}

fn render_math_nodes(nodes: &[MathNode], out: &mut String) -> Result<()> {
    for node in nodes {
        render_math(node, out)?;
    }
    Ok(())
}

fn render_math_accent(node: &MathNode, out: &mut String) -> Result<()> {
    let command = match math_scalar(node, "accent")? {
        "\u{302}" => "hat",
        "\u{303}" => "tilde",
        "\u{304}" => "bar",
        "\u{307}" => "dot",
        "\u{308}" => "ddot",
        "\u{20d7}" => "vec",
        accent => bail!("typlite markdown math accent `{accent}` is not implemented"),
    };
    render_math_one_arg_command(node, command, "base", out)
}

fn render_math_attach(node: &MathNode, out: &mut String) -> Result<()> {
    render_math(math_child(node, "base")?, out)?;
    if let Some(bottom) = math_optional_child(node, "b")? {
        out.push_str("_{");
        render_math(bottom, out)?;
        out.push('}');
    }
    if let Some(top) = math_optional_child(node, "t")? {
        out.push_str("^{");
        render_math(top, out)?;
        out.push('}');
    }
    Ok(())
}

fn render_math_cases(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\begin{cases}");
    for (index, child) in math_nodes(node, "children")?.iter().enumerate() {
        if index > 0 {
            out.push_str(r" \\ ");
        }
        render_math(child, out)?;
    }
    out.push_str(r"\end{cases}");
    Ok(())
}

fn render_math_matrix(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\begin{matrix}");
    for (row_index, row) in math_rows(node, "rows")?.iter().enumerate() {
        if row_index > 0 {
            out.push_str(r" \\ ");
        }
        for (cell_index, cell) in row.iter().enumerate() {
            if cell_index > 0 {
                out.push_str(" & ");
            }
            render_math(cell, out)?;
        }
    }
    out.push_str(r"\end{matrix}");
    Ok(())
}

fn render_math_op(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\operatorname{");
    render_math(math_child(node, "text")?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_primes(node: &MathNode, out: &mut String) -> Result<()> {
    let count = math_scalar(node, "count")?
        .parse::<usize>()
        .context_ut("math.primes count must be a number")?;
    for _ in 0..count {
        out.push('\'');
    }
    Ok(())
}

fn render_math_root(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\sqrt");
    if let Some(index) = math_optional_child(node, "index")? {
        out.push('[');
        render_math(index, out)?;
        out.push(']');
    }
    out.push('{');
    render_math(math_child(node, "radicand")?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_one_arg_command(
    node: &MathNode,
    command: &str,
    field: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math(math_child(node, field)?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_one_arg_nodes_command(
    node: &MathNode,
    command: &str,
    field: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math_nodes(math_nodes(node, field)?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_two_arg_command(
    node: &MathNode,
    command: &str,
    first: &str,
    second: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math_value(math_field(node, first)?, out)?;
    out.push_str("}{");
    render_math_value(math_field(node, second)?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_annotated_command(
    node: &MathNode,
    command: &str,
    body: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math(math_child(node, body)?, out)?;
    out.push('}');
    if let Some(annotation) = math_optional_child(node, "annotation")? {
        out.push_str("^{");
        render_math(annotation, out)?;
        out.push('}');
    }
    Ok(())
}

fn render_math_annotation_command(node: &MathNode, command: &str, out: &mut String) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math(math_child(node, "annotation")?, out)?;
    out.push_str("}{");
    render_math(math_child(node, "body")?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_value(value: &MathValue, out: &mut String) -> Result<()> {
    match value {
        MathValue::None => Ok(()),
        MathValue::Bool(value) => {
            out.push_str(if *value { "true" } else { "false" });
            Ok(())
        }
        MathValue::Scalar(value) => {
            out.push_str(value);
            Ok(())
        }
        MathValue::Node(node) => render_math(node, out),
        MathValue::Nodes(nodes) => render_math_nodes(nodes, out),
        MathValue::Rows(_) => bail!("math row value cannot be rendered as a scalar expression"),
    }
}

fn math_field<'a>(node: &'a MathNode, name: &str) -> Result<&'a MathValue> {
    let Some(value) = node
        .fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
    else {
        bail!("math.{} is missing field `{name}`", node.func);
    };
    Ok(value)
}

fn math_child<'a>(node: &'a MathNode, name: &str) -> Result<&'a MathNode> {
    match math_field(node, name)? {
        MathValue::Node(child) => Ok(child),
        _ => bail!("math.{} field `{name}` must be a node", node.func),
    }
}

fn math_optional_child<'a>(node: &'a MathNode, name: &str) -> Result<Option<&'a MathNode>> {
    match node.fields.iter().find(|field| field.name == name) {
        Some(field) => match &field.value {
            MathValue::Node(child) => Ok(Some(child)),
            MathValue::None => Ok(None),
            _ => bail!("math.{} field `{name}` must be a node", node.func),
        },
        None => Ok(None),
    }
}

fn math_nodes<'a>(node: &'a MathNode, name: &str) -> Result<&'a [MathNode]> {
    match math_field(node, name)? {
        MathValue::Nodes(nodes) => Ok(nodes),
        _ => bail!("math.{} field `{name}` must be a node list", node.func),
    }
}

fn math_rows<'a>(node: &'a MathNode, name: &str) -> Result<&'a [Vec<MathNode>]> {
    match math_field(node, name)? {
        MathValue::Rows(rows) => Ok(rows),
        _ => bail!("math.{} field `{name}` must be a row list", node.func),
    }
}

fn math_scalar<'a>(node: &'a MathNode, name: &str) -> Result<&'a str> {
    match math_field(node, name)? {
        MathValue::Scalar(value) => Ok(value),
        _ => bail!("math.{} field `{name}` must be a scalar", node.func),
    }
}

fn render_element_frame(node: &Inline, data: &InlineElementData, out: &mut String) -> Result<()> {
    let kind = inline_kind(node)?;
    let Some(frame) = data.inlines("frame").and_then(single_frame) else {
        bail!("typlite markdown {kind} rendering requires html.frame");
    };
    render_frame_image(kind, frame, out)
}

fn single_frame(inlines: &[Inline]) -> Option<&FrameImage> {
    match inlines {
        [Inline::Frame(frame)] => Some(frame),
        _ => None,
    }
}

fn render_frame_image(alt: &str, frame: &FrameImage, out: &mut String) -> Result<()> {
    if frame.svg.contains("viewBox=\"0 0 0 0\"") {
        bail!("typlite markdown frame image for {alt} is empty");
    }

    out.push_str("<img alt=\"");
    push_html_escaped(alt, out);
    out.push_str("\" src=\"data:image/svg+xml;utf8,");
    push_url_escaped(&frame.svg, out);
    out.push_str("\">");

    Ok(())
}

fn render_image(data: &InlineElementData, out: &mut String) -> Result<()> {
    let Some(source) = data.scalar("source").filter(|source| !source.is_empty()) else {
        bail!("typlite markdown image rendering requires source");
    };

    out.push_str("![");
    if let Some(alt) = data.scalar("alt") {
        push_markdown_link_text_escaped(alt, out);
    }
    out.push_str("](");
    push_markdown_url(source, out);
    out.push(')');

    Ok(())
}

fn render_image_html(data: &InlineElementData, out: &mut String) -> Result<()> {
    let Some(source) = data.scalar("source").filter(|source| !source.is_empty()) else {
        bail!("typlite HTML image rendering requires source");
    };

    out.push_str("<img alt=\"");
    if let Some(alt) = data.scalar("alt") {
        push_html_escaped(alt, out);
    }
    out.push_str("\" src=\"");
    push_html_escaped(source, out);
    out.push_str("\">");

    Ok(())
}

fn render_cite(data: &InlineElementData, out: &mut String) -> Result<()> {
    let Some(key) = data.scalar("key").or_else(|| data.scalar("label")) else {
        bail!("typlite markdown cite rendering requires key or label");
    };
    ensure_default_cite_field(data, "form", "normal")?;
    ensure_default_cite_field(data, "style", "auto")?;

    out.push_str("[@");
    out.push_str(key.trim_start_matches('<').trim_end_matches('>'));
    if let Some(supplement) = data
        .inlines("supplement")
        .filter(|value| has_semantic_inlines(value))
    {
        out.push_str(", ");
        render_inlines(supplement, out)?;
    }
    out.push(']');

    Ok(())
}

fn render_ref(data: &InlineElementData, out: &mut String) -> Result<()> {
    let Some(target) = data.scalar("target").or_else(|| data.scalar("label")) else {
        bail!("typlite markdown ref rendering requires target or label");
    };
    if let Some(form) = data.scalar("form").filter(|form| *form != "normal") {
        bail!("typlite markdown ref rendering does not support form `{form}`");
    }

    let target = target.trim_start_matches('<').trim_end_matches('>');
    if let Some(supplement) = data
        .inlines("supplement")
        .filter(|value| has_semantic_inlines(value))
    {
        render_ref_link(target, supplement, out)?;
        return Ok(());
    }

    if let Some(element) = data
        .inlines("element")
        .filter(|value| has_semantic_inlines(value))
    {
        render_ref_link(target, element, out)?;
        return Ok(());
    }

    out.push('@');
    out.push_str(target);
    Ok(())
}

fn render_ref_link(target: &str, body: &[Inline], out: &mut String) -> Result<()> {
    out.push('[');
    render_inlines(body, out)?;
    out.push_str("](#");
    out.push_str(target);
    out.push(')');
    Ok(())
}

fn render_inline_quote(data: &InlineElementData, out: &mut String) -> Result<()> {
    let quoted = match data.scalar("quotes").unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        value => bail!("typlite markdown quote rendering does not support quotes `{value}`"),
    };

    if quoted {
        out.push('"');
    }
    render_inlines(&data.body, out)?;
    if quoted {
        out.push('"');
    }

    if let Some(attribution) = data
        .inlines("attribution")
        .filter(|value| has_semantic_inlines(value))
    {
        out.push_str(" (");
        render_inlines(attribution, out)?;
        out.push(')');
    }

    Ok(())
}

fn render_inline_quote_html(data: &InlineElementData, out: &mut String) -> Result<()> {
    let quoted = match data.scalar("quotes").unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        value => bail!("typlite HTML quote rendering does not support quotes `{value}`"),
    };

    if quoted {
        out.push_str("<q>");
    }
    render_inlines_html(&data.body, out)?;
    if quoted {
        out.push_str("</q>");
    }

    if let Some(attribution) = data
        .inlines("attribution")
        .filter(|value| has_semantic_inlines(value))
    {
        out.push_str(" <cite>");
        render_inlines_html(attribution, out)?;
        out.push_str("</cite>");
    }

    Ok(())
}

fn render_smartquote(data: &InlineElementData, out: &mut String) -> Result<()> {
    out.push(smartquote_char(data)?);
    Ok(())
}

fn render_smartquote_html(data: &InlineElementData, out: &mut String) -> Result<()> {
    match smartquote_char(data)? {
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        _ => unreachable!(),
    }
    Ok(())
}

fn smartquote_char(data: &InlineElementData) -> Result<char> {
    match data.scalar("double").unwrap_or("true") {
        "true" => Ok('"'),
        "false" => Ok('\''),
        value => {
            bail!("typlite markdown smartquote rendering requires boolean double, got `{value}`")
        }
    }
}

fn has_semantic_inlines(value: &[Inline]) -> bool {
    !value.is_empty() && !is_auto_inlines(value) && !is_none_inlines(value)
}

fn is_auto_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(text)] if text.as_str() == "auto")
}

fn is_none_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(text)] if text.as_str() == "none")
}

fn ensure_default_cite_field(data: &InlineElementData, field: &str, default: &str) -> Result<()> {
    if let Some(value) = data.scalar(field).filter(|value| *value != default) {
        bail!("typlite markdown cite rendering does not support {field} `{value}`");
    }

    Ok(())
}

fn render_structured_inline(
    node: &Inline,
    data: &InlineElementData,
    out: &mut String,
) -> Result<()> {
    out.push_str(inline_kind(node)?);
    out.push('(');
    for (index, field) in data.fields.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(field.name);
        out.push_str(": ");
        render_field_value(&field.value, out)?;
    }
    out.push(')');

    Ok(())
}

fn render_field_value(value: &ElementFieldValue, out: &mut String) -> Result<()> {
    match value {
        ElementFieldValue::Scalar(value) => out.push_str(value),
        ElementFieldValue::Inlines(value) => render_inlines(value, out)?,
        ElementFieldValue::Blocks(value) => out.push_str(&render_blocks(value, 0)?),
    }

    Ok(())
}

fn inline_kind(node: &Inline) -> Result<&'static str> {
    let kind = match node {
        Inline::Box(_) => "box",
        Inline::Circle(_) => "circle",
        Inline::Cite(_) => "cite",
        Inline::Curve(_) => "curve",
        Inline::CurveClose(_) => "curve.close",
        Inline::CurveCubic(_) => "curve.cubic",
        Inline::CurveLine(_) => "curve.line",
        Inline::CurveMove(_) => "curve.move",
        Inline::CurveQuad(_) => "curve.quad",
        Inline::Document(_) => "document",
        Inline::Ellipse(_) => "ellipse",
        Inline::FigureCaption(_) => "figure.caption",
        Inline::Footnote(_) => "footnote",
        Inline::FootnoteEntry(_) => "footnote.entry",
        Inline::GridCell(_) => "grid.cell",
        Inline::GridFooter(_) => "grid.footer",
        Inline::GridHeader(_) => "grid.header",
        Inline::GridHline(_) => "grid.hline",
        Inline::GridVline(_) => "grid.vline",
        Inline::H(_) => "h",
        Inline::Hide(_) => "hide",
        Inline::Highlight(_) => "highlight",
        Inline::Image(_) => "image",
        Inline::Line(_) => "line",
        Inline::MathAccent(_) => "math.accent",
        Inline::MathAttach(_) => "math.attach",
        Inline::MathBinom(_) => "math.binom",
        Inline::MathCancel(_) => "math.cancel",
        Inline::MathCases(_) => "math.cases",
        Inline::MathClass(_) => "math.class",
        Inline::MathFrac(_) => "math.frac",
        Inline::MathLimits(_) => "math.limits",
        Inline::MathLr(_) => "math.lr",
        Inline::MathMat(_) => "math.mat",
        Inline::MathMid(_) => "math.mid",
        Inline::MathOp(_) => "math.op",
        Inline::MathOverbrace(_) => "math.overbrace",
        Inline::MathOverbracket(_) => "math.overbracket",
        Inline::MathOverline(_) => "math.overline",
        Inline::MathOverparen(_) => "math.overparen",
        Inline::MathOvershell(_) => "math.overshell",
        Inline::MathPrimes(_) => "math.primes",
        Inline::MathRoot(_) => "math.root",
        Inline::MathScripts(_) => "math.scripts",
        Inline::MathStretch(_) => "math.stretch",
        Inline::MathUnderbrace(_) => "math.underbrace",
        Inline::MathUnderbracket(_) => "math.underbracket",
        Inline::MathUnderline(_) => "math.underline",
        Inline::MathUnderparen(_) => "math.underparen",
        Inline::MathUndershell(_) => "math.undershell",
        Inline::MathVec(_) => "math.vec",
        Inline::Metadata(_) => "metadata",
        Inline::Move(_) => "move",
        Inline::OutlineEntry(_) => "outline.entry",
        Inline::Overline(_) => "overline",
        Inline::Pad(_) => "pad",
        Inline::Page(_) => "page",
        Inline::ParLine(_) => "par.line",
        Inline::Path(_) => "path",
        Inline::PdfArtifact(_) => "pdf.artifact",
        Inline::PdfAttach(_) => "pdf.attach",
        Inline::PdfEmbed(_) => "pdf.embed",
        Inline::Place(_) => "place",
        Inline::PlaceFlush(_) => "place.flush",
        Inline::Polygon(_) => "polygon",
        Inline::Quote(_) => "quote",
        Inline::RawLine(_) => "raw.line",
        Inline::Rect(_) => "rect",
        Inline::Ref(_) => "ref",
        Inline::Repeat(_) => "repeat",
        Inline::Rotate(_) => "rotate",
        Inline::Scale(_) => "scale",
        Inline::Skew(_) => "skew",
        Inline::Smallcaps(_) => "smallcaps",
        Inline::Smartquote(_) => "smartquote",
        Inline::Square(_) => "square",
        Inline::TableCell(_) => "table.cell",
        Inline::TableFooter(_) => "table.footer",
        Inline::TableHeader(_) => "table.header",
        Inline::TableHline(_) => "table.hline",
        Inline::TableVline(_) => "table.vline",
        Inline::Underline(_) => "underline",
        Inline::Text(_)
        | Inline::Emph(_)
        | Inline::Strong(_)
        | Inline::Link { .. }
        | Inline::Strike(_)
        | Inline::Sub(_)
        | Inline::Super(_)
        | Inline::Math(_)
        | Inline::Linebreak
        | Inline::Frame(_)
        | Inline::Raw { .. } => bail!("not a generated inline element"),
    };

    Ok(kind)
}

fn push_html_escaped(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn push_markdown_link_text_escaped(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '\\' | '[' | ']' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
}

fn push_markdown_url(value: &str, out: &mut String) {
    if value.contains(char::is_whitespace) || value.contains(')') {
        out.push('<');
        for ch in value.chars() {
            if ch == '>' {
                out.push_str("%3E");
            } else {
                out.push(ch);
            }
        }
        out.push('>');
    } else {
        out.push_str(value);
    }
}

fn push_url_escaped(value: &str, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
}
