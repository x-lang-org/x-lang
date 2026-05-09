use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

fn first_existing_path(candidates: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    candidates.into_iter().find(|candidate| candidate.exists())
}

fn cwd_stdlib_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("../../library/stdlib"),
        PathBuf::from("../library/stdlib"),
        PathBuf::from("library/stdlib"),
        PathBuf::from("/library/stdlib"),
    ]
}

fn managed_stdlib_candidates() -> Vec<PathBuf> {
    vec![crate::config::managed_stdlib_root()]
}

fn executable_stdlib_candidates() -> Vec<PathBuf> {
    let Ok(exe_path) = std::env::current_exe() else {
        return Vec::new();
    };
    let Some(exe_dir) = exe_path.parent() else {
        return Vec::new();
    };

    exe_dir
        .ancestors()
        .take(6)
        .map(|ancestor| ancestor.join("library/stdlib"))
        .collect()
}

fn trusted_stdlib_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(x_root) = std::env::var("X_ROOT") {
        candidates.push(PathBuf::from(x_root).join("library/stdlib"));
    }

    candidates.extend(managed_stdlib_candidates());
    candidates.extend(executable_stdlib_candidates());

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        candidates.push(PathBuf::from(manifest_dir).join("../../library/stdlib"));
    }

    candidates
}

pub fn find_trusted_stdlib_source() -> Result<PathBuf, String> {
    if let Some(path) = first_existing_path(trusted_stdlib_candidates()) {
        return path
            .canonicalize()
            .map_err(|e| format!("无法规范化标准库路径 {}: {}", path.display(), e));
    }

    Err(format!(
        "无法找到可信标准库目录。请设置 X_ROOT，或确保 {} 存在，或从包含 library/stdlib 的已构建仓库环境运行 x。",
        crate::config::managed_stdlib_root().display()
    ))
}

/// 查找标准库文件路径
pub fn find_stdlib_path() -> Result<PathBuf, String> {
    // 显式环境变量优先于推断路径
    if let Ok(x_root) = std::env::var("X_ROOT") {
        let path = PathBuf::from(x_root).join("library/stdlib");
        if path.exists() {
            return Ok(path);
        }
    }

    // 已安装工具优先使用复制到 X_HOME 下的托管标准库
    if let Some(path) = first_existing_path(managed_stdlib_candidates()) {
        return Ok(path);
    }

    // 开发场景允许直接从当前工作目录发现仓库内置标准库
    if let Some(path) = first_existing_path(cwd_stdlib_candidates()) {
        return Ok(path);
    }

    // 尝试从已构建的 x 可执行文件位置推断仓库根目录
    if let Some(path) = first_existing_path(executable_stdlib_candidates()) {
        return Ok(path);
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

    Err(format!(
        "无法找到标准库目录。请设置 X_ROOT，或确保 {} 存在，或从包含 library/stdlib 的仓库相对位置运行 x。",
        crate::config::managed_stdlib_root().display()
    ))
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
            // 直接尝试读取文件，避免 TOCTOU 问题
            if let Ok(source) = std::fs::read_to_string(&module_file) {
                self.resolved
                    .insert(module_name.to_string(), source.clone());
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

/// 读取标准库 prelude（只读取 prelude.x）
pub fn read_std_prelude() -> Result<String, String> {
    let stdlib_dir = find_stdlib_path()?;

    // 只读取 prelude.x
    let prelude_path = stdlib_dir.join("prelude.x");
    let prelude_source = std::fs::read_to_string(&prelude_path)
        .map_err(|e| format!("无法读取 prelude.x {:?}: {}", prelude_path, e))?;

    Ok(prelude_source)
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

/// 将标准库 prelude 声明插入到程序最前面。
pub fn inject_std_prelude(program: &mut x_parser::ast::Program) -> Result<(), String> {
    let existing_keys: HashSet<_> = program
        .declarations
        .iter()
        .filter_map(declaration_identity_key)
        .collect();

    let mut prelude_decls: Vec<_> = parse_std_prelude()?
        .into_iter()
        .filter(|decl| match declaration_identity_key(decl) {
            Some(key) => !existing_keys.contains(&key),
            None => true,
        })
        .collect();

    prelude_decls.extend(program.declarations.clone());
    program.declarations = prelude_decls;
    Ok(())
}

fn declaration_identity_key(decl: &x_parser::ast::Declaration) -> Option<(&'static str, String)> {
    use x_parser::ast::Declaration;

    match decl {
        Declaration::Variable(var) => var
            .simple_name()
            .map(|name| ("variable", name.to_string())),
        Declaration::Function(func) => Some(("function", func.name.clone())),
        Declaration::ExternFunction(func) => Some(("extern_function", func.name.clone())),
        Declaration::Class(class) => Some(("class", class.name.clone())),
        Declaration::Trait(trait_decl) => Some(("trait", trait_decl.name.clone())),
        Declaration::Enum(enum_decl) => Some(("enum", enum_decl.name.clone())),
        Declaration::Record(record) => Some(("record", record.name.clone())),
        Declaration::Effect(effect) => Some(("effect", effect.name.clone())),
        Declaration::TypeAlias(alias) => Some(("type_alias", alias.name.clone())),
        Declaration::Newtype(newtype) => Some(("newtype", newtype.name.clone())),
        Declaration::Module(module) => Some(("module", module.name.clone())),
        Declaration::Import(import_decl) => Some(("import", import_decl.module_path.clone())),
        Declaration::Export(export_decl) => Some(("export", export_decl.symbol.clone())),
        Declaration::Implement(_) => None,
    }
}

fn should_insert_declaration(
    program: &x_parser::ast::Program,
    decl: &x_parser::ast::Declaration,
) -> bool {
    if matches!(decl, x_parser::ast::Declaration::Import(_)) {
        return true;
    }

    match declaration_identity_key(decl) {
        Some(key) => !program
            .declarations
            .iter()
            .filter_map(declaration_identity_key)
            .any(|existing_key| existing_key == key),
        None => true,
    }
}

/// Parse a source string, resolve imports relative to a project directory, then inject prelude.
pub fn prepare_program(
    source: &str,
    project_dir: &Path,
) -> Result<x_parser::ast::Program, String> {
    let parser = x_parser::parser::XParser::new();
    let mut program = parser
        .parse(source)
        .map_err(|e| format!("解析错误: {}", e))?;

    let stdlib_dir = find_stdlib_path()?;
    resolve_imports(&mut program, &stdlib_dir, project_dir)?;
    inject_std_prelude(&mut program)?;

    Ok(program)
}

/// 解析并处理所有 import 语句
pub fn resolve_imports(
    program: &mut x_parser::ast::Program,
    stdlib_dir: &Path,
    project_dir: &Path,
) -> Result<(), String> {
    loop {
        let imports_to_process: Vec<(usize, x_parser::ast::ImportDecl)> = program
            .declarations
            .iter()
            .enumerate()
            .filter_map(|(idx, decl)| match decl {
                x_parser::ast::Declaration::Import(import_decl) => Some((idx, import_decl.clone())),
                _ => None,
            })
            .collect();

        if imports_to_process.is_empty() {
            break;
        }

        let processed_imports: Vec<x_parser::ast::ImportDecl> = imports_to_process
            .iter()
            .map(|(_, import_decl)| import_decl.clone())
            .collect();

        for (original_idx, import_decl) in imports_to_process {
            let module_path = &import_decl.module_path;

            // 解析模块源文件
            let module_source = resolve_import_module(module_path, stdlib_dir, project_dir)?;

            // 解析模块
            let parser = x_parser::parser::XParser::new();
            let module_program = parser
                .parse(&module_source)
                .map_err(|e| format!("无法解析模块 {}: {}", module_path, e))?;

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
                                insert_module_declarations(
                                    program,
                                    original_idx,
                                    module_program.clone(),
                                );
                            }
                            x_parser::ast::ImportSymbol::Named(name, alias) => {
                                // 导入特定符号 (name is String, alias is Option<String>)
                                insert_specific_symbol(
                                    program,
                                    original_idx,
                                    &module_program,
                                    name,
                                    alias.as_deref(),
                                );
                            }
                        }
                    }
                }
            }
        }

        // 移除本轮 import 声明，让新插入模块中的 import 在下一轮继续解析
        program.declarations.retain(|decl| match decl {
            x_parser::ast::Declaration::Import(import_decl) => !processed_imports
                .iter()
                .any(|processed| processed == import_decl),
            _ => true,
        });
    }

    Ok(())
}

/// 解析导入的模块
fn resolve_import_module(
    module_path: &str,
    stdlib_dir: &Path,
    project_dir: &Path,
) -> Result<String, String> {
    // 处理特殊路径 - 支持 std., std::, std_ 前缀
    let path_lower = module_path.to_lowercase();
    if path_lower.starts_with("std.")
        || path_lower.starts_with("std::")
        || path_lower == "std"
        || path_lower.starts_with("std_")
    {
        // 标准库模块 (支持 std.types, std::types, std_types, std 等形式)
        let module_name = module_path
            .trim_start_matches("std.")
            .trim_start_matches("std::")
            .trim_start_matches("std")
            .trim_start_matches("STD.")
            .trim_start_matches("STD::")
            .trim_start_matches("STD");
        let module_name = if let Some(stripped) = module_name.strip_prefix('_') {
            stripped
        } else if module_name.is_empty() {
            "prelude"
        } else {
            module_name
        };
        let std_path = stdlib_dir.join(format!("{}.x", module_name));
        // 直接尝试读取文件，避免 TOCTOU 问题
        if let Ok(source) = std::fs::read_to_string(&std_path) {
            return Ok(source);
        }
        // 尝试目录形式 std/io.x
        let std_path_dir = stdlib_dir
            .join(module_name.replace('.', "/"))
            .with_extension("x");
        if let Ok(source) = std::fs::read_to_string(&std_path_dir) {
            return Ok(source);
        }
    }

    // 尝试作为项目内模块解析（支持 foo.bar -> foo/bar.x）
    let module_file = project_dir
        .join(module_path.replace('.', "/"))
        .with_extension("x");
    if let Ok(source) = std::fs::read_to_string(&module_file) {
        return Ok(source);
    }

    // 尝试作为目录模块解析（foo -> foo/index.x 或 foo.x）
    let dir_module = project_dir
        .join(module_path.replace('.', "/"))
        .join("index.x");
    if let Ok(source) = std::fs::read_to_string(&dir_module) {
        return Ok(source);
    }

    Err(format!(
        "无法解析模块: {} (在 {:?} 和 {:?} 中未找到)",
        module_path, project_dir, stdlib_dir
    ))
}

/// 将模块的所有声明插入到程序中
fn insert_module_declarations(
    program: &mut x_parser::ast::Program,
    import_idx: usize,
    module_program: x_parser::ast::Program,
) {
    // 在 import 位置插入模块声明
    for decl in module_program.declarations.into_iter().rev() {
        if should_insert_declaration(program, &decl) {
            program.declarations.insert(import_idx, decl);
        }
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
            if should_insert_declaration(program, decl) {
                program.declarations.insert(import_idx, decl.clone());
            }
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
        let project_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let program = prepare_program(source, &project_dir)?;

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
    let project_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    run_pipeline_with_project_dir(source, &project_dir)
}

pub fn run_pipeline_with_project_dir(
    source: &str,
    project_dir: &Path,
) -> Result<PipelineOutput, String> {
    let ast = prepare_program(source, project_dir)?;

    // 类型检查并获取类型环境，用于 HIR 整合类型注解
    // 使用 type_check_with_env 以获取类型环境
    let type_env =
        x_typechecker::type_check_with_env(&ast).map_err(|e| format!("类型检查错误: {}", e))?;

    let hir = x_hir::ast_to_hir_with_type_env(&ast, &type_env)
        .map_err(|e| format!("HIR 转换错误: {}", e))?;
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

    #[test]
    fn test_lexer_tokenization() {
        let source = "let x = 42;";
        let lexer = x_lexer::new_lexer(source);
        let tokens: Vec<_> = lexer.map(|t| t.unwrap().0).collect();
        // Verify key tokens are present
        assert!(tokens
            .iter()
            .any(|t| matches!(t, x_lexer::token::Token::Let)));
        assert!(tokens
            .iter()
            .any(|t| matches!(t, x_lexer::token::Token::Ident(_))));
    }

    #[test]
    fn test_parser_basic_program() {
        let source = "let x = 42;";
        let parser = x_parser::parser::XParser::new();
        let result = parser.parse(source);
        assert!(result.is_ok(), "Basic program should parse");
    }

    #[test]
    fn test_typechecker_basic_types() {
        let source = "let x: integer = 42;";
        let parser = x_parser::parser::XParser::new();
        let ast = parser.parse(source).expect("Should parse");
        // Type check should succeed
        let result = x_typechecker::type_check(&ast);
        assert!(
            result.is_ok(),
            "Type checking should pass for valid program"
        );
    }

    #[test]
    fn test_interpreter_hello_world() {
        let source = r#"println("Hello, World!")"#;
        let parser = x_parser::parser::XParser::new();
        let mut ast = parser.parse(source).expect("Should parse");
        inject_std_prelude(&mut ast).expect("Should inject prelude");
        let result = x_typechecker::type_check(&ast);
        assert!(result.is_ok(), "Type checking should pass");
    }

    #[test]
    fn test_prepare_program_resolves_nested_imports_recursively() {
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("integration")
            .join("modules")
            .join("nested_imports");
        let source = std::fs::read_to_string(project_dir.join("main.x"))
            .expect("Should read nested import main fixture");

        let ast = prepare_program(&source, &project_dir).expect("Should resolve nested imports");

        assert!(ast.declarations.iter().any(|decl| matches!(
            decl,
            x_parser::ast::Declaration::Variable(var) if var.simple_name() == Some("base_value")
        )));
        assert!(ast.declarations.iter().any(|decl| matches!(
            decl,
            x_parser::ast::Declaration::Function(func) if func.name == "add_base"
        )));
        assert!(
            !ast.declarations.iter().any(|decl| matches!(
                decl,
                x_parser::ast::Declaration::Import(import_decl)
                    if import_decl.module_path == "middle" || import_decl.module_path == "base"
            )),
            "nested project imports should be resolved before type checking"
        );

        let result = x_typechecker::type_check(&ast);
        assert!(result.is_ok(), "Type checking should pass after nested imports are resolved");
    }

    #[test]
    fn test_prepare_program_dedupes_explicit_prelude_import_before_injection() {
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("library")
            .join("stdlib");
        let source = "import std.prelude\nprintln(\"hello\")";

        let ast = prepare_program(source, &project_dir).expect("Should prepare program");

        let puts_count = ast
            .declarations
            .iter()
            .filter(|decl| matches!(
                decl,
                x_parser::ast::Declaration::ExternFunction(func) if func.name == "puts"
            ))
            .count();

        assert_eq!(puts_count, 1, "prelude externs should only be injected once");
        assert!(x_typechecker::type_check(&ast).is_ok(), "Type checking should pass");
    }

    #[test]
    fn test_prepare_program_dedupes_transitive_std_imports() {
        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("library")
            .join("stdlib");
        let source = std::fs::read_to_string(project_dir.join("panic.x"))
            .expect("Should read std panic module fixture");

        let ast = prepare_program(&source, &project_dir).expect("Should prepare std panic module");

        let puts_count = ast
            .declarations
            .iter()
            .filter(|decl| matches!(
                decl,
                x_parser::ast::Declaration::ExternFunction(func) if func.name == "puts"
            ))
            .count();

        assert_eq!(puts_count, 1, "transitive std imports should not duplicate prelude externs");
    }
}
