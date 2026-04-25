//! Experimental extraction from Typst HTML custom elements into typlite IR.

use ecow::EcoString;
use typst_html::{HtmlElement, HtmlNode};

use crate::ir::{Block, Document, Inline};

/// Extracts typlite IR nodes from an HTML document root.
pub fn extract_document(root: &HtmlElement) -> Document {
    let mut blocks = Vec::new();
    collect_blocks(root, &mut blocks);
    Document { blocks }
}

fn collect_blocks(element: &HtmlElement, blocks: &mut Vec<Block>) {
    match tag_name(element).as_deref() {
        Some("typlite-heading") => {
            let level = field_value(element, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1);
            blocks.push(Block::Heading {
                level,
                body: field_children(element, "body")
                    .map(collect_inlines)
                    .unwrap_or_default(),
            });
        }
        Some("typlite-paragraph") => {
            blocks.push(Block::Paragraph(
                field_children(element, "body")
                    .map(collect_inlines)
                    .unwrap_or_default(),
            ));
        }
        Some("typlite-raw") => {
            blocks.push(Block::Raw {
                lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
                text: field_value(element, "text").unwrap_or_default(),
            });
        }
        _ => {
            for child in &element.children {
                if let HtmlNode::Element(child) = child {
                    collect_blocks(child, blocks);
                }
            }
        }
    }
}

fn collect_inlines(nodes: &[HtmlNode]) -> Vec<Inline> {
    let mut out = Vec::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push(Inline::Text(text.clone())),
            HtmlNode::Element(element) => {
                if is_field(element) {
                    continue;
                }

                match attr(element, "data-typlite").as_deref() {
                    Some("emph") => out.push(Inline::Emph(
                        field_children(element, "body")
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    )),
                    Some("strong") => out.push(Inline::Strong(
                        field_children(element, "body")
                            .map(collect_inlines)
                            .unwrap_or_default(),
                    )),
                    Some("raw") => out.push(Inline::Raw {
                        lang: field_value(element, "lang").filter(|lang| lang.as_str() != "none"),
                        text: field_value(element, "text").unwrap_or_default(),
                    }),
                    _ => out.extend(collect_inlines(&element.children)),
                }
            }
            HtmlNode::Tag(_) | HtmlNode::Frame(_) => {}
        }
    }

    out
}

fn collect_text(nodes: &[HtmlNode]) -> EcoString {
    let mut out = EcoString::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push_str(text),
            HtmlNode::Element(element) => {
                if !is_field(element) {
                    out.push_str(&collect_text(&element.children));
                }
            }
            HtmlNode::Tag(_) | HtmlNode::Frame(_) => {}
        }
    }

    out
}

fn tag_name(element: &HtmlElement) -> Option<String> {
    Some(element.tag.resolve().as_str().to_owned())
}

fn attr(element: &HtmlElement, name: &str) -> Option<EcoString> {
    element
        .attrs
        .0
        .iter()
        .find(|(attr, _)| attr.resolve().as_str() == name)
        .map(|(_, value)| value.clone())
}

fn field_value(element: &HtmlElement, name: &str) -> Option<EcoString> {
    field_element(element, name).map(|field| collect_text(&field.children))
}

fn field_children<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a [HtmlNode]> {
    field_element(element, name).map(|field| field.children.as_slice())
}

fn field_element<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a HtmlElement> {
    element.children.iter().find_map(|child| {
        let HtmlNode::Element(child) = child else {
            return None;
        };

        (is_field(child) && attr(child, "name").as_deref() == Some(name)).then_some(child)
    })
}

fn is_field(element: &HtmlElement) -> bool {
    attr(element, "data-typlite-field").as_deref() == Some("true")
}
