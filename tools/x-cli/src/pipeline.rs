use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// 查找标准库文件路径
fn find_stdlib_path() -> Result<PathBuf, String> {
    // 尝试从项目根目录查找
    let candidates = [
        PathBuf::from("../../library/stdlib"),
        PathBuf::from("../library/stdlib"),
        PathBuf::from("library/stdlib"),
        PathBuf::from("/library/stdlib"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 尝试从可执行文件位置查找（通过环境变量）
    if let Ok(x_root) = std::env::var("X_ROOT") {
        let path = PathBuf::from(x_root).join("library/stdlib");
        if path.exists() {
            return Ok(path);
        }
    }

    // 对于开发构建，从 Cargo 环境查找
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest_dir)
            .join("../../library/stdlib")
            .canonicalize()
            .map_err(|e| format!("无法定位标准库: {}", e))?;
        if path.exists() {
            return Ok(path);
        }
    }

    Err("无法找到标准库目录。请确保 X_ROOT 环境变量正确设置，或者在项目根目录中运行。".to_string())
}

/// CLI 编译流水线产物
pub struct PipelineOutput {
    pub ast: x_parser::ast::Program,
    pub hir: x_hir::Hir,
    pub mir: x_mir::MirModule,
    pub lir: x_lir::Program,
}

/// 模块解析器
pub struct ModuleResolver {
    /// 模块搜索路径
    pub search_paths: Vec<PathBuf>,
    /// 已解析的模块（模块名 -> 源代码）
    resolved: HashMap<String, String>,
    /// 模块导出符号（模块名 -> 导出符号集合）
    pub module_exports: HashMap<String, HashSet<String>>,
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
        if self.resolved.contains_key(module_name) {
            return Ok(Some(self.resolved.get(module_name).unwrap().clone()));
        }

        for search_path in &self.search_paths {
            let module_file = search_path.join(format!("{}.x", module_name.replace("::", "/")));
            if module_file.exists() {
                let source = std::fs::read_to_string(&module_file)
                    .map_err(|e| format!("无法读取模块文件 {:?}: {}", module_file, e))?;
                self.resolved.insert(module_name.to_string(), source.clone());
                return Ok(Some(source));
            }
        }
        Ok(None)
    }

    /// 注册模块导出符号
    pub fn register_exports(&mut self, module_name: &str, exports: HashSet<String>) {
        self.module_exports.insert(module_name.to_string(), exports);
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// 读取并合并整个标准库 prelude（包括 option.x 和 result.x）
pub fn read_std_prelude() -> Result<String, String> {
    let stdlib_dir = find_stdlib_path()?;

    // 读取 option.x
    let option_path = stdlib_dir.join("option.x");
    let mut source = std::fs::read_to_string(&option_path)
        .map_err(|e| format!("无法读取 option.x {:?}: {}", option_path, e))?;
    source.push_str("\n\n");

    // 读取 result.x
    let result_path = stdlib_dir.join("result.x");
    let result_source = std::fs::read_to_string(&result_path)
        .map_err(|e| format!("无法读取 result.x {:?}: {}", result_path, e))?;
    source.push_str(&result_source);
    source.push_str("\n\n");

    // 读取 prelude.x，但是跳过 export use 行（因为已经内联了 option 和 result）
    let prelude_path = stdlib_dir.join("prelude.x");
    let prelude_source = std::fs::read_to_string(&prelude_path)
        .map_err(|e| format!("无法读取 prelude.x {:?}: {}", prelude_path, e))?;

    // 跳过 export use 行，保留其他所有内容（包括注释）
    for line in prelude_source.lines() {
        if !line.trim_start().starts_with("export use ") {
            source.push_str(line);
            source.push_str("\n");
        }
    }

    Ok(source)
}

/// 解析标准库 prelude 并返回其声明
pub fn parse_std_prelude() -> Result<Vec<x_parser::ast::Declaration>, String> {
    let prelude_source = read_std_prelude()?;
    let parser = x_parser::parser::XParser::new();
    let prelude_program = parser
        .parse(&prelude_source)
        .map_err(|e| format!("无法解析标准库 prelude: {}", e))?;
    Ok(prelude_program.declarations)
}

/// 解析并处理所有 import 语句
pub fn resolve_imports(
    program: &mut x_parser::ast::Program,
    stdlib_dir: &Path,
) -> Result<(), String> {
    let mut imports_to_process: Vec<(usize, x_parser::ast::ImportDecl)> = Vec::new();

    // 收集所有 import 声明
    for (idx, decl) in program.declarations.iter().enumerate() {
        if let x_parser::ast::Declaration::Import(import_decl) = decl {
            imports_to_process.push((idx, import_decl.clone()));
        }
    }

    // 处理每个 import
    for (original_idx, import_decl) in imports_to_process {
        let module_path = &import_decl.module_path;

        // 解析模块源文件
        let module_source = resolve_import_module(module_path, stdlib_dir)?;

        // 解析模块
        let parser = x_parser::parser::XParser::new();
        let module_program = parser
            .parse(&module_source)
            .map_err(|e| format!("无法解析模块 {}: {}", module_path, e))?;

        // 收集模块的导出符号
        let exports = collect_module_exports(&module_program, module_path);

        // 根据 import 类型处理
        match &import_decl.symbols[..] {
            // import std.Option  -> 导入整个模块
            [] => {
                // 将模块的所有声明插入到当前程序中
                insert_module_declarations(program, original_idx, module_program);
            }
            // import std.Option.Some  -> 导入特定符号
            symbols => {
                for symbol in symbols {
                    match symbol {
                        x_parser::ast::ImportSymbol::All => {
                            // 导入所有 - 需要克隆因为 module_program 可能被多次使用
                            insert_module_declarations(program, original_idx, module_program.clone());
                        }
                        x_parser::ast::ImportSymbol::Named(name, alias) => {
                            // 导入特定符号 (name is String, alias is Option<String>)
                            insert_specific_symbol(program, original_idx, &module_program, name, alias.as_deref());
                        }
                    }
                }
            }
        }
    }

    // 移除 import 声明
    program.declarations.retain(|d| !matches!(d, x_parser::ast::Declaration::Import(_)));

    Ok(())
}

/// 解析导入的模块
fn resolve_import_module(module_path: &str, stdlib_dir: &Path) -> Result<String, String> {
    // 处理特殊路径
    if module_path.starts_with("std.") || module_path == "std" {
        // 标准库模块
        let std_path = stdlib_dir.join(format!("{}.x", module_path.replace("std.", "")));
        if std_path.exists() {
            return std::fs::read_to_string(&std_path)
                .map_err(|e| format!("无法读取标准库模块 {:?}: {}", std_path, e));
        }
    }

    // TODO: 处理其他模块路径
    Err(format!("无法解析模块: {}", module_path))
}

/// 收集模块的导出符号
fn collect_module_exports(program: &x_parser::ast::Program, module_path: &str) -> HashSet<String> {
    let mut exports = HashSet::new();

    for decl in &program.declarations {
        match decl {
            x_parser::ast::Declaration::TypeAlias(type_alias) => {
                exports.insert(type_alias.name.clone());
            }
            x_parser::ast::Declaration::Enum(enum_decl) => {
                exports.insert(enum_decl.name.clone());
                // 也添加变体
                for variant in &enum_decl.variants {
                    exports.insert(variant.name.clone());
                }
            }
            x_parser::ast::Declaration::Function(func_decl) => {
                exports.insert(func_decl.name.clone());
            }
            x_parser::ast::Declaration::Class(class_decl) => {
                exports.insert(class_decl.name.clone());
            }
            _ => {}
        }
    }

    exports
}

/// 将模块的所有声明插入到程序中
fn insert_module_declarations(
    program: &mut x_parser::ast::Program,
    import_idx: usize,
    module_program: x_parser::ast::Program,
) {
    // 在 import 位置插入模块声明
    for decl in module_program.declarations {
        program.declarations.insert(import_idx, decl);
    }
}

/// 插入特定的符号
fn insert_specific_symbol(
    program: &mut x_parser::ast::Program,
    import_idx: usize,
    module_program: &x_parser::ast::Program,
    name: &str,
    _alias: Option<&str>,
) {
    for decl in &module_program.declarations {
        let should_insert = match decl {
            x_parser::ast::Declaration::TypeAlias(ta) => ta.name == name,
            x_parser::ast::Declaration::Enum(e) => e.name == name,
            x_parser::ast::Declaration::Function(f) => f.name == name,
            x_parser::ast::Declaration::Class(c) => c.name == name,
            _ => false,
        };

        if should_insert {
            program.declarations.insert(import_idx, decl.clone());
            break;
        }
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
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("无法读取文件 {:?}: {}", path, e))?;

        self.compile_source(&source)
    }

    /// 编译源代码
    pub fn compile_source(&mut self, source: &str) -> Result<x_parser::ast::Program, String> {
        let parser = x_parser::parser::XParser::new();
        let mut program = parser
            .parse(source)
            .map_err(|e| format!("解析错误: {}", e))?;

        // 自动导入标准库 prelude
        let prelude_decls = parse_std_prelude()?;
        // 将 prelude 声明插入到用户程序最前面
        let mut new_decls = prelude_decls;
        new_decls.extend(program.declarations);
        program.declarations = new_decls;

        // 收集模块信息和导出
        for decl in &program.declarations {
            match decl {
                x_parser::ast::Declaration::Module(module_decl) => {
                    // 注册当前模块
                    self.resolver
                        .register_exports(&module_decl.name, HashSet::new());
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
            merged_program
                .declarations
                .extend(program.declarations.clone());
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

pub fn run_pipeline(source: &str) -> Result<PipelineOutput, String> {
    let parser = x_parser::parser::XParser::new();
    let mut ast = parser
        .parse(source)
        .map_err(|e| format!("解析错误: {}", e))?;

    // 自动导入标准库 prelude
    let prelude_decls = parse_std_prelude()?;
    // 将 prelude 声明插入到用户程序最前面
    let mut new_decls = prelude_decls;
    new_decls.extend(ast.declarations);
    ast.declarations = new_decls;

    type_check_with_big_stack(&ast)?;

    let hir = x_hir::ast_to_hir(&ast).map_err(|e| format!("HIR 转换错误: {}", e))?;
    let mir = x_mir::lower_hir_to_mir(&hir).map_err(|e| format!("MIR 转换错误: {}", e))?;
    let lir = x_lir::lower_mir_to_lir(&mir).map_err(|e| format!("LIR 转换错误: {}", e))?;

    Ok(PipelineOutput { ast, hir, mir, lir })
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
pub fn format_type_error(
    file: &str,
    source: &str,
    error: &x_typechecker::errors::TypeError,
) -> String {
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
