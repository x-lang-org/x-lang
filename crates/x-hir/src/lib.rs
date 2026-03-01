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