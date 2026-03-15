//! Zig 后端 - 将 X AST 编译为 Zig 代码
//!
//! 利用 Zig 的内存管理和错误处理特性，提供高效的编译输出

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use x_parser::ast::{self, ExpressionKind, StatementKind, Program as AstProgram};
use x_hir;
use x_perceus;

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
    indent: usize,
    output: String,
    /// Track global (top-level) variable declarations for forward decls
    global_vars: Vec<String>,
    /// Track imported Zig modules
    imported_modules: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ZigBackendError {
    #[error("Lowering error: {0}")]
    LoweringError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),
    #[error("Compiler error: {0}")]
    CompilerError(String),
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

pub type ZigResult<T> = Result<T, ZigBackendError>;

impl ZigBackend {
    pub fn new(config: ZigBackendConfig) -> Self {
        Self {
            config,
            indent: 0,
            output: String::new(),
            global_vars: Vec::new(),
            imported_modules: Vec::new(),
        }
    }

    pub fn generate_from_ast(&mut self, program: &AstProgram) -> ZigResult<super::CodegenOutput> {
        self.output.clear();
        self.indent = 0;
        self.global_vars.clear();
        self.imported_modules.clear();

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
            self.emit_main_function()?;
        }

        // Create output file
        let output_file = super::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output.as_bytes().to_vec(),
            file_type: super::FileType::Zig,
        };

        Ok(super::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 从 HIR 生成代码
    pub fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> ZigResult<super::CodegenOutput> {
        self.output.clear();
        self.indent = 0;
        self.global_vars.clear();
        self.imported_modules.clear();

        self.emit_header()?;

        // Emit functions from HIR
        for decl in &hir.declarations {
            if let x_hir::HirDeclaration::Function(func) = decl {
                self.emit_hir_function(func)?;
                self.line("")?;
            }
        }

        // Create output file
        let output_file = super::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output.as_bytes().to_vec(),
            file_type: super::FileType::Zig,
        };

        Ok(super::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 从 PerceusIR 生成代码（带内存管理）
    pub fn generate_from_pir(&mut self, pir: &x_perceus::PerceusIR) -> ZigResult<super::CodegenOutput> {
        self.output.clear();
        self.indent = 0;
        self.global_vars.clear();
        self.imported_modules.clear();

        self.emit_header()?;

        // Import std for memory management
        self.line("const std = @import(\"std\");")?;
        self.line("")?;

        // Emit functions with memory operations from Perceus analysis
        for func_analysis in &pir.functions {
            self.emit_pir_function(func_analysis)?;
            self.line("")?;
        }

        // Create output file
        let output_file = super::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output.as_bytes().to_vec(),
            file_type: super::FileType::Zig,
        };

        Ok(super::CodegenOutput {
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
        self.indent += 1;

        // Emit function body
        self.emit_hir_block(&func.body)?;

        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    /// Emit a function from PerceusIR with memory management
    fn emit_pir_function(&mut self, func: &x_perceus::FunctionAnalysis) -> ZigResult<()> {
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
        self.indent += 1;

        // Emit memory operations
        for mem_op in &func.memory_ops {
            self.emit_memory_op(mem_op)?;
        }

        // Emit control flow
        for block in &func.control_flow.blocks {
            self.emit_basic_block(block)?;
        }

        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    /// Emit a memory operation
    fn emit_memory_op(&mut self, op: &x_perceus::MemoryOp) -> ZigResult<()> {
        match op {
            x_perceus::MemoryOp::Dup { variable, target, .. } => {
                // In Zig, dup is typically allocator.dupe()
                self.line(&format!(
                    "var {} = try allocator.dupe(u8, {});",
                    target, variable
                ))?;
            }
            x_perceus::MemoryOp::Drop { variable, .. } => {
                // In Zig, we use defer for cleanup
                self.line(&format!("defer allocator.free({});", variable))?;
            }
            x_perceus::MemoryOp::Reuse { from, to, .. } => {
                // Memory reuse - reusing memory from one variable to another
                self.line(&format!("var {} = {}; // reuse", to, from))?;
            }
            x_perceus::MemoryOp::Alloc { variable, size, .. } => {
                self.line(&format!(
                    "var {} = try allocator.alloc(u8, {});",
                    variable, size
                ))?;
            }
        }
        Ok(())
    }

    /// Emit a basic block from control flow analysis
    fn emit_basic_block(&mut self, block: &x_perceus::BasicBlock) -> ZigResult<()> {
        self.line(&format!("// Block {} (statements: {:?})", block.id, block.statements))?;

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
            x_hir::HirType::Float => "f64".to_string(),
            x_hir::HirType::Bool => "bool".to_string(),
            x_hir::HirType::String => "[]const u8".to_string(),
            x_hir::HirType::Char => "u8".to_string(),
            x_hir::HirType::Unit => "void".to_string(),
            x_hir::HirType::Never => "noreturn".to_string(),
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
            x_hir::HirType::Generic(name) => name.clone(),
            _ => "anytype".to_string(),
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
                self.indent += 1;
                self.emit_hir_block(&if_stmt.then_block)?;
                self.indent -= 1;
                if let Some(else_block) = &if_stmt.else_block {
                    self.line("} else {")?;
                    self.indent += 1;
                    self.emit_hir_block(else_block)?;
                    self.indent -= 1;
                }
                self.line("}")?;
            }
            x_hir::HirStatement::While(while_stmt) => {
                let cond = self.emit_hir_expression(&while_stmt.condition)?;
                self.line(&format!("while ({}) {{", cond))?;
                self.indent += 1;
                self.emit_hir_block(&while_stmt.body)?;
                self.indent -= 1;
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
                self.indent += 1;
                self.emit_hir_block(&for_stmt.body)?;
                self.indent -= 1;
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
        }
        Ok(())
    }

    /// Emit HIR match statement
    fn emit_hir_match(&mut self, match_stmt: &x_hir::HirMatchStatement) -> ZigResult<()> {
        let expr = self.emit_hir_expression(&match_stmt.expression)?;
        self.line(&format!("switch ({}) {{", expr))?;
        self.indent += 1;

        for case in &match_stmt.cases {
            let pattern_str = self.emit_hir_pattern(&case.pattern)?;
            let condition = if let Some(guard) = &case.guard {
                let guard_str = self.emit_hir_expression(guard)?;
                format!("{} if ({})", pattern_str, guard_str)
            } else {
                pattern_str
            };
            self.line(&format!("{} => {{", condition))?;
            self.indent += 1;
            self.emit_hir_block(&case.body)?;
            self.indent -= 1;
            self.line("},")?;
        }

        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    /// Emit HIR try statement
    fn emit_hir_try(&mut self, try_stmt: &x_hir::HirTryStatement) -> ZigResult<()> {
        self.line("{")?;
        self.indent += 1;
        self.line("var __err: ?anyerror = null;")?;
        self.line("errdefer {")?;
        self.indent += 1;
        self.line("__err = error.UnexpectedError;")?;
        self.indent -= 1;
        self.line("}")?;
        self.emit_hir_block(&try_stmt.body)?;

        if !try_stmt.catch_clauses.is_empty() {
            self.line("if (__err) |err| {")?;
            self.indent += 1;

            for catch in &try_stmt.catch_clauses {
                if let Some(var_name) = &catch.variable_name {
                    self.line(&format!("const {} = err;", var_name))?;
                }
                self.emit_hir_block(&catch.body)?;
            }

            self.indent -= 1;
            self.line("}")?;
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.emit_hir_block(finally)?;
        }

        self.indent -= 1;
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
                let elem_strs: Vec<String> = elements.iter()
                    .map(|p| self.emit_hir_pattern(p))
                    .collect::<ZigResult<Vec<_>>>()?;
                Ok(format!("[{}]", elem_strs.join(", ")))
            }
            x_hir::HirPattern::Tuple(elements) => {
                let elem_strs: Vec<String> = elements.iter()
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
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| {
                        let v_str = self.emit_hir_pattern(v)?;
                        Ok(format!(".{} = {}", k, v_str))
                    })
                    .collect::<ZigResult<Vec<_>>>()?;
                Ok(format!("{}.{{ {} }}", name, field_strs.join(", ")))
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
            x_hir::HirExpression::Literal(lit) => {
                match lit {
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
                }
            }
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

    fn emit_main_function(&mut self) -> ZigResult<()> {
        self.line("pub fn main() !void {")?;
        self.indent += 1;
        self.line("const std = @import(\"std\");")?;
        self.line("const stdout = std.io.getStdOut().writer();")?;
        self.line("")?;
        self.line("// Initialize runtime")?;
        self.line("try stdout.print(\"Hello from Zig backend!\n\", .{});")?;
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_header(&mut self) -> ZigResult<()> {
        self.line("// Generated by X-Lang Zig backend")?;
        self.line("// DO NOT EDIT")?;
        self.line("")?;

        // 不总是导入std，而是在emit_import中处理

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
        let var_type = if let Some(type_annot) = &v.type_annot {
            format!(" : {}", self.emit_type(type_annot))
        } else {
            "".to_string()
        };
        self.line(&format!("var {}{} = {};", v.name, var_type, init))?;
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
                        format!(" : {}", self.emit_type(type_annot))
                    } else {
                        " : anytype".to_string()
                    };
                    format!("{} {}", p.name, param_type)
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let return_type = if let Some(return_type) = &f.return_type {
            format!(" -> {}", self.emit_type(return_type))
        } else {
            "".to_string()
        };
        // Emit async keyword for async functions
        let async_keyword = if f.is_async { "async " } else { "" };
        self.line(&format!(
            "{}fn {}({}){} {{",
            async_keyword, f.name, params, return_type
        ))?;
        self.indent += 1;
        self.emit_block(&f.body)?;
        if f.return_type.is_none() {
            self.line("return;")?;
        }
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    /// Emit a class as a Zig struct
    fn emit_class(&mut self, class: &ast::ClassDecl) -> ZigResult<()> {
        // Emit struct definition
        self.line(&format!("const {} = struct {{", class.name))?;
        self.indent += 1;

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

        self.indent -= 1;
        self.line("};")?;
        self.line("")?;
        Ok(())
    }

    /// Emit a class method as a Zig function
    fn emit_class_method(
        &mut self,
        class_name: &str,
        method: &ast::FunctionDecl,
    ) -> ZigResult<()> {
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
        self.line(&format!("{}fn {}({}){} {{", async_keyword, func_name, params_str, return_type))?;
        self.indent += 1;
        self.emit_block(&method.body)?;
        if method.return_type.is_none() {
            self.line("return;")?;
        }
        self.indent -= 1;
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
        self.indent += 1;

        // Initialize instance using 'this' name to match X language
        self.line(&format!("var this: {} = undefined;", class_name))?;
        self.emit_block(&constructor.body)?;
        self.line("return this;")?;

        self.indent -= 1;
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
                let var_type = if let Some(type_annot) = &v.type_annot {
                    format!(" : {}", self.emit_type(type_annot))
                } else {
                    "".to_string()
                };
                self.line(&format!("var {}{} = {};", v.name, var_type, init))?;
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
                self.indent += 1;
                self.emit_block(&while_stmt.body)?;
                self.indent -= 1;
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
                self.indent += 1;
                self.emit_block(&d.body)?;
                self.indent -= 1;
                let cond = self.emit_expr(&d.condition)?;
                self.line(&format!("}} while ({});", cond))?;
            }
        }
        Ok(())
    }

    fn emit_if(&mut self, if_stmt: &ast::IfStatement) -> ZigResult<()> {
        let cond = self.emit_expr(&if_stmt.condition)?;
        self.line(&format!("if ({}) {{", cond))?;
        self.indent += 1;
        self.emit_block(&if_stmt.then_block)?;
        self.indent -= 1;
        if let Some(else_block) = &if_stmt.else_block {
            self.line("} else {")?;
            self.indent += 1;
            self.emit_block(else_block)?;
            self.indent -= 1;
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
        self.indent += 1;
        self.emit_block(&for_stmt.body)?;
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_match(&mut self, match_stmt: &ast::MatchStatement) -> ZigResult<()> {
        // Zig 使用 switch 语句进行模式匹配
        let expr = self.emit_expr(&match_stmt.expression)?;
        self.line(&format!("switch ({}) {{", expr))?;
        self.indent += 1;

        for case in &match_stmt.cases {
            let pattern_str = self.emit_pattern(&case.pattern)?;

            // 处理 guard 条件
            if let Some(guard) = &case.guard {
                let guard_expr = self.emit_expr(guard)?;
                self.line(&format!("{} if {} => {{", pattern_str, guard_expr))?;
            } else {
                self.line(&format!("{} => {{", pattern_str))?;
            }

            self.indent += 1;
            self.emit_block(&case.body)?;
            self.indent -= 1;
            self.line("},")?;
        }

        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_try(&mut self, try_stmt: &ast::TryStatement) -> ZigResult<()> {
        // Zig 使用 errdefer 和 catch 处理错误
        self.line("{")?;
        self.indent += 1;
        self.line("errdefer {")?;
        self.indent += 1;

        // Emit catch clauses
        for catch in &try_stmt.catch_clauses {
            if let Some(var_name) = &catch.variable_name {
                self.line(&format!("var {} = error{};", var_name, var_name))?;
            }
            self.emit_block(&catch.body)?;
        }

        self.indent -= 1;
        self.line("}")?;

        // Emit try body
        self.emit_block(&try_stmt.body)?;

        // Emit finally block
        if let Some(finally) = &try_stmt.finally_block {
            self.line("defer {")?;
            self.indent += 1;
            self.emit_block(finally)?;
            self.indent -= 1;
            self.line("}")?;
        }

        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_pattern(&self, pattern: &ast::Pattern) -> ZigResult<String> {
        match pattern {
            ast::Pattern::Wildcard => Ok("_".to_string()),
            ast::Pattern::Variable(name) => Ok(name.clone()),
            ast::Pattern::Literal(lit) => self.emit_literal(lit),
            ast::Pattern::Array(patterns) => {
                let items: Vec<String> = patterns.iter().map(|p| self.emit_pattern(p)).collect::<Result<_, _>>()?;
                Ok(format!("[{}]", items.join(", ")))
            }
            ast::Pattern::Tuple(patterns) => {
                let items: Vec<String> = patterns.iter().map(|p| self.emit_pattern(p)).collect::<Result<_, _>>()?;
                Ok(format!(".{{ {} }}", items.join(", ")))
            }
            ast::Pattern::Record(name, fields) => {
                let field_patterns: Vec<String> = fields.iter().map(|(n, p)| {
                    let p_str = self.emit_pattern(p).unwrap_or_else(|_| "_".to_string());
                    format!(".{} = {}", n, p_str)
                }).collect();
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
                let items: Vec<String> = entries.iter().map(|(k, v)| {
                    let k_str = self.emit_pattern(k).unwrap_or_else(|_| "_".to_string());
                    let v_str = self.emit_pattern(v).unwrap_or_else(|_| "_".to_string());
                    format!("{}: {}", k_str, v_str)
                }).collect();
                Ok(format!(".{{ {} }}", items.join(", ")))
            }
        }
    }

    fn emit_expr(&self, expr: &ast::Expression) -> ZigResult<String> {
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
            ExpressionKind::Call(callee, args) => self.emit_call(callee, args),
            ExpressionKind::Assign(target, value) => self.emit_assign(target, value),
            ExpressionKind::Array(elements) => self.emit_array_literal(elements),
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
        }
    }

    fn emit_dict_literal(&self, entries: &[(ast::Expression, ast::Expression)]) -> ZigResult<String> {
        if entries.is_empty() {
            return Ok("std.AutoHashMap(anytype, anytype).init(std.heap.page_allocator)".to_string());
        }
        let entry_strs: Vec<String> = entries
            .iter()
            .map(|(k, v)| {
                let k_str = self.emit_expr(k)?;
                let v_str = self.emit_expr(v)?;
                Ok(format!("try map.put({}, {})", k_str, v_str))
            })
            .collect::<ZigResult<Vec<_>>>()?;
        Ok(format!("blk: {{ var map = std.AutoHashMap(anytype, anytype).init(std.heap.page_allocator); {}; break :blk map; }}", entry_strs.join("; ")))
    }

    fn emit_record_literal(&self, _name: &str, fields: &[(String, ast::Expression)]) -> ZigResult<String> {
        let field_strs: Vec<String> = fields
            .iter()
            .map(|(n, v)| {
                let v_str = self.emit_expr(v)?;
                Ok(format!(".{} = {}", n, v_str))
            })
            .collect::<ZigResult<Vec<_>>>()?;
        Ok(format!(".{{ {} }}", field_strs.join(", ")))
    }

    fn emit_lambda(&self, params: &[ast::Parameter], _body: &ast::Block) -> ZigResult<String> {
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
        // Note: For simplicity, we return a struct with a call method
        // A full implementation would need to handle captures
        Ok(format!(
            "struct {{ fn call({}) void {{ /* lambda body */ }} }}",
            param_strs.join(", ")
        ))
    }

    fn emit_range(&self, start: &ast::Expression, end: &ast::Expression, inclusive: bool) -> ZigResult<String> {
        let s = self.emit_expr(start)?;
        let e = self.emit_expr(end)?;
        if inclusive {
            Ok(format!("{}..{}", s, e))
        } else {
            Ok(format!("{}..{}", s, e))
        }
    }

    fn emit_pipe(&self, input: &ast::Expression, funcs: &[Box<ast::Expression>]) -> ZigResult<String> {
        let mut result = self.emit_expr(input)?;
        for func in funcs {
            let f = self.emit_expr(func)?;
            result = format!("{}({})", f, result);
        }
        Ok(result)
    }

    fn emit_wait(&self, wait_type: &ast::WaitType, exprs: &[ast::Expression]) -> ZigResult<String> {
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
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("await {}", e)).collect();
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
                    let frames: Vec<String> = expr_strs.iter().enumerate()
                        .map(|(i, _)| format!("__frame_{}", i))
                        .collect();
                    let starts: Vec<String> = expr_strs.iter().enumerate()
                        .map(|(i, e)| format!("var {} = async {};", frames[i], e))
                        .collect();
                    let awaits: Vec<String> = frames.iter()
                        .map(|f| format!("await {};", f))
                        .collect();
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
                    let frames: Vec<String> = expr_strs.iter().enumerate()
                        .map(|(i, _)| format!("__race_frame_{}", i))
                        .collect();
                    let starts: Vec<String> = expr_strs.iter().enumerate()
                        .map(|(i, e)| format!("var {} = async {};", frames[i], e))
                        .collect();
                    // In Zig, we would use a suspend/resume pattern for true racing
                    // For simplicity, we await in order and return the first result
                    let awaits: Vec<String> = frames.iter()
                        .map(|f| format!("await {}", f))
                        .collect();
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
        }
    }

    fn emit_literal(&self, lit: &ast::Literal) -> ZigResult<String> {
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

    fn emit_binop(&self, op: &ast::BinaryOp, l: &str, r: &str) -> String {
        match op {
            ast::BinaryOp::Add => format!("{} + {}", l, r),
            ast::BinaryOp::Sub => format!("{} - {}", l, r),
            ast::BinaryOp::Mul => format!("{} * {}", l, r),
            ast::BinaryOp::Div => format!("{} / {}", l, r),
            ast::BinaryOp::Mod => format!("{} % {}", l, r),
            ast::BinaryOp::Equal => format!("{} == {}", l, r),
            ast::BinaryOp::NotEqual => format!("{} != {}", l, r),
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

    fn emit_unaryop(&self, op: &ast::UnaryOp, e: &str) -> String {
        match op {
            ast::UnaryOp::Negate => format!("-{}", e),
            ast::UnaryOp::Not => format!("!{}", e),
            _ => format!("/* unsupported unary {:?} */ null", op),
        }
    }

    fn emit_call(&self, callee: &ast::Expression, args: &[ast::Expression]) -> ZigResult<String> {
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

    fn emit_builtin_or_call(&self, name: &str, args: &[String]) -> String {
        match name {
            "print" | "println" => {
                if args.len() == 1 {
                    format!("std.debug.print(\"{{}}\n\", .{{{}}})", args[0])
                } else {
                    "std.debug.print(\"\n\", .{{}})".to_string()
                }
            }
            "print_inline" => {
                if args.len() == 1 {
                    format!("std.debug.print(\"{{}}\", .{{{}}})", args[0])
                } else {
                    "std.debug.print(\"\", .{{}})".to_string()
                }
            }
            "concat" => {
                if args.len() == 2 {
                    format!(
                        "std.mem.concat(u8, &[_][]const u8{{ {}, {} }})",
                        args[0], args[1]
                    )
                } else {
                    "\"\"".to_string()
                }
            }
            "to_string" => format!(
                "std.fmt.allocPrint(std.heap.page_allocator, \"{{}}\", .{{{}}}) catch unreachable",
                args.first().unwrap_or(&"null".to_string())
            ),
            "type_of" => format!(
                "@typeName(@TypeOf({}))",
                args.first().unwrap_or(&"null".to_string())
            ),
            "panic" => {
                if args.len() == 1 {
                    format!("std.debug.panic(\"{{}}\", .{{{}}})", args[0])
                } else {
                    "std.debug.panic(\"panic\", .{{}})".to_string()
                }
            }
            "len" => format!("{}.len", args.first().unwrap_or(&"null".to_string())),
            _ => {
                format!("{}({})", name, args.join(", "))
            }
        }
    }

    fn emit_assign(&self, target: &ast::Expression, value: &ast::Expression) -> ZigResult<String> {
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

    fn emit_array_literal(&self, elements: &[ast::Expression]) -> ZigResult<String> {
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
            ast::Type::Float => "f64".to_string(),
            ast::Type::Bool => "bool".to_string(),
            ast::Type::String => "[]const u8".to_string(),
            ast::Type::Char => "u8".to_string(),
            ast::Type::Unit => "void".to_string(),
            ast::Type::Never => "noreturn".to_string(),
            ast::Type::Array(inner) => format!("[] {}", self.emit_type(inner)),
            ast::Type::Dictionary(key, value) => format!(
                "std.AutoHashMap({}, {})",
                self.emit_type(key),
                self.emit_type(value)
            ),
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
            ast::Type::Generic(name) | ast::Type::TypeParam(name) | ast::Type::Var(name) => name.clone(),
            ast::Type::Async(inner) => self.emit_type(inner),
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
        for _ in 0..self.indent {
            write!(self.output, "    ")?;
        }
        writeln!(self.output, "{}", s)?;
        Ok(())
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

        if self.config.debug_info {
            cmd.arg("-g");
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
            return Err(ZigBackendError::CompilerError(format!(
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
    use x_parser::ast::spanned;

    #[test]
    fn test_hello_world_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
            span: Span::default(),
                name: "main".to_string(),
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Expression(spanned(
                            ExpressionKind::Call(
                                Box::new(spanned(ExpressionKind::Variable("print".to_string()), Span::default())),
                                vec![spanned(ExpressionKind::Literal(ast::Literal::String(
                                    "Hello, World!".to_string(),
                                )), Span::default())],
                            ),
                            Span::default(),
                        )),
                        Span::default(),
                    )],
                },
                is_async: false,
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
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::For(ast::ForStatement {
                            pattern: ast::Pattern::Variable("i".to_string()),
                            iterator: spanned(ExpressionKind::Array(vec![
                                spanned(ExpressionKind::Literal(ast::Literal::Integer(1)), Span::default()),
                                spanned(ExpressionKind::Literal(ast::Literal::Integer(2)), Span::default()),
                                spanned(ExpressionKind::Literal(ast::Literal::Integer(3)), Span::default()),
                            ]), Span::default()),
                            body: ast::Block {
                                statements: vec![spanned(
                                    StatementKind::Expression(spanned(
                                        ExpressionKind::Call(
                                            Box::new(spanned(ExpressionKind::Variable("print".to_string()), Span::default())),
                                            vec![spanned(ExpressionKind::Variable("i".to_string()), Span::default())],
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
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Match(ast::MatchStatement {
                            expression: spanned(ExpressionKind::Literal(ast::Literal::Integer(1)), Span::default()),
                            cases: vec![
                                ast::MatchCase {
                                    pattern: ast::Pattern::Literal(ast::Literal::Integer(1)),
                                    body: ast::Block {
                                        statements: vec![spanned(
                                            StatementKind::Expression(spanned(
                                                ExpressionKind::Call(
                                                    Box::new(spanned(ExpressionKind::Variable("print".to_string()), Span::default())),
                                                    vec![spanned(ExpressionKind::Literal(ast::Literal::String("one".to_string())), Span::default())],
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
                                                    Box::new(spanned(ExpressionKind::Variable("print".to_string()), Span::default())),
                                                    vec![spanned(ExpressionKind::Literal(ast::Literal::String("other".to_string())), Span::default())],
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
                parameters: vec![],
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
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Try(ast::TryStatement {
                            body: ast::Block {
                                statements: vec![spanned(
                                    StatementKind::Expression(spanned(
                                        ExpressionKind::Call(
                                            Box::new(spanned(ExpressionKind::Variable("risky_operation".to_string()), Span::default())),
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
                                                Box::new(spanned(ExpressionKind::Variable("print".to_string()), Span::default())),
                                                vec![spanned(ExpressionKind::Variable("e".to_string()), Span::default())],
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
                                            Box::new(spanned(ExpressionKind::Variable("print".to_string()), Span::default())),
                                            vec![spanned(ExpressionKind::Literal(ast::Literal::String(
                                                "cleanup".to_string(),
                                            )), Span::default())],
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
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Variable(ast::VariableDecl {
            span: Span::default(),
                            name: "person".to_string(),
                            is_mutable: false,
                            type_annot: None,
                            initializer: Some(spanned(
                                ExpressionKind::Record(
                                    "Person".to_string(),
                                    vec![
                                        ("name".to_string(), spanned(ExpressionKind::Literal(ast::Literal::String("Alice".to_string())), Span::default())),
                                        ("age".to_string(), spanned(ExpressionKind::Literal(ast::Literal::Integer(30)), Span::default())),
                                    ],
                                ),
                                Span::default(),
                            )),
                        }),
                        Span::default(),
                    )],
                },
                is_async: false,
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
        use x_hir::{Hir, HirTypeEnv, HirPerceusInfo};

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
        use x_perceus::{PerceusIR, ReuseAnalysis};

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
        use x_perceus::{FunctionAnalysis, MemoryOp, OwnershipFact, OwnershipState, PerceusIR, ReuseAnalysis, ControlFlowAnalysis, BasicBlock, SourcePos};

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
                parameters: vec![],
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
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Expression(spanned(
                            ExpressionKind::Wait(
                                ast::WaitType::Single,
                                vec![spanned(
                                    ExpressionKind::Call(
                                        Box::new(spanned(ExpressionKind::Variable("fetch".to_string()), Span::default())),
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
                parameters: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![spanned(
                        StatementKind::Expression(spanned(
                            ExpressionKind::Wait(
                                ast::WaitType::Together,
                                vec![
                                    spanned(ExpressionKind::Variable("task1".to_string()), Span::default()),
                                    spanned(ExpressionKind::Variable("task2".to_string()), Span::default()),
                                ],
                            ),
                            Span::default(),
                        )),
                        Span::default(),
                    )],
                },
                is_async: true,
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
