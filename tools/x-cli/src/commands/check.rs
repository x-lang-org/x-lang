use crate::pipeline;
use crate::project::Project;
use std::time::Instant;

pub fn exec(file: Option<&str>, all_targets: bool) -> Result<(), String> {
    if let Some(f) = file {
        return check_file(f);
    }

    let project = Project::find()?;
    let start = Instant::now();

    crate::utils::status(
        "Checking",
        &format!(
            "{} v{} ({})",
            project.name(),
            project.version(),
            project.root.display()
        ),
    );

    let source_files = project.source_files();
    let mut error_count = 0;

    for path in &source_files {
        check_single_file(path, &mut error_count)?;
    }

    if all_targets {
        for path in project.test_files() {
            check_single_file(&path, &mut error_count)?;
        }
        for path in project.example_files() {
            check_single_file(&path, &mut error_count)?;
        }
    }

    let elapsed = start.elapsed();
    if error_count > 0 {
        Err(format!("检查发现 {} 个错误", error_count))
    } else {
        crate::utils::status(
            "Finished",
            &format!(
                "`dev` profile [unoptimized + debuginfo] target(s) in {}",
                crate::utils::elapsed_str(elapsed)
            ),
        );
        Ok(())
    }
}

fn check_file(file: &str) -> Result<(), String> {
    let content =
        std::fs::read_to_string(file).map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    let parser = x_parser::parser::XParser::new();
    let mut program = parser
        .parse(&content)
        .map_err(|e| pipeline::format_parse_error(file, &content, &e))?;

    // 解析模块导入：使用源文件所在目录作为项目根目录
    let stdlib_dir = crate::pipeline::find_stdlib_path()?;
    let project_dir = std::path::Path::new(file)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    crate::pipeline::resolve_imports(&mut program, &stdlib_dir, &project_dir)?;

    crate::pipeline::inject_std_prelude(&mut program)?;

    pipeline::type_check_with_big_stack_formatted(&program, file, &content)?;

    crate::utils::status("Finished", "检查通过（语法 + 类型）");
    Ok(())
}

fn check_single_file(path: &std::path::Path, error_count: &mut usize) -> Result<(), String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;

    let parser = x_parser::parser::XParser::new();
    let path_str = path.display().to_string();
    match parser.parse(&content) {
        Ok(mut program) => {
            // 解析模块导入：使用源文件所在目录作为项目根目录
            let stdlib_dir = crate::pipeline::find_stdlib_path()?;
            let project_dir = path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf();
            if let Err(e) =
                crate::pipeline::resolve_imports(&mut program, &stdlib_dir, &project_dir)
            {
                crate::utils::error(&e);
                *error_count += 1;
                return Ok(());
            }

            // 自动导入标准库 prelude
            if let Err(e) = crate::pipeline::inject_std_prelude(&mut program) {
                crate::utils::error(&e);
                *error_count += 1;
                return Ok(());
            }
            if let Err(e) =
                pipeline::type_check_with_big_stack_formatted(&program, &path_str, &content)
            {
                crate::utils::error(&e);
                *error_count += 1;
            }
        }
        Err(e) => {
            crate::utils::error(&pipeline::format_parse_error(&path_str, &content, &e));
            *error_count += 1;
        }
    }
    Ok(())
}
