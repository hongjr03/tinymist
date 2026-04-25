//! Experimental extraction from Typst HTML custom elements into typlite IR.

use ecow::EcoString;
use serde_json::Value;
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
        Some("nav") if attr(element, "role").as_deref() == Some("doc-toc") => {
            Some(Block::Block(BlockElementData {
                fields: Vec::new(),
                body: collect_outline_nav_blocks(element, introspector)?,
            }))
        }
        Some("typlite-heading") => Some({
            let level = field_value(element, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1);
            Block::Heading {
                id: attr(element, "id"),
                level,
                body: field_children(element, "body")
                    .map(|children| collect_inlines(children, introspector))
                    .transpose()?
                    .unwrap_or_default(),
            }
        }),
        Some("typlite-paragraph") => Some(Block::Paragraph(
            field_children(element, "body")
                .map(|children| collect_inlines(children, introspector))
                .transpose()?
                .unwrap_or_default(),
        )),
        Some("typlite-raw") => Some(Block::Raw {
            lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
            text: raw_text(element),
        }),
        Some("typlite-quote") => Some(Block::Quote(
            field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .transpose()?
                .unwrap_or_default(),
        )),
        Some("typlite-figure") => Some(Block::Figure {
            body: field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .transpose()?
                .unwrap_or_default(),
            caption: field_children(element, "caption")
                .map(|children| collect_inlines(children, introspector))
                .transpose()?
                .unwrap_or_default(),
            alt: field_value(element, "alt")
                .filter(|value| !value.is_empty() && value.as_str() != "none"),
        }),
        Some("typlite-align") => Some(Block::Align {
            alignment: field_value(element, "alignment"),
            body: field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .transpose()?
                .unwrap_or_default(),
        }),
        Some("typlite-math-equation") => Some(Block::Math(math_field(element, "body")?)),
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

fn collect_outline_nav_blocks(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();

    for child in &element.children {
        let HtmlNode::Element(child) = child else {
            continue;
        };

        if let Some(block) = block_from_element(child, introspector)? {
            blocks.push(block);
        } else if tag_name(child).as_deref() == Some("ol") {
            blocks.push(outline_list_from_ol(child, introspector)?);
        }
    }

    Ok(blocks)
}

fn outline_list_from_ol(element: &HtmlElement, introspector: &Introspector) -> Result<Block> {
    let mut items = Vec::new();

    for child in &element.children {
        let HtmlNode::Element(child) = child else {
            continue;
        };
        if tag_name(child).as_deref() == Some("li") {
            items.push(ListItem {
                number: None,
                body: collect_outline_li_blocks(child, introspector)?,
            });
        }
    }

    Ok(Block::List {
        ordered: false,
        tight: true,
        numbering: None,
        start: None,
        reversed: false,
        full: false,
        items,
    })
}

fn collect_outline_li_blocks(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    let mut run_start = 0usize;

    for (index, child) in element.children.iter().enumerate() {
        if let HtmlNode::Element(child) = child
            && tag_name(child).as_deref() == Some("ol")
        {
            if run_start < index {
                blocks.extend(collect_item_blocks(
                    &element.children[run_start..index],
                    introspector,
                )?);
            }
            blocks.push(outline_list_from_ol(child, introspector)?);
            run_start = index + 1;
        }
    }

    if run_start < element.children.len() {
        blocks.extend(collect_item_blocks(
            &element.children[run_start..],
            introspector,
        )?);
    }

    Ok(blocks)
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
        if let Some(children) = field_children(element, field) {
            blocks.extend(collect_field_blocks(children, introspector)?);
        }
    }

    Ok(blocks)
}

fn collect_field_blocks(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Block>> {
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

fn collect_list_items(
    element: &HtmlElement,
    ordered: bool,
    introspector: &Introspector,
) -> Result<Vec<ListItem>> {
    let Some(children) = field_children(element, "children") else {
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
    let columns = field_children(element, "columns")
        .map(|children| children.iter().filter_map(field_node).count())
        .filter(|columns| *columns > 0)
        .unwrap_or(1);
    let Some(children) = field_children(element, "children") else {
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
        blocks.push(Block::Paragraph(coalesce_raw_inlines(std::mem::take(
            inlines,
        ))));
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

                if tag_name(element).as_deref() == Some("a") {
                    let body = collect_link_body(element, introspector)?;
                    out.push(Inline::Link {
                        dest: attr(element, "href").unwrap_or_default(),
                        body,
                    });
                    continue;
                }

                match attr(element, "data-typlite").as_deref() {
                    Some("emph") => out.push(Inline::Emph(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    )),
                    Some("strong") => out.push(Inline::Strong(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    )),
                    Some("link") => out.push(Inline::Link {
                        dest: field_value(element, "dest").unwrap_or_default(),
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    }),
                    Some("strike") => out.push(Inline::Strike(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    )),
                    Some("sub") => out.push(Inline::Sub(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    )),
                    Some("super") => out.push(Inline::Super(
                        field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    )),
                    Some("math-equation") => out.push(Inline::Math(math_field(element, "body")?)),
                    Some("linebreak") => out.push(Inline::Linebreak),
                    Some("raw") => out.push(Inline::Raw {
                        lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
                        text: raw_text(element),
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

    Ok(coalesce_raw_inlines(out))
}

fn coalesce_raw_inlines(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut out = Vec::with_capacity(inlines.len());

    for inline in inlines {
        match (out.last_mut(), inline) {
            (
                Some(Inline::Raw {
                    lang: prev_lang,
                    text: prev_text,
                }),
                Inline::Raw { lang, text },
            ) if *prev_lang == lang => {
                prev_text.push_str(&text);
            }
            (_, inline) => out.push(inline),
        }
    }

    out
}

fn collect_link_body(element: &HtmlElement, introspector: &Introspector) -> Result<Vec<Inline>> {
    let body = collect_inlines(&element.children, introspector)?;
    if body.iter().any(inline_has_content) {
        return Ok(body);
    }

    let blocks = collect_item_blocks(&element.children, introspector)?;
    let text = plain_text_blocks(&blocks);
    if text.trim().is_empty() {
        Ok(body)
    } else {
        Ok(vec![Inline::Text(text.trim().into())])
    }
}

fn plain_text_blocks(blocks: &[Block]) -> EcoString {
    let mut out = EcoString::new();
    for block in blocks {
        match block {
            Block::Heading { body, .. } | Block::Paragraph(body) => {
                push_plain_text_inlines(body, &mut out)
            }
            Block::Quote(blocks) => out.push_str(&plain_text_blocks(blocks)),
            Block::Figure { body, caption, .. } => {
                out.push_str(&plain_text_blocks(body));
                push_plain_text_inlines(caption, &mut out);
            }
            Block::Align { body, .. } => out.push_str(&plain_text_blocks(body)),
            Block::Table { rows, .. } => {
                for row in rows {
                    for cell in &row.cells {
                        push_plain_text_inlines(&cell.body, &mut out);
                    }
                }
            }
            Block::List { items, .. } => {
                for item in items {
                    out.push_str(&plain_text_blocks(&item.body));
                }
            }
            Block::Terms { items } => {
                for item in items {
                    push_plain_text_inlines(&item.term, &mut out);
                    out.push_str(&plain_text_blocks(&item.description));
                }
            }
            _ => {
                if let Some(body) = block.generated_body() {
                    out.push_str(&plain_text_blocks(body));
                }
            }
        }
    }
    out
}

fn push_plain_text_inlines(inlines: &[Inline], out: &mut EcoString) {
    for inline in inlines {
        match inline {
            Inline::Text(text) | Inline::Raw { text, .. } => out.push_str(text),
            Inline::Linebreak | Inline::H(_) => out.push(' '),
            Inline::Emph(body)
            | Inline::Strong(body)
            | Inline::Strike(body)
            | Inline::Sub(body)
            | Inline::Super(body)
            | Inline::Link { body, .. } => push_plain_text_inlines(body, out),
            _ => {
                if let Some(body) = inline.generated_body() {
                    push_plain_text_inlines(body, out);
                }
            }
        }
    }
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

fn raw_text(element: &HtmlElement) -> EcoString {
    collect_raw_lines(element)
        .filter(|lines| !lines.is_empty())
        .map(|lines| lines.join("\n").into())
        .unwrap_or_else(|| field_value(element, "text").unwrap_or_default())
}

fn collect_raw_lines(element: &HtmlElement) -> Option<Vec<EcoString>> {
    let children = field_children(element, "lines")?;
    let mut lines = Vec::new();
    for child in children {
        collect_raw_lines_from_node(child, &mut lines);
    }
    Some(lines)
}

fn collect_raw_lines_from_node(node: &HtmlNode, out: &mut Vec<EcoString>) {
    let HtmlNode::Element(element) = node else {
        return;
    };

    if attr(element, "data-typlite").as_deref() == Some("raw-line") {
        out.push(field_value(element, "text").unwrap_or_default());
        return;
    }

    for child in &element.children {
        collect_raw_lines_from_node(child, out);
    }
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
