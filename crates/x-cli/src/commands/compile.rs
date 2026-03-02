use crate::pipeline;
use crate::utils;

#[allow(unused_variables)]
pub fn exec(
    file: &str,
    output: Option<&str>,
    emit: Option<&str>,
    no_link: bool,
) -> Result<(), String> {
    let content =
        std::fs::read_to_string(file).map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    if let Some(stage) = emit {
        return emit_stage(file, &content, stage);
    }

    let pipeline_result = pipeline::run_pipeline(&content);
    let (program, _, _) = match pipeline_result {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    #[cfg(not(feature = "codegen"))]
    {
        let _ = &program;
        utils::note("编译到目标文件需要启用 codegen 特性并安装 LLVM 21");
        utils::note("  cargo build --features codegen  且设置环境变量 LLVM_SYS_211_PREFIX");
        return Ok(());
    }

    #[cfg(feature = "codegen")]
    {
        let out_path = output.unwrap_or_else(|| file.strip_suffix(".x").unwrap_or(file));
        let obj_path = if out_path.ends_with(".o") || out_path.ends_with(".obj") {
            out_path.to_string()
        } else {
            let ext = if cfg!(windows) { "obj" } else { "o" };
            format!("{}.{}", out_path, ext)
        };

        let config = x_codegen::CodeGenConfig {
            target: x_codegen::Target::Native,
            ..Default::default()
        };
        let object_bytes = x_codegen::generate_code(&program, &config)
            .map_err(|e| format!("代码生成失败: {}", e))?;

        std::fs::write(&obj_path, &object_bytes)
            .map_err(|e| format!("无法写入目标文件 {}: {}", obj_path, e))?;
        utils::status("Generated", &format!("目标文件: {}", obj_path));

        if no_link {
            utils::note("未链接（已指定 --no-link）");
            return Ok(());
        }

        let exe_path = if out_path.ends_with(".o") || out_path.ends_with(".obj") {
            out_path.to_string()
        } else if cfg!(windows) {
            format!("{}.exe", out_path)
        } else {
            out_path.to_string()
        };

        if pipeline::try_link(&obj_path, &exe_path) {
            utils::status("Linked", &format!("可执行文件: {}", exe_path));
        } else {
            utils::warning(&format!(
                "自动链接未成功，可手动链接: clang {} -o {}",
                obj_path, exe_path
            ));
        }
        Ok(())
    }
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
            "未知 --emit 阶段: {}（支持: tokens, ast, hir, pir, llvm-ir）",
            stage
        )),
    }
}
