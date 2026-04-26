use crate::Result;
use crate::ir::*;
use tinymist_std::error::prelude::*;

use super::render_unimplemented;
use super::support::{
    math_bool, math_child, math_delim_pair, math_field, math_nodes, math_optional_child,
    math_optional_scalar, math_rows, math_scalar, math_style_command, matrix_env,
    push_latex_text_escaped,
};
use super::symbol::render_math_symbol;

pub(in crate::backend) fn render_math(node: &MathNode, out: &mut String) -> Result<()> {
    match node.func.as_str() {
        "sequence" => render_math_nodes(math_nodes(node, "children")?, out),
        "text" => {
            out.push_str(math_scalar(node, "text")?);
            Ok(())
        }
        "symbol" => {
            render_math_symbol(math_scalar(node, "text")?, out);
            Ok(())
        }
        "space" | "h" => {
            out.push(' ');
            Ok(())
        }
        "v" => Ok(()),
        "align-point" => Ok(()),
        "linebreak" => {
            out.push_str(r" \\ ");
            Ok(())
        }
        "accent" => render_math_accent(node, out),
        "attach" => render_math_attach(node, out),
        "binom" => render_math_two_arg_command(node, "binom", "upper", "lower", out),
        "cancel" => render_math_cancel(node, out),
        "cases" => render_math_cases(node, out),
        "class" | "lr" | "stretch" => render_math(math_child(node, "body")?, out),
        "limits" => render_math_limit_style(node, r"\limits", out),
        "mid" => render_math(math_child(node, "body")?, out),
        "scripts" => render_math(math_child(node, "body")?, out),
        "styled" => render_math_styled(node, out),
        "frac" => render_math_frac(node, out),
        "mat" => render_math_matrix(node, out),
        "op" => render_math_op(node, out),
        "overbrace" => render_math_annotated_command(node, "overbrace", "body", out),
        "overbracket" => render_math_annotated_command(node, "overbrace", "body", out),
        "overline" => render_math_one_arg_command(node, "overline", "body", out),
        "overparen" => render_math_annotated_command(node, "overbrace", "body", out),
        "overshell" => render_math_annotation_command(node, "overset", out),
        "primes" => render_math_primes(node, out),
        "root" => render_math_root(node, out),
        "underbrace" => render_math_under_annotated_command(node, "underbrace", out),
        "underbracket" => render_math_under_annotated_command(node, "underbrace", out),
        "underline" => render_math_one_arg_command(node, "underline", "body", out),
        "underparen" => render_math_under_annotated_command(node, "underbrace", out),
        "undershell" => render_math_annotation_command(node, "underset", out),
        "vec" => render_math_vec(node, out),
        _ => render_unimplemented(&format!("math.{}", node.func)),
    }
}

fn render_math_nodes(nodes: &[MathNode], out: &mut String) -> Result<()> {
    for node in nodes {
        render_math(node, out)?;
    }
    Ok(())
}

fn render_math_accent(node: &MathNode, out: &mut String) -> Result<()> {
    let command = match math_scalar(node, "accent")? {
        "\u{0300}" | "`" => "grave",
        "\u{0301}" | "'" => "acute",
        "\u{302}" => "hat",
        "\u{303}" => "tilde",
        "\u{304}" | "\u{305}" => "bar",
        "\u{033f}" => "overline",
        "\u{0306}" => "breve",
        "\u{307}" => "dot",
        "\u{308}" => "ddot",
        "\u{20db}" => "dddot",
        "\u{20dc}" => "ddddot",
        "\u{030a}" => "mathring",
        "\u{030b}" => "H",
        "\u{030c}" => "check",
        "\u{20d7}" | "\u{20d6}" | "\u{20e1}" => "vec",
        "\u{20d1}" | "\u{20d0}" => "vec",
        _ => return render_math_unknown_accent(node, out),
    };
    render_math_one_arg_command(node, command, "base", out)
}

fn render_math_unknown_accent(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\overset{");
    push_latex_text_escaped(math_scalar(node, "accent")?, out);
    out.push_str("}{");
    render_math(math_child(node, "base")?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_attach(node: &MathNode, out: &mut String) -> Result<()> {
    let mut rendered = String::new();
    if math_optional_child(node, "bl")?.is_some() || math_optional_child(node, "tl")?.is_some() {
        rendered.push_str("{}");
        render_math_script_pair(node, "bl", "tl", &mut rendered)?;
    }

    render_math(math_child(node, "base")?, &mut rendered)?;
    render_math_script_pair(node, "b", "t", &mut rendered)?;
    if math_optional_child(node, "br")?.is_some() || math_optional_child(node, "tr")?.is_some() {
        let base = std::mem::take(&mut rendered);
        rendered.push('{');
        rendered.push_str(&base);
        rendered.push('}');
        render_math_script_pair(node, "br", "tr", &mut rendered)?;
    }
    out.push_str(&rendered);
    Ok(())
}

fn render_math_script_pair(
    node: &MathNode,
    bottom_field: &str,
    top_field: &str,
    out: &mut String,
) -> Result<()> {
    if let Some(bottom) = math_optional_child(node, bottom_field)? {
        out.push_str("_{");
        render_math(bottom, out)?;
        out.push('}');
    }
    if let Some(top) = math_optional_child(node, top_field)? {
        out.push_str("^{");
        render_math(top, out)?;
        out.push('}');
    }
    Ok(())
}

fn render_math_cancel(node: &MathNode, out: &mut String) -> Result<()> {
    let command = if math_bool(node, "cross")? {
        "xcancel"
    } else if math_bool(node, "inverted")? {
        "bcancel"
    } else {
        "cancel"
    };
    render_math_one_arg_command(node, command, "body", out)
}

fn render_math_cases(node: &MathNode, out: &mut String) -> Result<()> {
    let (open, close) = math_delim_pair(node, "delim", Some("{"), Some("}"))?;
    let reverse = math_bool(node, "reverse")?;

    out.push_str(r"\left");
    out.push_str(if !reverse {
        open.as_deref().unwrap_or(".")
    } else {
        "."
    });
    out.push_str(r"\begin{array}{l}");
    for (index, child) in math_nodes(node, "children")?.iter().enumerate() {
        if index > 0 {
            out.push_str(r" \\ ");
        }
        render_math(child, out)?;
    }
    out.push_str(r"\end{array}\right");
    out.push_str(if reverse {
        close.as_deref().unwrap_or(".")
    } else {
        "."
    });
    Ok(())
}

fn render_math_matrix(node: &MathNode, out: &mut String) -> Result<()> {
    let (open, close) = math_delim_pair(node, "delim", Some("("), Some(")"))?;
    if let Some(env) = matrix_env(open.as_deref(), close.as_deref()) {
        out.push_str(r"\begin{");
        out.push_str(env);
        out.push('}');
        render_math_rows(math_rows(node, "rows")?, out)?;
        out.push_str(r"\end{");
        out.push_str(env);
        out.push('}');
    } else {
        render_math_delimited_matrix(
            open.as_deref(),
            close.as_deref(),
            math_rows(node, "rows")?,
            out,
        )?;
    }
    Ok(())
}

pub(in crate::backend::math) fn render_math_delimited_matrix(
    open: Option<&str>,
    close: Option<&str>,
    rows: &[Vec<MathNode>],
    out: &mut String,
) -> Result<()> {
    if open.is_some() || close.is_some() {
        out.push_str(r"\left");
        out.push_str(open.unwrap_or("."));
    }
    out.push_str(r"\begin{matrix}");
    render_math_rows(rows, out)?;
    out.push_str(r"\end{matrix}");
    if open.is_some() || close.is_some() {
        out.push_str(r"\right");
        out.push_str(close.unwrap_or("."));
    }
    Ok(())
}

pub(in crate::backend::math) fn render_math_rows(
    rows: &[Vec<MathNode>],
    out: &mut String,
) -> Result<()> {
    for (row_index, row) in rows.iter().enumerate() {
        if row_index > 0 {
            out.push_str(r" \\ ");
        }
        for (cell_index, cell) in row.iter().enumerate() {
            if cell_index > 0 {
                out.push_str(" & ");
            }
            render_math(cell, out)?;
        }
    }
    Ok(())
}

fn render_math_op(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\operatorname{");
    render_math(math_child(node, "text")?, out)?;
    out.push('}');
    if math_bool(node, "limits")? {
        out.push_str(r"\limits");
    }
    Ok(())
}

fn render_math_primes(node: &MathNode, out: &mut String) -> Result<()> {
    let count = math_scalar(node, "count")?
        .parse::<usize>()
        .context_ut("math.primes count must be a number")?;
    for _ in 0..count {
        out.push('\'');
    }
    Ok(())
}

fn render_math_root(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\sqrt");
    if let Some(index) = math_optional_child(node, "index")? {
        out.push('[');
        render_math(index, out)?;
        out.push(']');
    }
    out.push('{');
    render_math(math_child(node, "radicand")?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_frac(node: &MathNode, out: &mut String) -> Result<()> {
    match math_optional_scalar(node, "style")? {
        Some("skewed") | Some("horizontal") => {
            out.push('{');
            render_math_value(math_field(node, "num")?, out)?;
            out.push_str("}/{");
            render_math_value(math_field(node, "denom")?, out)?;
            out.push('}');
            Ok(())
        }
        Some("vertical") | None => render_math_two_arg_command(node, "frac", "num", "denom", out),
        Some(_) => render_math_two_arg_command(node, "frac", "num", "denom", out),
    }
}

fn render_math_limit_style(node: &MathNode, suffix: &str, out: &mut String) -> Result<()> {
    render_math(math_child(node, "body")?, out)?;
    out.push_str(suffix);
    Ok(())
}

fn render_math_styled(node: &MathNode, out: &mut String) -> Result<()> {
    if let Some(size) = math_optional_scalar(node, "size")? {
        let command = match size {
            "display" => r"\displaystyle ",
            "text" => r"\textstyle ",
            "script" => r"\scriptstyle ",
            "script-script" => r"\scriptscriptstyle ",
            _ => "",
        };
        out.push_str(command);
    }

    let variant = math_optional_scalar(node, "variant")?;
    let italic = math_optional_scalar(node, "italic")?;
    let bold = math_bool(node, "bold")?;

    if let Some(command) = math_style_command(variant, italic, bold) {
        out.push('\\');
        out.push_str(command);
        out.push('{');
        render_math(math_child(node, "child")?, out)?;
        out.push('}');
    } else {
        render_math(math_child(node, "child")?, out)?;
    }

    Ok(())
}

fn render_math_vec(node: &MathNode, out: &mut String) -> Result<()> {
    let (open, close) = math_delim_pair(node, "delim", Some("("), Some(")"))?;
    let rows = math_nodes(node, "children")?
        .iter()
        .cloned()
        .map(|node| vec![node])
        .collect::<Vec<_>>();
    render_math_delimited_matrix(open.as_deref(), close.as_deref(), &rows, out)
}

fn render_math_one_arg_command(
    node: &MathNode,
    command: &str,
    field: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math(math_child(node, field)?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_two_arg_command(
    node: &MathNode,
    command: &str,
    first: &str,
    second: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math_value(math_field(node, first)?, out)?;
    out.push_str("}{");
    render_math_value(math_field(node, second)?, out)?;
    out.push('}');
    Ok(())
}

fn render_math_annotated_command(
    node: &MathNode,
    command: &str,
    body: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math(math_child(node, body)?, out)?;
    out.push('}');
    if let Some(annotation) = math_optional_child(node, "annotation")? {
        out.push_str("^{");
        render_math(annotation, out)?;
        out.push('}');
    }
    Ok(())
}

fn render_math_under_annotated_command(
    node: &MathNode,
    command: &str,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math(math_child(node, "body")?, out)?;
    out.push('}');
    if let Some(annotation) = math_optional_child(node, "annotation")? {
        out.push_str("_{");
        render_math(annotation, out)?;
        out.push('}');
    }
    Ok(())
}

fn render_math_annotation_command(node: &MathNode, command: &str, out: &mut String) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math(math_child(node, "annotation")?, out)?;
    out.push_str("}{");
    render_math(math_child(node, "body")?, out)?;
    out.push('}');
    Ok(())
}

pub(in crate::backend::math) fn render_math_value(
    value: &MathValue,
    out: &mut String,
) -> Result<()> {
    match value {
        MathValue::None => Ok(()),
        MathValue::Bool(value) => {
            out.push_str(if *value { "true" } else { "false" });
            Ok(())
        }
        MathValue::Scalar(value) => {
            out.push_str(value);
            Ok(())
        }
        MathValue::Node(node) => render_math(node, out),
        MathValue::Nodes(nodes) => render_math_nodes(nodes, out),
        MathValue::Rows(rows) => {
            out.push_str(r"\begin{matrix}");
            render_math_rows(rows, out)?;
            out.push_str(r"\end{matrix}");
            Ok(())
        }
    }
}
