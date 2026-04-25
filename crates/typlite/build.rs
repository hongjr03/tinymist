//! Generates the Typst-side typlite IR library.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use typst::foundations::{Element, Module, Scope, Value};
use typst::{Library, LibraryExt};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR must be set");
    let out_dir = Path::new(&out_dir);

    let elements = collect_elements();

    write_typst_ir_lib(out_dir, &elements);
    write_element_spec(out_dir, &elements);
}

fn write_typst_ir_lib(out_dir: &Path, elements: &[ElementSpec]) {
    let mut out = String::new();
    out.push_str(
        r#"#let field(it, name, default: none) = it.fields().at(name, default: default)

#let encode_content(value) = __typlite_encode_content(value)

#let field_node(name, kind, body) = html.elem(
  "span",
  attrs: (data-typlite-field: "true", name: name, kind: kind),
  body,
)

#let value_node(name, value) = field_node(name, "json", __typlite_encode_value(value))

#let content_node(name, value) = if type(value) == type([]) {
  field_node(name, "content", value)
} else {
  []
}

#let encoded_content_node(name, value) = field_node(name, "content-ir", __typlite_encode_content(value))

#let frame_node(value) = if target() == "html" {
  html.elem(
    "span",
    attrs: (data-typlite-field: "true", name: "frame", kind: "frame"),
    html.frame(value),
  )
} else {
  value
}

#let list_item_node(index, item) = field_node(str(index), "list.item", {
  content_node("body", field(item, "body"))
})

#let enum_item_node(index, item) = field_node(str(index), "enum.item", {
  value_node("number", field(item, "number"))
  content_node("body", field(item, "body"))
})

#let term_item_node(index, item) = field_node(str(index), "terms.item", {
  content_node("term", field(item, "term"))
  content_node("description", field(item, "description"))
})

#let children_node(name, children, item_node) = field_node(name, "array", {
  for (index, item) in children.enumerate() {
    item_node(index, item)
  }
})

#let node(kind, body) = html.elem(
  "typlite-" + kind,
  attrs: (data-typlite-ir: "true"),
  body,
)

#let inline(kind, body) = html.elem(
  "span",
  attrs: (data-typlite: kind, data-typlite-ir: "true"),
  body,
)

#let typlite(body) = {
"#,
    );

    for element in elements {
        let selector = element.selector();
        if selector.starts_with("math.") && selector != "math.equation" {
            continue;
        }
        let kind = element.kind();
        let encoded = "__typlite_encode_element(it)";
        let fields = element
            .fields()
            .filter(|field| is_content_transport_field(element.selector().as_str(), field))
            .map(|field| element.content_field_node(field))
            .collect::<String>();
        let frame = element
            .needs_frame()
            .then_some("\n    frame_node(it)")
            .unwrap_or("");
        let body = if selector == "math.equation" {
            "{\n    encoded_content_node(\"body\", field(it, \"body\"))\n  }".to_owned()
        } else {
            format!("{{ {encoded}\n{fields}{frame} }}")
        };

        let call = match element.mode() {
            Mode::Block => format!("node({kind:?}, {body})"),
            Mode::Inline => format!("inline({kind:?}, {body})"),
            Mode::BlockOrInline if selector == "math.equation" => format!(
                "if __typlite_is_block_equation(it) {{ node({kind:?}, {body}) }} else {{ inline({kind:?}, {body}) }}"
            ),
            Mode::BlockOrInline => format!(
                "if field(it, \"block\", default: false) {{ node({kind:?}, {body}) }} else {{ inline({kind:?}, {body}) }}"
            ),
        };

        out.push_str(&format!(
            "  show {selector}: it => if target() == \"html\" {{ {call} }} else {{ it }}\n"
        ));
    }

    out.push_str("\n  body\n}\n");

    fs::write(out_dir.join("typlite-ir.typ"), out)
        .expect("failed to write generated typlite IR library");
}

fn write_element_spec(out_dir: &Path, elements: &[ElementSpec]) {
    let mut out = String::new();
    out.push_str(
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
"#,
    );

    for element in elements {
        let variant = element.variant();
        out.push_str(&format!("    {variant},\n"));
    }

    out.push_str(
        r#"}

impl ElementKind {
    pub fn selector(self) -> &'static str {
        match self {
"#,
    );

    for element in elements {
        let variant = element.variant();
        let selector = element.selector();
        out.push_str(&format!(
            "            ElementKind::{variant} => {selector:?},\n"
        ));
    }

    out.push_str(
        r#"        }
    }

    pub fn name(self) -> &'static str {
        match self {
"#,
    );

    for element in elements {
        let variant = element.variant();
        let kind = element.kind();
        out.push_str(&format!(
            "            ElementKind::{variant} => {kind:?},\n"
        ));
    }

    out.push_str(
        r#"        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementMode {
    Block,
    Inline,
    BlockOrInline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementSpec {
    pub selector: &'static str,
    pub kind: ElementKind,
    pub mode: ElementMode,
    pub fields: &'static [&'static str],
}

pub static ELEMENTS: &[ElementSpec] = &[
"#,
    );

    for element in elements {
        let selector = element.selector();
        let variant = element.variant();
        let mode = match element.mode() {
            Mode::Block => "ElementMode::Block",
            Mode::Inline => "ElementMode::Inline",
            Mode::BlockOrInline => "ElementMode::BlockOrInline",
        };
        let fields = element.fields().collect::<Vec<_>>();

        out.push_str("    ElementSpec {\n");
        out.push_str(&format!("        selector: {selector:?},\n"));
        out.push_str(&format!("        kind: ElementKind::{variant},\n"));
        out.push_str(&format!("        mode: {mode},\n"));
        out.push_str("        fields: &[\n");
        for field in fields {
            out.push_str(&format!("            {field:?},\n"));
        }
        out.push_str("        ],\n");
        out.push_str("    },\n");
    }

    out.push_str("];\n");

    fs::write(out_dir.join("typlite-elements.rs"), out)
        .expect("failed to write generated typlite element spec");
}

#[derive(Debug, Clone)]
struct ElementSpec {
    path: Vec<String>,
    elem: Element,
}

impl ElementSpec {
    fn selector(&self) -> String {
        self.path.join(".")
    }

    fn kind(&self) -> String {
        match self.selector().as_str() {
            "par" => "paragraph".to_owned(),
            selector => selector.replace('.', "-"),
        }
    }

    fn variant(&self) -> String {
        self.path
            .iter()
            .flat_map(|part| part.split(['-', '_']))
            .filter(|part| !part.is_empty())
            .map(upper_camel)
            .collect()
    }

    fn mode(&self) -> Mode {
        match self.selector().as_str() {
            "bibliography" | "block" | "colbreak" | "columns" | "enum" | "figure" | "grid"
            | "heading" | "list" | "move" | "outline" | "pad" | "pagebreak" | "par"
            | "parbreak" | "rotate" | "scale" | "skew" | "stack" | "table" | "terms" | "title"
            | "v" | "align" => Mode::Block,
            "math.equation" | "quote" | "raw" => Mode::BlockOrInline,
            _ => Mode::Inline,
        }
    }

    fn fields(&self) -> impl Iterator<Item = &'static str> + '_ {
        element_fields(self.elem)
    }

    fn needs_frame(&self) -> bool {
        matches!(
            self.selector().as_str(),
            "circle"
                | "curve"
                | "ellipse"
                | "image"
                | "line"
                | "path"
                | "polygon"
                | "rect"
                | "square"
        )
    }

    fn content_field_node(&self, field: &str) -> String {
        match (self.selector().as_str(), field) {
            ("enum", "children") => {
                format!("    children_node({field:?}, field(it, {field:?}), enum_item_node)\n")
            }
            ("list", "children") => {
                format!("    children_node({field:?}, field(it, {field:?}), list_item_node)\n")
            }
            ("terms", "children") => {
                format!("    children_node({field:?}, field(it, {field:?}), term_item_node)\n")
            }
            _ => format!("    content_node({field:?}, field(it, {field:?}))\n"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Block,
    Inline,
    BlockOrInline,
}

fn collect_elements() -> Vec<ElementSpec> {
    let library = Library::default();
    let mut elements = BTreeMap::new();
    let mut visited_modules = BTreeSet::new();
    let mut visited_scopes = BTreeSet::new();

    collect_module(
        &[],
        &library.global,
        &mut elements,
        &mut visited_modules,
        &mut visited_scopes,
    );

    elements.into_values().collect()
}

fn collect_module(
    prefix: &[String],
    module: &Module,
    elements: &mut BTreeMap<String, ElementSpec>,
    visited_modules: &mut BTreeSet<String>,
    visited_scopes: &mut BTreeSet<String>,
) {
    let name = module.name().map(ToString::to_string).unwrap_or_default();
    let key = format!("{}::{name}", prefix.join("."));
    if !visited_modules.insert(key) {
        return;
    }

    collect_scope(
        prefix,
        module.scope(),
        elements,
        visited_modules,
        visited_scopes,
    );
}

fn collect_scope(
    prefix: &[String],
    scope: &Scope,
    elements: &mut BTreeMap<String, ElementSpec>,
    visited_modules: &mut BTreeSet<String>,
    visited_scopes: &mut BTreeSet<String>,
) {
    for (name, binding) in scope.iter() {
        let mut path = prefix.to_vec();
        path.push(name.to_string());

        match binding.read() {
            Value::Func(func) => {
                if let Some(elem) = func.element() {
                    insert_element(path, elem, elements, visited_modules, visited_scopes);
                }
            }
            Value::Module(module) if name.as_str() != "html" => {
                collect_module(&path, module, elements, visited_modules, visited_scopes);
            }
            _ => {}
        }
    }
}

fn insert_element(
    path: Vec<String>,
    elem: Element,
    elements: &mut BTreeMap<String, ElementSpec>,
    visited_modules: &mut BTreeSet<String>,
    visited_scopes: &mut BTreeSet<String>,
) {
    let selector = path.join(".");
    let spec = ElementSpec {
        path: path.clone(),
        elem,
    };

    if should_generate(&spec) {
        elements.insert(selector.clone(), spec);
    }

    let scope_key = format!("{}@{}", selector, elem.name());
    if visited_scopes.insert(scope_key) {
        collect_scope(
            &path,
            elem.scope(),
            elements,
            visited_modules,
            visited_scopes,
        );
    }
}

fn should_generate(spec: &ElementSpec) -> bool {
    let selector = spec.selector();

    if matches!(
        selector.as_str(),
        "text" | "enum.item" | "list.item" | "terms.item"
    ) || spec.elem.name() == "text"
    {
        return false;
    }

    true
}

fn element_fields(elem: Element) -> impl Iterator<Item = &'static str> {
    (0..u8::MAX).filter_map(move |id| elem.field_name(id))
}

fn is_content_transport_field(selector: &str, field: &str) -> bool {
    matches!(
        field,
        "body"
            | "title"
            | "caption"
            | "attribution"
            | "term"
            | "description"
            | "supplement"
            | "citation"
    ) || matches!((selector, field), ("list" | "enum" | "terms", "children"))
}

fn upper_camel(part: &str) -> String {
    let mut out = String::new();
    let mut chars = part.chars();

    if let Some(first) = chars.next() {
        out.extend(first.to_uppercase());
    }
    out.extend(chars);
    out
}
