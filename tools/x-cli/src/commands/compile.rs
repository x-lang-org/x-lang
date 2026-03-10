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

    // Default compile: use Zig backend
    let parser = x_parser::parser::XParser::new();
    let program = parser
        .parse(&content)
        .map_err(|e| pipeline::format_parse_error(file, &content, &e))?;

    let out_path = output.unwrap_or_else(|| file.strip_suffix(".x").unwrap_or(file));

    // Use Zig backend by default
    let mut backend = x_codegen::zig_backend::ZigBackend::new(x_codegen::zig_backend::ZigBackendConfig {
        target: x_codegen::zig_backend::ZigTarget::Native,
        output_dir: None,
        optimize: false,
        debug_info: true,
    });

    // 注意：当前 Zig 后端只支持从 XIR 生成代码
    // 这里需要实现从 AST 到 XIR 的转换
    // 暂时返回一个错误，后续需要实现完整的转换逻辑
    Err("Zig backend currently only supports XIR input".to_string())
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
            // 注意：当前 Zig 后端只支持从 XIR 生成代码
            // 这里需要实现从 AST 到 XIR 的转换
            // 暂时返回一个错误，后续需要实现完整的转换逻辑
            Err("Zig backend currently only supports XIR input".to_string())
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
            "未知 --emit 阶段: {}（支持: tokens, ast, zig, hir, pir）",
            stage
        )),
    }
}
