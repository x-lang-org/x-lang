use thiserror::Error;

/// 词法分析错误
#[derive(Error, Debug, PartialEq, Clone)]
pub enum LexError {
    #[error("无效的标记")]
    InvalidToken,
    #[error("字符串未闭合")]
    UnclosedString,
    #[error("字符未闭合")]
    UnclosedChar,
    #[error("数字格式错误")]
    InvalidNumber,
}
