//! Experimental extraction from Typst HTML custom elements into typlite IR.

use ecow::EcoString;
use typst::introspection::Introspector;
use typst_html::{HtmlDocument, HtmlElement, HtmlFrame, HtmlNode};

use crate::element_spec::{ELEMENTS, ElementMode, ElementSpec};
use crate::ir::{
    Block, BlockElementData, Document, ElementField, ElementFieldValue, FrameImage, Inline,
    InlineElementData, ListItem, TableCell, TableRow, block_from_element_kind,
    inline_from_element_kind,
};

/// Extracts typlite IR nodes from an HTML document root.
pub fn extract_document(html: &HtmlDocument) -> Document {
    let mut blocks = Vec::new();
    collect_blocks(&html.root, &html.introspector, &mut blocks);
    Document { blocks }
}

fn collect_blocks(element: &HtmlElement, introspector: &Introspector, blocks: &mut Vec<Block>) {
    if let Some(block) = block_from_element(element, introspector) {
        blocks.push(block);
        return;
    }

    if is_field(element) {
        return;
    }

    for child in &element.children {
        if let HtmlNode::Element(child) = child {
            collect_blocks(child, introspector, blocks);
        }
    }
}

fn block_from_element(element: &HtmlElement, introspector: &Introspector) -> Option<Block> {
    match tag_name(element).as_deref() {
        Some("typlite-heading") => Some({
            let level = field_value(element, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1);
            Block::Heading {
                level,
                body: field_children(element, "body")
                    .map(|children| collect_inlines(children, introspector))
                    .unwrap_or_default(),
            }
        }),
        Some("typlite-paragraph") => Some(Block::Paragraph(
            field_children(element, "body")
                .map(|children| collect_inlines(children, introspector))
                .unwrap_or_default(),
        )),
        Some("typlite-raw") => Some(Block::Raw {
            lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
            text: field_value(element, "text").unwrap_or_default(),
        }),
        Some("typlite-quote") => Some(Block::Quote(
            field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .unwrap_or_default(),
        )),
        Some("typlite-figure") => Some(Block::Figure {
            body: field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .unwrap_or_default(),
            caption: field_children(element, "caption")
                .map(|children| collect_inlines(children, introspector))
                .unwrap_or_default(),
        }),
        Some("typlite-align") => Some(Block::Align(
            field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .unwrap_or_default(),
        )),
        Some("typlite-math-equation") => Some(Block::Math(
            field_children(element, "body")
                .map(|children| collect_inlines(children, introspector))
                .unwrap_or_default(),
        )),
        Some("typlite-table") => Some(Block::Table {
            rows: collect_table_rows(element, "table-cell", introspector),
        }),
        Some("typlite-grid") => Some(Block::Table {
            rows: collect_table_rows(element, "grid-cell", introspector),
        }),
        Some("typlite-list") => Some(Block::List {
            ordered: false,
            tight: field_bool(element, "tight"),
            numbering: None,
            start: None,
            reversed: false,
            full: false,
            items: collect_list_items(element, false, introspector),
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
            items: collect_list_items(element, true, introspector),
        }),
        Some(tag) => block_spec_from_tag(&tag).and_then(|spec| {
            block_from_element_kind(
                spec.kind,
                BlockElementData {
                    fields: collect_element_fields(element, spec, FieldMode::Block, introspector),
                    body: collect_block_element_body(element, spec, introspector),
                },
            )
        }),
        None => None,
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
) -> Vec<Block> {
    let mut blocks = Vec::new();

    for field in content_fields(spec) {
        if let Some(children) = field_children(element, field) {
            blocks.extend(collect_field_blocks(children, introspector));
        }
    }

    blocks
}

fn collect_field_blocks(nodes: &[HtmlNode], introspector: &Introspector) -> Vec<Block> {
    let mut blocks = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Element(element) if is_field(element) => {
                blocks.extend(collect_item_blocks(&element.children, introspector));
            }
            _ => blocks.extend(collect_item_blocks(
                std::slice::from_ref(node),
                introspector,
            )),
        }
    }

    blocks
}

fn collect_list_items(
    element: &HtmlElement,
    ordered: bool,
    introspector: &Introspector,
) -> Vec<ListItem> {
    let Some(children) = field_children(element, "children") else {
        return Vec::new();
    };

    children
        .iter()
        .filter_map(|node| {
            let HtmlNode::Element(item) = node else {
                return None;
            };

            let body = field_children(item, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .unwrap_or_default();
            let number = ordered
                .then(|| field_value(item, "number").filter(|value| value.as_str() != "auto"))
                .flatten();

            Some(ListItem { number, body })
        })
        .collect()
}

fn collect_table_rows(
    element: &HtmlElement,
    cell_kind: &str,
    introspector: &Introspector,
) -> Vec<TableRow> {
    let columns = field_children(element, "columns")
        .map(|children| children.iter().filter_map(field_node).count())
        .filter(|columns| *columns > 0)
        .unwrap_or(1);
    let Some(children) = field_children(element, "children") else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    let mut row = Vec::new();

    for child in children {
        collect_table_cells(child, cell_kind, introspector, &mut row);
        while row.len() >= columns {
            rows.push(TableRow {
                cells: row.drain(..columns).collect(),
            });
        }
    }

    if !row.is_empty() {
        rows.push(TableRow { cells: row });
    }

    rows
}

fn collect_table_cells(
    node: &HtmlNode,
    cell_kind: &str,
    introspector: &Introspector,
    out: &mut Vec<TableCell>,
) {
    let HtmlNode::Element(element) = node else {
        return;
    };

    if attr(element, "data-typlite").as_deref() == Some(cell_kind) {
        out.push(TableCell {
            body: field_children(element, "body")
                .map(|children| collect_inlines(children, introspector))
                .unwrap_or_default(),
        });

        let colspan = field_value(element, "colspan")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1);
        for _ in 1..colspan {
            out.push(TableCell { body: Vec::new() });
        }
        return;
    }

    for child in &element.children {
        collect_table_cells(child, cell_kind, introspector, out);
    }
}

fn collect_item_blocks(nodes: &[HtmlNode], introspector: &Introspector) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut inlines = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => inlines.push(Inline::Text(text.clone())),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                if let Some(block) = block_from_element(element, introspector) {
                    flush_paragraph(&mut inlines, &mut blocks);
                    blocks.push(block);
                } else {
                    inlines.extend(collect_inlines(std::slice::from_ref(node), introspector));
                }
            }
            HtmlNode::Frame(frame) => inlines.push(frame_to_inline(frame, introspector)),
            HtmlNode::Tag(_) => {}
        }
    }

    flush_paragraph(&mut inlines, &mut blocks);
    blocks
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
        | Inline::Super(body)
        | Inline::Math(body) => body.iter().any(inline_has_content),
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
        | Inline::Image(data)
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

fn collect_inlines(nodes: &[HtmlNode], introspector: &Introspector) -> Vec<Inline> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push(Inline::Text(text.clone())),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                match attr(element, "data-typlite").as_deref() {
                    Some("emph") => out.push(Inline::Emph(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .unwrap_or_default(),
                    )),
                    Some("strong") => out.push(Inline::Strong(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .unwrap_or_default(),
                    )),
                    Some("link") => out.push(Inline::Link {
                        dest: field_value(element, "dest").unwrap_or_default(),
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .unwrap_or_default(),
                    }),
                    Some("strike") => out.push(Inline::Strike(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .unwrap_or_default(),
                    )),
                    Some("sub") => out.push(Inline::Sub(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .unwrap_or_default(),
                    )),
                    Some("super") => out.push(Inline::Super(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .unwrap_or_default(),
                    )),
                    Some("math-equation") => out.push(Inline::Math(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .unwrap_or_default(),
                    )),
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
                                    ),
                                    body: collect_inline_element_body(element, spec, introspector),
                                },
                            ) {
                                out.push(inline);
                            }
                        }
                    }
                    None => {
                        out.extend(collect_inlines(&element.children, introspector));
                    }
                }
            }
            HtmlNode::Frame(frame) => out.push(frame_to_inline(frame, introspector)),
            HtmlNode::Tag(_) => {}
        }
    }

    out
}

fn collect_inline_element_body(
    element: &HtmlElement,
    spec: &'static ElementSpec,
    introspector: &Introspector,
) -> Vec<Inline> {
    for field in content_fields(spec) {
        if let Some(children) = field_children(element, field) {
            return collect_inlines(children, introspector);
        }
    }

    Vec::new()
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
) -> Vec<ElementField> {
    spec.fields
        .iter()
        .copied()
        .filter_map(|name| {
            let children = field_children(element, name)?;
            Some(ElementField {
                name,
                value: collect_element_field_value(name, children, mode, introspector),
            })
        })
        .chain(
            field_children(element, "frame").map(|children| ElementField {
                name: "frame",
                value: ElementFieldValue::Inlines(collect_inlines(children, introspector)),
            }),
        )
        .collect()
}

fn collect_element_field_value(
    name: &str,
    children: &[HtmlNode],
    mode: FieldMode,
    introspector: &Introspector,
) -> ElementFieldValue {
    if is_content_field_name(name) {
        match mode {
            FieldMode::Block => {
                ElementFieldValue::Blocks(collect_field_blocks(children, introspector))
            }
            FieldMode::Inline => {
                ElementFieldValue::Inlines(collect_inlines(children, introspector))
            }
        }
    } else {
        ElementFieldValue::Scalar(collect_text(children, introspector))
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
            | "term"
            | "description"
            | "supplement"
            | "citation"
            | "element"
    )
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
