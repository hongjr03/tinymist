use crate::Result;
use crate::ir::*;
use tinymist_std::error::prelude::*;

use super::{is_auto_or_none, render_unimplemented, render_unimplemented_inline};

pub(super) fn render_math_inline(node: &Inline, inline: bool, out: &mut String) -> Result<()> {
    if inline {
        out.push('$');
    } else {
        out.push_str("$$");
    }

    render_math_inline_body(node, out)?;

    if inline {
        out.push('$');
    } else {
        out.push_str("$$");
    }
    Ok(())
}

fn render_math_inline_body(node: &Inline, out: &mut String) -> Result<()> {
    match node {
        Inline::MathAccent(data) => render_math_inline_accent(data, out)?,
        Inline::MathAttach(data) => render_math_inline_attach(data, out)?,
        Inline::MathBinom(data) => render_math_inline_binom(data, out)?,
        Inline::MathCancel(data) => render_math_inline_cancel(data, out)?,
        Inline::MathCases(data) => render_math_inline_cases(data, out)?,
        Inline::MathClass(data) => render_math_inline_expr(data.body.as_deref(), out)?,
        Inline::MathFrac(data) => render_math_inline_frac(data, out)?,
        Inline::MathLimits(data) => render_math_inline_limits(data, out)?,
        Inline::MathLr(data) => render_math_inline_expr(data.body.as_deref(), out)?,
        Inline::MathMat(data) => render_math_inline_matrix(data, out)?,
        Inline::MathMid(data) => render_math_inline_expr(data.body.as_deref(), out)?,
        Inline::MathOp(data) => render_math_inline_op(data, out)?,
        Inline::MathOverbrace(data) => render_math_inline_annotated(
            "overbrace",
            data.body.as_deref(),
            data.annotation.as_deref(),
            '^',
            out,
        )?,
        Inline::MathOverbracket(data) => render_math_inline_annotated(
            "overbrace",
            data.body.as_deref(),
            data.annotation.as_deref(),
            '^',
            out,
        )?,
        Inline::MathOverline(data) => {
            render_math_inline_one_arg("overline", data.body.as_deref(), out)?
        }
        Inline::MathOverparen(data) => render_math_inline_annotated(
            "overbrace",
            data.body.as_deref(),
            data.annotation.as_deref(),
            '^',
            out,
        )?,
        Inline::MathOvershell(data) => render_math_inline_annotation(
            "overset",
            data.body.as_deref(),
            data.annotation.as_deref(),
            out,
        )?,
        Inline::MathPrimes(data) => render_math_inline_primes(data, out)?,
        Inline::MathRoot(data) => render_math_inline_root(data, out)?,
        Inline::MathScripts(data) => render_math_inline_expr(data.body.as_deref(), out)?,
        Inline::MathStretch(data) => render_math_inline_expr(data.body.as_deref(), out)?,
        Inline::MathUnderbrace(data) => render_math_inline_annotated(
            "underbrace",
            data.body.as_deref(),
            data.annotation.as_deref(),
            '_',
            out,
        )?,
        Inline::MathUnderbracket(data) => render_math_inline_annotated(
            "underbrace",
            data.body.as_deref(),
            data.annotation.as_deref(),
            '_',
            out,
        )?,
        Inline::MathUnderline(data) => {
            render_math_inline_one_arg("underline", data.body.as_deref(), out)?
        }
        Inline::MathUnderparen(data) => render_math_inline_annotated(
            "underbrace",
            data.body.as_deref(),
            data.annotation.as_deref(),
            '_',
            out,
        )?,
        Inline::MathUndershell(data) => render_math_inline_annotation(
            "underset",
            data.body.as_deref(),
            data.annotation.as_deref(),
            out,
        )?,
        Inline::MathVec(data) => render_math_inline_vec(data, out)?,
        _ => unreachable!("render_math_inline only receives math inline nodes"),
    }
    Ok(())
}

pub(super) fn render_math(node: &MathNode, out: &mut String) -> Result<()> {
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

fn render_math_inline_expr(value: Option<&str>, out: &mut String) -> Result<()> {
    let Some(value) = value.filter(|value| !is_auto_or_none(value)) else {
        return Ok(());
    };

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
        return render_math_value(&parse_math_json_value(&json)?, out);
    }

    render_math_symbol(value, out);
    Ok(())
}

fn render_math_inline_one_arg(command: &str, value: Option<&str>, out: &mut String) -> Result<()> {
    out.push('\\');
    out.push_str(command);
    out.push('{');
    render_math_inline_expr(value, out)?;
    out.push('}');
    Ok(())
}

fn render_math_inline_accent(data: &MathAccentInline, out: &mut String) -> Result<()> {
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

fn render_math_inline_attach(data: &MathAttachInline, out: &mut String) -> Result<()> {
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

fn render_math_inline_script_pair(
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

fn render_math_inline_binom(data: &MathBinomInline, out: &mut String) -> Result<()> {
    out.push_str(r"\binom{");
    render_math_inline_expr(data.upper.as_deref(), out)?;
    out.push_str("}{");
    render_math_inline_expr(data.lower.as_deref(), out)?;
    out.push('}');
    Ok(())
}

fn render_math_inline_cancel(data: &MathCancelInline, out: &mut String) -> Result<()> {
    let command = if data.cross {
        "xcancel"
    } else if data.inverted {
        "bcancel"
    } else {
        "cancel"
    };
    render_math_inline_one_arg(command, data.body.as_deref(), out)
}

fn render_math_inline_cases(data: &MathCasesInline, out: &mut String) -> Result<()> {
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

fn render_math_inline_frac(data: &MathFracInline, out: &mut String) -> Result<()> {
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

fn render_math_inline_limits(data: &MathLimitsInline, out: &mut String) -> Result<()> {
    render_math_inline_expr(data.body.as_deref(), out)?;
    if data.inline {
        out.push_str(r"\nolimits");
    } else {
        out.push_str(r"\limits");
    }
    Ok(())
}

fn render_math_inline_matrix(data: &MathMatInline, out: &mut String) -> Result<()> {
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

fn render_math_inline_op(data: &MathOpInline, out: &mut String) -> Result<()> {
    out.push_str(r"\operatorname{");
    render_math_inline_expr(data.text.as_deref(), out)?;
    out.push('}');
    if data.limits {
        out.push_str(r"\limits");
    }
    Ok(())
}

fn render_math_inline_annotated(
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

fn render_math_inline_annotation(
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

fn render_math_inline_primes(data: &MathPrimesInline, out: &mut String) -> Result<()> {
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

fn render_math_inline_root(data: &MathRootInline, out: &mut String) -> Result<()> {
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

fn render_math_inline_vec(data: &MathVecInline, out: &mut String) -> Result<()> {
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

fn render_inline_node_as_math(node: &Inline, out: &mut String) -> Result<()> {
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

fn render_math_nodes(nodes: &[MathNode], out: &mut String) -> Result<()> {
    for node in nodes {
        render_math(node, out)?;
    }
    Ok(())
}

fn render_math_symbol(symbol: &str, out: &mut String) {
    let command = match symbol {
        "∑" => Some(r"\sum"),
        "∏" => Some(r"\prod"),
        "∫" => Some(r"\int"),
        "∞" => Some(r"\infty"),
        "→" => Some(r"\to"),
        "←" => Some(r"\leftarrow"),
        "↔" => Some(r"\leftrightarrow"),
        "⇒" => Some(r"\Rightarrow"),
        "⇐" => Some(r"\Leftarrow"),
        "⇔" => Some(r"\Leftrightarrow"),
        "≤" => Some(r"\le"),
        "≥" => Some(r"\ge"),
        "≠" => Some(r"\ne"),
        "≈" => Some(r"\approx"),
        "∈" => Some(r"\in"),
        "∉" => Some(r"\notin"),
        "⊂" => Some(r"\subset"),
        "⊆" => Some(r"\subseteq"),
        "∂" => Some(r"\partial"),
        "…" => Some(r"\dots"),
        "α" => Some(r"\alpha"),
        "β" => Some(r"\beta"),
        "γ" => Some(r"\gamma"),
        "δ" => Some(r"\delta"),
        "ε" => Some(r"\epsilon"),
        "ζ" => Some(r"\zeta"),
        "η" => Some(r"\eta"),
        "θ" => Some(r"\theta"),
        "ι" => Some(r"\iota"),
        "κ" => Some(r"\kappa"),
        "λ" => Some(r"\lambda"),
        "μ" => Some(r"\mu"),
        "ν" => Some(r"\nu"),
        "ξ" => Some(r"\xi"),
        "π" => Some(r"\pi"),
        "ρ" => Some(r"\rho"),
        "σ" => Some(r"\sigma"),
        "τ" => Some(r"\tau"),
        "φ" => Some(r"\phi"),
        "χ" => Some(r"\chi"),
        "ψ" => Some(r"\psi"),
        "ω" => Some(r"\omega"),
        "Γ" => Some(r"\Gamma"),
        "Δ" => Some(r"\Delta"),
        "Θ" => Some(r"\Theta"),
        "Λ" => Some(r"\Lambda"),
        "Ξ" => Some(r"\Xi"),
        "Π" => Some(r"\Pi"),
        "Σ" => Some(r"\Sigma"),
        "Φ" => Some(r"\Phi"),
        "Ψ" => Some(r"\Psi"),
        "Ω" => Some(r"\Omega"),
        "‖" => Some(r"\Vert "),
        "⌊" => Some(r"\lfloor "),
        "⌋" => Some(r"\rfloor"),
        "⌈" => Some(r"\lceil "),
        "⌉" => Some(r"\rceil"),
        _ => None,
    };

    if let Some(command) = command {
        out.push_str(command);
    } else {
        out.push_str(symbol);
    }
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

fn render_math_delimited_matrix(
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

fn render_math_rows(rows: &[Vec<MathNode>], out: &mut String) -> Result<()> {
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

fn render_math_value(value: &MathValue, out: &mut String) -> Result<()> {
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

fn parse_math_json_value(value: &serde_json::Value) -> Result<MathValue> {
    match value {
        serde_json::Value::Null => Ok(MathValue::None),
        serde_json::Value::Bool(value) => Ok(MathValue::Bool(*value)),
        serde_json::Value::Number(value) => Ok(MathValue::Scalar(value.to_string().into())),
        serde_json::Value::String(value) => Ok(MathValue::Scalar(value.as_str().into())),
        serde_json::Value::Object(_) => Ok(MathValue::Node(Box::new(parse_math_json_node(value)?))),
        serde_json::Value::Array(values) => parse_math_json_array(values),
    }
}

fn parse_math_json_node(value: &serde_json::Value) -> Result<MathNode> {
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

fn parse_math_json_array(values: &[serde_json::Value]) -> Result<MathValue> {
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

fn push_latex_text_escaped(value: &str, out: &mut String) {
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

fn math_style_command(
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

fn matrix_env(open: Option<&str>, close: Option<&str>) -> Option<&'static str> {
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

fn math_delim_pair(
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

fn inline_delim_pair(
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

fn json_delim(value: &serde_json::Value) -> Option<&str> {
    match value {
        serde_json::Value::Null => None,
        serde_json::Value::String(value) => Some(value),
        _ => None,
    }
}

fn math_delim(value: &str) -> &str {
    match value {
        r"{" => r"\{",
        r"}" => r"\}",
        "‖" => r"\|",
        _ => value,
    }
}

fn math_matching_delim(open: &str) -> &str {
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

fn math_field<'a>(node: &'a MathNode, name: &str) -> Result<&'a MathValue> {
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

fn math_optional_field<'a>(node: &'a MathNode, name: &str) -> Option<&'a MathValue> {
    node.fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| &field.value)
}

fn math_child<'a>(node: &'a MathNode, name: &str) -> Result<&'a MathNode> {
    match math_field(node, name)? {
        MathValue::Node(child) => Ok(child),
        _ => bail!("math.{} field `{name}` must be a node", node.func),
    }
}

fn math_optional_child<'a>(node: &'a MathNode, name: &str) -> Result<Option<&'a MathNode>> {
    match node.fields.iter().find(|field| field.name == name) {
        Some(field) => match &field.value {
            MathValue::Node(child) => Ok(Some(child)),
            MathValue::None => Ok(None),
            _ => bail!("math.{} field `{name}` must be a node", node.func),
        },
        None => Ok(None),
    }
}

fn math_nodes<'a>(node: &'a MathNode, name: &str) -> Result<&'a [MathNode]> {
    match math_field(node, name)? {
        MathValue::Nodes(nodes) => Ok(nodes),
        _ => bail!("math.{} field `{name}` must be a node list", node.func),
    }
}

fn math_rows<'a>(node: &'a MathNode, name: &str) -> Result<&'a [Vec<MathNode>]> {
    match math_field(node, name)? {
        MathValue::Rows(rows) => Ok(rows),
        _ => bail!("math.{} field `{name}` must be a row list", node.func),
    }
}

fn math_scalar<'a>(node: &'a MathNode, name: &str) -> Result<&'a str> {
    match math_field(node, name)? {
        MathValue::Scalar(value) => Ok(value),
        _ => bail!("math.{} field `{name}` must be a scalar", node.func),
    }
}

fn math_optional_scalar<'a>(node: &'a MathNode, name: &str) -> Result<Option<&'a str>> {
    match math_optional_field(node, name) {
        Some(MathValue::Scalar(value)) => Ok(Some(value)),
        Some(MathValue::None) | None => Ok(None),
        _ => bail!("math.{} field `{name}` must be a scalar", node.func),
    }
}

fn math_bool(node: &MathNode, name: &str) -> Result<bool> {
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
