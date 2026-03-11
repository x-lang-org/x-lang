fn main() {
    use x_lexer::new_lexer;
    use x_parser::XParser;
    
    println!("Testing with: 1 + 1");
    let source = "1 + 1";
    
    println!("\n=== Lexing ===");
    let mut lexer = new_lexer(source);
    while let Some(token) = lexer.next() {
        match token {
            Ok((tok, span)) => println!("{:?} @ {:?}", tok, span),
            Err(e) => println!("Error: {:?}", e),
        }
    }
    
    println!("\n=== Parsing ===");
    let parser = XParser::new();
    match parser.parse(source) {
        Ok(ast) => println!("AST: {:?}", ast),
        Err(e) => println!("Error: {:?}", e),
    }
    
    println!("Done!");
}
