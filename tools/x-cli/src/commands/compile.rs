use crate::pipeline;
use crate::utils;
use x_codegen::c_backend::{CBackend, CBackendConfig, CStandard};
use x_codegen::zig_backend::ZigTarget;
use x_codegen::{get_code_generator, CodeGenConfig, CodeGenerator};
use x_codegen::Target;

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

    let out_path = output.unwrap_or_else(|| file.strip_suffix(".x").unwrap_or(file));

    // Parse target
    let parsed_target = match target {
        None | Some("native") => Target::Native,
        Some("c") => Target::C,
        Some("wasm" | "wasm32-wasi") => Target::Wasm,
        Some(t) => {
            if let Some(t) = Target::from_str(t) {
                t
            } else {
                return Err(format!(
                    "未知目标平台: {}（支持: native, wasm, wasm32-wasi, wasm32-freestanding, c）",
                    t
                ));
            }
        }
    };

    // Run full pipeline for all targets except direct AST generation
    // Parse the program
    let pipeline_output = pipeline::run_pipeline(&content)?;

    let codegen_config = CodeGenConfig {
        target: parsed_target,
        output_dir: None,
        optimize: release,
        debug_info: !release,
    };

    let mut generator = get_code_generator(parsed_target, codegen_config)
        .map_err(|e| format!("获取代码生成器失败: {}", e))?;

    // Generate from LIR using the full pipeline
    let codegen_output = match parsed_target {
        Target::C => {
            // C backend supports LIR generation
            generator.generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("C代码生成失败: {}", e))
        }
        Target::Native => {
            // For native Zig backend, we still use direct AST generation
            let zig_target = match target {
                None | Some("native") => ZigTarget::Native,
                Some("wasm" | "wasm32-wasi") => ZigTarget::Wasm32Wasi,
                Some("wasm32-freestanding") => ZigTarget::Wasm32Freestanding,
                _ => ZigTarget::Native,
            };
            let mut backend = x_codegen::zig_backend::ZigBackend::new(
                x_codegen::zig_backend::ZigBackendConfig {
                    output_dir: None,
                    optimize: release,
                    debug_info: !release,
                    target: zig_target,
                }
            );
            let output = backend.generate_from_ast(&pipeline_output.ast)
                .map_err(|e| format!("Zig代码生成失败: {}", e))?;

            // Get the generated code
            let zig_code = String::from_utf8_lossy(&output.files[0].content);

            // Compile Zig code to executable
            let output_path = std::path::PathBuf::from(out_path);

            // Display target info
            if let Some(t_str) = target {
                if t_str != "native" {
                    if let Some(zig_target) = match t_str {
                        "wasm" | "wasm32-wasi" => Some(ZigTarget::Wasm32Wasi),
                        "wasm32-freestanding" => Some(ZigTarget::Wasm32Freestanding),
                        _ => None,
                    } {
                        utils::status("Target", zig_target.as_zig_target());
                    }
                }
            }

            backend.compile_zig_code(&zig_code, &output_path)
                .map_err(|e| format!("Zig编译失败: {}", e))?;

            println!("编译成功: {}", output_path.display());
            return Ok(());
        }
        _ => Err(format!("目标平台 {:?} 尚不支持完整编译到二进制", parsed_target)),
    }?;

    let c_code = String::from_utf8_lossy(&codegen_output.files[0].content);

    // If --no-link is specified, just output the C code
    if no_link {
        let c_out_path = format!("{}.c", out_path);
        std::fs::write(&c_out_path, c_code.as_bytes())
            .map_err(|e| format!("无法写入C文件 {}: {}", c_out_path, e))?;
        println!("已生成C代码: {}", c_out_path);
        return Ok(());
    }

    // Compile C code to executable
    match parsed_target {
        Target::C => {
            let output_path = std::path::PathBuf::from(out_path);
            let mut backend = CBackend::new(CBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
                c_standard: CStandard::C23,
                generate_header: false,
            });
            backend.compile_c_code(
                &c_code,
                &output_path,
                CStandard::C23,
                release,
                !release,
            ).map_err(|e| format!("C编译失败: {}", e))?;

            println!("编译成功: {}", output_path.display());
        }
        _ => unreachable!(),
    }

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
            let mut program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            // 自动导入标准库 prelude
            let prelude_decls = crate::pipeline::parse_std_prelude()?;
            let mut new_decls = prelude_decls;
            new_decls.extend(program.declarations);
            program.declarations = new_decls;
            println!("{:#?}", program);
            Ok(())
        }
        "zig" => {
            let parser = x_parser::parser::XParser::new();
            let mut program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            // 自动导入标准库 prelude
            let prelude_decls = crate::pipeline::parse_std_prelude()?;
            let mut new_decls = prelude_decls;
            new_decls.extend(program.declarations);
            program.declarations = new_decls;
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
            let mut program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            // 自动导入标准库 prelude
            let prelude_decls = crate::pipeline::parse_std_prelude()?;
            let mut new_decls = prelude_decls;
            new_decls.extend(program.declarations);
            program.declarations = new_decls;
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
        "rust" => {
            let parser = x_parser::parser::XParser::new();
            let mut program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            // 自动导入标准库 prelude
            let prelude_decls = crate::pipeline::parse_std_prelude()?;
            let mut new_decls = prelude_decls;
            new_decls.extend(program.declarations);
            program.declarations = new_decls;
            let mut backend = x_codegen::rust_backend::RustBackend::new(
                x_codegen::rust_backend::RustBackendConfig::default(),
            );
            let output = backend
                .generate_from_ast(&program)
                .map_err(|e| format!("Rust code generation error: {}", e))?;
            let rust_code = String::from_utf8_lossy(&output.files[0].content);
            println!("{}", rust_code);
            Ok(())
        }
        "c" => {
            let parser = x_parser::parser::XParser::new();
            let mut program = parser
                .parse(content)
                .map_err(|e| pipeline::format_parse_error(file, content, &e))?;
            // 自动导入标准库 prelude
            let prelude_decls = crate::pipeline::parse_std_prelude()?;
            let mut new_decls = prelude_decls;
            new_decls.extend(program.declarations);
            program.declarations = new_decls;
            // 类型检查 - 完整流水线
            pipeline::type_check_with_big_stack(&program)?;
            let mut backend = x_codegen::c_backend::CBackend::new(
                x_codegen::c_backend::CBackendConfig::default(),
            );
            let output = backend
                .generate_from_ast(&program)
                .map_err(|e| format!("C code generation error: {}", e))?;
            let c_code = String::from_utf8_lossy(&output.files[0].content);
            println!("{}", c_code);
            Ok(())
        }
        "hir" => {
            let output = pipeline::run_pipeline(content)?;
            println!("{:#?}", output.hir);
            Ok(())
        }
        "mir" => {
            let output = pipeline::run_pipeline(content)?;
            println!("{:#?}", output.mir);
            Ok(())
        }
        "lir" => {
            let output = pipeline::run_pipeline(content)?;
            println!("{:#?}", output.lir);
            Ok(())
        }

        _ => Err(format!(
            "未知 --emit 阶段: {}（支持: tokens, ast, zig, dotnet, csharp, rust, c, hir, mir, lir）",
            stage
        )),
    }
}
