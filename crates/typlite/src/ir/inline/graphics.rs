use ecow::EcoString;
use typst_syntax::Span;

use super::{FrameImage, Inline};

/// Boxed content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoxInline {
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Baseline.
    pub baseline: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Corner radius.
    pub radius: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Clip behavior.
    pub clip: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
}

/// Circle shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CircleInline {
    /// Radius.
    pub radius: Option<EcoString>,
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Citation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CiteInline {
    /// Citation key.
    pub key: Option<EcoString>,
    /// Citation supplement.
    pub supplement: Vec<Inline>,
    /// Citation form.
    pub form: Option<EcoString>,
    /// Citation style.
    pub style: Option<EcoString>,
}

/// Curve shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveInline {
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Fill rule.
    pub fill_rule: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Curve components.
    pub components: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}

/// Curve close component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveCloseInline {
    /// Close mode.
    pub mode: Option<EcoString>,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve cubic component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveCubicInline {
    /// Start control point.
    pub control_start: Option<EcoString>,
    /// End control point.
    pub control_end: Option<EcoString>,
    /// End point.
    pub end: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve line component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveLineInline {
    /// End point.
    pub end: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve move component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveMoveInline {
    /// Start point.
    pub start: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Curve quadratic component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurveQuadInline {
    /// Control point.
    pub control: Option<EcoString>,
    /// End point.
    pub end: Option<EcoString>,
    /// Whether coordinates are relative.
    pub relative: bool,
    /// Source span.
    pub span: Option<Span>,
}

/// Document metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentInline {
    /// Title.
    pub title: Option<EcoString>,
    /// Author.
    pub author: Option<EcoString>,
    /// Description.
    pub description: Option<EcoString>,
    /// Keywords.
    pub keywords: Option<EcoString>,
    /// Date.
    pub date: Option<EcoString>,
}

/// Ellipse shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EllipseInline {
    /// Width.
    pub width: Option<EcoString>,
    /// Height.
    pub height: Option<EcoString>,
    /// Fill paint.
    pub fill: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
    /// Inset.
    pub inset: Option<EcoString>,
    /// Outset.
    pub outset: Option<EcoString>,
    /// Body.
    pub body: Vec<Inline>,
    /// Rendered frame.
    pub frame: Option<FrameImage>,
}
