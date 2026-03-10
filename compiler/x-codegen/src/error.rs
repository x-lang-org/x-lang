// 代码生成错误类型

use thiserror::Error;

/// 代码生成结果类型
pub type CodeGenResult<T> = Result<T, CodeGenError>;

/// 代码生成错误
#[derive(Error, Debug)]
pub enum CodeGenError {
    /// 通用的代码生成错误
    #[error("代码生成错误: {0}")]
    GenerationError(String),

    /// 不支持的特性
    #[error("不支持的特性: {0}")]
    UnsupportedFeature(String),

    /// 无效的配置
    #[error("无效的配置: {0}")]
    InvalidConfig(String),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 来自x-parser的错误（包装）
    #[error("解析错误: {0}")]
    ParseError(String),

    /// 来自x-typechecker的错误（包装）
    #[error("类型检查错误: {0}")]
    TypeCheckError(String),

    /// 来自x-hir的错误（包装）
    #[error("HIR转换错误: {0}")]
    HirError(String),

    /// 来自x-perceus的错误（包装）
    #[error("Perceus分析错误: {0}")]
    PerceusError(String),


    /// JVM特定错误
    #[cfg(feature = "jvm")]
    #[error("JVM错误: {0}")]
    JvmError(String),

    /// .NET特定错误
    #[cfg(feature = "dotnet")]
    #[error(".NET错误: {0}")]
    DotNetError(String),

    /// JavaScript特定错误
    #[cfg(feature = "js")]
    #[error("JavaScript错误: {0}")]
    JavaScriptError(String),
}
