//! Snapshot tests for the typlite conversion spike.

use std::path::Path;
use std::sync::Arc;

use tinymist_tests::run_with_sources;
use typlite::element_spec::{ELEMENTS, ElementKind, ElementMode};
use typlite::{Format, Typlite, TypliteFeat};
use typst_html::{HtmlElement, HtmlNode};

#[test]
fn generated_element_spec_covers_core_elements() {
    let table = element("table");
    assert_eq!(table.kind, ElementKind::Table);
    assert_eq!(table.mode, ElementMode::Block);
    assert!(table.fields.contains(&"align"));
    assert!(table.fields.contains(&"children"));

    let grid = element("grid");
    assert_eq!(grid.kind, ElementKind::Grid);
    assert_eq!(grid.mode, ElementMode::Block);
    assert!(grid.fields.contains(&"children"));

    let link = element("link");
    assert_eq!(link.mode, ElementMode::Inline);
    assert!(link.fields.contains(&"dest"));
    assert!(link.fields.contains(&"body"));

    let math = element("math.equation");
    assert_eq!(math.kind, ElementKind::MathEquation);
    assert_eq!(math.mode, ElementMode::BlockOrInline);
    assert!(math.fields.contains(&"body"));
}

#[test]
fn generated_element_spec_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(manifest_dir.join("fixtures/convert/snaps"));

    settings.bind(|| {
        insta::assert_snapshot!("generated_element_spec", render_element_spec());
    });
}

#[test]
fn rust_content_encoder_reads_styled_content() {
    let source = r##"
#import "@local/typlite-ir:0.1.0": encode_content

#raw(encode_content(text(red)[red]))

#raw(encode_content($bold(A)$))
"##;

    run_with_sources(source, |verse, _| {
        let world = Arc::new(verse.snapshot());
        let (world, _) = TypliteFeat::default()
            .prepare_world(&world, Format::Md)
            .unwrap();
        let html = typst::compile::<typst_html::HtmlDocument>(&world)
            .output
            .unwrap();
        let debug = render_debug_html(&html.root);

        assert!(debug.contains(r#"\"func\":\"styled"#), "{debug}");
        assert!(debug.contains(r#"#ff4136"#), "{debug}");
        assert!(debug.contains(r#"\"bold\":true"#), "{debug}");
    });
}

fn element(selector: &str) -> &'static typlite::element_spec::ElementSpec {
    ELEMENTS
        .iter()
        .find(|element| element.selector == selector)
        .unwrap_or_else(|| panic!("missing generated element spec for {selector}"))
}

fn render_element_spec() -> String {
    let mut out = String::new();

    for element in ELEMENTS {
        out.push_str(element.selector);
        out.push('\n');
        out.push_str("  enum: ");
        out.push_str(&format!("{:?}", element.kind));
        out.push('\n');
        out.push_str("  kind: ");
        out.push_str(element.kind.name());
        out.push('\n');
        out.push_str("  mode: ");
        out.push_str(match element.mode {
            ElementMode::Block => "block",
            ElementMode::Inline => "inline",
            ElementMode::BlockOrInline => "block-or-inline",
        });
        out.push('\n');
        out.push_str("  fields:");
        if element.fields.is_empty() {
            out.push_str(" []\n");
        } else {
            out.push('\n');
            for field in element.fields {
                out.push_str("    - ");
                out.push_str(field);
                out.push('\n');
            }
        }
        out.push('\n');
    }

    out
}

#[test]
fn convert_fixtures() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("fixtures/convert");

    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(fixture_dir.join("snaps"));

    settings.bind(|| {
        insta::glob!(fixture_dir, "*.typ", |path| {
            let source = std::fs::read_to_string(path).unwrap();
            let name = path.file_stem().unwrap().to_string_lossy();

            run_with_sources(&source, |verse, _| {
                let world = Arc::new(verse.snapshot());
                let markdown = Typlite::new(world).convert().unwrap();
                insta::assert_snapshot!(format!("{name}__markdown"), markdown);

                let world = Arc::new(verse.snapshot());
                let (world, _) = TypliteFeat::default()
                    .prepare_world(&world, Format::Md)
                    .unwrap();
                let html = typst::compile::<typst_html::HtmlDocument>(&world)
                    .output
                    .unwrap();
                insta::assert_snapshot!(format!("{name}__dom"), render_debug_html(&html.root));
            });
        });
    });
}

fn render_debug_html(root: &HtmlElement) -> String {
    fn render_element(element: &HtmlElement, indent: usize, out: &mut String) {
        out.push_str(&"  ".repeat(indent));
        out.push('<');
        out.push_str(element.tag.resolve().as_str());
        for (key, value) in &element.attrs.0 {
            out.push(' ');
            out.push_str(key.resolve().as_str());
            out.push_str("=\"");
            out.push_str(value);
            out.push('"');
        }
        out.push_str(">\n");

        for child in &element.children {
            match child {
                HtmlNode::Text(text, _) => {
                    out.push_str(&"  ".repeat(indent + 1));
                    out.push_str("text ");
                    out.push_str(&format!("{text:?}"));
                    out.push('\n');
                }
                HtmlNode::Element(child) => render_element(child, indent + 1, out),
                HtmlNode::Tag(_) => {
                    out.push_str(&"  ".repeat(indent + 1));
                    out.push_str("tag\n");
                }
                HtmlNode::Frame(_) => {
                    out.push_str(&"  ".repeat(indent + 1));
                    out.push_str("frame\n");
                }
            }
        }
    }

    let mut out = String::new();
    render_element(root, 0, &mut out);
    out
}
