
use x_lexer::new_lexer;
use x_parser::parser::XParser;

fn main() {
    println!("Testing x-parser with simple input...");

    let sources = ["1", "1 + 1", "println(1)", "fn foo() { 1 }"];

    for (i, source) in sources.iter().enumerate() {
        println!("\n=== Test {}: Source = \"{}\" ===", i + 1, source);

        println!("--- Lexing ---");
        let mut lexer = new_lexer(source);
        let mut lex_ok = true;
        while let Some(token) = lexer.next() {
            match token {
                Ok((tok, span)) => println!("  {:?} @ {:?}", tok, span),
                Err(e) => {
                    println!("  Error: {:?}", e);
                    lex_ok = false;
                    break;
                }
            }
        }

        if lex_ok {
            println!("--- Parsing ---");
            let parser = XParser::new();
            match parser.parse(source) {
                Ok(ast) => println!("  Success: {:?}", ast),
                Err(e) => println!("  Error: {:?}", e),
            }
        }
    }

    println!("\nAll tests completed!");
}
