use crate::Result;
use crate::ir::*;
use base64::Engine;
use tinymist_std::error::prelude::*;
use typst::visualize::{ExchangeFormat, ImageFormat, RasterFormat, VectorFormat};

use super::{
    BibliographyContext, is_auto_or_none, push_css_length, push_css_scale,
    push_html_comment_escaped, push_html_escaped, push_markdown_link_text_escaped,
    push_markdown_url, push_optional_css_length_value, push_url_escaped, render_inlines_html,
};

pub(super) fn render_element_frame(
    kind: &str,
    frame: Option<&FrameImage>,
    out: &mut String,
) -> Result<()> {
    let Some(frame) = frame else {
        bail!("typlite markdown {kind} rendering requires html.frame");
    };
    render_frame_image(kind, frame, out)
}

pub(super) fn render_frame_image(alt: &str, frame: &FrameImage, out: &mut String) -> Result<()> {
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

pub(super) fn render_pdf_embedding(path: Option<&str>, out: &mut String) {
    out.push_str("<!-- typlite-pdf");
    if let Some(path) = path.filter(|value| !value.is_empty()) {
        out.push_str(": ");
        push_html_comment_escaped(path, out);
    }
    out.push_str(" -->");
}

pub(super) fn render_image(data: &ImageInline, out: &mut String) -> Result<()> {
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

pub(super) fn render_image_html(data: &ImageInline, out: &mut String) -> Result<()> {
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

pub(super) fn render_box(
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

pub(super) fn render_repeat(
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

pub(super) fn render_pdf_artifact(
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

pub(super) fn render_pad(
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

pub(super) fn render_move(
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

pub(super) fn render_place(
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

pub(super) fn render_rotate(
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

pub(super) fn render_scale(
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

pub(super) fn render_skew(
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
