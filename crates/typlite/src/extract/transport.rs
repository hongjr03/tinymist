//! Transport view over a Typst HTML element.

use ecow::EcoString;
use serde_json::{Map, Value};
use tinymist_std::error::prelude::*;
use typst::introspection::Introspector;
use typst_html::{HtmlElement, HtmlNode};

use crate::Result;
use crate::ir::{Block, Inline, MathNode};

use super::{
    collect_inlines, collect_item_blocks, collect_text_without_frames, encoded, field_children,
    math_field,
};

pub(super) struct TransportElement<'a> {
    pub(super) element: &'a HtmlElement,
    pub(super) encoded: Option<Map<String, Value>>,
}

impl<'a> TransportElement<'a> {
    pub(super) fn new(element: &'a HtmlElement) -> Result<Self> {
        Ok(Self {
            element,
            encoded: encoded_object(element)?,
        })
    }

    pub(super) fn field(&self, name: &str) -> Option<&'a [HtmlNode]> {
        field_children(self.element, name)
    }

    pub(super) fn encoded_field(&self, name: &str) -> Option<&Value> {
        self.encoded.as_ref().and_then(|object| object.get(name))
    }

    pub(super) fn scalar(&self, name: &str) -> Option<EcoString> {
        if let Some(value) = self.encoded_field(name) {
            return Some(encoded::scalar(value));
        }

        self.field(name).map(collect_text_without_frames)
    }

    pub(super) fn bool(&self, name: &str) -> bool {
        self.scalar(name)
            .is_some_and(|value| value.as_str() == "true")
    }

    pub(super) fn content_blocks(
        &self,
        name: &str,
        introspector: &Introspector,
    ) -> Result<Vec<Block>> {
        if let Some(children) = self.field(name)
            && contains_frame(children)
        {
            return collect_item_blocks(children, introspector);
        }

        if let Some(value) = self.encoded_field(name) {
            return encoded::content_blocks(value, introspector);
        }

        if let Some(children) = self.field(name) {
            return collect_item_blocks(children, introspector);
        }

        Ok(Vec::new())
    }

    pub(super) fn content_inlines(
        &self,
        name: &str,
        introspector: &Introspector,
    ) -> Result<Vec<Inline>> {
        if let Some(children) = self.field(name)
            && contains_frame(children)
        {
            return collect_inlines(children, introspector);
        }

        if let Some(value) = self.encoded_field(name) {
            return encoded::content_inlines(value, introspector);
        }

        if let Some(children) = self.field(name) {
            return collect_inlines(children, introspector);
        }

        Ok(Vec::new())
    }

    pub(super) fn math(&self, name: &str) -> Result<MathNode> {
        if self.field(name).is_some() {
            return math_field(self.element, name);
        }

        if let Some(value) = self.encoded_field(name) {
            return encoded::math_node(value);
        }

        math_field(self.element, name)
    }
}

fn contains_frame(nodes: &[HtmlNode]) -> bool {
    nodes.iter().any(|node| match node {
        HtmlNode::Frame(_) => true,
        HtmlNode::Element(element) => contains_frame(&element.children),
        HtmlNode::Text(..) | HtmlNode::Tag(_) => false,
    })
}

fn encoded_object(element: &HtmlElement) -> Result<Option<Map<String, Value>>> {
    if super::attr(element, "data-typlite-ir").as_deref() != Some("true") {
        return Ok(None);
    }

    let raw = collect_text_without_frames(&element.children);
    if raw.trim().is_empty() {
        return Ok(None);
    }

    let value =
        serde_json::from_str::<Value>(&raw).with_context_ut("cannot parse typlite IR", || {
            Some(Box::new([
                ("tag", super::tag_name(element).unwrap_or_default()),
                ("raw", raw.to_string()),
            ]))
        })?;
    let Value::Object(object) = value else {
        bail!("typlite IR must be encoded as an object, got {value}");
    };
    Ok(Some(object))
}
