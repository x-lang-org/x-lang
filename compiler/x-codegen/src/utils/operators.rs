//! 运算符配置工具
//!
//! 提供各语言的运算符映射配置

use x_parser::ast::{BinaryOp, UnaryOp};

/// 运算符配置，用于不同语言的运算符映射
#[derive(Debug, Clone)]
pub struct OperatorConfig {
    /// 加法运算符
    pub add: &'static str,
    /// 减法运算符
    pub sub: &'static str,
    /// 乘法运算符
    pub mul: &'static str,
    /// 除法运算符
    pub div: &'static str,
    /// 取模运算符
    pub modulo: &'static str,
    /// 幂运算符（或函数调用格式）
    pub power: &'static str,
    /// 相等比较
    pub eq: &'static str,
    /// 不等比较
    pub ne: &'static str,
    /// 小于
    pub lt: &'static str,
    /// 小于等于
    pub le: &'static str,
    /// 大于
    pub gt: &'static str,
    /// 大于等于
    pub ge: &'static str,
    /// 逻辑与
    pub and: &'static str,
    /// 逻辑或
    pub or: &'static str,
    /// 位与
    pub bitand: &'static str,
    /// 位或
    pub bitor: &'static str,
    /// 位异或
    pub bitxor: &'static str,
    /// 左移
    pub shl: &'static str,
    /// 右移
    pub shr: &'static str,
    /// 范围运算符
    pub range: &'static str,
}

impl OperatorConfig {
    /// Java 风格运算符配置
    pub fn java() -> Self {
        Self {
            add: " + ",
            sub: " - ",
            mul: " * ",
            div: " / ",
            modulo: " % ",
            power: "Math.pow",
            eq: " == ",
            ne: " != ",
            lt: " < ",
            le: " <= ",
            gt: " > ",
            ge: " >= ",
            and: " && ",
            or: " || ",
            bitand: " & ",
            bitor: " | ",
            bitxor: " ^ ",
            shl: " << ",
            shr: " >> ",
            range: "..",
        }
    }

    /// C# 风格运算符配置
    pub fn csharp() -> Self {
        Self {
            add: " + ",
            sub: " - ",
            mul: " * ",
            div: " / ",
            modulo: " % ",
            power: "Math.Pow",
            eq: " == ",
            ne: " != ",
            lt: " < ",
            le: " <= ",
            gt: " > ",
            ge: " >= ",
            and: " && ",
            or: " || ",
            bitand: " & ",
            bitor: " | ",
            bitxor: " ^ ",
            shl: " << ",
            shr: " >> ",
            range: "..",
        }
    }

    /// Swift 风格运算符配置
    pub fn swift() -> Self {
        Self {
            add: " + ",
            sub: " - ",
            mul: " * ",
            div: " / ",
            modulo: " % ",
            power: "pow",
            eq: " == ",
            ne: " != ",
            lt: " < ",
            le: " <= ",
            gt: " > ",
            ge: " >= ",
            and: " && ",
            or: " || ",
            bitand: " & ",
            bitor: " | ",
            bitxor: " ^ ",
            shl: " << ",
            shr: " >> ",
            range: "...",
        }
    }

    /// Python 风格运算符配置
    pub fn python() -> Self {
        Self {
            add: " + ",
            sub: " - ",
            mul: " * ",
            div: " / ",
            modulo: " % ",
            power: " ** ",
            eq: " == ",
            ne: " != ",
            lt: " < ",
            le: " <= ",
            gt: " > ",
            ge: " >= ",
            and: " and ",
            or: " or ",
            bitand: " & ",
            bitor: " | ",
            bitxor: " ^ ",
            shl: " << ",
            shr: " >> ",
            range: "..",
        }
    }

    /// Rust 风格运算符配置
    pub fn rust() -> Self {
        Self {
            add: " + ",
            sub: " - ",
            mul: " * ",
            div: " / ",
            modulo: " % ",
            power: "pow",
            eq: " == ",
            ne: " != ",
            lt: " < ",
            le: " <= ",
            gt: " > ",
            ge: " >= ",
            and: " && ",
            or: " || ",
            bitand: " & ",
            bitor: " | ",
            bitxor: " ^ ",
            shl: " << ",
            shr: " >> ",
            range: "..",
        }
    }

    /// 获取二元运算符的字符串表示
    pub fn get_binary(&self, op: &BinaryOp) -> Option<&'static str> {
        match op {
            BinaryOp::Add => Some(self.add),
            BinaryOp::Sub => Some(self.sub),
            BinaryOp::Mul => Some(self.mul),
            BinaryOp::Div => Some(self.div),
            BinaryOp::Mod => Some(self.modulo),
            BinaryOp::Pow => Some(self.power),
            BinaryOp::Equal => Some(self.eq),
            BinaryOp::NotEqual => Some(self.ne),
            BinaryOp::Less => Some(self.lt),
            BinaryOp::LessEqual => Some(self.le),
            BinaryOp::Greater => Some(self.gt),
            BinaryOp::GreaterEqual => Some(self.ge),
            BinaryOp::And => Some(self.and),
            BinaryOp::Or => Some(self.or),
            BinaryOp::BitAnd => Some(self.bitand),
            BinaryOp::BitOr => Some(self.bitor),
            BinaryOp::BitXor => Some(self.bitxor),
            BinaryOp::LeftShift => Some(self.shl),
            BinaryOp::RightShift => Some(self.shr),
            _ => None,
        }
    }
}

/// 一元运算符配置
pub fn get_unary_op(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
        UnaryOp::Wait => "await ",
        UnaryOp::Reference => "&",
        UnaryOp::MutableReference => "&mut ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_config() {
        let config = OperatorConfig::java();
        assert_eq!(config.add, " + ");
        assert_eq!(config.eq, " == ");
        assert_eq!(config.and, " && ");
    }

    #[test]
    fn test_python_power() {
        let config = OperatorConfig::python();
        assert_eq!(config.power, " ** ");
    }

    #[test]
    fn test_swift_range() {
        let config = OperatorConfig::swift();
        assert_eq!(config.range, "...");
    }

    #[test]
    fn test_get_binary() {
        let config = OperatorConfig::java();
        assert_eq!(config.get_binary(&BinaryOp::Add), Some(" + "));
        assert_eq!(config.get_binary(&BinaryOp::And), Some(" && "));
    }
}
