// 代码生成错误类型

use thiserror::Error;

/// 代码生成结果类型
pub type CodeGenResult<T> = Result<T, CodeGenError>;

/// 代码生成错误
///
/// 这是所有后端共享的统一错误类型，避免每个后端重复定义相同的错误枚举。
#[derive(Error, Debug)]
pub enum CodeGenError {
    /// 通用的代码生成错误
    #[error("代码生成错误: {0}")]
    GenerationError(String),

    /// 未实现的功能
    #[error("未实现: {0}")]
    Unimplemented(String),

    /// 不支持的特性
    #[error("不支持的特性: {0}")]
    UnsupportedFeature(String),

    /// 无效的配置
    #[error("无效的配置: {0}")]
    InvalidConfig(String),

    /// 编译器错误（外部编译器如 Zig、javac 等）
    #[error("编译器错误: {0}")]
    CompilerError(String),

    /// Lowering 错误（IR 转换失败）
    #[error("Lowering 错误: {0}")]
    LoweringError(String),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 格式化错误
    #[error("格式化错误: {0}")]
    FmtError(#[from] std::fmt::Error),

    /// 来自x-parser的错误（包装）
    #[error("解析错误: {0}")]
    ParseError(String),

    /// 来自x-typechecker的错误（包装）
    #[error("类型检查错误: {0}")]
    TypeCheckError(String),

    /// 来自x-hir的错误（包装）
    #[error("HIR转换错误: {0}")]
    HirError(String),

    /// 来自x-lir的错误（包装）
    #[error("LIR转换错误: {0}")]
    LirError(String),
}
