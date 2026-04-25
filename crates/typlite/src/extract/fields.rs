use ecow::EcoString;
use serde_json::Value;
use tinymist_std::error::prelude::*;
use typst::introspection::Introspector;
use typst_html::{HtmlElement, HtmlFrame, HtmlNode};

use crate::Result;
use crate::element_spec::{ELEMENTS, ElementSpec};
use crate::ir::*;

use super::{collect_field_blocks, collect_inlines};

pub(super) fn scalar_field(element: &HtmlElement, name: &str) -> Option<EcoString> {
    field_value(element, name).filter(|value| !value.is_empty())
}

pub(super) fn source_field(element: &HtmlElement, name: &str) -> Option<EcoString> {
    field_children(element, name).map(|children| collect_text_without_frames(children).into())
}

pub(super) fn bool_field(element: &HtmlElement, name: &str) -> bool {
    matches!(field_value(element, name).as_deref(), Some("true"))
}

pub(super) fn source_span(element: &HtmlElement) -> Option<typst_syntax::Span> {
    (!element.span.is_detached()).then_some(element.span)
}

pub(super) fn inline_field(
    element: &HtmlElement,
    name: &str,
    introspector: &Introspector,
) -> Result<Vec<Inline>> {
    field_children(element, name)
        .map(|children| collect_inlines(children, introspector))
        .transpose()
        .map(Option::unwrap_or_default)
}

pub(super) fn block_field(
    element: &HtmlElement,
    name: &str,
    introspector: &Introspector,
) -> Result<Vec<Block>> {
    field_children(element, name)
        .map(|children| collect_field_blocks(children, introspector))
        .transpose()
        .map(Option::unwrap_or_default)
}

pub(super) fn frame_field(
    element: &HtmlElement,
    introspector: &Introspector,
) -> Result<Option<FrameImage>> {
    let Some(children) = field_children(element, "frame") else {
        return Ok(None);
    };
    Ok(match collect_inlines(children, introspector)?.as_slice() {
        [Inline::Frame(frame)] => Some(frame.image.clone()),
        _ => None,
    })
}

pub(super) fn spec_by_kind(kind: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|spec| spec.kind.name() == kind)
}

pub(super) fn frame_to_inline(frame: &HtmlFrame, introspector: &Introspector) -> Inline {
    Inline::Frame(FrameInline {
        image: FrameImage {
            svg: frame_to_svg(frame, introspector),
        },
    })
}

pub(super) fn frame_to_svg(frame: &HtmlFrame, introspector: &Introspector) -> EcoString {
    typst_svg::svg_html_frame(
        &frame.inner,
        frame.text_size,
        frame.id.as_deref(),
        &frame.link_points,
        introspector,
    )
    .into()
}

pub(super) fn math_field(element: &HtmlElement, name: &str) -> Result<MathNode> {
    let Some(raw) = field_value(element, name) else {
        bail!("missing math field `{name}`");
    };
    let value =
        serde_json::from_str::<Value>(&raw).context_ut("cannot parse math field as JSON")?;
    parse_math_node(&value)
}

pub(super) fn parse_math_node(value: &Value) -> Result<MathNode> {
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

pub(super) fn parse_math_value(value: &Value) -> Result<MathValue> {
    match value {
        Value::Null => Ok(MathValue::None),
        Value::Bool(value) => Ok(MathValue::Bool(*value)),
        Value::Number(value) => Ok(MathValue::Scalar(value.to_string().into())),
        Value::String(value) => Ok(MathValue::Scalar(value.as_str().into())),
        Value::Object(_) => Ok(MathValue::Node(Box::new(parse_math_node(value)?))),
        Value::Array(values) => parse_math_array(values),
    }
}

pub(super) fn parse_math_array(values: &[Value]) -> Result<MathValue> {
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

pub(super) fn tag_name(element: &HtmlElement) -> Option<String> {
    Some(element.tag.resolve().as_str().to_owned())
}

pub(super) fn attr(element: &HtmlElement, name: &str) -> Option<EcoString> {
    element
        .attrs
        .0
        .iter()
        .find(|(attr, _)| attr.resolve().as_str() == name)
        .map(|(_, value)| value.clone())
}

pub(super) fn field_value(element: &HtmlElement, name: &str) -> Option<EcoString> {
    field_element(element, name).map(|field| collect_text_without_frames(&field.children))
}

pub(super) fn collect_text_without_frames(nodes: &[HtmlNode]) -> EcoString {
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

pub(super) fn field_bool(element: &HtmlElement, name: &str) -> bool {
    field_value(element, name).is_some_and(|value| value.as_str() == "true")
}

pub(super) fn raw_text(element: &HtmlElement) -> EcoString {
    collect_raw_lines(element)
        .filter(|lines| !lines.is_empty())
        .map(|lines| lines.join("\n").into())
        .unwrap_or_else(|| field_value(element, "text").unwrap_or_default())
}

pub(super) fn collect_raw_lines(element: &HtmlElement) -> Option<Vec<EcoString>> {
    let children = field_children(element, "lines")?;
    let mut lines = Vec::new();
    for child in children {
        collect_raw_lines_from_node(child, &mut lines);
    }
    Some(lines)
}

pub(super) fn collect_raw_lines_from_node(node: &HtmlNode, out: &mut Vec<EcoString>) {
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

pub(super) fn field_children<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a [HtmlNode]> {
    field_element(element, name).map(|field| field.children.as_slice())
}

pub(super) fn field_element<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a HtmlElement> {
    element.children.iter().find_map(|child| {
        let HtmlNode::Element(child) = child else {
            return None;
        };

        (is_field(child) && attr(child, "name").as_deref() == Some(name)).then_some(child)
    })
}

pub(super) fn field_node(node: &HtmlNode) -> Option<&HtmlElement> {
    let HtmlNode::Element(element) = node else {
        return None;
    };

    is_field(element).then_some(element)
}

pub(super) fn is_field(element: &HtmlElement) -> bool {
    attr(element, "data-typlite-field").as_deref() == Some("true")
}
