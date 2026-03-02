// 字符串模块
//
// 提供字符串操作函数。

use super::{StdValue, StdError};

/// 字符串长度
pub fn str_len(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("str_len需要一个参数".to_string()));
    }
    let s = args[0].as_string()?;
    Ok(StdValue::Integer(s.len() as i64))
}

/// 获取字符串的字符列表
pub fn str_chars(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("str_chars需要一个参数".to_string()));
    }
    let s = args[0].as_string()?;
    let chars: Vec<StdValue> = s.chars().map(StdValue::Char).collect();
    Ok(StdValue::List(chars))
}

/// 字符串连接
pub fn str_concat(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Ok(StdValue::String(String::new()));
    }
    let mut result = String::new();
    for arg in args {
        result.push_str(arg.as_string()?);
    }
    Ok(StdValue::String(result))
}

/// 检查字符串是否包含子串
pub fn str_contains(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("str_contains需要两个参数: string, substring".to_string()));
    }
    let s = args[0].as_string()?;
    let substr = args[1].as_string()?;
    Ok(StdValue::Boolean(s.contains(substr)))
}

/// 检查字符串是否以指定前缀开头
pub fn str_starts_with(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("str_starts_with需要两个参数: string, prefix".to_string()));
    }
    let s = args[0].as_string()?;
    let prefix = args[1].as_string()?;
    Ok(StdValue::Boolean(s.starts_with(prefix)))
}

/// 检查字符串是否以指定后缀结尾
pub fn str_ends_with(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("str_ends_with需要两个参数: string, suffix".to_string()));
    }
    let s = args[0].as_string()?;
    let suffix = args[1].as_string()?;
    Ok(StdValue::Boolean(s.ends_with(suffix)))
}

/// 获取子字符串
pub fn str_substring(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("str_substring需要2-3个参数: string, start [, end]".to_string()));
    }
    let s = args[0].as_string()?;
    let start = args[1].as_integer()? as usize;

    if start > s.len() {
        return Err(StdError::OutOfBounds(format!("起始位置 {} 超出字符串长度 {}", start, s.len())));
    }

    if args.len() >= 3 {
        let end = args[2].as_integer()? as usize;
        if end > s.len() {
            return Err(StdError::OutOfBounds(format!("结束位置 {} 超出字符串长度 {}", end, s.len())));
        }
        if start > end {
            return Err(StdError::OutOfBounds("起始位置不能大于结束位置".to_string()));
        }
        Ok(StdValue::String(s[start..end].to_string()))
    } else {
        Ok(StdValue::String(s[start..].to_string()))
    }
}

/// 替换子字符串
pub fn str_replace(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 3 {
        return Err(StdError::InvalidArgument("str_replace需要三个参数: string, from, to".to_string()));
    }
    let s = args[0].as_string()?;
    let from = args[1].as_string()?;
    let to = args[2].as_string()?;
    Ok(StdValue::String(s.replace(from, to)))
}

/// 转换为小写
pub fn str_to_lowercase(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("str_to_lowercase需要一个参数".to_string()));
    }
    let s = args[0].as_string()?;
    Ok(StdValue::String(s.to_lowercase()))
}

/// 转换为大写
pub fn str_to_uppercase(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("str_to_uppercase需要一个参数".to_string()));
    }
    let s = args[0].as_string()?;
    Ok(StdValue::String(s.to_uppercase()))
}

/// 去除首尾空白
pub fn str_trim(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("str_trim需要一个参数".to_string()));
    }
    let s = args[0].as_string()?;
    Ok(StdValue::String(s.trim().to_string()))
}

/// 分割字符串
pub fn str_split(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("str_split需要两个参数: string, delimiter".to_string()));
    }
    let s = args[0].as_string()?;
    let delim = args[1].as_string()?;
    let parts: Vec<StdValue> = s.split(delim).map(|part| StdValue::String(part.to_string())).collect();
    Ok(StdValue::List(parts))
}

/// 解析为整数
pub fn str_parse_int(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("str_parse_int需要一个参数".to_string()));
    }
    let s = args[0].as_string()?;
    match s.parse::<i64>() {
        Ok(n) => Ok(StdValue::Integer(n)),
        Err(_) => Err(StdError::RuntimeError(format!("无法解析为整数: {}", s))),
    }
}

/// 解析为浮点数
pub fn str_parse_float(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("str_parse_float需要一个参数".to_string()));
    }
    let s = args[0].as_string()?;
    match s.parse::<f64>() {
        Ok(n) => Ok(StdValue::Float(n)),
        Err(_) => Err(StdError::RuntimeError(format!("无法解析为浮点数: {}", s))),
    }
}

/// 重复字符串
pub fn str_repeat(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("str_repeat需要两个参数: string, count".to_string()));
    }
    let s = args[0].as_string()?;
    let count = args[1].as_integer()? as usize;
    Ok(StdValue::String(s.repeat(count)))
}

/// 填充左侧
pub fn str_pad_left(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 3 {
        return Err(StdError::InvalidArgument("str_pad_left需要三个参数: string, width, pad_char".to_string()));
    }
    let s = args[0].as_string()?;
    let width = args[1].as_integer()? as usize;
    let pad_char = match &args[2] {
        StdValue::Char(c) => *c,
        StdValue::String(s) => s.chars().next().unwrap_or(' '),
        _ => return Err(StdError::TypeError("pad_char必须是字符或字符串".to_string())),
    };

    if s.len() >= width {
        return Ok(StdValue::String(s.to_string()));
    }

    let pad_len = width - s.len();
    let mut result = String::with_capacity(width);
    for _ in 0..pad_len {
        result.push(pad_char);
    }
    result.push_str(s);
    Ok(StdValue::String(result))
}

/// 填充右侧
pub fn str_pad_right(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 3 {
        return Err(StdError::InvalidArgument("str_pad_right需要三个参数: string, width, pad_char".to_string()));
    }
    let s = args[0].as_string()?;
    let width = args[1].as_integer()? as usize;
    let pad_char = match &args[2] {
        StdValue::Char(c) => *c,
        StdValue::String(s) => s.chars().next().unwrap_or(' '),
        _ => return Err(StdError::TypeError("pad_char必须是字符或字符串".to_string())),
    };

    if s.len() >= width {
        return Ok(StdValue::String(s.to_string()));
    }

    let mut result = String::with_capacity(width);
    result.push_str(s);
    while result.len() < width {
        result.push(pad_char);
    }
    Ok(StdValue::String(result))
}
