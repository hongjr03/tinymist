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
            let level = attr(element, "level")
                .and_then(|level| level.parse::<u8>().ok())
                .unwrap_or(1);
            blocks.push(Block::Heading {
                level,
                body: collect_inlines(&element.children),
            });
        }
        Some("typlite-paragraph") => {
            blocks.push(Block::Paragraph(collect_inlines(&element.children)));
        }
        Some("typlite-raw") => {
            blocks.push(Block::Raw {
                lang: attr(element, "lang"),
                text: collect_text(&element.children),
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
            HtmlNode::Element(element) => match attr(element, "data-typlite").as_deref() {
                Some("emph") => out.push(Inline::Emph(collect_inlines(&element.children))),
                Some("strong") => out.push(Inline::Strong(collect_inlines(&element.children))),
                Some("raw") => out.push(Inline::Raw {
                    lang: attr(element, "lang"),
                    text: collect_text(&element.children),
                }),
                _ => out.extend(collect_inlines(&element.children)),
            },
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
            HtmlNode::Element(element) => out.push_str(&collect_text(&element.children)),
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
