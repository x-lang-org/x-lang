//! 类型错误格式化模块
//!
//! 提供将类型错误格式化为用户友好消息的功能，包括文件名、行号、列号、源码片段、
//! 错误代码、严重程度和修复建议。

use crate::errors::{Severity, TypeError};

#[cfg(test)]
use x_lexer::span::Span;

/// 格式化类型错误为用户友好的消息
///
/// # 参数
/// - `file`: 文件名
/// - `source`: 源代码内容
/// - `error`: 类型错误
///
/// # 返回
/// 格式化的错误消息，格式为：`file:line:col: [E0001] error message\n  line | source snippet`
pub fn format_type_error(file: &str, source: &str, error: &TypeError) -> String {
    let span = error.span();
    let (line, col) = span.line_col(source);
    let snippet = span.snippet(source);
    let severity = match error.severity() {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    };
    let code = error.format_error_code();

    format!(
        "{}:{}:{}: {}[{}]: {}\n  {} | {}",
        file,
        line,
        col,
        severity,
        code,
        error,
        line,
        snippet.trim_end()
    )
}

/// 格式化类型错误（包含修复建议）
///
/// # 参数
/// - `file`: 文件名
/// - `source`: 源代码内容
/// - `error`: 类型错误
///
/// # 返回
/// 格式化的错误消息，包含源码片段和修复建议
pub fn format_type_error_with_suggestions(file: &str, source: &str, error: &TypeError) -> String {
    let mut result = format_type_error(file, source, error);

    let suggestions = error.fix_suggestions();
    if !suggestions.is_empty() {
        result.push_str("\n\n  提示:");
        for suggestion in suggestions {
            result.push_str(&format!("\n    • {}", suggestion.message));
        }
    }

    result
}

/// 格式化多个类型错误
///
/// # 参数
/// - `file`: 文件名
/// - `source`: 源代码内容
/// - `errors`: 类型错误列表
///
/// # 返回
/// 格式化的错误消息，多个错误用空行分隔
pub fn format_type_errors(file: &str, source: &str, errors: &[TypeError]) -> String {
    errors
        .iter()
        .map(|e| format_type_error_with_suggestions(file, source, e))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ErrorCode;
    use x_lexer::span::Span;

    #[test]
    fn format_undefined_variable_error() {
        let source = "let y = x;";
        let error = TypeError::UndefinedVariable {
            name: "x".to_string(),
            span: Span::new(8, 9),
        };
        let msg = format_type_error("test.x", source, &error);
        assert!(msg.contains("test.x:"), "should contain file name");
        assert!(msg.contains("未定义的变量: x"), "should contain error message");
        assert!(msg.contains("[E0001]"), "should contain error code");
    }

    #[test]
    fn format_type_mismatch_error() {
        let source = "let x: Int = \"hello\";";
        let error = TypeError::TypeMismatch {
            expected: "Int".to_string(),
            actual: "String".to_string(),
            span: Span::new(12, 19),
        };
        let msg = format_type_error("test.x", source, &error);
        assert!(msg.contains("类型不匹配"), "should contain type mismatch message");
        assert!(msg.contains("期望 Int"), "should contain expected type");
        assert!(msg.contains("实际 String"), "should contain actual type");
        assert!(msg.contains("[E0003]"), "should contain error code");
    }

    #[test]
    fn format_error_with_suggestions() {
        let source = "let y = x;";
        let error = TypeError::UndefinedVariable {
            name: "x".to_string(),
            span: Span::new(8, 9),
        };
        let msg = format_type_error_with_suggestions("test.x", source, &error);
        assert!(msg.contains("提示:"), "should contain suggestions header");
        assert!(msg.contains("检查变量"), "should contain suggestion");
    }

    #[test]
    fn error_code_matches_error_type() {
        let error = TypeError::UndefinedVariable {
            name: "x".to_string(),
            span: Span::new(0, 1),
        };
        assert_eq!(error.error_code(), ErrorCode::E0001);

        let error = TypeError::DuplicateDeclaration {
            name: "foo".to_string(),
            span: Span::new(0, 1),
        };
        assert_eq!(error.error_code(), ErrorCode::E0006);
    }
}
