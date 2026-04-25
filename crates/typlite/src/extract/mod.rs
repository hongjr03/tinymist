//! Experimental extraction from Typst HTML custom elements into typlite IR.

use ecow::EcoString;
use typst::introspection::Introspector;
use typst_html::{HtmlDocument, HtmlElement, HtmlNode};

use crate::Result;
use crate::ir::*;

mod block;
mod collections;
mod fields;
mod inline;
mod traversal;
use self::block::block_from_element;
use self::collections::*;
use self::fields::*;
use self::inline::{coalesce_raw_inlines, collect_link_body, inline_from_element_kind};
use self::traversal::{collect_field_blocks, collect_item_blocks, inline_has_content};

/// Extracts typlite IR nodes from an HTML document root.
pub fn extract_document(html: &HtmlDocument) -> Result<Document> {
    let mut blocks = Vec::new();
    collect_blocks(&html.root, &html.introspector, &mut blocks)?;
    Ok(Document { blocks })
}

fn collect_blocks(
    element: &HtmlElement,
    introspector: &Introspector,
    blocks: &mut Vec<Block>,
) -> Result<()> {
    if let Some(block) = block_from_element(element, introspector)? {
        blocks.push(block);
        return Ok(());
    }

    if is_field(element) {
        return Ok(());
    }

    for child in &element.children {
        if let HtmlNode::Element(child) = child {
            collect_blocks(child, introspector, blocks)?;
        }
    }

    Ok(())
}

fn collect_inlines(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Inline>> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push(Inline::Text(TextInline { text: text.clone() })),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                if tag_name(element).as_deref() == Some("a") {
                    let body = collect_link_body(element, introspector)?;
                    out.push(Inline::Link(LinkInline {
                        dest: attr(element, "href").unwrap_or_default(),
                        body,
                    }));
                    continue;
                }

                match attr(element, "data-typlite").as_deref() {
                    Some("emph") => out.push(Inline::Emph(EmphInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("strong") => out.push(Inline::Strong(StrongInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("link") => out.push(Inline::Link(LinkInline {
                        dest: field_value(element, "dest").unwrap_or_default(),
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("strike") => out.push(Inline::Strike(StrikeInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("sub") => out.push(Inline::Sub(SubInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("super") => out.push(Inline::Super(SuperInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("math-equation") => out.push(Inline::Math(MathInline {
                        body: math_field(element, "body")?,
                    })),
                    Some("linebreak") => out.push(Inline::Linebreak(LinebreakInline {})),
                    Some("raw") => out.push(Inline::Raw(RawInline {
                        lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
                        text: raw_text(element),
                    })),
                    Some(kind) => {
                        if let Some(spec) = spec_by_kind(kind) {
                            if let Some(inline) =
                                inline_from_element_kind(element, spec.kind, introspector)?
                            {
                                out.push(inline);
                            }
                        }
                    }
                    None => {
                        out.extend(collect_inlines(&element.children, introspector)?);
                    }
                }
            }
            HtmlNode::Frame(frame) => out.push(frame_to_inline(frame, introspector)),
            HtmlNode::Tag(_) => {}
        }
    }

    Ok(coalesce_raw_inlines(out))
}

fn plain_text_blocks(blocks: &[Block]) -> EcoString {
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
