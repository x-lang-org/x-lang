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
    let program = parser
        .parse(&content)
        .map_err(|e| pipeline::format_parse_error(file, &content, &e))?;

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
        Ok(program) => {
            if let Err(e) = pipeline::type_check_with_big_stack_formatted(&program, &path_str, &content) {
                crate::utils::error(&e);
                *error_count += 1;
            }
        }
        Err(e) => {
            crate::utils::error(&pipeline::format_parse_error(
                &path_str,
                &content,
                &e,
            ));
            *error_count += 1;
        }
    }
    Ok(())
}
