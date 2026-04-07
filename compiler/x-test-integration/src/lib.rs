//! X Language Compiler Integration Tests
//!
//! This module provides comprehensive integration tests for:
//! 1. Each compiler stage (Lexer, Parser, TypeChecker, HIR, MIR, LIR)
//! 2. Each code generation backend
//! 3. End-to-end compilation and execution
//!
//! Run with: `cd compiler && cargo test -p x-test-integration`

#[cfg(test)]
mod stage_tests {
    use x_parser::parser::XParser;
    use x_hir::ast_to_hir;
    use x_mir::lower_hir_to_mir;
    use x_lir::lower_mir_to_lir;
    use x_interpreter::Interpreter;

    /// Test source code samples for different complexity levels
    mod sources {
        /// Simple print statement
        pub const HELLO: &str = r#"println("Hello, World!")"#;

        /// Function with return value
        pub const RETURN_42: &str = r#"
            function main() -> integer {
                return 42
            }
        "#;

        /// Variable declaration and mutation
        pub const VARIABLE_MUTATION: &str = r#"
            let mutable x = 10
            x = x + 5
            println(x)
        "#;

        /// For loop over array
        pub const FOR_LOOP: &str = r#"
            let list = [1, 2, 3]
            for item in list {
                println(item)
            }
        "#;

        /// Conditional expression
        pub const IF_EXPR: &str = r#"
            let x = 5
            if x > 3 {
                println("yes")
            } else {
                println("no")
            }
        "#;

        /// Match expression
        pub const MATCH_EXPR: &str = r#"
            let value = Option.Some(42)
            match value {
                Option.Some(v) => println(v)
                Option.None => println("none")
            }
        "#;

        /// Function definition and call
        pub const FUNCTION_CALL: &str = r#"
            function add(a: integer, b: integer) -> integer {
                return a + b
            }
            println(add(3, 4))
        "#;

        /// Recursive function
        pub const RECURSIVE_FIB: &str = r#"
            function fib(n: integer) -> integer {
                if n <= 1 {
                    return n
                }
                return fib(n - 1) + fib(n - 2)
            }
            println(fib(10))
        "#;

        /// While loop
        pub const WHILE_LOOP: &str = r#"
            let mutable i = 0
            while i < 3 {
                println(i)
                i = i + 1
            }
        "#;

        /// Array literal
        pub const ARRAY_LITERAL: &str = r#"
            let arr = [1, 2, 3, 4, 5]
            println(arr[0])
        "#;

        /// String operations
        pub const STRING_OPS: &str = r#"
            let s = "Hello"
            println(s)
        "#;

        /// Arithmetic expressions
        pub const ARITHMETIC: &str = r#"
            let a = 10
            let b = 3
            println(a + b)
            println(a - b)
            println(a * b)
            println(a / b)
            println(a % b)
        "#;

        /// Boolean operations
        pub const BOOLEAN_OPS: &str = r#"
            let a = true
            let b = false
            println(a and b)
            println(a or b)
            println(not a)
        "#;

        /// Nested function calls
        pub const NESTED_CALLS: &str = r#"
            function square(x: integer) -> integer {
                return x * x
            }
            function double(x: integer) -> integer {
                return x + x
            }
            println(square(double(2)))
        "#;

        /// Multiple functions
        pub const MULTIPLE_FUNCTIONS: &str = r#"
            function increment(x: integer) -> integer {
                return x + 1
            }
            function decrement(x: integer) -> integer {
                return x - 1
            }
            println(increment(5))
            println(decrement(5))
        "#;

        /// Empty function
        pub const EMPTY_FUNCTION: &str = r#"
            function do_nothing() -> void {
                println("done")
            }
            do_nothing()
        "#;

        /// Type annotations
        pub const TYPE_ANNOTATIONS: &str = r#"
            let x: integer = 42
            let y: string = "hello"
            let z: boolean = true
            println(x)
            println(y)
            println(z)
        "#;
    }

    /// Stage 2: Parser Tests
    mod parser {
        use super::*;

        #[test]
        fn test_parse_hello() {
            let source = sources::HELLO;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok(), "Parse error: {:?}", result.err());
        }

        #[test]
        fn test_parse_return_42() {
            let source = sources::RETURN_42;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
            let program = result.unwrap();
            assert!(!program.declarations.is_empty() || !program.statements.is_empty());
        }

        #[test]
        fn test_parse_variable() {
            let source = sources::VARIABLE_MUTATION;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_for_loop() {
            let source = sources::FOR_LOOP;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_function() {
            let source = sources::FUNCTION_CALL;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_while_loop() {
            let source = sources::WHILE_LOOP;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_array() {
            let source = sources::ARRAY_LITERAL;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_nested_calls() {
            let source = sources::NESTED_CALLS;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
        }

        #[test]
        fn test_parse_multiple_functions() {
            let source = sources::MULTIPLE_FUNCTIONS;
            let parser = XParser::new();
            let result = parser.parse(source);
            assert!(result.is_ok());
        }
    }

    /// Stage 4: HIR Generation Tests
    mod hir {
        use super::*;

        #[test]
        fn test_hir_hello() {
            let source = sources::HELLO;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let result = x_hir::ast_to_hir(&ast);
            assert!(result.is_ok(), "HIR error: {:?}", result.err());
        }

        #[test]
        fn test_hir_return_42() {
            let source = sources::RETURN_42;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let result = x_hir::ast_to_hir(&ast);
            assert!(result.is_ok());
        }

        #[test]
        fn test_hir_function() {
            let source = sources::FUNCTION_CALL;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let result = x_hir::ast_to_hir(&ast);
            assert!(result.is_ok());
        }

        #[test]
        fn test_hir_while_loop() {
            let source = sources::WHILE_LOOP;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let result = x_hir::ast_to_hir(&ast);
            assert!(result.is_ok(), "HIR error: {:?}", result.err());
        }

        #[test]
        fn test_hir_arithmetic() {
            let source = sources::ARITHMETIC;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let result = x_hir::ast_to_hir(&ast);
            assert!(result.is_ok(), "HIR error: {:?}", result.err());
        }

        #[test]
        fn test_hir_multiple_functions() {
            let source = sources::MULTIPLE_FUNCTIONS;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let result = x_hir::ast_to_hir(&ast);
            assert!(result.is_ok(), "HIR error: {:?}", result.err());
        }
    }

    /// Stage 5: MIR Generation Tests
    mod mir {
        use super::*;

        #[test]
        fn test_mir_hello() {
            let source = sources::HELLO;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let result = x_mir::lower_hir_to_mir(&hir);
            assert!(result.is_ok(), "MIR error: {:?}", result.err());
        }

        #[test]
        fn test_mir_return_42() {
            let source = sources::RETURN_42;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let result = x_mir::lower_hir_to_mir(&hir);
            assert!(result.is_ok());
        }

        #[test]
        fn test_mir_while_loop() {
            let source = sources::WHILE_LOOP;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let result = x_mir::lower_hir_to_mir(&hir);
            assert!(result.is_ok(), "MIR error: {:?}", result.err());
        }

        #[test]
        fn test_mir_function() {
            let source = sources::FUNCTION_CALL;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let result = x_mir::lower_hir_to_mir(&hir);
            assert!(result.is_ok());
        }
    }

    /// Stage 6: LIR Generation Tests
    mod lir {
        use super::*;

        #[test]
        fn test_lir_hello() {
            let source = sources::HELLO;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let mir = x_mir::lower_hir_to_mir(&hir).unwrap();
            let result = x_lir::lower_mir_to_lir(&mir);
            assert!(result.is_ok(), "LIR error: {:?}", result.err());
        }

        #[test]
        fn test_lir_return_42() {
            let source = sources::RETURN_42;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let mir = x_mir::lower_hir_to_mir(&hir).unwrap();
            let result = x_lir::lower_mir_to_lir(&mir);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lir_function() {
            let source = sources::FUNCTION_CALL;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let mir = x_mir::lower_hir_to_mir(&hir).unwrap();
            let result = x_lir::lower_mir_to_lir(&mir);
            assert!(result.is_ok());
        }

        #[test]
        fn test_lir_while_loop() {
            let source = sources::WHILE_LOOP;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let mir = x_mir::lower_hir_to_mir(&hir).unwrap();
            let result = x_lir::lower_mir_to_lir(&mir);
            assert!(result.is_ok(), "LIR error: {:?}", result.err());
        }

        #[test]
        fn test_lir_multiple_functions() {
            let source = sources::MULTIPLE_FUNCTIONS;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let mir = x_mir::lower_hir_to_mir(&hir).unwrap();
            let result = x_lir::lower_mir_to_lir(&mir);
            assert!(result.is_ok(), "LIR error: {:?}", result.err());
        }
    }

    /// Full Pipeline Tests
    mod pipeline {
        use super::*;

        #[test]
        fn test_full_pipeline_hello() {
            let source = sources::HELLO;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let mir = x_mir::lower_hir_to_mir(&hir).unwrap();
            let lir = x_lir::lower_mir_to_lir(&mir).unwrap();
            assert!(!lir.declarations.is_empty());
        }

        #[test]
        fn test_full_pipeline_return_42() {
            let source = sources::RETURN_42;
            let parser = XParser::new();
            let ast = parser.parse(source).unwrap();
            let hir = x_hir::ast_to_hir(&ast).unwrap();
            let mir = x_mir::lower_hir_to_mir(&hir).unwrap();
            let lir = x_lir::lower_mir_to_lir(&mir).unwrap();
            assert!(!lir.declarations.is_empty());
        }
    }

    /// Interpreter Tests (AST execution)
    mod interpreter {
        use super::*;

        #[test]
        fn test_interpret_hello() {
            let source = sources::HELLO;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok(), "Runtime error: {:?}", result.err());
        }

        #[test]
        fn test_interpret_return_42() {
            let source = sources::RETURN_42;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok());
        }

        #[test]
        fn test_interpret_for_loop() {
            let source = sources::FOR_LOOP;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok(), "Error: {:?}", result.err());
        }

        #[test]
        fn test_interpret_recursive() {
            let source = sources::RECURSIVE_FIB;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok());
        }

        #[test]
        fn test_interpret_while_loop() {
            let source = sources::WHILE_LOOP;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok(), "Error: {:?}", result.err());
        }

        #[test]
        fn test_interpret_array() {
            let source = sources::ARRAY_LITERAL;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok(), "Error: {:?}", result.err());
        }

        #[test]
        fn test_interpret_arithmetic() {
            let source = sources::ARITHMETIC;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok(), "Error: {:?}", result.err());
        }

        #[test]
        fn test_interpret_nested_calls() {
            let source = sources::NESTED_CALLS;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok(), "Error: {:?}", result.err());
        }

        #[test]
        fn test_interpret_multiple_functions() {
            let source = sources::MULTIPLE_FUNCTIONS;
            let parser = XParser::new();
            let program = parser.parse(source).unwrap();

            let mut interpreter = Interpreter::new();
            let result = interpreter.run(&program);
            assert!(result.is_ok(), "Error: {:?}", result.err());
        }
    }
}