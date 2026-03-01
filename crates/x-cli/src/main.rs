// X语言命令行工具

use clap::{Parser, Subcommand};
use colored::*;
use env_logger::Env;
use log::info;
use std::io::Write;

#[derive(Parser)]
#[command(name = "x")]
#[command(version = "0.1.0")]
#[command(about = "X语言工具链 - 编译、运行、打包X语言程序")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "运行X语言源代码")]
    Run {
        #[arg(value_name = "FILE")]
        file: String,
    },

    #[command(about = "编译X语言源代码到可执行文件")]
    Compile {
        #[arg(value_name = "FILE")]
        file: String,

        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,

        /// 输出中间结果: tokens, ast, hir, pir, llvm-ir
        #[arg(long, value_name = "STAGE")]
        emit: Option<String>,

        /// 仅生成目标文件(.o/.obj)，不链接
        #[arg(long)]
        no_link: bool,
    },

    #[command(about = "打包X语言项目")]
    Package {
        #[arg(short, long)]
        output: Option<String>,
    },

    #[command(about = "检查X语言源代码的语法和类型")]
    Check {
        #[arg(value_name = "FILE")]
        file: String,
    },

    #[command(about = "格式化X语言源代码")]
    Format {
        #[arg(value_name = "FILE")]
        file: String,

        #[arg(short, long)]
        write: bool,
    },

    #[command(about = "启动X语言REPL")]
    Repl,
}

/// 完整流水线：读源文件 → 解析 → 类型检查 → HIR → Perceus → (可选) 代码生成
fn run_pipeline(
    source: &str,
) -> Result<
    (
        x_parser::ast::Program,
        x_hir::Hir,
        x_perceus::PerceusIR,
    ),
    String,
> {
    let parser = x_parser::parser::XParser::new();
    let program = parser.parse(source).map_err(|e| format!("解析错误: {}", e))?;

    x_typechecker::type_check(&program).map_err(|e| format!("类型检查错误: {}", e))?;

    let hir = x_hir::ast_to_hir(&program).map_err(|e| format!("HIR 转换错误: {}", e))?;

    let pir = x_perceus::analyze_hir(&hir).map_err(|e| format!("Perceus 分析错误: {}", e))?;

    Ok((program, hir, pir))
}

fn format_parse_error(file: &str, source: &str, e: &x_parser::errors::ParseError) -> String {
    if let Some(span) = e.span() {
        let (line, col) = span.line_col(source);
        let snippet = span.snippet(source);
        format!(
            "{}:{}:{}: {}\n  {} | {}",
            file,
            line,
            col,
            e,
            line,
            snippet.trim_end()
        )
    } else {
        format!("{}: {}", file, e)
    }
}

fn run_command(file: &str) {
    println!("{} 正在运行文件: {}", "执行中".green(), file);

    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} 无法读取文件: {}", "错误".red(), e);
            return;
        }
    };

    let parser = x_parser::parser::XParser::new();
    let program = match parser.parse(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} {}", "错误".red(), format_parse_error(file, &content, &e));
            return;
        }
    };

    if let Err(e) = x_typechecker::type_check(&program) {
        println!("{} 类型检查失败: {}", "错误".red(), e);
        return;
    }

    let mut interpreter = x_interpreter::Interpreter::new();
    match interpreter.run(&program) {
        Ok(()) => println!("{} 运行成功", "成功".green()),
        Err(e) => println!("{} 运行失败: {}", "错误".red(), e),
    }
}

fn compile_command(
    file: &str,
    output: Option<&str>,
    emit: Option<&str>,
    no_link: bool,
) {
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} 无法读取文件: {}", "错误".red(), e);
            return;
        }
    };

    if let Some(stage) = emit {
        let stage_lower = stage.to_lowercase();
        match stage_lower.as_str() {
            "tokens" => {
                let mut lexer = x_lexer::Lexer::new(&content);
                loop {
                    match lexer.next_token() {
                        Ok((token, span)) => {
                            println!("{:?}  @ {}..{}", token, span.start, span.end);
                            if matches!(token, x_lexer::token::Token::Eof) {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("{} 词法错误: {:?}", "错误".red(), e);
                            break;
                        }
                    }
                }
                return;
            }
            "ast" => {
                let parser = x_parser::parser::XParser::new();
                match parser.parse(&content) {
                    Ok(program) => println!("{:#?}", program),
                    Err(e) => println!("{} {}", "错误".red(), format_parse_error(file, &content, &e)),
                }
                return;
            }
            "hir" => {
                match run_pipeline(&content) {
                    Ok((_, hir, _)) => println!("{:#?}", hir),
                    Err(e) => println!("{} {}", "错误".red(), e),
                }
                return;
            }
            "pir" => {
                match run_pipeline(&content) {
                    Ok((_, _, pir)) => println!("{:#?}", pir),
                    Err(e) => println!("{} {}", "错误".red(), e),
                }
                return;
            }
            "llvm-ir" => {
                match run_pipeline(&content) {
                    Ok((program, _, _)) => {
                        #[cfg(feature = "codegen")]
                        {
                            let config = x_codegen::CodeGenConfig {
                                target: x_codegen::CodegenTarget::LlvmIr,
                            };
                            match x_codegen::generate_code(&program, &config) {
                                Ok(bytes) => {
                                    if let Ok(s) = String::from_utf8(bytes) {
                                        print!("{}", s);
                                    }
                                }
                                Err(e) => println!("{} 代码生成错误: {}", "错误".red(), e),
                            }
                        }
                        #[cfg(not(feature = "codegen"))]
                        println!("{} 需要启用 codegen 特性并安装 LLVM 21: cargo build --features codegen", "错误".red());
                    }
                    Err(e) => println!("{} {}", "错误".red(), e),
                }
                return;
            }
            _ => {
                println!(
                    "{} 未知 --emit 阶段: {}（支持: tokens, ast, hir, pir, llvm-ir）",
                    "错误".red(),
                    stage
                );
                return;
            }
        }
    }

    let (program, _, _) = match run_pipeline(&content) {
        Ok(t) => t,
        Err(e) => {
            println!("{} {}", "错误".red(), e);
            return;
        }
    };

    #[cfg(not(feature = "codegen"))]
    {
        println!("{} 编译到目标文件需要启用 codegen 特性并安装 LLVM 21", "提示".yellow());
        println!("  cargo build --features codegen  且设置环境变量 LLVM_SYS_211_PREFIX");
        return;
    }

    #[cfg(feature = "codegen")]
    {
    let out_path = output.unwrap_or_else(|| file.strip_suffix(".x").unwrap_or(file));
    let obj_path = if out_path.ends_with(".o") || out_path.ends_with(".obj") {
        out_path.to_string()
    } else {
        #[cfg(windows)]
        let ext = "obj";
        #[cfg(not(windows))]
        let ext = "o";
        format!("{}.{}", out_path, ext)
    };

    let config = x_codegen::CodeGenConfig {
        target: x_codegen::CodegenTarget::Native,
    };

    let object_bytes = match x_codegen::generate_code(&program, &config) {
        Ok(b) => b,
        Err(e) => {
            println!("{} 代码生成失败: {}", "错误".red(), e);
            return;
        }
    };

    if let Err(e) = std::fs::write(&obj_path, &object_bytes) {
        println!("{} 无法写入目标文件 {}: {}", "错误".red(), obj_path, e);
        return;
    }
    println!("{} 已生成目标文件: {}", "成功".green(), obj_path);

    if no_link {
        println!("{} 未链接（已指定 --no-link）", "提示".yellow());
        return;
    }

    let exe_path = if out_path.ends_with(".o") || out_path.ends_with(".obj") {
        out_path.to_string()
    } else {
        #[cfg(windows)]
        let ext = "exe";
        #[cfg(not(windows))]
        let ext = "";
        if ext.is_empty() {
            out_path.to_string()
        } else {
            format!("{}.{}", out_path, ext)
        }
    };

    let link_ok = try_link(&obj_path, &exe_path);
    if link_ok {
        println!("{} 已生成可执行文件: {}", "成功".green(), exe_path);
    } else {
        println!(
            "{} 自动链接未成功，可手动链接，例如: clang {} -o {}",
            "提示".yellow(),
            obj_path,
            exe_path
        );
    }
    }
}

fn try_link(obj_path: &str, exe_path: &str) -> bool {
    let clang = std::process::Command::new("clang")
        .arg(obj_path)
        .arg("-o")
        .arg(exe_path)
        .output();
    if let Ok(out) = clang {
        if out.status.success() {
            return true;
        }
    }

    let gcc = std::process::Command::new("gcc")
        .arg(obj_path)
        .arg("-o")
        .arg(exe_path)
        .output();
    if let Ok(out) = gcc {
        if out.status.success() {
            return true;
        }
    }

    #[cfg(windows)]
    {
        let link = std::process::Command::new("link")
            .args(["/OUT:", exe_path, obj_path, "/SUBSYSTEM:CONSOLE"])
            .output();
        if let Ok(out) = link {
            if out.status.success() {
                return true;
            }
        }
    }

    false
}

fn package_command(output: Option<&str>) {
    let output = output.unwrap_or("package.tar.gz");
    println!("{} 正在打包项目: {}", "打包中".cyan(), output);
    println!("{} 打包功能尚未实现", "警告".red());
    println!("{} 打包成功", "成功".green());
}

fn check_command(file: &str) {
    println!("{} 正在检查文件: {}", "检查中".yellow(), file);

    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            println!("{} 无法读取文件: {}", "错误".red(), e);
            return;
        }
    };

    let parser = x_parser::parser::XParser::new();
    let program = match parser.parse(&content) {
        Ok(p) => p,
        Err(e) => {
            println!("{} {}", "错误".red(), format_parse_error(file, &content, &e));
            return;
        }
    };

    match x_typechecker::type_check(&program) {
        Ok(()) => println!("{} 检查通过（语法 + 类型）", "成功".green()),
        Err(e) => println!("{} 类型检查失败: {}", "错误".red(), e),
    }
}

fn format_command(file: &str, _write: bool) {
    println!("{} 正在格式化文件: {}", "格式化中".purple(), file);
    println!("{} 格式化功能尚未实现", "警告".red());
    println!("{} 格式化成功", "成功".green());
}

fn repl_command() {
    println!("{} X语言REPL v0.1.0", "欢迎".cyan());
    println!("{} 输入表达式或命令，输入 :quit 退出", "提示".yellow());
    println!("{} 目前REPL功能尚未实现", "警告".red());
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let level = record.level();
            let level_style = match level {
                log::Level::Trace => level.to_string().cyan(),
                log::Level::Debug => level.to_string().blue(),
                log::Level::Info => level.to_string().green(),
                log::Level::Warn => level.to_string().yellow(),
                log::Level::Error => level.to_string().red(),
            };
            writeln!(buf, "[{}] {}", level_style, record.args())
        })
        .init();

    info!("X语言工具链启动");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { file } => run_command(file),
        Commands::Compile {
            file,
            output,
            emit,
            no_link,
        } => compile_command(file, output.as_deref(), emit.as_deref(), *no_link),
        Commands::Package { output } => package_command(output.as_deref()),
        Commands::Check { file } => check_command(file),
        Commands::Format { file, write } => format_command(file, *write),
        Commands::Repl => repl_command(),
    }
}
