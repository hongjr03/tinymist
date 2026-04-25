//! Experimental extraction from Typst HTML custom elements into typlite IR.

use ecow::EcoString;
use serde_json::Value;
use tinymist_std::error::prelude::*;
use typst::introspection::Introspector;
use typst_html::{HtmlDocument, HtmlElement, HtmlFrame, HtmlNode};

use crate::Result;
use crate::element_spec::{ELEMENTS, ElementKind, ElementMode, ElementSpec};
use crate::ir::*;

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
            Some(Block::Block(BlockBlock {
                body: collect_outline_nav_blocks(element, introspector)?,
                ..Default::default()
            }))
        }
        Some("typlite-heading") => Some({
            let level = field_value(element, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1);
            Block::Heading(HeadingBlock {
                id: attr(element, "id"),
                level,
                body: field_children(element, "body")
                    .map(|children| collect_inlines(children, introspector))
                    .transpose()?
                    .unwrap_or_default(),
            })
        }),
        Some("typlite-paragraph") => Some(Block::Paragraph(ParagraphBlock {
            body: field_children(element, "body")
                .map(|children| collect_inlines(children, introspector))
                .transpose()?
                .unwrap_or_default(),
        })),
        Some("typlite-raw") => Some(Block::Raw(RawBlock {
            lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
            text: raw_text(element),
        })),
        Some("typlite-quote") => Some(Block::Quote(QuoteBlock {
            body: field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .transpose()?
                .unwrap_or_default(),
        })),
        Some("typlite-figure") => Some(Block::Figure(FigureBlock {
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
        })),
        Some("typlite-align") => Some(Block::Align(AlignBlock {
            alignment: field_value(element, "alignment"),
            body: field_children(element, "body")
                .map(|children| collect_item_blocks(children, introspector))
                .transpose()?
                .unwrap_or_default(),
        })),
        Some("typlite-math-equation") => Some(Block::Math(MathBlock {
            body: math_field(element, "body")?,
        })),
        Some("typlite-table") => Some(Block::Table(TableBlock {
            rows: collect_table_rows(element, "table-cell", introspector)?,
            alignments: collect_table_alignments(element),
        })),
        Some("typlite-grid") => Some(Block::Table(TableBlock {
            rows: collect_table_rows(element, "grid-cell", introspector)?,
            alignments: collect_table_alignments(element),
        })),
        Some("typlite-list") => Some(Block::List(ListBlock {
            ordered: false,
            tight: field_bool(element, "tight"),
            numbering: None,
            start: None,
            reversed: false,
            full: false,
            items: collect_list_items(element, false, introspector)?,
        })),
        Some("typlite-enum") => Some(Block::List(ListBlock {
            ordered: true,
            tight: field_bool(element, "tight"),
            numbering: field_value(element, "numbering").filter(|value| value.as_str() != "none"),
            start: field_value(element, "start")
                .filter(|value| value.as_str() != "auto")
                .and_then(|value| value.parse::<i64>().ok()),
            reversed: field_bool(element, "reversed"),
            full: field_bool(element, "full"),
            items: collect_list_items(element, true, introspector)?,
        })),
        Some("typlite-terms") => Some(Block::Terms(TermsBlock {
            items: collect_term_items(element, introspector)?,
        })),
        Some(tag) => match block_spec_from_tag(&tag) {
            Some(spec) => block_from_element_kind(element, spec.kind, introspector)?,
            None => None,
        },
        None => None,
    })
}

fn block_from_element_kind(
    element: &HtmlElement,
    kind: ElementKind,
    introspector: &Introspector,
) -> Result<Option<Block>> {
    let block = match kind {
        ElementKind::Bibliography => Block::Bibliography(BibliographyBlock {
            sources: scalar_field(element, "sources"),
            title: inline_field(element, "title", introspector)?,
            full: bool_field(element, "full"),
            style: scalar_field(element, "style"),
        }),
        ElementKind::Block => Block::Block(BlockBlock {
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            breakable: scalar_field(element, "breakable"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            radius: scalar_field(element, "radius"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            spacing: scalar_field(element, "spacing"),
            above: scalar_field(element, "above"),
            below: scalar_field(element, "below"),
            clip: scalar_field(element, "clip"),
            sticky: scalar_field(element, "sticky"),
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::Colbreak => Block::Colbreak(ColbreakBlock {
            weak: bool_field(element, "weak"),
        }),
        ElementKind::Columns => Block::Columns(ColumnsBlock {
            count: scalar_field(element, "count"),
            gutter: scalar_field(element, "gutter"),
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::Move => Block::Move(MoveBlock {
            dx: scalar_field(element, "dx"),
            dy: scalar_field(element, "dy"),
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::Outline => Block::Outline(OutlineBlock {
            title: inline_field(element, "title", introspector)?,
            target: scalar_field(element, "target"),
            depth: scalar_field(element, "depth"),
            indent: scalar_field(element, "indent"),
        }),
        ElementKind::Pad => Block::Pad(PadBlock {
            left: scalar_field(element, "left"),
            top: scalar_field(element, "top"),
            right: scalar_field(element, "right"),
            bottom: scalar_field(element, "bottom"),
            x: scalar_field(element, "x"),
            y: scalar_field(element, "y"),
            rest: scalar_field(element, "rest"),
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::Pagebreak => Block::Pagebreak(PagebreakBlock {
            weak: bool_field(element, "weak"),
            to: scalar_field(element, "to"),
        }),
        ElementKind::Parbreak => Block::Parbreak(ParbreakBlock {}),
        ElementKind::Rotate => Block::Rotate(RotateBlock {
            angle: scalar_field(element, "angle"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::Scale => Block::Scale(ScaleBlock {
            factor: scalar_field(element, "factor"),
            x: scalar_field(element, "x"),
            y: scalar_field(element, "y"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::Skew => Block::Skew(SkewBlock {
            ax: scalar_field(element, "ax"),
            ay: scalar_field(element, "ay"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::Stack => Block::Stack(StackBlock {
            dir: scalar_field(element, "dir"),
            spacing: scalar_field(element, "spacing"),
            children: block_field(element, "children", introspector)?,
        }),
        ElementKind::Title => Block::Title(TitleBlock {
            body: block_field(element, "body", introspector)?,
        }),
        ElementKind::V => Block::V(VBlock {
            amount: scalar_field(element, "amount"),
            weak: bool_field(element, "weak"),
        }),
        _ => return Ok(None),
    };
    Ok(Some(block))
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

    Ok(Block::List(ListBlock {
        ordered: false,
        tight: true,
        numbering: None,
        start: None,
        reversed: false,
        full: false,
        items,
    }))
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

fn inline_from_element_kind(
    element: &HtmlElement,
    kind: ElementKind,
    introspector: &Introspector,
) -> Result<Option<Inline>> {
    let inline = match kind {
        ElementKind::Box => Inline::Box(BoxInline {
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            baseline: scalar_field(element, "baseline"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            radius: scalar_field(element, "radius"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            clip: scalar_field(element, "clip"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Circle => Inline::Circle(CircleInline {
            radius: scalar_field(element, "radius"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Cite => Inline::Cite(CiteInline {
            key: scalar_field(element, "key"),
            supplement: inline_field(element, "supplement", introspector)?,
            form: scalar_field(element, "form"),
            style: scalar_field(element, "style"),
        }),
        ElementKind::Curve => Inline::Curve(CurveInline {
            fill: scalar_field(element, "fill"),
            fill_rule: scalar_field(element, "fill-rule"),
            stroke: scalar_field(element, "stroke"),
            components: inline_field(element, "components", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::CurveClose => Inline::CurveClose(CurveCloseInline {
            mode: scalar_field(element, "mode"),
        }),
        ElementKind::CurveCubic => Inline::CurveCubic(CurveCubicInline {
            control_start: scalar_field(element, "control-start"),
            control_end: scalar_field(element, "control-end"),
            end: scalar_field(element, "end"),
            relative: bool_field(element, "relative"),
        }),
        ElementKind::CurveLine => Inline::CurveLine(CurveLineInline {
            end: scalar_field(element, "end"),
            relative: bool_field(element, "relative"),
        }),
        ElementKind::CurveMove => Inline::CurveMove(CurveMoveInline {
            start: scalar_field(element, "start"),
            relative: bool_field(element, "relative"),
        }),
        ElementKind::CurveQuad => Inline::CurveQuad(CurveQuadInline {
            control: scalar_field(element, "control"),
            end: scalar_field(element, "end"),
            relative: bool_field(element, "relative"),
        }),
        ElementKind::Document => Inline::Document(DocumentInline {
            title: scalar_field(element, "title"),
            author: scalar_field(element, "author"),
            description: scalar_field(element, "description"),
            keywords: scalar_field(element, "keywords"),
            date: scalar_field(element, "date"),
        }),
        ElementKind::Ellipse => Inline::Ellipse(EllipseInline {
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::FigureCaption => Inline::FigureCaption(FigureCaptionInline {
            position: scalar_field(element, "position"),
            separator: scalar_field(element, "separator"),
            body: inline_field(element, "body", introspector)?,
            kind: scalar_field(element, "kind"),
            supplement: inline_field(element, "supplement", introspector)?,
            numbering: scalar_field(element, "numbering"),
            counter: scalar_field(element, "counter"),
        }),
        ElementKind::Footnote => Inline::Footnote(FootnoteInline {
            numbering: scalar_field(element, "numbering"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::FootnoteEntry => Inline::FootnoteEntry(FootnoteEntryInline {
            note: inline_field(element, "note", introspector)?,
            separator: inline_field(element, "separator", introspector)?,
            clearance: scalar_field(element, "clearance"),
            gap: scalar_field(element, "gap"),
            indent: scalar_field(element, "indent"),
        }),
        ElementKind::GridCell => Inline::GridCell(grid_cell_inline(element, introspector)?),
        ElementKind::GridFooter => Inline::GridFooter(GridFooterInline {
            repeat: bool_field(element, "repeat"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::GridHeader => Inline::GridHeader(GridHeaderInline {
            repeat: bool_field(element, "repeat"),
            level: scalar_field(element, "level"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::GridHline => Inline::GridHline(grid_hline_inline(element)),
        ElementKind::GridVline => Inline::GridVline(grid_vline_inline(element)),
        ElementKind::H => Inline::H(HInline {
            amount: scalar_field(element, "amount"),
            weak: bool_field(element, "weak"),
        }),
        ElementKind::Hide => Inline::Hide(HideInline {
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Highlight => Inline::Highlight(HighlightInline {
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            top_edge: scalar_field(element, "top-edge"),
            bottom_edge: scalar_field(element, "bottom-edge"),
            extent: scalar_field(element, "extent"),
            radius: scalar_field(element, "radius"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Image => Inline::Image(ImageInline {
            source: source_field(element, "source"),
            format: scalar_field(element, "format"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            alt: scalar_field(element, "alt"),
            page: scalar_field(element, "page"),
            fit: scalar_field(element, "fit"),
            scaling: scalar_field(element, "scaling"),
            icc: scalar_field(element, "icc"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Line => Inline::Line(LineInline {
            start: scalar_field(element, "start"),
            end: scalar_field(element, "end"),
            length: scalar_field(element, "length"),
            angle: scalar_field(element, "angle"),
            stroke: scalar_field(element, "stroke"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Metadata => Inline::Metadata(MetadataInline {
            value: scalar_field(element, "value"),
        }),
        ElementKind::Move => Inline::Move(MoveInline {
            dx: scalar_field(element, "dx"),
            dy: scalar_field(element, "dy"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::OutlineEntry => Inline::OutlineEntry(OutlineEntryInline {
            level: scalar_field(element, "level"),
            element: block_field(element, "element", introspector)?,
            fill: scalar_field(element, "fill"),
        }),
        ElementKind::Overline => Inline::Overline(OverlineInline {
            stroke: scalar_field(element, "stroke"),
            offset: scalar_field(element, "offset"),
            extent: scalar_field(element, "extent"),
            evade: scalar_field(element, "evade"),
            background: scalar_field(element, "background"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Pad => Inline::Pad(PadInline {
            left: scalar_field(element, "left"),
            top: scalar_field(element, "top"),
            right: scalar_field(element, "right"),
            bottom: scalar_field(element, "bottom"),
            x: scalar_field(element, "x"),
            y: scalar_field(element, "y"),
            rest: scalar_field(element, "rest"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Path => Inline::Path(PathInline {
            fill: scalar_field(element, "fill"),
            fill_rule: scalar_field(element, "fill-rule"),
            stroke: scalar_field(element, "stroke"),
            closed: bool_field(element, "closed"),
            vertices: scalar_field(element, "vertices"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::PdfArtifact => Inline::PdfArtifact(PdfArtifactInline {
            kind: scalar_field(element, "kind"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::PdfAttach => Inline::PdfAttach(PdfAttachInline {
            path: scalar_field(element, "path"),
            data: scalar_field(element, "data"),
            relationship: scalar_field(element, "relationship"),
            mime_type: scalar_field(element, "mime-type"),
            description: scalar_field(element, "description"),
        }),
        ElementKind::PdfEmbed => Inline::PdfEmbed(PdfEmbedInline {
            path: scalar_field(element, "path"),
            data: scalar_field(element, "data"),
            relationship: scalar_field(element, "relationship"),
            mime_type: scalar_field(element, "mime-type"),
            description: scalar_field(element, "description"),
        }),
        ElementKind::Place => Inline::Place(PlaceInline {
            alignment: scalar_field(element, "alignment"),
            scope: scalar_field(element, "scope"),
            float: scalar_field(element, "float"),
            clearance: scalar_field(element, "clearance"),
            dx: scalar_field(element, "dx"),
            dy: scalar_field(element, "dy"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::PlaceFlush => Inline::PlaceFlush(PlaceFlushInline {}),
        ElementKind::Polygon => Inline::Polygon(PolygonInline {
            fill: scalar_field(element, "fill"),
            fill_rule: scalar_field(element, "fill-rule"),
            stroke: scalar_field(element, "stroke"),
            vertices: scalar_field(element, "vertices"),
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Quote => Inline::Quote(QuoteInline {
            block: bool_field(element, "block"),
            quotes: scalar_field(element, "quotes"),
            attribution: inline_field(element, "attribution", introspector)?,
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::RawLine => Inline::RawLine(RawLineInline {
            number: scalar_field(element, "number"),
            count: scalar_field(element, "count"),
            text: scalar_field(element, "text"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Rect => Inline::Rect(RectInline {
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            radius: scalar_field(element, "radius"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::Ref => Inline::Ref(RefInline {
            target: scalar_field(element, "target"),
            supplement: inline_field(element, "supplement", introspector)?,
            form: scalar_field(element, "form"),
            citation: inline_field(element, "citation", introspector)?,
            element: block_field(element, "element", introspector)?,
        }),
        ElementKind::Repeat => Inline::Repeat(RepeatInline {
            body: inline_field(element, "body", introspector)?,
            gap: scalar_field(element, "gap"),
            justify: scalar_field(element, "justify"),
        }),
        ElementKind::Rotate => Inline::Rotate(RotateInline {
            angle: scalar_field(element, "angle"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Scale => Inline::Scale(ScaleInline {
            factor: scalar_field(element, "factor"),
            x: scalar_field(element, "x"),
            y: scalar_field(element, "y"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Skew => Inline::Skew(SkewInline {
            ax: scalar_field(element, "ax"),
            ay: scalar_field(element, "ay"),
            origin: scalar_field(element, "origin"),
            reflow: bool_field(element, "reflow"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Smallcaps => Inline::Smallcaps(SmallcapsInline {
            all: bool_field(element, "all"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Smartquote => Inline::Smartquote(SmartquoteInline {
            double: scalar_field(element, "double"),
            enabled: scalar_field(element, "enabled"),
            alternative: scalar_field(element, "alternative"),
            quotes: scalar_field(element, "quotes"),
        }),
        ElementKind::Square => Inline::Square(SquareInline {
            size: scalar_field(element, "size"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            fill: scalar_field(element, "fill"),
            stroke: scalar_field(element, "stroke"),
            radius: scalar_field(element, "radius"),
            inset: scalar_field(element, "inset"),
            outset: scalar_field(element, "outset"),
            body: inline_field(element, "body", introspector)?,
            frame: frame_field(element, introspector)?,
        }),
        ElementKind::TableCell => Inline::TableCell(table_cell_inline(element, introspector)?),
        ElementKind::TableFooter => Inline::TableFooter(TableFooterInline {
            repeat: bool_field(element, "repeat"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::TableHeader => Inline::TableHeader(TableHeaderInline {
            repeat: bool_field(element, "repeat"),
            level: scalar_field(element, "level"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::TableHline => Inline::TableHline(table_hline_inline(element)),
        ElementKind::TableVline => Inline::TableVline(table_vline_inline(element)),
        ElementKind::Underline => Inline::Underline(UnderlineInline {
            stroke: scalar_field(element, "stroke"),
            offset: scalar_field(element, "offset"),
            extent: scalar_field(element, "extent"),
            evade: scalar_field(element, "evade"),
            background: scalar_field(element, "background"),
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::Page
        | ElementKind::ParLine
        | ElementKind::MathAccent
        | ElementKind::MathAttach
        | ElementKind::MathBinom
        | ElementKind::MathCancel
        | ElementKind::MathCases
        | ElementKind::MathClass
        | ElementKind::MathFrac
        | ElementKind::MathLimits
        | ElementKind::MathLr
        | ElementKind::MathMat
        | ElementKind::MathMid
        | ElementKind::MathOp
        | ElementKind::MathOverbrace
        | ElementKind::MathOverbracket
        | ElementKind::MathOverline
        | ElementKind::MathOverparen
        | ElementKind::MathOvershell
        | ElementKind::MathPrimes
        | ElementKind::MathRoot
        | ElementKind::MathScripts
        | ElementKind::MathStretch
        | ElementKind::MathUnderbrace
        | ElementKind::MathUnderbracket
        | ElementKind::MathUnderline
        | ElementKind::MathUnderparen
        | ElementKind::MathUndershell
        | ElementKind::MathVec => inline_from_element_kind_tail(element, kind, introspector)?,
        _ => return Ok(None),
    };
    Ok(Some(inline))
}

fn inline_from_element_kind_tail(
    element: &HtmlElement,
    kind: ElementKind,
    introspector: &Introspector,
) -> Result<Inline> {
    Ok(match kind {
        ElementKind::Page => Inline::Page(PageInline {
            paper: scalar_field(element, "paper"),
            width: scalar_field(element, "width"),
            height: scalar_field(element, "height"),
            flipped: bool_field(element, "flipped"),
            margin: scalar_field(element, "margin"),
            binding: scalar_field(element, "binding"),
            columns: scalar_field(element, "columns"),
            fill: scalar_field(element, "fill"),
            numbering: scalar_field(element, "numbering"),
            supplement: inline_field(element, "supplement", introspector)?,
            number_align: scalar_field(element, "number-align"),
            header: inline_field(element, "header", introspector)?,
            header_ascent: scalar_field(element, "header-ascent"),
            footer: inline_field(element, "footer", introspector)?,
            footer_descent: scalar_field(element, "footer-descent"),
            background: inline_field(element, "background", introspector)?,
            foreground: inline_field(element, "foreground", introspector)?,
            body: inline_field(element, "body", introspector)?,
        }),
        ElementKind::ParLine => Inline::ParLine(ParLineInline {
            numbering: scalar_field(element, "numbering"),
            number_align: scalar_field(element, "number-align"),
            number_margin: scalar_field(element, "number-margin"),
            number_clearance: scalar_field(element, "number-clearance"),
            numbering_scope: scalar_field(element, "numbering-scope"),
        }),
        ElementKind::MathAccent => Inline::MathAccent(MathAccentInline {
            base: scalar_field(element, "base"),
            accent: scalar_field(element, "accent"),
            size: scalar_field(element, "size"),
            dotless: bool_field(element, "dotless"),
        }),
        ElementKind::MathAttach => Inline::MathAttach(MathAttachInline {
            base: scalar_field(element, "base"),
            t: scalar_field(element, "t"),
            b: scalar_field(element, "b"),
            tl: scalar_field(element, "tl"),
            bl: scalar_field(element, "bl"),
            tr: scalar_field(element, "tr"),
            br: scalar_field(element, "br"),
        }),
        ElementKind::MathBinom => Inline::MathBinom(MathBinomInline {
            upper: scalar_field(element, "upper"),
            lower: scalar_field(element, "lower"),
        }),
        ElementKind::MathCancel => Inline::MathCancel(MathCancelInline {
            body: scalar_field(element, "body"),
            length: scalar_field(element, "length"),
            inverted: bool_field(element, "inverted"),
            cross: bool_field(element, "cross"),
            angle: scalar_field(element, "angle"),
            stroke: scalar_field(element, "stroke"),
        }),
        ElementKind::MathCases => Inline::MathCases(MathCasesInline {
            delim: scalar_field(element, "delim"),
            reverse: bool_field(element, "reverse"),
            gap: scalar_field(element, "gap"),
            children: inline_field(element, "children", introspector)?,
        }),
        ElementKind::MathClass => Inline::MathClass(MathClassInline {
            class: scalar_field(element, "class"),
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathFrac => Inline::MathFrac(MathFracInline {
            num: scalar_field(element, "num"),
            denom: scalar_field(element, "denom"),
            style: scalar_field(element, "style"),
        }),
        ElementKind::MathLimits => Inline::MathLimits(MathLimitsInline {
            body: scalar_field(element, "body"),
            inline: bool_field(element, "inline"),
        }),
        ElementKind::MathLr => Inline::MathLr(MathLrInline {
            size: scalar_field(element, "size"),
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathMat => Inline::MathMat(MathMatInline {
            delim: scalar_field(element, "delim"),
            align: scalar_field(element, "align"),
            augment: scalar_field(element, "augment"),
            gap: scalar_field(element, "gap"),
            row_gap: scalar_field(element, "row-gap"),
            column_gap: scalar_field(element, "column-gap"),
            rows: scalar_field(element, "rows"),
        }),
        ElementKind::MathMid => Inline::MathMid(MathMidInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathOp => Inline::MathOp(MathOpInline {
            text: scalar_field(element, "text"),
            limits: bool_field(element, "limits"),
        }),
        ElementKind::MathOverbrace => Inline::MathOverbrace(MathOverbraceInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathOverbracket => Inline::MathOverbracket(MathOverbracketInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathOverline => Inline::MathOverline(MathOverlineInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathOverparen => Inline::MathOverparen(MathOverparenInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathOvershell => Inline::MathOvershell(MathOvershellInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathPrimes => Inline::MathPrimes(MathPrimesInline {
            count: scalar_field(element, "count"),
        }),
        ElementKind::MathRoot => Inline::MathRoot(MathRootInline {
            index: scalar_field(element, "index"),
            radicand: scalar_field(element, "radicand"),
        }),
        ElementKind::MathScripts => Inline::MathScripts(MathScriptsInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathStretch => Inline::MathStretch(MathStretchInline {
            body: scalar_field(element, "body"),
            size: scalar_field(element, "size"),
        }),
        ElementKind::MathUnderbrace => Inline::MathUnderbrace(MathUnderbraceInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathUnderbracket => Inline::MathUnderbracket(MathUnderbracketInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathUnderline => Inline::MathUnderline(MathUnderlineInline {
            body: scalar_field(element, "body"),
        }),
        ElementKind::MathUnderparen => Inline::MathUnderparen(MathUnderparenInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathUndershell => Inline::MathUndershell(MathUndershellInline {
            body: scalar_field(element, "body"),
            annotation: scalar_field(element, "annotation"),
        }),
        ElementKind::MathVec => Inline::MathVec(MathVecInline {
            delim: scalar_field(element, "delim"),
            align: scalar_field(element, "align"),
            gap: scalar_field(element, "gap"),
            children: inline_field(element, "children", introspector)?,
        }),
        _ => unreachable!("tail only receives covered inline element kinds"),
    })
}

fn table_cell_inline(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<TableCellInline> {
    Ok(TableCellInline {
        body: inline_field(element, "body", introspector)?,
        x: scalar_field(element, "x"),
        y: scalar_field(element, "y"),
        colspan: scalar_field(element, "colspan"),
        rowspan: scalar_field(element, "rowspan"),
        inset: scalar_field(element, "inset"),
        align: scalar_field(element, "align"),
        fill: scalar_field(element, "fill"),
        stroke: scalar_field(element, "stroke"),
        breakable: scalar_field(element, "breakable"),
    })
}

fn grid_cell_inline(element: &HtmlElement, introspector: &Introspector) -> Result<GridCellInline> {
    let cell = table_cell_inline(element, introspector)?;
    Ok(GridCellInline {
        body: cell.body,
        x: cell.x,
        y: cell.y,
        colspan: cell.colspan,
        rowspan: cell.rowspan,
        inset: cell.inset,
        align: cell.align,
        fill: cell.fill,
        stroke: cell.stroke,
        breakable: cell.breakable,
    })
}

fn table_hline_inline(element: &HtmlElement) -> TableHlineInline {
    TableHlineInline {
        y: scalar_field(element, "y"),
        start: scalar_field(element, "start"),
        end: scalar_field(element, "end"),
        stroke: scalar_field(element, "stroke"),
        position: scalar_field(element, "position"),
    }
}

fn grid_hline_inline(element: &HtmlElement) -> GridHlineInline {
    let line = table_hline_inline(element);
    GridHlineInline {
        y: line.y,
        start: line.start,
        end: line.end,
        stroke: line.stroke,
        position: line.position,
    }
}

fn table_vline_inline(element: &HtmlElement) -> TableVlineInline {
    TableVlineInline {
        x: scalar_field(element, "x"),
        start: scalar_field(element, "start"),
        end: scalar_field(element, "end"),
        stroke: scalar_field(element, "stroke"),
        position: scalar_field(element, "position"),
    }
}

fn grid_vline_inline(element: &HtmlElement) -> GridVlineInline {
    let line = table_vline_inline(element);
    GridVlineInline {
        x: line.x,
        start: line.start,
        end: line.end,
        stroke: line.stroke,
        position: line.position,
    }
}

fn coalesce_raw_inlines(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut out = Vec::with_capacity(inlines.len());

    for inline in inlines {
        match (out.last_mut(), inline) {
            (Some(Inline::Raw(prev)), Inline::Raw(raw)) if prev.lang == raw.lang => {
                prev.text.push_str(&raw.text);
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
        Ok(vec![Inline::Text(TextInline {
            text: text.trim().into(),
        })])
    }
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

fn scalar_field(element: &HtmlElement, name: &str) -> Option<EcoString> {
    field_value(element, name).filter(|value| !value.is_empty())
}

fn source_field(element: &HtmlElement, name: &str) -> Option<EcoString> {
    field_children(element, name).map(|children| collect_text_without_frames(children).into())
}

fn bool_field(element: &HtmlElement, name: &str) -> bool {
    matches!(field_value(element, name).as_deref(), Some("true"))
}

fn inline_field(
    element: &HtmlElement,
    name: &str,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    field_children(element, name)
        .map(|children| collect_inlines(children, introspector))
        .transpose()
        .map(Option::unwrap_or_default)
}

fn block_field(
    element: &HtmlElement,
    name: &str,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    field_children(element, name)
        .map(|children| collect_field_blocks(children, introspector))
        .transpose()
        .map(Option::unwrap_or_default)
}

fn frame_field(element: &HtmlElement, introspector: &Introspector) -> Result<Option<FrameImage>> {
    let Some(children) = field_children(element, "frame") else {
        return Ok(None);
    };
    Ok(match collect_inlines(children, introspector)?.as_slice() {
        [Inline::Frame(frame)] => Some(frame.image.clone()),
        _ => None,
    })
}

fn spec_by_kind(kind: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|spec| spec.kind.name() == kind)
}

fn frame_to_inline(frame: &HtmlFrame, introspector: &Introspector) -> Inline {
    Inline::Frame(FrameInline {
        image: FrameImage {
            svg: frame_to_svg(frame, introspector),
        },
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
