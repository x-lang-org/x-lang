//! Vim syntax definition generator

use std::path::Path;

use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;

use crate::model::SyntaxModel;

/// Generate Vim syntax definition
pub fn generate(model: &SyntaxModel, output_dir: &Path) -> Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("vim", "templates/vim.vim.hbs")?;

    let data = json!({
        "language_name": model.language_name,
        "file_extensions": model.file_extensions,
        "keywords": model.keywords,
        "types": model.types,
        "builtins": model.builtins,
        "comment_line": model.comment.line,
        "comment_block_start": model.comment.block_start,
        "comment_block_end": model.comment.block_end,
    });

    let output = handlebars.render("vim", &data)?;
    let output_path = output_dir.join("x.vim");
    std::fs::write(&output_path, output)?;

    Ok(())
}
