use crate::pipeline;
use crate::project::Project;
use crate::utils;
use std::time::Instant;

pub fn exec(filter: Option<&str>, no_run: bool) -> Result<(), String> {
    let project = Project::find()?;
    let start = Instant::now();

    utils::status(
        "Benchmarking",
        &format!("{} v{}", project.name(), project.version()),
    );

    let mut bench_files = project.bench_files();

    if bench_files.is_empty() {
        utils::note("未找到基准测试文件");
        utils::note("在 benches/ 目录下创建 .x 文件来添加基准测试");
        return Ok(());
    }

    if let Some(pattern) = filter {
        bench_files.retain(|p| p.to_str().map_or(false, |s| s.contains(pattern)));
    }

    for path in &bench_files {
        let name = path
            .strip_prefix(&project.root)
            .unwrap_or(path)
            .display()
            .to_string();

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                utils::error(&format!("{}: 无法读取: {}", name, e));
                continue;
            }
        };

        let parser = x_parser::parser::XParser::new();
        match parser.parse(&content) {
            Ok(program) => {
                if let Err(e) = x_typechecker::type_check(&program) {
                    utils::error(&format!("{}: 类型检查失败: {}", name, e));
                    continue;
                }

                if !no_run {
                    let bench_start = Instant::now();
                    let mut interpreter = x_interpreter::Interpreter::new();
                    match interpreter.run(&program) {
                        Ok(()) => {
                            let bench_elapsed = bench_start.elapsed();
                            println!("bench {} ... {} ns/iter", name, bench_elapsed.as_nanos());
                        }
                        Err(e) => {
                            utils::error(&format!("{}: {}", name, e));
                        }
                    }
                }
            }
            Err(e) => {
                utils::error(&pipeline::format_parse_error(
                    &path.display().to_string(),
                    &content,
                    &e,
                ));
            }
        }
    }

    let elapsed = start.elapsed();
    utils::status(
        "Finished",
        &format!("bench target(s) in {}", utils::elapsed_str(elapsed)),
    );
    Ok(())
}
