// Result 类型 - 替代异常
//
// Result<T, E> 表示可能成功或失败的操作。
// Ok(value) - 成功
// Err(error) - 失败

use super::{StdValue, StdError};

/// Result 枚举
#[derive(Debug, PartialEq, Clone)]
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

/// 创建 Ok 值
pub fn ok(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("Ok需要一个参数".to_string()));
    }
    Ok(StdValue::Result(Box::new(Result::Ok(args[0].clone()))))
}

/// 创建 Err 值
pub fn err(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("Err需要一个参数".to_string()));
    }
    Ok(StdValue::Result(Box::new(Result::Err(args[0].clone()))))
}

/// 检查是否是 Ok
pub fn is_ok(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("is_ok需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Result(res) => Ok(StdValue::Boolean(matches!(res.as_ref(), Result::Ok(_)))),
        _ => Err(StdError::TypeError("is_ok需要Result类型".to_string())),
    }
}

/// 检查是否是 Err
pub fn is_err(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("is_err需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Result(res) => Ok(StdValue::Boolean(matches!(res.as_ref(), Result::Err(_)))),
        _ => Err(StdError::TypeError("is_err需要Result类型".to_string())),
    }
}

/// 解包 Result， panic 如果是 Err
pub fn unwrap(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("unwrap需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Result(res) => match res.as_ref() {
            Result::Ok(v) => Ok(v.clone()),
            Result::Err(e) => Err(StdError::RuntimeError(format!("解包Err值: {}", e.to_string_repr()))),
        },
        _ => Err(StdError::TypeError("unwrap需要Result类型".to_string())),
    }
}

/// 解包 Result，返回默认值如果是 Err
pub fn unwrap_or(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("unwrap_or需要两个参数".to_string()));
    }
    match &args[0] {
        StdValue::Result(res) => match res.as_ref() {
            Result::Ok(v) => Ok(v.clone()),
            Result::Err(_) => Ok(args[1].clone()),
        },
        _ => Err(StdError::TypeError("unwrap_or需要Result类型作为第一个参数".to_string())),
    }
}

/// 解包 Result 的 Err， panic 如果是 Ok
pub fn unwrap_err(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("unwrap_err需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Result(res) => match res.as_ref() {
            Result::Ok(v) => Err(StdError::RuntimeError(format!("解包Err但得到Ok: {}", v.to_string_repr()))),
            Result::Err(e) => Ok(e.clone()),
        },
        _ => Err(StdError::TypeError("unwrap_err需要Result类型".to_string())),
    }
}

/// 映射 Result 中的 Ok 值
pub fn map(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("map尚未实现".to_string()))
}

/// 映射 Result 中的 Err 值
pub fn map_err(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("map_err尚未实现".to_string()))
}

/// 如果是 Ok，继续执行下一个操作
pub fn and_then(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("and_then尚未实现".to_string()))
}

/// 如果是 Err，执行恢复操作
pub fn or_else(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("or_else尚未实现".to_string()))
}
