//! 运算符配置工具
//!
//! 提供各语言的运算符映射配置（基于 LIR 运算符类型）

use x_lir::{BinaryOp, UnaryOp};

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
        }
    }

    /// 获取二元运算符的字符串表示
    pub fn get_binary(&self, op: &BinaryOp) -> Option<&'static str> {
        match op {
            BinaryOp::Add => Some(self.add),
            BinaryOp::Subtract => Some(self.sub),
            BinaryOp::Multiply => Some(self.mul),
            BinaryOp::Divide => Some(self.div),
            BinaryOp::Modulo => Some(self.modulo),
            BinaryOp::Equal => Some(self.eq),
            BinaryOp::NotEqual => Some(self.ne),
            BinaryOp::LessThan => Some(self.lt),
            BinaryOp::LessThanEqual => Some(self.le),
            BinaryOp::GreaterThan => Some(self.gt),
            BinaryOp::GreaterThanEqual => Some(self.ge),
            BinaryOp::LogicalAnd => Some(self.and),
            BinaryOp::LogicalOr => Some(self.or),
            BinaryOp::BitAnd => Some(self.bitand),
            BinaryOp::BitOr => Some(self.bitor),
            BinaryOp::BitXor => Some(self.bitxor),
            BinaryOp::LeftShift => Some(self.shl),
            BinaryOp::RightShift => Some(self.shr),
            _ => None,
        }
    }
}

/// 获取一元运算符的字符串表示
pub fn get_unary_op(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Minus => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
        UnaryOp::Plus => "+",
        UnaryOp::Reference => "&",
        UnaryOp::MutableReference => "&mut ",
        UnaryOp::PreIncrement => "++",
        UnaryOp::PreDecrement => "--",
        UnaryOp::PostIncrement => "++",
        UnaryOp::PostDecrement => "--",
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
    fn test_python_config() {
        let config = OperatorConfig::python();
        assert_eq!(config.and, " and ");
    }

    #[test]
    fn test_swift_config() {
        let config = OperatorConfig::swift();
        assert_eq!(config.add, " + ");
    }

    #[test]
    fn test_get_binary() {
        let config = OperatorConfig::java();
        assert_eq!(config.get_binary(&BinaryOp::Add), Some(" + "));
        assert_eq!(config.get_binary(&BinaryOp::LogicalAnd), Some(" && "));
    }
}
