//! Element field classification used by extraction paths.

use crate::element_spec::ElementSpec;

pub(super) fn content_fields(spec: &'static ElementSpec) -> impl Iterator<Item = &'static str> {
    spec.fields
        .iter()
        .copied()
        .filter(|field| is_content_field_name(field))
}

pub(super) fn is_content_field_name(field: &str) -> bool {
    matches!(
        field,
        "body"
            | "children"
            | "title"
            | "caption"
            | "attribution"
            | "term"
            | "description"
            | "supplement"
            | "citation"
            | "element"
    )
}
