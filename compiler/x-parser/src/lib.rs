// 语法分析器库

pub mod ast;
pub mod errors;
pub mod parser;

use ast::Program;
use errors::ParseError;
use parser::XParser;

/// 语法分析器类型
pub type Parser = XParser;

/// 从字符串解析X语言程序为抽象语法树
pub fn parse_program(input: &str) -> Result<Program, ParseError> {
    Parser::new().parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Declaration, Pattern, Statement};

    #[test]
    fn parse_module_import_export() {
        let src = r#"
module foo;
import std.io;
export foo;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 3);
        assert!(matches!(program.declarations[0], Declaration::Module(_)));
        assert!(matches!(program.declarations[1], Declaration::Import(_)));
        assert!(matches!(program.declarations[2], Declaration::Export(_)));
    }

    #[test]
    fn parse_match_statement_basic() {
        let src = r#"
let x = 1;
match x {
  _ { return 1; }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Match(m) => {
                assert_eq!(m.cases.len(), 1);
                assert!(matches!(m.cases[0].pattern, Pattern::Wildcard));
            }
            other => panic!("expected match statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_match_statement_guard_and_or_pattern() {
        let src = r#"
match x {
  a | b when true { return; }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.statements[0] {
            Statement::Match(m) => {
                assert_eq!(m.cases.len(), 1);
                assert!(m.cases[0].guard.is_some());
                assert!(matches!(m.cases[0].pattern, Pattern::Or(_, _)));
            }
            other => panic!("expected match statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_try_catch_finally() {
        let src = r#"
try { return 1; }
catch { return 2; }
finally { return 3; }
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Try(t) => {
                assert_eq!(t.catch_clauses.len(), 1);
                assert!(t.finally_block.is_some());
            }
            other => panic!("expected try statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_try_catch_with_parens_type_and_var() {
        let src = r#"
try { return; }
catch (Exception e) { return; }
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.statements[0] {
            Statement::Try(t) => {
                assert_eq!(t.catch_clauses.len(), 1);
                assert_eq!(t.catch_clauses[0].exception_type.as_deref(), Some("Exception"));
                assert_eq!(t.catch_clauses[0].variable_name.as_deref(), Some("e"));
            }
            other => panic!("expected try statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_break_statement() {
        let src = "break;";
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Statement::Break));
    }

    #[test]
    fn parse_continue_statement() {
        let src = "continue;";
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Statement::Continue));
    }

    #[test]
    fn parse_do_while_statement() {
        let src = r#"
do { x = 1; } while (true);
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::DoWhile(d) => {
                assert_eq!(d.body.statements.len(), 1);
                assert!(matches!(&d.condition, crate::ast::Expression::Literal(_)));
            }
            other => panic!("expected do-while statement, got {other:?}"),
        }
    }
}
