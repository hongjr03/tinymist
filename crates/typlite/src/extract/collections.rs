use typst::introspection::Introspector;
use typst_html::{HtmlElement, HtmlNode};

use crate::Result;
use crate::ir::*;

use super::{
    attr, collect_inlines, collect_item_blocks, collect_text_without_frames, field_children,
    field_node, field_value, inline_field, scalar_field,
};

pub(super) fn collect_list_items(
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

pub(super) fn collect_term_items(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<Vec<TermItem>> {
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

pub(super) fn collect_table_rows(
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

pub(super) fn collect_table_alignments(element: &HtmlElement) -> Vec<TableAlign> {
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

pub(super) fn table_alignment(value: &str) -> TableAlign {
    match value.trim() {
        "left" | "start" => TableAlign::Left,
        "center" | "horizon" => TableAlign::Center,
        "right" | "end" => TableAlign::Right,
        _ => TableAlign::Default,
    }
}

pub(super) fn collect_table_cells(
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

pub(super) fn table_cell_inline(
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

pub(super) fn grid_cell_inline(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<GridCellInline> {
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

pub(super) fn table_hline_inline(element: &HtmlElement) -> TableHlineInline {
    TableHlineInline {
        y: scalar_field(element, "y"),
        start: scalar_field(element, "start"),
        end: scalar_field(element, "end"),
        stroke: scalar_field(element, "stroke"),
        position: scalar_field(element, "position"),
    }
}

pub(super) fn grid_hline_inline(element: &HtmlElement) -> GridHlineInline {
    let line = table_hline_inline(element);
    GridHlineInline {
        y: line.y,
        start: line.start,
        end: line.end,
        stroke: line.stroke,
        position: line.position,
    }
}

pub(super) fn table_vline_inline(element: &HtmlElement) -> TableVlineInline {
    TableVlineInline {
        x: scalar_field(element, "x"),
        start: scalar_field(element, "start"),
        end: scalar_field(element, "end"),
        stroke: scalar_field(element, "stroke"),
        position: scalar_field(element, "position"),
    }
}

pub(super) fn grid_vline_inline(element: &HtmlElement) -> GridVlineInline {
    let line = table_vline_inline(element);
    GridVlineInline {
        x: line.x,
        start: line.start,
        end: line.end,
        stroke: line.stroke,
        position: line.position,
    }
}
