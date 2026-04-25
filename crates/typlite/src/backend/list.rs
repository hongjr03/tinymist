use crate::Result;
use crate::ir::*;

use super::{BibliographyContext, render_blocks, render_inlines};

pub(super) fn render_list(
    ordered: bool,
    start: Option<i64>,
    reversed: bool,
    items: &[crate::ir::ListItem],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let mut next_number = if reversed {
        start.unwrap_or(items.len() as i64)
    } else {
        start.unwrap_or(1)
    };

    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        let marker = if ordered {
            let number = item
                .number
                .as_ref()
                .and_then(|number| number.parse::<i64>().ok())
                .unwrap_or(next_number);
            next_number = if reversed { number - 1 } else { number + 1 };
            format!("{number}.")
        } else {
            "-".to_owned()
        };
        let prefix = format!("{}{} ", " ".repeat(indent), marker);
        let continuation = indent + marker.len() + 1;
        let body = render_blocks(&item.body, continuation, bibliography)?;

        if body.is_empty() {
            out.push_str(&prefix);
            continue;
        }

        let mut lines = body.lines();
        if let Some(first) = lines.next() {
            out.push_str(&prefix);
            out.push_str(first.trim_start());
        }

        for line in lines {
            out.push('\n');
            out.push_str(line);
        }
    }

    Ok(())
}

pub(super) fn render_terms(
    items: &[TermItem],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push('\n');
            out.push('\n');
        }

        out.push_str(&" ".repeat(indent));
        render_inlines(item.term.as_slice(), bibliography, out)?;

        let description = render_blocks(&item.description, indent + 2, bibliography)?;
        if !description.is_empty() {
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            out.push_str(": ");
            let mut lines = description.lines();
            if let Some(first) = lines.next() {
                out.push_str(first.trim_start());
            }
            for line in lines {
                out.push('\n');
                out.push_str(line);
            }
        }
    }

    Ok(())
}
