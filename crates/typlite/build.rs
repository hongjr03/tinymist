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

#let scalar(value) = {
  let ty = type(value)
  if ty == type(none) {
    "none"
  } else if ty == type(auto) {
    "auto"
  } else if ty == type(true) {
    if value { "true" } else { "false" }
  } else if ty in (type(""), type(0), type(0.0), type(<label>), type(type)) {
    str(value)
  } else {
    let encoded = json.encode(value, pretty: false)
    if encoded.starts-with("\"") and encoded.ends-with("\"") {
      encoded.slice(1, -1)
    } else {
      encoded
    }
  }
}

#let field_node(name, kind, body, attrs: (:)) = html.elem(
  "span",
  attrs: (data-typlite-field: "true", name: name, kind: kind) + attrs,
  body,
)

#let value_node(name, value) = {
  let ty = type(value)
  let kind = str(ty)
  if ty == type(none) {
    field_node(name, kind, [])
  } else if ty == type([]) {
    field_node(name, kind, value)
  } else if ty == type((:)) {
    field_node(name, kind, {
      for (key, item) in value.pairs() {
        value_node(key, item)
      }
    })
  } else if ty == type(()) {
    field_node(name, kind, {
      for (index, item) in value.enumerate() {
        value_node(str(index), item)
      }
    })
  } else {
    field_node(name, kind, scalar(value))
  }
}

#let opaque_value_node(name, value) = {
  let ty = type(value)
  let kind = str(ty)
  if ty == type(none) {
    field_node(name, kind, [])
  } else {
    field_node(name, kind, scalar(value))
  }
}

#let frame_node(name, value) = field_node(name, "frame", html.frame(value))

#let html_target(value, fallback) = if target() == "html" {
  value
} else {
  fallback
}

#let list_item_node(index, item) = field_node(str(index), "list.item", {
  value_node("body", field(item, "body"))
})

#let enum_item_node(index, item) = field_node(str(index), "enum.item", {
  value_node("number", field(item, "number"))
  value_node("body", field(item, "body"))
})

#let term_item_node(index, item) = field_node(str(index), "terms.item", {
  value_node("term", field(item, "term"))
  value_node("description", field(item, "description"))
})

#let children_node(name, children, item_node) = field_node(name, "array", {
  for (index, item) in children.enumerate() {
    item_node(index, item)
  }
})

#let node(kind, body) = html.elem(
  "typlite-" + kind,
  body,
)

#let inline(kind, body) = html.elem(
  "span",
  attrs: (data-typlite: kind),
  body,
)

#let typlite(body) = {
"#,
    );

    for element in elements {
        let selector = element.selector();
        let kind = element.kind();
        let value_node = if selector.starts_with("math.") {
            "opaque_value_node"
        } else {
            "value_node"
        };
        let fields = element
            .fields()
            .map(|field| element.field_node(field, value_node))
            .collect::<String>();
        let frame = element
            .needs_frame()
            .then_some("    frame_node(\"frame\", it)\n")
            .unwrap_or("");
        let body = format!("{{\n{fields}{frame}  }}");

        let call = match element.mode() {
            Mode::Block => format!("node({kind:?}, {body})"),
            Mode::Inline => format!("inline({kind:?}, {body})"),
            Mode::BlockOrInline => format!(
                "if field(it, \"block\", default: false) {{ node({kind:?}, {body}) }} else {{ inline({kind:?}, {body}) }}"
            ),
        };

        out.push_str(&format!(
            "  show {selector}: it => html_target({call}, it)\n"
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
            "circle" | "curve" | "ellipse" | "line" | "path" | "polygon" | "rect" | "square"
        )
    }

    fn field_node(&self, field: &str, value_node: &str) -> String {
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
            ("metadata", "value") => {
                format!("    opaque_value_node({field:?}, field(it, {field:?}))\n")
            }
            ("bibliography", "sources") => {
                format!("    opaque_value_node({field:?}, field(it, {field:?}))\n")
            }
            _ => format!("    {value_node}({field:?}, field(it, {field:?}))\n"),
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

fn upper_camel(part: &str) -> String {
    let mut out = String::new();
    let mut chars = part.chars();

    if let Some(first) = chars.next() {
        out.extend(first.to_uppercase());
    }
    out.extend(chars);
    out
}
