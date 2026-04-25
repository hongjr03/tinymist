//! Experimental backends for typlite IR.

use crate::Result;
use crate::ir::{
    Block, Document, ElementFieldValue, FrameImage, Inline, InlineElementData, TableRow,
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
        Block::Figure { body, caption } => {
            render_blocks_into(body, indent, out)?;
            if !caption.is_empty() {
                if !body.is_empty() {
                    out.push('\n');
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent));
                out.push_str("Figure: ");
                render_inlines(caption, out)?;
            }
        }
        Block::Align(blocks) => render_blocks_into(blocks, indent, out)?,
        Block::Math(body) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("$$");
            render_inlines(body, out)?;
            out.push_str("$$");
        }
        Block::Table { rows } => render_table(rows, indent, out)?,
        Block::Raw { text, .. } => {
            out.push_str(&" ".repeat(indent));
            out.push_str(text);
        }
        Block::List {
            ordered,
            start,
            reversed,
            items,
            ..
        } => render_list(*ordered, *start, *reversed, items, indent, out)?,
        Block::Block(data)
        | Block::Columns(data)
        | Block::Stack(data)
        | Block::Terms(data)
        | Block::Title(data) => render_blocks_into(&data.body, indent, out)?,
        Block::Colbreak(_) | Block::Parbreak(_) | Block::V(_) => {}
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

fn render_table(rows: &[TableRow], indent: usize, out: &mut String) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    let columns = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    if columns == 0 {
        return Ok(());
    }

    render_table_row(&rows[0], columns, indent, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push('|');
    for _ in 0..columns {
        out.push_str(" --- |");
    }

    for row in &rows[1..] {
        out.push('\n');
        render_table_row(row, columns, indent, out)?;
    }

    Ok(())
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
                render_inlines(children, out)?;
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
            | Inline::Quote(data)
            | Inline::RawLine(data) => render_inlines(&data.body, out)?,
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
            Inline::FigureCaption(data) | Inline::FootnoteEntry(data) => {
                render_inlines(&data.body, out)?
            }
            Inline::Footnote(data) => {
                out.push_str("<sup>");
                render_inlines(&data.body, out)?;
                out.push_str("</sup>");
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
            Inline::Smartquote(_) => {}
            Inline::Underline(data) => {
                out.push_str("<u>");
                render_inlines(&data.body, out)?;
                out.push_str("</u>");
            }
        }
    }

    Ok(())
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
        out.push_str(alt);
    }
    out.push_str("](");
    out.push_str(source);
    out.push(')');

    Ok(())
}

fn render_cite(data: &InlineElementData, out: &mut String) -> Result<()> {
    let Some(key) = data.scalar("key").or_else(|| data.scalar("label")) else {
        bail!("typlite markdown cite rendering requires key or label");
    };
    out.push_str("[@");
    out.push_str(key.trim_start_matches('<').trim_end_matches('>'));
    out.push(']');

    Ok(())
}

fn render_ref(data: &InlineElementData, out: &mut String) -> Result<()> {
    let Some(target) = data.scalar("target").or_else(|| data.scalar("label")) else {
        bail!("typlite markdown ref rendering requires target or label");
    };
    if let Some(supplement) = data
        .inlines("supplement")
        .filter(|value| !value.is_empty() && !is_auto_inlines(value))
    {
        out.push('[');
        render_inlines(supplement, out)?;
        out.push_str("](#");
        out.push_str(target.trim_start_matches('<').trim_end_matches('>'));
        out.push(')');
        return Ok(());
    }

    out.push('@');
    out.push_str(target.trim_start_matches('<').trim_end_matches('>'));

    Ok(())
}

fn is_auto_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(text)] if text.as_str() == "auto")
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
