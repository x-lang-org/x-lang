use crate::utils;
use colored::*;

/// REPL 目标平台
enum ReplTarget {
    Interpreter,
}

pub fn exec(target: &str) -> Result<(), String> {
    let repl_target = match target.to_lowercase().as_str() {
        "interpreter" | "int" => ReplTarget::Interpreter,
        "javascript" | "js" => {
            return Err("JavaScript REPL 正在开发中，敬请期待！".to_string());
        }
        _ => {
            return Err(format!(
                "不支持的 REPL 目标: '{}'. 可用目标: interpreter",
                target
            ));
        }
    };

    match repl_target {
        ReplTarget::Interpreter => exec_interpreter(),
    }
}

/// 解释器 REPL
fn exec_interpreter() -> Result<(), String> {
    println!("{}", "X语言 REPL v0.1.0 (Interpreter)".cyan().bold());
    println!(
        "输入X语言表达式或声明，输入 {} 退出",
        ":quit".yellow()
    );
    println!("输入 {} 获取帮助", ":help".yellow());
    println!();

    let mut interpreter = x_interpreter::Interpreter::new();
    let mut line_num = 1;
    let mut buffer = String::new();
    let mut in_multiline = false;

    loop {
        let prompt = if in_multiline {
            format!("{:>4} | ", "...".cyan())
        } else {
            format!("{:>4} > ", format!("x[{}]", line_num).cyan())
        };

        print!("{}", prompt);
        let _ = std::io::Write::flush(&mut std::io::stdout());

        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                utils::error(&format!("读取输入失败: {}", e));
                break;
            }
        }

        let trimmed = input.trim();
        match trimmed {
            ":quit" | ":q" | ":exit" => break,
            ":help" | ":h" => {
                print_help();
                continue;
            }
            ":clear" | ":c" => {
                buffer.clear();
                in_multiline = false;
                print!("\x1B[2J\x1B[1;1H");
                let _ = std::io::Write::flush(&mut std::io::stdout());
                continue;
            }
            ":reset" | ":r" => {
                interpreter = x_interpreter::Interpreter::new();
                println!("{}", "解释器已重置".green());
                continue;
            }
            _ => {}
        }

        buffer.push_str(&input);

        let opens = buffer.chars().filter(|&c| c == '{').count();
        let closes = buffer.chars().filter(|&c| c == '}').count();
        if opens > closes {
            in_multiline = true;
            continue;
        }

        let source = buffer.trim().to_string();
        if source.is_empty() {
            buffer.clear();
            continue;
        }

        let parser = x_parser::parser::XParser::new();

        let final_source = if source.starts_with("fun ") {
            source.clone()
        } else {
            format!("fun main() {{\n{}\n}}", source)
        };

        match parser.parse(&final_source) {
            Ok(program) => match interpreter.run(&program) {
                Ok(()) => {}
                Err(e) => {
                    println!("{}: {}", "error".red(), e);
                }
            },
            Err(e) => {
                println!("{}: {}", "parse error".red(), e);
            }
        }

        buffer.clear();
        in_multiline = false;
        line_num += 1;
    }

    println!("\n{}", "再见!".cyan());
    Ok(())
}

fn print_help() {
    println!("REPL 命令:");
    println!("  {}    退出 REPL", ":quit, :q, :exit".yellow());
    println!("  {}      显示帮助", ":help, :h".yellow());
    println!("  {}   清空屏幕", ":clear, :c".yellow());
    println!("  {}   重置解释器", ":reset, :r".yellow());
    println!();
    println!("可以直接输入表达式、声明或多行代码块。");
    println!("多行输入会在检测到未闭合的 {{}} 时自动继续。");
}
