use ecow::EcoString;

/// A rendered frame image extracted from `html.frame`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameImage {
    /// SVG payload.
    pub svg: EcoString,
}
