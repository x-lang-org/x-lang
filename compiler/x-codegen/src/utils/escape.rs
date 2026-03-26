//! 字符串转义工具
//!
//! 提供高效的字符串转义功能，避免多次分配

use std::borrow::Cow;

/// 转义字符串中的特殊字符（单次遍历）
///
/// 默认转义: `\` `"` `\n` `\r` `\t`
pub fn escape_string(s: &str) -> Cow<'_, str> {
    // 快速路径：如果不需要转义，返回借用
    if s.chars().all(|c| !matches!(c, '\\' | '"' | '\n' | '\r' | '\t')) {
        return Cow::Borrowed(s);
    }

    // 需要转义：单次遍历
    let mut escaped = String::with_capacity(s.len() + s.len() / 4);
    for c in s.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(c),
        }
    }
    Cow::Owned(escaped)
}

/// 转义字符串（带自定义转义规则）
pub fn escape_string_with<F>(s: &str, escape_fn: F) -> Cow<'_, str>
where
    F: Fn(char) -> Option<&'static str>,
{
    // 快速路径
    if !s.chars().any(|c| escape_fn(c).is_some()) {
        return Cow::Borrowed(s);
    }

    let mut escaped = String::with_capacity(s.len() + s.len() / 4);
    for c in s.chars() {
        match escape_fn(c) {
            Some(replacement) => escaped.push_str(replacement),
            None => escaped.push(c),
        }
    }
    Cow::Owned(escaped)
}

/// LLVM IR 字符串转义
pub fn escape_llvm_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '\\' => escaped.push_str("\\5C"),
            '"' => escaped.push_str("\\22"),
            '\n' => escaped.push_str("\\0A"),
            '\r' => escaped.push_str("\\0D"),
            '\t' => escaped.push_str("\\09"),
            '\0' => escaped.push_str("\\00"),
            c if c.is_ascii() && !c.is_control() => escaped.push(c),
            c => {
                // 非 ASCII 字符转为十六进制
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                for byte in bytes.bytes() {
                    escaped.push_str(&format!("\\{:02X}", byte));
                }
            }
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_escape() {
        let s = "hello world";
        let result = escape_string(s);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, s);
    }

    #[test]
    fn test_escape_backslash() {
        let result = escape_string("path\\to\\file");
        assert_eq!(result, "path\\\\to\\\\file");
    }

    #[test]
    fn test_escape_quotes() {
        let result = escape_string("say \"hello\"");
        assert_eq!(result, "say \\\"hello\\\"");
    }

    #[test]
    fn test_escape_newline() {
        let result = escape_string("line1\nline2");
        assert_eq!(result, "line1\\nline2");
    }

    #[test]
    fn test_escape_all() {
        let result = escape_string("a\nb\tc\rd\\e\"f");
        assert_eq!(result, "a\\nb\\tc\\rd\\\\e\\\"f");
    }

    #[test]
    fn test_llvm_escape() {
        let result = escape_llvm_string("hello\nworld");
        assert_eq!(result, "hello\\0Aworld");
    }
}
