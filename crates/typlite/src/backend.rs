//! Experimental backends for typlite IR.

use crate::ir::{Block, Document, Inline};

/// Renders a document IR as Markdown.
pub fn render_markdown(doc: &Document) -> String {
    let mut out = String::new();

    for (idx, block) in doc.blocks.iter().enumerate() {
        if idx > 0 {
            out.push_str("\n\n");
        }

        match block {
            Block::Heading { level, body } => {
                out.push_str(&"#".repeat(*level as usize));
                out.push(' ');
                render_inlines(body, &mut out);
            }
            Block::Paragraph(body) => render_inlines(body, &mut out),
            Block::Raw { text, .. } => out.push_str(text),
        }
    }

    out
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
