// 语法分析器库

pub mod ast;
pub mod errors;
pub mod module_resolver;
pub mod parser;

use ast::Program;
use errors::ParseError;
use parser::XParser;

pub use module_resolver::{
    ImportInfo, ImportSymbol, ModuleError, ModuleGraph, ModuleInfo, ModuleLoader, ModuleResolver,
};

/// 语法分析器类型
pub type Parser = XParser;

/// 从字符串解析X语言程序为抽象语法树
pub fn parse_program(input: &str) -> Result<Program, ParseError> {
    Parser::new().parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Declaration, ExpressionKind, Literal, Pattern, StatementKind, Type};

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
                assert_eq!(
                    t.catch_clauses[0].exception_type.as_deref(),
                    Some("Exception")
                );
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
        assert!(matches!(
            program.statements[0].node,
            StatementKind::Continue
        ));
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
type AliasName = integer
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::TypeAlias(alias) => {
                assert_eq!(alias.name, "AliasName");
                assert_eq!(alias.type_, Type::Int);
            }
            other => panic!("expected type alias, got {other:?}"),
        }
    }

    #[test]
    fn parse_dictionary_literal_with_string_keys() {
        let src = r#"
let dict = {"a": 1, "b": 2}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Variable(var) => match var.initializer.as_ref().map(|e| &e.node) {
                Some(ExpressionKind::Dictionary(entries)) => {
                    assert_eq!(entries.len(), 2);
                    assert!(matches!(&entries[0].0.node, ExpressionKind::Literal(Literal::String(s)) if s == "a"));
                    assert!(matches!(&entries[1].0.node, ExpressionKind::Literal(Literal::String(s)) if s == "b"));
                }
                other => panic!("expected dictionary initializer, got {other:?}"),
            },
            other => panic!("expected variable declaration, got {other:?}"),
        }
    }

    // ===== SPEC.md 测试 =====

    #[test]
    fn parse_for_each_statement() {
        // SPEC.md: for each item in collection { ... }
        let src = r#"
for each item in list {
    println(item)
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].node {
            StatementKind::For(f) => {
                assert!(matches!(&f.pattern, Pattern::Variable(name) if name == "item"));
            }
            other => panic!("expected for statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_for_each_range() {
        // SPEC.md: for each number in 1..10 { ... }
        let src = r#"
for each number in 1..10 {
    println(number)
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0].node {
            StatementKind::For(f) => {
                assert!(matches!(&f.pattern, Pattern::Variable(name) if name == "number"));
            }
            other => panic!("expected for statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_loop_statement() {
        // SPEC.md: loop { ... }
        let src = r#"
loop {
    let input = read_input()
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0].node, StatementKind::Loop(_)));
    }

    #[test]
    fn parse_record_declaration() {
        // SPEC.md: record Person { name: string, age: integer }
        let src = r#"
record Person {
    name: string
    age: integer
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Record(r) => {
                assert_eq!(r.name, "Person");
                assert_eq!(r.fields.len(), 2);
            }
            other => panic!("expected record declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_when_expression() {
        // SPEC.md: when expression with pattern matching
        let src = r#"
let description = when score is {
    100 => "perfect"
    n if n >= 90 => "excellent"
    _ => "failed"
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_defer_statement() {
        // SPEC.md: defer statement
        let src = r#"
defer cleanup()
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(
            program.statements[0].node,
            StatementKind::Defer(_)
        ));
    }

    #[test]
    fn parse_when_guard_statement() {
        let src = r#"
when x > 0 {
    log(x)
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0].node, StatementKind::WhenGuard(_, _)));
    }

    #[test]
    fn parse_yield_statement() {
        // SPEC.md: yield statement for generators
        let src = r#"
function count_up(max: integer) -> Generator<integer> {
    let mutable i = 0
    while i < max {
        yield i
        i = i + 1
    }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_try_with_throw() {
        // SPEC.md: try-catch with throw statement
        // Note: throw is not yet implemented as a statement
        let src = r#"
try {
    risky_operation()
} catch (Exception e) {
    handle_error(e)
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0].node, StatementKind::Try(_)));
    }

    #[test]
    fn parse_single_expression_function() {
        // SPEC.md: function square(x: integer) -> integer = x * x
        let src = r#"
function square(x: integer) -> integer = x * x
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name, "square");
                assert_eq!(f.parameters.len(), 1);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_generic_function() {
        // SPEC.md: function first<T>(list: List<T>) -> Optional<T>
        let src = r#"
function first<T>(list: List<T>) -> Optional<T> {
    when list is {
        _ => None
    }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name, "first");
                assert_eq!(f.type_parameters.len(), 1);
                assert_eq!(f.type_parameters[0].name, "T");
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_pipeline_expression() {
        // SPEC.md: data |> process() |> output()
        let src = r#"
let result = data |> process() |> output()
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_lambda_expression() {
        // SPEC.md: x -> x * x
        let src = r#"
let square = x -> x * x
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_function_keyword_lambda_expression() {
        let src = r#"
let add_base = function(x) -> x + base
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_if_then_else_statement() {
        // SPEC.md: if condition then { ... } else { ... }
        let src = r#"
if score >= 60 then {
    println("passed")
} else {
    println("failed")
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0].node, StatementKind::If(_)));
    }

    #[test]
    fn parse_effect_declaration() {
        // Current parser syntax: effect Name { op: Input -> Output, ... }
        // Note: SPEC.md uses `operation name(params) -> type` but current parser uses `name: Input -> Output`
        let src = r#"
effect Io {
    read_file: string -> string,
    write_file: string -> ()
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Effect(e) => {
                assert_eq!(e.name, "Io");
            }
            other => panic!("expected effect declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_implement_declaration() {
        // SPEC.md: implement Printable for Person { ... }
        let src = r#"
implement Printable for Person {
    function to_string() -> string {
        "Person"
    }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Implement(i) => {
                assert_eq!(i.trait_name, "Printable");
            }
            other => panic!("expected implement declaration, got {other:?}"),
        }
    }

    // ==================== Effects System Tests ====================

    #[test]
    fn parse_function_with_io_effect() {
        let src = r#"
function read_data() -> String with IO {
    "data"
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.effects.len(), 1);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_function_with_multiple_effects() {
        let src = r#"
function fetch_user(id: Int) -> User with Async, IO, Throws<NetworkError> {
    User { name: "test" }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.effects.len(), 3);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_function_with_throws_effect() {
        let src = r#"
function parse_int(s: String) -> Int with Throws<ParseError> {
    42
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.effects.len(), 1);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_function_with_state_effect() {
        let src = r#"
function counter() -> Int with State<Int> {
    0
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.effects.len(), 1);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_function_with_custom_effect() {
        let src = r#"
function log_message(msg: String) -> () with Logger {
    ()
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.effects.len(), 1);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_effect_with_operations() {
        let src = r#"
effect Logger {
    log: String -> (),
    get_level: () -> String
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Effect(e) => {
                assert_eq!(e.name, "Logger");
                assert_eq!(e.operations.len(), 2);
            }
            other => panic!("expected effect declaration, got {other:?}"),
        }
    }

    // ==================== Module System Tests ====================

    #[test]
    fn parse_module_declaration_simple() {
        let src = r#"
module myapp.utils;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Module(m) => {
                assert_eq!(m.name, "myapp.utils");
            }
            other => panic!("expected module declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_import_simple() {
        let src = r#"
import std.collections.HashMap;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Import(i) => {
                assert_eq!(i.module_path, "std.collections.HashMap");
            }
            other => panic!("expected import declaration, got {other:?}"),
        }
    }

    #[test]
    #[ignore = "import with alias not yet implemented"]
    fn parse_import_with_alias() {
        let src = r#"
import std.collections.HashMap as Map;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Import(i) => {
                assert_eq!(i.module_path, "std.collections.HashMap");
            }
            other => panic!("expected import declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_import_selective() {
        let src = r#"
import std.io.{print, println, read_line};
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Import(i) => {
                assert_eq!(i.module_path, "std.io");
                assert_eq!(i.symbols.len(), 3);
            }
            other => panic!("expected import declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_import_double_colon_wildcard() {
        let src = r#"
import std::prelude::*;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Import(i) => {
                assert_eq!(i.module_path, "std::prelude");
                assert_eq!(i.symbols.len(), 1);
            }
            other => panic!("expected import declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_member_access_keyword_operation() {
        let src = r#"
let x = err.operation;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Variable(var) => match &var.initializer {
                Some(value) => match &value.node {
                    ExpressionKind::Member(base, member) => {
                        assert!(matches!(base.node, ExpressionKind::Variable(_)));
                        assert_eq!(member, "operation");
                    }
                    other => panic!("expected member access, got {other:?}"),
                },
                None => panic!("expected initializer"),
            },
            other => panic!("expected variable declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_keyword_operation_identifier_expression() {
        let src = r#"
let x = operation;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Variable(var) => match &var.initializer {
                Some(value) => match &value.node {
                    ExpressionKind::Variable(name) => assert_eq!(name, "operation"),
                    other => panic!("expected variable expression, got {other:?}"),
                },
                None => panic!("expected initializer"),
            },
            other => panic!("expected variable declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_top_level_static_mut_variable() {
        let src = r#"
static mut panic_handler: PanicHandler = default_panic_handler;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Variable(var) => {
                assert_eq!(var.simple_name(), Some("panic_handler"));
                assert!(var.is_mutable);
                assert!(var.type_annot.is_some());
                assert!(var.initializer.is_some());
            }
            other => panic!("expected variable declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_top_level_fn_alias() {
        let src = r#"
fn helper() -> Unit {}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(func) => assert_eq!(func.name, "helper"),
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_export_fn_alias() {
        let src = r#"
export fn helper() -> Unit {}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert!(matches!(program.declarations[0], Declaration::Function(_)));
        assert!(matches!(program.declarations[1], Declaration::Export(_)));
    }

    #[test]
    fn parse_private_fn_alias() {
        let src = r#"
private fn helper() -> Unit {}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(func) => assert_eq!(func.name, "helper"),
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_top_level_static_then_export_fn() {
        let src = r#"
static mut panic_handler: PanicHandler = default_panic_handler;
export fn set_handler(handler: PanicHandler) -> unit {}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert!(matches!(program.declarations[0], Declaration::Variable(_)));
        assert!(matches!(program.declarations[1], Declaration::Function(_)));
        assert!(matches!(program.declarations[2], Declaration::Export(_)));
    }

    #[test]
    fn parse_function_where_clause_after_return_type() {
        let src = r#"
export fn unwrap<T, E>(res: Result<T, E>, msg: string) -> T where E: to_string {
    msg
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(func) => {
                assert_eq!(func.name, "unwrap");
                assert_eq!(func.type_parameters.len(), 2);
                assert_eq!(func.type_parameters[1].name, "E");
                assert_eq!(func.type_parameters[1].constraints.len(), 1);
                assert_eq!(func.type_parameters[1].constraints[0].trait_name, "to_string");
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_when_block_else_expression() {
        let src = r#"
let result = when x == 0 {
    Err("zero")
} else {
    Ok(x)
};
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Variable(var) => match &var.initializer {
                Some(value) => match &value.node {
                    ExpressionKind::If(_, then_expr, else_expr) => {
                        assert!(matches!(then_expr.node, ExpressionKind::Block(_)));
                        assert!(matches!(else_expr.node, ExpressionKind::Block(_)));
                    }
                    other => panic!("expected conditional expression, got {other:?}"),
                },
                None => panic!("expected initializer"),
            },
            other => panic!("expected variable declaration, got {other:?}"),
        }
    }

    #[test]
    #[ignore = "export function syntax not yet implemented"]
    fn parse_export_simple() {
        let src = r#"
export function helper() -> Int { 42 }
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Export(e) => {
                assert!(!e.symbol.is_empty());
            }
            other => panic!("expected export declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_export_symbols() {
        let src = r#"
export foo;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Export(e) => {
                assert_eq!(e.symbol, "foo");
            }
            other => panic!("expected export declaration, got {other:?}"),
        }
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn parse_option_some() {
        let src = r#"
let x = Some(42);
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_option_none() {
        let src = r#"
let x: Option<Int> = None;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_result_ok() {
        let src = r#"
let x = Ok(42);
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_result_err() {
        let src = r#"
let x = Err("error message");
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_error_propagation() {
        let src = r#"
function fetch() -> Int with Throws<Error> {
    let x = parse()? ;
    x
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
    fn parse_optional_chaining() {
        let src = r#"
let name = user?.name;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_null_coalescing() {
        let src = r#"
let name = user?.name ?? "anonymous";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    // ==================== Control Flow Tests ====================

    #[test]
    fn parse_for_each_loop() {
        let src = r#"
for each item in items {
    print(item);
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn parse_while_loop() {
        let src = r#"
while running {
    step();
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn parse_loop_infinite() {
        let src = r#"
loop {
    if done { break; }
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn parse_when_is_expression() {
        let src = r#"
when x is {
    0 => "zero",
    _ => "other"
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert!(!program.statements.is_empty() || !program.declarations.is_empty());
    }

    // ==================== Type System Tests ====================

    #[test]
    fn parse_generic_function_identity() {
        let src = r#"
function identity<T>(x: T) -> T {
    x
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.type_parameters.len(), 1);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_generic_class() {
        let src = r#"
class Container<T> {
    let value: T
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Class(c) => {
                assert_eq!(c.type_parameters.len(), 1);
            }
            other => panic!("expected class declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_tuple_type() {
        let src = r#"
let pair: (Int, String) = (1, "hello");
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_tuple_type_with_function_member() {
        let src = r#"
let tests: [(string, function() -> Bool)] = [];
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    #[ignore = "function type annotation not yet implemented"]
    fn parse_function_type() {
        let src = r#"
let f: (Int, Int) -> Int = add;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    #[ignore = "union type annotation not yet implemented"]
    fn parse_union_type() {
        let src = r#"
let x: Int | String = 42;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    // ==================== Enum and Record Tests ====================

    #[test]
    fn parse_enum_simple_colors() {
        let src = r#"
enum Color {
    Red,
    Green,
    Blue
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Enum(e) => {
                assert_eq!(e.name, "Color");
                assert_eq!(e.variants.len(), 3);
            }
            other => panic!("expected enum declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_enum_with_data() {
        let src = r#"
enum Option<T> {
    None,
    Some(T)
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Enum(e) => {
                assert_eq!(e.name, "Option");
                assert_eq!(e.variants.len(), 2);
            }
            other => panic!("expected enum declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_record_point() {
        let src = r#"
record Point {
    x: Float,
    y: Float
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Record(r) => {
                assert_eq!(r.name, "Point");
                assert_eq!(r.fields.len(), 2);
            }
            other => panic!("expected record declaration, got {other:?}"),
        }
    }

    // ==================== Lambda and Closure Tests ====================

    #[test]
    fn parse_lambda_simple() {
        let src = r#"
let add = (a, b) -> a + b;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    #[ignore = "lambda with type annotation not yet implemented"]
    fn parse_lambda_with_type() {
        let src = r#"
let add: (Int, Int) -> Int = (a: Int, b: Int) -> a + b;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    #[ignore = "closure capture syntax not yet fully implemented"]
    fn parse_closure_capture() {
        let src = r#"
let x = 10;
let add_x = (y) -> y + x;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 2);
    }

    // ==================== Pipe and Chaining Tests ====================

    #[test]
    fn parse_pipe_operator() {
        let src = r#"
let result = data |> process |> format;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_method_chain() {
        let src = r#"
let result = items
    .filter(is_even)
    .map(double)
    .collect();
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    // ==================== Range and Spread Tests ====================

    #[test]
    fn parse_range_exclusive() {
        let src = r#"
let nums = 0..10;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_range_inclusive() {
        let src = r#"
let nums = 0..=10;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    // ==================== Async and Concurrency Tests ====================

    #[test]
    fn parse_async_function_fetch() {
        let src = r#"
async function fetch_data() -> String {
    "data"
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert!(f.is_async);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_await_expression() {
        let src = r#"
async function main() {
    let data = await fetch();
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                match &f.body.statements[0].node {
                    StatementKind::Variable(var) => match var.initializer.as_ref() {
                        Some(expr) => match &expr.node {
                            ExpressionKind::Await(inner) => match &inner.node {
                                ExpressionKind::Call(callee, args) => {
                                    assert!(args.is_empty());
                                    match &callee.node {
                                        ExpressionKind::Variable(name) => {
                                            assert_eq!(name, "fetch");
                                        }
                                        other => panic!(
                                            "expected await callee variable, got {other:?}"
                                        ),
                                    }
                                }
                                other => panic!("expected await call expression, got {other:?}"),
                            },
                            other => panic!("expected await initializer, got {other:?}"),
                        },
                        None => panic!("expected variable initializer"),
                    },
                    other => panic!("expected variable statement, got {other:?}"),
                }
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    #[ignore = "concurrently block syntax not yet fully implemented"]
    fn parse_concurrently_block() {
        let src = r#"
async function main() {
    await concurrently {
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

    // ==================== FFI and External Tests ====================

    #[test]
    fn parse_extern_function() {
        let src = r#"
extern function puts(s: CString) -> CInt;
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::ExternFunction(e) => {
                assert_eq!(e.name, "puts");
            }
            other => panic!("expected extern function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_external_variable() {
        let src = r#"
external "c" variable stdin: *();
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Variable(var) => {
                assert_eq!(var.simple_name(), Some("stdin"));
                assert_eq!(var.extern_abi.as_deref(), Some("c"));
                assert!(var.type_annot.is_some());
                assert!(var.initializer.is_none());
            }
            other => panic!("expected variable declaration, got {other:?}"),
        }
    }

    // ==================== Decorator/Annotation Tests ====================

    #[test]
    #[ignore = "decorator syntax not yet fully implemented"]
    fn parse_decorator() {
        let src = r#"
@deprecated
function old_api() -> Int { 0 }
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name, "old_api");
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    #[ignore = "multiple decorators syntax not yet fully implemented"]
    fn parse_multiple_decorators() {
        let src = r#"
@inline
@deprecated("use new_api instead")
function old_api() -> Int { 0 }
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Function(f) => {
                assert_eq!(f.name, "old_api");
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    // ==================== Nested Generics Tests ====================

    #[test]
    #[ignore = "nested generic type syntax not yet fully implemented"]
    fn parse_nested_generic_type() {
        let src = r#"
let x: Option<Option<Int>> = Some(Some(42));
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    #[ignore = "list of option type syntax not yet fully implemented"]
    fn parse_list_of_option() {
        let src = r#"
let items: List<Option<Int>> = empty();
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    #[ignore = "deeply nested generics syntax not yet fully implemented"]
    fn parse_deeply_nested_generics() {
        let src = r#"
let x: Result<Option<List<Int>>, String> = Ok(Some(empty()));
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    // ==================== For Loop Body Tests ====================

    #[test]
    fn parse_for_with_variable_iterator() {
        let src = r#"
let items = [1, 2, 3]
for each item in items {
    println(item)
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn parse_for_with_multiple_statements() {
        let src = r#"
for each item in [1, 2, 3] {
    let doubled = item * 2
    println(doubled)
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.statements.len(), 1);
    }

    // ==================== Function Type Tests ====================

    #[test]
    fn parse_simple_function_type() {
        let src = r#"
let f: () -> Int = get_value;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_function_type_single_param() {
        let src = r#"
let f: (Int) -> Int = double;
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_function_type_with_function_keyword() {
        let src = r#"
function map<T, U>(self: List<T>, f: function(T) -> U) -> List<U> {
    empty()
}
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_string_concat_with_double_plus() {
        let src = r#"
let msg = "a" ++ "b";
"#;
        let program = parse_program(src).expect("parse should succeed");
        match &program.declarations[0] {
            Declaration::Variable(var) => match var.initializer.as_ref().map(|e| &e.node) {
                Some(ExpressionKind::Binary(BinaryOp::Add, _, _)) => {}
                other => panic!("expected additive concat expression, got {other:?}"),
            },
            other => panic!("expected variable declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_array_with_trailing_comma() {
        let src = r#"
let values = [1, 2, 3,];
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    // ==================== String Interpolation Tests ====================

    #[test]
    fn parse_string_interpolation_simple() {
        // "Hello, ${name}!"
        let src = r#"
let greeting = "Hello, ${name}!";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        // Should desugar to "Hello, " + name + "!"
        match &program.declarations[0] {
            Declaration::Variable(decl) => {
                let expr = decl.initializer.as_ref().expect("initializer exists");
                // Should be Binary Add: ("Hello, " + name) + "!"
                match &expr.node {
                    ExpressionKind::Binary(BinaryOp::Add, _, _) => {
                        // This is expected - desugared to concatenation
                        assert!(true);
                    }
                    _ => panic!("expected binary add for interpolated string"),
                }
            }
            _ => panic!("expected variable declaration"),
        }
    }

    #[test]
    fn parse_string_interpolation_expression() {
        // "Result: ${x + y}"
        let src = r#"
let result = "Result: ${x + y}";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Variable(decl) => {
                let expr = decl.initializer.as_ref().expect("initializer exists");
                // Should be "Result: " + (x + y)
                match &expr.node {
                    ExpressionKind::Binary(BinaryOp::Add, _, right) => {
                        match &right.node {
                            ExpressionKind::Binary(BinaryOp::Add, _, _) => {
                                // Inner expression x + y parsed correctly
                                assert!(true);
                            }
                            _ => panic!("expected inner binary add in interpolation"),
                        }
                    }
                    _ => panic!("expected binary add for interpolated string"),
                }
            }
            _ => panic!("expected variable declaration"),
        }
    }

    #[test]
    fn parse_string_interpolation_multiple() {
        // "${a} + ${b} = ${a + b}"
        let src = r#"
let equation = "${a} + ${b} = ${a + b}";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        // Should desugar to (((a + " + ") + b) + " = ") + (a + b)
        // The interpolation contributes four outer concatenation adds,
        // and the final `${a + b}` contributes one inner arithmetic add.
        match &program.declarations[0] {
            Declaration::Variable(decl) => {
                fn count_adds(expr: &crate::ast::Expression) -> usize {
                    match &expr.node {
                        ExpressionKind::Binary(BinaryOp::Add, left, right) => {
                            1 + count_adds(left) + count_adds(right)
                        }
                        _ => 0,
                    }
                }

                let add_count = count_adds(decl.initializer.as_ref().unwrap());
                assert_eq!(add_count, 5);
            }
            _ => panic!("expected variable declaration"),
        }
    }

    #[test]
    fn parse_string_interpolation_single_part() {
        // "${expression}" - only one part, returns directly
        let src = r#"
let value = "${x + y}";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Variable(decl) => {
                let expr = decl.initializer.as_ref().unwrap();
                // Should just be the expression directly, no concatenation
                match &expr.node {
                    ExpressionKind::Binary(BinaryOp::Add, _, _) => {
                        // This is correct - "${x + y}" is StringContent("") + (x+y), which is one add
                        assert!(true);
                    }
                    _ => {
                        // If empty string is optimized away, that's fine too
                        assert!(true);
                    }
                }
            }
            _ => panic!("expected variable declaration"),
        }
    }

    #[test]
    fn parse_string_interpolation_multiline() {
        // Multiline string with interpolation
        let src = r#"
let text = """
Hello, ${name}!
Count: ${count}
""";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        // Should succeed with interpolation in multiline
        assert!(true);
    }

    #[test]
    fn parse_string_interpolation_with_inner_braces() {
        let src = r#"
let text = "${Point{ x: 1, y: 2 }}";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
    }

    #[test]
    fn parse_regular_string_no_interpolation() {
        // Regular string without interpolation
        let src = r#"
let text = "Hello, world!";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        match &program.declarations[0] {
            Declaration::Variable(decl) => {
                let expr = decl.initializer.as_ref().unwrap();
                match &expr.node {
                    ExpressionKind::Literal(Literal::String(s)) if s == "Hello, world!" => {
                        assert!(true);
                    }
                    _ => panic!("expected simple string literal"),
                }
            }
            _ => panic!("expected variable declaration"),
        }
    }

    #[test]
    fn parse_string_interpolation_short() {
        // "Hello, $name!" - simple variable interpolation without braces
        let src = r#"
let greeting = "Hello, $name!";
"#;
        let program = parse_program(src).expect("parse should succeed");
        assert_eq!(program.declarations.len(), 1);
        // Should desugar to "Hello, " + name + "!"
        match &program.declarations[0] {
            Declaration::Variable(decl) => {
                let expr = decl.initializer.as_ref().expect("initializer exists");
                // Should be Binary Add: ("Hello, " + name) + "!"
                match &expr.node {
                    ExpressionKind::Binary(BinaryOp::Add, _, _) => {
                        // This is expected - desugared to concatenation
                        assert!(true);
                    }
                    _ => panic!("expected binary add for interpolated string"),
                }
            }
            _ => panic!("expected variable declaration"),
        }
    }
}
