use crate::pipeline;
use crate::utils;
use x_codegen::zig_backend::ZigTarget;

#[allow(unused_variables)]
pub fn exec(
    file: &str,
    output: Option<&str>,
    emit: Option<&str>,
    no_link: bool,
    release: bool,
    target: Option<&str>,
) -> Result<(), String> {
    let content =
        std::fs::read_to_string(file).map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    if let Some(stage) = emit {
        return emit_stage(file, &content, stage);
    }

    // Default compile: use Zig backend
    // Parse the program
    let parser = x_parser::parser::XParser::new();
    let program = parser
        .parse(&content)
        .map_err(|e| format!("解析错误: {}", e))?;

    // Type check
    pipeline::type_check_with_big_stack(&program)?;

    let out_path = output.unwrap_or_else(|| file.strip_suffix(".x").unwrap_or(file));

    // Parse target
    let zig_target = match target {
        None | Some("native") => ZigTarget::Native,
        Some("wasm" | "wasm32-wasi") => ZigTarget::Wasm32Wasi,
        Some("wasm32-freestanding") => ZigTarget::Wasm32Freestanding,
        Some(t) => return Err(format!("未知目标平台: {}（支持: native, wasm, wasm32-wasi, wasm32-freestanding）", t)),
    };

    // Use Zig backend by default
    let mut backend =
        x_codegen::zig_backend::ZigBackend::new(x_codegen::zig_backend::ZigBackendConfig {
            output_dir: None,
            optimize: release,
            debug_info: !release,
            target: zig_target,
        });

    // Display target info
    if zig_target != ZigTarget::Native {
        utils::status("Target", zig_target.as_zig_target());
    }

    // Generate Zig code from AST (direct code generation)
    let codegen_output = backend
        .generate_from_ast(&program)
        .map_err(|e| format!("代码生成失败: {}", e))?;

    let zig_code = String::from_utf8_lossy(&codegen_output.files[0].content);

    // If --no-link is specified, just output the Zig code
    if no_link {
        let zig_out_path = format!("{}.zig", out_path);
        std::fs::write(&zig_out_path, zig_code.as_bytes())
            .map_err(|e| format!("无法写入Zig文件 {}: {}", zig_out_path, e))?;
        println!("已生成Zig代码: {}", zig_out_path);
        return Ok(());
    }

    // Compile Zig code to executable
    let output_path = std::path::PathBuf::from(out_path);
    backend
        .compile_zig_code(&zig_code, &output_path)
        .map_err(|e| format!("Zig编译失败: {}", e))?;

    println!("编译成功: {}", output_path.display());
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
        "zig" => {
            let parser = x_parser::parser::XParser::new();
            let program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            let mut backend =
                x_codegen::zig_backend::ZigBackend::new(x_codegen::zig_backend::ZigBackendConfig::default());
            let output = backend
                .generate_from_ast(&program)
                .map_err(|e| format!("Zig代码生成失败: {}", e))?;
            let zig_code = String::from_utf8_lossy(&output.files[0].content);
            println!("{}", zig_code);
            Ok(())
        }
        "dotnet" | "csharp" => {
            let parser = x_parser::parser::XParser::new();
            let program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            let mut backend = x_codegen::csharp_backend::CSharpBackend::new(
                x_codegen::csharp_backend::CSharpBackendConfig::default(),
            );
            let output = backend
                .generate_from_ast(&program)
                .map_err(|e| format!("C# code generation error: {}", e))?;
            let csharp_code = String::from_utf8_lossy(&output.files[0].content);
            println!("{}", csharp_code);
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
        _ => Err(format!(
            "未知 --emit 阶段: {}（支持: tokens, ast, zig, dotnet, csharp, hir, pir）",
            stage
        )),
    }
}
