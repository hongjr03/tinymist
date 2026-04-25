use ecow::EcoString;

use super::Inline;

/// Math accent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathAccentInline {
    /// Base expression.
    pub base: Option<EcoString>,
    /// Accent.
    pub accent: Option<EcoString>,
    /// Accent size.
    pub size: Option<EcoString>,
    /// Whether the base is dotless.
    pub dotless: bool,
}

/// Math attachment.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathAttachInline {
    /// Base expression.
    pub base: Option<EcoString>,
    /// Top attachment.
    pub t: Option<EcoString>,
    /// Bottom attachment.
    pub b: Option<EcoString>,
    /// Top-left attachment.
    pub tl: Option<EcoString>,
    /// Bottom-left attachment.
    pub bl: Option<EcoString>,
    /// Top-right attachment.
    pub tr: Option<EcoString>,
    /// Bottom-right attachment.
    pub br: Option<EcoString>,
}

/// Math binomial.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathBinomInline {
    /// Upper expression.
    pub upper: Option<EcoString>,
    /// Lower expression.
    pub lower: Option<EcoString>,
}

/// Math cancel.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathCancelInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Stroke length.
    pub length: Option<EcoString>,
    /// Whether cancellation is inverted.
    pub inverted: bool,
    /// Whether cancellation is crossed.
    pub cross: bool,
    /// Cancellation angle.
    pub angle: Option<EcoString>,
    /// Stroke paint.
    pub stroke: Option<EcoString>,
}

/// Math cases.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathCasesInline {
    /// Delimiter.
    pub delim: Option<EcoString>,
    /// Whether cases are reversed.
    pub reverse: bool,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Children.
    pub children: Vec<Inline>,
}

/// Math class.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathClassInline {
    /// Class.
    pub class: Option<EcoString>,
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math fraction.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathFracInline {
    /// Numerator.
    pub num: Option<EcoString>,
    /// Denominator.
    pub denom: Option<EcoString>,
    /// Fraction style.
    pub style: Option<EcoString>,
}

/// Math limits.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathLimitsInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Whether limits are inline.
    pub inline: bool,
}

/// Math left-right group.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathLrInline {
    /// Size.
    pub size: Option<EcoString>,
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math matrix.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathMatInline {
    /// Delimiter.
    pub delim: Option<EcoString>,
    /// Alignment.
    pub align: Option<EcoString>,
    /// Augment.
    pub augment: Option<EcoString>,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Row gap.
    pub row_gap: Option<EcoString>,
    /// Column gap.
    pub column_gap: Option<EcoString>,
    /// Rows.
    pub rows: Option<EcoString>,
}

/// Math middle delimiter.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathMidInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math operator.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOpInline {
    /// Operator text.
    pub text: Option<EcoString>,
    /// Whether limits are used.
    pub limits: bool,
}

/// Math overbrace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverbraceInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math overbracket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverbracketInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math overline.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverlineInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math overparen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOverparenInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math overshell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathOvershellInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math primes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathPrimesInline {
    /// Prime count.
    pub count: Option<EcoString>,
}

/// Math root.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathRootInline {
    /// Root index.
    pub index: Option<EcoString>,
    /// Radicand.
    pub radicand: Option<EcoString>,
}

/// Math scripts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathScriptsInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math stretch.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathStretchInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Stretch size.
    pub size: Option<EcoString>,
}

/// Math underbrace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderbraceInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math underbracket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderbracketInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math underline.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderlineInline {
    /// Body expression.
    pub body: Option<EcoString>,
}

/// Math underparen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUnderparenInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math undershell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathUndershellInline {
    /// Body expression.
    pub body: Option<EcoString>,
    /// Annotation expression.
    pub annotation: Option<EcoString>,
}

/// Math vector.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MathVecInline {
    /// Delimiter.
    pub delim: Option<EcoString>,
    /// Alignment.
    pub align: Option<EcoString>,
    /// Gap.
    pub gap: Option<EcoString>,
    /// Children.
    pub children: Vec<Inline>,
}
