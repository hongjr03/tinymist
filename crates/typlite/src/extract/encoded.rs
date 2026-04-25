//! Conversion from Rust-encoded Typst JSON payloads into typlite IR.

use ecow::EcoString;
use serde_json::{Map, Value};
use tinymist_std::error::prelude::*;
use typst::introspection::Introspector;

use crate::Result;
use crate::element_spec::{ELEMENTS, ElementSpec};
use crate::ir::{
    Block, BlockElementData, ElementField, ElementFieldValue, Inline, InlineElementData, ListItem,
    MathField, MathNode, MathValue, TableAlign, TableCell, TableRow, TermItem,
    block_from_element_kind, inline_from_element_kind,
};

use super::{
    FieldMode, content_fields, flush_paragraph, inline_has_content, is_content_field_name,
    table_alignment,
};

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

pub(super) fn list_items_from_array(
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
            .map(|body| content_blocks(body, introspector))
            .transpose()?
            .unwrap_or_default();
        let number = ordered
            .then(|| {
                item.get("number")
                    .map(scalar)
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

pub(super) fn term_items_from_array(
    children: &[Value],
    introspector: &Introspector,
) -> Result<Vec<TermItem>> {
    let mut items = Vec::new();
    for item in children {
        let Some(item) = item.as_object() else {
            continue;
        };
        let term = item
            .get("term")
            .map(|term| content_inlines(term, introspector))
            .transpose()?
            .unwrap_or_default();
        let description = item
            .get("description")
            .map(|description| content_blocks(description, introspector))
            .transpose()?
            .unwrap_or_default();
        items.push(TermItem { term, description });
    }
    Ok(items)
}

pub(super) fn table_alignments(value: &Value) -> Vec<TableAlign> {
    match value {
        Value::Array(values) => values
            .iter()
            .map(scalar)
            .map(|value| table_alignment(&value))
            .collect(),
        _ => {
            let alignment = table_alignment(&scalar(value));
            if alignment == TableAlign::Default {
                Vec::new()
            } else {
                vec![alignment]
            }
        }
    }
}

pub(super) fn collect_table_cells(
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
                .map(|body| content_inlines(body, introspector))
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
                    collect_table_cells(value, cell_kind, introspector, out)?;
                }
            }
            Value::Object(_) => collect_table_cells(value, cell_kind, introspector, out)?,
            _ => {}
        }
    }

    Ok(())
}

pub(super) fn content_inlines(value: &Value, introspector: &Introspector) -> Result<Vec<Inline>> {
    if let Value::Array(values) = value {
        let mut out = Vec::new();
        for value in values {
            out.extend(content_inlines(value, introspector)?);
        }
        return Ok(out);
    }

    let Some(object) = value.as_object() else {
        return Ok(vec![Inline::Text(scalar(value))]);
    };
    let func = json_func(object)?;

    Ok(match func {
        "sequence" => {
            let mut out = Vec::new();
            for child in json_array(object, "children") {
                out.extend(content_inlines(child, introspector)?);
            }
            out
        }
        "styled" => object
            .get("child")
            .map(|child| content_inlines(child, introspector))
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
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "strong" => vec![Inline::Strong(
            object
                .get("body")
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "link" => vec![Inline::Link {
            dest: json_scalar(object, "dest").unwrap_or_default(),
            body: object
                .get("body")
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        }],
        "strike" => vec![Inline::Strike(
            object
                .get("body")
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "sub" => vec![Inline::Sub(
            object
                .get("body")
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "super" => vec![Inline::Super(
            object
                .get("body")
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "equation" => vec![Inline::Math(math_node(
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
                    fields: element_fields(object, spec, FieldMode::Inline, introspector)?,
                    body: inline_element_body(object, spec, introspector)?,
                },
            ) else {
                bail!("encoded element `{kind}` is not an inline element");
            };
            vec![inline]
        }
    })
}

pub(super) fn content_blocks(value: &Value, introspector: &Introspector) -> Result<Vec<Block>> {
    if let Value::Array(values) = value {
        let mut out = Vec::new();
        for value in values {
            out.extend(content_blocks(value, introspector)?);
        }
        return Ok(out);
    }

    let Some(object) = value.as_object() else {
        let text = scalar(value);
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
                let child_blocks = content_blocks(child, introspector)?;
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
            .map(|child| content_blocks(child, introspector))
            .transpose()?
            .unwrap_or_default(),
        "tag" | "frame" => Vec::new(),
        "text" | "symbol" | "space" | "h" | "linebreak" | "equation" | "elem" => {
            let inlines = content_inlines(value, introspector)?;
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
                let inlines = content_inlines(value, introspector)?;
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
            .map(|body| content_inlines(body, introspector))
            .transpose()
            .map(Option::unwrap_or_default);
    };

    let Some(body) = object.get("body") else {
        return Ok(Vec::new());
    };
    let raw = content_inlines(body, introspector)?
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
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "strong" => vec![Inline::Strong(
            object
                .get("body")
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )],
        "raw" => vec![Inline::Raw {
            lang: json_scalar(object, "lang").filter(|lang| lang.as_str() != "none"),
            text: json_scalar(object, "text").unwrap_or_default(),
        }],
        "math-equation" => vec![Inline::Math(math_node(
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
                    fields: element_fields(object, spec, FieldMode::Inline, introspector)?,
                    body: inline_element_body(object, spec, introspector)?,
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
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        }),
        "par" | "paragraph" => Some(Block::Paragraph(
            object
                .get("body")
                .map(|body| content_inlines(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )),
        "math.equation" | "math-equation" | "equation" => Some(Block::Math(math_node(
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
                .map(|body| content_blocks(body, introspector))
                .transpose()?
                .unwrap_or_default(),
        )),
        "figure" => Some(Block::Figure {
            body: object
                .get("body")
                .map(|body| content_blocks(body, introspector))
                .transpose()?
                .unwrap_or_default(),
            caption: object
                .get("caption")
                .map(|caption| content_inlines(caption, introspector))
                .transpose()?
                .unwrap_or_default(),
            alt: json_scalar(object, "alt")
                .filter(|value| !value.is_empty() && value.as_str() != "none"),
        }),
        "align" => Some(Block::Align {
            alignment: json_scalar(object, "alignment"),
            body: object
                .get("body")
                .map(|body| content_blocks(body, introspector))
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
            }
            if !row.is_empty() {
                rows.push(TableRow { cells: row });
            }
            Some(Block::Table {
                rows,
                alignments: object
                    .get("align")
                    .map(table_alignments)
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
                    fields: element_fields(object, spec, FieldMode::Block, introspector)?,
                    body: block_element_body(object, spec, introspector)?,
                },
            )
        }
    })
}

fn element_fields(
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
                value: element_field_value(name, value, mode, introspector)?,
            });
        }
    }
    Ok(fields)
}

pub(super) fn element_field_value(
    name: &str,
    value: &Value,
    mode: FieldMode,
    introspector: &Introspector,
) -> Result<ElementFieldValue> {
    if name == "element" {
        return Ok(ElementFieldValue::Blocks(content_blocks(
            value,
            introspector,
        )?));
    }

    if is_content_field_name(name) {
        Ok(match mode {
            FieldMode::Block => ElementFieldValue::Blocks(content_blocks(value, introspector)?),
            FieldMode::Inline => ElementFieldValue::Inlines(content_inlines(value, introspector)?),
        })
    } else if matches!(name, "source" | "sources") {
        Ok(ElementFieldValue::Scalar(source_scalar(value)))
    } else {
        Ok(ElementFieldValue::Scalar(scalar(value)))
    }
}

fn block_element_body(
    object: &Map<String, Value>,
    spec: &'static ElementSpec,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();
    for field in content_fields(spec) {
        if let Some(value) = object.get(field) {
            blocks.extend(content_blocks(value, introspector)?);
        }
    }
    Ok(blocks)
}

pub(super) fn inline_element_body(
    object: &Map<String, Value>,
    spec: &'static ElementSpec,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    if let Some(value) = object.get("body") {
        return content_inlines(value, introspector);
    }

    for field in content_fields(spec) {
        if field == "body" {
            continue;
        }
        if let Some(value) = object.get(field) {
            return content_inlines(value, introspector);
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
    object.get(name).map(scalar)
}

fn spec_by_selector_or_kind(kind: &str) -> Option<&'static ElementSpec> {
    ELEMENTS
        .iter()
        .find(|spec| spec.selector == kind || spec.kind.name() == kind)
}

pub(super) fn math_node(value: &Value) -> Result<MathNode> {
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
            value: math_value(value).with_context_ut("cannot parse math field", || {
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

fn math_value(value: &Value) -> Result<MathValue> {
    match value {
        Value::Null => Ok(MathValue::None),
        Value::Bool(value) => Ok(MathValue::Bool(*value)),
        Value::Number(value) => Ok(MathValue::Scalar(value.to_string().into())),
        Value::String(value) => Ok(MathValue::Scalar(value.as_str().into())),
        Value::Object(_) => Ok(MathValue::Node(Box::new(math_node(value)?))),
        Value::Array(values) => math_array(values),
    }
}

fn math_array(values: &[Value]) -> Result<MathValue> {
    if values.is_empty() {
        return Ok(MathValue::Nodes(Vec::new()));
    }

    if values.iter().all(Value::is_object) {
        let mut nodes = Vec::new();
        for value in values {
            nodes.push(math_node(value)?);
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
                cells.push(math_node(cell)?);
            }
            rows.push(cells);
        }
        return Ok(MathValue::Rows(rows));
    }

    Ok(MathValue::Scalar(
        Value::Array(values.to_vec()).to_string().into(),
    ))
}

pub(super) fn scalar(value: &Value) -> EcoString {
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

fn source_scalar(value: &Value) -> EcoString {
    source_value(value).to_string().into()
}

fn source_value(value: &Value) -> Value {
    match value {
        Value::String(value) => {
            serde_json::json!({ "kind": "string", "value": value })
        }
        Value::Array(values) => Value::Array(values.iter().map(source_value).collect()),
        Value::Object(_) => value.clone(),
        Value::Null | Value::Bool(_) | Value::Number(_) => value.clone(),
    }
}
