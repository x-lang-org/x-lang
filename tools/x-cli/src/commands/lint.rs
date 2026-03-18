use crate::pipeline;
use crate::project::Project;
use crate::utils;
use colored::*;
use std::time::Instant;

#[allow(unused_variables)]
pub fn exec(
    fix: bool,
    allow: Vec<String>,
    deny: Vec<String>,
    warn: Vec<String>,
) -> Result<(), String> {
    let project = Project::find()?;
    let start = Instant::now();

    utils::status(
        "Linting",
        &format!("{} v{}", project.name(), project.version()),
    );

    let source_files = project.source_files();
    let mut warning_count = 0;
    let mut error_count = 0;

    for path in &source_files {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                utils::error(&format!("{}: {}", path.display(), e));
                error_count += 1;
                continue;
            }
        };

        let rel_path = path
            .strip_prefix(&project.root)
            .unwrap_or(path)
            .display()
            .to_string();

        let parser = x_parser::parser::XParser::new();
        match parser.parse(&content) {
            Ok(_program) => {
                let warnings = run_lint_checks(&content, &rel_path);
                for w in &warnings {
                    println!("{}: {}", "warning".yellow().bold(), w);
                    warning_count += 1;
                }
            }
            Err(e) => {
                utils::error(&pipeline::format_parse_error(&rel_path, &content, &e));
                error_count += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    if error_count > 0 || warning_count > 0 {
        println!();
        println!(
            "{}: {} generated {} warning(s), {} error(s) in {}",
            "Finished".green().bold(),
            project.name(),
            warning_count,
            error_count,
            utils::elapsed_str(elapsed)
        );
    } else {
        utils::status(
            "Finished",
            &format!("no warnings or errors in {}", utils::elapsed_str(elapsed)),
        );
    }

    if error_count > 0 {
        Err(format!("{} 个错误", error_count))
    } else {
        Ok(())
    }
}

fn run_lint_checks(source: &str, file: &str) -> Vec<String> {
    let mut warnings = Vec::new();

    for (i, line) in source.lines().enumerate() {
        let line_num = i + 1;

        if line != line.trim_end() {
            warnings.push(format!("{}:{}: 行尾有多余的空白字符", file, line_num));
        }

        if line.len() > 120 {
            warnings.push(format!(
                "{}:{}: 行长度超过 120 个字符 ({})",
                file,
                line_num,
                line.len()
            ));
        }

        if line.contains('\t') {
            warnings.push(format!("{}:{}: 使用了制表符，建议使用空格", file, line_num));
        }
    }

    if !source.is_empty() && !source.ends_with('\n') {
        warnings.push(format!("{}: 文件末尾缺少换行符", file));
    }

    warnings
}
