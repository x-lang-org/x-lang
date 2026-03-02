use crate::project::Project;
use crate::utils;

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

    let fixed = 0;
    for path in &source_files {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;

        let parser = x_parser::parser::XParser::new();
        match parser.parse(&content) {
            Ok(_program) => {
                // TODO: Apply automatic fixes
                // - Remove unused imports
                // - Fix deprecated syntax
                // - Apply type-suggested corrections
            }
            Err(_) => {
                // Skip files that don't parse
            }
        }
    }

    utils::status("Finished", &format!("已修复 {} 个问题", fixed));
    Ok(())
}
