use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// 模块解析器
pub struct ModuleResolver {
    /// 模块搜索路径
    search_paths: Vec<PathBuf>,
    /// 已解析的模块（模块名 -> 源代码）
    resolved: HashMap<String, String>,
    /// 模块导出符号（模块名 -> 导出符号集合）
    module_exports: HashMap<String, HashSet<String>>,
}

impl ModuleResolver {
    pub fn new() -> Self {
        Self {
            search_paths: vec![PathBuf::from(".")],
            resolved: HashMap::new(),
            module_exports: HashMap::new(),
        }
    }

    /// 添加搜索路径
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// 解析模块
    pub fn resolve_module(&mut self, module_name: &str) -> Result<Option<String>, String> {
        // 检查是否已解析
        if self.resolved.contains_key(module_name) {
            return Ok(Some(self.resolved.get(module_name).unwrap().clone()));
        }

        // 在搜索路径中查找模块文件
        for search_path in &self.search_paths {
            let module_file = search_path.join(format!("{}.x", module_name.replace("::", "/")));

            if module_file.exists() {
                let source = std::fs::read_to_string(&module_file)
                    .map_err(|e| format!("无法读取模块文件 {:?}: {}", module_file, e))?;

                self.resolved.insert(module_name.to_string(), source.clone());
                return Ok(Some(source));
            }
        }

        // 模块未找到，可能是标准库或外部依赖
        Ok(None)
    }

    /// 注册模块导出
    pub fn register_exports(&mut self, module_name: &str, exports: HashSet<String>) {
        self.module_exports.insert(module_name.to_string(), exports);
    }

    /// 获取模块导出
    pub fn get_exports(&self, module_name: &str) -> Option<&HashSet<String>> {
        self.module_exports.get(module_name)
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// 多文件编译上下文
pub struct CompilationContext {
    /// 模块解析器
    pub resolver: ModuleResolver,
    /// 已编译的模块
    pub compiled_modules: HashMap<String, x_parser::ast::Program>,
}

impl CompilationContext {
    pub fn new() -> Self {
        Self {
            resolver: ModuleResolver::new(),
            compiled_modules: HashMap::new(),
        }
    }

    /// 编译单个文件
    pub fn compile_file(&mut self, path: &Path) -> Result<x_parser::ast::Program, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取文件 {:?}: {}", path, e))?;

        self.compile_source(&source)
    }

    /// 编译源代码
    pub fn compile_source(&mut self, source: &str) -> Result<x_parser::ast::Program, String> {
        let parser = x_parser::parser::XParser::new();
        let program = parser
            .parse(source)
            .map_err(|e| format!("解析错误: {}", e))?;

        // 收集模块信息和导出
        for decl in &program.declarations {
            match decl {
                x_parser::ast::Declaration::Module(module_decl) => {
                    // 注册当前模块
                    self.resolver.register_exports(&module_decl.name, HashSet::new());
                }
                x_parser::ast::Declaration::Export(export_decl) => {
                    // 记录导出符号
                    // 注意：这里简化处理，实际应该与当前模块关联
                    let _ = &export_decl.symbol;
                }
                _ => {}
            }
        }

        Ok(program)
    }

    /// 链接所有已编译的模块
    pub fn link_all(&self) -> Result<x_parser::ast::Program, String> {
        // 创建一个合并后的程序
        let mut merged_program = x_parser::ast::Program {
            declarations: Vec::new(),
            statements: Vec::new(),
            span: x_lexer::span::Span::default(),
        };

        for program in self.compiled_modules.values() {
            merged_program.declarations.extend(program.declarations.clone());
            merged_program.statements.extend(program.statements.clone());
        }

        Ok(merged_program)
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        Self::new()
    }
}

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

/// 使用大栈空间进行类型检查，并返回格式化的错误消息
pub fn type_check_with_big_stack_formatted(
    program: &x_parser::ast::Program,
    file: &str,
    source: &str,
) -> Result<(), String> {
    let program = program.clone();
    let file = file.to_string();
    let source = source.to_string();
    let handle = std::thread::Builder::new()
        .name("x-typecheck".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || x_typechecker::type_check(&program))
        .map_err(|e| format!("无法启动类型检查线程: {}", e))?;

    match handle.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format_type_error(&file, &source, &e)),
        Err(_) => Err("类型检查线程崩溃".to_string()),
    }
}

/// 格式化解析错误
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

/// 格式化类型错误
pub fn format_type_error(file: &str, source: &str, error: &x_typechecker::errors::TypeError) -> String {
    x_typechecker::format::format_type_error(file, source, error)
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
