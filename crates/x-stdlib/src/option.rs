// Option 类型 - 替代 null
//
// Option<T> 表示可能存在或不存在的值。
// Some(value) - 存在值
// None - 不存在值

use super::{StdValue, StdError};

/// Option 枚举
#[derive(Debug, PartialEq, Clone)]
pub enum Option<T> {
    Some(T),
    None,
}

/// 创建 Some 值
pub fn some(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("Some需要一个参数".to_string()));
    }
    Ok(StdValue::Option(Box::new(Option::Some(args[0].clone()))))
}

/// 创建 None 值
pub fn none(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Ok(StdValue::Option(Box::new(Option::None)))
}

/// 检查是否是 Some
pub fn is_some(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("is_some需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Option(opt) => Ok(StdValue::Boolean(matches!(opt.as_ref(), Option::Some(_)))),
        _ => Err(StdError::TypeError("is_some需要Option类型".to_string())),
    }
}

/// 检查是否是 None
pub fn is_none(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("is_none需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Option(opt) => Ok(StdValue::Boolean(matches!(opt.as_ref(), Option::None))),
        _ => Err(StdError::TypeError("is_none需要Option类型".to_string())),
    }
}

/// 解包 Option， panic 如果是 None
pub fn unwrap(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("unwrap需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Option(opt) => match opt.as_ref() {
            Option::Some(v) => Ok(v.clone()),
            Option::None => Err(StdError::RuntimeError("解包None值".to_string())),
        },
        _ => Err(StdError::TypeError("unwrap需要Option类型".to_string())),
    }
}

/// 解包 Option，返回默认值如果是 None
pub fn unwrap_or(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("unwrap_or需要两个参数".to_string()));
    }
    match &args[0] {
        StdValue::Option(opt) => match opt.as_ref() {
            Option::Some(v) => Ok(v.clone()),
            Option::None => Ok(args[1].clone()),
        },
        _ => Err(StdError::TypeError("unwrap_or需要Option类型作为第一个参数".to_string())),
    }
}

/// 解包 Option，执行闭包如果是 None
pub fn unwrap_or_else(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("unwrap_or_else尚未实现".to_string()))
}

/// 映射 Option 中的值
pub fn map(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("map尚未实现".to_string()))
}

/// 过滤 Option
pub fn filter(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("filter尚未实现".to_string()))
}

/// 如果是 Some，返回第二个 Option，否则返回 None
pub fn and_then(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("and_then尚未实现".to_string()))
}

/// 如果是 Some，返回自己，否则返回另一个 Option
pub fn or_else(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("or_else尚未实现".to_string()))
}
