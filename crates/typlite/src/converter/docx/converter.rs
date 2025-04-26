//! Core functionality for converting HTML to DOCX format

use std::path::Path;
use typst::html::HtmlElement;

use crate::converter::{FormatWriter, HtmlToAstParser};
use crate::Result;
use crate::TypliteFeat;

use crate::converter::docx::writer::DocxWriter;
use crate::converter::docx::config::DocxConfig;

/// DOCX Converter implementation
#[derive(Clone, Debug)]
pub struct DocxConverter {
    pub feat: TypliteFeat,
    pub config: Option<DocxConfig>,
}

impl DocxConverter {
    /// Create a new DOCX converter
    pub fn new(feat: TypliteFeat) -> Self {
        Self { feat, config: None }
    }
    
    /// Create a new DOCX converter with custom configuration
    pub fn with_config(feat: TypliteFeat, config: DocxConfig) -> Self {
        Self { feat, config: Some(config) }
    }
    
    /// Create a new DOCX converter with configuration from file
    pub fn from_config_file(feat: TypliteFeat, config_path: &Path) -> Result<Self> {
        match DocxConfig::from_file(config_path) {
            Ok(config) => Ok(Self { feat, config: Some(config) }),
            Err(err) => Err(format!("Failed to load DOCX configuration: {}", err).into()),
        }
    }

    /// Convert HTML element to DOCX format
    pub fn convert(&mut self, root: &HtmlElement) -> Result<Vec<u8>> {
        // Parse HTML to AST using shared parser
        let parser = HtmlToAstParser::new(self.feat.clone());
        let document = parser.parse(root)?;

        // Create and initialize DocxWriter with config if available
        let mut writer = if let Some(config) = self.config.clone() {
            DocxWriter::with_config(self.feat.clone(), config)
        } else {
            DocxWriter::new(self.feat.clone())
        };

        // Process AST using DocxWriter
        writer.write_vec(&document)
    }
}
