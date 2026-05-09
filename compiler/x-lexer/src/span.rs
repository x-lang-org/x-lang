//! 源码位置区间（字节偏移），用于错误报告与 IDE。
//! 工业级编译器必备：所有错误带 file:line:col，可点击定位。

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// 将字节偏移转换为 (行号, 列号)，1-based，用于显示。
    pub fn line_col(&self, source: &str) -> (usize, usize) {
        let offset = clamp_to_char_boundary(source, self.start.min(source.len()));
        let head = &source[..offset];
        let line = head.split('\n').count().max(1);
        let last_newline = head.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = offset.saturating_sub(last_newline).saturating_add(1);
        (line, col)
    }

    /// 取该 span 对应的源码片段（用于错误信息中的代码引用）。
    pub fn snippet<'a>(&self, source: &'a str) -> &'a str {
        let start = clamp_to_char_boundary(source, self.start.min(source.len()));
        let end = clamp_to_char_boundary(source, self.end.min(source.len())).max(start);
        &source[start..end]
    }
}

fn clamp_to_char_boundary(source: &str, offset: usize) -> usize {
    let mut clamped = offset.min(source.len());
    while clamped > 0 && !source.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}
