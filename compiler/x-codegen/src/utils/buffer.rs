//! 代码缓冲区工具
//!
//! 提供带缩进的代码输出功能，所有源码生成后端共享

use std::fmt::Write;

/// 代码缓冲区，支持缩进管理
#[derive(Debug, Default)]
pub struct CodeBuffer {
    /// 输出缓冲区
    output: String,
    /// 当前缩进级别
    indent: usize,
    /// 缩进字符串（默认为两个空格）
    indent_str: &'static str,
}

impl CodeBuffer {
    /// 创建新的代码缓冲区
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            indent_str: "    ",
        }
    }

    /// 使用指定缩进字符串创建缓冲区
    pub fn with_indent_str(indent_str: &'static str) -> Self {
        Self {
            output: String::new(),
            indent: 0,
            indent_str,
        }
    }

    /// 输出一行代码（带缩进）
    pub fn line(&mut self, s: &str) -> std::fmt::Result {
        for _ in 0..self.indent {
            write!(self.output, "{}", self.indent_str)?;
        }
        writeln!(self.output, "{}", s)
    }

    /// 输出不换行的内容
    pub fn write(&mut self, s: &str) -> std::fmt::Result {
        for _ in 0..self.indent {
            write!(self.output, "{}", self.indent_str)?;
        }
        write!(self.output, "{}", s)
    }

    /// 增加缩进
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// 减少缩进
    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    /// 获取当前缩进级别
    pub fn indent_level(&self) -> usize {
        self.indent
    }

    /// 获取输出内容的引用
    pub fn as_str(&self) -> &str {
        &self.output
    }

    /// 获取输出内容并清空缓冲区
    pub fn take(&mut self) -> String {
        self.indent = 0;
        std::mem::take(&mut self.output)
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.output.clear();
        self.indent = 0;
    }

    /// 预分配容量
    pub fn reserve(&mut self, additional: usize) {
        self.output.reserve(additional);
    }

    /// 获取输出长度
    pub fn len(&self) -> usize {
        self.output.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }
}

impl From<CodeBuffer> for String {
    fn from(buffer: CodeBuffer) -> String {
        buffer.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_with_indent() {
        let mut buf = CodeBuffer::new();
        buf.line("fn main() {").unwrap();
        buf.indent();
        buf.line("println!(\"Hello\");").unwrap();
        buf.dedent();
        buf.line("}").unwrap();

        let expected = "fn main() {\n    println!(\"Hello\");\n}\n";
        assert_eq!(buf.as_str(), expected);
    }

    #[test]
    fn test_take() {
        let mut buf = CodeBuffer::new();
        buf.line("test").unwrap();
        let output = buf.take();
        assert_eq!(output, "test\n");
        assert!(buf.is_empty());
    }

    #[test]
    fn test_custom_indent() {
        let mut buf = CodeBuffer::with_indent_str("\t");
        buf.line("fn main() {").unwrap();
        buf.indent();
        buf.line("body").unwrap();

        assert!(buf.as_str().contains("\tbody"));
    }
}
