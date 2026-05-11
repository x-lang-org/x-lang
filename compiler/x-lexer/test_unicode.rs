use x_lexer::new_lexer;
use x_lexer::token::Token;

fn main() {
    let input = "let 变量 = 42";
    println!("Testing input: {}", input);
    let iter = new_lexer(input);
    for result in iter {
        match result {
            Ok((token, span)) => println!("{:?} at {:?}", token, span),
            Err(err) => println!("Error: {:?}", err),
        }
    }
}
