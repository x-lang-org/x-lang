pub fn run_pipeline(
    source: &str,
) -> Result<(x_parser::ast::Program, x_hir::Hir, x_perceus::PerceusIR), String> {
    let parser = x_parser::parser::XParser::new();
    let program = parser
        .parse(source)
        .map_err(|e| format!("解析错误: {}", e))?;

    type_check_with_big_stack(&program)?;

    let hir = x_hir::ast_to_hir(&program).map_err(|e| format!("HIR 转换错误: {}", e))?;

    let pir = x_perceus::analyze_hir(&hir).map_err(|e| format!("Perceus 分析错误: {}", e))?;

    Ok((program, hir, pir))
}

pub fn type_check_with_big_stack(program: &x_parser::ast::Program) -> Result<(), String> {
    // 避免类型检查在复杂 AST 上触发栈溢出：在更大栈空间的线程里执行
    let program = program.clone();
    let handle = std::thread::Builder::new()
        .name("x-typecheck".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || x_typechecker::type_check(&program))
        .map_err(|e| format!("无法启动类型检查线程: {}", e))?;

    match handle.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!("类型检查错误: {}", e)),
        Err(_) => Err("类型检查线程崩溃".to_string()),
    }
}

pub fn format_parse_error(file: &str, source: &str, e: &x_parser::errors::ParseError) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_parse_error_includes_location_and_snippet() {
        let file = "test.x";
        let source = "let x =\n";
        let parser = x_parser::parser::XParser::new();
        let err = parser.parse(source).expect_err("should fail");
        let msg = format_parse_error(file, source, &err);
        assert!(msg.contains("test.x:"), "{msg}");
        assert!(msg.contains(":1:"), "{msg}");
        assert!(msg.contains("="), "{msg}");
    }
}
