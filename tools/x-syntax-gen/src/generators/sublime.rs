//! Sublime Text syntax definition generator

use std::path::Path;

use anyhow::Result;

use crate::model::SyntaxModel;

/// Generate Sublime Text syntax definition
pub fn generate(model: &SyntaxModel, output_dir: &Path) -> Result<()> {
    // TODO: Implement Sublime Text syntax generation
    Ok(())
}
