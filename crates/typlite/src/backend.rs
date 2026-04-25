//! Experimental backends for typlite IR.

use crate::ir::{Block, Document, Inline, TableRow};

/// Renders a document IR as Markdown.
pub fn render_markdown(doc: &Document) -> String {
    render_blocks(&doc.blocks, 0)
}

fn render_blocks(blocks: &[Block], indent: usize) -> String {
    let mut out = String::new();

    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, &mut rendered);
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(&rendered);
    }

    out
}

fn render_block(block: &Block, indent: usize, out: &mut String) {
    match block {
        Block::Heading { level, body } => {
            out.push_str(&" ".repeat(indent));
            out.push_str(&"#".repeat(*level as usize));
            out.push(' ');
            render_inlines(body, out);
        }
        Block::Paragraph(body) => {
            out.push_str(&" ".repeat(indent));
            render_inlines(body, out);
        }
        Block::Quote(blocks) => render_quote(blocks, indent, out),
        Block::Figure { body, caption } => {
            render_blocks_into(body, indent, out);
            if !caption.is_empty() {
                if !body.is_empty() {
                    out.push('\n');
                    out.push('\n');
                }
                out.push_str(&" ".repeat(indent));
                out.push_str("Figure: ");
                render_inlines(caption, out);
            }
        }
        Block::Align(blocks) => render_blocks_into(blocks, indent, out),
        Block::Math(body) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("$$");
            render_inlines(body, out);
            out.push_str("$$");
        }
        Block::Table { rows } => render_table(rows, indent, out),
        Block::Element(element) => {
            render_blocks_into(&element.body, indent, out);
        }
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
        } => render_list(*ordered, *start, *reversed, items, indent, out),
    }
}

fn render_blocks_into(blocks: &[Block], indent: usize, out: &mut String) {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, &mut rendered);
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(&rendered);
    }
}

fn render_quote(blocks: &[Block], indent: usize, out: &mut String) {
    let body = render_blocks(blocks, 0);
    let prefix = format!("{}> ", " ".repeat(indent));

    for (index, line) in body.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&prefix);
        out.push_str(line);
    }
}

fn render_table(rows: &[TableRow], indent: usize, out: &mut String) {
    if rows.is_empty() {
        return;
    }

    let columns = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    if columns == 0 {
        return;
    }

    render_table_row(&rows[0], columns, indent, out);
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push('|');
    for _ in 0..columns {
        out.push_str(" --- |");
    }

    for row in &rows[1..] {
        out.push('\n');
        render_table_row(row, columns, indent, out);
    }
}

fn render_table_row(row: &TableRow, columns: usize, indent: usize, out: &mut String) {
    out.push_str(&" ".repeat(indent));
    out.push('|');
    for index in 0..columns {
        out.push(' ');
        if let Some(cell) = row.cells.get(index) {
            render_inlines(&cell.body, out);
        }
        out.push_str(" |");
    }
}

fn render_list(
    ordered: bool,
    start: Option<i64>,
    reversed: bool,
    items: &[crate::ir::ListItem],
    indent: usize,
    out: &mut String,
) {
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
        let body = render_blocks(&item.body, continuation);

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
}

fn render_inlines(nodes: &[Inline], out: &mut String) {
    for node in nodes {
        match node {
            Inline::Text(text) => out.push_str(text),
            Inline::Emph(children) => {
                out.push('*');
                render_inlines(children, out);
                out.push('*');
            }
            Inline::Strong(children) => {
                out.push_str("**");
                render_inlines(children, out);
                out.push_str("**");
            }
            Inline::Link { dest, body } => {
                out.push('[');
                render_inlines(body, out);
                out.push_str("](");
                out.push_str(dest);
                out.push(')');
            }
            Inline::Strike(children) => {
                out.push_str("~~");
                render_inlines(children, out);
                out.push_str("~~");
            }
            Inline::Sub(children) => {
                out.push_str("<sub>");
                render_inlines(children, out);
                out.push_str("</sub>");
            }
            Inline::Super(children) => {
                out.push_str("<sup>");
                render_inlines(children, out);
                out.push_str("</sup>");
            }
            Inline::Math(children) => {
                out.push('$');
                render_inlines(children, out);
                out.push('$');
            }
            Inline::Linebreak => out.push('\n'),
            Inline::Element(element) => render_inlines(&element.body, out),
            Inline::Raw { text, .. } => {
                out.push('`');
                out.push_str(text);
                out.push('`');
            }
        }
    }
}
