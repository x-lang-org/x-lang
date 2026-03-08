use crate::pipeline;
use crate::utils;

#[allow(unused_variables)]
pub fn exec(
    file: &str,
    output: Option<&str>,
    emit: Option<&str>,
    no_link: bool,
    compiler: Option<&str>,
) -> Result<(), String> {
    let content =
        std::fs::read_to_string(file).map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    if let Some(stage) = emit {
        return emit_stage(file, &content, stage);
    }

    // Default compile: use C backend
    let parser = x_parser::parser::XParser::new();
    let program = parser
        .parse(&content)
        .map_err(|e| pipeline::format_parse_error(file, &content, &e))?;

    let out_path = output.unwrap_or_else(|| file.strip_suffix(".x").unwrap_or(file));

    // Use C backend by default
    let mut config = x_codegen::CBackendConfig::default();
    if let Some(compiler_path) = compiler {
        // Set custom compiler path
        std::env::set_var("CC", compiler_path);
        // Set compiler type to Msvc if path contains cl.exe
        if compiler_path.contains("cl.exe") {
            config.compiler = x_codegen::CCompiler::Msvc;
        }
    }
    let mut backend = x_codegen::CBackend::new(config);
    let c_code = backend
        .generate_from_ast(&program)
        .map_err(|e| format!("C 代码生成失败: {}", e))?;

    let exe_path = if cfg!(windows) {
        format!("{}.exe", out_path)
    } else {
        out_path.to_string()
    };

    let exe_pb = std::path::PathBuf::from(&exe_path);
    backend
        .compile_c_code(&c_code, &exe_pb)
        .map_err(|e| format!("C 编译失败: {}", e))?;

    utils::status("Compiled", &format!("可执行文件: {}", exe_path));
    Ok(())
}

fn emit_stage(file: &str, content: &str, stage: &str) -> Result<(), String> {
    match stage.to_lowercase().as_str() {
        "tokens" => {
            let mut lexer = x_lexer::Lexer::new(content);
            loop {
                match lexer.next_token() {
                    Ok((token, span)) => {
                        println!("{:?}  @ {}..{}", token, span.start, span.end);
                        if matches!(token, x_lexer::token::Token::Eof) {
                            break;
                        }
                    }
                    Err(e) => {
                        return Err(format!("词法错误: {:?}", e));
                    }
                }
            }
            Ok(())
        }
        "ast" => {
            let parser = x_parser::parser::XParser::new();
            let program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            println!("{:#?}", program);
            Ok(())
        }
        "c" => {
            let parser = x_parser::parser::XParser::new();
            let program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            let mut backend = x_codegen::CBackend::new(x_codegen::CBackendConfig::default());
            let c_code = backend
                .generate_from_ast(&program)
                .map_err(|e| format!("C 代码生成失败: {}", e))?;
            print!("{}", c_code);
            Ok(())
        }
        "hir" => {
            let (_, hir, _) = pipeline::run_pipeline(content)?;
            println!("{:#?}", hir);
            Ok(())
        }
        "pir" => {
            let (_, _, pir) = pipeline::run_pipeline(content)?;
            println!("{:#?}", pir);
            Ok(())
        }
        "llvm-ir" => {
            let pipeline_result = pipeline::run_pipeline(content)?;
            #[cfg(feature = "codegen")]
            {
                let (program, _, _) = pipeline_result;
                let config = x_codegen::CodeGenConfig {
                    target: x_codegen::Target::LlvmIr,
                    ..Default::default()
                };
                let bytes = x_codegen::generate_code(&program, &config)
                    .map_err(|e| format!("代码生成错误: {}", e))?;
                if let Ok(s) = String::from_utf8(bytes) {
                    print!("{}", s);
                }
            }
            #[cfg(not(feature = "codegen"))]
            {
                let _ = pipeline_result;
                return Err(
                    "需要启用 codegen 特性并安装 LLVM 21: cargo build --features codegen"
                        .to_string(),
                );
            }
            #[allow(unreachable_code)]
            Ok(())
        }
        _ => Err(format!(
            "未知 --emit 阶段: {}（支持: tokens, ast, c, hir, pir, llvm-ir）",
            stage
        )),
    }
}
