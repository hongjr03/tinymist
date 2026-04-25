//! Experimental extraction from Typst HTML custom elements into typlite IR.

use ecow::EcoString;
use serde_json::{Map, Value};
use tinymist_std::error::prelude::*;
use typst::introspection::Introspector;
use typst_html::{HtmlDocument, HtmlElement, HtmlFrame, HtmlNode};

use crate::Result;
use crate::element_spec::{ELEMENTS, ElementMode, ElementSpec};
use crate::ir::{
    Block, BlockElementData, Document, ElementField, ElementFieldValue, FrameImage, Inline,
    InlineElementData, ListItem, MathField, MathNode, MathValue, TableAlign, TableCell, TableRow,
    TermItem, block_from_element_kind, inline_from_element_kind,
};

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

fn block_from_element(element: &HtmlElement, introspector: &Introspector) -> Result<Option<Block>> {
    Ok(match tag_name(element).as_deref() {
        Some("typlite-heading") => Some({
            let level = field_value(element, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1);
            Block::Heading {
                level,
                body: element_field_inlines(element, "body", introspector)?,
            }
        }),
        Some("typlite-paragraph") => Some(Block::Paragraph(element_field_inlines(
            element,
            "body",
            introspector,
        )?)),
        Some("typlite-raw") => Some(Block::Raw {
            lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
            text: field_value(element, "text").unwrap_or_default(),
        }),
        Some("typlite-quote") => Some(Block::Quote(element_field_blocks(
            element,
            "body",
            introspector,
        )?)),
        Some("typlite-figure") => Some(Block::Figure {
            body: element_field_blocks(element, "body", introspector)?,
            caption: element_field_inlines(element, "caption", introspector)?,
            alt: field_value(element, "alt")
                .filter(|value| !value.is_empty() && value.as_str() != "none"),
        }),
        Some("typlite-align") => Some(Block::Align {
            alignment: field_value(element, "alignment"),
            body: element_field_blocks(element, "body", introspector)?,
        }),
        Some("typlite-math-equation") => Some(Block::Math(element_field_math(element, "body")?)),
        Some("typlite-table") => Some(Block::Table {
            rows: collect_table_rows(element, "table-cell", introspector)?,
            alignments: collect_table_alignments(element),
        }),
        Some("typlite-grid") => Some(Block::Table {
            rows: collect_table_rows(element, "grid-cell", introspector)?,
            alignments: collect_table_alignments(element),
        }),
        Some("typlite-list") => Some(Block::List {
            ordered: false,
            tight: field_bool(element, "tight"),
            numbering: None,
            start: None,
            reversed: false,
            full: false,
            items: collect_list_items(element, false, introspector)?,
        }),
        Some("typlite-enum") => Some(Block::List {
            ordered: true,
            tight: field_bool(element, "tight"),
            numbering: field_value(element, "numbering").filter(|value| value.as_str() != "none"),
            start: field_value(element, "start")
                .filter(|value| value.as_str() != "auto")
                .and_then(|value| value.parse::<i64>().ok()),
            reversed: field_bool(element, "reversed"),
            full: field_bool(element, "full"),
            items: collect_list_items(element, true, introspector)?,
        }),
        Some("typlite-terms") => Some(Block::Terms {
            items: collect_term_items(element, introspector)?,
        }),
        Some(tag) => match block_spec_from_tag(&tag) {
            Some(spec) => block_from_element_kind(
                spec.kind,
                BlockElementData {
                    fields: collect_element_fields(element, spec, FieldMode::Block, introspector)?,
                    body: collect_block_element_body(element, spec, introspector)?,
                },
            ),
            None => None,
        },
        None => None,
    })
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
    let mut blocks = Vec::new();

    for field in content_fields(spec) {
        blocks.extend(element_field_blocks(element, field, introspector)?);
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

fn element_field_blocks(
    element: &HtmlElement,
    name: &str,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    if let Some(children) = field_children(element, name) {
        return collect_item_blocks(children, introspector);
    }

    if let Some(value) = encoded_field(element, name)? {
        return content_blocks_from_json(&value, introspector);
    }

    Ok(Vec::new())
}

fn element_field_inlines(
    element: &HtmlElement,
    name: &str,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    if let Some(children) = field_children(element, name) {
        return collect_inlines(children, introspector);
    }

    if let Some(value) = encoded_field(element, name)? {
        return content_inlines_from_json(&value, introspector);
    }

    Ok(Vec::new())
}

fn element_field_math(element: &HtmlElement, name: &str) -> Result<MathNode> {
    if field_children(element, name).is_some() {
        return math_field(element, name);
    }

    if let Some(value) = encoded_field(element, name)? {
        return parse_math_node(&value);
    }

    math_field(element, name)
}

fn collect_list_items(
    element: &HtmlElement,
    ordered: bool,
    introspector: &Introspector,
) -> Result<Vec<ListItem>> {
    let Some(children) = field_children(element, "children") else {
        if let Some(Value::Array(children)) = encoded_field(element, "children")? {
            return list_items_from_array(&children, ordered, introspector);
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
    let Some(children) = field_children(element, "children") else {
        if let Some(Value::Array(children)) = encoded_field(element, "children")? {
            return term_items_from_array(&children, introspector);
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

fn list_items_from_json(
    value: Option<&Value>,
    ordered: bool,
    introspector: &Introspector,
) -> Result<Vec<ListItem>> {
    let Some(Value::Array(children)) = value else {
        return Ok(Vec::new());
    };
    list_items_from_array(children, ordered, introspector)
}

fn list_items_from_array(
    children: &[Value],
    ordered: bool,
    introspector: &Introspector,
) -> Result<Vec<ListItem>> {
    let mut items = Vec::new();
    for item in children {
        let Some(item) = item.as_object() else {
            continue;
        };
        let body = item
            .get("body")
            .map(|body| content_blocks_from_json(body, introspector))
            .transpose()?
            .unwrap_or_default();
        let number = ordered
            .then(|| {
                item.get("number")
                    .map(scalar_from_json)
                    .filter(|value| value.as_str() != "auto")
            })
            .flatten();

        items.push(ListItem { number, body });
    }
    Ok(items)
}

fn term_items_from_json(
    value: Option<&Value>,
    introspector: &Introspector,
) -> Result<Vec<TermItem>> {
    let Some(Value::Array(children)) = value else {
        return Ok(Vec::new());
    };
    term_items_from_array(children, introspector)
}

fn term_items_from_array(children: &[Value], introspector: &Introspector) -> Result<Vec<TermItem>> {
    let mut items = Vec::new();
    for item in children {
        let Some(item) = item.as_object() else {
            continue;
        };
        let term = item
            .get("term")
            .map(|term| content_inlines_from_json(term, introspector))
            .transpose()?
            .unwrap_or_default();
        let description = item
            .get("description")
            .map(|description| content_blocks_from_json(description, introspector))
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
    let columns = field_children(element, "columns")
        .map(|children| children.iter().filter_map(field_node).count())
        .filter(|columns| *columns > 0)
        .or_else(|| {
            encoded_field(element, "columns")
                .ok()
                .flatten()
                .and_then(|value| match value {
                    Value::Array(values) => Some(values.len()),
                    _ => None,
                })
                .filter(|columns| *columns > 0)
        })
        .unwrap_or(1);

    let Some(children) = field_children(element, "children") else {
        if let Some(Value::Array(children)) = encoded_field(element, "children")? {
            let mut rows = Vec::new();
            let mut row = Vec::new();
            for child in &children {
                collect_table_cells_from_json(child, cell_kind, introspector, &mut row)?;
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
    if let Some(value) = encoded_field(element, "align").ok().flatten() {
        return table_alignments_from_json(&value);
    }

    let Some(children) = field_children(element, "align") else {
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

fn table_alignments_from_json(value: &Value) -> Vec<TableAlign> {
    match value {
        Value::Array(values) => values
            .iter()
            .map(scalar_from_json)
            .map(|value| table_alignment(&value))
            .collect(),
        _ => {
            let alignment = table_alignment(&scalar_from_json(value));
            if alignment == TableAlign::Default {
                Vec::new()
            } else {
                vec![alignment]
            }
        }
    }
}

fn table_alignment(value: &str) -> TableAlign {
    match value.trim() {
        "left" | "start" => TableAlign::Left,
        "center" | "horizon" => TableAlign::Center,
        "right" | "end" => TableAlign::Right,
        _ => TableAlign::Default,
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

fn collect_table_cells_from_json(
    value: &Value,
    cell_kind: &str,
    introspector: &Introspector,
    out: &mut Vec<TableCell>,
) -> Result<()> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };

    let func = json_func(object)?;
    if func == cell_kind || func == cell_kind.replace('-', ".") || func == "cell" {
        out.push(TableCell {
            body: object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
            colspan: json_scalar(object, "colspan")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1),
            rowspan: json_scalar(object, "rowspan")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1),
            align: json_scalar(object, "align")
                .map(|value| table_alignment(&value))
                .unwrap_or(TableAlign::Default),
        });
        return Ok(());
    }

    for value in object.values() {
        match value {
            Value::Array(values) => {
                for value in values {
                    collect_table_cells_from_json(value, cell_kind, introspector, out)?;
                }
            }
            Value::Object(_) => collect_table_cells_from_json(value, cell_kind, introspector, out)?,
            _ => {}
        }
    }

    Ok(())
}

fn collect_item_blocks(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut inlines = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => inlines.push(Inline::Text(text.clone())),
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
        blocks.push(Block::Paragraph(std::mem::take(inlines)));
    } else {
        inlines.clear();
    }
}

fn inline_has_content(inline: &Inline) -> bool {
    match inline {
        Inline::Text(text) => !text.trim().is_empty(),
        Inline::Linebreak => false,
        Inline::Frame(_) => true,
        Inline::Emph(body)
        | Inline::Strong(body)
        | Inline::Strike(body)
        | Inline::Sub(body)
        | Inline::Super(body) => body.iter().any(inline_has_content),
        Inline::Math(_) => true,
        Inline::Link { dest, body } => !dest.is_empty() || body.iter().any(inline_has_content),
        Inline::Raw { text, .. } => !text.is_empty(),
        Inline::Circle(data)
        | Inline::Curve(data)
        | Inline::Ellipse(data)
        | Inline::Line(data)
        | Inline::Path(data)
        | Inline::Polygon(data)
        | Inline::Rect(data)
        | Inline::Square(data) => data
            .inlines("frame")
            .is_some_and(|frames| !frames.is_empty()),
        Inline::Image(data) => data
            .scalar("source")
            .is_some_and(|source| !source.is_empty()),
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
        | Inline::Underline(data) => data.body.iter().any(inline_has_content),
    }
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
    if let Some(children) = field_children(element, "body") {
        return collect_inlines(children, introspector);
    }

    for field in content_fields(spec) {
        if field == "body" {
            continue;
        }
        if let Some(children) = field_children(element, field) {
            return collect_inlines(children, introspector);
        }
    }

    if let Some(object) = encoded_object(element)? {
        return inline_element_body_from_object(&object, spec, introspector);
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

    if let Some(object) = encoded_object(element)? {
        for name in spec.fields.iter().copied() {
            if is_content_field_name(name)
                && let Some(children) = field_children(element, name)
            {
                fields.push(ElementField {
                    name,
                    value: collect_element_field_value(name, children, mode, introspector)?,
                });
            } else if let Some(value) = object.get(name) {
                fields.push(ElementField {
                    name,
                    value: element_field_value_from_json(name, value, mode, introspector)?,
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

        return Ok(fields);
    }

    for name in spec.fields.iter().copied() {
        if let Some(children) = field_children(element, name) {
            fields.push(ElementField {
                name,
                value: collect_element_field_value(name, children, mode, introspector)?,
            });
        }
    }

    if let Some(children) = field_children(element, "frame") {
        fields.push(ElementField {
            name: "frame",
            value: ElementFieldValue::Inlines(collect_inlines(children, introspector)?),
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

fn element_field_value_from_json(
    name: &str,
    value: &Value,
    mode: FieldMode,
    introspector: &Introspector,
) -> Result<ElementFieldValue> {
    if name == "element" {
        return Ok(ElementFieldValue::Blocks(content_blocks_from_json(
            value,
            introspector,
        )?));
    }

    if is_content_field_name(name) {
        Ok(match mode {
            FieldMode::Block => {
                ElementFieldValue::Blocks(content_blocks_from_json(value, introspector)?)
            }
            FieldMode::Inline => {
                ElementFieldValue::Inlines(content_inlines_from_json(value, introspector)?)
            }
        })
    } else if matches!(name, "source" | "sources") {
        Ok(ElementFieldValue::Scalar(source_scalar_from_json(value)))
    } else {
        Ok(ElementFieldValue::Scalar(scalar_from_json(value)))
    }
}

fn content_fields(spec: &'static ElementSpec) -> impl Iterator<Item = &'static str> {
    spec.fields
        .iter()
        .copied()
        .filter(|field| is_content_field_name(field))
}

fn is_content_field_name(field: &str) -> bool {
    matches!(
        field,
        "body"
            | "children"
            | "title"
            | "caption"
            | "attribution"
            | "term"
            | "description"
            | "supplement"
            | "citation"
            | "element"
    )
}

fn content_inlines_from_json(value: &Value, introspector: &Introspector) -> Result<Vec<Inline>> {
    if let Value::Array(values) = value {
        let mut out = Vec::new();
        for value in values {
            out.extend(content_inlines_from_json(value, introspector)?);
        }
        return Ok(out);
    }

    let Some(object) = value.as_object() else {
        return Ok(vec![Inline::Text(scalar_from_json(value))]);
    };
    let func = json_func(object)?;

    Ok(match func {
        "sequence" => {
            let mut out = Vec::new();
            for child in json_array(object, "children") {
                out.extend(content_inlines_from_json(child, introspector)?);
            }
            out
        }
        "styled" => object
            .get("child")
            .map(|child| content_inlines_from_json(child, introspector))
            .transpose()?
            .unwrap_or_default(),
        "text" | "symbol" => vec![Inline::Text(
            json_scalar(object, "text").unwrap_or_default(),
        )],
        "tag" | "frame" => Vec::new(),
        "elem" => html_elem_inlines_from_json(object, introspector)?,
        "space" | "h" => vec![Inline::Text(" ".into())],
        "linebreak" => vec![Inline::Linebreak],
        "emph" => vec![Inline::Emph(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "strong" => vec![Inline::Strong(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "link" => vec![Inline::Link {
            dest: json_scalar(object, "dest").unwrap_or_default(),
            body: object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        }],
        "strike" => vec![Inline::Strike(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "sub" => vec![Inline::Sub(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "super" => vec![Inline::Super(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "equation" => vec![Inline::Math(parse_math_node(
            object
                .get("body")
                .context("encoded equation is missing `body`")?,
        )?)],
        "raw" => vec![Inline::Raw {
            lang: json_scalar(object, "lang").filter(|lang| lang.as_str() != "none"),
            text: json_scalar(object, "text").unwrap_or_default(),
        }],
        kind => {
            let Some(spec) = spec_by_selector_or_kind(kind) else {
                bail!("typlite content inline rendering is not implemented for `{kind}`");
            };
            let Some(inline) = inline_from_element_kind(
                spec.kind,
                InlineElementData {
                    fields: element_fields_from_object(
                        object,
                        spec,
                        FieldMode::Inline,
                        introspector,
                    )?,
                    body: inline_element_body_from_object(object, spec, introspector)?,
                },
            ) else {
                bail!("encoded element `{kind}` is not an inline element");
            };
            vec![inline]
        }
    })
}

fn content_blocks_from_json(value: &Value, introspector: &Introspector) -> Result<Vec<Block>> {
    if let Value::Array(values) = value {
        let mut out = Vec::new();
        for value in values {
            out.extend(content_blocks_from_json(value, introspector)?);
        }
        return Ok(out);
    }

    let Some(object) = value.as_object() else {
        let text = scalar_from_json(value);
        return Ok((!text.trim().is_empty())
            .then(|| Block::Paragraph(vec![Inline::Text(text)]))
            .into_iter()
            .collect());
    };
    let func = json_func(object)?;

    Ok(match func {
        "sequence" => {
            let mut out = Vec::new();
            let mut inlines = Vec::new();
            for child in json_array(object, "children") {
                let child_blocks = content_blocks_from_json(child, introspector)?;
                if child_blocks.len() == 1
                    && let Block::Paragraph(child_inlines) = &child_blocks[0]
                {
                    inlines.extend(child_inlines.clone());
                } else {
                    flush_paragraph(&mut inlines, &mut out);
                    out.extend(child_blocks);
                }
            }
            flush_paragraph(&mut inlines, &mut out);
            out
        }
        "styled" => object
            .get("child")
            .map(|child| content_blocks_from_json(child, introspector))
            .transpose()?
            .unwrap_or_default(),
        "tag" | "frame" => Vec::new(),
        "text" | "symbol" | "space" | "h" | "linebreak" | "equation" | "elem" => {
            let inlines = content_inlines_from_json(value, introspector)?;
            if inlines.iter().any(inline_has_content) {
                vec![Block::Paragraph(inlines)]
            } else {
                Vec::new()
            }
        }
        kind => {
            if let Some(block) = block_from_object(kind, object, introspector)? {
                vec![block]
            } else {
                let inlines = content_inlines_from_json(value, introspector)?;
                if inlines.iter().any(inline_has_content) {
                    vec![Block::Paragraph(inlines)]
                } else {
                    Vec::new()
                }
            }
        }
    })
}

fn html_elem_inlines_from_json(
    object: &Map<String, Value>,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    let attrs = object.get("attrs").and_then(Value::as_object);
    let Some(kind) = attrs
        .and_then(|attrs| attrs.get("data-typlite"))
        .and_then(Value::as_str)
    else {
        return object
            .get("body")
            .map(|body| content_inlines_from_json(body, introspector))
            .transpose()
            .map(Option::unwrap_or_default);
    };

    let Some(body) = object.get("body") else {
        return Ok(Vec::new());
    };
    let raw = content_inlines_from_json(body, introspector)?
        .into_iter()
        .filter_map(|inline| match inline {
            Inline::Text(text) => Some(text),
            _ => None,
        })
        .collect::<EcoString>();
    let value = serde_json::from_str::<Value>(&raw).with_context_ut(
        "cannot parse nested typlite HTML transport",
        || {
            Some(Box::new([
                ("kind", kind.to_owned()),
                ("raw", raw.to_string()),
            ]))
        },
    )?;
    let object = value
        .as_object()
        .context("nested typlite HTML transport must be an object")?;

    Ok(match kind {
        "emph" => vec![Inline::Emph(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "strong" => vec![Inline::Strong(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "raw" => vec![Inline::Raw {
            lang: json_scalar(object, "lang").filter(|lang| lang.as_str() != "none"),
            text: json_scalar(object, "text").unwrap_or_default(),
        }],
        "math-equation" => vec![Inline::Math(parse_math_node(
            object
                .get("body")
                .context("encoded math equation is missing `body`")?,
        )?)],
        kind => {
            let Some(spec) = spec_by_selector_or_kind(kind) else {
                bail!("nested typlite HTML transport `{kind}` is not implemented");
            };
            let Some(inline) = inline_from_element_kind(
                spec.kind,
                InlineElementData {
                    fields: element_fields_from_object(
                        object,
                        spec,
                        FieldMode::Inline,
                        introspector,
                    )?,
                    body: inline_element_body_from_object(object, spec, introspector)?,
                },
            ) else {
                bail!("nested typlite HTML transport `{kind}` is not an inline element");
            };
            vec![inline]
        }
    })
}

fn block_from_object(
    kind: &str,
    object: &Map<String, Value>,
    introspector: &Introspector,
) -> Result<Option<Block>> {
    Ok(match kind {
        "heading" => Some(Block::Heading {
            level: json_scalar(object, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1),
            body: object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        }),
        "par" | "paragraph" => Some(Block::Paragraph(
            object
                .get("body")
                .map(|body| content_inlines_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )),
        "math.equation" | "math-equation" | "equation" => Some(Block::Math(parse_math_node(
            object
                .get("body")
                .context("encoded equation is missing `body`")?,
        )?)),
        "raw" => Some(Block::Raw {
            lang: json_scalar(object, "lang").filter(|lang| lang.as_str() != "none"),
            text: json_scalar(object, "text").unwrap_or_default(),
        }),
        "quote" => Some(Block::Quote(
            object
                .get("body")
                .map(|body| content_blocks_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )),
        "figure" => Some(Block::Figure {
            body: object
                .get("body")
                .map(|body| content_blocks_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
            caption: object
                .get("caption")
                .map(|caption| content_inlines_from_json(caption, introspector))
                .transpose()?
                .unwrap_or_default(),
            alt: json_scalar(object, "alt")
                .filter(|value| !value.is_empty() && value.as_str() != "none"),
        }),
        "align" => Some(Block::Align {
            alignment: json_scalar(object, "alignment"),
            body: object
                .get("body")
                .map(|body| content_blocks_from_json(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        }),
        "list" => Some(Block::List {
            ordered: false,
            tight: json_scalar(object, "tight").is_some_and(|value| value.as_str() == "true"),
            numbering: None,
            start: None,
            reversed: false,
            full: false,
            items: list_items_from_json(object.get("children"), false, introspector)?,
        }),
        "enum" => Some(Block::List {
            ordered: true,
            tight: json_scalar(object, "tight").is_some_and(|value| value.as_str() == "true"),
            numbering: json_scalar(object, "numbering").filter(|value| value.as_str() != "none"),
            start: json_scalar(object, "start")
                .filter(|value| value.as_str() != "auto")
                .and_then(|value| value.parse::<i64>().ok()),
            reversed: json_scalar(object, "reversed").is_some_and(|value| value.as_str() == "true"),
            full: json_scalar(object, "full").is_some_and(|value| value.as_str() == "true"),
            items: list_items_from_json(object.get("children"), true, introspector)?,
        }),
        "terms" => Some(Block::Terms {
            items: term_items_from_json(object.get("children"), introspector)?,
        }),
        "table" | "grid" => {
            let cell_kind = if kind == "grid" {
                "grid-cell"
            } else {
                "table-cell"
            };
            let columns = object
                .get("columns")
                .and_then(Value::as_array)
                .map(Vec::len)
                .filter(|columns| *columns > 0)
                .unwrap_or(1);
            let mut rows = Vec::new();
            let mut row = Vec::new();
            if let Some(Value::Array(children)) = object.get("children") {
                for child in children {
                    collect_table_cells_from_json(child, cell_kind, introspector, &mut row)?;
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
            }
            if !row.is_empty() {
                rows.push(TableRow { cells: row });
            }
            Some(Block::Table {
                rows,
                alignments: object
                    .get("align")
                    .map(table_alignments_from_json)
                    .unwrap_or_default(),
            })
        }
        kind => {
            let Some(spec) = spec_by_selector_or_kind(kind) else {
                return Ok(None);
            };
            block_from_element_kind(
                spec.kind,
                BlockElementData {
                    fields: element_fields_from_object(
                        object,
                        spec,
                        FieldMode::Block,
                        introspector,
                    )?,
                    body: block_element_body_from_object(object, spec, introspector)?,
                },
            )
        }
    })
}

fn element_fields_from_object(
    object: &Map<String, Value>,
    spec: &'static ElementSpec,
    mode: FieldMode,
    introspector: &Introspector,
) -> Result<Vec<ElementField>> {
    let mut fields = Vec::new();
    for name in spec.fields.iter().copied() {
        if let Some(value) = object.get(name) {
            fields.push(ElementField {
                name,
                value: element_field_value_from_json(name, value, mode, introspector)?,
            });
        }
    }
    Ok(fields)
}

fn block_element_body_from_object(
    object: &Map<String, Value>,
    spec: &'static ElementSpec,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    for field in content_fields(spec) {
        if let Some(value) = object.get(field) {
            blocks.extend(content_blocks_from_json(value, introspector)?);
        }
    }
    Ok(blocks)
}

fn inline_element_body_from_object(
    object: &Map<String, Value>,
    spec: &'static ElementSpec,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    if let Some(value) = object.get("body") {
        return content_inlines_from_json(value, introspector);
    }

    for field in content_fields(spec) {
        if field == "body" {
            continue;
        }
        if let Some(value) = object.get(field) {
            return content_inlines_from_json(value, introspector);
        }
    }

    Ok(Vec::new())
}

fn json_func(object: &Map<String, Value>) -> Result<&str> {
    object
        .get("func")
        .and_then(Value::as_str)
        .context("encoded content is missing string field `func`")
}

fn json_array<'a>(object: &'a Map<String, Value>, name: &str) -> &'a [Value] {
    object
        .get(name)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn json_scalar(object: &Map<String, Value>, name: &str) -> Option<EcoString> {
    object.get(name).map(scalar_from_json)
}

fn spec_by_kind(kind: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|spec| spec.kind.name() == kind)
}

fn spec_by_selector_or_kind(kind: &str) -> Option<&'static ElementSpec> {
    ELEMENTS
        .iter()
        .find(|spec| spec.selector == kind || spec.kind.name() == kind)
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
    parse_math_node(&value)
}

fn parse_math_node(value: &Value) -> Result<MathNode> {
    let Some(object) = value.as_object() else {
        bail!("math node must be encoded as an object, got {value}");
    };
    let func = object
        .get("func")
        .and_then(Value::as_str)
        .context("math node is missing string field `func`")?;

    let mut fields = Vec::new();
    for (name, value) in object {
        if name == "func" {
            continue;
        }
        fields.push(MathField {
            name: name.as_str().into(),
            value: parse_math_value(value).with_context_ut("cannot parse math field", || {
                Some(Box::new([
                    ("func", func.to_owned()),
                    ("field", name.to_owned()),
                    ("value", value.to_string()),
                ]))
            })?,
        });
    }

    Ok(MathNode {
        func: func.into(),
        fields,
    })
}

fn parse_math_value(value: &Value) -> Result<MathValue> {
    match value {
        Value::Null => Ok(MathValue::None),
        Value::Bool(value) => Ok(MathValue::Bool(*value)),
        Value::Number(value) => Ok(MathValue::Scalar(value.to_string().into())),
        Value::String(value) => Ok(MathValue::Scalar(value.as_str().into())),
        Value::Object(_) => Ok(MathValue::Node(Box::new(parse_math_node(value)?))),
        Value::Array(values) => parse_math_array(values),
    }
}

fn parse_math_array(values: &[Value]) -> Result<MathValue> {
    if values.is_empty() {
        return Ok(MathValue::Nodes(Vec::new()));
    }

    if values.iter().all(Value::is_object) {
        let mut nodes = Vec::new();
        for value in values {
            nodes.push(parse_math_node(value)?);
        }
        return Ok(MathValue::Nodes(nodes));
    }

    if values.iter().all(Value::is_array) {
        let mut rows = Vec::new();
        for row in values {
            let Some(row) = row.as_array() else {
                unreachable!("checked by all(Value::is_array)");
            };
            let mut cells = Vec::new();
            for cell in row {
                cells.push(parse_math_node(cell)?);
            }
            rows.push(cells);
        }
        return Ok(MathValue::Rows(rows));
    }

    Ok(MathValue::Scalar(
        Value::Array(values.to_vec()).to_string().into(),
    ))
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
    if let Some(value) = encoded_field(element, name).ok().flatten() {
        return Some(scalar_from_json(&value));
    }

    field_element(element, name).map(|field| collect_text_without_frames(&field.children))
}

fn encoded_field(element: &HtmlElement, name: &str) -> Result<Option<Value>> {
    Ok(encoded_object(element)?.and_then(|object| object.get(name).cloned()))
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

fn scalar_from_json(value: &Value) -> EcoString {
    match value {
        Value::Null => "none".into(),
        Value::Bool(value) => {
            if *value {
                "true".into()
            } else {
                "false".into()
            }
        }
        Value::Number(value) => value.to_string().into(),
        Value::String(value) => value.as_str().into(),
        Value::Array(_) | Value::Object(_) => value.to_string().into(),
    }
}

fn source_scalar_from_json(value: &Value) -> EcoString {
    source_value_from_json(value).to_string().into()
}

fn source_value_from_json(value: &Value) -> Value {
    match value {
        Value::String(value) => {
            serde_json::json!({ "kind": "string", "value": value })
        }
        Value::Array(values) => Value::Array(values.iter().map(source_value_from_json).collect()),
        Value::Object(_) => value.clone(),
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
    }
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

fn field_bool(element: &HtmlElement, name: &str) -> bool {
    field_value(element, name).is_some_and(|value| value.as_str() == "true")
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
