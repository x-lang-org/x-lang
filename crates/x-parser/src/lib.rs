// 语法分析器库

pub mod ast;
pub mod errors;
pub mod parser;

use errors::ParseError;
use ast::Program;
use parser::XParser;
use x_lexer::token::Token;

/// 语法分析器类型
pub type Parser = XParser;

/// 从字符串解析X语言程序为抽象语法树
pub fn parse_program(input: &str) -> Result<Program, ParseError> {
    Parser::new().parse(input)
}