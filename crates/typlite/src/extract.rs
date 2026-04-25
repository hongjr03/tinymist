//! Experimental extraction from Typst HTML custom elements into typlite IR.

use ecow::EcoString;
use typst_html::{HtmlElement, HtmlNode};

use crate::element_spec::{ELEMENTS, ElementMode, ElementSpec};
use crate::ir::{
    Block, BlockElementData, Document, Inline, InlineElementData, ListItem, TableCell, TableRow,
    block_from_element_kind, inline_from_element_kind,
};

/// Extracts typlite IR nodes from an HTML document root.
pub fn extract_document(root: &HtmlElement) -> Document {
    let mut blocks = Vec::new();
    collect_blocks(root, &mut blocks);
    Document { blocks }
}

fn collect_blocks(element: &HtmlElement, blocks: &mut Vec<Block>) {
    if let Some(block) = block_from_element(element) {
        blocks.push(block);
        return;
    }

    if is_field(element) {
        return;
    }

    for child in &element.children {
        if let HtmlNode::Element(child) = child {
            collect_blocks(child, blocks);
        }
    }
}

fn block_from_element(element: &HtmlElement) -> Option<Block> {
    match tag_name(element).as_deref() {
        Some("typlite-heading") => Some({
            let level = field_value(element, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1);
            Block::Heading {
                level,
                body: field_children(element, "body")
                    .map(collect_inlines)
                    .unwrap_or_default(),
            }
        }),
        Some("typlite-paragraph") => Some(Block::Paragraph(
            field_children(element, "body")
                .map(collect_inlines)
                .unwrap_or_default(),
        )),
        Some("typlite-raw") => Some(Block::Raw {
            lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
            text: field_value(element, "text").unwrap_or_default(),
        }),
        Some("typlite-quote") => Some(Block::Quote(
            field_children(element, "body")
                .map(collect_item_blocks)
                .unwrap_or_default(),
        )),
        Some("typlite-figure") => Some(Block::Figure {
            body: field_children(element, "body")
                .map(collect_item_blocks)
                .unwrap_or_default(),
            caption: field_children(element, "caption")
                .map(collect_inlines)
                .unwrap_or_default(),
        }),
        Some("typlite-align") => Some(Block::Align(
            field_children(element, "body")
                .map(collect_item_blocks)
                .unwrap_or_default(),
        )),
        Some("typlite-math-equation") => Some(Block::Math(
            field_children(element, "body")
                .map(collect_inlines)
                .unwrap_or_default(),
        )),
        Some("typlite-table") => Some(Block::Table {
            rows: collect_table_rows(element, "table-cell"),
        }),
        Some("typlite-grid") => Some(Block::Table {
            rows: collect_table_rows(element, "grid-cell"),
        }),
        Some("typlite-list") => Some(Block::List {
            ordered: false,
            tight: field_bool(element, "tight"),
            numbering: None,
            start: None,
            reversed: false,
            full: false,
            items: collect_list_items(element, false),
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
            items: collect_list_items(element, true),
        }),
        Some(tag) => block_spec_from_tag(&tag).and_then(|spec| {
            block_from_element_kind(
                spec.kind,
                BlockElementData {
                    fields: Vec::new(),
                    body: collect_block_element_body(element, spec),
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

fn collect_block_element_body(element: &HtmlElement, spec: &'static ElementSpec) -> Vec<Block> {
    let mut blocks = Vec::new();

    for field in content_fields(spec) {
        if let Some(children) = field_children(element, field) {
            blocks.extend(collect_field_blocks(children));
        }
    }

    blocks
}

fn collect_field_blocks(nodes: &[HtmlNode]) -> Vec<Block> {
    let mut blocks = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Element(element) if is_field(element) => {
                blocks.extend(collect_item_blocks(&element.children));
            }
            _ => blocks.extend(collect_item_blocks(std::slice::from_ref(node))),
        }
    }

    blocks
}

fn collect_list_items(element: &HtmlElement, ordered: bool) -> Vec<ListItem> {
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
                .map(collect_item_blocks)
                .unwrap_or_default();
            let number = ordered
                .then(|| field_value(item, "number").filter(|value| value.as_str() != "auto"))
                .flatten();

            Some(ListItem { number, body })
        })
        .collect()
}

fn collect_table_rows(element: &HtmlElement, cell_kind: &str) -> Vec<TableRow> {
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
        collect_table_cells(child, cell_kind, &mut row);
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

fn collect_table_cells(node: &HtmlNode, cell_kind: &str, out: &mut Vec<TableCell>) {
    let HtmlNode::Element(element) = node else {
        return;
    };

    if attr(element, "data-typlite").as_deref() == Some(cell_kind) {
        out.push(TableCell {
            body: field_children(element, "body")
                .map(collect_inlines)
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
        collect_table_cells(child, cell_kind, out);
    }
}

fn collect_item_blocks(nodes: &[HtmlNode]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut inlines = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => inlines.push(Inline::Text(text.clone())),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                if let Some(block) = block_from_element(element) {
                    flush_paragraph(&mut inlines, &mut blocks);
                    blocks.push(block);
                } else {
                    inlines.extend(collect_inlines(std::slice::from_ref(node)));
                }
            }
            HtmlNode::Tag(_) | HtmlNode::Frame(_) => {}
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
        Inline::Emph(body)
        | Inline::Strong(body)
        | Inline::Strike(body)
        | Inline::Sub(body)
        | Inline::Super(body)
        | Inline::Math(body) => body.iter().any(inline_has_content),
        Inline::Link { dest, body } => !dest.is_empty() || body.iter().any(inline_has_content),
        Inline::Raw { text, .. } => !text.is_empty(),
        _ => inline
            .generated_body()
            .is_some_and(|body| body.iter().any(inline_has_content)),
    }
}

fn collect_inlines(nodes: &[HtmlNode]) -> Vec<Inline> {
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
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    )),
                    Some("strong") => out.push(Inline::Strong(
                        field_children(element, "body")
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    )),
                    Some("link") => out.push(Inline::Link {
                        dest: field_value(element, "dest").unwrap_or_default(),
                        body: field_children(element, "body")
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    }),
                    Some("strike") => out.push(Inline::Strike(
                        field_children(element, "body")
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    )),
                    Some("sub") => out.push(Inline::Sub(
                        field_children(element, "body")
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    )),
                    Some("super") => out.push(Inline::Super(
                        field_children(element, "body")
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    )),
                    Some("math-equation") => out.push(Inline::Math(
                        field_children(element, "body")
                            .map(collect_inlines)
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
                                    fields: Vec::new(),
                                    body: collect_inline_element_body(element, spec),
                                },
                            ) {
                                out.push(inline);
                            }
                        }
                    }
                    None => {
                        out.extend(collect_inlines(&element.children));
                    }
                }
            }
            HtmlNode::Tag(_) | HtmlNode::Frame(_) => {}
        }
    }

    out
}

fn collect_inline_element_body(element: &HtmlElement, spec: &'static ElementSpec) -> Vec<Inline> {
    for field in content_fields(spec) {
        if let Some(children) = field_children(element, field) {
            return collect_inlines(children);
        }
    }

    Vec::new()
}

fn content_fields(spec: &'static ElementSpec) -> impl Iterator<Item = &'static str> {
    spec.fields.iter().copied().filter(|field| {
        matches!(
            *field,
            "body" | "children" | "title" | "caption" | "term" | "description"
        )
    })
}

fn spec_by_kind(kind: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|spec| spec.kind.name() == kind)
}

fn collect_text(nodes: &[HtmlNode]) -> EcoString {
    let mut out = EcoString::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push_str(text),
            HtmlNode::Element(element) => {
                if !is_field(element) {
                    out.push_str(&collect_text(&element.children));
                }
            }
            HtmlNode::Tag(_) | HtmlNode::Frame(_) => {}
        }
    }

    out
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
    field_element(element, name).map(|field| collect_text(&field.children))
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
