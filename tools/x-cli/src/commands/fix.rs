use crate::project::Project;
use crate::utils;
use std::path::Path;

pub fn exec(allow_dirty: bool, allow_staged: bool) -> Result<(), String> {
    let project = Project::find()?;

    utils::status(
        "Fixing",
        &format!("{} v{}", project.name(), project.version()),
    );

    let source_files = project.source_files();
    if source_files.is_empty() {
        utils::note("未找到源文件");
        return Ok(());
    }

    if !allow_dirty && !allow_staged {
        let git_status = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&project.root)
            .output();
        if let Ok(output) = git_status {
            let status_str = String::from_utf8_lossy(&output.stdout);
            if !status_str.is_empty() {
                return Err(
                    "工作目录有未提交的更改，使用 --allow-dirty 或 --allow-staged 跳过检查"
                        .to_string(),
                );
            }
        }
    }

    let mut total_fixes = 0;
    let mut files_fixed = 0;

    for path in &source_files {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;

        let parser = x_parser::parser::XParser::new();
        let parse_result = parser.parse(&content);

        let (fixed_content, fix_count) = apply_fixes(&content, path, parse_result.is_ok());

        if fix_count > 0 {
            std::fs::write(path, &fixed_content)
                .map_err(|e| format!("无法写入 {}: {}", path.display(), e))?;
            total_fixes += fix_count;
            files_fixed += 1;
            utils::status(
                "Fixed",
                &format!("{} ({} issues)", path.display(), fix_count),
            );
        }
    }

    if total_fixes == 0 {
        utils::status("Finished", "没有需要修复的问题");
    } else {
        utils::status(
            "Finished",
            &format!("已修复 {} 个问题，涉及 {} 个文件", total_fixes, files_fixed),
        );
    }
    Ok(())
}

/// Apply automatic fixes to source code
fn apply_fixes(content: &str, path: &Path, parses: bool) -> (String, usize) {
    let mut result = String::with_capacity(content.len());
    let mut fix_count = 0;

    // Fix 1: Remove trailing whitespace from each line
    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        let trimmed = line.trim_end();
        if trimmed.len() != line.len() {
            fix_count += 1;
        }
        result.push_str(trimmed);
        result.push('\n');
    }

    // Fix 2: Ensure file ends with newline
    if !content.is_empty() && !content.ends_with('\n') {
        fix_count += 1;
    }

    // Fix 3: Replace tabs with 4 spaces (only for files that parse)
    if parses {
        let with_spaces = result.replace('\t', "    ");
        if with_spaces != result {
            // Count tab replacements
            let tab_count = result.chars().filter(|&c| c == '\t').count();
            fix_count += tab_count;
            result = with_spaces;
        }
    }

    // Fix 4: Remove multiple consecutive blank lines (more than 2)
    let mut final_result = String::with_capacity(result.len());
    let mut blank_count = 0;
    let mut removed_blanks = 0;

    for line in result.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                final_result.push_str(line);
                final_result.push('\n');
            } else {
                removed_blanks += 1;
            }
        } else {
            blank_count = 0;
            final_result.push_str(line);
            final_result.push('\n');
        }
    }
    fix_count += removed_blanks;

    // Fix 5: Normalize line endings (already handled by the above)

    (final_result, fix_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_trailing_whitespace() {
        let content = "let x = 1   \nlet y = 2\t\n";
        let (fixed, count) = apply_fixes(content, Path::new("test.x"), true);
        assert_eq!(count, 2); // Two lines with trailing whitespace
        assert_eq!(fixed, "let x = 1\nlet y = 2\n");
    }

    #[test]
    fn test_ensure_trailing_newline() {
        let content = "let x = 1";
        let (fixed, count) = apply_fixes(content, Path::new("test.x"), true);
        assert_eq!(count, 1);
        assert!(fixed.ends_with('\n'));
    }

    #[test]
    fn test_replace_tabs_with_spaces() {
        let content = "let x = 1\n\tlet y = 2\n";
        let (fixed, count) = apply_fixes(content, Path::new("test.x"), true);
        assert!(count >= 1);
        assert!(!fixed.contains('\t'));
    }

    #[test]
    fn test_no_changes_needed() {
        let content = "let x = 1\nlet y = 2\n";
        let (fixed, count) = apply_fixes(content, Path::new("test.x"), true);
        assert_eq!(count, 0);
        assert_eq!(fixed, content);
    }

    #[test]
    fn test_remove_multiple_blank_lines() {
        let content = "let x = 1\n\n\n\n\nlet y = 2\n";
        let (fixed, count) = apply_fixes(content, Path::new("test.x"), true);
        assert!(count >= 2); // At least 2 blank lines removed
        // Should have at most 2 consecutive blank lines
        assert!(!fixed.contains("\n\n\n\n"));
    }
}
