use thiserror::Error;
use x_lexer::errors::LexError;
use x_lexer::span::Span;

/// 语法分析错误（带可选源码位置，用于 file:line:col 报告）
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("词法分析错误: {0}")]
    LexError(#[from] LexError),

    #[error("语法错误: {message}")]
    SyntaxError {
        message: String,
        span: Option<Span>,
    },

    #[error("意外的标记: {0:?}")]
    UnexpectedToken(String),

    #[error("缺少期望的标记: {0}")]
    MissingToken(String),

    #[error("文件读取错误: {0}")]
    IoError(#[from] std::io::Error),
}

impl ParseError {
    pub fn span(&self) -> Option<Span> {
        match self {
            ParseError::SyntaxError { span, .. } => *span,
            _ => None,
        }
    }
}