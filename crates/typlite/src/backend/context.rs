use crate::ir::*;
use ecow::{EcoString, EcoVec};
use std::cell::RefCell;
use std::collections::BTreeMap;
use typst::diag::SourceDiagnostic;

/// Rendered bibliography entries available to the Markdown backend.
#[derive(Debug, Default, Clone)]
pub struct BibliographyContext {
    entries: BTreeMap<EcoString, EcoString>,
    citations: BTreeMap<EcoString, EcoString>,
    citation_offsets: RefCell<BTreeMap<EcoString, usize>>,
    reference_anchors: RefCell<BTreeMap<String, Vec<EcoString>>>,
    warnings: RefCell<EcoVec<SourceDiagnostic>>,
    order: Vec<EcoString>,
}

impl BibliographyContext {
    /// Creates a bibliography context from rendered entries.
    pub fn new(
        entries: impl IntoIterator<Item = (EcoString, EcoString)>,
        citations: impl IntoIterator<Item = (EcoString, EcoString)>,
    ) -> Self {
        let mut map = BTreeMap::new();
        let mut order = Vec::new();

        for (key, rendered) in entries {
            if !map.contains_key(&key) {
                order.push(key.clone());
            }
            map.insert(key, rendered);
        }

        Self {
            entries: map,
            citations: citations.into_iter().collect(),
            citation_offsets: RefCell::default(),
            reference_anchors: RefCell::default(),
            warnings: RefCell::default(),
            order,
        }
    }

    pub(super) fn reset_render_state(&self, doc: &Document) {
        self.citation_offsets.borrow_mut().clear();
        self.warnings.borrow_mut().clear();
        let mut anchors = self.reference_anchors.borrow_mut();
        anchors.clear();
        collect_reference_anchors(&doc.blocks, &mut anchors);
    }

    pub(super) fn ordered_entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.order.iter().filter_map(|key| {
            self.entries
                .get(key)
                .map(|rendered| (key.as_str(), rendered.as_str()))
        })
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(super) fn citation(&self, key: &str) -> Option<&str> {
        self.citations.get(key).map(EcoString::as_str)
    }

    pub(super) fn next_citation_id(&self, key: &str) -> String {
        let mut offsets = self.citation_offsets.borrow_mut();
        let offset = offsets.entry(key.into()).or_default();
        *offset += 1;
        format!("cite-{key}-{offset}")
    }

    pub(super) fn citation_count(&self, key: &str) -> usize {
        self.citation_offsets
            .borrow()
            .get(key)
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn take_reference_anchors(&self, block: &Block) -> Vec<EcoString> {
        self.reference_anchors
            .borrow_mut()
            .remove(&reference_anchor_key(block))
            .unwrap_or_default()
    }

    pub(super) fn warn(&self, warning: SourceDiagnostic) {
        self.warnings.borrow_mut().push(warning);
    }

    pub(super) fn take_warnings(&self) -> EcoVec<SourceDiagnostic> {
        std::mem::take(&mut *self.warnings.borrow_mut())
    }
}

/// Rendered Markdown and diagnostics produced while rendering.
#[derive(Debug, Clone)]
pub struct RenderedMarkdown {
    /// Markdown output.
    pub output: String,
    /// Non-fatal rendering diagnostics.
    pub warnings: EcoVec<SourceDiagnostic>,
}

fn collect_reference_anchors(blocks: &[Block], out: &mut BTreeMap<String, Vec<EcoString>>) {
    for block in blocks {
        match block {
            Block::Heading(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Block::Paragraph(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Block::Quote(data) => collect_reference_anchors(&data.body, out),
            Block::Figure(data) => {
                collect_reference_anchors(&data.body, out);
                collect_reference_anchors_in_inlines(&data.caption, out);
            }
            Block::Align(data) => collect_reference_anchors(&data.body, out),
            Block::Table(data) => {
                for row in &data.rows {
                    for cell in &row.cells {
                        collect_reference_anchors_in_inlines(&cell.body, out);
                    }
                }
            }
            Block::List(data) => {
                for item in &data.items {
                    collect_reference_anchors(&item.body, out);
                }
            }
            Block::Terms(data) => {
                for item in &data.items {
                    collect_reference_anchors_in_inlines(&item.term, out);
                    collect_reference_anchors(&item.description, out);
                }
            }
            Block::Block(data) => collect_reference_anchors(&data.body, out),
            Block::Columns(data) => collect_reference_anchors(&data.body, out),
            Block::Move(data) => collect_reference_anchors(&data.body, out),
            Block::Pad(data) => collect_reference_anchors(&data.body, out),
            Block::Rotate(data) => collect_reference_anchors(&data.body, out),
            Block::Scale(data) => collect_reference_anchors(&data.body, out),
            Block::Skew(data) => collect_reference_anchors(&data.body, out),
            Block::Stack(data) => collect_reference_anchors(&data.children, out),
            Block::Title(data) => collect_reference_anchors(&data.body, out),
            Block::Math(_) | Block::Raw(_) => {}
            Block::Bibliography(_)
            | Block::Colbreak(_)
            | Block::Outline(_)
            | Block::Pagebreak(_)
            | Block::Parbreak(_)
            | Block::V(_) => {}
        }
    }
}

fn collect_reference_anchors_in_inlines(
    inlines: &[Inline],
    out: &mut BTreeMap<String, Vec<EcoString>>,
) {
    for inline in inlines {
        match inline {
            Inline::Ref(data) => {
                if let Some(target) = data.target.as_deref() {
                    if let Some(element) = data.element.first() {
                        push_reference_anchor(
                            out,
                            reference_anchor_key(element),
                            normalized_label(target).into(),
                        );
                    }
                }
                collect_reference_anchors_in_inlines(&data.supplement, out);
                collect_reference_anchors_in_inlines(&data.citation, out);
                collect_reference_anchors(&data.element, out);
            }
            Inline::Emph(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Strong(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Strike(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Sub(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Super(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Link(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Text(_)
            | Inline::Math(_)
            | Inline::Linebreak(_)
            | Inline::Frame(_)
            | Inline::Raw(_) => {}
            Inline::Box(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Circle(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Curve(data) => collect_reference_anchors_in_inlines(&data.components, out),
            Inline::Ellipse(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::FigureCaption(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Footnote(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::GridCell(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::GridFooter(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::GridHeader(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::Hide(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Highlight(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::MathCases(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::MathVec(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::Move(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Overline(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Pad(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Page(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::PdfArtifact(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Place(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Quote(data) => {
                collect_reference_anchors_in_inlines(&data.attribution, out);
                collect_reference_anchors_in_inlines(&data.body, out);
            }
            Inline::RawLine(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Repeat(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Rotate(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Scale(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Skew(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Smallcaps(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::TableCell(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::TableFooter(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::TableHeader(data) => collect_reference_anchors_in_inlines(&data.children, out),
            Inline::Underline(data) => collect_reference_anchors_in_inlines(&data.body, out),
            Inline::Cite(_)
            | Inline::CurveClose(_)
            | Inline::CurveCubic(_)
            | Inline::CurveLine(_)
            | Inline::CurveMove(_)
            | Inline::CurveQuad(_)
            | Inline::Document(_)
            | Inline::FootnoteEntry(_)
            | Inline::GridHline(_)
            | Inline::GridVline(_)
            | Inline::H(_)
            | Inline::Image(_)
            | Inline::Line(_)
            | Inline::MathAccent(_)
            | Inline::MathAttach(_)
            | Inline::MathBinom(_)
            | Inline::MathCancel(_)
            | Inline::MathClass(_)
            | Inline::MathFrac(_)
            | Inline::MathLimits(_)
            | Inline::MathLr(_)
            | Inline::MathMat(_)
            | Inline::MathMid(_)
            | Inline::MathOp(_)
            | Inline::MathOverbrace(_)
            | Inline::MathOverbracket(_)
            | Inline::MathOverline(_)
            | Inline::MathOverparen(_)
            | Inline::MathOvershell(_)
            | Inline::MathPrimes(_)
            | Inline::MathRoot(_)
            | Inline::MathScripts(_)
            | Inline::MathStretch(_)
            | Inline::MathUnderbrace(_)
            | Inline::MathUnderbracket(_)
            | Inline::MathUnderline(_)
            | Inline::MathUnderparen(_)
            | Inline::MathUndershell(_)
            | Inline::Metadata(_)
            | Inline::OutlineEntry(_)
            | Inline::ParLine(_)
            | Inline::Path(_)
            | Inline::PdfAttach(_)
            | Inline::PdfEmbed(_)
            | Inline::PlaceFlush(_)
            | Inline::Polygon(_)
            | Inline::Rect(_)
            | Inline::Smartquote(_)
            | Inline::Square(_)
            | Inline::TableHline(_)
            | Inline::TableVline(_) => {}
        }
    }
}

fn push_reference_anchor(
    out: &mut BTreeMap<String, Vec<EcoString>>,
    key: String,
    anchor: EcoString,
) {
    let anchors = out.entry(key).or_default();
    if !anchors.iter().any(|existing| existing == &anchor) {
        anchors.push(anchor);
    }
}

fn reference_anchor_key(block: &Block) -> String {
    format!("{block:?}")
}

fn normalized_label(label: &str) -> &str {
    label.trim_start_matches('<').trim_end_matches('>')
}
