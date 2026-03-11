//! Emacs syntax definition generator

use std::path::Path;

use anyhow::Result;

use crate::model::SyntaxModel;

/// Generate Emacs syntax definition
pub fn generate(model: &SyntaxModel, output_dir: &Path) -> Result<()> {
    // TODO: Implement Emacs syntax generation
    Ok(())
}
