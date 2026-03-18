//! VS Code syntax definition generator

use std::path::Path;

use anyhow::Result;
use handlebars::Handlebars;
use serde_json::json;

use crate::model::SyntaxModel;

/// Generate VS Code syntax definition
pub fn generate(model: &SyntaxModel, output_dir: &Path) -> Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("vscode", "templates/vscode.json.hbs")?;

    let data = json!({
        "language_name": model.language_name,
        "file_extensions": model.file_extensions,
        "rules": model.rules,
        "keywords": model.keywords,
        "types": model.types,
        "builtins": model.builtins,
        "operators": model.operators,
        "comment_line": model.comment.line,
        "comment_block_start": model.comment.block_start,
        "comment_block_end": model.comment.block_end,
    });

    let output = handlebars.render("vscode", &data)?;
    let output_path = output_dir.join("x.tmLanguage.json");
    std::fs::write(&output_path, output)?;

    Ok(())
}
