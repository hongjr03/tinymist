//! Experimental backends for typlite IR.

use crate::Result;
use crate::ir::*;
use ecow::EcoString;
use tinymist_std::error::prelude::*;
use typst::diag::SourceDiagnostic;
use typst_syntax::Span;
mod context;
mod list;
mod math;
mod media;
mod table;
pub use self::context::{BibliographyContext, RenderedMarkdown};
use self::list::{render_list, render_terms};
use self::math::{render_math, render_math_inline};
use self::media::{
    render_box, render_element_frame, render_frame_image, render_image, render_image_html,
    render_move, render_pad, render_pdf_artifact, render_pdf_embedding, render_place,
    render_repeat, render_rotate, render_scale, render_skew,
};
use self::table::render_table;

/// Renders a document IR as Markdown.
pub fn render_markdown(doc: &Document) -> Result<String> {
    render_markdown_with_bibliography(doc, &BibliographyContext::default())
}

/// Renders a document IR as Markdown with a bibliography context.
pub fn render_markdown_with_bibliography(
    doc: &Document,
    bibliography: &BibliographyContext,
) -> Result<String> {
    Ok(render_markdown_with_diagnostics(doc, bibliography)?.output)
}

/// Renders a document IR as Markdown with diagnostics.
pub fn render_markdown_with_diagnostics(
    doc: &Document,
    bibliography: &BibliographyContext,
) -> Result<RenderedMarkdown> {
    bibliography.reset_render_state(doc);
    let output = render_blocks(&doc.blocks, 0, bibliography)?;
    let warnings = bibliography.take_warnings();
    Ok(RenderedMarkdown { output, warnings })
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
            | Inline::CurveQuad(_) => render_curve_component_warning(node, bibliography, out)?,
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
            | Inline::CurveQuad(_) => render_curve_component_warning(node, bibliography, out)?,
        }
    }

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

fn render_curve_component_warning(
    node: &Inline,
    bibliography: &BibliographyContext,
    out: &mut String,
) -> Result<()> {
    let kind = inline_kind(node)?;
    if let Some(span) = curve_component_span(node) {
        bibliography.warn(SourceDiagnostic::warning(
            span,
            format!(
                "typlite markdown rendering for `{kind}` requires wrapping the parent curve in html.frame"
            ),
        ));
    }
    out.push_str("<!-- typlite-warning: ");
    push_html_comment_escaped(kind, out);
    out.push_str(" requires wrapping the parent curve in html.frame -->");
    Ok(())
}

fn curve_component_span(node: &Inline) -> Option<Span> {
    match node {
        Inline::CurveClose(data) => data.span,
        Inline::CurveCubic(data) => data.span,
        Inline::CurveLine(data) => data.span,
        Inline::CurveMove(data) => data.span,
        Inline::CurveQuad(data) => data.span,
        _ => None,
    }
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
