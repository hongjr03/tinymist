//! Snapshot tests for the typlite conversion spike.

use std::path::Path;
use std::sync::Arc;

use tinymist_tests::run_with_sources;
use typlite::element_spec::{ELEMENTS, ElementKind, ElementMode};
use typlite::ir::*;
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

#[test]
fn markdown_backend_renders_resilient_gap_paths() {
    let doc = Document {
        blocks: vec![Block::Paragraph(ParagraphBlock {
            body: vec![
                Inline::Math(MathInline {
                    body: MathNode {
                        func: "text".into(),
                        fields: vec![MathField {
                            name: "text".into(),
                            value: MathValue::Scalar("x".into()),
                        }],
                    },
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::Cite(CiteInline {
                    key: Some("missing".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::Ref(RefInline {
                    target: Some("unknown".into()),
                    form: Some("page".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::PdfEmbed(PdfEmbedInline {
                    path: Some("asset.pdf".into()),
                    ..Default::default()
                }),
            ],
        })],
    };

    let markdown = typlite::backend::render_markdown(&doc).unwrap();
    assert!(markdown.contains("$x$"));
    assert!(markdown.contains("[@missing](#ref-missing)"));
    assert!(markdown.contains("[unknown](#unknown)"));
    assert!(markdown.contains("<!-- typlite-pdf: asset.pdf -->"));
}

#[test]
fn markdown_backend_renders_dedicated_math_inline_nodes() {
    let doc = Document {
        blocks: vec![Block::Paragraph(ParagraphBlock {
            body: vec![
                Inline::MathAccent(MathAccentInline {
                    base: Some("x".into()),
                    accent: Some("hat".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathAttach(MathAttachInline {
                    base: Some("x".into()),
                    b: Some("1".into()),
                    t: Some("2".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathBinom(MathBinomInline {
                    upper: Some("n".into()),
                    lower: Some("k".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathCancel(MathCancelInline {
                    body: Some("x".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathCases(MathCasesInline {
                    children: vec![
                        Inline::Text(TextInline { text: "1".into() }),
                        Inline::Text(TextInline { text: "2".into() }),
                    ],
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathClass(MathClassInline {
                    body: Some("x".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathFrac(MathFracInline {
                    num: Some("1".into()),
                    denom: Some("2".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathLimits(MathLimitsInline {
                    body: Some(r"\sum".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathLr(MathLrInline {
                    body: Some("(x)".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathMat(MathMatInline {
                    rows: Some(
                        r#"[ [{"func":"text","text":"1"}], [{"func":"text","text":"2"}] ]"#.into(),
                    ),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathMid(MathMidInline {
                    body: Some("|".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathOp(MathOpInline {
                    text: Some("lim".into()),
                    limits: true,
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathOverbrace(MathOverbraceInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathOverbracket(MathOverbracketInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathOverline(MathOverlineInline {
                    body: Some("x".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathOverparen(MathOverparenInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathOvershell(MathOvershellInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathPrimes(MathPrimesInline {
                    count: Some("3".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathRoot(MathRootInline {
                    index: Some("3".into()),
                    radicand: Some("x".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathScripts(MathScriptsInline {
                    body: Some("x".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathStretch(MathStretchInline {
                    body: Some(r"\to".into()),
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathUnderbrace(MathUnderbraceInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathUnderbracket(MathUnderbracketInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathUnderline(MathUnderlineInline {
                    body: Some("x".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathUnderparen(MathUnderparenInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathUndershell(MathUndershellInline {
                    body: Some("x".into()),
                    annotation: Some("1".into()),
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::MathVec(MathVecInline {
                    children: vec![
                        Inline::Text(TextInline { text: "1".into() }),
                        Inline::Text(TextInline { text: "2".into() }),
                    ],
                    ..Default::default()
                }),
                Inline::Text(TextInline { text: " ".into() }),
                Inline::CurveLine(CurveLineInline {
                    end: Some("(1pt, 1pt)".into()),
                    ..Default::default()
                }),
            ],
        })],
    };

    let markdown = typlite::backend::render_markdown(&doc).unwrap();
    assert!(markdown.contains(r"$\hat{x}$"));
    assert!(markdown.contains(r"$\frac{1}{2}$"));
    assert!(markdown.contains(r"$\sqrt[3]{x}$"));
    assert!(markdown.contains(r"$\left(\begin{matrix}1 \\ 2\end{matrix}\right)$"));
    assert!(
        markdown.contains(
            "typlite-warning: curve.line requires wrapping the parent curve in html.frame"
        )
    );
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
