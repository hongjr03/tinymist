//! Experimental backends for typlite IR.

use crate::Result;
use crate::ir::{
    Block, BlockElementData, Document, ElementFieldValue, FrameImage, Inline, InlineElementData,
    MathNode, MathValue, TableAlign, TableRow, TermItem,
};
use base64::Engine;
use ecow::EcoString;
use std::cell::RefCell;
use std::collections::BTreeMap;
use tinymist_std::error::prelude::*;
use typst::visualize::{ExchangeFormat, ImageFormat, RasterFormat, VectorFormat};

/// Rendered bibliography entries available to the Markdown backend.
#[derive(Debug, Default, Clone)]
pub struct BibliographyContext {
    entries: BTreeMap<EcoString, EcoString>,
    citations: BTreeMap<EcoString, EcoString>,
    citation_offsets: RefCell<BTreeMap<EcoString, usize>>,
    reference_anchors: RefCell<BTreeMap<String, Vec<EcoString>>>,
    order: Vec<EcoString>,
}

impl BibliographyContext {
    /// Creates a bibliography context from rendered entries.
    pub fn new(
        entries: impl IntoIterator<Item = (EcoString, EcoString)>,
        citations: impl IntoIterator<Item = (EcoString, EcoString)>,
    ) -> Self {
        let mut map = BTreeMap::new();
        let mut order = Vec::new();

        for (key, rendered) in entries {
            if !map.contains_key(&key) {
                order.push(key.clone());
            }
            map.insert(key, rendered);
        }

        Self {
            entries: map,
            citations: citations.into_iter().collect(),
            citation_offsets: RefCell::default(),
            reference_anchors: RefCell::default(),
            order,
        }
    }

    fn reset_render_state(&self, doc: &Document) {
        self.citation_offsets.borrow_mut().clear();
        let mut anchors = self.reference_anchors.borrow_mut();
        anchors.clear();
        collect_reference_anchors(&doc.blocks, &mut anchors);
    }

    fn ordered_entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.order.iter().filter_map(|key| {
            self.entries
                .get(key)
                .map(|rendered| (key.as_str(), rendered.as_str()))
        })
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn citation(&self, key: &str) -> Option<&str> {
        self.citations.get(key).map(EcoString::as_str)
    }

    fn next_citation_id(&self, key: &str) -> String {
        let mut offsets = self.citation_offsets.borrow_mut();
        let offset = offsets.entry(key.into()).or_default();
        *offset += 1;
        format!("cite-{key}-{offset}")
    }

    fn citation_count(&self, key: &str) -> usize {
        self.citation_offsets
            .borrow()
            .get(key)
            .copied()
            .unwrap_or_default()
    }

    fn take_reference_anchors(&self, block: &Block) -> Vec<EcoString> {
        self.reference_anchors
            .borrow_mut()
            .remove(&reference_anchor_key(block))
            .unwrap_or_default()
    }
}

/// Renders a document IR as Markdown.
pub fn render_markdown(doc: &Document) -> Result<String> {
    render_markdown_with_bibliography(doc, &BibliographyContext::default())
}

/// Renders a document IR as Markdown with a bibliography context.
pub fn render_markdown_with_bibliography(
    doc: &Document,
    bibliography: &BibliographyContext,
) -> Result<String> {
    bibliography.reset_render_state(doc);
    render_blocks(&doc.blocks, 0, bibliography)
}

fn collect_reference_anchors(blocks: &[Block], out: &mut BTreeMap<String, Vec<EcoString>>) {
    for block in blocks {
        match block {
            Block::Heading { body, .. } | Block::Paragraph(body) => {
                collect_reference_anchors_in_inlines(body, out);
            }
            Block::Quote(blocks) => collect_reference_anchors(blocks, out),
            Block::Figure { body, caption, .. } => {
                collect_reference_anchors(body, out);
                collect_reference_anchors_in_inlines(caption, out);
            }
            Block::Align { body, .. } => collect_reference_anchors(body, out),
            Block::Table { rows, .. } => {
                for row in rows {
                    for cell in &row.cells {
                        collect_reference_anchors_in_inlines(&cell.body, out);
                    }
                }
            }
            Block::List { items, .. } => {
                for item in items {
                    collect_reference_anchors(&item.body, out);
                }
            }
            Block::Terms { items } => {
                for item in items {
                    collect_reference_anchors_in_inlines(&item.term, out);
                    collect_reference_anchors(&item.description, out);
                }
            }
            Block::Math(_) | Block::Raw { .. } => {}
            _ => {
                if let Some(body) = block.generated_body() {
                    collect_reference_anchors(body, out);
                }
            }
        }
    }
}

fn collect_reference_anchors_in_inlines(
    inlines: &[Inline],
    out: &mut BTreeMap<String, Vec<EcoString>>,
) {
    for inline in inlines {
        match inline {
            Inline::Ref(data) => {
                if let Some(target) = data.scalar("target").or_else(|| data.scalar("label")) {
                    if let Some(element) = data.blocks("element").and_then(|blocks| blocks.first())
                    {
                        push_reference_anchor(
                            out,
                            reference_anchor_key(element),
                            normalized_label(target).into(),
                        );
                    }
                }
                collect_reference_anchors_in_fields(&data.fields, out);
            }
            Inline::Emph(children)
            | Inline::Strong(children)
            | Inline::Strike(children)
            | Inline::Sub(children)
            | Inline::Super(children)
            | Inline::Link { body: children, .. } => {
                collect_reference_anchors_in_inlines(children, out);
            }
            Inline::Text(_)
            | Inline::Math(_)
            | Inline::Linebreak
            | Inline::Frame(_)
            | Inline::Raw { .. } => {}
            _ => {
                if let Some(body) = inline.generated_body() {
                    collect_reference_anchors_in_inlines(body, out);
                }
            }
        }
    }
}

fn collect_reference_anchors_in_fields(
    fields: &[crate::ir::ElementField],
    out: &mut BTreeMap<String, Vec<EcoString>>,
) {
    for field in fields {
        match &field.value {
            ElementFieldValue::Inlines(inlines) => {
                collect_reference_anchors_in_inlines(inlines, out)
            }
            ElementFieldValue::Blocks(blocks) => collect_reference_anchors(blocks, out),
            ElementFieldValue::Scalar(_) => {}
        }
    }
}

fn push_reference_anchor(
    out: &mut BTreeMap<String, Vec<EcoString>>,
    key: String,
    anchor: EcoString,
) {
    let anchors = out.entry(key).or_default();
    if !anchors.iter().any(|existing| existing == &anchor) {
        anchors.push(anchor);
    }
}

fn reference_anchor_key(block: &Block) -> String {
    format!("{block:?}")
}

fn normalized_label(label: &str) -> &str {
    label.trim_start_matches('<').trim_end_matches('>')
}

fn render_blocks(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
) -> Result<String> {
    let mut out = String::new();

    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, bibliography, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(rendered.trim_end_matches([' ', '\t']));
    }

    Ok(out)
}

fn render_block(
    block: &Block,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_reference_anchors(bibliography.take_reference_anchors(block), indent, out);
    match block {
        Block::Heading { level, body } => {
            out.push_str(&" ".repeat(indent));
            out.push_str(&"#".repeat(*level as usize));
            out.push(' ');
            render_inlines(body, bibliography, out)?;
        }
        Block::Paragraph(body) => {
            out.push_str(&" ".repeat(indent));
            render_inlines(body, bibliography, out)?;
        }
        Block::Quote(blocks) => render_quote(blocks, indent, bibliography, out)?,
        Block::Figure { body, caption, alt } => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<figure");
            if let Some(alt) = alt {
                out.push_str(" aria-label=\"");
                push_html_escaped(alt, out);
                out.push('"');
            }
            out.push_str(">\n");
            render_blocks_html_into(body, indent, bibliography, out)?;
            if !caption.is_empty() {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
                out.push_str("<figcaption>");
                render_inlines_html(caption, bibliography, out)?;
                out.push_str("</figcaption>");
            }
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            out.push_str("</figure>");
        }
        Block::Align { alignment, body } => {
            render_align(alignment.as_deref(), body, indent, bibliography, out)?
        }
        Block::Math(body) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("$$");
            render_math(body, out)?;
            out.push_str("$$");
        }
        Block::Table { rows, alignments } => {
            render_table(rows, alignments, indent, bibliography, out)?
        }
        Block::Raw { lang, text } => {
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
            if let Some(lang) = lang {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
        }
        Block::List {
            ordered,
            start,
            reversed,
            items,
            ..
        } => render_list(
            *ordered,
            *start,
            *reversed,
            items,
            indent,
            bibliography,
            out,
        )?,
        Block::Columns(data) => render_columns(data, indent, bibliography, out)?,
        Block::Move(data) => render_move_block(data, indent, bibliography, out)?,
        Block::Pad(data) => render_pad_block(data, indent, bibliography, out)?,
        Block::Rotate(data) => {
            render_transform_block(data, "rotate", &["angle"], indent, bibliography, out)?
        }
        Block::Scale(data) => render_scale_block(data, indent, bibliography, out)?,
        Block::Skew(data) => {
            render_transform_block(data, "skew", &["ax", "ay"], indent, bibliography, out)?
        }
        Block::Stack(data) => render_stack(data, indent, bibliography, out)?,
        Block::Block(data) | Block::Title(data) => {
            render_blocks_into(&data.body, indent, bibliography, out)?
        }
        Block::Terms { items } => render_terms(items, indent, bibliography, out)?,
        Block::Colbreak(_) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<div style=\"break-after: column\"></div>");
        }
        Block::V(data) => render_vertical_space(data, indent, out)?,
        Block::Parbreak(_) => {}
        Block::Outline(data) => {
            if let Some(title) = data.field("title") {
                out.push_str(&" ".repeat(indent));
                render_field_value(title, out)?;
            }
        }
        Block::Pagebreak(_) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<div style=\"break-after: page\"></div>");
        }
        Block::Bibliography(data) => render_bibliography(data, bibliography, indent, out)?,
    }

    Ok(())
}

fn render_blocks_into(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block(block, indent, bibliography, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push_str("\n\n");
        }
        rendered_count += 1;
        out.push_str(rendered.trim_end_matches([' ', '\t']));
    }

    Ok(())
}

fn render_reference_anchors(anchors: Vec<EcoString>, indent: usize, out: &mut String) {
    for anchor in anchors {
        out.push_str(&" ".repeat(indent));
        out.push_str("<a id=\"");
        push_html_escaped(&anchor, out);
        out.push_str("\"></a>\n");
    }
}

fn render_blocks_html_into(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let mut rendered_count = 0usize;

    for block in blocks {
        let mut rendered = String::new();
        render_block_html(block, indent, bibliography, &mut rendered)?;
        if rendered.is_empty() {
            continue;
        }

        if rendered_count > 0 {
            out.push('\n');
        }
        rendered_count += 1;
        out.push_str(&rendered);
    }

    Ok(())
}

fn render_block_html(
    block: &Block,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    match block {
        Block::Paragraph(body) => {
            out.push_str(&" ".repeat(indent));
            render_inlines_html(body, bibliography, out)
        }
        Block::Raw { lang, text } => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<pre><code");
            if let Some(lang) = lang {
                out.push_str(" class=\"language-");
                push_html_escaped(lang, out);
                out.push('"');
            }
            out.push('>');
            push_html_escaped(text, out);
            out.push_str("</code></pre>");
            Ok(())
        }
        _ => render_block(block, indent, bibliography, out),
    }
}

fn render_align(
    alignment: Option<&str>,
    body: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(text_align) = alignment.and_then(css_text_align) else {
        return render_blocks_into(body, indent, bibliography, out);
    };

    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"text-align: ");
    out.push_str(text_align);
    out.push_str("\">\n");
    render_blocks_html_into(body, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_columns(
    data: &BlockElementData,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"");
    let mut has_style = false;
    if let Some(count) = data.scalar("count").filter(|value| !value.is_empty()) {
        has_style = true;
        out.push_str("column-count: ");
        push_html_escaped(count, out);
    }
    if let Some(gutter) = data.scalar("gutter").filter(|value| !value.is_empty()) {
        if has_style {
            out.push_str("; ");
        }
        out.push_str("column-gap: ");
        push_css_length(gutter, out);
    }
    out.push_str("\">\n");
    render_blocks_html_into(&data.body, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_stack(
    data: &BlockElementData,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"display: flex");
    if let Some(direction) = data.scalar("dir").and_then(css_stack_direction) {
        out.push_str("; flex-direction: ");
        out.push_str(direction);
    }
    if let Some(spacing) = data.scalar("spacing").filter(|value| !value.is_empty()) {
        out.push_str("; gap: ");
        push_css_length(spacing, out);
    }
    out.push_str("\">\n");
    render_blocks_html_into(&data.body, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_vertical_space(data: &BlockElementData, indent: usize, out: &mut String) -> Result<()> {
    let Some(amount) = data.scalar("amount").filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"height: ");
    push_css_length(amount, out);
    out.push_str("\"></div>");

    Ok(())
}

fn render_bibliography(
    data: &BlockElementData,
    bibliography: &BibliographyContext,
    indent: usize,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<section data-typlite-bibliography=\"true\">");
    if let Some(title) = bibliography_title(data)? {
        out.push('\n');
        out.push_str(&" ".repeat(indent + 2));
        out.push_str("<h2>");
        push_html_escaped(&title, out);
        out.push_str("</h2>");
    }
    if bibliography.is_empty() {
        out.push('\n');
        out.push_str(&" ".repeat(indent + 2));
        out.push_str("<!-- typlite-bibliography: no rendered entries -->");
    } else {
        for (key, entry) in bibliography.ordered_entries() {
            out.push('\n');
            out.push_str(&" ".repeat(indent + 2));
            out.push_str("<div id=\"ref-");
            push_html_escaped(key, out);
            out.push_str("\" class=\"csl-entry\">");
            push_html_escaped(entry, out);
            for citation_index in 1..=bibliography.citation_count(key) {
                out.push_str(" <a href=\"#cite-");
                push_html_escaped(key, out);
                out.push('-');
                out.push_str(&citation_index.to_string());
                out.push_str(
                    "\" class=\"csl-backref\" aria-label=\"Back to citation\">&#8617;</a>",
                );
            }
            out.push_str("</div>");
        }
    }
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</section>");
    Ok(())
}

fn bibliography_title(data: &BlockElementData) -> Result<Option<String>> {
    let Some(title) = data.field("title") else {
        return Ok(None);
    };

    let mut rendered = String::new();
    render_field_value(title, &mut rendered)?;
    if rendered.is_empty() || rendered == "auto" || rendered == "none" {
        Ok(None)
    } else {
        Ok(Some(rendered))
    }
}

fn render_pad_block(
    data: &BlockElementData,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(data, indent, bibliography, out, |data, out| {
        out.push_str("display: block");
        push_optional_block_css_length(data, "left", "padding-left", out);
        push_optional_block_css_length(data, "top", "padding-top", out);
        push_optional_block_css_length(data, "right", "padding-right", out);
        push_optional_block_css_length(data, "bottom", "padding-bottom", out);
    })
}

fn render_move_block(
    data: &BlockElementData,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(data, indent, bibliography, out, |data, out| {
        out.push_str("display: block; transform: translate(");
        push_css_length(data.scalar("dx").unwrap_or("0pt"), out);
        out.push_str(", ");
        push_css_length(data.scalar("dy").unwrap_or("0pt"), out);
        out.push(')');
    })
}

fn render_transform_block(
    data: &BlockElementData,
    function: &str,
    fields: &[&str],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(data, indent, bibliography, out, |data, out| {
        out.push_str("display: block; transform: ");
        out.push_str(function);
        out.push('(');
        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            push_html_escaped(data.scalar(field).unwrap_or("0deg"), out);
        }
        out.push(')');
    })
}

fn render_scale_block(
    data: &BlockElementData,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(data, indent, bibliography, out, |data, out| {
        out.push_str("display: block; transform: scale(");
        push_css_scale(
            data.scalar("x")
                .or_else(|| data.scalar("factor"))
                .unwrap_or("100%"),
            out,
        );
        out.push_str(", ");
        push_css_scale(
            data.scalar("y")
                .or_else(|| data.scalar("factor"))
                .unwrap_or("100%"),
            out,
        );
        out.push(')');
    })
}

fn render_layout_div(
    data: &BlockElementData,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
    push_style: impl FnOnce(&BlockElementData, &mut String),
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"");
    push_style(data, out);
    out.push_str("\">\n");
    render_blocks_html_into(&data.body, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");
    Ok(())
}

fn css_stack_direction(value: &str) -> Option<&'static str> {
    match value.trim() {
        "ttb" => Some("column"),
        "btt" => Some("column-reverse"),
        "ltr" => Some("row"),
        "rtl" => Some("row-reverse"),
        _ => None,
    }
}

fn css_text_align(value: &str) -> Option<&'static str> {
    let value = value.trim();
    if value.contains("center") || value.contains("horizon") {
        Some("center")
    } else if value.contains("right") || value.contains("end") {
        Some("right")
    } else if value.contains("left") || value.contains("start") {
        Some("left")
    } else {
        None
    }
}

fn push_css_length(value: &str, out: &mut String) {
    if value.contains('+') || value.contains('-') {
        out.push_str("calc(");
        push_html_escaped(value, out);
        out.push(')');
    } else {
        push_html_escaped(value, out);
    }
}

fn render_quote(
    blocks: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let body = render_blocks(blocks, 0, bibliography)?;
    let prefix = format!("{}> ", " ".repeat(indent));

    for (index, line) in body.lines().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&prefix);
        out.push_str(line);
    }

    Ok(())
}

fn render_table(
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
        row.cells
            .iter()
            .any(|cell| cell.colspan != 1 || cell.rowspan != 1 || cell.align != TableAlign::Default)
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
            render_inlines_html(&cell.body, bibliography, out)?;
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

fn render_list(
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

fn render_terms(
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

fn render_inlines(
    nodes: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    for node in nodes {
        match node {
            Inline::Text(text) => out.push_str(text),
            Inline::Emph(children) => {
                out.push('*');
                render_inlines(children, bibliography, out)?;
                out.push('*');
            }
            Inline::Strong(children) => {
                out.push_str("**");
                render_inlines(children, bibliography, out)?;
                out.push_str("**");
            }
            Inline::Link { dest, body } => {
                out.push('[');
                render_inlines(body, bibliography, out)?;
                out.push_str("](");
                out.push_str(dest);
                out.push(')');
            }
            Inline::Strike(children) => {
                out.push_str("~~");
                render_inlines(children, bibliography, out)?;
                out.push_str("~~");
            }
            Inline::Sub(children) => {
                out.push_str("<sub>");
                render_inlines(children, bibliography, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(children) => {
                out.push_str("<sup>");
                render_inlines(children, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Math(children) => {
                out.push('$');
                render_math(children, out)?;
                out.push('$');
            }
            Inline::Linebreak => out.push('\n'),
            Inline::Frame(frame) => render_frame_image("frame", frame, out)?,
            Inline::Raw { text, .. } => {
                out.push('`');
                out.push_str(text);
                out.push('`');
            }
            Inline::Repeat(data) => render_repeat(data, bibliography, out)?,
            Inline::TableCell(data)
            | Inline::TableFooter(data)
            | Inline::TableHeader(data)
            | Inline::GridCell(data)
            | Inline::GridFooter(data)
            | Inline::GridHeader(data)
            | Inline::ParLine(data)
            | Inline::RawLine(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::PdfArtifact(data) => render_pdf_artifact(data, bibliography, out)?,
            Inline::Box(data) => render_box(data, bibliography, out)?,
            Inline::Move(data) => render_move(data, bibliography, out)?,
            Inline::Pad(data) => render_pad(data, bibliography, out)?,
            Inline::Place(data) => render_place(data, bibliography, out)?,
            Inline::Rotate(data) => {
                render_transform(data, "rotate", &["angle"], bibliography, out)?
            }
            Inline::Scale(data) => render_scale(data, bibliography, out)?,
            Inline::Skew(data) => render_transform(data, "skew", &["ax", "ay"], bibliography, out)?,
            Inline::Quote(data) => render_inline_quote(data, bibliography, out)?,
            Inline::Circle(data)
            | Inline::Curve(data)
            | Inline::Ellipse(data)
            | Inline::Line(data)
            | Inline::Path(data)
            | Inline::Polygon(data)
            | Inline::Rect(data)
            | Inline::Square(data) => render_element_frame(node, data, out)?,
            Inline::Cite(data) => render_cite(data, bibliography, out)?,
            Inline::Metadata(data) => render_metadata(data, out),
            Inline::Document(_) | Inline::Hide(_) | Inline::Page(_) => {}
            Inline::FigureCaption(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::FootnoteEntry(_) => {}
            Inline::Footnote(data) => {
                out.push_str("^[");
                render_inlines(&data.body, bibliography, out)?;
                out.push(']');
            }
            Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::PlaceFlush(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
            Inline::H(_) => out.push(' '),
            Inline::Highlight(data) => {
                out.push_str("<mark>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</mark>");
            }
            Inline::Image(data) => render_image(data, out)?,
            Inline::MathAccent(data)
            | Inline::MathAttach(data)
            | Inline::MathBinom(data)
            | Inline::MathCancel(data)
            | Inline::MathCases(data)
            | Inline::MathClass(data)
            | Inline::MathFrac(data)
            | Inline::MathLimits(data)
            | Inline::MathLr(data)
            | Inline::MathMat(data)
            | Inline::MathMid(data)
            | Inline::MathOp(data)
            | Inline::MathOverbrace(data)
            | Inline::MathOverbracket(data)
            | Inline::MathOverline(data)
            | Inline::MathOverparen(data)
            | Inline::MathOvershell(data)
            | Inline::MathPrimes(data)
            | Inline::MathRoot(data)
            | Inline::MathScripts(data)
            | Inline::MathStretch(data)
            | Inline::MathUnderbrace(data)
            | Inline::MathUnderbracket(data)
            | Inline::MathUnderline(data)
            | Inline::MathUnderparen(data)
            | Inline::MathUndershell(data)
            | Inline::MathVec(data)
            | Inline::CurveClose(data)
            | Inline::CurveCubic(data)
            | Inline::CurveLine(data)
            | Inline::CurveMove(data)
            | Inline::CurveQuad(data) => render_structured_inline(node, data, out)?,
            Inline::OutlineEntry(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::Overline(data) => {
                out.push_str("<span style=\"text-decoration: overline\">");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::PdfAttach(_) | Inline::PdfEmbed(_) => {
                bail!("typlite markdown PDF embedding rendering is not implemented")
            }
            Inline::Ref(data) => render_ref(data, bibliography, out)?,
            Inline::Smallcaps(data) => {
                out.push_str("<span style=\"font-variant: small-caps\">");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::Smartquote(data) => render_smartquote(data, out)?,
            Inline::Underline(data) => {
                out.push_str("<u>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</u>");
            }
        }
    }

    Ok(())
}

fn render_inlines_html(
    nodes: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    for node in nodes {
        match node {
            Inline::Text(text) => push_html_escaped(text, out),
            Inline::Emph(children) => {
                out.push_str("<em>");
                render_inlines_html(children, bibliography, out)?;
                out.push_str("</em>");
            }
            Inline::Strong(children) => {
                out.push_str("<strong>");
                render_inlines_html(children, bibliography, out)?;
                out.push_str("</strong>");
            }
            Inline::Link { dest, body } => {
                out.push_str("<a href=\"");
                push_html_escaped(dest, out);
                out.push_str("\">");
                render_inlines_html(body, bibliography, out)?;
                out.push_str("</a>");
            }
            Inline::Strike(children) => {
                out.push_str("<del>");
                render_inlines_html(children, bibliography, out)?;
                out.push_str("</del>");
            }
            Inline::Sub(children) => {
                out.push_str("<sub>");
                render_inlines_html(children, bibliography, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(children) => {
                out.push_str("<sup>");
                render_inlines_html(children, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Raw { text, .. } => {
                out.push_str("<code>");
                push_html_escaped(text, out);
                out.push_str("</code>");
            }
            Inline::Linebreak => out.push_str("<br>"),
            Inline::Math(math) => {
                out.push('$');
                render_math(math, out)?;
                out.push('$');
            }
            Inline::Frame(frame) => render_frame_image("frame", frame, out)?,
            Inline::Footnote(data) => {
                out.push_str("<sup>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Highlight(data) => {
                out.push_str("<mark>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</mark>");
            }
            Inline::Image(data) => render_image_html(data, out)?,
            Inline::Overline(data) => {
                out.push_str("<span style=\"text-decoration: overline\">");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::Smallcaps(data) => {
                out.push_str("<span style=\"font-variant: small-caps\">");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::Underline(data) => {
                out.push_str("<u>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</u>");
            }
            Inline::Repeat(data) => render_repeat(data, bibliography, out)?,
            Inline::TableCell(data)
            | Inline::TableFooter(data)
            | Inline::TableHeader(data)
            | Inline::GridCell(data)
            | Inline::GridFooter(data)
            | Inline::GridHeader(data)
            | Inline::ParLine(data)
            | Inline::RawLine(data)
            | Inline::FigureCaption(data)
            | Inline::OutlineEntry(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::PdfArtifact(data) => render_pdf_artifact(data, bibliography, out)?,
            Inline::Box(data) => render_box(data, bibliography, out)?,
            Inline::Move(data) => render_move(data, bibliography, out)?,
            Inline::Pad(data) => render_pad(data, bibliography, out)?,
            Inline::Place(data) => render_place(data, bibliography, out)?,
            Inline::Rotate(data) => {
                render_transform(data, "rotate", &["angle"], bibliography, out)?
            }
            Inline::Scale(data) => render_scale(data, bibliography, out)?,
            Inline::Skew(data) => render_transform(data, "skew", &["ax", "ay"], bibliography, out)?,
            Inline::Quote(data) => render_inline_quote_html(data, bibliography, out)?,
            Inline::FootnoteEntry(_) => {}
            Inline::Circle(data)
            | Inline::Curve(data)
            | Inline::Ellipse(data)
            | Inline::Line(data)
            | Inline::Path(data)
            | Inline::Polygon(data)
            | Inline::Rect(data)
            | Inline::Square(data) => render_element_frame(node, data, out)?,
            Inline::Cite(data) => render_cite(data, bibliography, out)?,
            Inline::Metadata(data) => render_metadata(data, out),
            Inline::Document(_) | Inline::Hide(_) | Inline::Page(_) => {}
            Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::PlaceFlush(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
            Inline::H(_) => out.push(' '),
            Inline::PdfAttach(_) | Inline::PdfEmbed(_) => {
                bail!("typlite markdown PDF embedding rendering is not implemented")
            }
            Inline::Ref(data) => render_ref(data, bibliography, out)?,
            Inline::Smartquote(data) => render_smartquote_html(data, out)?,
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
            | Inline::MathVec(_)
            | Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_) => {
                bail!("typlite HTML table cell rendering is not implemented for generated inline")
            }
        }
    }

    Ok(())
}

fn render_math(node: &MathNode, out: &mut String) -> Result<()> {
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
        "align-point" => {
            out.push('&');
            Ok(())
        }
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
        "mid" => render_math_mid(node, out),
        "scripts" => render_math_limit_style(node, r"\nolimits", out),
        "styled" => render_math_styled(node, out),
        "frac" => render_math_frac(node, out),
        "mat" => render_math_matrix(node, out),
        "op" => render_math_op(node, out),
        "overbrace" => render_math_annotated_command(node, "overbrace", "body", out),
        "overbracket" => render_math_annotated_command(node, "overbracket", "body", out),
        "overline" => render_math_one_arg_command(node, "overline", "body", out),
        "overparen" => render_math_annotated_command(node, "overparen", "body", out),
        "overshell" => render_math_annotation_command(node, "overset", out),
        "primes" => render_math_primes(node, out),
        "root" => render_math_root(node, out),
        "underbrace" => render_math_under_annotated_command(node, "underbrace", out),
        "underbracket" => render_math_under_annotated_command(node, "underbracket", out),
        "underline" => render_math_one_arg_command(node, "underline", "body", out),
        "underparen" => render_math_under_annotated_command(node, "underparen", out),
        "undershell" => render_math_annotation_command(node, "underset", out),
        "vec" => render_math_vec(node, out),
        func => bail!("typlite markdown math rendering is not implemented for `{func}`"),
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
        "\u{20d1}" | "\u{20d0}" => "rightharpoonaccent",
        accent => bail!("typlite markdown math accent `{accent}` is not implemented"),
    };
    render_math_one_arg_command(node, command, "base", out)
}

fn render_math_attach(node: &MathNode, out: &mut String) -> Result<()> {
    if math_optional_child(node, "bl")?.is_some() || math_optional_child(node, "tl")?.is_some() {
        out.push_str("{}");
        render_math_script_pair(node, "bl", "tl", out)?;
    }

    render_math(math_child(node, "base")?, out)?;
    render_math_script_pair(node, "b", "t", out)?;
    render_math_script_pair(node, "br", "tr", out)?;
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
        Some(style) => bail!("typlite markdown math fraction style `{style}` is not implemented"),
    }
}

fn render_math_limit_style(node: &MathNode, suffix: &str, out: &mut String) -> Result<()> {
    render_math(math_child(node, "body")?, out)?;
    out.push_str(suffix);
    Ok(())
}

fn render_math_mid(node: &MathNode, out: &mut String) -> Result<()> {
    out.push_str(r"\middle");
    render_math(math_child(node, "body")?, out)
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
        MathValue::Rows(_) => bail!("math row value cannot be rendered as a scalar expression"),
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

fn render_element_frame(node: &Inline, data: &InlineElementData, out: &mut String) -> Result<()> {
    let kind = inline_kind(node)?;
    let Some(frame) = data.inlines("frame").and_then(single_frame) else {
        bail!("typlite markdown {kind} rendering requires html.frame");
    };
    render_frame_image(kind, frame, out)
}

fn single_frame(inlines: &[Inline]) -> Option<&FrameImage> {
    match inlines {
        [Inline::Frame(frame)] => Some(frame),
        _ => None,
    }
}

fn render_frame_image(alt: &str, frame: &FrameImage, out: &mut String) -> Result<()> {
    if frame.svg.contains("viewBox=\"0 0 0 0\"") {
        bail!("typlite markdown frame image for {alt} is empty");
    }

    out.push_str("<img alt=\"");
    push_html_escaped(alt, out);
    out.push_str("\" src=\"data:image/svg+xml;utf8,");
    push_url_escaped(&frame.svg, out);
    out.push_str("\">");

    Ok(())
}

fn render_image(data: &InlineElementData, out: &mut String) -> Result<()> {
    let source = image_source(data)?;
    if source.mime == Some("application/pdf") {
        render_pdf_image_frame(data, out)?;
        return Ok(());
    }

    out.push_str("![");
    if let Some(alt) = data.scalar("alt") {
        push_markdown_link_text_escaped(alt, out);
    }
    out.push_str("](");
    push_markdown_url(&source.url, out);
    out.push(')');

    Ok(())
}

fn render_image_html(data: &InlineElementData, out: &mut String) -> Result<()> {
    let source = image_source(data)?;
    if source.mime == Some("application/pdf") {
        render_pdf_image_frame(data, out)?;
        return Ok(());
    }

    out.push_str("<img alt=\"");
    if let Some(alt) = data.scalar("alt") {
        push_html_escaped(alt, out);
    }
    out.push_str("\" src=\"");
    push_html_escaped(&source.url, out);
    out.push_str("\">");

    Ok(())
}

fn render_pdf_image_frame(data: &InlineElementData, out: &mut String) -> Result<()> {
    let Some(frame) = data.inlines("frame").and_then(single_frame) else {
        bail!("typlite markdown PDF image rendering requires html.frame");
    };
    render_frame_image(data.scalar("alt").unwrap_or("PDF"), frame, out)
}

enum SourceValue {
    String(String),
    Bytes(Vec<u8>),
}

struct ImageSource {
    url: String,
    mime: Option<&'static str>,
}

fn image_source(data: &InlineElementData) -> Result<ImageSource> {
    match source_value(data, "image")? {
        SourceValue::String(source) => {
            let mime = image_source_path_mime(&source);
            Ok(ImageSource { url: source, mime })
        }
        SourceValue::Bytes(bytes) => {
            let mime = if let Some(format) = data
                .scalar("format")
                .filter(|format| !is_auto_or_none(format))
            {
                image_format_mime(format)?
            } else {
                ImageFormat::detect(&bytes)
                    .and_then(image_format_mime_detected)
                    .context_ut("typlite markdown image bytes source requires known image format")?
            };
            Ok(ImageSource {
                url: format!(
                    "data:{mime};base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(bytes)
                ),
                mime: Some(mime),
            })
        }
    }
}

fn source_value(data: &InlineElementData, element: &str) -> Result<SourceValue> {
    let Some(raw) = data.scalar("source").filter(|source| !source.is_empty()) else {
        bail!("typlite markdown {element} rendering requires source");
    };

    let value = serde_json::from_str::<serde_json::Value>(raw)
        .context_ut("typlite source must be encoded as JSON")?;
    let serde_json::Value::Object(mut value) = value else {
        bail!("typlite source must be encoded as an object, got {value}");
    };
    match value.remove("kind") {
        Some(serde_json::Value::String(kind)) if kind == "string" => {
            let Some(serde_json::Value::String(value)) = value.remove("value") else {
                bail!("typlite source string must contain string field `value`");
            };
            Ok(SourceValue::String(value))
        }
        Some(serde_json::Value::String(kind)) if kind == "path" => {
            let Some(serde_json::Value::String(path)) = value.remove("path") else {
                bail!("typlite source path must contain string field `path`");
            };
            Ok(SourceValue::String(path))
        }
        Some(serde_json::Value::String(kind)) if kind == "bytes" => {
            let Some(bytes) = value.remove("bytes") else {
                bail!("typlite source bytes must contain field `bytes`");
            };
            Ok(SourceValue::Bytes(decode_source_bytes(bytes)?))
        }
        Some(kind) => bail!("unsupported typlite source kind {kind}"),
        None => bail!("typlite source object must contain field `kind`"),
    }
}

fn image_format_mime(format: &str) -> Result<&'static str> {
    match format {
        "png" => Ok("image/png"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "gif" => Ok("image/gif"),
        "svg" => Ok("image/svg+xml"),
        "webp" => Ok("image/webp"),
        "pdf" => Ok("application/pdf"),
        value => bail!("typlite markdown image bytes source does not support format `{value}`"),
    }
}

fn image_source_path_mime(source: &str) -> Option<&'static str> {
    let source = source.split(['?', '#']).next().unwrap_or(source);
    let extension = source.rsplit_once('.').map(|(_, extension)| extension)?;
    match extension.to_ascii_lowercase().as_str() {
        "pdf" => Some("application/pdf"),
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "svg" | "svgz" => Some("image/svg+xml"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

fn image_format_mime_detected(format: ImageFormat) -> Option<&'static str> {
    match format {
        ImageFormat::Raster(RasterFormat::Exchange(exchange)) => match exchange {
            ExchangeFormat::Png => Some("image/png"),
            ExchangeFormat::Jpg => Some("image/jpeg"),
            ExchangeFormat::Gif => Some("image/gif"),
            ExchangeFormat::Webp => Some("image/webp"),
        },
        ImageFormat::Vector(vector) => match vector {
            VectorFormat::Svg => Some("image/svg+xml"),
            VectorFormat::Pdf => Some("application/pdf"),
        },
        ImageFormat::Raster(RasterFormat::Pixel(_)) => None,
    }
}

fn decode_source_bytes(bytes: serde_json::Value) -> Result<Vec<u8>> {
    let serde_json::Value::Array(values) = bytes else {
        bail!("typlite source bytes field `bytes` must be an array");
    };
    let mut bytes = Vec::with_capacity(values.len());
    for value in values {
        let Some(byte) = value.as_u64().and_then(|value| u8::try_from(value).ok()) else {
            bail!("typlite source bytes contains non-byte value {value}");
        };
        bytes.push(byte);
    }
    Ok(bytes)
}

fn render_box(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(data, bibliography, out, |data, out| {
        out.push_str("display: inline-block");
        push_optional_css_length(data, "width", "width", out);
        push_optional_css_length(data, "height", "height", out);
    })
}

fn render_repeat(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str("<span data-typlite-repeat=\"true\"");
    if let Some(gap) = data.scalar("gap").filter(|value| !is_auto_or_none(value)) {
        out.push_str(" data-gap=\"");
        push_html_escaped(gap, out);
        out.push('"');
    }
    if let Some(justify) = data.scalar("justify").filter(|value| !value.is_empty()) {
        out.push_str(" data-justify=\"");
        push_html_escaped(justify, out);
        out.push('"');
    }

    out.push_str(" style=\"display: inline-flex");
    if let Some(gap) = data.scalar("gap").filter(|value| !is_auto_or_none(value)) {
        out.push_str("; gap: ");
        push_css_length(gap, out);
    }
    if data.scalar("justify") == Some("true") {
        out.push_str("; justify-content: space-between");
    }
    out.push_str("\">");
    render_inlines_html(&data.body, bibliography, out)?;
    out.push_str("</span>");
    Ok(())
}

fn render_pdf_artifact(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str("<span data-typlite-pdf-artifact=\"");
    push_html_escaped(data.scalar("kind").unwrap_or("artifact"), out);
    out.push_str("\">");
    render_inlines_html(&data.body, bibliography, out)?;
    out.push_str("</span>");
    Ok(())
}

fn render_pad(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(data, bibliography, out, |data, out| {
        out.push_str("display: inline-block");
        push_optional_css_length(data, "left", "padding-left", out);
        push_optional_css_length(data, "top", "padding-top", out);
        push_optional_css_length(data, "right", "padding-right", out);
        push_optional_css_length(data, "bottom", "padding-bottom", out);
    })
}

fn render_move(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(data, bibliography, out, |data, out| {
        out.push_str("display: inline-block; transform: translate(");
        push_css_length(data.scalar("dx").unwrap_or("0pt"), out);
        out.push_str(", ");
        push_css_length(data.scalar("dy").unwrap_or("0pt"), out);
        out.push(')');
    })
}

fn render_place(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(data, bibliography, out, |data, out| {
        out.push_str("display: inline-block; position: relative");
        if let Some(dx) = data.scalar("dx").filter(|value| !is_auto_or_none(value)) {
            out.push_str("; left: ");
            push_css_length(dx, out);
        }
        if let Some(dy) = data.scalar("dy").filter(|value| !is_auto_or_none(value)) {
            out.push_str("; top: ");
            push_css_length(dy, out);
        }
    })
}

fn render_transform(
    data: &InlineElementData,
    function: &str,
    fields: &[&str],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(data, bibliography, out, |data, out| {
        out.push_str("display: inline-block; transform: ");
        out.push_str(function);
        out.push('(');
        for (index, field) in fields.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            push_html_escaped(data.scalar(field).unwrap_or("0deg"), out);
        }
        out.push(')');
    })
}

fn render_scale(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(data, bibliography, out, |data, out| {
        out.push_str("display: inline-block; transform: scale(");
        push_css_scale(
            data.scalar("x")
                .or_else(|| data.scalar("factor"))
                .unwrap_or("100%"),
            out,
        );
        out.push_str(", ");
        push_css_scale(
            data.scalar("y")
                .or_else(|| data.scalar("factor"))
                .unwrap_or("100%"),
            out,
        );
        out.push(')');
    })
}

fn render_layout_span(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
    push_style: impl FnOnce(&InlineElementData, &mut String),
) -> Result<()> {
    out.push_str("<span style=\"");
    push_style(data, out);
    out.push_str("\">");
    render_inlines_html(&data.body, bibliography, out)?;
    out.push_str("</span>");
    Ok(())
}

fn push_optional_css_length(
    data: &InlineElementData,
    field: &str,
    property: &str,
    out: &mut String,
) {
    if let Some(value) = data.scalar(field).filter(|value| !is_auto_or_none(value)) {
        out.push_str("; ");
        out.push_str(property);
        out.push_str(": ");
        push_css_length(value, out);
    }
}

fn push_optional_block_css_length(
    data: &BlockElementData,
    field: &str,
    property: &str,
    out: &mut String,
) {
    if let Some(value) = data.scalar(field).filter(|value| !is_auto_or_none(value)) {
        out.push_str("; ");
        out.push_str(property);
        out.push_str(": ");
        push_css_length(value, out);
    }
}

fn push_css_scale(value: &str, out: &mut String) {
    if let Some(percent) = value
        .strip_suffix('%')
        .and_then(|value| value.parse::<f64>().ok())
    {
        out.push_str(&(percent / 100.0).to_string());
    } else {
        push_html_escaped(value, out);
    }
}

fn is_auto_or_none(value: &str) -> bool {
    value.is_empty() || value == "auto" || value == "none"
}

fn render_cite(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(key) = data.scalar("key").or_else(|| data.scalar("label")) else {
        bail!("typlite markdown cite rendering requires key or label");
    };
    ensure_default_cite_field(data, "form", "normal")?;
    ensure_default_cite_field(data, "style", "auto")?;

    let key = key.trim_start_matches('<').trim_end_matches('>');
    if let Some(citation) = bibliography.citation(key) {
        let id = bibliography.next_citation_id(key);
        render_citation_link(&id, key, citation, out);
        return Ok(());
    }

    bail!("typlite markdown cite rendering requires rendered bibliography citation for `{key}`")
}

fn render_ref(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(target) = data.scalar("target").or_else(|| data.scalar("label")) else {
        bail!("typlite markdown ref rendering requires target or label");
    };
    if let Some(form) = data.scalar("form").filter(|form| *form != "normal") {
        bail!("typlite markdown ref rendering does not support form `{form}`");
    }

    let target = target.trim_start_matches('<').trim_end_matches('>');
    if let Some(citation) = bibliography.citation(target) {
        let id = bibliography.next_citation_id(target);
        render_citation_link(&id, target, citation, out);
        return Ok(());
    }

    if let Some(supplement) = data
        .inlines("supplement")
        .filter(|value| has_semantic_inlines(value))
    {
        render_ref_link(target, supplement, bibliography, out)?;
        return Ok(());
    }

    if let Some(element) = data.blocks("element").filter(|value| !value.is_empty()) {
        render_ref_element_link(target, element, bibliography, out)?;
        return Ok(());
    }

    bail!("typlite markdown ref rendering requires supplement or resolved element for `{target}`")
}

fn render_citation_link(id: &str, key: &str, citation: &str, out: &mut String) {
    out.push_str("<a id=\"");
    push_html_escaped(id, out);
    out.push_str("\" href=\"#ref-");
    push_html_escaped(key, out);
    out.push_str("\">");
    push_html_escaped(citation, out);
    out.push_str("</a>");
}

fn render_ref_link(
    target: &str,
    body: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push('[');
    render_inlines(body, bibliography, out)?;
    out.push_str("](#");
    out.push_str(target);
    out.push(')');
    Ok(())
}

fn render_ref_element_link(
    target: &str,
    blocks: &[Block],
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    match blocks {
        [Block::Heading { body, .. }] if has_semantic_inlines(body) => {
            render_ref_link(target, body, bibliography, out)
        }
        [Block::Figure { caption, .. }] if has_semantic_inlines(caption) => {
            render_ref_link(target, caption, bibliography, out)
        }
        [_] => render_ref_text_link(target, target, out),
        _ => bail!("typlite markdown ref rendering expected one resolved element for `{target}`"),
    }
}

fn render_ref_text_link(target: &str, text: &str, out: &mut String) -> Result<()> {
    out.push('[');
    push_markdown_link_text_escaped(text, out);
    out.push_str("](#");
    out.push_str(target);
    out.push(')');
    Ok(())
}

fn render_inline_quote(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let quoted = match data.scalar("quotes").unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        value => bail!("typlite markdown quote rendering does not support quotes `{value}`"),
    };

    if quoted {
        out.push('"');
    }
    render_inlines(&data.body, bibliography, out)?;
    if quoted {
        out.push('"');
    }

    if let Some(attribution) = data
        .inlines("attribution")
        .filter(|value| has_semantic_inlines(value))
    {
        out.push_str(" (");
        render_inlines(attribution, bibliography, out)?;
        out.push(')');
    }

    Ok(())
}

fn render_inline_quote_html(
    data: &InlineElementData,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let quoted = match data.scalar("quotes").unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        value => bail!("typlite HTML quote rendering does not support quotes `{value}`"),
    };

    if quoted {
        out.push_str("<q>");
    }
    render_inlines_html(&data.body, bibliography, out)?;
    if quoted {
        out.push_str("</q>");
    }

    if let Some(attribution) = data
        .inlines("attribution")
        .filter(|value| has_semantic_inlines(value))
    {
        out.push_str(" <cite>");
        render_inlines_html(attribution, bibliography, out)?;
        out.push_str("</cite>");
    }

    Ok(())
}

fn render_metadata(data: &InlineElementData, out: &mut String) {
    let Some(value) = data.scalar("value").filter(|value| !value.is_empty()) else {
        return;
    };

    out.push_str("<!-- typlite-metadata: ");
    push_html_comment_escaped(value, out);
    out.push_str(" -->");
}

fn render_smartquote(data: &InlineElementData, out: &mut String) -> Result<()> {
    out.push(smartquote_char(data)?);
    Ok(())
}

fn render_smartquote_html(data: &InlineElementData, out: &mut String) -> Result<()> {
    match smartquote_char(data)? {
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        _ => unreachable!(),
    }
    Ok(())
}

fn smartquote_char(data: &InlineElementData) -> Result<char> {
    match data.scalar("double").unwrap_or("true") {
        "true" => Ok('"'),
        "false" => Ok('\''),
        value => {
            bail!("typlite markdown smartquote rendering requires boolean double, got `{value}`")
        }
    }
}

fn has_semantic_inlines(value: &[Inline]) -> bool {
    !value.is_empty() && !is_auto_inlines(value) && !is_none_inlines(value)
}

fn is_auto_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(text)] if text.as_str() == "auto")
}

fn is_none_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(text)] if text.as_str() == "none")
}

fn ensure_default_cite_field(data: &InlineElementData, field: &str, default: &str) -> Result<()> {
    if let Some(value) = data.scalar(field).filter(|value| *value != default) {
        bail!("typlite markdown cite rendering does not support {field} `{value}`");
    }

    Ok(())
}

fn render_structured_inline(
    node: &Inline,
    data: &InlineElementData,
    out: &mut String,
) -> Result<()> {
    out.push_str(inline_kind(node)?);
    out.push('(');
    for (index, field) in data.fields.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(field.name);
        out.push_str(": ");
        render_field_value(&field.value, out)?;
    }
    out.push(')');

    Ok(())
}

fn render_field_value(value: &ElementFieldValue, out: &mut String) -> Result<()> {
    match value {
        ElementFieldValue::Scalar(value) => out.push_str(value),
        ElementFieldValue::Inlines(value) => {
            render_inlines(value, &BibliographyContext::default(), out)?
        }
        ElementFieldValue::Blocks(value) => {
            out.push_str(&render_blocks(value, 0, &BibliographyContext::default())?)
        }
    }

    Ok(())
}

fn inline_kind(node: &Inline) -> Result<&'static str> {
    let kind = match node {
        Inline::Box(_) => "box",
        Inline::Circle(_) => "circle",
        Inline::Cite(_) => "cite",
        Inline::Curve(_) => "curve",
        Inline::CurveClose(_) => "curve.close",
        Inline::CurveCubic(_) => "curve.cubic",
        Inline::CurveLine(_) => "curve.line",
        Inline::CurveMove(_) => "curve.move",
        Inline::CurveQuad(_) => "curve.quad",
        Inline::Document(_) => "document",
        Inline::Ellipse(_) => "ellipse",
        Inline::FigureCaption(_) => "figure.caption",
        Inline::Footnote(_) => "footnote",
        Inline::FootnoteEntry(_) => "footnote.entry",
        Inline::GridCell(_) => "grid.cell",
        Inline::GridFooter(_) => "grid.footer",
        Inline::GridHeader(_) => "grid.header",
        Inline::GridHline(_) => "grid.hline",
        Inline::GridVline(_) => "grid.vline",
        Inline::H(_) => "h",
        Inline::Hide(_) => "hide",
        Inline::Highlight(_) => "highlight",
        Inline::Image(_) => "image",
        Inline::Line(_) => "line",
        Inline::MathAccent(_) => "math.accent",
        Inline::MathAttach(_) => "math.attach",
        Inline::MathBinom(_) => "math.binom",
        Inline::MathCancel(_) => "math.cancel",
        Inline::MathCases(_) => "math.cases",
        Inline::MathClass(_) => "math.class",
        Inline::MathFrac(_) => "math.frac",
        Inline::MathLimits(_) => "math.limits",
        Inline::MathLr(_) => "math.lr",
        Inline::MathMat(_) => "math.mat",
        Inline::MathMid(_) => "math.mid",
        Inline::MathOp(_) => "math.op",
        Inline::MathOverbrace(_) => "math.overbrace",
        Inline::MathOverbracket(_) => "math.overbracket",
        Inline::MathOverline(_) => "math.overline",
        Inline::MathOverparen(_) => "math.overparen",
        Inline::MathOvershell(_) => "math.overshell",
        Inline::MathPrimes(_) => "math.primes",
        Inline::MathRoot(_) => "math.root",
        Inline::MathScripts(_) => "math.scripts",
        Inline::MathStretch(_) => "math.stretch",
        Inline::MathUnderbrace(_) => "math.underbrace",
        Inline::MathUnderbracket(_) => "math.underbracket",
        Inline::MathUnderline(_) => "math.underline",
        Inline::MathUnderparen(_) => "math.underparen",
        Inline::MathUndershell(_) => "math.undershell",
        Inline::MathVec(_) => "math.vec",
        Inline::Metadata(_) => "metadata",
        Inline::Move(_) => "move",
        Inline::OutlineEntry(_) => "outline.entry",
        Inline::Overline(_) => "overline",
        Inline::Pad(_) => "pad",
        Inline::Page(_) => "page",
        Inline::ParLine(_) => "par.line",
        Inline::Path(_) => "path",
        Inline::PdfArtifact(_) => "pdf.artifact",
        Inline::PdfAttach(_) => "pdf.attach",
        Inline::PdfEmbed(_) => "pdf.embed",
        Inline::Place(_) => "place",
        Inline::PlaceFlush(_) => "place.flush",
        Inline::Polygon(_) => "polygon",
        Inline::Quote(_) => "quote",
        Inline::RawLine(_) => "raw.line",
        Inline::Rect(_) => "rect",
        Inline::Ref(_) => "ref",
        Inline::Repeat(_) => "repeat",
        Inline::Rotate(_) => "rotate",
        Inline::Scale(_) => "scale",
        Inline::Skew(_) => "skew",
        Inline::Smallcaps(_) => "smallcaps",
        Inline::Smartquote(_) => "smartquote",
        Inline::Square(_) => "square",
        Inline::TableCell(_) => "table.cell",
        Inline::TableFooter(_) => "table.footer",
        Inline::TableHeader(_) => "table.header",
        Inline::TableHline(_) => "table.hline",
        Inline::TableVline(_) => "table.vline",
        Inline::Underline(_) => "underline",
        Inline::Text(_)
        | Inline::Emph(_)
        | Inline::Strong(_)
        | Inline::Link { .. }
        | Inline::Strike(_)
        | Inline::Sub(_)
        | Inline::Super(_)
        | Inline::Math(_)
        | Inline::Linebreak
        | Inline::Frame(_)
        | Inline::Raw { .. } => bail!("not a generated inline element"),
    };

    Ok(kind)
}

fn push_html_escaped(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
}

fn push_html_comment_escaped(value: &str, out: &mut String) {
    let mut prev_was_hyphen = false;

    for ch in value.chars() {
        if prev_was_hyphen && ch == '-' {
            out.push(' ');
        }
        out.push(ch);
        prev_was_hyphen = ch == '-';
    }

    if prev_was_hyphen {
        out.push(' ');
    }
}

fn push_markdown_link_text_escaped(value: &str, out: &mut String) {
    for ch in value.chars() {
        match ch {
            '\\' | '[' | ']' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
}

fn push_markdown_url(value: &str, out: &mut String) {
    if value.contains(char::is_whitespace) || value.contains(')') {
        out.push('<');
        for ch in value.chars() {
            if ch == '>' {
                out.push_str("%3E");
            } else {
                out.push(ch);
            }
        }
        out.push('>');
    } else {
        out.push_str(value);
    }
}

fn push_url_escaped(value: &str, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
}
