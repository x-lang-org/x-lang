use x_lexer::{new_lexer, token::Token};

fn main() {
    let input = r#"fun main() {
  val message = "Hello, X语言!"
  print(message)
}"#;

    println!("开始测试词法分析器对Hello World程序的解析...");
    println!("--------------------------------------------------");

    let mut lexer = new_lexer(input);
    let mut tokens = Vec::new();

    while let Some(token_result) = lexer.next() {
        match token_result {
            Ok(token) => {
                println!("{}", token);
                tokens.push(token);
            }
            Err(e) => {
                println!("错误: {}", e);
                return;
            }
        }
    }

    println!("--------------------------------------------------");
    println!("词法分析器解析完成，共找到 {} 个标记", tokens.len());
}
