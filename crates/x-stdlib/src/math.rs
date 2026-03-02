// 数学模块
//
// 提供常用的数学函数和常量。

use super::{StdValue, StdError};

/// 数学常量 π
pub const PI: f64 = std::f64::consts::PI;

/// 数学常数 e
pub const E: f64 = std::f64::consts::E;

/// 自然对数 2
pub const LN_2: f64 = std::f64::consts::LN_2;

/// 自然对数 10
pub const LN_10: f64 = std::f64::consts::LN_10;

/// log2(e)
pub const LOG2_E: f64 = std::f64::consts::LOG2_E;

/// log10(e)
pub const LOG10_E: f64 = std::f64::consts::LOG10_E;

/// 平方根 2
pub const SQRT_2: f64 = std::f64::consts::SQRT_2;

/// 1/平方根 2
pub const FRAC_1_SQRT_2: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// 绝对值
pub fn abs(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("abs需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Integer(n) => Ok(StdValue::Integer(n.abs())),
        StdValue::Float(n) => Ok(StdValue::Float(n.abs())),
        _ => Err(StdError::TypeError("abs需要数字类型".to_string())),
    }
}

/// 平方根
pub fn sqrt(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("sqrt需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.sqrt()))
}

/// 幂运算
pub fn pow(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("pow需要两个参数".to_string()));
    }
    let base = args[0].as_float()?;
    let exp = args[1].as_float()?;
    Ok(StdValue::Float(base.powf(exp)))
}

/// 正弦
pub fn sin(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("sin需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.sin()))
}

/// 余弦
pub fn cos(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("cos需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.cos()))
}

/// 正切
pub fn tan(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("tan需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.tan()))
}

/// 反正弦
pub fn asin(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("asin需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.asin()))
}

/// 反余弦
pub fn acos(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("acos需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.acos()))
}

/// 反正切
pub fn atan(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("atan需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.atan()))
}

/// 反正切2（获取点的角度）
pub fn atan2(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("atan2需要两个参数".to_string()));
    }
    let y = args[0].as_float()?;
    let x = args[1].as_float()?;
    Ok(StdValue::Float(y.atan2(x)))
}

/// 指数函数 e^x
pub fn exp(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("exp需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.exp()))
}

/// 自然对数
pub fn ln(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("ln需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.ln()))
}

/// 以2为底的对数
pub fn log2(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("log2需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.log2()))
}

/// 以10为底的对数
pub fn log10(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("log10需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.log10()))
}

/// 向下取整
pub fn floor(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("floor需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.floor()))
}

/// 向上取整
pub fn ceil(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("ceil需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.ceil()))
}

/// 四舍五入
pub fn round(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("round需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.round()))
}

/// 截断小数部分
pub fn trunc(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("trunc需要一个参数".to_string()));
    }
    let n = args[0].as_float()?;
    Ok(StdValue::Float(n.trunc()))
}

/// 最小值
pub fn min(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("min需要至少两个参数".to_string()));
    }

    let mut result = args[0].clone();
    for arg in &args[1..] {
        match (&result, arg) {
            (StdValue::Integer(a), StdValue::Integer(b)) => {
                if b < a {
                    result = StdValue::Integer(*b);
                }
            }
            (StdValue::Float(a), StdValue::Float(b)) => {
                if b < a {
                    result = StdValue::Float(*b);
                }
            }
            (StdValue::Integer(a), StdValue::Float(b)) => {
                if b < &(*a as f64) {
                    result = StdValue::Float(*b);
                }
            }
            (StdValue::Float(a), StdValue::Integer(b)) => {
                if &(*b as f64) < a {
                    result = StdValue::Integer(*b);
                }
            }
            _ => {
                return Err(StdError::TypeError("min需要可比较的数字类型".to_string()));
            }
        }
    }
    Ok(result)
}

/// 最大值
pub fn max(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 2 {
        return Err(StdError::InvalidArgument("max需要至少两个参数".to_string()));
    }

    let mut result = args[0].clone();
    for arg in &args[1..] {
        match (&result, arg) {
            (StdValue::Integer(a), StdValue::Integer(b)) => {
                if b > a {
                    result = StdValue::Integer(*b);
                }
            }
            (StdValue::Float(a), StdValue::Float(b)) => {
                if b > a {
                    result = StdValue::Float(*b);
                }
            }
            (StdValue::Integer(a), StdValue::Float(b)) => {
                if b > &(*a as f64) {
                    result = StdValue::Float(*b);
                }
            }
            (StdValue::Float(a), StdValue::Integer(b)) => {
                if &(*b as f64) > a {
                    result = StdValue::Integer(*b);
                }
            }
            _ => {
                return Err(StdError::TypeError("max需要可比较的数字类型".to_string()));
            }
        }
    }
    Ok(result)
}

/// 限制值在范围内
pub fn clamp(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.len() < 3 {
        return Err(StdError::InvalidArgument("clamp需要三个参数: value, min, max".to_string()));
    }
    let value = args[0].as_float()?;
    let min = args[1].as_float()?;
    let max = args[2].as_float()?;
    Ok(StdValue::Float(value.clamp(min, max)))
}

/// 符号函数
pub fn signum(args: &[StdValue]) -> Result<StdValue, StdError> {
    if args.is_empty() {
        return Err(StdError::InvalidArgument("signum需要一个参数".to_string()));
    }
    match &args[0] {
        StdValue::Integer(n) => Ok(StdValue::Integer(n.signum())),
        StdValue::Float(n) => Ok(StdValue::Float(n.signum())),
        _ => Err(StdError::TypeError("signum需要数字类型".to_string())),
    }
}

/// 获取 PI 常量
pub fn pi(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Ok(StdValue::Float(PI))
}

/// 获取 e 常量
pub fn e(_args: &[StdValue]) -> Result<StdValue, StdError> {
    Ok(StdValue::Float(E))
}
