//! Map x-lexer tokens to generic syntax model

use anyhow::Result;

use crate::model::{SyntaxModel, SyntaxRule, TokenType};

/// Build the complete syntax model from x-lexer token definitions
pub fn build_syntax_model() -> Result<SyntaxModel> {
    let mut model = SyntaxModel::default();

    // Add X language specific keywords
    model.keywords = vec![
        "needs", "given", "wait", "when", "is", "can", "atomic",
        "fn", "func", "function", "let", "var", "const", "if", "else",
        "while", "for", "loop", "match", "return", "break", "continue",
        "struct", "enum", "type", "impl", "trait", "use", "mod", "pub",
        "mut", "ref", "self", "Self", "true", "false", "null", "None",
        "Some", "Ok", "Err",
    ].into_iter().map(|s| s.to_string()).collect();

    // Add type keywords
    model.types = vec![
        "int", "i8", "i16", "i32", "i64", "isize",
        "uint", "u8", "u16", "u32", "u64", "usize",
        "float", "f32", "f64", "bool", "string", "char",
        "void", "never", "any",
    ].into_iter().map(|s| s.to_string()).collect();

    // Add built-in functions
    model.builtins = vec![
        "print", "println", "dbg", "panic", "assert", "assert_eq",
        "len", "push", "pop", "insert", "remove", "get", "set",
        "parse", "to_string", "to_int", "to_float",
    ].into_iter().map(|s| s.to_string()).collect();

    // Add operators
    model.operators = vec![
        "+", "-", "*", "/", "%", "=", "+=", "-=", "*=", "/=", "%=",
        "==", "!=", "<", ">", "<=", ">=", "&&", "||", "!", "~", "&",
        "|", "^", "<<", ">>", "->", "=>", "::", ".", "..", "...",
    ].into_iter().map(|s| s.to_string()).collect();

    // Add syntax rules
    model.rules = vec![
        // Keywords
        SyntaxRule {
            name: "keyword.control".to_string(),
            token_type: TokenType::Keyword,
            pattern: format!(r"\b({})\b", model.keywords.join("|")),
            description: Some("Control keywords".to_string()),
        },

        // Types
        SyntaxRule {
            name: "storage.type".to_string(),
            token_type: TokenType::Type,
            pattern: format!(r"\b({})\b", model.types.join("|")),
            description: Some("Type keywords".to_string()),
        },

        // Strings
        SyntaxRule {
            name: "string.quoted.double".to_string(),
            token_type: TokenType::String,
            pattern: r#""[^"\\]*(\\.[^"\\]*)*""#.to_string(),
            description: Some("Double-quoted string literals".to_string()),
        },
        SyntaxRule {
            name: "string.quoted.single".to_string(),
            token_type: TokenType::String,
            pattern: r"'[^'\\]*(\\.[^'\\]*)*'".to_string(),
            description: Some("Single-quoted string literals".to_string()),
        },

        // Characters
        SyntaxRule {
            name: "constant.character".to_string(),
            token_type: TokenType::Character,
            pattern: r"'\\?.'".to_string(),
            description: Some("Character literals".to_string()),
        },

        // Numbers
        SyntaxRule {
            name: "constant.numeric".to_string(),
            token_type: TokenType::Number,
            pattern: r"\b[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?\b".to_string(),
            description: Some("Numeric literals".to_string()),
        },

        // Booleans
        SyntaxRule {
            name: "constant.language.boolean".to_string(),
            token_type: TokenType::Boolean,
            pattern: r"\b(true|false)\b".to_string(),
            description: Some("Boolean literals".to_string()),
        },

        // Comments
        SyntaxRule {
            name: "comment.line.double-slash".to_string(),
            token_type: TokenType::Comment,
            pattern: r"//.*$".to_string(),
            description: Some("Line comments".to_string()),
        },
        SyntaxRule {
            name: "comment.block".to_string(),
            token_type: TokenType::Comment,
            pattern: r"/\*[\s\S]*?\*/".to_string(),
            description: Some("Block comments".to_string()),
        },

        // Identifiers
        SyntaxRule {
            name: "variable.other".to_string(),
            token_type: TokenType::Identifier,
            pattern: r"\b[a-zA-Z_][a-zA-Z0-9_]*\b".to_string(),
            description: Some("Identifiers".to_string()),
        },

        // Operators
        SyntaxRule {
            name: "keyword.operator".to_string(),
            token_type: TokenType::Operator,
            pattern: format!(r"({})", regex::escape(&model.operators.join("|"))),
            description: Some("Operators".to_string()),
        },

        // Punctuation
        SyntaxRule {
            name: "punctuation".to_string(),
            token_type: TokenType::Punctuation,
            pattern: r"[{}()\[\],;:.]".to_string(),
            description: Some("Punctuation".to_string()),
        },

        // Built-ins
        SyntaxRule {
            name: "support.function".to_string(),
            token_type: TokenType::Builtin,
            pattern: format!(r"\b({})\b", model.builtins.join("|")),
            description: Some("Built-in functions".to_string()),
        },
    ];

    Ok(model)
}
