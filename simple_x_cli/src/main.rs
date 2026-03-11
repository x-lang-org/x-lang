use x_interpreter::Interpreter;
use x_parser::parser::XParser;

pub fn run_file(file_path: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(file_path).map_err(|e| format!("无法读取文件 {}: {}", file_path, e))?;

    let parser = XParser::new();
    let program = parser.parse(&content).map_err(|e| format!("解析错误: {}", e))?;

    let mut interpreter = Interpreter::new();
    interpreter.run(&program).map_err(|e| format!("运行失败: {}", e))?;

    Ok(())
}

pub fn print_usage() {
    println!("Usage: simple_x_cli run <file.x>");
}

pub fn parse_args(args: Vec<String>) -> Result<String, ()> {
    if args.len() != 3 || args[1] != "run" {
        Err(())
    } else {
        Ok(args[2].clone())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match parse_args(args) {
        Ok(file_path) => {
            match run_file(&file_path) {
                Ok(_) => println!("运行成功"),
                Err(e) => println!("{}", e),
            }
        }
        Err(_) => print_usage(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args_invalid_length() {
        let args = vec!["simple_x_cli".to_string()];
        assert!(parse_args(args).is_err());
    }

    #[test]
    fn test_parse_args_invalid_command() {
        let args = vec!["simple_x_cli".to_string(), "invalid".to_string(), "test.x".to_string()];
        assert!(parse_args(args).is_err());
    }

    #[test]
    fn test_parse_args_valid() {
        let args = vec!["simple_x_cli".to_string(), "run".to_string(), "test.x".to_string()];
        assert_eq!(parse_args(args).unwrap(), "test.x");
    }

    #[test]
    #[ignore = "需要实际测试文件"]
    fn test_run_valid_file() {
        let result = run_file("../test_simple.x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_nonexistent_file() {
        let result = run_file("nonexistent_file.x");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("无法读取文件"));
    }

    #[test]
    fn test_run_invalid_syntax() {
        use std::fs;
        use std::path::Path;

        let temp_file = "temp_invalid_test.x";
        fs::write(temp_file, "invalid x syntax").expect("无法创建临时文件");

        let result = run_file(temp_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("解析错误"));

        fs::remove_file(temp_file).expect("无法删除临时文件");
    }
}
