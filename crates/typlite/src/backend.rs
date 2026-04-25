//! Experimental backends for typlite IR.

use crate::Result;
use crate::ir::*;
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
            Block::Heading(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Block::Paragraph(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Block::Quote(data) => collect_reference_anchors(&data.body, out),
            Block::Figure(data) => {
                collect_reference_anchors(&data.body, out);
                collect_reference_anchors_in_inlines(&data.caption, out);
            }
            Block::Align(data) => collect_reference_anchors(&data.body, out),
            Block::Table(data) => {
                for row in &data.rows {
                    for cell in &row.cells {
                        collect_reference_anchors_in_inlines(&cell.body, out);
                    }
                }
            }
            Block::List(data) => {
                for item in &data.items {
                    collect_reference_anchors(&item.body, out);
                }
            }
            Block::Terms(data) => {
                for item in &data.items {
                    collect_reference_anchors_in_inlines(&item.term, out);
                    collect_reference_anchors(&item.description, out);
                }
            }
            Block::Block(data) => collect_reference_anchors(&data.body, out),
            Block::Columns(data) => collect_reference_anchors(&data.body, out),
            Block::Move(data) => collect_reference_anchors(&data.body, out),
            Block::Pad(data) => collect_reference_anchors(&data.body, out),
            Block::Rotate(data) => collect_reference_anchors(&data.body, out),
            Block::Scale(data) => collect_reference_anchors(&data.body, out),
            Block::Skew(data) => collect_reference_anchors(&data.body, out),
            Block::Stack(data) => collect_reference_anchors(&data.children, out),
            Block::Title(data) => collect_reference_anchors(&data.body, out),
            Block::Math(_) | Block::Raw(_) => {}
            Block::Bibliography(_)
            | Block::Colbreak(_)
            | Block::Outline(_)
            | Block::Pagebreak(_)
            | Block::Parbreak(_)
            | Block::V(_) => {}
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
                if let Some(target) = data.target.as_deref() {
                    if let Some(element) = data.element.first() {
                        push_reference_anchor(
                            out,
                            reference_anchor_key(element),
                            normalized_label(target).into(),
                        );
                    }
                }
                collect_reference_anchors_in_inlines(&data.supplement, out);
                collect_reference_anchors_in_inlines(&data.citation, out);
                collect_reference_anchors(&data.element, out);
            }
            Inline::Emph(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Strong(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Strike(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Sub(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Super(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Link(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Text(_)
            | Inline::Math(_)
            | Inline::Linebreak(_)
            | Inline::Frame(_)
            | Inline::Raw(_) => {}
            Inline::Box(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Circle(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Curve(data) => collect_reference_anchors_in_inlines(&data.components, out),
            Inline::Ellipse(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::FigureCaption(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Footnote(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::GridCell(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::GridFooter(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::GridHeader(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::Hide(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Highlight(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::MathCases(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::MathVec(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::Move(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Overline(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Pad(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Page(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::PdfArtifact(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Place(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Quote(data) => {
                collect_reference_anchors_in_inlines(&data.attribution, out);
                collect_reference_anchors_in_inlines(&data.body, out);
            }
            Inline::RawLine(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Repeat(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Rotate(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Scale(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Skew(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Smallcaps(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::TableCell(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::TableFooter(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::TableHeader(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::Underline(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Cite(_)
            | Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_)
            | Inline::Document(_)
            | Inline::FootnoteEntry(_)
            | Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::H(_)
            | Inline::Image(_)
            | Inline::Line(_)
            | Inline::MathAccent(_)
            | Inline::MathAttach(_)
            | Inline::MathBinom(_)
            | Inline::MathCancel(_)
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
            | Inline::Metadata(_)
            | Inline::OutlineEntry(_)
            | Inline::ParLine(_)
            | Inline::Path(_)
            | Inline::PdfAttach(_)
            | Inline::PdfEmbed(_)
            | Inline::PlaceFlush(_)
            | Inline::Polygon(_)
            | Inline::Rect(_)
            | Inline::Smartquote(_)
            | Inline::Square(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
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

fn render_blocks_compact_into(
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
            out.push('\n');
        }
        rendered_count += 1;
        out.push_str(rendered.trim_end_matches([' ', '\t']));
    }

    Ok(())
}

fn render_block(
    block: &Block,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_reference_anchors(bibliography.take_reference_anchors(block), indent, out);
    match block {
        Block::Heading(data) => {
            if let Some(id) = &data.id {
                out.push_str(&" ".repeat(indent));
                out.push_str("<a id=\"");
                push_html_escaped(id, out);
                out.push_str("\"></a>\n");
            }
            out.push_str(&" ".repeat(indent));
            out.push_str(&"#".repeat(data.level as usize));
            out.push(' ');
            render_inlines(&data.body, bibliography, out)?;
        }
        Block::Paragraph(data) => {
            out.push_str(&" ".repeat(indent));
            render_inlines(&data.body, bibliography, out)?;
        }
        Block::Quote(data) => render_quote(&data.body, indent, bibliography, out)?,
        Block::Figure(data) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<figure");
            if let Some(alt) = &data.alt {
                out.push_str(" aria-label=\"");
                push_html_escaped(alt, out);
                out.push('"');
            }
            out.push_str(">\n");
            render_blocks_html_into(&data.body, indent, bibliography, out)?;
            if !data.caption.is_empty() {
                out.push('\n');
                out.push_str(&" ".repeat(indent));
                out.push_str("<figcaption>");
                render_inlines_html(&data.caption, bibliography, out)?;
                out.push_str("</figcaption>");
            }
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            out.push_str("</figure>");
        }
        Block::Align(data) => render_align(
            data.alignment.as_deref(),
            &data.body,
            indent,
            bibliography,
            out,
        )?,
        Block::Math(data) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("$$");
            render_math(&data.body, out)?;
            out.push_str("$$");
        }
        Block::Table(data) => {
            render_table(&data.rows, &data.alignments, indent, bibliography, out)?
        }
        Block::Raw(data) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
            if let Some(lang) = &data.lang {
                out.push_str(lang);
            }
            out.push('\n');
            out.push_str(&data.text);
            if !data.text.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&" ".repeat(indent));
            out.push_str("```");
        }
        Block::List(data) => render_list(
            data.ordered,
            data.start,
            data.reversed,
            &data.items,
            indent,
            bibliography,
            out,
        )?,
        Block::Columns(data) => render_columns(data, indent, bibliography, out)?,
        Block::Move(data) => render_move_block(data, indent, bibliography, out)?,
        Block::Pad(data) => render_pad_block(data, indent, bibliography, out)?,
        Block::Rotate(data) => render_rotate_block(data, indent, bibliography, out)?,
        Block::Scale(data) => render_scale_block(data, indent, bibliography, out)?,
        Block::Skew(data) => render_skew_block(data, indent, bibliography, out)?,
        Block::Stack(data) => render_stack(data, indent, bibliography, out)?,
        Block::Block(data) => render_blocks_compact_into(&data.body, indent, bibliography, out)?,
        Block::Title(data) => render_blocks_into(&data.body, indent, bibliography, out)?,
        Block::Terms(data) => render_terms(&data.items, indent, bibliography, out)?,
        Block::Colbreak(_) => {
            out.push_str(&" ".repeat(indent));
            out.push_str("<div style=\"break-after: column\"></div>");
        }
        Block::V(data) => render_vertical_space(data, indent, out)?,
        Block::Parbreak(_) => {}
        Block::Outline(data) => {
            if !is_auto_inlines(&data.title)
                && !is_none_inlines(&data.title)
                && !data.title.is_empty()
            {
                out.push_str(&" ".repeat(indent));
                render_inlines(&data.title, bibliography, out)?;
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
        Block::Paragraph(data) => {
            out.push_str(&" ".repeat(indent));
            render_inlines_html(&data.body, bibliography, out)
        }
        Block::Raw(data) => {
            out.push_str(&" ".repeat(indent));
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
    data: &ColumnsBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"");
    let mut has_style = false;
    if let Some(count) = data.count.as_deref().filter(|value| !value.is_empty()) {
        has_style = true;
        out.push_str("column-count: ");
        push_html_escaped(count, out);
    }
    if let Some(gutter) = data.gutter.as_deref().filter(|value| !value.is_empty()) {
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
    data: &StackBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"display: flex");
    if let Some(direction) = data.dir.as_deref().and_then(css_stack_direction) {
        out.push_str("; flex-direction: ");
        out.push_str(direction);
    }
    if let Some(spacing) = data.spacing.as_deref().filter(|value| !value.is_empty()) {
        out.push_str("; gap: ");
        push_css_length(spacing, out);
    }
    out.push_str("\">\n");
    render_blocks_html_into(&data.children, indent + 2, bibliography, out)?;
    out.push('\n');
    out.push_str(&" ".repeat(indent));
    out.push_str("</div>");

    Ok(())
}

fn render_vertical_space(data: &VBlock, indent: usize, out: &mut String) -> Result<()> {
    let Some(amount) = data.amount.as_deref().filter(|value| !value.is_empty()) else {
        return Ok(());
    };

    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"height: ");
    push_css_length(amount, out);
    out.push_str("\"></div>");

    Ok(())
}

fn render_bibliography(
    data: &BibliographyBlock,
    bibliography: &BibliographyContext,
    indent: usize,
    out: &mut String,
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<section data-typlite-bibliography=\"true\">");
    if let Some(title) = bibliography_title(data, bibliography)? {
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

fn bibliography_title(
    data: &BibliographyBlock,
    bibliography: &BibliographyContext,
) -> Result<Option<String>> {
    if data.title.is_empty() {
        return Ok(None);
    }

    let mut rendered = String::new();
    render_inlines(&data.title, bibliography, &mut rendered)?;
    if rendered.is_empty() || rendered == "auto" || rendered == "none" {
        Ok(None)
    } else {
        Ok(Some(rendered))
    }
}

fn render_pad_block(
    data: &PadBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block");
        push_optional_css_length_value(data.left.as_deref(), "padding-left", out);
        push_optional_css_length_value(data.top.as_deref(), "padding-top", out);
        push_optional_css_length_value(data.right.as_deref(), "padding-right", out);
        push_optional_css_length_value(data.bottom.as_deref(), "padding-bottom", out);
    })
}

fn render_move_block(
    data: &MoveBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: translate(");
        push_css_length(data.dx.as_deref().unwrap_or("0pt"), out);
        out.push_str(", ");
        push_css_length(data.dy.as_deref().unwrap_or("0pt"), out);
        out.push(')');
    })
}

fn render_rotate_block(
    data: &RotateBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: rotate(");
        push_html_escaped(data.angle.as_deref().unwrap_or("0deg"), out);
        out.push(')');
    })
}

fn render_scale_block(
    data: &ScaleBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: scale(");
        push_css_scale(
            data.x
                .as_deref()
                .or(data.factor.as_deref())
                .unwrap_or("100%"),
            out,
        );
        out.push_str(", ");
        push_css_scale(
            data.y
                .as_deref()
                .or(data.factor.as_deref())
                .unwrap_or("100%"),
            out,
        );
        out.push(')');
    })
}

fn render_skew_block(
    data: &SkewBlock,
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_div(&data.body, indent, bibliography, out, |out| {
        out.push_str("display: block; transform: skew(");
        push_html_escaped(data.ax.as_deref().unwrap_or("0deg"), out);
        out.push_str(", ");
        push_html_escaped(data.ay.as_deref().unwrap_or("0deg"), out);
        out.push(')');
    })
}

fn render_layout_div(
    body: &[Block],
    indent: usize,
    bibliography: &BibliographyContext,
    out: &mut String,
    push_style: impl FnOnce(&mut String),
) -> Result<()> {
    out.push_str(&" ".repeat(indent));
    out.push_str("<div style=\"");
    push_style(out);
    out.push_str("\">\n");
    render_blocks_html_into(body, indent + 2, bibliography, out)?;
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
            Inline::Text(data) => out.push_str(&data.text),
            Inline::Emph(data) => {
                out.push('*');
                render_inlines(&data.body, bibliography, out)?;
                out.push('*');
            }
            Inline::Strong(data) => {
                out.push_str("**");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("**");
            }
            Inline::Link(data) => {
                out.push('[');
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("](");
                out.push_str(&data.dest);
                out.push(')');
            }
            Inline::Strike(data) => {
                out.push_str("~~");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("~~");
            }
            Inline::Sub(data) => {
                out.push_str("<sub>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(data) => {
                out.push_str("<sup>");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Math(data) => {
                out.push('$');
                render_math(&data.body, out)?;
                out.push('$');
            }
            Inline::Linebreak(_) => out.push_str("  \n"),
            Inline::Frame(data) => render_frame_image("frame", &data.image, out)?,
            Inline::Raw(data) => {
                out.push('`');
                out.push_str(&data.text);
                out.push('`');
            }
            Inline::Repeat(data) => render_repeat(data, bibliography, out)?,
            Inline::TableCell(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::TableFooter(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::TableHeader(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::GridCell(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::GridFooter(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::GridHeader(data) => render_inlines(&data.children, bibliography, out)?,
            Inline::ParLine(_) => {}
            Inline::RawLine(data) => render_inlines(&data.body, bibliography, out)?,
            Inline::PdfArtifact(data) => render_pdf_artifact(data, bibliography, out)?,
            Inline::Box(data) => render_box(data, bibliography, out)?,
            Inline::Move(data) => render_move(data, bibliography, out)?,
            Inline::Pad(data) => render_pad(data, bibliography, out)?,
            Inline::Place(data) => render_place(data, bibliography, out)?,
            Inline::Rotate(data) => render_rotate(data, bibliography, out)?,
            Inline::Scale(data) => render_scale(data, bibliography, out)?,
            Inline::Skew(data) => render_skew(data, bibliography, out)?,
            Inline::Quote(data) => render_inline_quote(data, bibliography, out)?,
            Inline::Circle(data) => render_element_frame("circle", data.frame.as_ref(), out)?,
            Inline::Curve(data) => render_element_frame("curve", data.frame.as_ref(), out)?,
            Inline::Ellipse(data) => render_element_frame("ellipse", data.frame.as_ref(), out)?,
            Inline::Line(data) => render_element_frame("line", data.frame.as_ref(), out)?,
            Inline::Path(data) => render_element_frame("path", data.frame.as_ref(), out)?,
            Inline::Polygon(data) => render_element_frame("polygon", data.frame.as_ref(), out)?,
            Inline::Rect(data) => render_element_frame("rect", data.frame.as_ref(), out)?,
            Inline::Square(data) => render_element_frame("square", data.frame.as_ref(), out)?,
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
            | Inline::MathVec(_) => render_math_inline(node, true, out)?,
            Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_) => render_curve_component_warning(node, out)?,
            Inline::OutlineEntry(_) => {}
            Inline::Overline(data) => {
                out.push_str("<span style=\"text-decoration: overline\">");
                render_inlines(&data.body, bibliography, out)?;
                out.push_str("</span>");
            }
            Inline::PdfAttach(data) => render_pdf_embedding(data.path.as_deref(), out),
            Inline::PdfEmbed(data) => render_pdf_embedding(data.path.as_deref(), out),
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
            Inline::Text(data) => push_html_escaped(&data.text, out),
            Inline::Emph(data) => {
                out.push_str("<em>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</em>");
            }
            Inline::Strong(data) => {
                out.push_str("<strong>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</strong>");
            }
            Inline::Link(data) => {
                out.push_str("<a href=\"");
                push_html_escaped(&data.dest, out);
                out.push_str("\">");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</a>");
            }
            Inline::Strike(data) => {
                out.push_str("<del>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</del>");
            }
            Inline::Sub(data) => {
                out.push_str("<sub>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</sub>");
            }
            Inline::Super(data) => {
                out.push_str("<sup>");
                render_inlines_html(&data.body, bibliography, out)?;
                out.push_str("</sup>");
            }
            Inline::Raw(data) => {
                out.push_str("<code>");
                push_html_escaped(&data.text, out);
                out.push_str("</code>");
            }
            Inline::Linebreak(_) => out.push_str("<br>"),
            Inline::Math(data) => {
                out.push('$');
                render_math(&data.body, out)?;
                out.push('$');
            }
            Inline::Frame(data) => render_frame_image("frame", &data.image, out)?,
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
            Inline::TableCell(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::TableFooter(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::TableHeader(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::GridCell(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::GridFooter(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::GridHeader(data) => render_inlines_html(&data.children, bibliography, out)?,
            Inline::ParLine(_) => {}
            Inline::RawLine(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::FigureCaption(data) => render_inlines_html(&data.body, bibliography, out)?,
            Inline::OutlineEntry(_) => {}
            Inline::PdfArtifact(data) => render_pdf_artifact(data, bibliography, out)?,
            Inline::Box(data) => render_box(data, bibliography, out)?,
            Inline::Move(data) => render_move(data, bibliography, out)?,
            Inline::Pad(data) => render_pad(data, bibliography, out)?,
            Inline::Place(data) => render_place(data, bibliography, out)?,
            Inline::Rotate(data) => render_rotate(data, bibliography, out)?,
            Inline::Scale(data) => render_scale(data, bibliography, out)?,
            Inline::Skew(data) => render_skew(data, bibliography, out)?,
            Inline::Quote(data) => render_inline_quote_html(data, bibliography, out)?,
            Inline::FootnoteEntry(_) => {}
            Inline::Circle(data) => render_element_frame("circle", data.frame.as_ref(), out)?,
            Inline::Curve(data) => render_element_frame("curve", data.frame.as_ref(), out)?,
            Inline::Ellipse(data) => render_element_frame("ellipse", data.frame.as_ref(), out)?,
            Inline::Line(data) => render_element_frame("line", data.frame.as_ref(), out)?,
            Inline::Path(data) => render_element_frame("path", data.frame.as_ref(), out)?,
            Inline::Polygon(data) => render_element_frame("polygon", data.frame.as_ref(), out)?,
            Inline::Rect(data) => render_element_frame("rect", data.frame.as_ref(), out)?,
            Inline::Square(data) => render_element_frame("square", data.frame.as_ref(), out)?,
            Inline::Cite(data) => render_cite(data, bibliography, out)?,
            Inline::Metadata(data) => render_metadata(data, out),
            Inline::Document(_) | Inline::Hide(_) | Inline::Page(_) => {}
            Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::PlaceFlush(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
            Inline::H(_) => out.push(' '),
            Inline::PdfAttach(data) => render_pdf_embedding(data.path.as_deref(), out),
            Inline::PdfEmbed(data) => render_pdf_embedding(data.path.as_deref(), out),
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
            | Inline::MathVec(_) => render_math_inline(node, true, out)?,
            Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_) => render_curve_component_warning(node, out)?,
        }
    }

    Ok(())
}

fn render_math_inline(node: &Inline, inline: bool, out: &mut String) -> Result<()> {
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

fn render_element_frame(kind: &str, frame: Option<&FrameImage>, out: &mut String) -> Result<()> {
    let Some(frame) = frame else {
        bail!("typlite markdown {kind} rendering requires html.frame");
    };
    render_frame_image(kind, frame, out)
}

fn render_frame_image(alt: &str, frame: &FrameImage, out: &mut String) -> Result<()> {
    if frame.svg.contains("viewBox=\"0 0 0 0\"") {
        out.push_str("<!-- typlite-empty-frame: ");
        push_html_comment_escaped(alt, out);
        out.push_str(" -->");
        return Ok(());
    }

    out.push_str("<img alt=\"");
    push_html_escaped(alt, out);
    out.push_str("\" src=\"data:image/svg+xml;utf8,");
    push_url_escaped(&frame.svg, out);
    out.push_str("\">");

    Ok(())
}

fn render_pdf_embedding(path: Option<&str>, out: &mut String) {
    out.push_str("<!-- typlite-pdf");
    if let Some(path) = path.filter(|value| !value.is_empty()) {
        out.push_str(": ");
        push_html_comment_escaped(path, out);
    }
    out.push_str(" -->");
}

fn render_image(data: &ImageInline, out: &mut String) -> Result<()> {
    let source = image_source(data)?;
    if source.mime == Some("application/pdf") {
        render_pdf_image_frame(data, out)?;
        return Ok(());
    }

    out.push_str("![");
    if let Some(alt) = data.alt.as_deref() {
        push_markdown_link_text_escaped(alt, out);
    }
    out.push_str("](");
    push_markdown_url(&source.url, out);
    out.push(')');

    Ok(())
}

fn render_image_html(data: &ImageInline, out: &mut String) -> Result<()> {
    let source = image_source(data)?;
    if source.mime == Some("application/pdf") {
        render_pdf_image_frame(data, out)?;
        return Ok(());
    }

    out.push_str("<img alt=\"");
    if let Some(alt) = data.alt.as_deref() {
        push_html_escaped(alt, out);
    }
    out.push_str("\" src=\"");
    push_html_escaped(&source.url, out);
    out.push_str("\">");

    Ok(())
}

fn render_pdf_image_frame(data: &ImageInline, out: &mut String) -> Result<()> {
    let Some(frame) = data.frame.as_ref() else {
        bail!("typlite markdown PDF image rendering requires html.frame");
    };
    render_frame_image(data.alt.as_deref().unwrap_or("PDF"), frame, out)
}

enum SourceValue {
    String(String),
    Bytes(Vec<u8>),
}

struct ImageSource {
    url: String,
    mime: Option<&'static str>,
}

fn image_source(data: &ImageInline) -> Result<ImageSource> {
    match source_value(data, "image")? {
        SourceValue::String(source) => {
            let mime = image_source_path_mime(&source);
            Ok(ImageSource { url: source, mime })
        }
        SourceValue::Bytes(bytes) => {
            let mime = if let Some(format) = data
                .format
                .as_deref()
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

fn source_value(data: &ImageInline, element: &str) -> Result<SourceValue> {
    let Some(raw) = data.source.as_deref().filter(|source| !source.is_empty()) else {
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
    data: &BoxInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(&data.body, bibliography, out, |out| {
        out.push_str("display: inline-block");
        push_optional_css_length_value(data.width.as_deref(), "width", out);
        push_optional_css_length_value(data.height.as_deref(), "height", out);
    })
}

fn render_repeat(
    data: &RepeatInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str("<span data-typlite-repeat=\"true\"");
    if let Some(gap) = data.gap.as_deref().filter(|value| !is_auto_or_none(value)) {
        out.push_str(" data-gap=\"");
        push_html_escaped(gap, out);
        out.push('"');
    }
    if let Some(justify) = data.justify.as_deref().filter(|value| !value.is_empty()) {
        out.push_str(" data-justify=\"");
        push_html_escaped(justify, out);
        out.push('"');
    }

    out.push_str(" style=\"display: inline-flex");
    if let Some(gap) = data.gap.as_deref().filter(|value| !is_auto_or_none(value)) {
        out.push_str("; gap: ");
        push_css_length(gap, out);
    }
    if data.justify.as_deref() == Some("true") {
        out.push_str("; justify-content: space-between");
    }
    out.push_str("\">");
    render_inlines_html(&data.body, bibliography, out)?;
    out.push_str("</span>");
    Ok(())
}

fn render_pdf_artifact(
    data: &PdfArtifactInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    out.push_str("<span data-typlite-pdf-artifact=\"");
    push_html_escaped(data.kind.as_deref().unwrap_or("artifact"), out);
    out.push_str("\">");
    render_inlines_html(&data.body, bibliography, out)?;
    out.push_str("</span>");
    Ok(())
}

fn render_pad(
    data: &PadInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(&data.body, bibliography, out, |out| {
        out.push_str("display: inline-block");
        push_optional_css_length_value(data.left.as_deref(), "padding-left", out);
        push_optional_css_length_value(data.top.as_deref(), "padding-top", out);
        push_optional_css_length_value(data.right.as_deref(), "padding-right", out);
        push_optional_css_length_value(data.bottom.as_deref(), "padding-bottom", out);
    })
}

fn render_move(
    data: &MoveInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(&data.body, bibliography, out, |out| {
        out.push_str("display: inline-block; transform: translate(");
        push_css_length(data.dx.as_deref().unwrap_or("0pt"), out);
        out.push_str(", ");
        push_css_length(data.dy.as_deref().unwrap_or("0pt"), out);
        out.push(')');
    })
}

fn render_place(
    data: &PlaceInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(&data.body, bibliography, out, |out| {
        out.push_str("display: inline-block; position: relative");
        if let Some(dx) = data.dx.as_deref().filter(|value| !is_auto_or_none(value)) {
            out.push_str("; left: ");
            push_css_length(dx, out);
        }
        if let Some(dy) = data.dy.as_deref().filter(|value| !is_auto_or_none(value)) {
            out.push_str("; top: ");
            push_css_length(dy, out);
        }
    })
}

fn render_rotate(
    data: &RotateInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(&data.body, bibliography, out, |out| {
        out.push_str("display: inline-block; transform: rotate(");
        push_html_escaped(data.angle.as_deref().unwrap_or("0deg"), out);
        out.push(')');
    })
}

fn render_scale(
    data: &ScaleInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(&data.body, bibliography, out, |out| {
        out.push_str("display: inline-block; transform: scale(");
        push_css_scale(
            data.x
                .as_deref()
                .or(data.factor.as_deref())
                .unwrap_or("100%"),
            out,
        );
        out.push_str(", ");
        push_css_scale(
            data.y
                .as_deref()
                .or(data.factor.as_deref())
                .unwrap_or("100%"),
            out,
        );
        out.push(')');
    })
}

fn render_skew(
    data: &SkewInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    render_layout_span(&data.body, bibliography, out, |out| {
        out.push_str("display: inline-block; transform: skew(");
        push_html_escaped(data.ax.as_deref().unwrap_or("0deg"), out);
        out.push_str(", ");
        push_html_escaped(data.ay.as_deref().unwrap_or("0deg"), out);
        out.push(')');
    })
}

fn render_layout_span(
    body: &[Inline],
    bibliography: &BibliographyContext,
    out: &mut String,
    push_style: impl FnOnce(&mut String),
) -> Result<()> {
    out.push_str("<span style=\"");
    push_style(out);
    out.push_str("\">");
    render_inlines_html(body, bibliography, out)?;
    out.push_str("</span>");
    Ok(())
}

fn push_optional_css_length_value(value: Option<&str>, property: &str, out: &mut String) {
    if let Some(value) = value.filter(|value| !is_auto_or_none(value)) {
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
    data: &CiteInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(key) = data.key.as_deref() else {
        return render_unimplemented("cite without key");
    };

    let key = key.trim_start_matches('<').trim_end_matches('>');
    if let Some(citation) = bibliography.citation(key) {
        let id = bibliography.next_citation_id(key);
        render_citation_link(&id, key, citation, out);
        return Ok(());
    }

    out.push_str("[@");
    push_markdown_link_text_escaped(key, out);
    out.push_str("](#ref-");
    out.push_str(key);
    out.push(')');
    Ok(())
}

fn render_ref(
    data: &RefInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let Some(target) = data.target.as_deref() else {
        return render_unimplemented("ref without target");
    };

    let target = target.trim_start_matches('<').trim_end_matches('>');
    if let Some(citation) = bibliography.citation(target) {
        let id = bibliography.next_citation_id(target);
        render_citation_link(&id, target, citation, out);
        return Ok(());
    }

    if has_semantic_inlines(&data.supplement) {
        let supplement = &data.supplement;
        render_ref_link(target, supplement, bibliography, out)?;
        return Ok(());
    }

    if !data.element.is_empty() {
        render_ref_element_link(target, &data.element, bibliography, out)?;
        return Ok(());
    }

    render_ref_text_link(target, target, out)
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
        [Block::Heading(data)] if has_semantic_inlines(&data.body) => {
            render_ref_link(target, &data.body, bibliography, out)
        }
        [Block::Figure(data)] if has_semantic_inlines(&data.caption) => {
            render_ref_link(target, &data.caption, bibliography, out)
        }
        [_] | [_, ..] => render_ref_text_link(target, target, out),
        [] => render_ref_text_link(target, target, out),
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
    data: &QuoteInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let quoted = match data.quotes.as_deref().unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        _ => true,
    };

    if quoted {
        out.push('"');
    }
    render_inlines(&data.body, bibliography, out)?;
    if quoted {
        out.push('"');
    }

    if has_semantic_inlines(&data.attribution) {
        out.push_str(" (");
        render_inlines(&data.attribution, bibliography, out)?;
        out.push(')');
    }

    Ok(())
}

fn render_inline_quote_html(
    data: &QuoteInline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let quoted = match data.quotes.as_deref().unwrap_or("auto") {
        "auto" | "true" => true,
        "false" => false,
        _ => true,
    };

    if quoted {
        out.push_str("<q>");
    }
    render_inlines_html(&data.body, bibliography, out)?;
    if quoted {
        out.push_str("</q>");
    }

    if has_semantic_inlines(&data.attribution) {
        out.push_str(" <cite>");
        render_inlines_html(&data.attribution, bibliography, out)?;
        out.push_str("</cite>");
    }

    Ok(())
}

fn render_metadata(data: &MetadataInline, out: &mut String) {
    let Some(value) = data.value.as_deref().filter(|value| !value.is_empty()) else {
        return;
    };

    out.push_str("<!-- typlite-metadata: ");
    push_html_comment_escaped(value, out);
    out.push_str(" -->");
}

fn render_smartquote(data: &SmartquoteInline, out: &mut String) -> Result<()> {
    out.push(smartquote_char(data)?);
    Ok(())
}

fn render_smartquote_html(data: &SmartquoteInline, out: &mut String) -> Result<()> {
    match smartquote_char(data)? {
        '"' => out.push_str("&quot;"),
        '\'' => out.push_str("&#39;"),
        _ => unreachable!(),
    }
    Ok(())
}

fn smartquote_char(data: &SmartquoteInline) -> Result<char> {
    match data.double.as_deref().unwrap_or("true") {
        "true" => Ok('"'),
        "false" => Ok('\''),
        _ => Ok('"'),
    }
}

fn has_semantic_inlines(value: &[Inline]) -> bool {
    !value.is_empty() && !is_auto_inlines(value) && !is_none_inlines(value)
}

fn is_auto_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(data)] if data.text.as_str() == "auto")
}

fn is_none_inlines(value: &[Inline]) -> bool {
    matches!(value, [Inline::Text(data)] if data.text.as_str() == "none")
}

fn render_unimplemented_inline(node: &Inline) -> Result<()> {
    render_unimplemented(inline_kind(node)?)
}

fn render_curve_component_warning(node: &Inline, out: &mut String) -> Result<()> {
    let kind = inline_kind(node)?;
    eprintln!(
        "warning: typlite markdown rendering for `{kind}` requires wrapping the parent curve in html.frame"
    );
    out.push_str("<!-- typlite-warning: ");
    push_html_comment_escaped(kind, out);
    out.push_str(" requires wrapping the parent curve in html.frame -->");
    Ok(())
}

fn render_unimplemented(feature: &str) -> Result<()> {
    bail!("typlite markdown rendering for `{feature}` is not implemented")
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
        | Inline::Link(_)
        | Inline::Strike(_)
        | Inline::Sub(_)
        | Inline::Super(_)
        | Inline::Math(_)
        | Inline::Linebreak(_)
        | Inline::Frame(_)
        | Inline::Raw(_) => bail!("not a generated inline element"),
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
