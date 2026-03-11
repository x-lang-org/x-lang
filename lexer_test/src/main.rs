use x_lexer::new_lexer;

fn main() {
    let source = "1 + 1";
    let mut lexer = new_lexer(source);

    println!("Testing lexer with source: {}", source);

    while let Some(token) = lexer.next() {
        match token {
            Ok((tok, span)) => println!("Token: {:?}, Span: {:?}", tok, span),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    println!("Lexer test completed successfully!");
}
