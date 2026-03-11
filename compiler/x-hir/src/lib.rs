// 高级中间表示库

#[derive(Debug, PartialEq, Clone)]
pub struct Hir {
    // 高级中间表示的根结构
}

/// 将抽象语法树转换为高级中间表示
pub fn ast_to_hir(_program: &x_parser::ast::Program) -> Result<Hir, HirError> {
    Ok(Hir {})
}

/// 高级中间表示错误
#[derive(thiserror::Error, Debug)]
pub enum HirError {
    #[error("转换错误: {0}")]
    ConversionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_to_hir_returns_ok_for_minimal_program() {
        let source = "let x = 1;";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");
        assert_eq!(hir, Hir {});
    }

    #[test]
    fn ast_to_hir_returns_ok_for_program_with_function() {
        let source = "function main() { println(\"hi\") }";
        let parser = x_parser::parser::XParser::new();
        let program = parser.parse(source).expect("parse");
        let hir = ast_to_hir(&program).expect("ast_to_hir");
        assert_eq!(hir, Hir {});
    }

    #[test]
    fn hir_error_displays_message() {
        let e = HirError::ConversionError("test message".to_string());
        assert!(e.to_string().contains("转换错误"));
        assert!(e.to_string().contains("test message"));
    }
}
