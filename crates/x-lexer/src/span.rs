//! 源码位置区间（字节偏移），用于错误报告与 IDE。
//! 工业级编译器必备：所有错误带 file:line:col，可点击定位。

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// 将字节偏移转换为 (行号, 列号)，1-based，用于显示。
    pub fn line_col(&self, source: &str) -> (usize, usize) {
        let offset = self.start.min(source.len());
        let head = &source[..offset];
        let line = head.lines().count();
        let col = head
            .lines()
            .last()
            .map(|s| s.len())
            .unwrap_or(0)
            .saturating_add(1);
        (line.max(1), col)
    }

    /// 取该 span 对应的源码片段（用于错误信息中的代码引用）。
    pub fn snippet<'a>(&self, source: &'a str) -> &'a str {
        let start = self.start.min(source.len());
        let end = self.end.min(source.len()).max(start);
        &source[start..end]
    }
}
