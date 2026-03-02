use crate::project::Project;
use crate::utils;

pub fn exec(file: Option<&str>, check: bool, _all: bool) -> Result<(), String> {
    if let Some(f) = file {
        return format_file(f, check);
    }

    let project = Project::find()?;

    utils::status(
        "Formatting",
        &format!("{} v{}", project.name(), project.version()),
    );

    let source_files = project.source_files();
    if source_files.is_empty() {
        utils::note("未找到源文件");
        return Ok(());
    }

    let mut formatted = 0;
    let mut unchanged = 0;

    for path in &source_files {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;

        let formatted_content = format_source(&content);

        if formatted_content != content {
            if check {
                utils::error(&format!("未格式化: {}", path.display()));
                formatted += 1;
            } else {
                std::fs::write(path, &formatted_content)
                    .map_err(|e| format!("无法写入 {}: {}", path.display(), e))?;
                formatted += 1;
            }
        } else {
            unchanged += 1;
        }
    }

    if check && formatted > 0 {
        return Err(format!("{} 个文件需要格式化", formatted));
    }

    utils::status(
        "Finished",
        &format!("已格式化 {} 个文件, {} 个无需更改", formatted, unchanged),
    );
    Ok(())
}

fn format_file(file: &str, check: bool) -> Result<(), String> {
    let content =
        std::fs::read_to_string(file).map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    let formatted = format_source(&content);

    if formatted != content {
        if check {
            return Err(format!("{} 需要格式化", file));
        }
        std::fs::write(file, &formatted)
            .map_err(|e| format!("无法写入 {}: {}", file, e))?;
        utils::status("Formatted", file);
    } else {
        utils::status("Unchanged", file);
    }

    Ok(())
}

fn format_source(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut indent_level: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            result.push('\n');
            continue;
        }

        if trimmed.starts_with('}') || trimmed.starts_with(')') || trimmed.starts_with(']') {
            indent_level = (indent_level - 1).max(0);
        }

        for _ in 0..indent_level {
            result.push_str("    ");
        }
        result.push_str(trimmed);
        result.push('\n');

        let opens = trimmed.chars().filter(|&c| c == '{').count() as i32;
        let closes = trimmed.chars().filter(|&c| c == '}').count() as i32;
        let net = opens - closes;

        if trimmed.starts_with('}') {
            indent_level = (indent_level + net + 1).max(0);
        } else {
            indent_level = (indent_level + net).max(0);
        }
    }

    if !result.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }

    result
}
