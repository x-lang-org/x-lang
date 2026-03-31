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
    use crate::ast::{Declaration, ExpressionKind, Pattern, Statement, StatementKind};

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
        match &program.statements[0].node {
            StatementKind::Match(m) => {
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
        match &program.statements[0].node {
            StatementKind::Match(m) => {
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
        match &program.statements[0].node {
            StatementKind::Try(t) => {
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
        match &program.statements[0].node {
            StatementKind::Try(t) => {
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
        assert!(matches!(program.statements[0].node, StatementKind::Break));
    }

    #[test]
    fn parse_continue_statement() {
        let src = "continue;";
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0].node, StatementKind::Continue));
    }

    #[test]
    fn parse_do_while_statement() {
        let src = r#"
do { x = 1; } while (true);
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].node {
            StatementKind::DoWhile(d) => {
                assert_eq!(d.body.statements.len(), 1);
                assert!(matches!(&d.condition.node, ExpressionKind::Literal(_)));
            }
            other => panic!("expected do-while statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_class_basic() {
        let src = r#"
class Point {
    let x: Int
    let y: Int

    new(x: Int, y: Int) {
        this.x = x
        this.y = y
    }

    function getX() -> Int {
        return this.x
    }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Class(c) => {
                assert_eq!(c.name, "Point");
                assert!(c.extends.is_none());
                assert!(c.implements.is_empty());
                assert_eq!(c.members.len(), 4); // 2 fields + 1 constructor + 1 method
            }
            other => panic!("expected class declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_class_with_extends() {
        let src = r#"
class Dog extends Animal {
    let name: String
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Class(c) => {
                assert_eq!(c.name, "Dog");
                assert_eq!(c.extends.as_deref(), Some("Animal"));
            }
            other => panic!("expected class declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_class_with_implements() {
        let src = r#"
class MyHandler implement Runnable, Serializable {
    function run() { return; }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Class(c) => {
                assert_eq!(c.name, "MyHandler");
                assert_eq!(c.implements.len(), 2);
                assert_eq!(c.implements[0], "Runnable");
                assert_eq!(c.implements[1], "Serializable");
            }
            other => panic!("expected class declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_trait_basic() {
        let src = r#"
trait Drawable {
    function draw() -> Unit;
    function move(x: Int, y: Int) -> Unit;
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Trait(t) => {
                assert_eq!(t.name, "Drawable");
                assert_eq!(t.methods.len(), 2);
            }
            other => panic!("expected trait declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_trait_with_function_bodies() {
        let src = r#"
trait Logger {
    function log(msg: String) -> Unit {
        print(msg);
    }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Trait(t) => {
                assert_eq!(t.name, "Logger");
                assert_eq!(t.methods.len(), 1);
                assert!(!t.methods[0].body.statements.is_empty());
            }
            other => panic!("expected trait declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_async_function() {
        let src = r#"
async function fetchData(id: Int) -> Async<String> {
    return "data";
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name, "fetchData");
                assert!(f.is_async);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_wait_single() {
        let src = r#"
function main() {
    let result = wait fetch(42);
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert!(!f.body.statements.is_empty());
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_wait_together() {
        let src = r#"
function main() {
    wait together {
        fetch(1),
        fetch(2)
    };
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert!(!f.body.statements.is_empty());
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_wait_race() {
        let src = r#"
function main() {
    wait race {
        fetch(1),
        fetch(2)
    };
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert!(!f.body.statements.is_empty());
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_wait_timeout() {
        let src = r#"
function main() {
    wait timeout(5000) {
        fetch(42)
    };
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert!(!f.body.statements.is_empty());
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_class_with_constructor() {
        let src = r#"
class Point {
    let x: Int
    let y: Int

    new(x: Int, y: Int) {
        this.x = x
        this.y = y
    }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_enum_simple() {
        let src = r#"
enum Color {
    Red
    Green
    Blue
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_trait_declaration() {
        let src = r#"
trait Printable {
    function to_string() -> String
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_type_alias() {
        let src = r#"
type AliasName = Int
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }
}
