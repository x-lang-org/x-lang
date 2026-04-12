/// X语言的所有词法单元类型
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // 关键字
    Let,
    Mut,
    Mutable, // mutable 关键字 (全称)
    Val,     // 保留 Val/Var 用于向后兼容
    Var,
    Constant, // constant 关键字 (全称)
    Const,
    Function,
    Async,
    Await,
    Class,
    Struct,
    Enum,
    Extends,
    Trait,
    Interface,  // interface 关键字 (trait 的别名)
    Implement,  // implement 关键字
    Implements, // implements 关键字
    Abstract,   // abstract 关键字
    Super,      // super 关键字
    Type,
    Newtype, // newtype 关键字
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
    Then, // then 关键字 (if then else)
    Else,
    For,
    Each, // each 关键字 (for each x in)
    In,
    While,
    Break,    // break 关键字
    Continue, // continue 关键字
    Match,
    When,
    Is,
    Where,
    And,
    Or,
    Not,
    Eq, // eq 关键字 (相等比较)
    Ne, // ne 关键字 (不等比较)
    True,
    False,
    Null,
    Effect,

    // self/Self 关键字
    SelfLower, // self 关键字 (实例引用)
    SelfUpper, // Self 关键字 (自身类型)

    // 并发关键字
    Concurrently, // concurrently 关键字

    Needs,
    Given,
    Wait,
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
    Defer,
    Yield,
    Loop,

    // 类型定义关键字
    Record,      // record 关键字
    Constructor, // constructor 关键字

    // 效果系统关键字
    Perform,   // perform 关键字
    Operation, // operation 关键字

    Extern,
    Foreign,
    External,
    Unsafe,
    As,

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
    /// 原始字符串（反引号）开始/结束
    RawStringQuote,
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
    AmpersandEquals,  // &=
    PipeEquals,       // |=
    LeftShiftEquals,  // <<=
    RightShiftEquals, // >>=
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Caret,
    Equals,
    LessThan,
    GreaterThan,
    LeftShift,  // << 左移
    RightShift, // >> 右移
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
    QuestionMarkDot,
    DoubleQuestionMark,
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
            Token::Mutable => write!(f, "Mutable"),
            Token::Constant => write!(f, "Constant"),
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
