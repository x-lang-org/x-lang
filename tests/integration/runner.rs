use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;

fn repo_root() -> PathBuf {
    let source_path = PathBuf::from(file!());
    source_path
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .expect("tests/integration/runner.rs should live under the repository root")
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

#[derive(Debug, Clone)]
pub struct IntegrationTest {
    pub name: String,
    pub file_path: PathBuf,
    pub category: String,
    pub expected_exit_code: i32,
    pub expected_stdout: Option<String>,
    pub expected_stderr: Option<String>,
    pub should_compile_fail: bool,
    pub error_contains: Option<String>,
    pub skip: bool,
    pub skip_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestCategory {
    pub name: String,
    pub path: PathBuf,
    pub test_count: usize,
}

#[derive(Debug)]
pub struct TestResult {
    pub test: IntegrationTest,
    pub passed: bool,
    pub skipped: bool,
    pub message: Option<String>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Default)]
pub struct TestReport {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub results: Vec<TestResult>,
    pub total_duration_ms: u64,
    pub categories: HashMap<String, CategoryStats>,
}

#[derive(Debug, Default)]
pub struct CategoryStats {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

pub struct IntegrationTestRunner {
    x_cli_path: PathBuf,
    temp_dir: PathBuf,
    verbose: bool,
}

impl IntegrationTestRunner {
    pub fn new() -> Result<Self, String> {
        let x_cli_path = find_x_cli()?;
        let temp_dir = std::env::temp_dir().join("x-lang-integration-tests");
        fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

        Ok(Self {
            x_cli_path,
            temp_dir,
            verbose: false,
        })
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn discover_tests(&self, integration_dir: &Path) -> Result<Vec<IntegrationTest>, String> {
        let mut tests = Vec::new();

        if !integration_dir.exists() {
            return Err(format!(
                "Integration test directory not found: {}",
                integration_dir.display()
            ));
        }

        for entry in
            fs::read_dir(integration_dir).map_err(|e| format!("Failed to read directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let category = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                self.discover_tests_in_dir(&path, &category, &mut tests)?;
            }
        }

        tests.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(tests)
    }

    fn discover_tests_in_dir(
        &self,
        dir: &Path,
        category: &str,
        tests: &mut Vec<IntegrationTest>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "x") {
                let test = self.parse_test_file(&path, category)?;
                tests.push(test);
            }
        }

        Ok(())
    }

    fn parse_test_file(&self, path: &Path, category: &str) -> Result<IntegrationTest, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut test = IntegrationTest {
            name: name.clone(),
            file_path: path.to_path_buf(),
            category: category.to_string(),
            expected_exit_code: 0,
            expected_stdout: None,
            expected_stderr: None,
            should_compile_fail: false,
            error_contains: None,
            skip: false,
            skip_reason: None,
        };

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("// @exit_code:") {
                test.expected_exit_code = trimmed
                    .strip_prefix("// @exit_code:")
                    .unwrap_or("0")
                    .trim()
                    .parse()
                    .unwrap_or(0);
            } else if trimmed.starts_with("// @stdout:") {
                test.expected_stdout = Some(
                    trimmed
                        .strip_prefix("// @stdout:")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
            } else if trimmed.starts_with("// @stderr:") {
                test.expected_stderr = Some(
                    trimmed
                        .strip_prefix("// @stderr:")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
            } else if trimmed.starts_with("// @compile_fail") {
                test.should_compile_fail = true;
            } else if trimmed.starts_with("// @error_contains:") {
                test.error_contains = Some(
                    trimmed
                        .strip_prefix("// @error_contains:")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
            } else if trimmed.starts_with("// @skip") {
                test.skip = true;
                if let Some(reason) = trimmed.strip_prefix("// @skip:") {
                    test.skip_reason = Some(reason.trim().to_string());
                }
            } else if !trimmed.starts_with("// @") {
                break;
            }
        }

        Ok(test)
    }

    pub fn run_test(&self, test: &IntegrationTest) -> TestResult {
        let start = Instant::now();

        if test.skip {
            return TestResult {
                test: test.clone(),
                passed: true,
                skipped: true,
                message: test.skip_reason.clone(),
                duration_ms: start.elapsed().as_millis() as u64,
                stdout: String::new(),
                stderr: String::new(),
            };
        }

        let output = match self.execute_test(test) {
            Ok(o) => o,
            Err(e) => {
                return TestResult {
                    test: test.clone(),
                    passed: false,
                    skipped: false,
                    message: Some(e),
                    duration_ms: start.elapsed().as_millis() as u64,
                    stdout: String::new(),
                    stderr: String::new(),
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        let result = self.verify_test_result(test, exit_code, &stdout, &stderr);

        TestResult {
            test: test.clone(),
            passed: result.is_ok(),
            skipped: false,
            message: result.err(),
            duration_ms: start.elapsed().as_millis() as u64,
            stdout,
            stderr,
        }
    }

    fn execute_test(&self, test: &IntegrationTest) -> Result<Output, String> {
        Command::new(&self.x_cli_path)
            .arg("run")
            .arg(&test.file_path)
            .output()
            .map_err(|e| format!("Failed to execute x-cli: {}", e))
    }

    fn verify_test_result(
        &self,
        test: &IntegrationTest,
        exit_code: i32,
        stdout: &str,
        stderr: &str,
    ) -> Result<(), Option<String>> {
        if test.should_compile_fail {
            if exit_code == 0 {
                return Err(Some(
                    "Expected compilation failure but succeeded".to_string(),
                ));
            }

            if let Some(ref expected) = test.error_contains {
                if !stderr.contains(expected) && !stdout.contains(expected) {
                    return Err(Some(format!(
                        "Error message does not contain '{}'",
                        expected
                    )));
                }
            }

            return Ok(());
        }

        if exit_code != test.expected_exit_code {
            return Err(Some(format!(
                "Exit code mismatch: expected {}, got {}",
                test.expected_exit_code, exit_code
            )));
        }

        if let Some(ref expected) = test.expected_stdout {
            let normalized_stdout = stdout.replace("\r\n", "\n");
            let normalized_expected = expected.replace("\\n", "\n");
            if !normalized_stdout.contains(&normalized_expected) {
                return Err(Some(format!(
                    "Stdout mismatch: expected to contain '{}', got '{}'",
                    normalized_expected, normalized_stdout
                )));
            }
        }

        if let Some(ref expected) = test.expected_stderr {
            let normalized_stderr = stderr.replace("\r\n", "\n");
            let normalized_expected = expected.replace("\\n", "\n");
            if !normalized_stderr.contains(&normalized_expected) {
                return Err(Some(format!(
                    "Stderr mismatch: expected to contain '{}', got '{}'",
                    normalized_expected, normalized_stderr
                )));
            }
        }

        Ok(())
    }

    pub fn run_all(&self, integration_dir: &Path) -> Result<TestReport, String> {
        let tests = self.discover_tests(integration_dir)?;

        if tests.is_empty() {
            println!(
                "No integration tests found in {}",
                integration_dir.display()
            );
            return Ok(TestReport::default());
        }

        println!("\n{}", "=".repeat(60));
        println!("X Language Integration Test Suite");
        println!("Found {} tests", tests.len());
        println!("{}", "=".repeat(60));

        let mut report = TestReport::default();
        let start = Instant::now();

        for test in &tests {
            if self.verbose {
                println!("\nRunning: {}::{}", test.category, test.name);
            }

            let result = self.run_test(test);
            let passed = result.passed && !result.skipped;
            let failed = !result.passed && !result.skipped;
            let skipped = result.skipped;

            report.passed += passed as usize;
            report.failed += failed as usize;
            report.skipped += skipped as usize;

            let stats = report.categories.entry(test.category.clone()).or_default();
            stats.passed += passed as usize;
            stats.failed += failed as usize;
            stats.skipped += skipped as usize;

            let status = if skipped {
                "\x1b[33mSKIPPED\x1b[0m"
            } else if passed {
                "\x1b[32mPASS\x1b[0m"
            } else {
                "\x1b[31mFAIL\x1b[0m"
            };

            println!(
                "test {}::{} ... {} ({}ms)",
                test.category, test.name, status, result.duration_ms
            );

            if failed {
                if let Some(ref msg) = result.message {
                    for line in msg.lines() {
                        println!("  {}", line);
                    }
                }
            }

            report.results.push(result);
        }

        report.total_duration_ms = start.elapsed().as_millis() as u64;
        self.print_report(&report);

        Ok(report)
    }

    fn print_report(&self, report: &TestReport) {
        println!("\n{}", "=".repeat(60));
        println!("Test Summary by Category:");
        println!("{}", "=".repeat(60));

        let mut categories: Vec<_> = report.categories.iter().collect();
        categories.sort_by_key(|(name, _)| *name);

        for (category, stats) in categories {
            let total = stats.passed + stats.failed + stats.skipped;
            println!(
                "  {:15} {} passed, {} failed, {} skipped (total: {})",
                category, stats.passed, stats.failed, stats.skipped, total
            );
        }

        println!("\n{}", "=".repeat(60));

        let failures: Vec<_> = report
            .results
            .iter()
            .filter(|r| !r.passed && !r.skipped)
            .collect();

        if !failures.is_empty() {
            println!("\x1b[31mFailures:\x1b[0m");
            for result in &failures {
                println!("\n  {}::{}", result.test.category, result.test.name);
                if let Some(ref msg) = result.message {
                    for line in msg.lines() {
                        println!("    {}", line);
                    }
                }
            }
        }

        println!("\n{}", "=".repeat(60));
        print!("test result: ");
        if report.failed > 0 {
            print!("\x1b[31mFAILED\x1b[0m");
        } else {
            print!("\x1b[32mok\x1b[0m");
        }
        println!(
            ". {} passed; {} failed; {} skipped; finished in {}ms",
            report.passed, report.failed, report.skipped, report.total_duration_ms
        );
        println!("{}", "=".repeat(60));
    }
}

fn find_x_cli() -> Result<PathBuf, String> {
    for candidate in x_cli_candidates() {
        if candidate.exists() {
            return Ok(candidate.canonicalize().unwrap_or(candidate));
        }
    }

    if let Ok(path) = which::which("x") {
        return Ok(path);
    }

    Err("Could not find x-cli. Please build it first with 'cd tools/x-cli && cargo build --release'".to_string())
}

pub fn run_integration_tests(verbose: bool) -> Result<TestReport, String> {
    let runner = IntegrationTestRunner::new()?.with_verbose(verbose);
    let integration_dir = PathBuf::from("tests/integration");
    runner.run_all(&integration_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_root_contains_tests_directory() {
        let root = repo_root();
        assert!(root.join("tests").exists(), "repo root should contain tests/: {}", root.display());
    }

    #[test]
    fn find_x_cli_resolves_existing_binary() {
        let resolved = find_x_cli().expect("should find built x-cli binary");
        assert!(resolved.exists(), "resolved path should exist: {}", resolved.display());
        assert_eq!(resolved.file_stem().and_then(|stem| stem.to_str()), Some("x"));
    }

    #[test]
    fn test_parse_test_annotations() {
        let runner = IntegrationTestRunner::new().unwrap();
        let content = "// @exit_code: 42\n// @stdout: Hello\nprintln(\"Hello\")";
        let temp_file = std::env::temp_dir().join("test_parse.x");
        fs::write(&temp_file, content).unwrap();

        let test = runner.parse_test_file(&temp_file, "test").unwrap();
        assert_eq!(test.expected_exit_code, 42);
        assert_eq!(test.expected_stdout, Some("Hello".to_string()));

        fs::remove_file(&temp_file).ok();
    }
}
