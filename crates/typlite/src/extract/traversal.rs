use typst::introspection::Introspector;
use typst_html::HtmlNode;

use crate::Result;
use crate::ir::*;

use super::{block_from_element, coalesce_raw_inlines, collect_inlines, frame_to_inline, is_field};

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

pub(super) fn collect_item_blocks(
    nodes: &[HtmlNode],
    introspector: &Introspector,
) -> Result<Vec<Block>> {
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

pub(super) fn inline_has_content(inline: &Inline) -> bool {
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
