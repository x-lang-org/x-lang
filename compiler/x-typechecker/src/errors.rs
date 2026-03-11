use thiserror::Error;

/// 类型检查错误
#[derive(Error, Debug)]
pub enum TypeError {
    #[error("未定义的变量: {0}")]
    UndefinedVariable(String),

    #[error("未定义的类型: {0}")]
    UndefinedType(String),

    #[error("类型不匹配: 期望 {expected}, 实际 {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("函数调用参数数量不匹配: 期望 {expected}, 实际 {actual}")]
    ArgumentCountMismatch { expected: usize, actual: usize },

    #[error("函数返回类型不匹配: 期望 {expected}, 实际 {actual}")]
    ReturnTypeMismatch { expected: String, actual: String },

    #[error("无效的类型注解")]
    InvalidTypeAnnotation,

    #[error("类型参数不匹配")]
    TypeParameterMismatch,

    #[error("类型约束不满足")]
    TypeConstraintViolation,

    #[error("递归类型定义")]
    RecursiveType,

    #[error("无法推断类型")]
    TypeInferenceFailure,

    #[error("未实现的功能: {0}")]
    NotImplemented(String),

    #[error("内部错误: {0}")]
    InternalError(String),
}
