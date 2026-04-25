use crate::Result;
use crate::ir::*;

use super::{render_unimplemented, render_unimplemented_inline};

mod inline_compat;
mod node;
mod support;
mod symbol;
use self::inline_compat::*;
pub(super) use self::node::render_math;

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

pub(super) fn render_math_inline_body(node: &Inline, out: &mut String) -> Result<()> {
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
