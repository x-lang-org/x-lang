use std::fs;
use std::path::{Path, PathBuf};
use x_interpreter::Interpreter;
use x_parser::parser::XParser;

pub fn find_test_files(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(paths) = fs::read_dir(dir) {
        for path in paths.flatten() {
            let path = path.path();
            if path.is_dir() {
                if let Some(path_str) = path.to_str() {
                    files.extend(find_test_files(path_str));
                }
            } else if path.extension().is_some_and(|ext| ext == "x") {
                files.push(path);
            }
        }
    }
    files
}

pub fn run_test(file: &Path) -> Result<(), String> {
    let content = fs::read_to_string(file).map_err(|e| format!("无法读取文件: {}", e))?;

    let parser = XParser::new();
    let program = parser.parse(&content).map_err(|e| format!("解析错误: {}", e))?;

    let mut interpreter = Interpreter::new();
    interpreter.run(&program).map_err(|e| format!("运行错误: {}", e))?;

    Ok(())
}

pub fn run_tests(test_dir: &str) -> (usize, usize) {
    let test_files = find_test_files(test_dir);

    let mut passed = 0;
    let mut failed = 0;

    for test_file in test_files {
        match run_test(&test_file) {
            Ok(_) => {
                passed += 1;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    (passed, failed)
}

fn main() {
    let test_dir = "../test";
    let test_files = find_test_files(test_dir);

    println!("找到 {} 个测试文件", test_files.len());

    let mut passed = 0;
    let mut failed = 0;

    for test_file in test_files {
        println!("运行测试: {}", test_file.display());
        match run_test(&test_file) {
            Ok(_) => {
                println!("✓ 测试通过");
                passed += 1;
            }
            Err(e) => {
                println!("✗ 测试失败: {}", e);
                failed += 1;
            }
        }
        println!();
    }

    println!("测试结果: {} 通过, {} 失败", passed, failed);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_find_test_files_nonexistent_dir() {
        let files = find_test_files("nonexistent_test_dir");
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_test_files_ignores_non_x_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(file_path).unwrap();

        let files = find_test_files(dir.path().to_str().unwrap());
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_test_files_finds_x_files() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.x");
        File::create(file_path).unwrap();

        let files = find_test_files(dir.path().to_str().unwrap());
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_run_test_nonexistent_file() {
        let result = run_test(Path::new("nonexistent.x"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无法读取文件"));
    }

    #[test]
    fn test_run_test_invalid_syntax() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid.x");
        let mut file = File::create(file_path.clone()).unwrap();
        writeln!(file, "invalid syntax").unwrap();

        let result = run_test(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("解析错误"));
    }

    #[test]
    fn test_run_tests_empty_dir() {
        let dir = tempdir().unwrap();
        let (passed, failed) = run_tests(dir.path().to_str().unwrap());
        assert_eq!(passed, 0);
        assert_eq!(failed, 0);
    }
}
