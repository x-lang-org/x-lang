use x_parser::parser::parse_program;
use x_typechecker::type_check;

#[test]
fn test_type_alias_basic() {
    let src = r#"
type MyInt = Int;
let x: MyInt = 42;
let y: Int = x;
"#;
    let program = parse_program(src).expect("parse ok");
    let result = type_check(&program);
    assert!(result.is_ok(), "Type alias should be compatible with its base type: {:?}", result.err());
}

#[test]
fn test_type_alias_mismatch() {
    let src = r#"
type MyInt = Int;
let x: MyInt = "not an int";
"#;
    let program = parse_program(src).expect("parse ok");
    let result = type_check(&program);
    assert!(result.is_err(), "Type alias should still check against its base type");
}
