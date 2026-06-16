//! Shared shell types for typlite.

/// Valid output formats for typlite conversion.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// GitHub-flavored Markdown output.
    #[default]
    Md,
    /// LaTeX output.
    LaTeX,
    /// Plain text output.
    Text,
    /// DOCX output.
    #[cfg(feature = "docx")]
    Docx,
}
