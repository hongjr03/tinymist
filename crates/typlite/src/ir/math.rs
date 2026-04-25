use ecow::EcoString;

/// A Typst math expression node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathNode {
    /// Typst math function name.
    pub func: EcoString,
    /// Function fields.
    pub fields: Vec<MathField>,
}

/// A Typst math node field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MathField {
    /// Field name.
    pub name: EcoString,
    /// Field value.
    pub value: MathValue,
}

/// A Typst math field value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathValue {
    /// Absent value.
    None,
    /// Boolean value.
    Bool(bool),
    /// Scalar value.
    Scalar(EcoString),
    /// Nested math expression.
    Node(Box<MathNode>),
    /// Math expression list.
    Nodes(Vec<MathNode>),
    /// Two-dimensional math expression list.
    Rows(Vec<Vec<MathNode>>),
}
