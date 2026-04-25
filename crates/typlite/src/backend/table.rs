use crate::Result;
use crate::ir::*;

use super::{BibliographyContext, push_html_escaped, render_inlines, render_inlines_html};

pub(super) fn render_table(
    rows: &[TableRow],
    alignments: &[TableAlign],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }

    let columns = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);
    if columns == 0 {
        return Ok(());
    }

    if requires_html_table(rows) {
        return render_html_table(rows, indent, bibliography, out);
    }

    render_table_row(&rows[0], columns, indent, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push('|');
    for index in 0..columns {
        out.push(' ');
        out.push_str(table_align_marker(
            alignments
                .get(index)
                .copied()
                .unwrap_or(TableAlign::Default),
        ));
        out.push_str(" |");
    }

    for row in &rows[1..] {
        out.push('\n');
        render_table_row(row, columns, indent, bibliography, out)?;
    }

    Ok(())
}

fn requires_html_table(rows: &[TableRow]) -> bool {
    rows.iter().any(|row| {
        row.cells.iter().any(|cell| {
            cell.colspan != 1
                || cell.rowspan != 1
                || cell.align != TableAlign::Default
                || table_cell_requires_html(&cell.body)
        })
    })
}

fn table_cell_requires_html(body: &[Inline]) -> bool {
    body.iter().any(|inline| match inline {
        Inline::Raw(data) => data.lang.is_some() || data.text.contains('\n'),
        Inline::Linebreak(_) | Inline::Frame(_) => true,
        Inline::TableCell(_)
        | Inline::TableFooter(_)
        | Inline::TableHeader(_)
        | Inline::GridCell(_)
        | Inline::GridFooter(_)
        | Inline::GridHeader(_) => true,
        _ => false,
    })
}

fn render_html_table(
    rows: &[TableRow],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let indentation = " ".repeat(indent);
    out.push_str(&indentation);
    out.push_str("<table>");

    for row in rows {
        out.push('\n');
        out.push_str(&indentation);
        out.push_str("  <tr>");
        for cell in &row.cells {
            out.push('\n');
            out.push_str(&indentation);
            out.push_str("    <td");
            if cell.colspan > 1 {
                out.push_str(" colspan=\"");
                out.push_str(&cell.colspan.to_string());
                out.push('"');
            }
            if cell.rowspan > 1 {
                out.push_str(" rowspan=\"");
                out.push_str(&cell.rowspan.to_string());
                out.push('"');
            }
            if let Some(align) = table_align_style(cell.align) {
                out.push_str(" style=\"text-align: ");
                out.push_str(align);
                out.push('"');
            }
            out.push('>');
            render_table_cell_html(&cell.body, bibliography, out)?;
            out.push_str("</td>");
        }
        out.push('\n');
        out.push_str(&indentation);
        out.push_str("  </tr>");
    }

    out.push('\n');
    out.push_str(&indentation);
    out.push_str("</table>");
    Ok(())
}

fn render_table_cell_html(
    body: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    match body {
        [Inline::Raw(data)] if data.lang.is_some() || data.text.contains('\n') => {
            out.push_str("<pre><code");
            if let Some(lang) = &data.lang {
                out.push_str(" class=\"language-");
                push_html_escaped(lang, out);
                out.push('"');
            }
            out.push('>');
            push_html_escaped(&data.text, out);
            out.push_str("</code></pre>");
            Ok(())
        }
        _ => render_inlines_html(body, bibliography, out),
    }
}

fn table_align_style(align: TableAlign) -> Option<&'static str> {
    match align {
        TableAlign::Default => None,
        TableAlign::Left => Some("left"),
        TableAlign::Center => Some("center"),
        TableAlign::Right => Some("right"),
    }
}

fn table_align_marker(align: TableAlign) -> &'static str {
    match align {
        TableAlign::Default => "---",
        TableAlign::Left => ":---",
        TableAlign::Center => ":---:",
        TableAlign::Right => "---:",
    }
}

fn render_table_row(
    row: &TableRow,
    columns: usize,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push('|');
    for index in 0..columns {
        out.push(' ');
        if let Some(cell) = row.cells.get(index) {
            render_inlines(&cell.body, bibliography, out)?;
        }
        out.push_str(" |");
    }

    Ok(())
}
