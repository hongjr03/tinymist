//! Experimental extraction from Typst HTML custom elements into typlite IR.

use typst::introspection::Introspector;
use typst_html::{HtmlDocument, HtmlElement, HtmlNode};

use crate::Result;
use crate::ir::*;

mod block;
mod collections;
mod fields;
mod inline;
mod text;
mod traversal;
use self::block::block_from_element;
use self::collections::*;
use self::fields::*;
use self::inline::{coalesce_raw_inlines, collect_link_body, inline_from_element_kind};
use self::text::plain_text_blocks;
use self::traversal::{collect_field_blocks, collect_item_blocks, inline_has_content};

/// Extracts typlite IR nodes from an HTML document root.
pub fn extract_document(html: &HtmlDocument) -> Result<Document> {
    let mut blocks = Vec::new();
    collect_blocks(&html.root, &html.introspector, &mut blocks)?;
    Ok(Document { blocks })
}

fn collect_blocks(
    element: &HtmlElement,
    introspector: &Introspector,
    blocks: &mut Vec<Block>,
) -> Result<()> {
    if let Some(block) = block_from_element(element, introspector)? {
        blocks.push(block);
        return Ok(());
    }

    if is_field(element) {
        return Ok(());
    }

    for child in &element.children {
        if let HtmlNode::Element(child) = child {
            collect_blocks(child, introspector, blocks)?;
        }
    }

    Ok(())
}

fn collect_inlines(nodes: &[HtmlNode], introspector: &Introspector) -> Result<Vec<Inline>> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push(Inline::Text(TextInline { text: text.clone() })),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                if tag_name(element).as_deref() == Some("a") {
                    let body = collect_link_body(element, introspector)?;
                    out.push(Inline::Link(LinkInline {
                        dest: attr(element, "href").unwrap_or_default(),
                        body,
                    }));
                    continue;
                }

                match attr(element, "data-typlite").as_deref() {
                    Some("emph") => out.push(Inline::Emph(EmphInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("strong") => out.push(Inline::Strong(StrongInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("link") => out.push(Inline::Link(LinkInline {
                        dest: field_value(element, "dest").unwrap_or_default(),
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("strike") => out.push(Inline::Strike(StrikeInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("sub") => out.push(Inline::Sub(SubInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("super") => out.push(Inline::Super(SuperInline {
                        body: field_children(element, "body")
                            .map(|children| collect_inlines(children, introspector))
                            .transpose()?
                            .unwrap_or_default(),
                    })),
                    Some("math-equation") => out.push(Inline::Math(MathInline {
                        body: math_field(element, "body")?,
                    })),
                    Some("linebreak") => out.push(Inline::Linebreak(LinebreakInline {})),
                    Some("raw") => out.push(Inline::Raw(RawInline {
                        lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
                        text: raw_text(element),
                    })),
                    Some(kind) => {
                        if let Some(spec) = spec_by_kind(kind) {
                            if let Some(inline) =
                                inline_from_element_kind(element, spec.kind, introspector)?
                            {
                                out.push(inline);
                            }
                        }
                    }
                    None => {
                        out.extend(collect_inlines(&element.children, introspector)?);
                    }
                }
            }
            HtmlNode::Frame(frame) => out.push(frame_to_inline(frame, introspector)),
            HtmlNode::Tag(_) => {}
        }
    }

    Ok(coalesce_raw_inlines(out))
}
