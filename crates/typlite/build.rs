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
    let encoded = json.encode(value)
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
        let fields = element
            .fields()
            .map(|field| format!("    value_node({field:?}, field(it, {field:?}))\n"))
            .collect::<String>();
        let body = format!("{{\n{fields}  }}");

        let call = match element.mode() {
            Mode::Block => format!("node({kind:?}, {body})"),
            Mode::Inline => format!("inline({kind:?}, {body})"),
            Mode::Raw => format!(
                "if field(it, \"block\", default: false) {{ node({kind:?}, {body}) }} else {{ inline({kind:?}, {body}) }}"
            ),
        };

        out.push_str(&format!("  show {selector}: it => {call}\n"));
    }

    out.push_str("\n  body\n}\n");

    fs::write(out_dir.join("typlite-ir.typ"), out)
        .expect("failed to write generated typlite IR library");
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

    fn mode(&self) -> Mode {
        match self.selector().as_str() {
            "heading" | "par" => Mode::Block,
            "raw" => Mode::Raw,
            _ => Mode::Inline,
        }
    }

    fn fields(&self) -> impl Iterator<Item = &'static str> + '_ {
        element_fields(self.elem)
    }

    fn has_field(&self, name: &str) -> bool {
        element_fields(self.elem).any(|field| field == name)
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Block,
    Inline,
    Raw,
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
    if matches!(spec.selector().as_str(), "block" | "parbreak" | "text")
        || spec.elem.name() == "text"
    {
        return false;
    }

    spec.has_field("body") || spec.has_field("text")
}

fn element_fields(elem: Element) -> impl Iterator<Item = &'static str> {
    (0..u8::MAX).filter_map(move |id| elem.field_name(id))
}
