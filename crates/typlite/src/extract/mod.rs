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
use self::block::block_from_element;
use self::collections::*;
use self::fields::*;
use self::inline::{coalesce_raw_inlines, collect_link_body, inline_from_element_kind};

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

pub(super) fn collect_field_blocks(
    nodes: &[HtmlNode],
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut run_start = 0usize;

    for (index, node) in nodes.iter().enumerate() {
        if let HtmlNode::Element(element) = node
            && is_field(element)
        {
            if run_start < index {
                blocks.extend(collect_item_blocks(&nodes[run_start..index], introspector)?);
            }
            blocks.extend(collect_item_blocks(&element.children, introspector)?);
            run_start = index + 1;
        }
    }

    if run_start < nodes.len() {
        blocks.extend(collect_item_blocks(&nodes[run_start..], introspector)?);
    }

    Ok(blocks)
}

fn collect_item_blocks(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut inlines = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => {
                inlines.push(Inline::Text(TextInline { text: text.clone() }))
            }
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                if let Some(block) = block_from_element(element, introspector)? {
                    flush_paragraph(&mut inlines, &mut blocks);
                    blocks.push(block);
                } else {
                    inlines.extend(collect_inlines(std::slice::from_ref(node), introspector)?);
                }
            }
            HtmlNode::Frame(frame) => inlines.push(frame_to_inline(frame, introspector)),
            HtmlNode::Tag(_) => {}
        }
    }

    flush_paragraph(&mut inlines, &mut blocks);
    Ok(blocks)
}

fn flush_paragraph(inlines: &mut Vec<Inline>, blocks: &mut Vec<Block>) {
    if inlines.iter().any(inline_has_content) {
        blocks.push(Block::Paragraph(ParagraphBlock {
            body: coalesce_raw_inlines(std::mem::take(inlines)),
        }));
    } else {
        inlines.clear();
    }
}

fn inline_has_content(inline: &Inline) -> bool {
    match inline {
        Inline::Text(data) => !data.text.trim().is_empty(),
        Inline::Linebreak(_) => false,
        Inline::Frame(_) => true,
        Inline::Emph(data) => data.body.iter().any(inline_has_content),
        Inline::Strong(data) => data.body.iter().any(inline_has_content),
        Inline::Strike(data) => data.body.iter().any(inline_has_content),
        Inline::Sub(data) => data.body.iter().any(inline_has_content),
        Inline::Super(data) => data.body.iter().any(inline_has_content),
        Inline::Math(_) => true,
        Inline::Link(data) => !data.dest.is_empty() || data.body.iter().any(inline_has_content),
        Inline::Raw(data) => !data.text.is_empty(),
        Inline::Circle(data) => data.frame.is_some() || data.body.iter().any(inline_has_content),
        Inline::Curve(data) => {
            data.frame.is_some() || data.components.iter().any(inline_has_content)
        }
        Inline::Ellipse(data) => data.frame.is_some() || data.body.iter().any(inline_has_content),
        Inline::Line(data) => data.frame.is_some(),
        Inline::Path(data) => data.frame.is_some(),
        Inline::Polygon(data) => data.frame.is_some(),
        Inline::Rect(data) => data.frame.is_some() || data.body.iter().any(inline_has_content),
        Inline::Square(data) => data.frame.is_some() || data.body.iter().any(inline_has_content),
        Inline::Image(data) => data
            .source
            .as_ref()
            .is_some_and(|source| !source.is_empty()),
        Inline::Box(data) => data.body.iter().any(inline_has_content),
        Inline::FigureCaption(data) => data.body.iter().any(inline_has_content),
        Inline::Footnote(data) => data.body.iter().any(inline_has_content),
        Inline::GridCell(data) => data.body.iter().any(inline_has_content),
        Inline::GridFooter(data) => data.children.iter().any(inline_has_content),
        Inline::GridHeader(data) => data.children.iter().any(inline_has_content),
        Inline::Hide(data) => data.body.iter().any(inline_has_content),
        Inline::Highlight(data) => data.body.iter().any(inline_has_content),
        Inline::MathCases(data) => data.children.iter().any(inline_has_content),
        Inline::MathVec(data) => data.children.iter().any(inline_has_content),
        Inline::Move(data) => data.body.iter().any(inline_has_content),
        Inline::Overline(data) => data.body.iter().any(inline_has_content),
        Inline::Pad(data) => data.body.iter().any(inline_has_content),
        Inline::Page(data) => data.body.iter().any(inline_has_content),
        Inline::PdfArtifact(data) => data.body.iter().any(inline_has_content),
        Inline::Place(data) => data.body.iter().any(inline_has_content),
        Inline::Quote(data) => data.body.iter().any(inline_has_content),
        Inline::RawLine(data) => data.body.iter().any(inline_has_content),
        Inline::Repeat(data) => data.body.iter().any(inline_has_content),
        Inline::Rotate(data) => data.body.iter().any(inline_has_content),
        Inline::Scale(data) => data.body.iter().any(inline_has_content),
        Inline::Skew(data) => data.body.iter().any(inline_has_content),
        Inline::Smallcaps(data) => data.body.iter().any(inline_has_content),
        Inline::TableCell(data) => data.body.iter().any(inline_has_content),
        Inline::TableFooter(data) => data.children.iter().any(inline_has_content),
        Inline::TableHeader(data) => data.children.iter().any(inline_has_content),
        Inline::Underline(data) => data.body.iter().any(inline_has_content),
        Inline::Cite(_)
        | Inline::CurveClose(_)
        | Inline::CurveCubic(_)
        | Inline::CurveLine(_)
        | Inline::CurveMove(_)
        | Inline::CurveQuad(_)
        | Inline::Document(_)
        | Inline::FootnoteEntry(_)
        | Inline::GridHline(_)
        | Inline::GridVline(_)
        | Inline::H(_)
        | Inline::MathAccent(_)
        | Inline::MathAttach(_)
        | Inline::MathBinom(_)
        | Inline::MathCancel(_)
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
        | Inline::Metadata(_)
        | Inline::OutlineEntry(_)
        | Inline::ParLine(_)
        | Inline::PdfAttach(_)
        | Inline::PdfEmbed(_)
        | Inline::PlaceFlush(_)
        | Inline::Ref(_)
        | Inline::Smartquote(_)
        | Inline::TableHline(_)
        | Inline::TableVline(_) => false,
    }
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
