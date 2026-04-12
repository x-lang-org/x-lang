use thiserror::Error;
use x_lexer::span::Span;

/// 错误严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// 错误：必须修复，否则无法继续编译
    Error,
    /// 警告：建议修复，但不阻止编译
    Warning,
    /// 信息：仅供参考
    Info,
}

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// 名称解析错误（未定义变量、未定义类型等）
    NameResolution,
    /// 类型错误（类型不匹配、类型推断失败等）
    TypeMismatch,
    /// 声明错误（重复声明、无效声明等）
    Declaration,
    /// 参数错误（参数数量/类型不匹配）
    Parameter,
    /// 约束错误（类型约束不满足）
    Constraint,
    /// 内部错误
    Internal,
}

/// 错误代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    E0001, // 未定义变量
    E0002, // 未定义类型
    E0003, // 类型不匹配
    E0004, // 参数数量不匹配
    E0005, // 返回类型不匹配
    E0006, // 重复声明
    E0007, // 无效的类型注解
    E0008, // 类型参数不匹配
    E0009, // 类型约束不满足
    E0010, // 递归类型定义
    E0011, // 无法推断类型
    E0012, // 未实现的功能
    E0013, // 内部错误
    E0014, // 参数类型不匹配
    E0015, // 类型不兼容
    E0016, // 无效的 wait 表达式
    E0017, // 异步类型不匹配
}

/// 修复建议
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    /// 建议描述
    pub message: String,
    /// 可选的替换文本
    pub replacement: Option<String>,
}

/// 类型检查错误（携带源码位置，用于 file:line:col 报告）
#[derive(Error, Debug)]
pub enum TypeError {
    #[error("未定义的变量: {name}")]
    UndefinedVariable { name: String, span: Span },

    #[error("未定义的类型: {name}")]
    UndefinedType { name: String, span: Span },

    #[error("类型不匹配: 期望 {expected}, 实际 {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
        span: Span,
    },

    #[error("函数调用参数数量不匹配: 期望 {expected}, 实际 {actual}")]
    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
        span: Span,
    },

    #[error("函数返回类型不匹配: 期望 {expected}, 实际 {actual}")]
    ReturnTypeMismatch {
        expected: String,
        actual: String,
        span: Span,
    },

    #[error("重复声明: {name}")]
    DuplicateDeclaration { name: String, span: Span },

    #[error("无效的类型注解")]
    InvalidTypeAnnotation { span: Span },

    #[error("类型参数不匹配")]
    TypeParameterMismatch { span: Span },

    #[error("类型约束不满足")]
    TypeConstraintViolation { span: Span },

    #[error("递归类型定义")]
    RecursiveType { span: Span },

    #[error("无法推断类型")]
    CannotInferType { span: Span },

    #[error("未实现的功能: {feature}")]
    NotImplemented { feature: String, span: Span },

    #[error("内部错误: {message}")]
    InternalError { message: String, span: Span },

    #[error("函数参数数量不匹配: 期望 {expected}, 实际 {actual}")]
    ParameterCountMismatch {
        expected: usize,
        actual: usize,
        span: Span,
    },

    #[error("函数调用参数类型不匹配")]
    ParameterTypeMismatch { span: Span },

    #[error("类型不兼容")]
    TypeIncompatible { span: Span },

    #[error("缺少 trait 方法实现: {trait_name}::{method_name}")]
    MissingTraitMethod {
        trait_name: String,
        method_name: String,
        span: Span,
    },

    #[error("未定义的成员: {name}")]
    UndefinedMember { name: String, span: Span },

    #[error("无效的成员访问: {message}")]
    InvalidMemberAccess { message: String, span: Span },

    #[error("无效的方法重写: {message}")]
    InvalidOverride { message: String, span: Span },

    #[error("无效的 wait 表达式: 期望 Async<T> 类型, 实际为 {actual_type}")]
    InvalidAwait { actual_type: String, span: Span },

    #[error("异步类型不匹配: 期望 {expected}, 实际 {actual}")]
    AsyncTypeMismatch {
        expected: String,
        actual: String,
        span: Span,
    },

    #[error("继承循环: 类 '{class_name}' 的继承链中存在循环")]
    InheritanceCycle { class_name: String, span: Span },

    #[error("无法继承 final 类: '{class_name}'")]
    CannotExtendFinalClass { class_name: String, span: Span },

    #[error("无法重写非虚方法: '{method_name}'")]
    CannotOverrideNonVirtual { method_name: String, span: Span },

    #[error("未实现的抽象方法: '{method_name}'")]
    UnimplementedAbstractMethod { method_name: String, span: Span },

    #[error("方法重写签名不匹配: '{method_name}' - {message}")]
    OverrideSignatureMismatch {
        method_name: String,
        message: String,
        span: Span,
    },

    // 效果系统错误
    #[error("未声明的效果: '{effect}'")]
    UndeclaredEffect { effect: String, span: Span },

    #[error("效果不匹配: 声明 '{declared}', 实际 '{actual}'")]
    EffectMismatch {
        declared: String,
        actual: String,
        span: Span,
    },

    #[error("缺少效果声明: 需要 '{required}'")]
    MissingEffectDeclaration { required: String, span: Span },

    #[error("无效的 needs 语法: '{syntax}', 期望格式 'Effect.operation'")]
    InvalidNeedsSyntax { syntax: String, span: Span },

    #[error("效果 '{effect}' 中未定义的操作: '{operation}'")]
    UndefinedEffectOperation {
        effect: String,
        operation: String,
        span: Span,
    },

    // 可见性和访问控制错误
    #[error("字段 '{field}' 在类 '{class}' 中不可见")]
    FieldNotVisible {
        class: String,
        field: String,
        span: Span,
    },

    #[error("方法 '{method}' 在类 '{class}' 中不可见")]
    MethodNotVisible {
        class: String,
        method: String,
        span: Span,
    },

    #[error("方法 '{method}' 需要 override 关键字")]
    MissingOverrideKeyword { method: String, span: Span },

    #[error("构造函数必须首先调用 super()")]
    MissingSuperCall { span: Span },

    #[error("super() 调用参数数量不匹配: 期望 {expected}, 实际 {actual}")]
    SuperCallArgumentMismatch {
        expected: usize,
        actual: usize,
        span: Span,
    },

    #[error("类型 '{sub}' 不是类型 '{sup}' 的子类型")]
    NotSubtype {
        sub: String,
        sup: String,
        span: Span,
    },

    #[error("方法 '{method}' 重写时变元不正确: {message}")]
    VarianceError {
        method: String,
        message: String,
        span: Span,
    },

    #[error("未定义的字段: {name}")]
    UndefinedField { name: String, span: Span },

    #[error("未定义的枚举变体: {enum_name}::{variant_name}")]
    UndefinedVariant {
        enum_name: String,
        variant_name: String,
        span: Span,
    },
}

impl TypeError {
    /// 获取错误的源码位置
    pub fn span(&self) -> Span {
        match self {
            TypeError::UndefinedVariable { span, .. } => *span,
            TypeError::UndefinedType { span, .. } => *span,
            TypeError::TypeMismatch { span, .. } => *span,
            TypeError::ArgumentCountMismatch { span, .. } => *span,
            TypeError::ReturnTypeMismatch { span, .. } => *span,
            TypeError::DuplicateDeclaration { span, .. } => *span,
            TypeError::InvalidTypeAnnotation { span } => *span,
            TypeError::TypeParameterMismatch { span } => *span,
            TypeError::TypeConstraintViolation { span } => *span,
            TypeError::RecursiveType { span } => *span,
            TypeError::CannotInferType { span } => *span,
            TypeError::NotImplemented { span, .. } => *span,
            TypeError::InternalError { span, .. } => *span,
            TypeError::ParameterCountMismatch { span, .. } => *span,
            TypeError::ParameterTypeMismatch { span } => *span,
            TypeError::TypeIncompatible { span } => *span,
            TypeError::MissingTraitMethod { span, .. } => *span,
            TypeError::UndefinedMember { span, .. } => *span,
            TypeError::InvalidMemberAccess { span, .. } => *span,
            TypeError::InvalidOverride { span, .. } => *span,
            TypeError::InvalidAwait { span, .. } => *span,
            TypeError::AsyncTypeMismatch { span, .. } => *span,
            TypeError::InheritanceCycle { span, .. } => *span,
            TypeError::CannotExtendFinalClass { span, .. } => *span,
            TypeError::CannotOverrideNonVirtual { span, .. } => *span,
            TypeError::UnimplementedAbstractMethod { span, .. } => *span,
            TypeError::OverrideSignatureMismatch { span, .. } => *span,
            TypeError::UndeclaredEffect { span, .. } => *span,
            TypeError::EffectMismatch { span, .. } => *span,
            TypeError::MissingEffectDeclaration { span, .. } => *span,
            TypeError::InvalidNeedsSyntax { span, .. } => *span,
            TypeError::UndefinedEffectOperation { span, .. } => *span,
            TypeError::FieldNotVisible { span, .. } => *span,
            TypeError::MethodNotVisible { span, .. } => *span,
            TypeError::MissingOverrideKeyword { span, .. } => *span,
            TypeError::MissingSuperCall { span, .. } => *span,
            TypeError::SuperCallArgumentMismatch { span, .. } => *span,
            TypeError::NotSubtype { span, .. } => *span,
            TypeError::VarianceError { span, .. } => *span,
            TypeError::UndefinedField { span, .. } => *span,
            TypeError::UndefinedVariant { span, .. } => *span,
        }
    }

    /// 获取错误严重程度
    pub fn severity(&self) -> Severity {
        match self {
            TypeError::NotImplemented { .. } => Severity::Warning,
            _ => Severity::Error,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            TypeError::UndefinedVariable { .. } | TypeError::UndefinedType { .. } => {
                ErrorCategory::NameResolution
            }
            TypeError::TypeMismatch { .. }
            | TypeError::ReturnTypeMismatch { .. }
            | TypeError::CannotInferType { .. }
            | TypeError::TypeIncompatible { .. }
            | TypeError::InvalidMemberAccess { .. }
            | TypeError::InvalidOverride { .. }
            | TypeError::InvalidAwait { .. }
            | TypeError::AsyncTypeMismatch { .. }
            | TypeError::OverrideSignatureMismatch { .. } => ErrorCategory::TypeMismatch,
            TypeError::DuplicateDeclaration { .. } | TypeError::InvalidTypeAnnotation { .. } => {
                ErrorCategory::Declaration
            }
            TypeError::ArgumentCountMismatch { .. }
            | TypeError::ParameterCountMismatch { .. }
            | TypeError::ParameterTypeMismatch { .. } => ErrorCategory::Parameter,
            TypeError::TypeParameterMismatch { .. }
            | TypeError::TypeConstraintViolation { .. }
            | TypeError::RecursiveType { .. }
            | TypeError::MissingTraitMethod { .. }
            | TypeError::InheritanceCycle { .. }
            | TypeError::CannotExtendFinalClass { .. }
            | TypeError::CannotOverrideNonVirtual { .. }
            | TypeError::UnimplementedAbstractMethod { .. }
            | TypeError::UndeclaredEffect { .. }
            | TypeError::EffectMismatch { .. }
            | TypeError::MissingEffectDeclaration { .. }
            | TypeError::InvalidNeedsSyntax { .. }
            | TypeError::UndefinedEffectOperation { .. }
            | TypeError::FieldNotVisible { .. }
            | TypeError::MethodNotVisible { .. }
            | TypeError::MissingOverrideKeyword { .. }
            | TypeError::MissingSuperCall { .. }
            | TypeError::SuperCallArgumentMismatch { .. }
            | TypeError::NotSubtype { .. }
            | TypeError::VarianceError { .. } => ErrorCategory::Constraint,
            TypeError::UndefinedMember { .. }
            | TypeError::UndefinedField { .. }
            | TypeError::UndefinedVariant { .. } => ErrorCategory::NameResolution,
            TypeError::NotImplemented { .. } | TypeError::InternalError { .. } => {
                ErrorCategory::Internal
            }
        }
    }

    /// 获取错误代码
    pub fn error_code(&self) -> ErrorCode {
        match self {
            TypeError::UndefinedVariable { .. } => ErrorCode::E0001,
            TypeError::UndefinedType { .. } => ErrorCode::E0002,
            TypeError::TypeMismatch { .. } => ErrorCode::E0003,
            TypeError::ArgumentCountMismatch { .. } => ErrorCode::E0004,
            TypeError::ReturnTypeMismatch { .. } => ErrorCode::E0005,
            TypeError::DuplicateDeclaration { .. } => ErrorCode::E0006,
            TypeError::InvalidTypeAnnotation { .. } => ErrorCode::E0007,
            TypeError::TypeParameterMismatch { .. } => ErrorCode::E0008,
            TypeError::TypeConstraintViolation { .. } => ErrorCode::E0009,
            TypeError::RecursiveType { .. } => ErrorCode::E0010,
            TypeError::CannotInferType { .. } => ErrorCode::E0011,
            TypeError::NotImplemented { .. } => ErrorCode::E0012,
            TypeError::InternalError { .. } => ErrorCode::E0013,
            TypeError::ParameterTypeMismatch { .. } => ErrorCode::E0014,
            TypeError::TypeIncompatible { .. } => ErrorCode::E0015,
            TypeError::ParameterCountMismatch { .. } => ErrorCode::E0004,
            TypeError::MissingTraitMethod { .. } => ErrorCode::E0009,
            TypeError::UndefinedMember { .. } => ErrorCode::E0001,
            TypeError::UndefinedField { .. } => ErrorCode::E0001,
            TypeError::UndefinedVariant { .. } => ErrorCode::E0002,
            TypeError::InvalidMemberAccess { .. } => ErrorCode::E0003,
            TypeError::InvalidOverride { .. } => ErrorCode::E0009,
            TypeError::InvalidAwait { .. } => ErrorCode::E0016,
            TypeError::AsyncTypeMismatch { .. } => ErrorCode::E0017,
            TypeError::InheritanceCycle { .. } => ErrorCode::E0009,
            TypeError::CannotExtendFinalClass { .. } => ErrorCode::E0009,
            TypeError::CannotOverrideNonVirtual { .. } => ErrorCode::E0009,
            TypeError::UnimplementedAbstractMethod { .. } => ErrorCode::E0009,
            TypeError::OverrideSignatureMismatch { .. } => ErrorCode::E0009,
            TypeError::UndeclaredEffect { .. } => ErrorCode::E0009,
            TypeError::EffectMismatch { .. } => ErrorCode::E0009,
            TypeError::MissingEffectDeclaration { .. } => ErrorCode::E0009,
            TypeError::InvalidNeedsSyntax { .. } => ErrorCode::E0002,
            TypeError::UndefinedEffectOperation { .. } => ErrorCode::E0002,
            TypeError::FieldNotVisible { .. } => ErrorCode::E0003,
            TypeError::MethodNotVisible { .. } => ErrorCode::E0003,
            TypeError::MissingOverrideKeyword { .. } => ErrorCode::E0009,
            TypeError::MissingSuperCall { .. } => ErrorCode::E0009,
            TypeError::SuperCallArgumentMismatch { .. } => ErrorCode::E0004,
            TypeError::NotSubtype { .. } => ErrorCode::E0003,
            TypeError::VarianceError { .. } => ErrorCode::E0009,
        }
    }

    /// 获取修复建议
    pub fn fix_suggestions(&self) -> Vec<FixSuggestion> {
        match self {
            TypeError::UndefinedVariable { name, .. } => vec![
                FixSuggestion {
                    message: format!("检查变量 '{}' 是否已声明", name),
                    replacement: None,
                },
                FixSuggestion {
                    message: format!("如果这是新变量，请添加声明：let {} = ...", name),
                    replacement: None,
                },
            ],
            TypeError::UndefinedType { name, .. } => vec![
                FixSuggestion {
                    message: format!("检查类型名 '{}' 是否正确", name),
                    replacement: None,
                },
                FixSuggestion {
                    message: "确保已导入定义该类型的模块".to_string(),
                    replacement: None,
                },
            ],
            TypeError::TypeMismatch {
                expected, actual, ..
            } => vec![FixSuggestion {
                message: format!("尝试将 {} 转换为 {}", actual, expected),
                replacement: None,
            }],
            TypeError::DuplicateDeclaration { name, .. } => vec![FixSuggestion {
                message: format!("重命名第二个 '{}' 或移除重复声明", name),
                replacement: None,
            }],
            TypeError::ArgumentCountMismatch {
                expected, actual, ..
            } => vec![FixSuggestion {
                message: format!("函数期望 {} 个参数，但提供了 {} 个", expected, actual),
                replacement: None,
            }],
            _ => vec![],
        }
    }

    /// 格式化错误代码
    pub fn format_error_code(&self) -> String {
        format!("{:?}", self.error_code())
    }
}
