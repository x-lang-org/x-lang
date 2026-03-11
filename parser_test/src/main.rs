use x_parser::parser::XParser;

fn main() {
    let source = "1 + 1";
    let parser = XParser::new();

    println!("Testing parser with source: {}", source);

    match parser.parse(source) {
        Ok(program) => {
            println!("Parser test completed successfully!");
            println!("Program declarations: {:?}", program.declarations);
            println!("Program statements: {:?}", program.statements);
        }
        Err(e) => {
            println!("Parser error: {:?}", e);
        }
    }
}
