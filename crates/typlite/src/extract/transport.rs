//! Transport view over a Typst HTML element.

use std::collections::VecDeque;

use ecow::EcoString;
use serde_json::{Map, Value};
use tinymist_std::error::prelude::*;
use typst::introspection::Introspector;
use typst_html::{HtmlElement, HtmlNode};

use crate::Result;
use crate::ir::{Block, ElementField, ElementFieldValue, Inline, InlineElementData, MathNode};

use super::dom::{attr, collect_text_without_frames, field_children, frame_to_inline, tag_name};
use super::{Extractor, encoded, flow::inline_has_content, math_field};

pub(super) struct TransportElement<'a> {
    pub(super) element: &'a HtmlElement,
    pub(super) encoded: Option<Map<String, Value>>,
}

impl<'a> TransportElement<'a> {
    pub(super) fn new(element: &'a HtmlElement) -> Result<Self> {
        Ok(Self {
            element,
            encoded: encoded_object(element)?,
        })
    }

    pub(super) fn field(&self, name: &str) -> Option<&'a [HtmlNode]> {
        field_children(self.element, name)
    }

    pub(super) fn encoded_field(&self, name: &str) -> Option<&Value> {
        self.encoded.as_ref().and_then(|object| object.get(name))
    }

    pub(super) fn scalar(&self, name: &str) -> Option<EcoString> {
        if let Some(value) = self.encoded_field(name) {
            return Some(encoded::scalar(value));
        }

        self.field(name).map(collect_text_without_frames)
    }

    pub(super) fn bool(&self, name: &str) -> bool {
        self.scalar(name)
            .is_some_and(|value| value.as_str() == "true")
    }

    pub(super) fn content_blocks(&self, name: &str, extractor: &Extractor) -> Result<Vec<Block>> {
        if let Some(value) = self.encoded_field(name) {
            let mut blocks = encoded::content_blocks(value, extractor.introspector)?;
            if let Some(children) = self.field(name) {
                let mut frames = collect_frame_inlines(children, extractor.introspector);
                if !frames.is_empty() && blocks.is_empty() {
                    let mut inlines = encoded::content_inlines(value, extractor.introspector)?;
                    patch_frames_in_inlines(&mut inlines, &mut frames);
                    if inlines.iter().any(inline_has_content) {
                        return Ok(vec![Block::Paragraph(inlines)]);
                    }
                    return Ok(blocks);
                }
                patch_frames_in_blocks(&mut blocks, &mut frames);
            }
            return Ok(blocks);
        }

        if let Some(children) = self.field(name) {
            return extractor.item_blocks(children);
        }

        Ok(Vec::new())
    }

    pub(super) fn content_inlines(&self, name: &str, extractor: &Extractor) -> Result<Vec<Inline>> {
        if let Some(value) = self.encoded_field(name) {
            let mut inlines = encoded::content_inlines(value, extractor.introspector)?;
            if let Some(children) = self.field(name) {
                let mut frames = collect_frame_inlines(children, extractor.introspector);
                patch_frames_in_inlines(&mut inlines, &mut frames);
            }
            return Ok(inlines);
        }

        if let Some(children) = self.field(name) {
            return extractor.inlines(children);
        }

        Ok(Vec::new())
    }

    pub(super) fn math(&self, name: &str) -> Result<MathNode> {
        if self.field(name).is_some() {
            return math_field(self.element, name);
        }

        if let Some(value) = self.encoded_field(name) {
            return encoded::math_node(value);
        }

        math_field(self.element, name)
    }
}

fn collect_frame_inlines(nodes: &[HtmlNode], introspector: &Introspector) -> VecDeque<Inline> {
    let mut frames = VecDeque::new();
    collect_frame_inlines_into(nodes, introspector, &mut frames);
    frames
}

fn collect_frame_inlines_into(
    nodes: &[HtmlNode],
    introspector: &Introspector,
    out: &mut VecDeque<Inline>,
) {
    for node in nodes {
        match node {
            HtmlNode::Frame(frame) => out.push_back(frame_to_inline(frame, introspector)),
            HtmlNode::Element(element) => {
                collect_frame_inlines_into(&element.children, introspector, out)
            }
            HtmlNode::Text(..) | HtmlNode::Tag(_) => {}
        }
    }
}

fn patch_frames_in_blocks(blocks: &mut [Block], frames: &mut VecDeque<Inline>) {
    for block in blocks {
        match block {
            Block::Heading { body, .. } | Block::Paragraph(body) => {
                patch_frames_in_inlines(body, frames)
            }
            Block::Quote(body) => patch_frames_in_blocks(body, frames),
            Block::Figure { body, caption, .. } => {
                patch_frames_in_blocks(body, frames);
                patch_frames_in_inlines(caption, frames);
            }
            Block::Align { body, .. } => patch_frames_in_blocks(body, frames),
            Block::Table { rows, .. } => {
                for row in rows {
                    for cell in &mut row.cells {
                        patch_frames_in_inlines(&mut cell.body, frames);
                    }
                }
            }
            Block::Terms { items } => {
                for item in items {
                    patch_frames_in_inlines(&mut item.term, frames);
                    patch_frames_in_blocks(&mut item.description, frames);
                }
            }
            Block::List { items, .. } => {
                for item in items {
                    patch_frames_in_blocks(&mut item.body, frames);
                }
            }
            Block::Bibliography(data)
            | Block::Block(data)
            | Block::Colbreak(data)
            | Block::Columns(data)
            | Block::Move(data)
            | Block::Outline(data)
            | Block::Pad(data)
            | Block::Pagebreak(data)
            | Block::Parbreak(data)
            | Block::Rotate(data)
            | Block::Scale(data)
            | Block::Skew(data)
            | Block::Stack(data)
            | Block::Title(data)
            | Block::V(data) => patch_frames_in_blocks(&mut data.body, frames),
            Block::Math(_) | Block::Raw { .. } => {}
        }
    }
}

fn patch_frames_in_inlines(inlines: &mut [Inline], frames: &mut VecDeque<Inline>) {
    for inline in inlines {
        match inline {
            Inline::Circle(data)
            | Inline::Curve(data)
            | Inline::Ellipse(data)
            | Inline::Line(data)
            | Inline::Path(data)
            | Inline::Polygon(data)
            | Inline::Rect(data)
            | Inline::Square(data) => {
                patch_frame_field(data, frames);
                patch_frames_in_inlines(&mut data.body, frames);
            }
            Inline::Emph(body)
            | Inline::Strong(body)
            | Inline::Strike(body)
            | Inline::Sub(body)
            | Inline::Super(body) => patch_frames_in_inlines(body, frames),
            Inline::Link { body, .. } => patch_frames_in_inlines(body, frames),
            Inline::Box(data)
            | Inline::Cite(data)
            | Inline::CurveClose(data)
            | Inline::CurveCubic(data)
            | Inline::CurveLine(data)
            | Inline::CurveMove(data)
            | Inline::CurveQuad(data)
            | Inline::Document(data)
            | Inline::FigureCaption(data)
            | Inline::Footnote(data)
            | Inline::FootnoteEntry(data)
            | Inline::GridCell(data)
            | Inline::GridFooter(data)
            | Inline::GridHeader(data)
            | Inline::GridHline(data)
            | Inline::GridVline(data)
            | Inline::H(data)
            | Inline::Hide(data)
            | Inline::Highlight(data)
            | Inline::MathAccent(data)
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
            | Inline::Metadata(data)
            | Inline::Move(data)
            | Inline::OutlineEntry(data)
            | Inline::Overline(data)
            | Inline::Pad(data)
            | Inline::Page(data)
            | Inline::ParLine(data)
            | Inline::PdfArtifact(data)
            | Inline::PdfAttach(data)
            | Inline::PdfEmbed(data)
            | Inline::Place(data)
            | Inline::PlaceFlush(data)
            | Inline::Quote(data)
            | Inline::RawLine(data)
            | Inline::Ref(data)
            | Inline::Repeat(data)
            | Inline::Rotate(data)
            | Inline::Scale(data)
            | Inline::Skew(data)
            | Inline::Smallcaps(data)
            | Inline::Smartquote(data)
            | Inline::TableCell(data)
            | Inline::TableFooter(data)
            | Inline::TableHeader(data)
            | Inline::TableHline(data)
            | Inline::TableVline(data)
            | Inline::Underline(data) => patch_frames_in_inlines(&mut data.body, frames),
            Inline::Text(_)
            | Inline::Math(_)
            | Inline::Linebreak
            | Inline::Frame(_)
            | Inline::Image(_)
            | Inline::Raw { .. } => {}
        }
    }
}

fn patch_frame_field(data: &mut InlineElementData, frames: &mut VecDeque<Inline>) {
    if data.field("frame").is_some() {
        return;
    }

    let Some(frame) = frames.pop_front() else {
        return;
    };

    data.fields.push(ElementField {
        name: "frame",
        value: ElementFieldValue::Inlines(vec![frame]),
    });
}

fn encoded_object(element: &HtmlElement) -> Result<Option<Map<String, Value>>> {
    if attr(element, "data-typlite-ir").as_deref() != Some("true") {
        return Ok(None);
    }

    let raw = collect_text_without_frames(&element.children);
    if raw.trim().is_empty() {
        return Ok(None);
    }

    let value =
        serde_json::from_str::<Value>(&raw).with_context_ut("cannot parse typlite IR", || {
            Some(Box::new([
                ("tag", tag_name(element).unwrap_or_default()),
                ("raw", raw.to_string()),
            ]))
        })?;
    let Value::Object(object) = value else {
        bail!("typlite IR must be encoded as an object, got {value}");
    };
    Ok(Some(object))
}
