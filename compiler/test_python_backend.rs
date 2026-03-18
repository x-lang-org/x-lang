use x_parser::ast::{self, Program};
use x_codegen::python_backend::{PythonBackend, PythonBackendConfig};

fn main() {
    let program = Program {
        declarations: vec![
            ast::Declaration::Function(ast::FunctionDecl {
                name: "main".to_string(),
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![
                        ast::Statement::Expression(
                            ast::Expression::Call(
                                Box::new(ast::Expression::Variable("print".to_string())),
                                vec![ast::Expression::Literal(ast::Literal::String("Hello, World!".to_string()))],
                            )
                        ),
                    ],
                },
                is_async: false,
            }),
        ],
        statements: vec![],
    };

    let mut backend = PythonBackend::new(PythonBackendConfig::default());
    let output = backend.generate_from_ast(&program).unwrap();
    for file in &output.files {
        println!("Generated file: {:?}", file.path);
        println!("{}", String::from_utf8_lossy(&file.content));
    }
}
