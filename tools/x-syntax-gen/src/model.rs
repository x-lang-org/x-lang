//! Generic syntax definition model

use serde::Serialize;

/// Represents a syntax token type
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum TokenType {
    /// Keywords (needs, given, wait, when, is, can, atomic, etc.)
    Keyword,
    /// Type keywords (int, string, bool, etc.)
    Type,
    /// String literals
    String,
    /// Character literals
    Character,
    /// Numeric literals (integers, floats)
    Number,
    /// Boolean literals (true, false)
    Boolean,
    /// Comments (line and block)
    Comment,
    /// Identifiers (variable names, function names)
    Identifier,
    /// Function names
    Function,
    /// Operators (+, -, *, /, =, etc.)
    Operator,
    /// Punctuation (;, :, ,, (), {}, etc.)
    Punctuation,
    /// Built-in functions and types
    Builtin,
    /// Attributes and annotations
    Attribute,
    /// Error tokens
    Error,
}

/// Represents a syntax rule
#[derive(Debug, Clone, Serialize)]
pub struct SyntaxRule {
    /// Name of the rule
    pub name: String,
    /// Token type this rule applies to
    pub token_type: TokenType,
    /// Regular expression pattern to match
    pub pattern: String,
    /// Optional description
    pub description: Option<String>,
}

/// Represents the complete syntax definition
#[derive(Debug, Clone, Serialize)]
pub struct SyntaxModel {
    /// Language name
    pub language_name: String,
    /// File extensions
    pub file_extensions: Vec<String>,
    /// List of syntax rules
    pub rules: Vec<SyntaxRule>,
    /// List of keywords
    pub keywords: Vec<String>,
    /// List of type keywords
    pub types: Vec<String>,
    /// List of built-in functions
    pub builtins: Vec<String>,
    /// List of operators
    pub operators: Vec<String>,
    /// Comment syntax
    pub comment: CommentSyntax,
}

/// Comment syntax definition
#[derive(Debug, Clone, Serialize)]
pub struct CommentSyntax {
    /// Line comment start
    pub line: String,
    /// Block comment start
    pub block_start: String,
    /// Block comment end
    pub block_end: String,
}

impl Default for SyntaxModel {
    fn default() -> Self {
        Self {
            language_name: "X".to_string(),
            file_extensions: vec!["x".to_string()],
            rules: Vec::new(),
            keywords: Vec::new(),
            types: Vec::new(),
            builtins: Vec::new(),
            operators: Vec::new(),
            comment: CommentSyntax {
                line: "//".to_string(),
                block_start: "/*".to_string(),
                block_end: "*/".to_string(),
            },
        }
    }
}
