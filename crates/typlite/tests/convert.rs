//! Snapshot tests for the typlite conversion spike.

use std::path::Path;
use std::sync::Arc;

use tinymist_tests::run_with_sources;
use typlite::{Format, Typlite, TypliteFeat};
use typst_html::{HtmlElement, HtmlNode};

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
