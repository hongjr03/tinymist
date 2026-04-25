//! DOM helpers for Typst HTML extraction.

use ecow::EcoString;
use typst::introspection::Introspector;
use typst_html::{HtmlElement, HtmlFrame, HtmlNode};

use crate::ir::{FrameImage, Inline};

pub(super) fn tag_name(element: &HtmlElement) -> Option<String> {
    Some(element.tag.resolve().as_str().to_owned())
}

pub(super) fn attr(element: &HtmlElement, name: &str) -> Option<EcoString> {
    element
        .attrs
        .0
        .iter()
        .find(|(attr, _)| attr.resolve().as_str() == name)
        .map(|(_, value)| value.clone())
}

pub(super) fn is_field(element: &HtmlElement) -> bool {
    attr(element, "data-typlite-field").as_deref() == Some("true")
}

pub(super) fn field_children<'a>(element: &'a HtmlElement, name: &str) -> Option<&'a [HtmlNode]> {
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

pub(super) fn field_node(node: &HtmlNode) -> Option<&HtmlElement> {
    let HtmlNode::Element(element) = node else {
        return None;
    };

    is_field(element).then_some(element)
}

pub(super) fn collect_text_without_frames(nodes: &[HtmlNode]) -> EcoString {
    let mut out = EcoString::new();

    for node in nodes {
        match node {
            HtmlNode::Text(text, _) => out.push_str(text),
            HtmlNode::Element(element) => {
                if !is_field(element) {
                    out.push_str(&collect_text_without_frames(&element.children));
                }
            }
            HtmlNode::Tag(_) | HtmlNode::Frame(_) => {}
        }
    }

    out
}

pub(super) fn frame_to_inline(frame: &HtmlFrame, introspector: &Introspector) -> Inline {
    Inline::Frame(FrameImage {
        svg: frame_to_svg(frame, introspector),
    })
}

pub(super) fn frame_to_svg(frame: &HtmlFrame, introspector: &Introspector) -> EcoString {
    typst_svg::svg_html_frame(
        &frame.inner,
        frame.text_size,
        frame.id.as_deref(),
        &frame.link_points,
        introspector,
    )
    .into()
}
