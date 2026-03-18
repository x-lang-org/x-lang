use crate::pipeline;
use crate::project::Project;
use crate::utils;
use colored::*;
use std::time::Instant;

#[allow(unused_variables)]
pub fn exec(
    filter: Option<&str>,
    release: bool,
    lib: bool,
    doc: bool,
    no_run: bool,
    jobs: Option<u32>,
) -> Result<(), String> {
    let project = Project::find()?;
    let start = Instant::now();

    utils::status(
        "Testing",
        &format!(
            "{} v{} ({})",
            project.name(),
            project.version(),
            project.root.display()
        ),
    );

    let mut test_files = Vec::new();
    test_files.extend(project.test_files());

    if lib {
        test_files.extend(project.source_files());
    }

    if test_files.is_empty() {
        utils::note("未找到测试文件");
        utils::note("在 tests/ 目录下创建 .x 文件来添加测试");
        return Ok(());
    }

    if let Some(pattern) = filter {
        test_files.retain(|p| p.to_str().map_or(false, |s| s.contains(pattern)));
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for path in &test_files {
        let name = path
            .strip_prefix(&project.root)
            .unwrap_or(path)
            .display()
            .to_string();

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("{}: 无法读取: {}", name, e));
                failed += 1;
                continue;
            }
        };

        let parser = x_parser::parser::XParser::new();
        match parser.parse(&content) {
            Ok(program) => {
                if let Err(e) = x_typechecker::type_check(&program) {
                    errors.push(format!("{}: 类型检查失败: {}", name, e));
                    failed += 1;
                    continue;
                }

                if !no_run {
                    let mut interpreter = x_interpreter::Interpreter::new();
                    match interpreter.run(&program) {
                        Ok(()) => {
                            println!("test {} ... {}", name, "ok".green());
                            passed += 1;
                        }
                        Err(e) => {
                            println!("test {} ... {}", name, "FAILED".red());
                            errors.push(format!("{}: {}", name, e));
                            failed += 1;
                        }
                    }
                } else {
                    println!("test {} ... {}", name, "ok (no run)".yellow());
                    passed += 1;
                }
            }
            Err(e) => {
                println!("test {} ... {}", name, "FAILED".red());
                errors.push(pipeline::format_parse_error(
                    &path.display().to_string(),
                    &content,
                    &e,
                ));
                failed += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    println!();

    if !errors.is_empty() {
        println!("failures:");
        for err in &errors {
            println!("  {}", err);
        }
        println!();
    }

    let result_str = if failed > 0 {
        format!("{}", "FAILED".red().bold())
    } else {
        format!("{}", "ok".green().bold())
    };

    println!(
        "test result: {}. {} passed; {} failed; finished in {}",
        result_str,
        passed,
        failed,
        utils::elapsed_str(elapsed)
    );

    if failed > 0 {
        Err(format!("{} 个测试失败", failed))
    } else {
        Ok(())
    }
}
