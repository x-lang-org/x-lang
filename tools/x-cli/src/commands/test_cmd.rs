use crate::pipeline;
use crate::project::Project;
use crate::utils;
use colored::*;
use std::io::Write;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Default, PartialEq, Eq)]
struct IntegrationExpectations {
    exit_code: i32,
    stdout_lines: Vec<String>,
    stdin: Option<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn integration_tests_dir() -> PathBuf {
    repo_root().join("tests/integration")
}

fn x_cli_candidates() -> [PathBuf; 4] {
    let root = repo_root();
    [
        root.join("tools/target/debug/x.exe"),
        root.join("tools/target/release/x.exe"),
        root.join("tools/target/debug/x"),
        root.join("tools/target/release/x"),
    ]
}

fn parse_integration_expectations(content: &str) -> IntegrationExpectations {
    let mut expectations = IntegrationExpectations {
        exit_code: 0,
        stdout_lines: Vec::new(),
        stdin: None,
    };

    for line in content.lines() {
        let trimmed_start = line.trim_start();
        if let Some(value) = trimmed_start.strip_prefix("// @exit_code:") {
            expectations.exit_code = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = trimmed_start.strip_prefix("// @stdin:") {
            let value = value.strip_prefix(' ').unwrap_or(value);
            expectations.stdin = Some(
                value
                    .replace("\\r", "\r")
                    .replace("\\n", "\n")
                    .replace("\\t", "\t"),
            );
        } else if let Some(value) = trimmed_start.strip_prefix("// @stdout:") {
            expectations.stdout_lines.push(value.trim().to_string());
        } else if trimmed_start.starts_with("// @") {
            continue;
        } else if !trimmed_start.trim().is_empty() {
            break;
        }
    }

    expectations
}

fn verify_integration_output(
    expectations: &IntegrationExpectations,
    exit_code: i32,
    stdout: &str,
) -> Result<(), String> {
    if exit_code != expectations.exit_code {
        return Err(format!(
            "exit code mismatch: expected {}, got {}",
            expectations.exit_code, exit_code
        ));
    }

    let normalized_stdout = stdout.replace("\r\n", "\n");
    let mut search_start = 0usize;
    for expected in &expectations.stdout_lines {
        let Some(relative_idx) = normalized_stdout[search_start..].find(expected) else {
            return Err(format!("stdout missing expected fragment in order: {}", expected));
        };
        search_start += relative_idx + expected.len();
    }

    Ok(())
}

fn discover_integration_tests(integration_dir: &std::path::Path) -> Vec<(String, PathBuf)> {
    let mut tests = Vec::new();

    for entry in walkdir::WalkDir::new(integration_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "x"))
    {
        let path = entry.into_path();
        let relative = path
            .strip_prefix(integration_dir)
            .expect("integration test path should be under integration dir");
        let depth = relative.components().count();
        let is_top_level_file = depth == 1;
        let is_category_test_file = depth == 2;
        let is_nested_entrypoint = relative
            .file_name()
            .is_some_and(|file_name| file_name == "main.x")
            && depth > 2;

        if !is_top_level_file && !is_category_test_file && !is_nested_entrypoint {
            continue;
        }

        let category = if relative.parent().is_none_or(|parent| parent.as_os_str().is_empty()) {
            "root".to_string()
        } else {
            relative
                .components()
                .next()
                .and_then(|component| component.as_os_str().to_str())
                .map(str::to_string)
                .unwrap_or_else(|| "root".to_string())
        };
        tests.push((category, path));
    }

    tests.sort_by(|(category_a, path_a), (category_b, path_b)| {
        category_a
            .cmp(category_b)
            .then_with(|| path_a.cmp(path_b))
    });

    tests
}

fn parse_integration_filter(filter: Option<&str>) -> Option<Option<&str>> {
    match filter {
        Some("integration") => Some(None),
        Some(value) => value.strip_prefix("integration/").map(Some),
        None => None,
    }
}

fn filter_integration_tests_by_category(
    tests: Vec<(String, PathBuf)>,
    category: Option<&str>,
) -> Vec<(String, PathBuf)> {
    match category {
        None => tests,
        Some(category) => tests
            .into_iter()
            .filter(|(test_category, _)| test_category == category)
            .collect(),
    }
}

#[allow(unused_variables)]
pub fn exec(
    filter: Option<&str>,
    release: bool,
    lib: bool,
    doc: bool,
    no_run: bool,
    jobs: Option<u32>,
    verbose: bool,
) -> Result<(), String> {
    if let Some(category) = parse_integration_filter(filter) {
        return run_integration_tests(category, verbose);
    }

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
        test_files.retain(|p| p.to_str().is_some_and(|s| s.contains(pattern)));
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
            Ok(mut program) => {
                if let Err(e) = pipeline::inject_std_prelude(&mut program) {
                    errors.push(format!("{}: 无法加载 prelude: {}", name, e));
                    failed += 1;
                    continue;
                }

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

pub fn run_integration_tests(category: Option<&str>, verbose: bool) -> Result<(), String> {
    use std::process::{Command, Stdio};

    let integration_dir = integration_tests_dir();
    if !integration_dir.exists() {
        utils::note(&format!(
            "integration tests directory not found: {}",
            integration_dir.display()
        ));
        utils::note("create tests in: tests/integration/<category>/<test>.x");
        return Ok(());
    }

    let x_cli_path = find_x_cli()?;

    println!("\n{}", "=".repeat(60));
    println!("X Language Integration Test Suite");
    println!("{}", "=".repeat(60));

    let mut passed = 0usize;
    let mut failed = 0usize;
    let skipped = 0usize;
    let mut total = 0usize;

    let tests = filter_integration_tests_by_category(discover_integration_tests(&integration_dir), category);
    if tests.is_empty() {
        let target = category.unwrap_or("<all>");
        return Err(format!("未找到 integration 测试分类 `{}`", target));
    }
    let mut current_category: Option<String> = None;

    for (category, path) in tests {
        if current_category.as_deref() != Some(category.as_str()) {
            println!("\n{} tests:", category);
            current_category = Some(category.clone());
        }

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                println!("{}", "ERROR".red());
                if verbose {
                    println!("    failed to read {}: {}", path.display(), e);
                }
                failed += 1;
                total += 1;
                continue;
            }
        };
        let expectations = parse_integration_expectations(&content);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        total += 1;

        if verbose {
            print!("  {}::{} ... ", category, name);
        } else {
            print!("  {} ... ", name);
        }

        let mut command = Command::new(&x_cli_path);
        command.arg("run").arg(&path);

        if expectations.stdin.is_some() {
            command.stdin(Stdio::piped());
        }

        let result = if let Some(stdin) = &expectations.stdin {
            match command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
                Ok(mut child) => {
                    if let Some(mut child_stdin) = child.stdin.take() {
                        if let Err(e) = child_stdin.write_all(stdin.as_bytes()) {
                            Err(e)
                        } else {
                            drop(child_stdin);
                            child.wait_with_output()
                        }
                    } else {
                        child.wait_with_output()
                    }
                }
                Err(e) => Err(e),
            }
        } else {
            command.output()
        };

        match result {
            Ok(output) => {
                let exit_code = output.status.code().unwrap_or(-1);
                let stdout = String::from_utf8_lossy(&output.stdout);
                match verify_integration_output(&expectations, exit_code, &stdout) {
                    Ok(()) => {
                        println!("{}", "ok".green());
                        passed += 1;
                    }
                    Err(message) => {
                        println!("{}", "FAILED".red());
                        if verbose {
                            println!("    {}", message);
                            println!("    exit code: {}", exit_code);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            if !stderr.trim().is_empty() {
                                println!("    stderr:");
                                for line in stderr.lines().take(8) {
                                    println!("      {}", line);
                                }
                            }

                            if !stdout.trim().is_empty() {
                                println!("    stdout:");
                                for line in stdout.lines().take(8) {
                                    println!("      {}", line);
                                }
                            }
                        }
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                println!("{}", "ERROR".red());
                if verbose {
                    println!("    {}", e);
                }
                failed += 1;
            }
        }
    }

    println!("\n{}", "=".repeat(60));
    print!("test result: ");
    if failed > 0 {
        print!("{}", "FAILED".red().bold());
    } else {
        print!("{}", "ok".green().bold());
    }
    println!(
        ". {} passed; {} failed; {} skipped; {} total",
        passed, failed, skipped, total
    );
    println!("{}", "=".repeat(60));

    if failed > 0 {
        Err(format!("{} integration tests failed", failed))
    } else {
        Ok(())
    }
}

fn find_x_cli() -> Result<PathBuf, String> {
    for candidate in x_cli_candidates() {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    if let Ok(path) = which::which("x") {
        return Ok(path);
    }

    Err("Could not find x-cli. Build it first: cd tools/x-cli && cargo build --release".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_root_contains_workspace_markers() {
        let root = repo_root();
        assert!(root.join("tools").exists(), "repo root should contain tools/: {}", root.display());
        assert!(root.join("tests").exists(), "repo root should contain tests/: {}", root.display());
    }

    #[test]
    fn integration_tests_dir_exists() {
        let dir = integration_tests_dir();
        assert!(dir.exists(), "integration tests dir should exist: {}", dir.display());
    }

    #[test]
    fn find_x_cli_resolves_existing_binary() {
        let resolved = find_x_cli().expect("should find built x-cli binary");
        assert!(resolved.exists(), "resolved path should exist: {}", resolved.display());
        assert_eq!(resolved.file_stem().and_then(|stem| stem.to_str()), Some("x"));
    }

    #[test]
    fn parse_integration_expectations_collects_repeated_stdout_lines() {
        let content = r#"
// @stdout: Hello
// @stdout: World
// @exit_code: 0

println("Hello")
"#;
        let expectations = parse_integration_expectations(content);
        assert_eq!(
            expectations,
            IntegrationExpectations {
                exit_code: 0,
                stdout_lines: vec!["Hello".to_string(), "World".to_string()],
                stdin: None,
            }
        );
    }

    #[test]
    fn parse_integration_expectations_collects_stdin() {
        let content = r#"
// @stdin: hello from stdin\n
// @stdout: hello from stdin

println("placeholder")
"#;

        let expectations = parse_integration_expectations(content);
        assert_eq!(expectations.stdin.as_deref(), Some("hello from stdin\n"));
        assert_eq!(expectations.stdout_lines, vec!["hello from stdin".to_string()]);
    }

    #[test]
    fn parse_integration_expectations_preserves_stdin_boundary_spaces() {
        let content = r#"
// @stdin:   padded input  

println("placeholder")
"#;

        let expectations = parse_integration_expectations(content);
        assert_eq!(expectations.stdin.as_deref(), Some("  padded input  "));
    }

    #[test]
    fn verify_integration_output_requires_ordered_stdout_fragments() {
        let expectations = IntegrationExpectations {
            exit_code: 0,
            stdout_lines: vec!["Hello".to_string(), "World".to_string()],
            stdin: None,
        };
        assert!(verify_integration_output(&expectations, 0, "Hello\nWorld\n").is_ok());
        assert!(verify_integration_output(&expectations, 0, "World\nHello\n").is_err());
    }

    #[test]
    fn verify_integration_output_checks_exit_code() {
        let expectations = IntegrationExpectations::default();
        let err = verify_integration_output(&expectations, 1, "").unwrap_err();
        assert!(err.contains("exit code mismatch"));
    }

    #[test]
    fn discover_integration_tests_includes_nested_and_root_cases() {
        let tests = discover_integration_tests(&integration_tests_dir());
        let paths: Vec<_> = tests
            .iter()
            .map(|(category, path)| (category.clone(), path.file_name().unwrap().to_string_lossy().to_string()))
            .collect();

        assert!(paths.iter().any(|(category, name)| category == "modules" && name == "main.x"));
        assert!(paths.iter().any(|(category, name)| category == "root" && name == "error_handling.x"));
        assert!(!paths.iter().any(|(category, _)| category == "error_handling.x"));
        assert!(!paths.iter().any(|(category, name)| category == "modules" && name == "helper.x"));
        assert!(!paths.iter().any(|(category, name)| category == "modules" && name == "base.x"));
        assert!(!paths.iter().any(|(category, name)| category == "modules" && name == "a.x"));
        assert!(!paths.iter().any(|(category, name)| category == "modules" && name == "b.x"));
    }

    #[test]
    fn parse_integration_filter_supports_full_suite_and_category_runs() {
        assert_eq!(parse_integration_filter(Some("integration")), Some(None));
        assert_eq!(parse_integration_filter(Some("integration/basic")), Some(Some("basic")));
        assert_eq!(parse_integration_filter(Some("integration/stdlib")), Some(Some("stdlib")));
        assert_eq!(parse_integration_filter(Some("basic")), None);
        assert_eq!(parse_integration_filter(None), None);
    }

    #[test]
    fn filter_integration_tests_by_category_limits_results() {
        let tests = discover_integration_tests(&integration_tests_dir());
        let basic_only = filter_integration_tests_by_category(tests.clone(), Some("basic"));
        assert!(!basic_only.is_empty(), "basic category should contain tests");
        assert!(basic_only.iter().all(|(category, _)| category == "basic"));

        let all = filter_integration_tests_by_category(tests, None);
        assert!(all.iter().any(|(category, _)| category == "basic"));
        assert!(all.iter().any(|(category, _)| category == "types"));
    }
}
