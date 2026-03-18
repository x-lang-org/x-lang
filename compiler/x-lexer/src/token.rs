/// X语言的所有词法单元类型
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // 关键字
    Let,
    Mut,
    Val, // 保留 Val/Var 用于向后兼容
    Var,
    Const,
    Function,
    Async,
    Class,
    Struct,
    Enum,
    Extends,
    Trait,
    Interface,   // interface 关键字 (trait 的别名)
    Implement,   // implement 关键字
    Implements,  // implements 关键字
    Abstract,    // abstract 关键字
    Super,       // super 关键字
    Type,
    New,
    Virtual,
    Override,
    Final,
    Static,
    Private,
    Public,
    Protected,
    Module,
    Internal,
    Import,
    Export,
    Return,
    If,
    Else,
    For,
    In,
    While,
    Match,
    When,
    Is,
    Where,
    And,
    Or,
    Not,
    True,
    False,
    Null,

    Needs,
    Given,
    Wait,
    Together,
    Race,
    Timeout,
    Atomic,
    Retry,
    Use,
    With,
    Throws,
    Try,
    Catch,
    Finally,
    Throw,
    Handle,

    Extern,
    Foreign,
    Unsafe,

    // 标识符
    Ident(String),

    // 数字字面量
    HexInt(String),
    OctInt(String),
    BinInt(String),
    Float(String),
    DecimalInt(String),

    // 字符串字面量
    StringQuote,
    MultilineStringQuote,
    StringContent(String),
    /// 字符串插值开始标记 `${`
    InterpolateStart,
    /// 字符串插值结束标记 `}`
    InterpolateEnd,

    // 字符字面量
    CharQuote,
    CharContent(String),

    // 运算符
    AndAnd,
    OrOr,
    Pipe,
    PipePipe,
    DoubleColon,
    Arrow,
    FatArrow,
    DoubleEquals,
    NotEquals,
    LessThanEquals,
    GreaterThanEquals,
    RangeInclusive,
    RangeExclusive,
    PlusEquals,
    MinusEquals,
    AsteriskEquals,
    SlashEquals,
    PercentEquals,
    CaretEquals,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    Equals,
    LessThan,
    GreaterThan,
    NotOperator,
    Colon,
    Dot,
    Comma,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    VerticalBar,
    Ampersand,
    Tilde,
    QuestionMark,
    AtSign,
    Hash,

    // 结束标记
    Eof,
}

use std::fmt;

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Let => write!(f, "Let"),
            Token::Mut => write!(f, "Mut"),
            Token::Val => write!(f, "Val"),
            Token::Var => write!(f, "Var"),
            Token::Function => write!(f, "Function"),
            Token::Ident(s) => write!(f, "Ident({})", s),
            Token::StringQuote => write!(f, "StringQuote"),
            Token::StringContent(s) => write!(f, "StringContent({})", s),
            Token::LeftParen => write!(f, "LeftParen"),
            Token::RightParen => write!(f, "RightParen"),
            Token::LeftBrace => write!(f, "LeftBrace"),
            Token::RightBrace => write!(f, "RightBrace"),
            Token::Equals => write!(f, "Equals"),
            Token::Eof => write!(f, "Eof"),
            _ => write!(f, "{:?}", self),
        }
    }
}
