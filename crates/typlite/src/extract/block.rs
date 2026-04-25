use typst::introspection::Introspector;
use typst_html::{HtmlElement, HtmlNode};

use crate::Result;
use crate::element_spec::{ElementKind, ElementMode, ElementSpec};
use crate::ir::*;

use super::{
    attr, block_field, bool_field, collect_inlines, collect_item_blocks, collect_list_items,
    collect_table_alignments, collect_table_rows, collect_term_items, field_bool, field_children,
    field_value, inline_field, math_field, raw_text, scalar_field, spec_by_kind, tag_name,
};

pub(super) fn block_from_element(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<Option<Block>> {
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
