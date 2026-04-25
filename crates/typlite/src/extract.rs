//! Experimental extraction from Typst HTML custom elements into typlite IR.

mod encoded;
mod fields;
mod flow;
mod transport;

use ecow::EcoString;
use serde_json::Value;
use tinymist_std::error::prelude::*;
use typst::introspection::Introspector;
use typst_html::{HtmlDocument, HtmlElement, HtmlFrame, HtmlNode};

use crate::Result;
use crate::element_spec::{ELEMENTS, ElementMode, ElementSpec};
use crate::ir::{
    Block, BlockElementData, Document, ElementField, ElementFieldValue, FrameImage, Inline,
    InlineElementData, ListItem, MathNode, TableAlign, TableCell, TableRow, TermItem,
    block_from_element_kind, inline_from_element_kind,
};

use self::fields::{content_fields, is_content_field_name};
use self::flow::{flush_paragraph, table_alignment};
use self::transport::TransportElement;

/// Extracts typlite IR nodes from an HTML document root.
pub fn extract_document(html: &HtmlDocument) -> Result<Document> {
    Extractor::new(&html.introspector).document(&html.root)
}

struct Extractor<'a> {
    introspector: &'a Introspector,
}

impl<'a> Extractor<'a> {
    fn new(introspector: &'a Introspector) -> Self {
        Self { introspector }
    }

    fn document(&self, root: &HtmlElement) -> Result<Document> {
        let mut blocks = Vec::new();
        self.collect_blocks(root, &mut blocks)?;
        Ok(Document { blocks })
    }

    fn collect_blocks(&self, element: &HtmlElement, blocks: &mut Vec<Block>) -> Result<()> {
        if let Some(block) = self.block_from_element(element)? {
            blocks.push(block);
            return Ok(());
        }

        if is_field(element) {
            return Ok(());
        }

        for child in &element.children {
            if let HtmlNode::Element(child) = child {
                self.collect_blocks(child, blocks)?;
            }
        }

        Ok(())
    }

    fn block_from_element(&self, element: &HtmlElement) -> Result<Option<Block>> {
        let transport = TransportElement::new(element)?;
        Ok(match tag_name(element).as_deref() {
            Some("typlite-heading") => Some({
                let level = transport
                    .scalar("level")
                    .and_then(|level| level.parse::<u8>().ok())
                    .unwrap_or(1);
                Block::Heading {
                    level,
                    body: transport.content_inlines("body", self.introspector)?,
                }
            }),
            Some("typlite-paragraph") => Some(Block::Paragraph(
                transport.content_inlines("body", self.introspector)?,
            )),
            Some("typlite-raw") => Some(Block::Raw {
                lang: transport
                    .scalar("lang")
                    .filter(|lang| lang.as_str() != "none"),
                text: transport.scalar("text").unwrap_or_default(),
            }),
            Some("typlite-quote") => Some(Block::Quote(
                transport.content_blocks("body", self.introspector)?,
            )),
            Some("typlite-figure") => Some(Block::Figure {
                body: transport.content_blocks("body", self.introspector)?,
                caption: transport.content_inlines("caption", self.introspector)?,
                alt: transport
                    .scalar("alt")
                    .filter(|value| !value.is_empty() && value.as_str() != "none"),
            }),
            Some("typlite-align") => Some(Block::Align {
                alignment: transport.scalar("alignment"),
                body: transport.content_blocks("body", self.introspector)?,
            }),
            Some("typlite-math-equation") => Some(Block::Math(transport.math("body")?)),
            Some("typlite-table") => Some(Block::Table {
                rows: collect_table_rows(element, "table-cell", self.introspector)?,
                alignments: collect_table_alignments(element),
            }),
            Some("typlite-grid") => Some(Block::Table {
                rows: collect_table_rows(element, "grid-cell", self.introspector)?,
                alignments: collect_table_alignments(element),
            }),
            Some("typlite-list") => Some(Block::List {
                ordered: false,
                tight: transport.bool("tight"),
                numbering: None,
                start: None,
                reversed: false,
                full: false,
                items: collect_list_items(element, false, self.introspector)?,
            }),
            Some("typlite-enum") => Some(Block::List {
                ordered: true,
                tight: transport.bool("tight"),
                numbering: transport
                    .scalar("numbering")
                    .filter(|value| value.as_str() != "none"),
                start: transport
                    .scalar("start")
                    .filter(|value| value.as_str() != "auto")
                    .and_then(|value| value.parse::<i64>().ok()),
                reversed: transport.bool("reversed"),
                full: transport.bool("full"),
                items: collect_list_items(element, true, self.introspector)?,
            }),
            Some("typlite-terms") => Some(Block::Terms {
                items: collect_term_items(element, self.introspector)?,
            }),
            Some(tag) => match block_spec_from_tag(&tag) {
                Some(spec) => block_from_element_kind(
                    spec.kind,
                    BlockElementData {
                        fields: collect_element_fields(
                            element,
                            spec,
                            FieldMode::Block,
                            self.introspector,
                        )?,
                        body: collect_block_element_body(element, spec, self.introspector)?,
                    },
                ),
                None => None,
            },
            None => None,
        })
    }
}

fn block_spec_from_tag(tag: &str) -> Option<&'static ElementSpec> {
    let kind = tag.strip_prefix("typlite-")?;
    let spec = spec_by_kind(kind)?;
    matches!(spec.mode, ElementMode::Block | ElementMode::BlockOrInline).then_some(spec)
}

fn collect_block_element_body(
    element: &HtmlElement,
    spec: &'static ElementSpec,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    let transport = TransportElement::new(element)?;
    let mut blocks = Vec::new();

    for field in content_fields(spec) {
        blocks.extend(transport.content_blocks(field, introspector)?);
    }

    Ok(blocks)
}

fn collect_field_blocks(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Element(element) if is_field(element) => {
                blocks.extend(collect_item_blocks(&element.children, introspector)?);
            }
            _ => blocks.extend(collect_item_blocks(
                std::slice::from_ref(node),
                introspector,
            )?),
        }
    }

    Ok(blocks)
}

fn element_field_inlines(
    element: &HtmlElement,
    name: &str,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    TransportElement::new(element)?.content_inlines(name, introspector)
}

fn element_field_math(element: &HtmlElement, name: &str) -> Result<MathNode> {
    TransportElement::new(element)?.math(name)
}

fn collect_list_items(
    element: &HtmlElement,
    ordered: bool,
    introspector: &Introspector,
) -> Result<Vec<ListItem>> {
    let transport = TransportElement::new(element)?;
    let Some(children) = transport.field("children") else {
        if let Some(Value::Array(children)) = transport.encoded_field("children") {
            return encoded::list_items_from_array(&children, ordered, introspector);
        }
        return Ok(Vec::new());
    };

    let mut items = Vec::new();
    for node in children {
        let item = {
            let HtmlNode::Element(item) = node else {
                continue;
            };

            let body = field_children(item, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .transpose()?
                .unwrap_or_default();
            let number = ordered
                .then(|| field_value(item, "number").filter(|value| value.as_str() != "auto"))
                .flatten();

            ListItem { number, body }
        };
        items.push(item);
    }

    Ok(items)
}

fn collect_term_items(element: &HtmlElement, introspector: &Introspector) -> Result<Vec<TermItem>> {
    let transport = TransportElement::new(element)?;
    let Some(children) = transport.field("children") else {
        if let Some(Value::Array(children)) = transport.encoded_field("children") {
            return encoded::term_items_from_array(&children, introspector);
        }
        return Ok(Vec::new());
    };

    let mut items = Vec::new();
    for node in children {
        let HtmlNode::Element(item) = node else {
            continue;
        };

        let term = field_children(item, "term")
            .map(|children| collect_inlines(children, introspector))
            .transpose()?
            .unwrap_or_default();
        let description = field_children(item, "description")
            .map(|children| collect_item_blocks(children, introspector))
            .transpose()?
            .unwrap_or_default();

        items.push(TermItem { term, description });
    }

    Ok(items)
}

fn collect_table_rows(
    element: &HtmlElement,
    cell_kind: &str,
    introspector: &Introspector,
) -> Result<Vec<TableRow>> {
    let transport = TransportElement::new(element)?;
    let columns = transport
        .field("columns")
        .map(|children| children.iter().filter_map(field_node).count())
        .filter(|columns| *columns > 0)
        .or_else(|| {
            transport
                .encoded_field("columns")
                .and_then(|value| match value {
                    Value::Array(values) => Some(values.len()),
                    _ => None,
                })
                .filter(|columns| *columns > 0)
        })
        .unwrap_or(1);

    let Some(children) = transport.field("children") else {
        if let Some(Value::Array(children)) = transport.encoded_field("children") {
            let mut rows = Vec::new();
            let mut row = Vec::new();
            for child in children {
                encoded::collect_table_cells(child, cell_kind, introspector, &mut row)?;
                let mut occupied_columns: usize = row.iter().map(|cell| cell.colspan).sum();
                while occupied_columns >= columns {
                    let mut drained = Vec::new();
                    let mut drained_columns = 0usize;
                    while drained_columns < columns {
                        let cell = row.remove(0);
                        drained_columns += cell.colspan;
                        drained.push(cell);
                    }
                    rows.push(TableRow { cells: drained });
                    occupied_columns = row.iter().map(|cell| cell.colspan).sum();
                }
            }
            if !row.is_empty() {
                rows.push(TableRow { cells: row });
            }
            return Ok(rows);
        }
        return Ok(Vec::new());
    };

    let mut rows = Vec::new();
    let mut row = Vec::new();

    for child in children {
        collect_table_cells(child, cell_kind, introspector, &mut row)?;
        let mut occupied_columns: usize = row.iter().map(|cell| cell.colspan).sum();
        while occupied_columns >= columns {
            let mut drained = Vec::new();
            let mut drained_columns = 0usize;
            while drained_columns < columns {
                let cell = row.remove(0);
                drained_columns += cell.colspan;
                drained.push(cell);
            }
            rows.push(TableRow { cells: drained });
            occupied_columns = row.iter().map(|cell| cell.colspan).sum();
        }
    }

    if !row.is_empty() {
        rows.push(TableRow { cells: row });
    }

    Ok(rows)
}

fn collect_table_alignments(element: &HtmlElement) -> Vec<TableAlign> {
    let Ok(transport) = TransportElement::new(element) else {
        return Vec::new();
    };

    if let Some(value) = transport.encoded_field("align") {
        return encoded::table_alignments(&value);
    }

    let Some(children) = transport.field("align") else {
        return Vec::new();
    };

    let alignments = children
        .iter()
        .filter_map(field_node)
        .map(|field| table_alignment(&collect_text_without_frames(&field.children)))
        .collect::<Vec<_>>();

    if alignments.is_empty() {
        let alignment = table_alignment(&collect_text_without_frames(children));
        if alignment == TableAlign::Default {
            Vec::new()
        } else {
            vec![alignment]
        }
    } else {
        alignments
    }
}

fn collect_table_cells(
    node: &HtmlNode,
    cell_kind: &str,
    introspector: &Introspector,
    out: &mut Vec<TableCell>,
) -> Result<()> {
    let HtmlNode::Element(element) = node else {
        return Ok(());
    };

    if attr(element, "data-typlite").as_deref() == Some(cell_kind) {
        out.push(TableCell {
            body: field_children(element, "body")
                .map(|children| collect_inlines(children, introspector))
                .transpose()?
                .unwrap_or_default(),
            colspan: field_value(element, "colspan")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1),
            rowspan: field_value(element, "rowspan")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1),
            align: field_value(element, "align")
                .map(|value| table_alignment(&value))
                .unwrap_or(TableAlign::Default),
        });
        return Ok(());
    }

    for child in &element.children {
        collect_table_cells(child, cell_kind, introspector, out)?;
    }

    Ok(())
}

fn collect_item_blocks(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Block>> {
    let extractor = Extractor::new(introspector);
    let mut blocks = Vec::new();
    let mut inlines = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => inlines.push(Inline::Text(text.clone())),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                if let Some(block) = extractor.block_from_element(element)? {
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

fn collect_inlines(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Inline>> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push(Inline::Text(text.clone())),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                match attr(element, "data-typlite").as_deref() {
                    Some("emph") => out.push(Inline::Emph(element_field_inlines(
                        element,
                        "body",
                        introspector,
                    )?)),
                    Some("strong") => out.push(Inline::Strong(element_field_inlines(
                        element,
                        "body",
                        introspector,
                    )?)),
                    Some("link") => out.push(Inline::Link {
                        dest: field_value(element, "dest").unwrap_or_default(),
                        body: element_field_inlines(element, "body", introspector)?,
                    }),
                    Some("strike") => out.push(Inline::Strike(element_field_inlines(
                        element,
                        "body",
                        introspector,
                    )?)),
                    Some("sub") => out.push(Inline::Sub(element_field_inlines(
                        element,
                        "body",
                        introspector,
                    )?)),
                    Some("super") => out.push(Inline::Super(element_field_inlines(
                        element,
                        "body",
                        introspector,
                    )?)),
                    Some("math-equation") => {
                        out.push(Inline::Math(element_field_math(element, "body")?))
                    }
                    Some("linebreak") => out.push(Inline::Linebreak),
                    Some("raw") => out.push(Inline::Raw {
                        lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
                        text: field_value(element, "text").unwrap_or_default(),
                    }),
                    Some(kind) => {
                        if let Some(spec) = spec_by_kind(kind) {
                            if let Some(inline) = inline_from_element_kind(
                                spec.kind,
                                InlineElementData {
                                    fields: collect_element_fields(
                                        element,
                                        spec,
                                        FieldMode::Inline,
                                        introspector,
                                    )?,
                                    body: collect_inline_element_body(element, spec, introspector)?,
                                },
                            ) {
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

    Ok(out)
}

fn collect_inline_element_body(
    element: &HtmlElement,
    spec: &'static ElementSpec,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    let transport = TransportElement::new(element)?;
    if let Some(children) = transport.field("body") {
        return collect_inlines(children, introspector);
    }

    for field in content_fields(spec) {
        if field == "body" {
            continue;
        }
        if let Some(children) = transport.field(field) {
            return collect_inlines(children, introspector);
        }
    }

    if let Some(object) = transport.encoded.as_ref() {
        return encoded::inline_element_body(&object, spec, introspector);
    }

    Ok(Vec::new())
}

#[derive(Debug, Clone, Copy)]
enum FieldMode {
    Block,
    Inline,
}

fn collect_element_fields(
    element: &HtmlElement,
    spec: &'static ElementSpec,
    mode: FieldMode,
    introspector: &Introspector,
) -> Result<Vec<ElementField>> {
    let mut fields = Vec::new();
    let transport = TransportElement::new(element)?;

    if transport.encoded.is_none() {
        return Ok(fields);
    }

    for name in spec.fields.iter().copied() {
        if let Some(value) = transport.encoded_field(name) {
            fields.push(ElementField {
                name,
                value: encoded::element_field_value(name, value, mode, introspector)?,
            });
        } else if is_content_field_name(name)
            && let Some(children) = transport.field(name)
        {
            fields.push(ElementField {
                name,
                value: collect_element_field_value(name, children, mode, introspector)?,
            });
        }
    }

    let mut frames = Vec::new();
    collect_frames(&element.children, &mut frames);
    for frame in frames {
        fields.push(ElementField {
            name: "frame",
            value: ElementFieldValue::Inlines(vec![frame_to_inline(frame, introspector)]),
        });
    }

    Ok(fields)
}

fn collect_element_field_value(
    name: &str,
    children: &[HtmlNode],
    mode: FieldMode,
    introspector: &Introspector,
) -> Result<ElementFieldValue> {
    if name == "element" {
        return Ok(ElementFieldValue::Blocks(collect_field_blocks(
            children,
            introspector,
        )?));
    }

    if is_content_field_name(name) {
        Ok(match mode {
            FieldMode::Block => {
                ElementFieldValue::Blocks(collect_field_blocks(children, introspector)?)
            }
            FieldMode::Inline => {
                ElementFieldValue::Inlines(collect_inlines(children, introspector)?)
            }
        })
    } else {
        Ok(ElementFieldValue::Scalar(collect_text(
            children,
            introspector,
        )))
    }
}

fn spec_by_kind(kind: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|spec| spec.kind.name() == kind)
}

fn collect_text(nodes: &[HtmlNode], introspector: &Introspector) -> EcoString {
    let mut out = EcoString::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push_str(text),
            HtmlNode::Element(element) => {
                if !is_field(element) {
                    out.push_str(&collect_text(&element.children, introspector));
                }
            }
            HtmlNode::Frame(frame) => {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&frame_to_svg(frame, introspector));
            }
            HtmlNode::Tag(_) => {}
        }
    }

    out
}

fn collect_frames<'a>(nodes: &'a [HtmlNode], out: &mut Vec<&'a HtmlFrame>) {
    for node in nodes {
        match node {
            HtmlNode::Frame(frame) => out.push(frame),
            HtmlNode::Element(element) => collect_frames(&element.children, out),
            HtmlNode::Text(..) | HtmlNode::Tag(_) => {}
        }
    }
}

fn frame_to_inline(frame: &HtmlFrame, introspector: &Introspector) -> Inline {
    Inline::Frame(FrameImage {
        svg: frame_to_svg(frame, introspector),
    })
}

fn frame_to_svg(frame: &HtmlFrame, introspector: &Introspector) -> EcoString {
    typst_svg::svg_html_frame(
        &frame.inner,
        frame.text_size,
        frame.id.as_deref(),
        &frame.link_points,
        introspector,
    )
    .into()
}

fn math_field(element: &HtmlElement, name: &str) -> Result<MathNode> {
    let Some(raw) = field_value(element, name) else {
        bail!("missing math field `{name}`");
    };
    let value =
        serde_json::from_str::<Value>(&raw).context_ut("cannot parse math field as JSON")?;
    encoded::math_node(&value)
}

fn tag_name(element: &HtmlElement) -> Option<String> {
    Some(element.tag.resolve().as_str().to_owned())
}

fn attr(element: &HtmlElement, name: &str) -> Option<EcoString> {
    element
        .attrs
        .0
        .iter()
        .find(|(attr, _)| attr.resolve().as_str() == name)
        .map(|(_, value)| value.clone())
}

fn field_value(element: &HtmlElement, name: &str) -> Option<EcoString> {
    if let Ok(transport) = TransportElement::new(element) {
        return transport.scalar(name);
    }

    field_element(element, name).map(|field| collect_text_without_frames(&field.children))
}

fn collect_text_without_frames(nodes: &[HtmlNode]) -> EcoString {
    let mut out = EcoString::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push_str(text),
            HtmlNode::Element(element) => {
                if !is_field(element) {
                    out.push_str(&collect_text_without_frames(&element.children));
                }
            }
            HtmlNode::Tag(_) | HtmlNode::Frame(_) => {}
        }
    }

    out
}

fn field_children<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a [HtmlNode]> {
    field_element(element, name).map(|field| field.children.as_slice())
}

fn field_element<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a HtmlElement> {
    element.children.iter().find_map(|child| {
        let HtmlNode::Element(child) = child else {
            return None;
        };

        (is_field(child) && attr(child, "name").as_deref() == Some(name)).then_some(child)
    })
}

fn field_node(node: &HtmlNode) -> Option<&HtmlElement> {
    let HtmlNode::Element(element) = node else {
        return None;
    };

    is_field(element).then_some(element)
}

fn is_field(element: &HtmlElement) -> bool {
    attr(element, "data-typlite-field").as_deref() == Some("true")
}
