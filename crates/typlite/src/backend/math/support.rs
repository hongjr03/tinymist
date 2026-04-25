use crate::Result;
use crate::ir::*;
use tinymist_std::error::prelude::*;

use super::super::is_auto_or_none;

pub(super) fn parse_math_json_value(value: &serde_json::Value) -> Result<MathValue> {
    match value {
        serde_json::Value::Null => Ok(MathValue::None),
        serde_json::Value::Bool(value) => Ok(MathValue::Bool(*value)),
        serde_json::Value::Number(value) => Ok(MathValue::Scalar(value.to_string().into())),
        serde_json::Value::String(value) => Ok(MathValue::Scalar(value.as_str().into())),
        serde_json::Value::Object(_) => Ok(MathValue::Node(Box::new(parse_math_json_node(value)?))),
        serde_json::Value::Array(values) => parse_math_json_array(values),
    }
}

pub(super) fn parse_math_json_node(value: &serde_json::Value) -> Result<MathNode> {
    let Some(object) = value.as_object() else {
        bail!("math node must be encoded as an object, got {value}");
    };
    let func = object
        .get("func")
        .and_then(serde_json::Value::as_str)
        .context("math node is missing string field `func`")?;
    let mut fields = Vec::new();
    for (name, value) in object {
        if name == "func" {
            continue;
        }
        fields.push(MathField {
            name: name.as_str().into(),
            value: parse_math_json_value(value)?,
        });
    }
    Ok(MathNode {
        func: func.into(),
        fields,
    })
}

pub(super) fn parse_math_json_array(values: &[serde_json::Value]) -> Result<MathValue> {
    if values.is_empty() {
        return Ok(MathValue::Nodes(Vec::new()));
    }

    if values.iter().all(serde_json::Value::is_object) {
        return values
            .iter()
            .map(parse_math_json_node)
            .collect::<Result<Vec<_>>>()
            .map(MathValue::Nodes);
    }

    if values.iter().all(serde_json::Value::is_array) {
        let mut rows = Vec::new();
        for row in values {
            let Some(row) = row.as_array() else {
                unreachable!("checked by all(Value::is_array)");
            };
            rows.push(
                row.iter()
                    .map(parse_math_json_node)
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        return Ok(MathValue::Rows(rows));
    }

    Ok(MathValue::Scalar(
        serde_json::Value::Array(values.to_vec()).to_string().into(),
    ))
}

pub(super) fn push_latex_text_escaped(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str(r"\backslash{}"),
            '{' => out.push_str(r"\{"),
            '}' => out.push_str(r"\}"),
            '_' => out.push_str(r"\_"),
            '^' => out.push_str(r"\^{}"),
            '&' => out.push_str(r"\&"),
            '%' => out.push_str(r"\%"),
            '$' => out.push_str(r"\$"),
            '#' => out.push_str(r"\#"),
            _ => out.push(ch),
        }
    }
}

pub(super) fn math_style_command(
    variant: Option<&str>,
    italic: Option<&str>,
    bold: bool,
) -> Option<&'static str> {
    if bold {
        return Some("mathbf");
    }

    match variant {
        Some("plain") => Some("mathrm"),
        Some("sans-serif") => Some("mathsf"),
        Some("chancery") => Some("mathcal"),
        Some("roundhand") => Some("mathscr"),
        Some("fraktur") => Some("mathfrak"),
        Some("monospace") => Some("mathtt"),
        Some("double-struck") => Some("mathbb"),
        _ => match italic {
            Some("false") => Some("mathrm"),
            Some("true") => Some("mathit"),
            _ => None,
        },
    }
}

pub(super) fn matrix_env(open: Option<&str>, close: Option<&str>) -> Option<&'static str> {
    match (open, close) {
        (Some("("), Some(")")) => Some("pmatrix"),
        (Some("["), Some("]")) => Some("bmatrix"),
        (Some(r"\{"), Some(r"\}")) => Some("Bmatrix"),
        (Some("|"), Some("|")) => Some("vmatrix"),
        (Some(r"\|"), Some(r"\|")) => Some("Vmatrix"),
        (None, None) => Some("matrix"),
        _ => None,
    }
}

pub(super) fn math_delim_pair(
    node: &MathNode,
    field: &str,
    default_open: Option<&str>,
    default_close: Option<&str>,
) -> Result<(Option<String>, Option<String>)> {
    let Some(value) = math_optional_field(node, field) else {
        return Ok((
            default_open.map(math_delim).map(str::to_owned),
            default_close.map(math_delim).map(str::to_owned),
        ));
    };

    let MathValue::Scalar(value) = value else {
        return Ok((
            default_open.map(math_delim).map(str::to_owned),
            default_close.map(math_delim).map(str::to_owned),
        ));
    };

    if value == "none" {
        return Ok((None, None));
    }

    if let Ok(serde_json::Value::Array(values)) = serde_json::from_str::<serde_json::Value>(value) {
        let open = values
            .first()
            .and_then(json_delim)
            .map(math_delim)
            .map(str::to_owned);
        let close = values
            .get(1)
            .and_then(json_delim)
            .map(math_delim)
            .map(str::to_owned);
        return Ok((open, close));
    }

    let open = math_delim(value);
    Ok((
        Some(open.to_owned()),
        Some(math_matching_delim(open).to_owned()),
    ))
}

pub(super) fn inline_delim_pair(
    value: Option<&str>,
    default_open: Option<&str>,
    default_close: Option<&str>,
) -> (Option<String>, Option<String>) {
    let Some(value) = value.filter(|value| !is_auto_or_none(value)) else {
        return (
            default_open.map(math_delim).map(str::to_owned),
            default_close.map(math_delim).map(str::to_owned),
        );
    };

    if value == "none" {
        return (None, None);
    }

    if let Ok(serde_json::Value::Array(values)) = serde_json::from_str::<serde_json::Value>(value) {
        return (
            values
                .first()
                .and_then(json_delim)
                .map(math_delim)
                .map(str::to_owned),
            values
                .get(1)
                .and_then(json_delim)
                .map(math_delim)
                .map(str::to_owned),
        );
    }

    let open = math_delim(value);
    (
        Some(open.to_owned()),
        Some(math_matching_delim(open).to_owned()),
    )
}

pub(super) fn json_delim(value: &serde_json::Value) -> Option<&str> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::String(value) => Some(value),
        _ => None,
    }
}

pub(super) fn math_delim(value: &str) -> &str {
    match value {
        r"{" => r"\{",
        r"}" => r"\}",
        "‖" => r"\|",
        _ => value,
    }
}

pub(super) fn math_matching_delim(open: &str) -> &str {
    match open {
        "(" => ")",
        "[" => "]",
        r"\{" => r"\}",
        r"\}" => r"\{",
        ")" => "(",
        "]" => "[",
        "|" => "|",
        r"\|" => r"\|",
        _ => open,
    }
}

pub(super) fn math_field<'a>(node: &'a MathNode, name: &str) -> Result<&'a MathValue> {
    let Some(value) = node
        .fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
    else {
        let fields = node
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        bail!(
            "math.{} is missing field `{name}`; available fields: {fields}",
            node.func
        );
    };
    Ok(value)
}

pub(super) fn math_optional_field<'a>(node: &'a MathNode, name: &str) -> Option<&'a MathValue> {
    node.fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
}

pub(super) fn math_child<'a>(node: &'a MathNode, name: &str) -> Result<&'a MathNode> {
    match math_field(node, name)? {
        MathValue::Node(child) => Ok(child),
        _ => bail!("math.{} field `{name}` must be a node", node.func),
    }
}

pub(super) fn math_optional_child<'a>(
    node: &'a MathNode,
    name: &str,
) -> Result<Option<&'a MathNode>> {
    match node.fields.iter().find(|field| field.name == name) {
        Some(field) => match &field.value {
            MathValue::Node(child) => Ok(Some(child)),
            MathValue::None => Ok(None),
            _ => bail!("math.{} field `{name}` must be a node", node.func),
        },
        None => Ok(None),
    }
}

pub(super) fn math_nodes<'a>(node: &'a MathNode, name: &str) -> Result<&'a [MathNode]> {
    match math_field(node, name)? {
        MathValue::Nodes(nodes) => Ok(nodes),
        _ => bail!("math.{} field `{name}` must be a node list", node.func),
    }
}

pub(super) fn math_rows<'a>(node: &'a MathNode, name: &str) -> Result<&'a [Vec<MathNode>]> {
    match math_field(node, name)? {
        MathValue::Rows(rows) => Ok(rows),
        _ => bail!("math.{} field `{name}` must be a row list", node.func),
    }
}

pub(super) fn math_scalar<'a>(node: &'a MathNode, name: &str) -> Result<&'a str> {
    match math_field(node, name)? {
        MathValue::Scalar(value) => Ok(value),
        _ => bail!("math.{} field `{name}` must be a scalar", node.func),
    }
}

pub(super) fn math_optional_scalar<'a>(node: &'a MathNode, name: &str) -> Result<Option<&'a str>> {
    match math_optional_field(node, name) {
        Some(MathValue::Scalar(value)) => Ok(Some(value)),
        Some(MathValue::None) | None => Ok(None),
        _ => bail!("math.{} field `{name}` must be a scalar", node.func),
    }
}

pub(super) fn math_bool(node: &MathNode, name: &str) -> Result<bool> {
    match math_optional_field(node, name) {
        Some(MathValue::Bool(value)) => Ok(*value),
        Some(MathValue::Scalar(value)) => match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => bail!("math.{} field `{name}` must be a bool", node.func),
        },
        Some(MathValue::None) | None => Ok(false),
        _ => bail!("math.{} field `{name}` must be a bool", node.func),
    }
}
