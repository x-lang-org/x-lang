//! Neovim Tree-sitter syntax definition generator

use std::path::Path;

use anyhow::Result;

use crate::model::SyntaxModel;

/// Generate Neovim Tree-sitter syntax definition
pub fn generate(model: &SyntaxModel, output_dir: &Path) -> Result<()> {
    // TODO: Implement Tree-sitter grammar generation
    Ok(())
}
