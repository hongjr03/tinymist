use crate::Result;
use crate::ir::*;
use tinymist_std::error::prelude::*;

use super::super::is_auto_or_none;
use super::node::{render_math, render_math_delimited_matrix, render_math_rows, render_math_value};
use super::support::{
    inline_delim_pair, matrix_env, parse_math_json_value, push_latex_text_escaped,
};
use super::symbol::render_math_symbol;
use super::{render_math_inline_body, render_unimplemented_inline};
pub(super) fn render_math_inline_expr(value: Option<&str>, out: &mut String) -> Result<()> {
    let Some(value) = value.filter(|value| !is_auto_or_none(value)) else {
        return Ok(());
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
        return render_math_value(&parse_math_json_value(&json)?, out);
    }

    render_math_symbol(value, out);
    Ok(())
}

pub(super) fn render_math_inline_one_arg(
    command: &str,
    value: Option<&str>,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math_inline_expr(value, out)?;
    out.push('}');
    Ok(())
}

pub(super) fn render_math_inline_accent(data: &MathAccentInline, out: &mut String) -> Result<()> {
    let command = match data.accent.as_deref().unwrap_or("") {
        "\u{0300}" | "`" => "grave",
        "\u{0301}" | "'" => "acute",
        "\u{302}" | "hat" => "hat",
        "\u{303}" | "tilde" => "tilde",
        "\u{304}" | "\u{305}" | "macron" | "dash" => "bar",
        "\u{033f}" => "overline",
        "\u{0306}" | "breve" => "breve",
        "\u{307}" | "dot" => "dot",
        "\u{308}" => "ddot",
        "\u{20db}" => "dddot",
        "\u{20dc}" => "ddddot",
        "\u{030a}" | "circle" => "mathring",
        "\u{030b}" => "H",
        "\u{030c}" | "caron" => "check",
        "\u{20d7}" | "\u{20d6}" | "\u{20e1}" | "\u{20d1}" | "\u{20d0}" | "arrow" => "vec",
        _ => {
            out.push_str(r"\overset{");
            push_latex_text_escaped(data.accent.as_deref().unwrap_or(""), out);
            out.push_str("}{");
            render_math_inline_expr(data.base.as_deref(), out)?;
            out.push('}');
            return Ok(());
        }
    };
    render_math_inline_one_arg(command, data.base.as_deref(), out)
}

pub(super) fn render_math_inline_attach(data: &MathAttachInline, out: &mut String) -> Result<()> {
    if data.bl.is_some() || data.tl.is_some() {
        out.push_str("{}");
        render_math_inline_script_pair(data.bl.as_deref(), data.tl.as_deref(), out)?;
    }
    render_math_inline_expr(data.base.as_deref(), out)?;
    render_math_inline_script_pair(data.b.as_deref(), data.t.as_deref(), out)?;
    if data.br.is_some() || data.tr.is_some() {
        render_math_inline_script_pair(data.br.as_deref(), data.tr.as_deref(), out)?;
    }
    Ok(())
}

pub(super) fn render_math_inline_script_pair(
    bottom: Option<&str>,
    top: Option<&str>,
    out: &mut String,
) -> Result<()> {
    if bottom.is_some() {
        out.push_str("_{");
        render_math_inline_expr(bottom, out)?;
        out.push('}');
    }
    if top.is_some() {
        out.push_str("^{");
        render_math_inline_expr(top, out)?;
        out.push('}');
    }
    Ok(())
}

pub(super) fn render_math_inline_binom(data: &MathBinomInline, out: &mut String) -> Result<()> {
    out.push_str(r"\binom{");
    render_math_inline_expr(data.upper.as_deref(), out)?;
    out.push_str("}{");
    render_math_inline_expr(data.lower.as_deref(), out)?;
    out.push('}');
    Ok(())
}

pub(super) fn render_math_inline_cancel(data: &MathCancelInline, out: &mut String) -> Result<()> {
    let command = if data.cross {
        "xcancel"
    } else if data.inverted {
        "bcancel"
    } else {
        "cancel"
    };
    render_math_inline_one_arg(command, data.body.as_deref(), out)
}

pub(super) fn render_math_inline_cases(data: &MathCasesInline, out: &mut String) -> Result<()> {
    let (open, close) = inline_delim_pair(data.delim.as_deref(), Some("{"), Some("}"));
    out.push_str(r"\left");
    out.push_str(if !data.reverse {
        open.as_deref().unwrap_or(".")
    } else {
        "."
    });
    out.push_str(r"\begin{array}{l}");
    for (index, child) in data.children.iter().enumerate() {
        if index > 0 {
            out.push_str(r" \\ ");
        }
        render_inline_node_as_math(child, out)?;
    }
    out.push_str(r"\end{array}\right");
    out.push_str(if data.reverse {
        close.as_deref().unwrap_or(".")
    } else {
        "."
    });
    Ok(())
}

pub(super) fn render_math_inline_frac(data: &MathFracInline, out: &mut String) -> Result<()> {
    match data.style.as_deref() {
        Some("skewed") | Some("horizontal") => {
            out.push('{');
            render_math_inline_expr(data.num.as_deref(), out)?;
            out.push_str("}/{");
            render_math_inline_expr(data.denom.as_deref(), out)?;
            out.push('}');
            Ok(())
        }
        _ => {
            out.push_str(r"\frac{");
            render_math_inline_expr(data.num.as_deref(), out)?;
            out.push_str("}{");
            render_math_inline_expr(data.denom.as_deref(), out)?;
            out.push('}');
            Ok(())
        }
    }
}

pub(super) fn render_math_inline_limits(data: &MathLimitsInline, out: &mut String) -> Result<()> {
    render_math_inline_expr(data.body.as_deref(), out)?;
    if data.inline {
        out.push_str(r"\nolimits");
    } else {
        out.push_str(r"\limits");
    }
    Ok(())
}

pub(super) fn render_math_inline_matrix(data: &MathMatInline, out: &mut String) -> Result<()> {
    let (open, close) = inline_delim_pair(data.delim.as_deref(), Some("("), Some(")"));
    let env = matrix_env(open.as_deref(), close.as_deref()).unwrap_or("matrix");
    out.push_str(r"\begin{");
    out.push_str(env);
    out.push('}');
    if let Some(rows) = data.rows.as_deref() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(rows)
            && let MathValue::Rows(rows) = parse_math_json_value(&json)?
        {
            render_math_rows(&rows, out)?;
        } else {
            push_latex_text_escaped(rows, out);
        }
    }
    out.push_str(r"\end{");
    out.push_str(env);
    out.push('}');
    Ok(())
}

pub(super) fn render_math_inline_op(data: &MathOpInline, out: &mut String) -> Result<()> {
    out.push_str(r"\operatorname{");
    render_math_inline_expr(data.text.as_deref(), out)?;
    out.push('}');
    if data.limits {
        out.push_str(r"\limits");
    }
    Ok(())
}

pub(super) fn render_math_inline_annotated(
    command: &str,
    body: Option<&str>,
    annotation: Option<&str>,
    script: char,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math_inline_expr(body, out)?;
    out.push('}');
    if annotation.is_some() {
        out.push(script);
        out.push('{');
        render_math_inline_expr(annotation, out)?;
        out.push('}');
    }
    Ok(())
}

pub(super) fn render_math_inline_annotation(
    command: &str,
    body: Option<&str>,
    annotation: Option<&str>,
    out: &mut String,
) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math_inline_expr(annotation, out)?;
    out.push_str("}{");
    render_math_inline_expr(body, out)?;
    out.push('}');
    Ok(())
}

pub(super) fn render_math_inline_primes(data: &MathPrimesInline, out: &mut String) -> Result<()> {
    let count = data
        .count
        .as_deref()
        .unwrap_or("1")
        .parse::<usize>()
        .context_ut("math.primes count must be a number")?;
    for _ in 0..count {
        out.push('\'');
    }
    Ok(())
}

pub(super) fn render_math_inline_root(data: &MathRootInline, out: &mut String) -> Result<()> {
    out.push_str(r"\sqrt");
    if data.index.is_some() {
        out.push('[');
        render_math_inline_expr(data.index.as_deref(), out)?;
        out.push(']');
    }
    out.push('{');
    render_math_inline_expr(data.radicand.as_deref(), out)?;
    out.push('}');
    Ok(())
}

pub(super) fn render_math_inline_vec(data: &MathVecInline, out: &mut String) -> Result<()> {
    let (open, close) = inline_delim_pair(data.delim.as_deref(), Some("("), Some(")"));
    render_math_delimited_matrix(
        open.as_deref(),
        close.as_deref(),
        &data
            .children
            .iter()
            .map(|child| {
                let mut rendered = String::new();
                render_inline_node_as_math(child, &mut rendered)?;
                Ok(vec![MathNode {
                    func: "text".into(),
                    fields: vec![MathField {
                        name: "text".into(),
                        value: MathValue::Scalar(rendered.into()),
                    }],
                }])
            })
            .collect::<Result<Vec<_>>>()?,
        out,
    )
}

pub(super) fn render_inline_node_as_math(node: &Inline, out: &mut String) -> Result<()> {
    match node {
        Inline::Text(data) => {
            render_math_symbol(&data.text, out);
            Ok(())
        }
        Inline::Raw(data) => {
            render_math_symbol(&data.text, out);
            Ok(())
        }
        Inline::Math(data) => render_math(&data.body, out),
        Inline::MathAccent(_)
        | Inline::MathAttach(_)
        | Inline::MathBinom(_)
        | Inline::MathCancel(_)
        | Inline::MathCases(_)
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
        | Inline::MathVec(_) => render_math_inline_body(node, out),
        _ => render_unimplemented_inline(node),
    }
}
