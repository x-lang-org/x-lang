// 迭代器模块
//
// 提供迭代器 trait 和常用迭代器适配器。

use super::{StdValue, StdError};

/// 创建范围迭代器
pub fn iter_range(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("iter_range需要两个参数: start, end".to_string()));
    }
    let start = args[0].as_integer()?;
    let end = args[1].as_integer()?;

    let mut result = Vec::new();
    if start <= end {
        for i in start..end {
            result.push(StdValue::Integer(i));
        }
    } else {
        for i in (end+1..=start).rev() {
            result.push(StdValue::Integer(i));
        }
    }
    Ok(StdValue::List(result))
}

/// 映射迭代器
pub fn iter_map(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("iter_map尚未实现".to_string()))
}

/// 过滤迭代器
pub fn iter_filter(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("iter_filter尚未实现".to_string()))
}

/// 收集迭代器
pub fn iter_collect(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Err(StdError::RuntimeError("iter_collect尚未实现".to_string()))
}
