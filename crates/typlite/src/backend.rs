//! Experimental backends for typlite IR.

use crate::ir::{Block, Document, Inline};

/// Renders a document IR as Markdown.
pub fn render_markdown(doc: &Document) -> String {
    render_blocks(&doc.blocks, 0)
}

fn render_blocks(blocks: &[Block], indent: usize) -> String {
    let mut out = String::new();

    for (idx, block) in blocks.iter().enumerate() {
        if idx > 0 {
            out.push_str("\n\n");
        }

        render_block(block, indent, &mut out);
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
            render_inlines(body, out);
        }
        Block::Raw { text, .. } => {
            out.push_str(&" ".repeat(indent));
            out.push_str(text);
        }
        Block::List { ordered, items } => render_list(*ordered, items, indent, out),
    }
}

fn render_blocks_into(blocks: &[Block], indent: usize, out: &mut String) {
    for (idx, block) in blocks.iter().enumerate() {
        if idx > 0 {
            out.push_str("\n\n");
        }
        render_block(block, indent, out);
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

fn render_list(ordered: bool, items: &[crate::ir::ListItem], indent: usize, out: &mut String) {
    let mut next_number = 1usize;

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        let marker = if ordered {
            let number = item
                .number
                .as_ref()
                .and_then(|number| number.parse::<usize>().ok())
                .unwrap_or(next_number);
            next_number = number + 1;
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
            Inline::Raw { text, .. } => {
                out.push('`');
                out.push_str(text);
                out.push('`');
            }
        }
    }
}
