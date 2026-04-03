//! Zig 后端 - 将 X AST 编译为 Zig 0.15 代码
//!
//! 利用 Zig 的内存管理和错误处理特性，提供高效的编译输出
//!
//! ## Zig 0.15 特性支持 (2025年10月发布)
//! - Improved AstGen/Zon syntax
//! - 命名空间隔离改进
//! - 改进的错误处理
//! - @import 语义更新
//! - 自定义增量编译
//! - Wasm 改进（wasm32-wasi, wasm32-freestanding）
//! - Improved C interoperability
//! - Better incremental compilation

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use x_codegen::headers;
use x_hir;
use x_lir;
use x_mir;
use x_parser::ast::{self, ExpressionKind, Program as AstProgram, StatementKind};

/// 编译目标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZigTarget {
    #[default]
    Native,
    Wasm32Wasi,
    Wasm32Freestanding,
}

impl ZigTarget {
    pub fn as_zig_target(&self) -> &'static str {
        match self {
            ZigTarget::Native => "native",
            ZigTarget::Wasm32Wasi => "wasm32-wasi",
            ZigTarget::Wasm32Freestanding => "wasm32-freestanding",
        }
    }

    pub fn output_extension(&self) -> &'static str {
        match self {
            ZigTarget::Native => "", // Platform-specific
            ZigTarget::Wasm32Wasi | ZigTarget::Wasm32Freestanding => ".wasm",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZigBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub target: ZigTarget,
}

impl Default for ZigBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            target: ZigTarget::Native,
        }
    }
}

pub struct ZigBackend {
    config: ZigBackendConfig,
    /// 代码缓冲区（统一管理输出和缩进）
    buffer: x_codegen::CodeBuffer,
    /// Track global (top-level) variable declarations for forward decls
    global_vars: Vec<String>,
    /// Track imported Zig modules
    imported_modules: Vec<String>,
    /// Lambda counter for unique naming
    lambda_counter: usize,
    /// Track variable types for dictionary type inference
    var_types: std::collections::HashMap<String, String>,
    /// Track if we need to link libc (for extern "c" functions)
    needs_libc: bool,
}

pub type ZigResult<T> = Result<T, x_codegen::CodeGenError>;

impl ZigBackend {
    // 常量定义
    const MATCH_CASE_NO_EXPRESSION_ERROR: &'static str =
        "Match case body must contain an expression";
    const MATCH_CASE_MULTIPLE_EXPRESSIONS_ERROR: &'static str =
        "Match case body can only contain one expression";

    /// 从 block 中提取第一个表达式语句
    /// 用于 match 表达式和函数返回值
    fn extract_first_expression_from_block(&mut self, block: &ast::Block) -> ZigResult<String> {
        let mut expression_count = 0;
        let mut result = None;

        for stmt in &block.statements {
            if let StatementKind::Expression(expr) = &stmt.node {
                expression_count += 1;
                if expression_count == 1 {
                    result = Some(self.emit_expr(expr)?);
                }
                // 不立即 break，为了计数并检查是否有多个表达式
            }
        }

        match (result, expression_count) {
            (Some(expr_str), 1) => Ok(expr_str),
            (None, 0) => Err(x_codegen::CodeGenError::CompilerError(
                Self::MATCH_CASE_NO_EXPRESSION_ERROR.to_string(),
            )),
            (Some(_), count) if count > 1 => Err(x_codegen::CodeGenError::CompilerError(
                Self::MATCH_CASE_MULTIPLE_EXPRESSIONS_ERROR.to_string(),
            )),
            _ => unreachable!(),
        }
    }

    pub fn new(config: ZigBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
            global_vars: Vec::new(),
            imported_modules: Vec::new(),
            lambda_counter: 0,
            var_types: std::collections::HashMap::new(),
            needs_libc: false,
        }
    }

    pub fn generate_from_ast(&mut self, program: &AstProgram) -> ZigResult<x_codegen::CodegenOutput> {
        self.buffer.clear();
        self.needs_libc = false;
        self.global_vars.clear();
        self.imported_modules.clear();
        self.lambda_counter = 0;
        self.var_types.clear();

        self.emit_header()?;

        // Emit imports
        for decl in &program.declarations {
            if let ast::Declaration::Import(import) = decl {
                self.emit_import(import)?;
            }
        }

        // Emit classes as structs
        for decl in &program.declarations {
            if let ast::Declaration::Class(class) = decl {
                self.emit_class(class)?;
            }
        }

        // Emit global variables
        for decl in &program.declarations {
            if let ast::Declaration::Variable(v) = decl {
                self.emit_global_var(v)?;
            }
        }

        // Emit extern functions
        for decl in &program.declarations {
            if let ast::Declaration::ExternFunction(f) = decl {
                self.emit_extern_function(f)?;
            }
        }

        // Emit functions (including class methods and standalone functions)
        let mut has_main = false;
        for decl in &program.declarations {
            if let ast::Declaration::Function(f) = decl {
                self.emit_function(f)?;
                self.line("")?;
                if f.name == "main" {
                    has_main = true;
                }
            }
        }

        // Emit class methods
        for decl in &program.declarations {
            if let ast::Declaration::Class(class) = decl {
                for member in &class.members {
                    if let ast::ClassMember::Method(method) = member {
                        self.emit_class_method(&class.name, method)?;
                        self.line("")?;
                    }
                    if let ast::ClassMember::Constructor(constructor) = member {
                        self.emit_constructor(&class.name, constructor)?;
                        self.line("")?;
                    }
                }
            }
        }

        // Emit main function only if not already defined
        if !has_main {
            self.emit_main_function(&program.statements)?;
        }

        // Create output file
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output().as_bytes().to_vec(),
            file_type: x_codegen::FileType::Zig,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 从 HIR 生成代码
    pub fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> ZigResult<x_codegen::CodegenOutput> {
        self.buffer.clear();
        self.global_vars.clear();
        self.imported_modules.clear();
        self.var_types.clear();

        self.emit_header()?;

        // Emit functions from HIR
        for decl in &hir.declarations {
            if let x_hir::HirDeclaration::Function(func) = decl {
                self.emit_hir_function(func)?;
                self.line("")?;
            }
        }

        // Create output file
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output().as_bytes().to_vec(),
            file_type: x_codegen::FileType::Zig,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 从 PerceusIR 生成代码（带内存管理）
    pub fn generate_from_pir(&mut self, pir: &x_mir::PerceusIR) -> ZigResult<x_codegen::CodegenOutput> {
        self.buffer.clear();
        self.global_vars.clear();
        self.imported_modules.clear();
        self.var_types.clear();

        self.emit_header()?;

        // Emit functions with memory operations from Perceus analysis
        for func_analysis in &pir.functions {
            self.emit_pir_function(func_analysis)?;
            self.line("")?;
        }

        // Create output file
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output().as_bytes().to_vec(),
            file_type: x_codegen::FileType::Zig,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 从 LIR 生成代码（低层中间表示 - 后端统一正式输入）
    pub fn generate_from_lir(&mut self, lir: &x_lir::Program) -> ZigResult<x_codegen::CodegenOutput> {
        self.buffer.clear();
        self.global_vars.clear();
        self.imported_modules.clear();
        self.var_types.clear();

        self.emit_header()?;

        // Emit extern functions first
        for decl in &lir.declarations {
            if let x_lir::Declaration::ExternFunction(extern_func) = decl {
                self.emit_lir_extern_function(extern_func)?;
            }
        }

        // Emit global variables
        for decl in &lir.declarations {
            if let x_lir::Declaration::Global(global_var) = decl {
                self.emit_lir_global_var(global_var)?;
            }
        }

        // Emit struct definitions
        for decl in &lir.declarations {
            if let x_lir::Declaration::Struct(struct_def) = decl {
                self.emit_lir_struct(struct_def)?;
            }
        }

        // Emit enum definitions
        for decl in &lir.declarations {
            if let x_lir::Declaration::Enum(enum_def) = decl {
                self.emit_lir_enum(enum_def)?;
            }
        }

        // Emit functions
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(func) = decl {
                self.emit_lir_function(func)?;
                self.line("")?;
            }
        }

        // Create output file
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output().as_bytes().to_vec(),
            file_type: x_codegen::FileType::Zig,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// Emit a function from HIR
    fn emit_hir_function(&mut self, func: &x_hir::HirFunctionDecl) -> ZigResult<()> {
        let params = if func.parameters.is_empty() {
            "".to_string()
        } else {
            func.parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.emit_hir_type(&p.ty)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let return_type = self.emit_hir_type(&func.return_type);
        self.line(&format!("fn {}({}) {} {{", func.name, params, return_type))?;
        self.indent();

        // Emit function body
        self.emit_hir_block(&func.body)?;

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit a function from PerceusIR with memory management
    fn emit_pir_function(&mut self, func: &x_mir::FunctionAnalysis) -> ZigResult<()> {
        let params = if func.param_ownership.is_empty() {
            "".to_string()
        } else {
            func.param_ownership
                .iter()
                .map(|p| format!("{}: {}", p.variable, self.ownership_type_to_zig(&p.ty)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let return_type = self.ownership_type_to_zig(&func.return_ownership.ty);
        self.line(&format!("fn {}({}) {} {{", func.name, params, return_type))?;
        self.indent();

        // Emit memory operations
        for mem_op in &func.memory_ops {
            self.emit_memory_op(mem_op)?;
        }

        // Emit control flow
        for block in &func.control_flow.blocks {
            self.emit_basic_block(block)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit a memory operation
    fn emit_memory_op(&mut self, op: &x_mir::MemoryOp) -> ZigResult<()> {
        match op {
            x_mir::MemoryOp::Dup {
                variable, target, ..
            } => {
                // In Zig, dup is typically allocator.dupe()
                self.line(&format!(
                    "var {} = try allocator.dupe(u8, {});",
                    target, variable
                ))?;
            }
            x_mir::MemoryOp::Drop { variable, .. } => {
                // In Zig, we use defer for cleanup
                self.line(&format!("defer allocator.free({});", variable))?;
            }
            x_mir::MemoryOp::Reuse { from, to, .. } => {
                // Memory reuse - reusing memory from one variable to another
                self.line(&format!("var {} = {}; // reuse", to, from))?;
            }
            x_mir::MemoryOp::Alloc { variable, size, .. } => {
                self.line(&format!(
                    "var {} = try allocator.alloc(u8, {});",
                    variable, size
                ))?;
            }
        }
        Ok(())
    }

    /// Emit a basic block from control flow analysis
    fn emit_basic_block(&mut self, block: &x_mir::BasicBlock) -> ZigResult<()> {
        self.line(&format!(
            "// Block {} (statements: {:?})",
            block.id, block.statements
        ))?;

        // In a full implementation, we would emit the actual statements here
        // For now, we emit placeholder comments showing entry/exit states
        self.line(&format!("// Entry state: {:?}", block.entry_state))?;
        self.line(&format!("// Exit state: {:?}", block.exit_state))?;

        Ok(())
    }

    /// Convert HIR type to Zig type string
    fn emit_hir_type(&self, ty: &x_hir::HirType) -> String {
        match ty {
            x_hir::HirType::Int => "i32".to_string(),
            x_hir::HirType::UnsignedInt => "u32".to_string(),
            x_hir::HirType::Float => "f64".to_string(),
            x_hir::HirType::Bool => "bool".to_string(),
            x_hir::HirType::String => "[]const u8".to_string(),
            x_hir::HirType::Char => "u8".to_string(),
            x_hir::HirType::Unit => "void".to_string(),
            x_hir::HirType::Never => "noreturn".to_string(),
            x_hir::HirType::Dynamic => "anytype".to_string(),
            x_hir::HirType::Array(inner) => format!("[]{}", self.emit_hir_type(inner)),
            x_hir::HirType::Option(inner) => format!("?{}", self.emit_hir_type(inner)),
            x_hir::HirType::Result(ok, err) => {
                format!("{}!{}", self.emit_hir_type(err), self.emit_hir_type(ok))
            }
            x_hir::HirType::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.emit_hir_type(t)).collect();
                format!("struct {{ {} }}", type_strs.join(", "))
            }
            x_hir::HirType::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, self.emit_hir_type(t)))
                    .collect();
                format!("struct {} {{ {} }}", name, field_strs.join(", "))
            }
            x_hir::HirType::Generic(name) | x_hir::HirType::TypeParam(name) => name.clone(),
            x_hir::HirType::TypeConstructor(name, args) => {
                let args_str: Vec<String> = args.iter().map(|t| self.emit_hir_type(t)).collect();
                format!("{}({})", name, args_str.join(", "))
            }
            x_hir::HirType::Reference(inner) => format!("&{}", self.emit_hir_type(inner)),
            x_hir::HirType::MutableReference(inner) => format!("*mut {}", self.emit_hir_type(inner)),
            // FFI types
            x_hir::HirType::Pointer(inner) => format!("*{}", self.emit_hir_type(inner)),
            x_hir::HirType::ConstPointer(inner) => format!("*const {}", self.emit_hir_type(inner)),
            x_hir::HirType::Void => "void".to_string(),
            // C FFI types
            x_hir::HirType::CInt => "c_int".to_string(),
            x_hir::HirType::CUInt => "c_uint".to_string(),
            x_hir::HirType::CLong => "c_long".to_string(),
            x_hir::HirType::CULong => "c_ulong".to_string(),
            x_hir::HirType::CLongLong => "c_longlong".to_string(),
            x_hir::HirType::CULongLong => "c_ulonglong".to_string(),
            x_hir::HirType::CFloat => "c_float".to_string(),
            x_hir::HirType::CDouble => "c_double".to_string(),
            x_hir::HirType::CChar => "c_char".to_string(),
            x_hir::HirType::CSize => "usize".to_string(),
            x_hir::HirType::CString => "[*c]u8".to_string(),
            // Other types
            x_hir::HirType::Dictionary(k, v) => {
                format!(
                    "std.AutoHashMap({}, {})",
                    self.emit_hir_type(k),
                    self.emit_hir_type(v)
                )
            }
            x_hir::HirType::Union(name, _) => name.clone(),
            x_hir::HirType::Function(params, ret) => {
                let param_types: Vec<String> =
                    params.iter().map(|t| self.emit_hir_type(t)).collect();
                format!(
                    "fn({}) -> {}",
                    param_types.join(", "),
                    self.emit_hir_type(ret)
                )
            }
            x_hir::HirType::Async(inner) => self.emit_hir_type(inner),
            x_hir::HirType::Unknown => "anytype".to_string(),
        }
    }

    /// Convert ownership type string to Zig type
    fn ownership_type_to_zig(&self, ty: &str) -> String {
        match ty {
            "Int" => "i32".to_string(),
            "Float" => "f64".to_string(),
            "Bool" => "bool".to_string(),
            "String" => "[]const u8".to_string(),
            "Char" => "u8".to_string(),
            "Unit" => "void".to_string(),
            _ if ty.starts_with("Array<") => {
                let inner = ty.trim_start_matches("Array<").trim_end_matches('>');
                format!("[]{}", self.ownership_type_to_zig(inner))
            }
            _ if ty.starts_with("Option<") => {
                let inner = ty.trim_start_matches("Option<").trim_end_matches('>');
                format!("?{}", self.ownership_type_to_zig(inner))
            }
            _ if ty.starts_with("Result<") => {
                let content = ty.trim_start_matches("Result<").trim_end_matches('>');
                let parts: Vec<&str> = content.split(", ").collect();
                if parts.len() == 2 {
                    format!(
                        "{}!{}",
                        self.ownership_type_to_zig(parts[1]),
                        self.ownership_type_to_zig(parts[0])
                    )
                } else {
                    "anytype".to_string()
                }
            }
            _ => "anytype".to_string(),
        }
    }

    /// Emit HIR block
    fn emit_hir_block(&mut self, block: &x_hir::HirBlock) -> ZigResult<()> {
        for stmt in &block.statements {
            self.emit_hir_statement(stmt)?;
        }
        Ok(())
    }

    /// Emit HIR statement
    fn emit_hir_statement(&mut self, stmt: &x_hir::HirStatement) -> ZigResult<()> {
        match stmt {
            x_hir::HirStatement::Variable(var_decl) => {
                let init = if let Some(init) = &var_decl.initializer {
                    self.emit_hir_expression(init)?
                } else {
                    "undefined".to_string()
                };
                let var_type = self.emit_hir_type(&var_decl.ty);
                self.line(&format!("var {}: {} = {};", var_decl.name, var_type, init))?;
            }
            x_hir::HirStatement::Expression(expr) => {
                let e = self.emit_hir_expression(expr)?;
                self.line(&format!("{};", e))?;
            }
            x_hir::HirStatement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let e = self.emit_hir_expression(expr)?;
                    self.line(&format!("return {};", e))?;
                } else {
                    self.line("return;")?;
                }
            }
            x_hir::HirStatement::If(if_stmt) => {
                let cond = self.emit_hir_expression(&if_stmt.condition)?;
                self.line(&format!("if ({}) {{", cond))?;
                self.indent();
                self.emit_hir_block(&if_stmt.then_block)?;
                self.dedent();
                if let Some(else_block) = &if_stmt.else_block {
                    self.line("} else {")?;
                    self.indent();
                    self.emit_hir_block(else_block)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            x_hir::HirStatement::While(while_stmt) => {
                let cond = self.emit_hir_expression(&while_stmt.condition)?;
                self.line(&format!("while ({}) {{", cond))?;
                self.indent();
                self.emit_hir_block(&while_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_hir::HirStatement::For(for_stmt) => {
                let iterator = self.emit_hir_expression(&for_stmt.iterator)?;
                let pattern_name = match &for_stmt.pattern {
                    x_hir::HirPattern::Variable(name) => name.clone(),
                    x_hir::HirPattern::Wildcard => "_".to_string(),
                    _ => "_item".to_string(),
                };
                self.line(&format!("for ({}) |{}| {{", iterator, pattern_name))?;
                self.indent();
                self.emit_hir_block(&for_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_hir::HirStatement::Break => self.line("break;")?,
            x_hir::HirStatement::Continue => self.line("continue;")?,
            x_hir::HirStatement::Match(match_stmt) => {
                self.emit_hir_match(match_stmt)?;
            }
            x_hir::HirStatement::Try(try_stmt) => {
                self.emit_hir_try(try_stmt)?;
            }
            x_hir::HirStatement::Unsafe(block) => {
                // Zig doesn't need special unsafe syntax, emit block with comment
                self.line("// unsafe block")?;
                self.line("{")?;
                self.indent();
                self.emit_hir_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            x_hir::HirStatement::Defer(expr) => {
                let e = self.emit_hir_expression(expr)?;
                self.line(&format!("defer {};", e))?;
            }
            x_hir::HirStatement::Yield(_) => {
                // Generators not supported in Zig output, stub
                self.line("// yield (generator not supported)")?;
            }
            x_hir::HirStatement::Loop(body) => {
                self.line("while (true) {")?;
                self.indent();
                self.emit_hir_block(body)?;
                self.dedent();
                self.line("}")?;
            }
        }
        Ok(())
    }

    /// Emit HIR match statement
    fn emit_hir_match(&mut self, match_stmt: &x_hir::HirMatchStatement) -> ZigResult<()> {
        let expr = self.emit_hir_expression(&match_stmt.expression)?;
        self.line(&format!("switch ({}) {{", expr))?;
        self.indent();

        for case in &match_stmt.cases {
            let pattern_str = self.emit_hir_pattern(&case.pattern)?;
            if let Some(guard) = &case.guard {
                let guard_str = self.emit_hir_expression(guard)?;
                self.line(&format!("{} if ({}) => {{", pattern_str, guard_str))?;
            } else {
                self.line(&format!("{} => {{", pattern_str))?;
            }
            self.indent();
            self.emit_hir_block(&case.body)?;
            self.dedent();
            self.line("},")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit HIR try statement
    fn emit_hir_try(&mut self, try_stmt: &x_hir::HirTryStatement) -> ZigResult<()> {
        self.line("{")?;
        self.indent();
        self.line("var __err: ?anyerror = null;")?;
        self.line("errdefer {")?;
        self.indent();
        self.line("__err = error.UnexpectedError;")?;
        self.dedent();
        self.line("}")?;
        self.emit_hir_block(&try_stmt.body)?;

        if !try_stmt.catch_clauses.is_empty() {
            self.line("if (__err) |err| {")?;
            self.indent();

            for catch in &try_stmt.catch_clauses {
                if let Some(var_name) = &catch.variable_name {
                    self.line(&format!("const {} = err;", var_name))?;
                }
                self.emit_hir_block(&catch.body)?;
            }

            self.dedent();
            self.line("}")?;
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.emit_hir_block(finally)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit HIR pattern for match statement
    fn emit_hir_pattern(&self, pattern: &x_hir::HirPattern) -> ZigResult<String> {
        match pattern {
            x_hir::HirPattern::Wildcard => Ok("_".to_string()),
            x_hir::HirPattern::Variable(name) => Ok(name.clone()),
            x_hir::HirPattern::Literal(lit) => self.emit_hir_literal(lit),
            x_hir::HirPattern::Array(elements) => {
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|p| self.emit_hir_pattern(p))
                    .collect::<ZigResult<Vec<_>>>()?;
                Ok(format!("[{}]", elem_strs.join(", ")))
            }
            x_hir::HirPattern::Tuple(elements) => {
                let elem_strs: Vec<String> = elements
                    .iter()
                    .map(|p| self.emit_hir_pattern(p))
                    .collect::<ZigResult<Vec<_>>>()?;
                Ok(format!(".{{ {} }}", elem_strs.join(", ")))
            }
            x_hir::HirPattern::Or(left, right) => {
                let left_str = self.emit_hir_pattern(left)?;
                let right_str = self.emit_hir_pattern(right)?;
                Ok(format!("{}, {}", left_str, right_str))
            }
            x_hir::HirPattern::Dictionary(entries) => {
                // Zig doesn't have dictionary patterns, use placeholder
                Ok("_".to_string())
            }
            x_hir::HirPattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| {
                        let v_str = self.emit_hir_pattern(v)?;
                        Ok(format!(".{} = {}", k, v_str))
                    })
                    .collect::<ZigResult<Vec<_>>>()?;
                Ok(format!("{}.{{ {} }}", name, field_strs.join(", ")))
            }
            x_hir::HirPattern::EnumConstructor(type_name, variant_name, patterns) => {
                // Zig enum pattern: .VariantName => or .VariantName(patterns) =>
                if patterns.is_empty() {
                    Ok(format!(".{}", variant_name))
                } else {
                    let pattern_strs: Vec<String> = patterns
                        .iter()
                        .map(|p| self.emit_hir_pattern(p))
                        .collect::<ZigResult<Vec<_>>>()?;
                    Ok(format!(".{}({})", variant_name, pattern_strs.join(", ")))
                }
            }
        }
    }

    /// Emit HIR literal for pattern matching
    fn emit_hir_literal(&self, lit: &x_hir::HirLiteral) -> ZigResult<String> {
        match lit {
            x_hir::HirLiteral::Integer(n) => Ok(format!("{}", n)),
            x_hir::HirLiteral::Float(f) => Ok(format!("{}", f)),
            x_hir::HirLiteral::Boolean(b) => Ok(format!("{}", b)),
            x_hir::HirLiteral::String(s) => Ok(format!("\"{}\"", s)),
            x_hir::HirLiteral::Char(c) => Ok(format!("'{}'", c)),
            x_hir::HirLiteral::Unit => Ok("void".to_string()),
            x_hir::HirLiteral::None => Ok("null".to_string()),
        }
    }

    /// Emit HIR expression
    fn emit_hir_expression(&self, expr: &x_hir::HirExpression) -> ZigResult<String> {
        match expr {
            x_hir::HirExpression::Literal(lit) => match lit {
                x_hir::HirLiteral::Integer(n) => Ok(format!("{}", n)),
                x_hir::HirLiteral::Float(f) => Ok(format!("{}", f)),
                x_hir::HirLiteral::Boolean(b) => Ok(format!("{}", b)),
                x_hir::HirLiteral::String(s) => {
                    let escaped = s
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t");
                    Ok(format!("\"{}\"", escaped))
                }
                x_hir::HirLiteral::Char(c) => Ok(format!("'{}'", c)),
                x_hir::HirLiteral::Unit => Ok("void".to_string()),
                x_hir::HirLiteral::None => Ok("null".to_string()),
            },
            x_hir::HirExpression::Variable(name) => Ok(name.clone()),
            x_hir::HirExpression::Binary(op, lhs, rhs) => {
                let l = self.emit_hir_expression(lhs)?;
                let r = self.emit_hir_expression(rhs)?;
                Ok(self.emit_hir_binop(op, &l, &r))
            }
            x_hir::HirExpression::Unary(op, expr) => {
                let e = self.emit_hir_expression(expr)?;
                Ok(self.emit_hir_unaryop(op, &e))
            }
            x_hir::HirExpression::Call(callee, args) => {
                let callee_str = self.emit_hir_expression(callee)?;
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_hir_expression(a))
                    .collect::<ZigResult<Vec<_>>>()?;
                Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
            }
            x_hir::HirExpression::Array(items) => {
                let item_strs: Vec<String> = items
                    .iter()
                    .map(|i| self.emit_hir_expression(i))
                    .collect::<ZigResult<Vec<_>>>()?;
                Ok(format!("[_]anytype{{{}}}", item_strs.join(", ")))
            }
            x_hir::HirExpression::Member(obj, field) => {
                let o = self.emit_hir_expression(obj)?;
                Ok(format!("{}.{}", o, field))
            }
            x_hir::HirExpression::Handle(inner, handlers) => {
                let inner_expr = self.emit_hir_expression(inner)?;
                let mut handler_code = String::new();
                for (effect_name, handler) in handlers {
                    let handler_expr = self.emit_hir_expression(handler)?;
                    handler_code.push_str(&format!(
                        "    // handler for {}: {}\n",
                        effect_name, handler_expr
                    ));
                }
                Ok(format!(
                    "// handle {} with {{\n{}}}",
                    inner_expr, handler_code
                ))
            }
            _ => Ok("/* unimplemented HIR expr */".to_string()),
        }
    }

    /// Emit HIR binary operator
    fn emit_hir_binop(&self, op: &x_hir::HirBinaryOp, l: &str, r: &str) -> String {
        match op {
            x_hir::HirBinaryOp::Add => format!("{} + {}", l, r),
            x_hir::HirBinaryOp::Sub => format!("{} - {}", l, r),
            x_hir::HirBinaryOp::Mul => format!("{} * {}", l, r),
            x_hir::HirBinaryOp::Div => format!("{} / {}", l, r),
            x_hir::HirBinaryOp::Mod => format!("{} % {}", l, r),
            x_hir::HirBinaryOp::Equal => format!("{} == {}", l, r),
            x_hir::HirBinaryOp::NotEqual => format!("{} != {}", l, r),
            x_hir::HirBinaryOp::Less => format!("{} < {}", l, r),
            x_hir::HirBinaryOp::LessEqual => format!("{} <= {}", l, r),
            x_hir::HirBinaryOp::Greater => format!("{} > {}", l, r),
            x_hir::HirBinaryOp::GreaterEqual => format!("{} >= {}", l, r),
            x_hir::HirBinaryOp::And => format!("{} and {}", l, r),
            x_hir::HirBinaryOp::Or => format!("{} or {}", l, r),
            _ => format!("/* unsupported binop */ null"),
        }
    }

    /// Emit HIR unary operator
    fn emit_hir_unaryop(&self, op: &x_hir::HirUnaryOp, e: &str) -> String {
        match op {
            x_hir::HirUnaryOp::Negate => format!("-{}", e),
            x_hir::HirUnaryOp::Not => format!("!{}", e),
            _ => format!("/* unsupported unary */ null"),
        }
    }

    fn emit_main_function(&mut self, statements: &[ast::Statement]) -> ZigResult<()> {
        self.line("pub fn main() !void {")?;
        self.indent();

        // Emit top-level statements
        if statements.is_empty() {
            self.line("const stdout = std.fs.File.stdout();")?;
            self.line("")?;
            self.line("// Initialize runtime")?;
            self.line(r#"try stdout.writeAll("Hello from Zig backend!\n");"#)?;
        } else {
            for stmt in statements {
                self.emit_statement(stmt)?;
            }
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_header(&mut self) -> ZigResult<()> {
        self.line(headers::ZIG)?;
        self.line("// DO NOT EDIT")?;
        self.line("")?;

        // 默认导入 std
        self.line("const std = @import(\"std\");")?;
        self.line("")?;

        // 全局 allocator
        self.line("const allocator = std.heap.page_allocator;")?;
        self.line("")?;

        // Helper function for equality comparison (handles strings and other types)
        self.line("fn xEqual(__lhs: anytype, __rhs: @TypeOf(__lhs)) bool {")?;
        self.line("    return if (@typeInfo(@TypeOf(__lhs)) == .pointer)")?;
        self.line("        std.mem.eql(u8, __lhs, __rhs)")?;
        self.line("    else")?;
        self.line("        __lhs == __rhs;")?;
        self.line("}")?;
        self.line("")?;

        // HTTP Server runtime
        self.line("var http_server_handle: ?std.net.Server = null;")?;
        self.line("")?;

        self.line("fn http_listen(host: []const u8, port: u16) void {")?;
        self.indent();
        self.line("const addr = std.net.Address.parseIp(host, port) catch {")?;
        self.indent();
        self.line("std.debug.print(\"Failed to parse address\\\\n\", .{});")?;
        self.line("return;")?;
        self.dedent();
        self.line("};")?;
        self.line("http_server_handle = addr.listen(.{ .reuse_address = true }) catch {")?;
        self.indent();
        self.line("std.debug.print(\"Failed to start server\\\\n\", .{});")?;
        self.line("return;")?;
        self.dedent();
        self.line("};")?;
        self.line(
            "std.debug.print(\"HTTP Server listening on http://{s}:{d}\\\\n\", .{ host, port });",
        )?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        self.line("fn http_accept() ?[]const u8 {")?;
        self.indent();
        self.line("const server = http_server_handle orelse return null;")?;
        self.line("var conn = server.accept() catch return null;")?;
        self.line("defer conn.stream.close();")?;
        self.line("")?;
        self.line("var buf: [4096]u8 = undefined;")?;
        self.line("const n = conn.stream.read(&buf) catch return null;")?;
        self.line("if (n == 0) return null;")?;
        self.line("")?;
        self.line("const request = allocator.alloc(u8, n) catch return null;")?;
        self.line("@memcpy(request, buf[0..n]);")?;
        self.line("return request;")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        self.line(
            "fn http_respond(status: u16, content_type: []const u8, body: []const u8) void {",
        )?;
        self.indent();
        self.line("const server = http_server_handle orelse return;")?;
        self.line("var conn = server.accept() catch return;")?;
        self.line("defer conn.stream.close();")?;
        self.line("")?;
        self.line("var buf: [1024]u8 = undefined;")?;
        self.line("const response = std.fmt.bufPrint(&buf,")?;
        self.indent();
        self.line("\\\\\"HTTP/1.1 {d} OK\\\\r\\\\n\\\\\" ++")?;
        self.line("\\\\\"Content-Type: {s}\\\\r\\\\n\\\\\" ++")?;
        self.line("\\\\\"Content-Length: {d}\\\\r\\\\n\\\\r\\\\n\\\\\"")?;
        self.line(", .{ status, content_type, body.len }) catch return;")?;
        self.dedent();
        self.line("")?;
        self.line("_ = conn.stream.writeAll(response) catch {};")?;
        self.line("_ = conn.stream.writeAll(body) catch {};")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        Ok(())
    }

    fn emit_forward_decl(&mut self, _f: &ast::FunctionDecl) -> ZigResult<()> {
        // Zig不需要前向声明，所以这个函数为空
        Ok(())
    }

    fn emit_global_var(&mut self, v: &ast::VariableDecl) -> ZigResult<()> {
        let init = if let Some(expr) = &v.initializer {
            self.emit_expr(expr)?
        } else {
            "undefined".to_string()
        };

        // Determine type annotation
        let var_type = if let Some(type_annot) = &v.type_annot {
            format!(": {}", self.emit_type(type_annot))
        } else if v.is_mutable {
            // For mutable variables without explicit type, we need to infer the type
            // to avoid comptime_int/comptime_float issues in Zig
            if let Some(expr) = &v.initializer {
                if let Some(inferred) = self.infer_expr_type(expr) {
                    format!(": {}", inferred)
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        // Use 'const' for immutable variables, 'var' for mutable
        let kw = if v.is_mutable { "var" } else { "const" };
        self.line(&format!("{} {} {} = {};", kw, v.name, var_type, init))?;
        self.global_vars.push(v.name.clone());
        Ok(())
    }

    fn emit_function(&mut self, f: &ast::FunctionDecl) -> ZigResult<()> {
        let params = if f.parameters.is_empty() {
            "".to_string()
        } else {
            f.parameters
                .iter()
                .map(|p| {
                    let param_type = if let Some(type_annot) = &p.type_annot {
                        self.emit_type(type_annot)
                    } else {
                        "anytype".to_string()
                    };
                    format!("{}: {}", p.name, param_type)
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let return_type = if let Some(return_type) = &f.return_type {
            format!(" {}", self.emit_type(return_type))
        } else if f.name == "main" {
            // main function needs explicit return type in Zig
            " void".to_string()
        } else {
            "".to_string()
        };
        // Emit async keyword for async functions
        let async_keyword = if f.is_async { "async " } else { "" };
        // Add 'pub' for main function
        let pub_keyword = if f.name == "main" { "pub " } else { "" };
        self.line(&format!(
            "{}{}fn {}({}){} {{",
            pub_keyword, async_keyword, f.name, params, return_type
        ))?;
        self.indent();

        // Check if function has a return type (not void/unit)
        let has_return_type = f.return_type.is_some();

        // Emit all statements except the last one
        let stmt_count = f.body.statements.len();
        for (i, stmt) in f.body.statements.iter().enumerate() {
            let is_last = i == stmt_count - 1;

            // If this is the last statement, function has a return type, and it's an expression,
            // emit it as a return statement
            if is_last && has_return_type {
                if let StatementKind::Expression(expr) = &stmt.node {
                    let e = self.emit_expr(expr)?;
                    self.line(&format!("return {};", e))?;
                } else {
                    self.emit_statement(stmt)?;
                    self.line("return;")?;
                }
            } else {
                self.emit_statement(stmt)?;
            }
        }

        // If no statements and no return type, add return
        if stmt_count == 0 && f.return_type.is_none() {
            self.line("return;")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit an extern function declaration
    fn emit_extern_function(&mut self, f: &ast::ExternFunctionDecl) -> ZigResult<()> {
        // Build parameter string
        let params = if f.parameters.is_empty() && !f.is_variadic {
            "".to_string()
        } else {
            let mut param_strs: Vec<String> = f
                .parameters
                .iter()
                .map(|p| {
                    let param_type = if let Some(type_annot) = &p.type_annot {
                        self.emit_ffi_type(type_annot)
                    } else {
                        "anytype".to_string()
                    };
                    format!("{}: {}", p.name, param_type)
                })
                .collect();

            if f.is_variadic {
                param_strs.push("...".to_string());
            }

            param_strs.join(", ")
        };

        // Build return type
        let return_type = if let Some(ret) = &f.return_type {
            self.emit_ffi_type(ret)
        } else {
            "void".to_string()
        };

        // Emit extern declaration based on ABI
        // Note: Zig 0.13+ doesn't need callconv(.C) for extern "c" functions
        match f.abi.as_str() {
            "C" | "c" => {
                // C ABI: pub extern "c" fn name(params) ReturnType;
                self.needs_libc = true; // Need to link libc for C FFI
                if f.is_variadic {
                    self.line(&format!(
                        "pub extern \"c\" fn {}({}, ...) {};",
                        f.name, params, return_type
                    ))?;
                } else {
                    self.line(&format!(
                        "pub extern \"c\" fn {}({}) {};",
                        f.name, params, return_type
                    ))?;
                }
            }
            "zig" => {
                // Zig ABI: pub extern fn name(params) ReturnType;
                self.line(&format!(
                    "pub extern fn {}({}) {};",
                    f.name, params, return_type
                ))?;
            }
            _ => {
                // Custom ABI: pub extern "abi" fn name(params) ReturnType;
                self.line(&format!(
                    "pub extern \"{}\" fn {}({}) {};",
                    f.abi, f.name, params, return_type
                ))?;
            }
        }

        Ok(())
    }

    /// Emit a class as a Zig struct
    fn emit_class(&mut self, class: &ast::ClassDecl) -> ZigResult<()> {
        // Emit struct definition
        self.line(&format!("const {} = struct {{", class.name))?;
        self.indent();

        // If there's a parent class, embed it as the first field for inheritance
        if let Some(parent) = &class.extends {
            self.line(&format!("base: {},", parent))?;
        }

        // Emit fields
        for member in &class.members {
            if let ast::ClassMember::Field(field) = member {
                let field_type = if let Some(type_annot) = &field.type_annot {
                    self.emit_type(type_annot)
                } else {
                    "anytype".to_string()
                };
                self.line(&format!("{}: {},", field.name, field_type))?;
            }
        }

        // If there are virtual methods, add vtable pointer
        let has_virtual = class
            .members
            .iter()
            .any(|m| matches!(m, ast::ClassMember::Method(m) if m.modifiers.is_virtual));
        if has_virtual {
            self.line(&format!("vtable: *const {}_VTable,", class.name))?;
        }

        self.dedent();
        self.line("};")?;
        self.line("")?;

        // Emit VTable if there are virtual methods
        if has_virtual {
            self.emit_vtable(class)?;
        }

        Ok(())
    }

    /// Emit a vtable for a class with virtual methods
    fn emit_vtable(&mut self, class: &ast::ClassDecl) -> ZigResult<()> {
        self.line(&format!("const {}_VTable = struct {{", class.name))?;
        self.indent();

        for member in &class.members {
            if let ast::ClassMember::Method(method) = member {
                if method.modifiers.is_virtual {
                    let return_type = if let Some(rt) = &method.return_type {
                        format!(" -> {}", self.emit_type(rt))
                    } else {
                        String::new()
                    };

                    // Build function pointer type
                    let mut params = format!("*{}", class.name);
                    for param in &method.parameters {
                        let pt = if let Some(t) = &param.type_annot {
                            self.emit_type(t)
                        } else {
                            "anytype".to_string()
                        };
                        params.push_str(&format!(", {}", pt));
                    }

                    self.line(&format!(
                        "{}: *const fn({}){},",
                        method.name, params, return_type
                    ))?;
                }
            }
        }

        self.dedent();
        self.line("};")?;
        self.line("")?;
        Ok(())
    }

    /// Emit a class method as a Zig function
    fn emit_class_method(&mut self, class_name: &str, method: &ast::FunctionDecl) -> ZigResult<()> {
        // Method name is prefixed with class name
        let func_name = format!("{}_{}", class_name, method.name);

        // Build parameters including this (use 'this' to match X language)
        let mut params_str = format!("this: *{}", class_name);
        for param in &method.parameters {
            let param_type = if let Some(type_annot) = &param.type_annot {
                self.emit_type(type_annot)
            } else {
                "anytype".to_string()
            };
            params_str.push_str(&format!(", {}: {}", param.name, param_type));
        }

        let return_type = if let Some(return_type) = &method.return_type {
            format!(" -> {}", self.emit_type(return_type))
        } else {
            "".to_string()
        };

        // Emit async keyword for async methods
        let async_keyword = if method.is_async { "async " } else { "" };
        self.line(&format!(
            "{}fn {}({}){} {{",
            async_keyword, func_name, params_str, return_type
        ))?;
        self.indent();
        self.emit_block(&method.body)?;
        if method.return_type.is_none() {
            self.line("return;")?;
        }
        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit a constructor as a Zig factory function
    fn emit_constructor(
        &mut self,
        class_name: &str,
        constructor: &ast::ConstructorDecl,
    ) -> ZigResult<()> {
        let func_name = format!("{}_new", class_name);

        // Build parameters
        let mut params_str = String::new();
        for (i, param) in constructor.parameters.iter().enumerate() {
            if i > 0 {
                params_str.push_str(", ");
            }
            let param_type = if let Some(type_annot) = &param.type_annot {
                self.emit_type(type_annot)
            } else {
                "anytype".to_string()
            };
            params_str.push_str(&format!("{}: {}", param.name, param_type));
        }

        self.line(&format!(
            "fn {}({}) {} {{",
            func_name, params_str, class_name
        ))?;
        self.indent();

        // Initialize instance using 'this' name to match X language
        self.line(&format!("var this: {} = undefined;", class_name))?;
        self.emit_block(&constructor.body)?;
        self.line("return this;")?;

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_block(&mut self, block: &ast::Block) -> ZigResult<()> {
        for stmt in &block.statements {
            self.emit_statement(stmt)?;
        }
        Ok(())
    }

    fn emit_statement(&mut self, stmt: &ast::Statement) -> ZigResult<()> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&format!("{};", e))?;
            }
            StatementKind::Variable(v) => {
                let init = if let Some(expr) = &v.initializer {
                    self.emit_expr(expr)?
                } else {
                    "undefined".to_string()
                };
                // Handle underscore variable (discard value in Zig)
                // Variables starting with _ are also treated as intentionally unused
                if v.name == "_" || v.name.starts_with('_') {
                    self.line(&format!("_ = {};", init))?;
                } else {
                    let var_type = if let Some(type_annot) = &v.type_annot {
                        let zig_type = self.emit_type(type_annot);
                        // Track variable type for dictionary inference
                        self.var_types.insert(v.name.clone(), zig_type.clone());
                        format!(" : {}", zig_type)
                    } else {
                        // Try to infer type from initializer
                        if let Some(expr) = &v.initializer {
                            if let Some(inferred_type) = self.infer_expr_type(expr) {
                                self.var_types.insert(v.name.clone(), inferred_type.clone());
                                format!(" : {}", inferred_type)
                            } else {
                                "".to_string()
                            }
                        } else {
                            "".to_string()
                        }
                    };
                    self.line(&format!("const {}{} = {};", v.name, var_type, init))?;
                }
            }
            StatementKind::Return(opt) => {
                if let Some(expr) = opt {
                    let e = self.emit_expr(expr)?;
                    self.line(&format!("return {};", e))?;
                } else {
                    self.line("return;")?;
                }
            }
            StatementKind::If(if_stmt) => {
                self.emit_if(if_stmt)?;
            }
            StatementKind::While(while_stmt) => {
                let cond = self.emit_expr(&while_stmt.condition)?;
                self.line(&format!("while ({}) {{", cond))?;
                self.indent();
                self.emit_block(&while_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            StatementKind::For(for_stmt) => {
                self.emit_for(for_stmt)?;
            }
            StatementKind::Match(match_stmt) => {
                self.emit_match(match_stmt)?;
            }
            StatementKind::Try(try_stmt) => {
                self.emit_try(try_stmt)?;
            }
            StatementKind::Break => {
                self.line("break;")?;
            }
            StatementKind::Continue => {
                self.line("continue;")?;
            }
            StatementKind::DoWhile(d) => {
                self.line("do {")?;
                self.indent();
                self.emit_block(&d.body)?;
                self.dedent();
                let cond = self.emit_expr(&d.condition)?;
                self.line(&format!("}} while ({});", cond))?;
            }
            StatementKind::Unsafe(block) => {
                // Zig 不需要特殊的 unsafe 语法，直接输出块内容
                // 添加注释标记 unsafe 块开始
                self.line("// unsafe block")?;
                self.line("{")?;
                self.indent();
                self.emit_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            StatementKind::Defer(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&format!("defer {};", e))?;
            }
            StatementKind::Yield(_) => {
                // Generators not supported in X-to-Zig, stub
                self.line("// yield (generator not supported)")?;
            }
            StatementKind::Loop(body) => {
                self.line("while (true) {")?;
                self.indent();
                self.emit_block(body)?;
                self.dedent();
                self.line("}")?;
            }
        }
        Ok(())
    }

    fn emit_if(&mut self, if_stmt: &ast::IfStatement) -> ZigResult<()> {
        let cond = self.emit_expr(&if_stmt.condition)?;
        self.line(&format!("if ({}) {{", cond))?;
        self.indent();
        self.emit_block(&if_stmt.then_block)?;
        self.dedent();
        if let Some(else_block) = &if_stmt.else_block {
            self.line("} else {")?;
            self.indent();
            self.emit_block(else_block)?;
            self.dedent();
        }
        self.line("}")?;
        Ok(())
    }

    fn emit_for(&mut self, for_stmt: &ast::ForStatement) -> ZigResult<()> {
        // Zig for 循环语法：for (items) |item| { }
        let iterator = self.emit_expr(&for_stmt.iterator)?;
        let pattern_name = match &for_stmt.pattern {
            ast::Pattern::Variable(name) => name.clone(),
            ast::Pattern::Wildcard => "_".to_string(),
            _ => "_item".to_string(),
        };

        self.line(&format!("for ({}) |{}| {{", iterator, pattern_name))?;
        self.indent();
        self.emit_block(&for_stmt.body)?;
        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_match(&mut self, match_stmt: &ast::MatchStatement) -> ZigResult<()> {
        // Zig 使用 switch 语句进行模式匹配
        let expr = self.emit_expr(&match_stmt.expression)?;
        self.line(&format!("switch ({}) {{", expr))?;
        self.indent();

        for case in &match_stmt.cases {
            let pattern_str = self.emit_pattern(&case.pattern)?;

            // 处理 guard 条件
            if let Some(guard) = &case.guard {
                let guard_expr = self.emit_expr(guard)?;
                self.line(&format!("{} if {} => {{", pattern_str, guard_expr))?;
            } else {
                self.line(&format!("{} => {{", pattern_str))?;
            }

            self.indent();
            self.emit_block(&case.body)?;
            self.dedent();
            self.line("},")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_try(&mut self, try_stmt: &ast::TryStatement) -> ZigResult<()> {
        // Zig 使用 errdefer 和 catch 处理错误
        self.line("{")?;
        self.indent();
        self.line("errdefer {")?;
        self.indent();

        // Emit catch clauses
        for catch in &try_stmt.catch_clauses {
            if let Some(var_name) = &catch.variable_name {
                self.line(&format!("var {} = error{};", var_name, var_name))?;
            }
            self.emit_block(&catch.body)?;
        }

        self.dedent();
        self.line("}")?;

        // Emit try body
        self.emit_block(&try_stmt.body)?;

        // Emit finally block
        if let Some(finally) = &try_stmt.finally_block {
            self.line("defer {")?;
            self.indent();
            self.emit_block(finally)?;
            self.dedent();
            self.line("}")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_pattern(&mut self, pattern: &ast::Pattern) -> ZigResult<String> {
        match pattern {
            ast::Pattern::Wildcard => Ok("_".to_string()),
            ast::Pattern::Variable(name) => Ok(name.clone()),
            ast::Pattern::Literal(lit) => self.emit_literal(lit),
            ast::Pattern::Array(patterns) => {
                let items: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<Result<_, _>>()?;
                Ok(format!("[{}]", items.join(", ")))
            }
            ast::Pattern::Tuple(patterns) => {
                let items: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<Result<_, _>>()?;
                Ok(format!(".{{ {} }}", items.join(", ")))
            }
            ast::Pattern::Record(name, fields) => {
                let field_patterns: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| {
                        let p_str = self.emit_pattern(p).unwrap_or_else(|_| "_".to_string());
                        format!(".{} = {}", n, p_str)
                    })
                    .collect();
                Ok(format!("{}{{ {} }}", name, field_patterns.join(", ")))
            }
            ast::Pattern::Or(left, right) => {
                let l = self.emit_pattern(left)?;
                let r = self.emit_pattern(right)?;
                Ok(format!("{}, {}", l, r))
            }
            ast::Pattern::Guard(inner, _) => {
                // Guard 在 emit_match 中单独处理
                self.emit_pattern(inner)
            }
            ast::Pattern::Dictionary(entries) => {
                let items: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let k_str = self.emit_pattern(k).unwrap_or_else(|_| "_".to_string());
                        let v_str = self.emit_pattern(v).unwrap_or_else(|_| "_".to_string());
                        format!("{}: {}", k_str, v_str)
                    })
                    .collect();
                Ok(format!(".{{ {} }}", items.join(", ")))
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                // Zig enum pattern: .VariantName or .VariantName(patterns)
                if patterns.is_empty() {
                    Ok(format!(".{}", variant_name))
                } else {
                    let pattern_strs: Vec<String> = patterns
                        .iter()
                        .map(|p| self.emit_pattern(p))
                        .collect::<Result<_, _>>()?;
                    Ok(format!(".{}({})", variant_name, pattern_strs.join(", ")))
                }
            }
        }
    }

    fn emit_expr(&mut self, expr: &ast::Expression) -> ZigResult<String> {
        match &expr.node {
            ExpressionKind::Literal(lit) => self.emit_literal(lit),
            ExpressionKind::Variable(name) => Ok(name.clone()),
            ExpressionKind::Binary(op, lhs, rhs) => {
                let l = self.emit_expr(lhs)?;
                let r = self.emit_expr(rhs)?;
                Ok(self.emit_binop(op, &l, &r))
            }
            ExpressionKind::Unary(op, expr) => {
                let e = self.emit_expr(expr)?;
                Ok(self.emit_unaryop(op, &e))
            }
            ExpressionKind::Cast(expr, ty) => {
                let e = self.emit_expr(expr)?;
                let zig_ty = self.emit_type(ty);
                Ok(format!("@as({}, {})", zig_ty, e))
            }
            ExpressionKind::Call(callee, args) => self.emit_call(callee, args),
            ExpressionKind::Assign(target, value) => self.emit_assign(target, value),
            ExpressionKind::Array(elements) => self.emit_array_literal(elements),
            ExpressionKind::Tuple(elements) => self.emit_array_literal(elements), // 元组作为数组处理
            ExpressionKind::Dictionary(entries) => self.emit_dict_literal(entries),
            ExpressionKind::Record(name, fields) => self.emit_record_literal(name, fields),
            ExpressionKind::Lambda(params, body) => self.emit_lambda(params, body),
            ExpressionKind::Range(start, end, inclusive) => self.emit_range(start, end, *inclusive),
            ExpressionKind::Parenthesized(inner) => {
                let e = self.emit_expr(inner)?;
                Ok(format!("({})", e))
            }
            ExpressionKind::If(cond, then_e, else_e) => {
                let c = self.emit_expr(cond)?;
                let t = self.emit_expr(then_e)?;
                let e = self.emit_expr(else_e)?;
                Ok(format!("if ({}) {} else {}", c, t, e))
            }
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                Ok(format!("{}.{}", o, field))
            }
            ExpressionKind::Pipe(input, funcs) => self.emit_pipe(input, funcs),
            ExpressionKind::Wait(wait_type, exprs) => self.emit_wait(wait_type, exprs),
            ExpressionKind::Needs(name) => Ok(format!("// needs: {}", name)),
            ExpressionKind::Given(name, value) => {
                let v = self.emit_expr(value)?;
                Ok(format!("// given: {} = {}", name, v))
            }
            ExpressionKind::Handle(inner_expr, handlers) => {
                let inner = self.emit_expr(inner_expr)?;
                // Generate handler code
                let mut handler_code = String::new();
                for (effect_name, handler) in handlers {
                    let handler_expr = self.emit_expr(handler)?;
                    handler_code.push_str(&format!(
                        "    // handler for {}: {}\n",
                        effect_name, handler_expr
                    ));
                }
                Ok(format!("// handle {} with {{\n{}}}", inner, handler_code))
            }
            ExpressionKind::TryPropagate(inner_expr) => {
                // ? 运算符：在 Zig 中使用 try 或 orelse
                let e = self.emit_expr(inner_expr)?;
                Ok(format!("{} orelse return error.PropagateError", e))
            }
            ExpressionKind::Match(discriminant, cases) => {
                // given discriminant { ... } -> switch on discriminant
                let d = self.emit_expr(discriminant)?;
                let mut output = String::with_capacity(cases.len() * 200); // 预分配容量

                // 使用 write! 宏避免创建临时字符串
                write!(output, "switch ({}) {{\n", d)?;

                for case in cases {
                    // Generate pattern string
                    let pattern_str = self.emit_pattern(&case.pattern)?;

                    // Generate guard if present
                    if let Some(guard) = &case.guard {
                        let guard_expr = self.emit_expr(guard)?;
                        // 直接将内容写入 output，避免创建中间字符串
                        write!(
                            output,
                            "    {} if {} => {},\n",
                            pattern_str,
                            guard_expr,
                            self.extract_first_expression_from_block(&case.body)?
                        )?;
                    } else {
                        write!(
                            output,
                            "    {} => {},\n",
                            pattern_str,
                            self.extract_first_expression_from_block(&case.body)?
                        )?;
                    }
                }

                write!(output, "}}")?;
                Ok(output)
            }
            ExpressionKind::Await(expr) => {
                let e = self.emit_expr(expr)?;
                Ok(format!("await {}", e))
            }
            ExpressionKind::OptionalChain(base, member) => {
                let b = self.emit_expr(base)?;
                Ok(format!("{}?.{}", b, member))
            }
            ExpressionKind::NullCoalescing(left, right) => {
                let l = self.emit_expr(left)?;
                let r = self.emit_expr(right)?;
                Ok(format!("{} ?? {}", l, r))
            }
        }
    }

    fn emit_dict_literal(
        &mut self,
        entries: &[(ast::Expression, ast::Expression)],
    ) -> ZigResult<String> {
        if entries.is_empty() {
            return Ok("std.StringHashMap(void).init(std.heap.page_allocator)".to_string());
        }

        // 从第一个条目推断键值类型
        let key_type = self
            .infer_expr_type(&entries[0].0)
            .unwrap_or_else(|| "[]const u8".to_string());
        let value_type = self
            .infer_expr_type(&entries[0].1)
            .unwrap_or_else(|| "void".to_string());

        let entry_strs: Vec<String> = entries
            .iter()
            .map(|(k, v)| {
                let k_str = self.emit_expr(k)?;
                let v_str = self.emit_expr(v)?;
                Ok(format!("map.put({}, {}) catch unreachable", k_str, v_str))
            })
            .collect::<ZigResult<Vec<_>>>()?;

        // 根据键类型选择合适的 HashMap 类型
        let map_type = if key_type == "[]const u8" || key_type.starts_with("[]const") {
            format!("std.StringHashMap({})", value_type)
        } else {
            format!("std.AutoHashMap({}, {})", key_type, value_type)
        };

        Ok(format!(
            "blk: {{ var map = {}.init(std.heap.page_allocator); {}; break :blk map; }}",
            map_type,
            entry_strs.join("; ")
        ))
    }

    /// 从表达式推断 Zig 类型
    fn infer_expr_type(&self, expr: &ast::Expression) -> Option<String> {
        match &expr.node {
            ExpressionKind::Literal(lit) => match lit {
                ast::Literal::Integer(_) => Some("i32".to_string()),
                ast::Literal::Float(_) => Some("f64".to_string()),
                ast::Literal::String(_) => Some("[]const u8".to_string()),
                ast::Literal::Boolean(_) => Some("bool".to_string()),
                ast::Literal::Char(_) => Some("u8".to_string()),
                ast::Literal::Null => Some("?void".to_string()),
                ast::Literal::None => Some("?void".to_string()),
                ast::Literal::Unit => Some("void".to_string()),
            },
            ExpressionKind::Variable(name) => {
                // 尝试从类型环境获取变量类型
                self.var_types.get(name).cloned()
            }
            _ => None,
        }
    }

    fn emit_record_literal(
        &mut self,
        _name: &str,
        fields: &[(String, ast::Expression)],
    ) -> ZigResult<String> {
        let field_strs: Vec<String> = fields
            .iter()
            .map(|(n, v)| {
                let v_str = self.emit_expr(v)?;
                Ok(format!(".{} = {}", n, v_str))
            })
            .collect::<ZigResult<Vec<_>>>()?;
        Ok(format!(".{{ {} }}", field_strs.join(", ")))
    }

    fn emit_lambda(&mut self, params: &[ast::Parameter], body: &ast::Block) -> ZigResult<String> {
        // Generate unique name for the lambda
        let lambda_name = format!("__Lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        // Build parameter strings
        let param_strs: Vec<String> = params
            .iter()
            .map(|p| {
                if let Some(type_annot) = &p.type_annot {
                    format!("{}: {}", p.name, self.emit_type(type_annot))
                } else {
                    format!("{}: anytype", p.name)
                }
            })
            .collect();

        // Analyze closure captures (variables used but not defined in lambda)
        let captures = self.analyze_captures(params, body);

        // Build capture fields
        let capture_fields: Vec<String> = captures
            .iter()
            .map(|(name, ty)| format!("{}: {}", name, ty))
            .collect();

        // Emit closure struct
        self.line(&format!("const {} = struct {{", lambda_name))?;
        self.indent();

        // Emit capture fields
        for (name, ty) in &captures {
            self.line(&format!("{},", format!("{}: {}", name, ty)))?;
        }

        // Emit call method
        self.line(&format!(
            "fn call({}) anytype {{",
            if captures.is_empty() {
                param_strs.join(", ")
            } else {
                let mut all_params = vec![format!("self: *const @This()")];
                all_params.extend(param_strs.iter().cloned());
                all_params.join(", ")
            }
        ))?;
        self.indent();

        // Emit body
        self.emit_block(body)?;

        // Add implicit return if body doesn't have one
        self.dedent();
        self.line("}")?;

        self.dedent();
        self.line("};")?;

        // Return instance creation
        if captures.is_empty() {
            Ok(format!("{}.init()", lambda_name))
        } else {
            let init_fields: Vec<String> = captures
                .iter()
                .map(|(name, _)| format!(".{} = {}", name, name))
                .collect();
            Ok(format!(
                "{}.{{ {} }}.init()",
                lambda_name,
                init_fields.join(", ")
            ))
        }
    }

    /// Analyze which variables are captured by a lambda
    fn analyze_captures(
        &self,
        params: &[ast::Parameter],
        body: &ast::Block,
    ) -> Vec<(String, String)> {
        let mut captures = Vec::new();

        // Collect parameter names
        let param_names: std::collections::HashSet<String> =
            params.iter().map(|p| p.name.clone()).collect();

        // Walk the body to find free variables
        for stmt in &body.statements {
            self.collect_free_variables(&stmt.node, &param_names, &mut captures);
        }

        captures
    }

    /// Collect free variables from a statement
    fn collect_free_variables(
        &self,
        stmt: &ast::StatementKind,
        local_vars: &std::collections::HashSet<String>,
        captures: &mut Vec<(String, String)>,
    ) {
        match stmt {
            ast::StatementKind::Expression(expr) => {
                self.collect_free_variables_from_expr(expr, local_vars, captures);
            }
            ast::StatementKind::Variable(v) => {
                // Check initializer
                if let Some(init) = &v.initializer {
                    self.collect_free_variables_from_expr(init, local_vars, captures);
                }
                // Variable is now local, not captured
            }
            ast::StatementKind::Return(Some(expr)) => {
                self.collect_free_variables_from_expr(expr, local_vars, captures);
            }
            ast::StatementKind::If(if_stmt) => {
                self.collect_free_variables_from_expr(&if_stmt.condition, local_vars, captures);
                for s in &if_stmt.then_block.statements {
                    self.collect_free_variables(&s.node, local_vars, captures);
                }
                if let Some(else_block) = &if_stmt.else_block {
                    for s in &else_block.statements {
                        self.collect_free_variables(&s.node, local_vars, captures);
                    }
                }
            }
            ast::StatementKind::While(while_stmt) => {
                self.collect_free_variables_from_expr(&while_stmt.condition, local_vars, captures);
                for s in &while_stmt.body.statements {
                    self.collect_free_variables(&s.node, local_vars, captures);
                }
            }
            _ => {}
        }
    }

    /// Collect free variables from an expression
    fn collect_free_variables_from_expr(
        &self,
        expr: &ast::Expression,
        local_vars: &std::collections::HashSet<String>,
        captures: &mut Vec<(String, String)>,
    ) {
        match &expr.node {
            ast::ExpressionKind::Variable(name) => {
                // If it's not a parameter or local, it's captured
                if !local_vars.contains(name) {
                    // Add to captures if not already there
                    if !captures.iter().any(|(n, _)| n == name) {
                        captures.push((name.clone(), "anytype".to_string()));
                    }
                }
            }
            ast::ExpressionKind::Call(callee, args) => {
                self.collect_free_variables_from_expr(callee, local_vars, captures);
                for arg in args {
                    self.collect_free_variables_from_expr(arg, local_vars, captures);
                }
            }
            ast::ExpressionKind::Binary(_, left, right) => {
                self.collect_free_variables_from_expr(left, local_vars, captures);
                self.collect_free_variables_from_expr(right, local_vars, captures);
            }
            ast::ExpressionKind::Unary(_, inner) => {
                self.collect_free_variables_from_expr(inner, local_vars, captures);
            }
            ast::ExpressionKind::Member(obj, _) => {
                self.collect_free_variables_from_expr(obj, local_vars, captures);
            }
            ast::ExpressionKind::Assign(target, value) => {
                self.collect_free_variables_from_expr(target, local_vars, captures);
                self.collect_free_variables_from_expr(value, local_vars, captures);
            }
            ast::ExpressionKind::If(cond, then_expr, else_expr) => {
                self.collect_free_variables_from_expr(cond, local_vars, captures);
                self.collect_free_variables_from_expr(then_expr, local_vars, captures);
                self.collect_free_variables_from_expr(else_expr, local_vars, captures);
            }
            ast::ExpressionKind::Array(elements) => {
                for e in elements {
                    self.collect_free_variables_from_expr(e, local_vars, captures);
                }
            }
            ast::ExpressionKind::Lambda(_, body) => {
                // Nested lambda - need more sophisticated analysis
                for s in &body.statements {
                    self.collect_free_variables(&s.node, local_vars, captures);
                }
            }
            _ => {}
        }
    }

    fn emit_range(
        &mut self,
        start: &ast::Expression,
        end: &ast::Expression,
        inclusive: bool,
    ) -> ZigResult<String> {
        let s = self.emit_expr(start)?;
        let e = self.emit_expr(end)?;
        if inclusive {
            Ok(format!("{}..{}", s, e))
        } else {
            Ok(format!("{}..{}", s, e))
        }
    }

    fn emit_pipe(
        &mut self,
        input: &ast::Expression,
        funcs: &[Box<ast::Expression>],
    ) -> ZigResult<String> {
        let mut result = self.emit_expr(input)?;
        for func in funcs {
            let f = self.emit_expr(func)?;
            result = format!("{}({})", f, result);
        }
        Ok(result)
    }

    fn emit_wait(
        &mut self,
        wait_type: &ast::WaitType,
        exprs: &[ast::Expression],
    ) -> ZigResult<String> {
        let expr_strs: Vec<String> = exprs
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<ZigResult<Vec<_>>>()?;
        match wait_type {
            ast::WaitType::Single => {
                // Single await: await expr
                if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    // Multiple expressions without specific wait type - just await each
                    let awaited: Vec<String> =
                        expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(awaited.join(", "))
                }
            }
            ast::WaitType::Together => {
                // Parallel execution: start all, then await all
                // In Zig, we use a block that starts all async operations and then awaits them
                if expr_strs.is_empty() {
                    Ok("void {}".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    // Generate frame variables for each async operation
                    let frames: Vec<String> = expr_strs
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format!("__frame_{}", i))
                        .collect();
                    let starts: Vec<String> = expr_strs
                        .iter()
                        .enumerate()
                        .map(|(i, e)| format!("var {} = async {};", frames[i], e))
                        .collect();
                    let awaits: Vec<String> =
                        frames.iter().map(|f| format!("await {};", f)).collect();
                    Ok(format!(
                        "blk: {{ {}; {}; break :blk {}; }}",
                        starts.join(" "),
                        awaits.join(" "),
                        frames.last().map(|f| f.as_str()).unwrap_or("void")
                    ))
                }
            }
            ast::WaitType::Race => {
                // Race: first to complete wins
                // In Zig, we can use a select-like pattern or simply race the frames
                if expr_strs.is_empty() {
                    Ok("void {}".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    // Use a race pattern with frame variables
                    let frames: Vec<String> = expr_strs
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format!("__race_frame_{}", i))
                        .collect();
                    let starts: Vec<String> = expr_strs
                        .iter()
                        .enumerate()
                        .map(|(i, e)| format!("var {} = async {};", frames[i], e))
                        .collect();
                    // In Zig, we would use a suspend/resume pattern for true racing
                    // For simplicity, we await in order and return the first result
                    let awaits: Vec<String> =
                        frames.iter().map(|f| format!("await {}", f)).collect();
                    Ok(format!(
                        "blk: {{ {}; break :blk ({}); }}",
                        starts.join(" "),
                        awaits.join(" orelse ")
                    ))
                }
            }
            ast::WaitType::Timeout(timeout_expr) => {
                // Timeout: await with a time limit
                let timeout = self.emit_expr(timeout_expr)?;
                if expr_strs.is_empty() {
                    Ok(format!("void {{ _ = {}; }}", timeout))
                } else {
                    // In Zig, we can use std.time and event loop for timeout
                    // Generate code that races between the operation and a timer
                    let expr = &expr_strs[0];
                    Ok(format!(
                        "blk: {{ const __result = async {}; const __timeout = async std.time.sleep({}); break :blk (await __result) orelse (await __timeout); }}",
                        expr, timeout
                    ))
                }
            }
            ast::WaitType::Atomic => {
                // Atomic: atomic wait/notify operation
                // In Zig, we just await with a comment noting it's atomic
                if expr_strs.len() == 1 {
                    Ok(format!("// atomic\nawait {}", expr_strs[0]))
                } else {
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(format!("// atomic\n{{ {} }}", awaited.join(" ")))
                }
            }
            ast::WaitType::Retry => {
                // Retry: retry an operation automatically
                // Just emit the operation with a comment
                if expr_strs.len() == 1 {
                    Ok(format!("// retry\nawait {}", expr_strs[0]))
                } else {
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(format!("// retry\n{{ {} }}", awaited.join(" ")))
                }
            }
        }
    }

    fn emit_literal(&mut self, lit: &ast::Literal) -> ZigResult<String> {
        match lit {
            ast::Literal::Integer(n) => Ok(format!("{}", n)),
            ast::Literal::Float(f) => Ok(format!("{}", f)),
            ast::Literal::Boolean(b) => Ok(format!("{}", b)),
            ast::Literal::String(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                Ok(format!("\"{}\"", escaped))
            }
            ast::Literal::Char(c) => Ok(format!("'{}'", c)),
            ast::Literal::Null => Ok("null".to_string()),
            ast::Literal::None => Ok("null".to_string()),
            ast::Literal::Unit => Ok("void".to_string()),
        }
    }

    fn emit_binop(&mut self, op: &ast::BinaryOp, l: &str, r: &str) -> String {
        match op {
            ast::BinaryOp::Add => format!("{} + {}", l, r),
            ast::BinaryOp::Sub => format!("{} - {}", l, r),
            ast::BinaryOp::Mul => format!("{} * {}", l, r),
            ast::BinaryOp::Div => format!("{} / {}", l, r),
            ast::BinaryOp::Mod => format!("{} % {}", l, r),
            // Use xEqual helper for equality (handles strings correctly)
            ast::BinaryOp::Equal => format!("xEqual({}, {})", l, r),
            ast::BinaryOp::NotEqual => format!("!xEqual({}, {})", l, r),
            ast::BinaryOp::Less => format!("{} < {}", l, r),
            ast::BinaryOp::LessEqual => format!("{} <= {}", l, r),
            ast::BinaryOp::Greater => format!("{} > {}", l, r),
            ast::BinaryOp::GreaterEqual => format!("{} >= {}", l, r),
            ast::BinaryOp::And => format!("{} and {}", l, r),
            ast::BinaryOp::Or => format!("{} or {}", l, r),
            ast::BinaryOp::Pow => format!("@exp(@log({}) * {})", l, r),
            _ => format!("/* unsupported binop {:?} */ null", op),
        }
    }

    fn emit_unaryop(&mut self, op: &ast::UnaryOp, e: &str) -> String {
        match op {
            ast::UnaryOp::Negate => format!("-{}", e),
            ast::UnaryOp::Not => format!("!{}", e),
            _ => format!("/* unsupported unary {:?} */ null", op),
        }
    }

    fn emit_call(
        &mut self,
        callee: &ast::Expression,
        args: &[ast::Expression],
    ) -> ZigResult<String> {
        // Handle special pointer operations like Pointer(T).null()
        if let ExpressionKind::Member(obj, method) = &callee.node {
            // Check if this is a Pointer(T).null() or ConstPointer(T).null() call
            if method == "null" && args.is_empty() {
                if let ExpressionKind::Call(inner_callee, inner_args) = &obj.node {
                    if let ExpressionKind::Variable(type_name) = &inner_callee.node {
                        if (type_name == "Pointer" || type_name == "ConstPointer")
                            && inner_args.len() == 1
                        {
                            // Get the inner type
                            let inner_type_str = self.emit_expr(&inner_args[0])?;
                            // Map X type to Zig type for pointer
                            let zig_inner_type = match inner_type_str.as_str() {
                                "Void" => "void",
                                "Int" => "i32",
                                "Float" => "f64",
                                "Bool" => "bool",
                                "String" => "u8",
                                "Char" => "u8",
                                "CInt" => "c_int",
                                "CLong" => "c_long",
                                "CSize" => "usize",
                                "CChar" => "c_char",
                                other => other, // Use as-is for other types
                            };
                            if type_name == "Pointer" {
                                return Ok(format!("@as(?*{}, null)", zig_inner_type));
                            } else {
                                return Ok(format!("@as(?*const {}, null)", zig_inner_type));
                            }
                        }
                    }
                }
            }

            // Regular member call
            let callee_str = self.emit_expr(callee)?;
            let arg_strs: Vec<String> = args
                .iter()
                .map(|a| self.emit_expr(a))
                .collect::<ZigResult<Vec<_>>>()?;
            return Ok(format!("{}({})", callee_str, arg_strs.join(", ")));
        }

        if let ExpressionKind::Variable(name) = &callee.node {
            let arg_strs: Vec<String> = args
                .iter()
                .map(|a| self.emit_expr(a))
                .collect::<ZigResult<Vec<_>>>()?;
            return Ok(self.emit_builtin_or_call(name, &arg_strs));
        }
        let callee_str = self.emit_expr(callee)?;
        let arg_strs: Vec<String> = args
            .iter()
            .map(|a| self.emit_expr(a))
            .collect::<ZigResult<Vec<_>>>()?;

        // 处理allocator.alloc等返回错误联合类型的函数
        if callee_str.ends_with(".alloc") {
            Ok(format!(
                "{}({}, {}) catch unreachable",
                callee_str, arg_strs[0], arg_strs[1]
            ))
        } else {
            Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
        }
    }

    fn emit_builtin_or_call(&mut self, name: &str, args: &[String]) -> String {
        match name {
            "print" | "println" => {
                if args.len() == 1 {
                    // Detect if the argument is a string literal (starts and ends with quotes)
                    let arg = &args[0];
                    let is_string_literal = arg.starts_with('"') && arg.ends_with('"');
                    let format_spec = if is_string_literal { "{s}" } else { "{any}" };
                    format!("std.debug.print(\"{}\\n\", .{{{}}})", format_spec, args[0])
                } else {
                    "std.debug.print(\"\\n\", .{{}})".to_string()
                }
            }
            "print_inline" => {
                if args.len() == 1 {
                    let arg = &args[0];
                    let is_string_literal = arg.starts_with('"') && arg.ends_with('"');
                    let format_spec = if is_string_literal { "{s}" } else { "{any}" };
                    format!("std.debug.print(\"{}\", .{{{}}})", format_spec, args[0])
                } else {
                    "std.debug.print(\"\", .{{}})".to_string()
                }
            }
            "concat" => {
                if args.len() == 2 {
                    format!(
                        "std.mem.concat(allocator, u8, &[_][]const u8{{ {}, {} }}) catch unreachable",
                        args[0], args[1]
                    )
                } else {
                    "\"\"".to_string()
                }
            }
            "to_string" => format!(
                "std.fmt.allocPrint(allocator, \"{{}}\", .{{{}}}) catch unreachable",
                args.first().map(|s| s.as_str()).unwrap_or("null")
            ),
            "string_length" => {
                let s = args.first().map(|s| s.as_str()).unwrap_or("\"\"");
                format!("{}.len", s)
            }
            "string_find" => {
                let s = args.get(0).map(|s| s.as_str()).unwrap_or("\"\"");
                let substr = args.get(1).map(|s| s.as_str()).unwrap_or("\"\"");
                format!(
                    r#"(blk: {{
    const idx = std.mem.indexOf(u8, {}, {});
    break :blk if (idx) |i| @as(i32, @intCast(i)) else @as(i32, -1);
}})"#,
                    s, substr
                )
            }
            "string_substring" => {
                let s = args.get(0).map(|s| s.as_str()).unwrap_or("\"\"");
                let start = args.get(1).map(|s| s.as_str()).unwrap_or("0");
                let end = args.get(2).map(|s| s.as_str()).unwrap_or("0");
                format!("{}[{}..{}]", s, start, end)
            }
            "int_to_string" => {
                let n = args.first().map(|s| s.as_str()).unwrap_or("0");
                format!(
                    "std.fmt.allocPrint(allocator, \"{{d}}\", .{{{}}}) catch unreachable",
                    n
                )
            }
            "type_of" => format!(
                "@typeName(@TypeOf({}))",
                args.first().map(|s| s.as_str()).unwrap_or("null")
            ),
            "panic" => {
                if args.len() == 1 {
                    format!("std.debug.panic(\"{{}}\", .{{{}}})", args[0])
                } else {
                    "std.debug.panic(\"panic\", .{{}})".to_string()
                }
            }
            "len" => format!("{}.len", args.first().map(|s| s.as_str()).unwrap_or("null")),
            _ => {
                format!("{}({})", name, args.join(", "))
            }
        }
    }

    fn emit_assign(
        &mut self,
        target: &ast::Expression,
        value: &ast::Expression,
    ) -> ZigResult<String> {
        let val = self.emit_expr(value)?;
        match &target.node {
            ExpressionKind::Variable(name) => Ok(format!("{} = {}", name, val)),
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                Ok(format!("{}.{} = {}", o, field, val))
            }
            _ => {
                let t = self.emit_expr(target)?;
                Ok(format!("{} = {}", t, val))
            }
        }
    }

    fn emit_array_literal(&mut self, elements: &[ast::Expression]) -> ZigResult<String> {
        if elements.is_empty() {
            return Ok("[]anytype{}".to_string());
        }
        let elem_strs: Vec<String> = elements
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<ZigResult<Vec<_>>>()?;
        Ok(format!("[_]anytype{{{}}}", elem_strs.join(", ")))
    }

    fn emit_type(&self, ty: &ast::Type) -> String {
        match ty {
            ast::Type::Int => "i32".to_string(),
            ast::Type::UnsignedInt => "u32".to_string(),
            ast::Type::Float => "f64".to_string(),
            ast::Type::Bool => "bool".to_string(),
            ast::Type::String => "[]const u8".to_string(),
            ast::Type::Char => "u8".to_string(),
            ast::Type::Unit => "void".to_string(),
            ast::Type::Never => "noreturn".to_string(),
            ast::Type::Dynamic => "anytype".to_string(),
            ast::Type::Array(inner) => format!("[] {}", self.emit_type(inner)),
            ast::Type::Dictionary(key, value) => {
                let key_type = self.emit_type(key);
                let value_type = self.emit_type(value);
                // 使用 StringHashMap 当键是字符串类型
                if key_type == "[]const u8" {
                    format!("std.StringHashMap({})", value_type)
                } else {
                    format!("std.AutoHashMap({}, {})", key_type, value_type)
                }
            }
            ast::Type::Option(inner) => format!("?{}", self.emit_type(inner)),
            ast::Type::Result(ok, err) => format!("{}!{}", self.emit_type(err), self.emit_type(ok)),
            ast::Type::Function(params, return_type) => {
                let param_types = params
                    .iter()
                    .map(|p| self.emit_type(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", param_types, self.emit_type(return_type))
            }
            ast::Type::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.emit_type(t)).collect();
                format!("struct {{ {} }}", type_strs.join(", "))
            }
            ast::Type::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, self.emit_type(t)))
                    .collect();
                format!("struct {} {{ {} }}", name, field_strs.join(", "))
            }
            ast::Type::Union(name, _) => name.clone(),
            ast::Type::Generic(name) | ast::Type::TypeParam(name) | ast::Type::Var(name) => {
                // Handle builtin type names
                match name.as_str() {
                    "string" => "[]const u8".to_string(),
                    "integer" => "i32".to_string(),
                    "float" => "f64".to_string(),
                    "boolean" => "bool".to_string(),
                    "character" => "u8".to_string(),
                    _ => name.clone(),
                }
            }
            ast::Type::TypeConstructor(name, type_args) => {
                // 泛型类型应用，如 List<Int>
                let args: Vec<String> = type_args.iter().map(|t| self.emit_type(t)).collect();
                format!("{}({})", name, args.join(", "))
            }
            ast::Type::Async(inner) => self.emit_type(inner),
            ast::Type::Dynamic => "anytype".to_string(),
            ast::Type::Reference(inner) => format!("&{}", self.emit_type(inner)),
            ast::Type::MutableReference(inner) => format!("*mut {}", self.emit_type(inner)),
            // FFI pointer types
            ast::Type::Pointer(inner) => format!("*{}", self.emit_type(inner)),
            ast::Type::ConstPointer(inner) => format!("*const {}", self.emit_type(inner)),
            ast::Type::Void => "void".to_string(),
            // C FFI types - map to Zig's C ABI types
            ast::Type::CInt => "c_int".to_string(),
            ast::Type::CUInt => "c_uint".to_string(),
            ast::Type::CLong => "c_long".to_string(),
            ast::Type::CULong => "c_ulong".to_string(),
            ast::Type::CLongLong => "c_longlong".to_string(),
            ast::Type::CULongLong => "c_ulonglong".to_string(),
            ast::Type::CFloat => "c_float".to_string(),
            ast::Type::CDouble => "c_double".to_string(),
            ast::Type::CChar => "c_char".to_string(),
            ast::Type::CSize => "usize".to_string(),
            ast::Type::CString => "[*c]u8".to_string(),
        }
    }

    /// Emit type for FFI (extern function) declarations
    /// Key difference: pointers are emitted as nullable (?*T) for C ABI compatibility
    fn emit_ffi_type(&self, ty: &ast::Type) -> String {
        match ty {
            // References are still non-nullable in FFI
            ast::Type::Reference(inner) => format!("&{}", self.emit_type(inner)),
            ast::Type::MutableReference(inner) => format!("*mut {}", self.emit_type(inner)),
            // For FFI, emit nullable pointers since C pointers can be null
            ast::Type::Pointer(inner) => format!("?*{}", self.emit_type(inner)),
            ast::Type::ConstPointer(inner) => format!("?*const {}", self.emit_type(inner)),
            // Other types are the same
            _ => self.emit_type(ty),
        }
    }

    fn emit_import(&mut self, import: &ast::ImportDecl) -> ZigResult<()> {
        // 检查是否是Zig标准库导入
        if import.module_path.starts_with("zig::") {
            let zig_module = import.module_path.trim_start_matches("zig::");

            // 处理子模块导入，如 zig::std::math
            let zig_import_path = zig_module.replace("::", ".");

            // 处理导入符号
            if import.symbols.is_empty() || import.symbols.contains(&ast::ImportSymbol::All) {
                // 通配导入或无符号导入，导入整个模块
                let module_name = zig_module.split("::").last().unwrap_or(zig_module);
                let import_stmt =
                    format!("const {} = @import(\"{}\");", module_name, zig_import_path);

                // 避免重复导入
                if !self.imported_modules.contains(&module_name.to_string()) {
                    self.line(&import_stmt)?;
                    self.imported_modules.push(module_name.to_string());
                }
            } else {
                // 命名导入
                for symbol in &import.symbols {
                    if let ast::ImportSymbol::Named(name, alias) = symbol {
                        let import_name = alias.as_ref().unwrap_or(name);
                        let import_stmt = format!(
                            "const {} = @import(\"{}\").{};",
                            import_name, zig_import_path, name
                        );

                        // 避免重复导入
                        if !self.imported_modules.contains(&import_name.to_string()) {
                            self.line(&import_stmt)?;
                            self.imported_modules.push(import_name.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn line(&mut self, s: &str) -> ZigResult<()> {
        self.buffer.line(s).map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))
    }

    /// 增加缩进
    fn indent(&mut self) {
        self.buffer.indent();
    }

    /// 减少缩进
    fn dedent(&mut self) {
        self.buffer.dedent();
    }

    /// 获取当前输出
    fn output(&self) -> &str {
        self.buffer.as_str()
    }

    /// Compile generated Zig code to executable
    pub fn compile_zig_code(&self, zig_code: &str, output_file: &PathBuf) -> ZigResult<()> {
        use std::process::Command;

        let zig_file = output_file.with_extension("zig");
        std::fs::write(&zig_file, zig_code)?;

        // Build zig command
        let mut cmd = Command::new("zig");
        cmd.arg("build-exe")
            .arg(&zig_file)
            .arg("-O")
            .arg(if self.config.optimize {
                "ReleaseFast"
            } else {
                "Debug"
            });

        // Add target if not native
        if self.config.target != ZigTarget::Native {
            cmd.arg("-target").arg(self.config.target.as_zig_target());
        }

        // Debug info is already included in Debug optimization mode
        // The -g flag format changed in Zig 0.15+, and Debug mode includes debug info by default

        // Link libc if needed (for extern "c" functions)
        if self.needs_libc {
            cmd.arg("-lc");
        }

        // For Wasm targets, set output name explicitly
        if self.config.target != ZigTarget::Native {
            let wasm_output = output_file.with_extension(self.config.target.output_extension());
            cmd.arg("-femit-bin=").arg(&wasm_output);
        }

        // Execute compilation
        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(x_codegen::CodeGenError::CompilerError(format!(
                "Zig compiler failed:\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::{spanned, MethodModifiers, Visibility};

    #[test]
    fn test_hello_world_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Expression(spanned(
                            ExpressionKind::Call(
                                Box::new(spanned(
                                    ExpressionKind::Variable("print".to_string()),
                                    Span::default(),
                                )),
                                vec![spanned(
                                    ExpressionKind::Literal(ast::Literal::String(
                                        "Hello, World!".to_string(),
                                    )),
                                    Span::default(),
                                )],
                            ),
                            Span::default(),
                        )),
                        Span::default(),
                    )],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("std.debug.print"));
        assert!(zig_code.contains("Hello, World!"));
        assert!(zig_code.contains("fn main()"));
    }

    #[test]
    fn test_for_loop_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::For(ast::ForStatement {
                            pattern: ast::Pattern::Variable("i".to_string()),
                            iterator: spanned(
                                ExpressionKind::Array(vec![
                                    spanned(
                                        ExpressionKind::Literal(ast::Literal::Integer(1)),
                                        Span::default(),
                                    ),
                                    spanned(
                                        ExpressionKind::Literal(ast::Literal::Integer(2)),
                                        Span::default(),
                                    ),
                                    spanned(
                                        ExpressionKind::Literal(ast::Literal::Integer(3)),
                                        Span::default(),
                                    ),
                                ]),
                                Span::default(),
                            ),
                            body: ast::Block {
                                statements: vec![spanned(
                                    StatementKind::Expression(spanned(
                                        ExpressionKind::Call(
                                            Box::new(spanned(
                                                ExpressionKind::Variable("print".to_string()),
                                                Span::default(),
                                            )),
                                            vec![spanned(
                                                ExpressionKind::Variable("i".to_string()),
                                                Span::default(),
                                            )],
                                        ),
                                        Span::default(),
                                    )),
                                    Span::default(),
                                )],
                            },
                        }),
                        Span::default(),
                    )],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("for"));
        assert!(zig_code.contains("|i|"));
    }

    #[test]
    fn test_match_statement_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Match(ast::MatchStatement {
                            expression: spanned(
                                ExpressionKind::Literal(ast::Literal::Integer(1)),
                                Span::default(),
                            ),
                            cases: vec![
                                ast::MatchCase {
                                    pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                    body: ast::Block {
                                        statements: vec![spanned(
                                            StatementKind::Expression(spanned(
                                                ExpressionKind::Call(
                                                    Box::new(spanned(
                                                        ExpressionKind::Variable(
                                                            "print".to_string(),
                                                        ),
                                                        Span::default(),
                                                    )),
                                                    vec![spanned(
                                                        ExpressionKind::Literal(
                                                            ast::Literal::String("one".to_string()),
                                                        ),
                                                        Span::default(),
                                                    )],
                                                ),
                                                Span::default(),
                                            )),
                                            Span::default(),
                                        )],
                                    },
                                    guard: None,
                                },
                                ast::MatchCase {
                                    pattern: ast::Pattern::Wildcard,
                                    body: ast::Block {
                                        statements: vec![spanned(
                                            StatementKind::Expression(spanned(
                                                ExpressionKind::Call(
                                                    Box::new(spanned(
                                                        ExpressionKind::Variable(
                                                            "print".to_string(),
                                                        ),
                                                        Span::default(),
                                                    )),
                                                    vec![spanned(
                                                        ExpressionKind::Literal(
                                                            ast::Literal::String(
                                                                "other".to_string(),
                                                            ),
                                                        ),
                                                        Span::default(),
                                                    )],
                                                ),
                                                Span::default(),
                                            )),
                                            Span::default(),
                                        )],
                                    },
                                    guard: None,
                                },
                            ],
                        }),
                        Span::default(),
                    )],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("switch"));
        assert!(zig_code.contains("=>"));
    }

    #[test]
    fn test_option_type_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "maybe_value".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: Some(ast::Type::Option(Box::new(ast::Type::Int))),
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Return(Some(spanned(
                            ExpressionKind::Literal(ast::Literal::Integer(42)),
                            Span::default(),
                        ))),
                        Span::default(),
                    )],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        // Option<Int> maps to ?i32 in Zig
        assert!(zig_code.contains("?i32"));
        assert!(zig_code.contains("fn maybe_value()"));
    }

    #[test]
    fn test_result_type_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "divide".to_string(),
                type_parameters: vec![],
                parameters: vec![ast::Parameter {
                    name: "x".to_string(),
                    type_annot: Some(ast::Type::Int),
                    default: None,
                    span: Span::default(),
                }],
                return_type: Some(ast::Type::Result(
                    Box::new(ast::Type::Int),
                    Box::new(ast::Type::String),
                )),
                effects: vec![],
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Return(Some(spanned(
                            ExpressionKind::Literal(ast::Literal::Integer(10)),
                            Span::default(),
                        ))),
                        Span::default(),
                    )],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        // Result<Int, String> maps to []const u8!i32 in Zig (error type first)
        assert!(zig_code.contains("!i32"));
        assert!(zig_code.contains("fn divide"));
    }

    #[test]
    fn test_try_statement_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Try(ast::TryStatement {
                            body: ast::Block {
                                statements: vec![spanned(
                                    StatementKind::Expression(spanned(
                                        ExpressionKind::Call(
                                            Box::new(spanned(
                                                ExpressionKind::Variable(
                                                    "risky_operation".to_string(),
                                                ),
                                                Span::default(),
                                            )),
                                            vec![],
                                        ),
                                        Span::default(),
                                    )),
                                    Span::default(),
                                )],
                            },
                            catch_clauses: vec![ast::CatchClause {
                                exception_type: Some("Error".to_string()),
                                variable_name: Some("e".to_string()),
                                body: ast::Block {
                                    statements: vec![spanned(
                                        StatementKind::Expression(spanned(
                                            ExpressionKind::Call(
                                                Box::new(spanned(
                                                    ExpressionKind::Variable("print".to_string()),
                                                    Span::default(),
                                                )),
                                                vec![spanned(
                                                    ExpressionKind::Variable("e".to_string()),
                                                    Span::default(),
                                                )],
                                            ),
                                            Span::default(),
                                        )),
                                        Span::default(),
                                    )],
                                },
                            }],
                            finally_block: Some(ast::Block {
                                statements: vec![spanned(
                                    StatementKind::Expression(spanned(
                                        ExpressionKind::Call(
                                            Box::new(spanned(
                                                ExpressionKind::Variable("print".to_string()),
                                                Span::default(),
                                            )),
                                            vec![spanned(
                                                ExpressionKind::Literal(ast::Literal::String(
                                                    "cleanup".to_string(),
                                                )),
                                                Span::default(),
                                            )],
                                        ),
                                        Span::default(),
                                    )),
                                    Span::default(),
                                )],
                            }),
                        }),
                        Span::default(),
                    )],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("errdefer"));
        assert!(zig_code.contains("defer"));
    }

    #[test]
    fn test_record_expression_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Variable(ast::VariableDecl {
                            span: Span::default(),
                            name: "person".to_string(),
                            is_mutable: false,
                            is_constant: false,
                            type_annot: None,
                            initializer: Some(spanned(
                                ExpressionKind::Record(
                                    "Person".to_string(),
                                    vec![
                                        (
                                            "name".to_string(),
                                            spanned(
                                                ExpressionKind::Literal(ast::Literal::String(
                                                    "Alice".to_string(),
                                                )),
                                                Span::default(),
                                            ),
                                        ),
                                        (
                                            "age".to_string(),
                                            spanned(
                                                ExpressionKind::Literal(ast::Literal::Integer(30)),
                                                Span::default(),
                                            ),
                                        ),
                                    ],
                                ),
                                Span::default(),
                            )),
                            visibility: Visibility::default(),
                        }),
                        Span::default(),
                    )],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains(".name"));
        assert!(zig_code.contains(".age"));
        assert!(zig_code.contains("Alice"));
    }

    #[test]
    fn test_generate_from_hir_empty() {
        use std::collections::HashMap;
        use x_hir::{Hir, HirPerceusInfo, HirTypeEnv};

        let hir = Hir {
            module_name: "test".to_string(),
            declarations: vec![],
            statements: vec![],
            type_env: HirTypeEnv {
                variables: HashMap::new(),
                functions: HashMap::new(),
                types: HashMap::new(),
            },
            perceus_info: HirPerceusInfo::default(),
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_hir(&hir).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("// Generated by X-Lang"));
    }

    #[test]
    fn test_generate_from_pir_empty() {
        use x_mir::{PerceusIR, ReuseAnalysis};

        let pir = PerceusIR {
            functions: vec![],
            global_ops: vec![],
            reuse_analysis: ReuseAnalysis {
                reuse_pairs: vec![],
                estimated_savings: 0,
            },
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_pir(&pir).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("// Generated by X-Lang"));
        assert!(zig_code.contains("const std = @import"));
    }

    #[test]
    fn test_generate_from_pir_with_memory_ops() {
        use x_mir::{
            BasicBlock, ControlFlowAnalysis, FunctionAnalysis, MemoryOp, OwnershipFact,
            OwnershipState, PerceusIR, ReuseAnalysis, SourcePos,
        };

        let pir = PerceusIR {
            functions: vec![FunctionAnalysis {
                name: "test_func".to_string(),
                param_ownership: vec![OwnershipFact {
                    variable: "x".to_string(),
                    state: OwnershipState::Owned,
                    ty: "Int".to_string(),
                }],
                return_ownership: OwnershipFact {
                    variable: "return".to_string(),
                    state: OwnershipState::Owned,
                    ty: "Int".to_string(),
                },
                memory_ops: vec![
                    MemoryOp::Dup {
                        variable: "x".to_string(),
                        target: "x_dup".to_string(),
                        position: SourcePos { line: 1, column: 1 },
                    },
                    MemoryOp::Drop {
                        variable: "temp".to_string(),
                        position: SourcePos { line: 2, column: 1 },
                    },
                ],
                control_flow: ControlFlowAnalysis {
                    blocks: vec![],
                    edges: vec![],
                },
            }],
            global_ops: vec![],
            reuse_analysis: ReuseAnalysis {
                reuse_pairs: vec![],
                estimated_savings: 0,
            },
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_pir(&pir).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("fn test_func"));
        assert!(zig_code.contains("allocator.dupe"));
        assert!(zig_code.contains("defer allocator.free"));
    }

    #[test]
    fn test_async_function_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "fetch_data".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: Some(ast::Type::String),
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Return(Some(spanned(
                            ExpressionKind::Literal(ast::Literal::String("data".to_string())),
                            Span::default(),
                        ))),
                        Span::default(),
                    )],
                },
                is_async: true,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("async fn fetch_data"));
    }

    #[test]
    fn test_wait_single_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Expression(spanned(
                            ExpressionKind::Wait(
                                ast::WaitType::Single,
                                vec![spanned(
                                    ExpressionKind::Call(
                                        Box::new(spanned(
                                            ExpressionKind::Variable("fetch".to_string()),
                                            Span::default(),
                                        )),
                                        vec![],
                                    ),
                                    Span::default(),
                                )],
                            ),
                            Span::default(),
                        )),
                        Span::default(),
                    )],
                },
                is_async: true,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("await fetch()"));
    }

    #[test]
    fn test_wait_together_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Expression(spanned(
                            ExpressionKind::Wait(
                                ast::WaitType::Together,
                                vec![
                                    spanned(
                                        ExpressionKind::Variable("task1".to_string()),
                                        Span::default(),
                                    ),
                                    spanned(
                                        ExpressionKind::Variable("task2".to_string()),
                                        Span::default(),
                                    ),
                                ],
                            ),
                            Span::default(),
                        )),
                        Span::default(),
                    )],
                },
                is_async: true,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        // Together should use async frame pattern
        assert!(zig_code.contains("__frame_0"));
        assert!(zig_code.contains("__frame_1"));
        assert!(zig_code.contains("async task1"));
        assert!(zig_code.contains("async task2"));
    }
}

// ============================================================================
// LIR 辅助函数
// ============================================================================

impl ZigBackend {
    /// 发出外部函数声明（来自 LIR）
    fn emit_lir_extern_function(&mut self, extern_func: &x_lir::ExternFunction) -> ZigResult<()> {
        // Output generic type parameters if any: (T: type, U: type)
        let type_params_str = if extern_func.type_params.is_empty() {
            "".to_string()
        } else {
            let type_params = extern_func.type_params
                .iter()
                .map(|tp| format!("{}: type", tp))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", type_params)
        };

        let params = if extern_func.parameters.is_empty() {
            "".to_string()
        } else {
            extern_func
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param_type)| format!("arg{}: {}", i, self.emit_lir_type(param_type)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        // Combine type params and regular params for Zig generic function syntax
        let full_params = match (type_params_str.is_empty(), params.is_empty()) {
            (true, true) => "".to_string(),
            (true, false) => params,
            (false, true) => type_params_str,
            (false, false) => format!("{}({})", type_params_str, params),
        };

        let return_type = self.emit_lir_type(&extern_func.return_type);
        match &extern_func.abi {
            Some(abi) if abi == "C" => {
                self.line(&format!(
                    "pub extern \"c\" fn {}{} {};",
                    extern_func.name, full_params, return_type
                ))?;
            }
            Some(abi) => {
                self.line(&format!(
                    "pub extern \"{}\" fn {}{} {};",
                    abi, extern_func.name, full_params, return_type
                ))?;
            }
            None => {
                self.line(&format!(
                    "extern fn {}{} {};",
                    extern_func.name, full_params, return_type
                ))?;
            }
        }
        Ok(())
    }

    /// 发出全局变量（来自 LIR）
    fn emit_lir_global_var(&mut self, global_var: &x_lir::GlobalVar) -> ZigResult<()> {
        let type_str = self.emit_lir_type(&global_var.type_);
        if let Some(initializer) = &global_var.initializer {
            let init_str = self.emit_lir_expression(initializer)?;
            self.line(&format!(
                "pub var {} : {} = {};",
                global_var.name, type_str, init_str
            ))?;
        } else {
            self.line(&format!(
                "pub var {} : {} = undefined;",
                global_var.name, type_str
            ))?;
        }
        Ok(())
    }

    /// 发出结构体定义（来自 LIR）
    fn emit_lir_struct(&mut self, struct_def: &x_lir::Struct) -> ZigResult<()> {
        self.line(&format!("pub const {} = struct {{", struct_def.name))?;
        self.indent();

        for field in &struct_def.fields {
            let type_str = self.emit_lir_type(&field.type_);
            self.line(&format!("{}: {},", field.name, type_str))?;
        }

        self.dedent();
        self.line("};")?;
        self.line("")?;
        Ok(())
    }

    /// 发出枚举定义（来自 LIR）
    fn emit_lir_enum(&mut self, enum_def: &x_lir::Enum) -> ZigResult<()> {
        self.line(&format!("pub const {} = enum {{", enum_def.name))?;
        self.indent();

        for variant in &enum_def.variants {
            if let Some(value) = variant.value {
                self.line(&format!("{} = {},", variant.name, value))?;
            } else {
                self.line(&format!("{},", variant.name))?;
            }
        }

        self.dedent();
        self.line("};")?;
        self.line("")?;
        Ok(())
    }
}

// ============================================================================
// 实现 CodeGenerator trait
// ============================================================================

impl x_codegen::CodeGenerator for ZigBackend {
    type Config = ZigBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        ZigBackend::new(config)
    }

    fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> Result<x_codegen::CodegenOutput, Self::Error> {
        ZigBackend::generate_from_ast(self, program)
    }

    fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> Result<x_codegen::CodegenOutput, Self::Error> {
        ZigBackend::generate_from_hir(self, hir)
    }

    fn generate_from_lir(
        &mut self,
        lir: &x_lir::Program,
    ) -> Result<x_codegen::CodegenOutput, Self::Error> {
        ZigBackend::generate_from_lir(self, lir)
    }
}

impl ZigBackend {
    /// 发出函数定义（来自 LIR）
    fn emit_lir_function(&mut self, func: &x_lir::Function) -> ZigResult<()> {
        // Output generic type parameters if any: (T: type, U: type)
        let type_params_str = if func.type_params.is_empty() {
            "".to_string()
        } else {
            let type_params = func.type_params
                .iter()
                .map(|tp| format!("{}: type", tp))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", type_params)
        };

        let params = if func.parameters.is_empty() {
            "".to_string()
        } else {
            func.parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.emit_lir_type(&p.type_)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let return_type = self.emit_lir_type(&func.return_type);
        let pub_str = if func.name == "main" { "pub " } else { "" };

        // Combine type params and regular params for Zig generic function syntax
        let full_params = match (type_params_str.is_empty(), params.is_empty()) {
            (true, true) => "".to_string(),
            (true, false) => params,
            (false, true) => type_params_str,
            (false, false) => format!("{}({})", type_params_str, params),
        };

        self.line(&format!(
            "{}fn {}{} {} {{",
            pub_str, func.name, full_params, return_type
        ))?;
        self.indent();

        // Emit function body
        self.emit_lir_block(&func.body)?;

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// 发出块（来自 LIR）
    fn emit_lir_block(&mut self, block: &x_lir::Block) -> ZigResult<()> {
        for stmt in &block.statements {
            self.emit_lir_statement(stmt)?;
        }
        Ok(())
    }

    /// 发出声明（来自 LIR）
    fn emit_lir_declaration(&mut self, decl: &x_lir::Declaration) -> ZigResult<()> {
        match decl {
            x_lir::Declaration::Function(_) => {
                // Functions are handled separately in program level
            }
            x_lir::Declaration::Global(global_var) => {
                self.emit_lir_global_var(global_var)?;
            }
            x_lir::Declaration::Struct(struct_def) => {
                self.emit_lir_struct(struct_def)?;
            }
            x_lir::Declaration::Class(class_def) => {
                // Classes are compiled to structs in Zig
                self.line(&format!("pub const {} = struct {{", class_def.name))?;
                self.indent();

                for field in &class_def.fields {
                    let type_str = self.emit_lir_type(&field.type_);
                    self.line(&format!("{}: {},", field.name, type_str))?;
                }

                self.dedent();
                self.line("};")?;
                self.line("")?;
            }
            x_lir::Declaration::Enum(enum_def) => {
                self.emit_lir_enum(enum_def)?;
            }
            x_lir::Declaration::Import(import) => {
                // 处理导入声明，类似 AST 版本逻辑
                if import.module_path.starts_with("zig::") {
                    let zig_module = import.module_path.trim_start_matches("zig::");
                    let zig_import_path = zig_module.replace("::", ".");

                    let module_name = zig_module.split("::").last().unwrap_or(zig_module);

                    if import.import_all || import.symbols.is_empty() {
                        // 导入整个模块
                        let import_stmt =
                            format!("const {} = @import(\"{}\");", module_name, zig_import_path);

                        if !self.imported_modules.contains(&module_name.to_string()) {
                            self.line(&import_stmt)?;
                            self.imported_modules.push(module_name.to_string());
                        }
                    } else {
                        // 导入指定符号
                        for (name, alias) in &import.symbols {
                            let import_name = alias.as_ref().unwrap_or(name);
                            let import_stmt = format!(
                                "const {} = @import(\"{}\").{};",
                                import_name, zig_import_path, name
                            );

                            if !self.imported_modules.contains(&import_name.to_string()) {
                                self.line(&import_stmt)?;
                                self.imported_modules.push(import_name.to_string());
                            }
                        }
                    }
                }
            }
            _ => {
                // Handle other declaration types if needed
            }
        }
        Ok(())
    }

    /// 发出语句（来自 LIR）
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> ZigResult<()> {
        match stmt {
            x_lir::Statement::Expression(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                self.line(&format!("{};", expr_str))?;
            }
            x_lir::Statement::Variable(var) => {
                let type_str = self.emit_lir_type(&var.type_);
                if let Some(initializer) = &var.initializer {
                    let init_str = self.emit_lir_expression(initializer)?;
                    self.line(&format!("var {} : {} = {};", var.name, type_str, init_str))?;
                } else {
                    self.line(&format!("var {} : {} = undefined;", var.name, type_str))?;
                }
            }
            x_lir::Statement::If(if_stmt) => {
                let cond_str = self.emit_lir_expression(&if_stmt.condition)?;
                self.line(&format!("if ({}) {{", cond_str))?;
                self.indent();
                self.emit_lir_statement(&if_stmt.then_branch)?;
                self.dedent();

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.line("} else {")?;
                    self.indent();
                    self.emit_lir_statement(else_branch)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            x_lir::Statement::While(while_stmt) => {
                let cond_str = self.emit_lir_expression(&while_stmt.condition)?;
                self.line(&format!("while ({}) {{", cond_str))?;
                self.indent();
                self.emit_lir_statement(&while_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::DoWhile(do_while_stmt) => {
                self.line("while (true) {")?;
                self.indent();
                self.emit_lir_statement(&do_while_stmt.body)?;
                let cond_str = self.emit_lir_expression(&do_while_stmt.condition)?;
                self.line(&format!("if (!{}) break;", cond_str))?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::For(for_stmt) => {
                // Zig doesn't have C-style for loops, so we emulate with while
                if let Some(init) = &for_stmt.initializer {
                    self.emit_lir_statement(init)?;
                }
                let cond_str = for_stmt
                    .condition
                    .as_ref()
                    .map(|e| self.emit_lir_expression(e))
                    .transpose()?
                    .unwrap_or_else(|| "true".to_string());
                self.line(&format!("while ({}) {{", cond_str))?;
                self.indent();
                self.emit_lir_statement(&for_stmt.body)?;
                if let Some(increment) = &for_stmt.increment {
                    let _ = self.emit_lir_expression(increment)?;
                }
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Switch(switch_stmt) => {
                let expr_str = self.emit_lir_expression(&switch_stmt.expression)?;
                self.line(&format!("switch ({}) {{", expr_str))?;
                self.indent();

                for case in &switch_stmt.cases {
                    let value_str = self.emit_lir_expression(&case.value)?;
                    self.line(&format!("{} => {{", value_str))?;
                    self.indent();
                    self.emit_lir_statement(&case.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                if let Some(default) = &switch_stmt.default {
                    self.line("_ => {")?;
                    self.indent();
                    self.emit_lir_statement(default)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Match(match_stmt) => {
                let scrutinee_str = self.emit_lir_expression(&match_stmt.scrutinee)?;
                self.line(&format!("switch ({}) {{", scrutinee_str))?;
                self.indent();

                for case in &match_stmt.cases {
                    let pattern_str = self.emit_lir_pattern(&case.pattern)?;
                    self.line(&format!("{} => {{", pattern_str))?;
                    self.indent();
                    self.emit_lir_block(&case.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Try(try_stmt) => {
                self.line("{")?;
                self.indent();
                self.emit_lir_block(&try_stmt.body)?;
                for catch in &try_stmt.catch_clauses {
                    if let Some(var_name) = &catch.variable_name {
                        self.line(&format!("// catch {}", var_name))?;
                    }
                    self.emit_lir_block(&catch.body)?;
                }
                if let Some(finally) = &try_stmt.finally_block {
                    self.emit_lir_block(finally)?;
                }
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Return(expr) => {
                if let Some(expr) = expr {
                    let expr_str = self.emit_lir_expression(expr)?;
                    self.line(&format!("return {};", expr_str))?;
                } else {
                    self.line("return;")?;
                }
            }
            x_lir::Statement::Break => self.line("break;")?,
            x_lir::Statement::Continue => self.line("continue;")?,
            x_lir::Statement::Goto(label) => self.line(&format!("// goto {}", label))?,
            x_lir::Statement::Label(label) => self.line(&format!("{}: // label", label))?,
            x_lir::Statement::Empty => { /* do nothing */ }
            x_lir::Statement::Compound(block) => {
                self.line("{")?;
                self.indent();
                self.emit_lir_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Declaration(_) => {
                // Already handled at top level - shouldn't happen in LIR block
            }
        }
        Ok(())
    }

    /// 发出表达式（来自 LIR）
    fn emit_lir_expression(&mut self, expr: &x_lir::Expression) -> ZigResult<String> {
        match expr {
            x_lir::Expression::Literal(lit) => match lit {
                x_lir::Literal::Integer(n) => Ok(format!("{}", n)),
                x_lir::Literal::UnsignedInteger(n) => Ok(format!("{}", n)),
                x_lir::Literal::Long(n) => Ok(format!("{}", n)),
                x_lir::Literal::UnsignedLong(n) => Ok(format!("{}", n)),
                x_lir::Literal::LongLong(n) => Ok(format!("{}", n)),
                x_lir::Literal::UnsignedLongLong(n) => Ok(format!("{}", n)),
                x_lir::Literal::Float(f) => Ok(format!("{}", f)),
                x_lir::Literal::Double(f) => Ok(format!("{}", f)),
                x_lir::Literal::String(s) => {
                    let escaped = s
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t");
                    Ok(format!("\"{}\"", escaped))
                }
                x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                x_lir::Literal::NullPointer => Ok("null".to_string()),
            },
            x_lir::Expression::Variable(name) => Ok(name.clone()),
            x_lir::Expression::Unary(op, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let op_str = match op {
                    x_lir::UnaryOp::Plus => "+",
                    x_lir::UnaryOp::Minus => "-",
                    x_lir::UnaryOp::Not => "!",
                    x_lir::UnaryOp::BitNot => "~",
                    x_lir::UnaryOp::PreIncrement => "++",
                    x_lir::UnaryOp::PreDecrement => "--",
                    x_lir::UnaryOp::PostIncrement => "/* post++ */",
                    x_lir::UnaryOp::PostDecrement => "/* post-- */",
                };
                Ok(format!("{}({})", op_str, expr_str))
            }
            x_lir::Expression::Binary(op, lhs, rhs) => {
                let lhs_str = self.emit_lir_expression(lhs)?;
                let rhs_str = self.emit_lir_expression(rhs)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+",
                    x_lir::BinaryOp::Subtract => "-",
                    x_lir::BinaryOp::Multiply => "*",
                    x_lir::BinaryOp::Divide => "/",
                    x_lir::BinaryOp::Modulo => "%",
                    x_lir::BinaryOp::LeftShift => "<<",
                    x_lir::BinaryOp::RightShift => ">>>",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>",
                    x_lir::BinaryOp::LessThan => "<",
                    x_lir::BinaryOp::LessThanEqual => "<=",
                    x_lir::BinaryOp::GreaterThan => ">",
                    x_lir::BinaryOp::GreaterThanEqual => ">=",
                    x_lir::BinaryOp::Equal => "==",
                    x_lir::BinaryOp::NotEqual => "!=",
                    x_lir::BinaryOp::BitAnd => "&",
                    x_lir::BinaryOp::BitXor => "^",
                    x_lir::BinaryOp::BitOr => "|",
                    x_lir::BinaryOp::LogicalAnd => "and",
                    x_lir::BinaryOp::LogicalOr => "or",
                };
                Ok(format!("({} {} {})", lhs_str, op_str, rhs_str))
            }
            x_lir::Expression::Call(callee, args) => {
                let callee_str = self.emit_lir_expression(callee)?;
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|arg| self.emit_lir_expression(arg))
                    .collect::<Result<_, _>>()?;
                Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
            }
            x_lir::Expression::Index(array, index) => {
                let array_str = self.emit_lir_expression(array)?;
                let index_str = self.emit_lir_expression(index)?;
                Ok(format!("{}[{}]", array_str, index_str))
            }
            x_lir::Expression::Member(obj, field) => {
                let obj_str = self.emit_lir_expression(obj)?;
                Ok(format!("{}.{}", obj_str, field))
            }
            x_lir::Expression::Dereference(ptr) => {
                let ptr_str = self.emit_lir_expression(ptr)?;
                Ok(format!("({}.*)", ptr_str))
            }
            x_lir::Expression::AddressOf(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                Ok(format!("&({})", expr_str))
            }
            x_lir::Expression::Cast(type_, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let type_str = self.emit_lir_type(type_);
                Ok(format!("@as({}, {})", type_str, expr_str))
            }
            x_lir::Expression::Assign(lhs, rhs) => {
                let lhs_str = self.emit_lir_expression(lhs)?;
                let rhs_str = self.emit_lir_expression(rhs)?;
                Ok(format!("({} = {})", lhs_str, rhs_str))
            }
            x_lir::Expression::AssignOp(op, lhs, rhs) => {
                let lhs_str = self.emit_lir_expression(lhs)?;
                let rhs_str = self.emit_lir_expression(rhs)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+=",
                    x_lir::BinaryOp::Subtract => "-=",
                    x_lir::BinaryOp::Multiply => "*=",
                    x_lir::BinaryOp::Divide => "/=",
                    x_lir::BinaryOp::Modulo => "%=",
                    x_lir::BinaryOp::BitAnd => "&=",
                    x_lir::BinaryOp::BitOr => "|=",
                    x_lir::BinaryOp::BitXor => "^=",
                    x_lir::BinaryOp::LeftShift => "<<=",
                    x_lir::BinaryOp::RightShift => ">>>=",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>=",
                    _ => "=/* unknown op */",
                };
                Ok(format!("({} {} {})", lhs_str, op_str, rhs_str))
            }
            x_lir::Expression::Ternary(cond, then, else_) => {
                let cond_str = self.emit_lir_expression(cond)?;
                let then_str = self.emit_lir_expression(then)?;
                let else_str = self.emit_lir_expression(else_)?;
                Ok(format!("if ({}) {} else {}", cond_str, then_str, else_str))
            }
            x_lir::Expression::PointerMember(ptr, field) => {
                let ptr_str = self.emit_lir_expression(ptr)?;
                Ok(format!("{}.{}", ptr_str, field))
            }
            x_lir::Expression::SizeOf(ty) => {
                let ty_str = self.emit_lir_type(ty);
                Ok(format!("@sizeOf({})", ty_str))
            }
            x_lir::Expression::SizeOfExpr(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                Ok(format!("@sizeOf({})", expr_str))
            }
            x_lir::Expression::AlignOf(ty) => {
                let ty_str = self.emit_lir_type(ty);
                Ok(format!("@alignOf({})", ty_str))
            }
            x_lir::Expression::Comma(exprs) => {
                let expr_strs: Vec<String> = exprs
                    .iter()
                    .map(|e| self.emit_lir_expression(e))
                    .collect::<Result<_, _>>()?;
                Ok(expr_strs.join(", "))
            }
            x_lir::Expression::Parenthesized(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                Ok(format!("({})", expr_str))
            }
            x_lir::Expression::InitializerList(inits) => {
                // In Zig, this becomes .{ ... }
                let mut init_strs = Vec::new();
                for init in inits {
                    init_strs.push(self.emit_lir_initializer(init)?);
                }
                Ok(format!(".{{ {} }}", init_strs.join(", ")))
            }
            x_lir::Expression::CompoundLiteral(ty, inits) => {
                let ty_str = self.emit_lir_type(ty);
                let mut init_strs = Vec::new();
                for init in inits {
                    init_strs.push(self.emit_lir_initializer(init)?);
                }
                Ok(format!("{} {{ {} }}", ty_str, init_strs.join(", ")))
            }
            _ => {
                // Handle other expression types as needed
                Ok("/* unimplemented expression */".to_string())
            }
        }
    }

    /// 发出初始化器（用于复合字面量）
    fn emit_lir_initializer(&mut self, init: &x_lir::Initializer) -> ZigResult<String> {
        match init {
            x_lir::Initializer::Expression(expr) => self.emit_lir_expression(expr),
            x_lir::Initializer::List(list) => {
                let mut items = Vec::new();
                for i in list {
                    items.push(self.emit_lir_initializer(i)?);
                }
                Ok(format!(".{{ {} }}", items.join(", ")))
            }
            x_lir::Initializer::Named(name, init) => {
                let init_str = self.emit_lir_initializer(init)?;
                Ok(format!(".{} = {}", name, init_str))
            }
            x_lir::Initializer::Indexed(idx, init) => {
                let idx_str = self.emit_lir_expression(idx)?;
                let init_str = self.emit_lir_initializer(init)?;
                Ok(format!("[{}] = {}", idx_str, init_str))
            }
        }
    }

    /// 发出模式（来自 LIR）
    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> ZigResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard => Ok("_".to_string()),
            x_lir::Pattern::Variable(name) => Ok(name.clone()),
            x_lir::Pattern::Literal(lit) => match lit {
                x_lir::Literal::Integer(n) => Ok(format!("{}", n)),
                x_lir::Literal::String(s) => Ok(format!("\"{}\"", s)),
                x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                _ => Ok("_".to_string()),
            },
            x_lir::Pattern::Constructor(name, patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                if pattern_strs.is_empty() {
                    Ok(format!(".{}", name))
                } else {
                    Ok(format!(".{}({})", name, pattern_strs.join(", ")))
                }
            }
            x_lir::Pattern::Tuple(patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                Ok(format!(".{{ {} }}", pattern_strs.join(", ")))
            }
            x_lir::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| {
                        let v_str = self.emit_lir_pattern(v).unwrap_or_else(|_| "_".to_string());
                        format!(".{} = {}", k, v_str)
                    })
                    .collect();
                Ok(format!("{}.{{ {} }}", name, field_strs.join(", ")))
            }
            x_lir::Pattern::Or(left, right) => {
                let left_str = self.emit_lir_pattern(left)?;
                let right_str = self.emit_lir_pattern(right)?;
                Ok(format!("{}, {}", left_str, right_str))
            }
        }
    }

    /// 发出类型（来自 LIR）
    fn emit_lir_type(&self, type_: &x_lir::Type) -> String {
        match type_ {
            x_lir::Type::Void => "void".to_string(),
            x_lir::Type::Bool => "bool".to_string(),
            x_lir::Type::Char => "u8".to_string(),
            x_lir::Type::Schar => "i8".to_string(),
            x_lir::Type::Uchar => "u8".to_string(),
            x_lir::Type::Short => "i16".to_string(),
            x_lir::Type::Ushort => "u16".to_string(),
            x_lir::Type::Int => "i32".to_string(),
            x_lir::Type::Uint => "u32".to_string(),
            x_lir::Type::Long => "i64".to_string(),
            x_lir::Type::Ulong => "u64".to_string(),
            x_lir::Type::LongLong => "i128".to_string(),
            x_lir::Type::UlongLong => "u128".to_string(),
            x_lir::Type::Float => "f32".to_string(),
            x_lir::Type::Double => "f64".to_string(),
            x_lir::Type::LongDouble => "f128".to_string(),
            x_lir::Type::Size => "usize".to_string(),
            x_lir::Type::Ptrdiff => "isize".to_string(),
            x_lir::Type::Intptr => "isize".to_string(),
            x_lir::Type::Uintptr => "usize".to_string(),
            x_lir::Type::Pointer(inner) => format!("*{}", self.emit_lir_type(inner)),
            x_lir::Type::Array(inner, Some(size)) => {
                format!("[{}]{}", size, self.emit_lir_type(inner))
            }
            x_lir::Type::Array(inner, None) => {
                format!("[]{}", self.emit_lir_type(inner))
            }
            x_lir::Type::FunctionPointer(ret_type, param_types) => {
                let param_str = param_types
                    .iter()
                    .map(|t| self.emit_lir_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) {}", param_str, self.emit_lir_type(ret_type))
            }
            x_lir::Type::Named(name) => name.clone(),
            x_lir::Type::Qualified(_, inner) => self.emit_lir_type(inner),
        }
    }
}
