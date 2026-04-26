use hayagriva::Library;

use crate::ir::{Block, Inline};

pub(super) fn collect_cite_keys(blocks: &[Block], library: &Library, out: &mut Vec<String>) {
    for block in blocks {
        match block {
            Block::Heading(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Block::Paragraph(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Block::Quote(data) => collect_cite_keys(&data.body, library, out),
            Block::Figure(data) => {
                collect_cite_keys(&data.body, library, out);
                collect_cite_keys_in_inlines(&data.caption, library, out);
            }
            Block::Align(data) => collect_cite_keys(&data.body, library, out),
            Block::Block(data) => collect_cite_keys(&data.body, library, out),
            Block::Columns(data) => collect_cite_keys(&data.body, library, out),
            Block::Move(data) => collect_cite_keys(&data.body, library, out),
            Block::Pad(data) => collect_cite_keys(&data.body, library, out),
            Block::Rotate(data) => collect_cite_keys(&data.body, library, out),
            Block::Scale(data) => collect_cite_keys(&data.body, library, out),
            Block::Skew(data) => collect_cite_keys(&data.body, library, out),
            Block::Stack(data) => collect_cite_keys(&data.children, library, out),
            Block::Title(data) => collect_cite_keys(&data.body, library, out),
            Block::List(data) => {
                for item in &data.items {
                    collect_cite_keys(&item.body, library, out);
                }
            }
            Block::Terms(data) => {
                for item in &data.items {
                    collect_cite_keys_in_inlines(&item.term, library, out);
                    collect_cite_keys(&item.description, library, out);
                }
            }
            _ => {}
        }
    }
}

fn collect_cite_keys_in_inlines(inlines: &[Inline], library: &Library, out: &mut Vec<String>) {
    for inline in inlines {
        if let Inline::Cite(data) = inline {
            if let Some(key) = data.key.as_deref() {
                push_cite_key(key, out);
            }
        }

        match inline {
            Inline::Emph(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Strong(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Strike(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Sub(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Super(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Math(_)
            | Inline::Text(_)
            | Inline::Linebreak(_)
            | Inline::Frame(_)
            | Inline::Raw(_) => {}
            Inline::Link(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Box(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Circle(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Curve(data) => collect_cite_keys_in_inlines(&data.components, library, out),
            Inline::Ellipse(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::FigureCaption(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Footnote(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::GridCell(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::GridFooter(data) => collect_cite_keys_in_inlines(&data.children, library, out),
            Inline::GridHeader(data) => collect_cite_keys_in_inlines(&data.children, library, out),
            Inline::Hide(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Highlight(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::MathCases(data) => collect_cite_keys_in_inlines(&data.children, library, out),
            Inline::MathVec(data) => collect_cite_keys_in_inlines(&data.children, library, out),
            Inline::Move(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Overline(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Pad(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Page(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::PdfArtifact(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Place(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Quote(data) => {
                collect_cite_keys_in_inlines(&data.attribution, library, out);
                collect_cite_keys_in_inlines(&data.body, library, out);
            }
            Inline::RawLine(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Ref(data) => {
                if let Some(target) = data.target.as_deref() {
                    let target = normalize_key(target);
                    if library.get(&target).is_some() {
                        push_cite_key(&target, out);
                    }
                }
                collect_cite_keys_in_inlines(&data.supplement, library, out);
            }
            Inline::Repeat(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Rotate(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Scale(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Skew(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::Smallcaps(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::TableCell(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            Inline::TableFooter(data) => collect_cite_keys_in_inlines(&data.children, library, out),
            Inline::TableHeader(data) => collect_cite_keys_in_inlines(&data.children, library, out),
            Inline::Underline(data) => collect_cite_keys_in_inlines(&data.body, library, out),
            _ => {}
        }
    }
}

fn push_cite_key(key: &str, out: &mut Vec<String>) {
    let key = normalize_key(key);
    if !out.contains(&key) {
        out.push(key);
    }
}

fn normalize_key(key: &str) -> String {
    key.trim_start_matches('<').trim_end_matches('>').to_owned()
}
