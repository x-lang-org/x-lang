// Prelude - 自动导入的核心类型和函数
//
// 这个模块包含X语言中最常用的函数，无需显式导入即可使用。

use super::{StdValue, StdError};
use std::io::Write;

/// 打印值到标准输出（不换行）
pub fn print(args: &[StdValue]) -> Result<StdValue, StdError> {
    for arg in args {
        print!("{}", arg.to_string_repr());
    }
    std::io::stdout().flush().map_err(|e| StdError::IoError(e.to_string()))?;
    Ok(StdValue::Unit)
}

/// 打印值到标准输出（带换行）
pub fn println(args: &[StdValue]) -> Result<StdValue, StdError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg.to_string_repr());
    }
    println!();
    Ok(StdValue::Unit)
}

/// 从标准输入读取一行
pub fn input(args: &[StdValue]) -> Result<StdValue, StdError> {
    if !args.is_empty() {
        print!("{}", args[0].to_string_repr());
        std::io::stdout().flush().map_err(|e| StdError::IoError(e.to_string()))?;
    }
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).map_err(|e| StdError::IoError(e.to_string()))?;
    Ok(StdValue::String(line.trim().to_string()))
}

/// 格式化字符串（简单版本）
pub fn format(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Ok(StdValue::String(String::new()));
    }

    let template = args[0].as_string()?;
    let mut result = String::new();
    let mut arg_idx = 1;
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'}') {
            chars.next();
            if arg_idx < args.len() {
                result.push_str(&args[arg_idx].to_string_repr());
                arg_idx += 1;
            } else {
                result.push_str("{}");
            }
        } else {
            result.push(c);
        }
    }

    Ok(StdValue::String(result))
}

/// 断言函数
pub fn assert(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("assert需要至少一个参数".to_string()));
    }

    let condition = args[0].as_boolean()?;
    if !condition {
        let message = if args.len() > 1 {
            args[1].to_string_repr()
        } else {
            "断言失败".to_string()
        };
        return Err(StdError::RuntimeError(message));
    }

    Ok(StdValue::Unit)
}

/// 相等性断言
pub fn assert_eq(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("assert_eq需要两个参数".to_string()));
    }

    if args[0] != args[1] {
        let message = if args.len() > 2 {
            format!("{}: 期望 {:?}，得到 {:?}", args[2].to_string_repr(), args[1], args[0])
        } else {
            format!("断言失败: 期望 {:?}，得到 {:?}", args[1], args[0])
        };
        return Err(StdError::RuntimeError(message));
    }

    Ok(StdValue::Unit)
}

/// 恐慌函数
pub fn panic(args: &[StdValue]) -> Result<StdValue, StdError> {
    let message = if args.is_empty() {
        "panic".to_string()
    } else {
        args[0].to_string_repr()
    };
    Err(StdError::RuntimeError(message))
}

/// 转换为字符串
pub fn to_string(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("to_string需要一个参数".to_string()));
    }
    Ok(StdValue::String(args[0].to_string_repr()))
}

/// 类型名称
pub fn type_of(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("type_of需要一个参数".to_string()));
    }
    Ok(StdValue::String(args[0].type_name().to_string()))
}
