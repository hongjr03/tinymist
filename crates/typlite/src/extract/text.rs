use ecow::EcoString;

use crate::ir::*;

pub(super) fn plain_text_blocks(blocks: &[Block]) -> EcoString {
    let mut out = EcoString::new();
    for block in blocks {
        match block {
            Block::Heading(data) => push_plain_text_inlines(&data.body, &mut out),
            Block::Paragraph(data) => push_plain_text_inlines(&data.body, &mut out),
            Block::Quote(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Figure(data) => {
                out.push_str(&plain_text_blocks(&data.body));
                push_plain_text_inlines(&data.caption, &mut out);
            }
            Block::Align(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Table(data) => {
                for row in &data.rows {
                    for cell in &row.cells {
                        push_plain_text_inlines(&cell.body, &mut out);
                    }
                }
            }
            Block::List(data) => {
                for item in &data.items {
                    out.push_str(&plain_text_blocks(&item.body));
                }
            }
            Block::Terms(data) => {
                for item in &data.items {
                    push_plain_text_inlines(&item.term, &mut out);
                    out.push_str(&plain_text_blocks(&item.description));
                }
            }
            Block::Block(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Columns(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Move(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Pad(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Rotate(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Scale(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Skew(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Stack(data) => out.push_str(&plain_text_blocks(&data.children)),
            Block::Title(data) => out.push_str(&plain_text_blocks(&data.body)),
            Block::Bibliography(_)
            | Block::Colbreak(_)
            | Block::Math(_)
            | Block::Outline(_)
            | Block::Pagebreak(_)
            | Block::Parbreak(_)
            | Block::Raw(_)
            | Block::V(_) => {}
        }
    }
    out
}

fn push_plain_text_inlines(inlines: &[Inline], out: &mut EcoString) {
    for inline in inlines {
        match inline {
            Inline::Text(data) => out.push_str(&data.text),
            Inline::Raw(data) => out.push_str(&data.text),
            Inline::Linebreak(_) | Inline::H(_) => out.push(' '),
            Inline::Emph(data) => push_plain_text_inlines(&data.body, out),
            Inline::Strong(data) => push_plain_text_inlines(&data.body, out),
            Inline::Strike(data) => push_plain_text_inlines(&data.body, out),
            Inline::Sub(data) => push_plain_text_inlines(&data.body, out),
            Inline::Super(data) => push_plain_text_inlines(&data.body, out),
            Inline::Link(data) => push_plain_text_inlines(&data.body, out),
            Inline::Box(data) => push_plain_text_inlines(&data.body, out),
            Inline::Circle(data) => push_plain_text_inlines(&data.body, out),
            Inline::Curve(data) => push_plain_text_inlines(&data.components, out),
            Inline::Ellipse(data) => push_plain_text_inlines(&data.body, out),
            Inline::FigureCaption(data) => push_plain_text_inlines(&data.body, out),
            Inline::Footnote(data) => push_plain_text_inlines(&data.body, out),
            Inline::GridCell(data) => push_plain_text_inlines(&data.body, out),
            Inline::GridFooter(data) => push_plain_text_inlines(&data.children, out),
            Inline::GridHeader(data) => push_plain_text_inlines(&data.children, out),
            Inline::Hide(data) => push_plain_text_inlines(&data.body, out),
            Inline::Highlight(data) => push_plain_text_inlines(&data.body, out),
            Inline::MathCases(data) => push_plain_text_inlines(&data.children, out),
            Inline::MathVec(data) => push_plain_text_inlines(&data.children, out),
            Inline::Move(data) => push_plain_text_inlines(&data.body, out),
            Inline::Overline(data) => push_plain_text_inlines(&data.body, out),
            Inline::Pad(data) => push_plain_text_inlines(&data.body, out),
            Inline::Page(data) => push_plain_text_inlines(&data.body, out),
            Inline::PdfArtifact(data) => push_plain_text_inlines(&data.body, out),
            Inline::Place(data) => push_plain_text_inlines(&data.body, out),
            Inline::Quote(data) => push_plain_text_inlines(&data.body, out),
            Inline::RawLine(data) => push_plain_text_inlines(&data.body, out),
            Inline::Repeat(data) => push_plain_text_inlines(&data.body, out),
            Inline::Rotate(data) => push_plain_text_inlines(&data.body, out),
            Inline::Scale(data) => push_plain_text_inlines(&data.body, out),
            Inline::Skew(data) => push_plain_text_inlines(&data.body, out),
            Inline::Smallcaps(data) => push_plain_text_inlines(&data.body, out),
            Inline::TableCell(data) => push_plain_text_inlines(&data.body, out),
            Inline::TableFooter(data) => push_plain_text_inlines(&data.children, out),
            Inline::TableHeader(data) => push_plain_text_inlines(&data.children, out),
            Inline::Underline(data) => push_plain_text_inlines(&data.body, out),
            _ => {}
        }
    }
}
