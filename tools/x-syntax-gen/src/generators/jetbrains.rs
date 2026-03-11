//! JetBrains IDE syntax definition generator

use std::path::Path;

use anyhow::Result;

use crate::model::SyntaxModel;

/// Generate JetBrains IDE syntax definition
pub fn generate(model: &SyntaxModel, output_dir: &Path) -> Result<()> {
    // TODO: Implement JetBrains syntax generation
    Ok(())
}
