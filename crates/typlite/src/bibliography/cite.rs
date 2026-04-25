use crate::ir::{Block, Inline};

pub(super) fn collect_cite_keys(blocks: &[Block], out: &mut Vec<String>) {
    for block in blocks {
        match block {
            Block::Heading(data) => collect_cite_keys_in_inlines(&data.body, out),
            Block::Paragraph(data) => collect_cite_keys_in_inlines(&data.body, out),
            Block::Quote(data) => collect_cite_keys(&data.body, out),
            Block::Figure(data) => {
                collect_cite_keys(&data.body, out);
                collect_cite_keys_in_inlines(&data.caption, out);
            }
            Block::Align(data) => collect_cite_keys(&data.body, out),
            Block::Block(data) => collect_cite_keys(&data.body, out),
            Block::Columns(data) => collect_cite_keys(&data.body, out),
            Block::Move(data) => collect_cite_keys(&data.body, out),
            Block::Pad(data) => collect_cite_keys(&data.body, out),
            Block::Rotate(data) => collect_cite_keys(&data.body, out),
            Block::Scale(data) => collect_cite_keys(&data.body, out),
            Block::Skew(data) => collect_cite_keys(&data.body, out),
            Block::Stack(data) => collect_cite_keys(&data.children, out),
            Block::Title(data) => collect_cite_keys(&data.body, out),
            Block::List(data) => {
                for item in &data.items {
                    collect_cite_keys(&item.body, out);
                }
            }
            Block::Terms(data) => {
                for item in &data.items {
                    collect_cite_keys_in_inlines(&item.term, out);
                    collect_cite_keys(&item.description, out);
                }
            }
            _ => {}
        }
    }
}

fn collect_cite_keys_in_inlines(inlines: &[Inline], out: &mut Vec<String>) {
    for inline in inlines {
        if let Inline::Cite(data) = inline {
            if let Some(key) = data.key.as_deref() {
                let key = key.trim_start_matches('<').trim_end_matches('>').to_owned();
                if !out.contains(&key) {
                    out.push(key);
                }
            }
        }

        match inline {
            Inline::Emph(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Strong(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Strike(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Sub(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Super(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Math(_)
            | Inline::Text(_)
            | Inline::Linebreak(_)
            | Inline::Frame(_)
            | Inline::Raw(_) => {}
            Inline::Link(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Box(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Circle(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Curve(data) => collect_cite_keys_in_inlines(&data.components, out),
            Inline::Ellipse(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::FigureCaption(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Footnote(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::GridCell(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::GridFooter(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::GridHeader(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::Hide(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Highlight(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::MathCases(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::MathVec(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::Move(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Overline(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Pad(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Page(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::PdfArtifact(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Place(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Quote(data) => {
                collect_cite_keys_in_inlines(&data.attribution, out);
                collect_cite_keys_in_inlines(&data.body, out);
            }
            Inline::RawLine(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Ref(data) => {
                collect_cite_keys_in_inlines(&data.supplement, out);
                collect_cite_keys_in_inlines(&data.citation, out);
                collect_cite_keys(&data.element, out);
            }
            Inline::Repeat(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Rotate(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Scale(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Skew(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::Smallcaps(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::TableCell(data) => collect_cite_keys_in_inlines(&data.body, out),
            Inline::TableFooter(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::TableHeader(data) => collect_cite_keys_in_inlines(&data.children, out),
            Inline::Underline(data) => collect_cite_keys_in_inlines(&data.body, out),
            _ => {}
        }
    }
}
