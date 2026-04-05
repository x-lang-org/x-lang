//! Rust 后端 - 将 X AST 编译为 Rust 1.94+ 代码
//!
//! 生成清晰可读的 Rust 源代码，支持基本的 X 语言特性
//!
//! ## Rust 1.94 特性支持 (2026年3月发布)
//! - async fn in traits（trait 中的异步函数）
//! - return-position impl Trait in trait bodies
//! - if-let chains（let-else 链）
//! - Slice patterns
//! - Generic Associated Types (GATs)
//! - Const generics
//! - LazyCell / LazyLock in std
//! - Async closures
//! - Return position impl Trait in trait
//! - Improved trait solving

#![allow(
    clippy::collapsible_if,
    clippy::format_in_format_args,
    clippy::only_used_in_recursion,
    clippy::option_as_ref_deref,
    clippy::single_char_add_str,
    clippy::unnecessary_map_or,
    clippy::useless_asref,
    clippy::useless_format,
    clippy::useless_vec
)]

use std::collections::HashMap;
use std::path::PathBuf;
use x_codegen::{headers, CodegenOutput, OutputFile};
use x_lir::{self, Program as LirProgram};
use x_parser::ast::{
    self, BinaryOp, ExpressionKind, Program as AstProgram, StatementKind, UnaryOp,
};

#[derive(Debug, Clone)]
pub struct RustBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl Default for RustBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
        }
    }
}

/// Type information for variables during code generation
#[derive(Debug, Clone, PartialEq)]
enum VarType {
    Int,
    Float,
    Bool,
    String,
    UnsignedInt,
    Unknown,
}

pub struct RustBackend {
    #[allow(dead_code)]
    config: RustBackendConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
    /// Track variable types from type annotations
    var_types: HashMap<String, VarType>,
    /// Track current class fields (for resolving implicit self access)
    current_class_fields: Vec<String>,
    /// Track local variables/parameters that shadow fields
    local_vars: std::collections::HashSet<String>,
    /// Whether the last expression should be returned (not terminated with semicolon)
    #[allow(dead_code)]
    last_expr_is_return: bool,
}

pub type RustResult<T> = Result<T, x_codegen::CodeGenError>;

impl RustBackend {
    pub fn new(config: RustBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
            var_types: HashMap::new(),
            current_class_fields: Vec::new(),
            local_vars: std::collections::HashSet::new(),
            last_expr_is_return: false,
        }
    }

    fn line(&mut self, s: &str) -> RustResult<()> {
        self.buffer
            .line(s)
            .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))
    }

    fn indent(&mut self) {
        self.buffer.indent();
    }
    fn dedent(&mut self) {
        self.buffer.dedent();
    }
    fn output(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> RustResult<x_codegen::CodegenOutput> {
        self.buffer.clear();
        self.var_types.clear();

        // Collect variable types from type annotations
        self.collect_var_types(program);

        // Check if we need dynamic type support
        let needs_dynamic = self.needs_dynamic_types(program);

        self.emit_header()?;

        // Single pass to categorize declarations
        let mut extern_funcs = Vec::new();
        let mut classes = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut global_vars = Vec::new();
        let mut functions = Vec::new();

        for decl in &program.declarations {
            match decl {
                ast::Declaration::ExternFunction(ext) => extern_funcs.push(ext),
                ast::Declaration::Class(class) => classes.push(class),
                ast::Declaration::Enum(enum_decl) => enums.push(enum_decl),
                ast::Declaration::Trait(trait_decl) => traits.push(trait_decl),
                ast::Declaration::Variable(v) => {
                    if v.visibility == ast::Visibility::Public {
                        global_vars.push(v);
                    }
                }
                ast::Declaration::Function(f) => functions.push(f),
                _ => {}
            }
        }

        // Emit extern functions
        for ext in &extern_funcs {
            self.emit_extern_function(ext)?;
        }

        // Emit use statements
        self.emit_uses()?;

        // Emit dynamic value enum if needed
        if needs_dynamic {
            self.emit_dynamic_value_enum()?;
        }

        // Emit structs/enums/traits
        for class in &classes {
            self.emit_struct(class)?;
        }
        for enum_decl in &enums {
            self.emit_enum(enum_decl)?;
        }
        for trait_decl in &traits {
            self.emit_trait(trait_decl)?;
        }

        // Emit global statics/consts
        for v in &global_vars {
            self.emit_global_var(v)?;
        }

        // Emit functions
        let mut has_main = false;
        for f in &functions {
            self.emit_function(f)?;
            self.line("")?;
            if f.name == "main" {
                has_main = true;
            }
        }

        // Emit main function only if not already defined
        if !has_main {
            self.emit_main_function(program)?;
        }

        // Create output file
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("output.rs"),
            content: self.output().as_bytes().to_vec(),
            file_type: x_codegen::FileType::Rust,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    fn emit_header(&mut self) -> RustResult<()> {
        self.line(headers::RUST)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: Rust 1.94 (March 2026)")?;
        self.line("#![allow(unused)]")?;
        self.line("#![allow(dead_code)]")?;
        self.line("#![allow(clippy::all)]")?;
        self.line("")?;
        Ok(())
    }

    fn emit_uses(&mut self) -> RustResult<()> {
        self.line("use std::collections::HashMap;")?;
        self.line("use std::fmt;")?;
        self.line("")?;
        Ok(())
    }

    fn emit_dynamic_value_enum(&mut self) -> RustResult<()> {
        self.line("#[derive(Debug, Clone, PartialEq)]")?;
        self.line("pub enum DynamicValue {")?;
        self.indent();
        self.line("Int(i64),")?;
        self.line("Float(f64),")?;
        self.line("Bool(bool),")?;
        self.line("String(String),")?;
        self.line("Array(Vec<DynamicValue>),")?;
        self.line("Map(HashMap<String, DynamicValue>),")?;
        self.line("Null,")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        // Implement Display for DynamicValue
        self.line("impl fmt::Display for DynamicValue {")?;
        self.indent();
        self.line("fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {")?;
        self.indent();
        self.line("match self {")?;
        self.indent();
        self.line("DynamicValue::Int(n) => write!(f, \"{}\", n),")?;
        self.line("DynamicValue::Float(n) => write!(f, \"{}\", n),")?;
        self.line("DynamicValue::Bool(b) => write!(f, \"{}\", b),")?;
        self.line("DynamicValue::String(s) => write!(f, \"{}\", s),")?;
        self.line("DynamicValue::Array(arr) => {")?;
        self.indent();
        self.line("write!(f, \"[\")?;")?;
        self.line("for (i, v) in arr.iter().enumerate() {")?;
        self.indent();
        self.line("if i > 0 { write!(f, \", \")?; }")?;
        self.line("write!(f, \"{}\", v)?;")?;
        self.dedent();
        self.line("}")?;
        self.line("write!(f, \"]\")")?;
        self.dedent();
        self.line("}")?;
        self.line("DynamicValue::Map(m) => {")?;
        self.indent();
        self.line("write!(f, \"{{\")?;")?;
        self.line("for (i, (k, v)) in m.iter().enumerate() {")?;
        self.indent();
        self.line("if i > 0 { write!(f, \", \")?; }")?;
        self.line("write!(f, \"{}: {}\", k, v)?;")?;
        self.dedent();
        self.line("}")?;
        self.line("write!(f, \"}}\")")?;
        self.dedent();
        self.line("}")?;
        self.line("DynamicValue::Null => write!(f, \"null\"),")?;
        self.dedent();
        self.line("}")?;
        self.dedent();
        self.line("}")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Check if the program needs dynamic type support (mixed-type arrays/dicts)
    fn needs_dynamic_types(&self, program: &AstProgram) -> bool {
        // Check declarations
        for decl in &program.declarations {
            if self.decl_needs_dynamic(decl) {
                return true;
            }
        }
        // Check statements
        for stmt in &program.statements {
            if self.stmt_needs_dynamic(stmt) {
                return true;
            }
        }
        false
    }

    fn decl_needs_dynamic(&self, decl: &ast::Declaration) -> bool {
        match decl {
            ast::Declaration::Function(f) => {
                f.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
            }
            ast::Declaration::Variable(v) => v
                .initializer
                .as_ref()
                .map_or(false, |e| self.expr_needs_dynamic(e)),
            ast::Declaration::Class(c) => c.members.iter().any(|m| match m {
                ast::ClassMember::Field(f) => f
                    .initializer
                    .as_ref()
                    .map_or(false, |e| self.expr_needs_dynamic(e)),
                ast::ClassMember::Method(m) => {
                    m.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
                }
                ast::ClassMember::Constructor(c) => {
                    c.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
                }
            }),
            ast::Declaration::Enum(_)
            | ast::Declaration::Trait(_)
            | ast::Declaration::ExternFunction(_) => false,
            ast::Declaration::TypeAlias(_)
            | ast::Declaration::Module(_)
            | ast::Declaration::Import(_)
            | ast::Declaration::Export(_) => false,
            _ => false,
        }
    }

    fn expr_needs_dynamic(&self, expr: &ast::Expression) -> bool {
        match &expr.node {
            ExpressionKind::Array(elements) => {
                // Check if elements have different types
                let types: std::collections::HashSet<String> =
                    elements.iter().map(|e| self.expr_type_name(e)).collect();
                types.len() > 1 || elements.iter().any(|e| self.expr_needs_dynamic(e))
            }
            ExpressionKind::Tuple(elements) => {
                // Check if elements have different types
                let types: std::collections::HashSet<String> =
                    elements.iter().map(|e| self.expr_type_name(e)).collect();
                types.len() > 1 || elements.iter().any(|e| self.expr_needs_dynamic(e))
            }
            ExpressionKind::Dictionary(entries) => {
                // Check if values have different types
                let value_types: std::collections::HashSet<String> = entries
                    .iter()
                    .map(|(_, v)| self.expr_type_name(v))
                    .collect();
                value_types.len() > 1
                    || entries.iter().any(|(_, v)| self.expr_needs_dynamic(v))
                    || entries.iter().any(|(k, _)| self.expr_needs_dynamic(k))
            }
            ExpressionKind::Call(_, args) => args.iter().any(|a| self.expr_needs_dynamic(a)),
            ExpressionKind::Member(obj, _) => self.expr_needs_dynamic(obj),
            ExpressionKind::Binary(_, left, right) => {
                self.expr_needs_dynamic(left) || self.expr_needs_dynamic(right)
            }
            ExpressionKind::Unary(_, e) => self.expr_needs_dynamic(e),
            ExpressionKind::Cast(e, _) => self.expr_needs_dynamic(e),
            ExpressionKind::If(_, then, else_) => {
                self.expr_needs_dynamic(then) || self.expr_needs_dynamic(else_)
            }
            ExpressionKind::Lambda(_, body) => {
                body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
            }
            ExpressionKind::Record(_, fields) => {
                fields.iter().any(|(_, v)| self.expr_needs_dynamic(v))
            }
            ExpressionKind::Pipe(input, stages) => {
                self.expr_needs_dynamic(input) || stages.iter().any(|s| self.expr_needs_dynamic(s))
            }
            ExpressionKind::Assign(_, v) => self.expr_needs_dynamic(v),
            ExpressionKind::Parenthesized(inner) => self.expr_needs_dynamic(inner),
            ExpressionKind::Wait(_, exprs) => exprs.iter().any(|e| self.expr_needs_dynamic(e)),
            ExpressionKind::TryPropagate(e) => self.expr_needs_dynamic(e),
            ExpressionKind::Match(discriminant, cases) => {
                self.expr_needs_dynamic(discriminant)
                    || cases.iter().any(|c| {
                        c.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
                            || c.guard
                                .as_ref()
                                .map_or(false, |g| self.expr_needs_dynamic(g))
                    })
            }
            ExpressionKind::Handle(e, handlers) => {
                self.expr_needs_dynamic(e)
                    || handlers.iter().any(|(_, h)| self.expr_needs_dynamic(h))
            }
            ExpressionKind::Needs(_) | ExpressionKind::Given(_, _) => false,
            ExpressionKind::Literal(_) | ExpressionKind::Variable(_) => false,
            ExpressionKind::Range(_, _, _) => false,
            ExpressionKind::Await(e) => self.expr_needs_dynamic(e),
            ExpressionKind::OptionalChain(base, _) => self.expr_needs_dynamic(base),
            ExpressionKind::NullCoalescing(left, right) => {
                self.expr_needs_dynamic(left) || self.expr_needs_dynamic(right)
            }
        }
    }

    fn stmt_needs_dynamic(&self, stmt: &ast::Statement) -> bool {
        match &stmt.node {
            StatementKind::Expression(e) => self.expr_needs_dynamic(e),
            StatementKind::Variable(v) => v
                .initializer
                .as_ref()
                .map_or(false, |e| self.expr_needs_dynamic(e)),
            StatementKind::Return(e) => e.as_ref().map_or(false, |e| self.expr_needs_dynamic(e)),
            StatementKind::If(if_stmt) => {
                self.expr_needs_dynamic(&if_stmt.condition)
                    || if_stmt
                        .then_block
                        .statements
                        .iter()
                        .any(|s| self.stmt_needs_dynamic(s))
                    || if_stmt.else_block.as_ref().map_or(false, |b| {
                        b.statements.iter().any(|s| self.stmt_needs_dynamic(s))
                    })
            }
            StatementKind::While(w) => {
                self.expr_needs_dynamic(&w.condition)
                    || w.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
            }
            StatementKind::For(f) => {
                self.expr_needs_dynamic(&f.iterator)
                    || f.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
            }
            StatementKind::Match(m) => {
                self.expr_needs_dynamic(&m.expression)
                    || m.cases
                        .iter()
                        .any(|c| c.body.statements.iter().any(|s| self.stmt_needs_dynamic(s)))
            }
            StatementKind::Try(t) => {
                t.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
                    || t.catch_clauses
                        .iter()
                        .any(|c| c.body.statements.iter().any(|s| self.stmt_needs_dynamic(s)))
                    || t.finally_block.as_ref().map_or(false, |b| {
                        b.statements.iter().any(|s| self.stmt_needs_dynamic(s))
                    })
            }
            StatementKind::DoWhile(d) => {
                self.expr_needs_dynamic(&d.condition)
                    || d.body.statements.iter().any(|s| self.stmt_needs_dynamic(s))
            }
            StatementKind::Unsafe(b) => b.statements.iter().any(|s| self.stmt_needs_dynamic(s)),
            StatementKind::Break | StatementKind::Continue => false,
            StatementKind::Defer(e) => self.expr_needs_dynamic(e),
            StatementKind::Yield(opt_e) => {
                opt_e.as_ref().map_or(false, |e| self.expr_needs_dynamic(e))
            }
            StatementKind::Loop(b) => b.statements.iter().any(|s| self.stmt_needs_dynamic(s)),
        }
    }

    /// Collect variable types from type annotations in the program
    fn collect_var_types(&mut self, program: &AstProgram) {
        // Collect from declarations
        for decl in &program.declarations {
            if let ast::Declaration::Variable(v) = decl {
                if let Some(type_annot) = &v.type_annot {
                    let var_type = match type_annot {
                        ast::Type::Int => VarType::Int,
                        ast::Type::UnsignedInt => VarType::UnsignedInt,
                        ast::Type::Float => VarType::Float,
                        ast::Type::Bool => VarType::Bool,
                        ast::Type::String => VarType::String,
                        _ => VarType::Unknown,
                    };
                    self.var_types.insert(v.name.clone(), var_type);
                }
            }
        }
        // Collect from local variables in statements (will be added to main)
        for stmt in &program.statements {
            if let StatementKind::Variable(v) = &stmt.node {
                if let Some(type_annot) = &v.type_annot {
                    let var_type = match type_annot {
                        ast::Type::Int => VarType::Int,
                        ast::Type::UnsignedInt => VarType::UnsignedInt,
                        ast::Type::Float => VarType::Float,
                        ast::Type::Bool => VarType::Bool,
                        ast::Type::String => VarType::String,
                        _ => VarType::Unknown,
                    };
                    self.var_types.insert(v.name.clone(), var_type);
                }
            }
        }
    }

    /// Get the type of a variable from tracked types
    fn get_var_type(&self, name: &str) -> VarType {
        self.var_types
            .get(name)
            .cloned()
            .unwrap_or(VarType::Unknown)
    }

    fn expr_type_name(&self, expr: &ast::Expression) -> String {
        match &expr.node {
            ExpressionKind::Literal(lit) => match lit {
                ast::Literal::Integer(_) => "int".to_string(),
                ast::Literal::Float(_) => "float".to_string(),
                ast::Literal::Boolean(_) => "bool".to_string(),
                ast::Literal::String(_) => "string".to_string(),
                ast::Literal::Char(_) => "char".to_string(),
                ast::Literal::Null | ast::Literal::None => "null".to_string(),
                ast::Literal::Unit => "unit".to_string(),
            },
            ExpressionKind::Array(_) => "array".to_string(),
            ExpressionKind::Tuple(_) => "tuple".to_string(),
            ExpressionKind::Dictionary(_) => "dict".to_string(),
            ExpressionKind::Variable(name) => {
                // Look up the tracked variable type
                match self.get_var_type(name) {
                    VarType::Int => "int".to_string(),
                    VarType::Float => "float".to_string(),
                    VarType::Bool => "bool".to_string(),
                    VarType::String => "string".to_string(),
                    VarType::UnsignedInt => "uint".to_string(),
                    VarType::Unknown => "unknown".to_string(),
                }
            }
            _ => "unknown".to_string(),
        }
    }

    fn emit_extern_function(&mut self, ext: &ast::ExternFunctionDecl) -> RustResult<()> {
        let params: Vec<String> = ext
            .parameters
            .iter()
            .map(|p| {
                let ty = if let Some(t) = &p.type_annot {
                    self.emit_type(t)
                } else {
                    "std::ffi::c_void".to_string()
                };
                format!("{}: {}", p.name, ty)
            })
            .collect();

        let return_type = if let Some(rt) = &ext.return_type {
            self.emit_type(rt)
        } else {
            "()".to_string()
        };

        self.line(&format!("#[link(name = \"{}\")]", ext.abi.to_lowercase()))?;
        self.line("extern \"C\" {")?;
        self.indent();
        self.line(&format!(
            "fn {}({}) -> {};",
            ext.name,
            params.join(", "),
            return_type
        ))?;
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    fn emit_struct(&mut self, class: &ast::ClassDecl) -> RustResult<()> {
        let derives = vec!["Debug", "Clone", "PartialEq"];
        self.line(&format!("#[derive({})]", derives.join(", ")))?;

        self.line(&format!("pub struct {} {{", class.name))?;
        self.indent();

        // Collect field names for later use in methods
        let mut field_names: Vec<String> = Vec::new();

        for member in &class.members {
            if let ast::ClassMember::Field(field) = member {
                let ty = if let Some(t) = &field.type_annot {
                    self.emit_type(t)
                } else if let Some(init) = &field.initializer {
                    self.infer_type_from_expr(init)
                } else {
                    "()".to_string()
                };
                let visibility = if field.visibility == ast::Visibility::Public {
                    "pub "
                } else {
                    ""
                };
                self.line(&format!("{}{}: {},", visibility, field.name, ty))?;
                field_names.push(field.name.clone());
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;

        // Emit impl block for methods and constructor
        let has_methods = class
            .members
            .iter()
            .any(|m| matches!(m, ast::ClassMember::Method(_)));
        let has_constructor = class
            .members
            .iter()
            .any(|m| matches!(m, ast::ClassMember::Constructor(_)));

        if has_methods || has_constructor {
            // Set current class fields for resolving implicit self access
            self.current_class_fields = field_names.clone();

            self.line(&format!("impl {} {{", class.name))?;
            self.indent();

            // Emit constructor
            for member in &class.members {
                if let ast::ClassMember::Constructor(constructor) = member {
                    self.emit_constructor(constructor, &class.name, &field_names)?;
                }
            }

            // Emit methods
            for member in &class.members {
                if let ast::ClassMember::Method(method) = member {
                    self.emit_method(method)?;
                    self.line("")?;
                }
            }

            self.dedent();
            self.line("}")?;
            self.line("")?;

            // Clear current class fields
            self.current_class_fields.clear();
        }

        // Emit trait implementations
        for trait_name in &class.implements {
            self.line(&format!("impl {} for {} {{", trait_name, class.name))?;
            self.indent();

            // Emit methods that implement the trait
            for member in &class.members {
                if let ast::ClassMember::Method(method) = member {
                    self.current_class_fields = field_names.clone();
                    self.emit_method(method)?;
                    self.line("")?;
                    self.current_class_fields.clear();
                }
            }

            self.dedent();
            self.line("}")?;
            self.line("")?;
        }

        Ok(())
    }

    fn emit_constructor(
        &mut self,
        constructor: &ast::ConstructorDecl,
        _class_name: &str,
        field_names: &[String],
    ) -> RustResult<()> {
        // Add parameters to local vars to avoid prefixing with self
        for p in &constructor.parameters {
            self.local_vars.insert(p.name.clone());
        }

        let params: Vec<String> = constructor
            .parameters
            .iter()
            .map(|p| {
                let ty = if let Some(t) = &p.type_annot {
                    self.emit_type(t)
                } else {
                    "_".to_string()
                };
                format!("{}: {}", p.name, ty)
            })
            .collect();

        self.line(&format!("pub fn new({}) -> Self {{", params.join(", ")))?;
        self.indent();

        // Emit statements that are not `this.field = value` assignments
        // (those are handled by the struct literal return)
        for stmt in &constructor.body.statements {
            if let StatementKind::Expression(expr) = &stmt.node {
                if let ExpressionKind::Assign(target, _value) = &expr.node {
                    // Skip `this.field = value` assignments
                    if let ExpressionKind::Member(obj, _name) = &target.node {
                        if let ExpressionKind::Variable(var) = &obj.node {
                            if var == "this" {
                                continue; // Skip this assignment
                            }
                        }
                    }
                }
            }
            self.emit_statement(stmt)?;
        }

        // Generate return statement with initialized fields
        let fields_init: Vec<String> = field_names
            .iter()
            .map(|f| format!("{}: {}", f, f))
            .collect();
        self.line(&format!("Self {{ {} }}", fields_init.join(", ")))?;

        self.dedent();
        self.line("}")?;
        self.line("")?;

        // Remove parameters from local vars
        for p in &constructor.parameters {
            self.local_vars.remove(&p.name);
        }

        Ok(())
    }

    fn emit_method(&mut self, method: &ast::FunctionDecl) -> RustResult<()> {
        // Add parameters to local vars
        for p in &method.parameters {
            self.local_vars.insert(p.name.clone());
        }

        let mut params = vec!["&self".to_string()];
        for p in &method.parameters {
            let ty = if let Some(t) = &p.type_annot {
                self.emit_type(t)
            } else {
                "_".to_string()
            };
            params.push(format!("{}: {}", p.name, ty));
        }

        let return_type = if let Some(rt) = &method.return_type {
            format!(" -> {}", self.emit_type(rt))
        } else {
            "".to_string()
        };

        let visibility = if method.modifiers.visibility == ast::Visibility::Public {
            "pub "
        } else {
            ""
        };

        let async_keyword = if method.is_async { "async " } else { "" };

        self.line(&format!(
            "{}{}fn {}({}){} {{",
            visibility,
            async_keyword,
            method.name,
            params.join(", "),
            return_type
        ))?;
        self.indent();

        // Check if last statement is an expression (implicit return)
        // Only treat as implicit return if there's a declared return type
        let has_return_type = method.return_type.is_some();
        let last_is_expr = !method.body.statements.is_empty()
            && matches!(
                method.body.statements.last().unwrap().node,
                StatementKind::Expression(_)
            );

        // Emit all statements
        let stmt_count = method.body.statements.len();
        for (i, stmt) in method.body.statements.iter().enumerate() {
            let is_last = i == stmt_count - 1;
            // If this is the last statement, it's an expression, AND there's a return type,
            // emit without semicolon (implicit return)
            // Otherwise emit with semicolon (as a statement)
            if is_last && last_is_expr && has_return_type {
                if let StatementKind::Expression(expr) = &stmt.node {
                    let e = self.emit_expr(expr)?;
                    self.line(&format!("{}", e))?; // No semicolon - implicit return
                } else {
                    self.emit_statement(stmt)?;
                }
            } else {
                self.emit_statement(stmt)?;
            }
        }

        // Add default return only if there's a return type but no statements/last isn't expr
        if has_return_type && !last_is_expr {
            self.line("Default::default()")?;
        }

        self.dedent();
        self.line("}")?;

        // Remove parameters from local vars
        for p in &method.parameters {
            self.local_vars.remove(&p.name);
        }

        Ok(())
    }

    fn emit_enum(&mut self, enum_decl: &ast::EnumDecl) -> RustResult<()> {
        self.line("#[derive(Debug, Clone, PartialEq)]")?;

        // Handle type parameters
        let type_params = if enum_decl.type_parameters.is_empty() {
            String::new()
        } else {
            let params: Vec<String> = enum_decl
                .type_parameters
                .iter()
                .map(|p| p.name.clone())
                .collect();
            format!("<{}>", params.join(", "))
        };

        self.line(&format!("pub enum {}{} {{", enum_decl.name, type_params))?;
        self.indent();

        for variant in &enum_decl.variants {
            match &variant.data {
                ast::EnumVariantData::Unit => {
                    self.line(&format!("{},", variant.name))?;
                }
                ast::EnumVariantData::Tuple(types) => {
                    let type_strs: Vec<String> = types.iter().map(|t| self.emit_type(t)).collect();
                    self.line(&format!("{}({}),", variant.name, type_strs.join(", ")))?;
                }
                ast::EnumVariantData::Record(fields) => {
                    self.line(&format!("{} {{", variant.name))?;
                    self.indent();
                    for (name, ty) in fields {
                        self.line(&format!("{}: {},", name, self.emit_type(ty)))?;
                    }
                    self.dedent();
                    self.line("},")?;
                }
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    fn emit_trait(&mut self, trait_decl: &ast::TraitDecl) -> RustResult<()> {
        self.line(&format!("pub trait {} {{", trait_decl.name))?;
        self.indent();

        for method in &trait_decl.methods {
            // Check if this is a static method or instance method
            // In X, methods without explicit parameters are instance methods (have implicit self)
            let is_static = method.modifiers.is_static;

            let mut params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| {
                    let ty = if let Some(t) = &p.type_annot {
                        self.emit_type(t)
                    } else {
                        "_".to_string()
                    };
                    format!("{}: {}", p.name, ty)
                })
                .collect();

            // Add &self for instance methods
            if !is_static {
                params.insert(0, "&self".to_string());
            }

            let return_type = if let Some(rt) = &method.return_type {
                format!(" -> {}", self.emit_type(rt))
            } else {
                "".to_string()
            };

            self.line(&format!(
                "fn {}({}){};",
                method.name,
                params.join(", "),
                return_type
            ))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    fn emit_global_var(&mut self, v: &ast::VariableDecl) -> RustResult<()> {
        let init = if let Some(expr) = &v.initializer {
            self.emit_expr(expr)?
        } else {
            "Default::default()".to_string()
        };

        let ty = if let Some(t) = &v.type_annot {
            format!(": {}", self.emit_type(t))
        } else {
            "".to_string()
        };

        let keyword = if v.is_mutable { "static mut" } else { "static" };
        self.line(&format!("{} {}{} = {};", keyword, v.name, ty, init))?;
        Ok(())
    }

    fn emit_function(&mut self, f: &ast::FunctionDecl) -> RustResult<()> {
        let params: Vec<String> = f
            .parameters
            .iter()
            .map(|p| {
                let ty = if let Some(t) = &p.type_annot {
                    self.emit_type(t)
                } else {
                    "_".to_string()
                };
                format!("{}: {}", p.name, ty)
            })
            .collect();

        let return_type = if let Some(rt) = &f.return_type {
            format!(" -> {}", self.emit_type(rt))
        } else {
            "".to_string()
        };

        let visibility = if f.modifiers.visibility == ast::Visibility::Public {
            "pub "
        } else {
            ""
        };

        let async_keyword = if f.is_async { "async " } else { "" };

        self.line(&format!(
            "{}{}fn {}({}){} {{",
            visibility,
            async_keyword,
            f.name,
            params.join(", "),
            return_type
        ))?;
        self.indent();

        // Track parameter types for this function scope
        // Save old values in case they shadow outer variables
        let saved_types: HashMap<String, Option<VarType>> = f
            .parameters
            .iter()
            .map(|p| {
                let old = self.var_types.get(&p.name).cloned();
                if let Some(type_annot) = &p.type_annot {
                    let var_type = match type_annot {
                        ast::Type::Int => VarType::Int,
                        ast::Type::UnsignedInt => VarType::UnsignedInt,
                        ast::Type::Float => VarType::Float,
                        ast::Type::Bool => VarType::Bool,
                        ast::Type::String => VarType::String,
                        _ => VarType::Unknown,
                    };
                    self.var_types.insert(p.name.clone(), var_type);
                }
                (p.name.clone(), old)
            })
            .collect();

        self.emit_block(&f.body)?;

        // Restore old values (or remove if there was no old value)
        for (name, old_type) in saved_types {
            match old_type {
                Some(t) => {
                    self.var_types.insert(name, t);
                }
                None => {
                    self.var_types.remove(&name);
                }
            }
        }

        if f.return_type.is_none() {
            self.line("()")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_main_function(&mut self, program: &AstProgram) -> RustResult<()> {
        self.line("fn main() {")?;
        self.indent();

        // Emit local variables from declarations (private variables)
        for decl in &program.declarations {
            if let ast::Declaration::Variable(v) = decl {
                if v.visibility != ast::Visibility::Public {
                    self.emit_local_var(v)?;
                }
            }
        }

        // Emit top-level statements
        for stmt in &program.statements {
            self.emit_statement(stmt)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_local_var(&mut self, v: &ast::VariableDecl) -> RustResult<()> {
        let init = if let Some(expr) = &v.initializer {
            self.emit_expr(expr)?
        } else {
            "Default::default()".to_string()
        };

        let keyword = if v.is_mutable { "let mut" } else { "let" };
        let ty = if let Some(t) = &v.type_annot {
            format!(": {}", self.emit_type(t))
        } else {
            "".to_string()
        };

        self.line(&format!("{} {}{} = {};", keyword, v.name, ty, init))?;
        Ok(())
    }

    fn emit_block(&mut self, block: &ast::Block) -> RustResult<()> {
        for stmt in &block.statements {
            self.emit_statement(stmt)?;
        }
        Ok(())
    }

    fn emit_statement(&mut self, stmt: &ast::Statement) -> RustResult<()> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&format!("{};", e))?;
            }
            StatementKind::Variable(v) => {
                let init = if let Some(expr) = &v.initializer {
                    self.emit_expr(expr)?
                } else {
                    "Default::default()".to_string()
                };

                let keyword = if v.is_mutable { "let mut" } else { "let" };
                let ty = if let Some(t) = &v.type_annot {
                    format!(": {}", self.emit_type(t))
                } else {
                    "".to_string()
                };

                self.line(&format!("{} {}{} = {};", keyword, v.name, ty, init))?;
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
                self.line(&format!("while {} {{", cond))?;
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
                self.line("loop {")?;
                self.indent();
                self.emit_block(&d.body)?;
                let cond = self.emit_expr(&d.condition)?;
                self.line(&format!("if !({}) {{ break; }}", cond))?;
                self.dedent();
                self.line("}")?;
            }
            StatementKind::Unsafe(block) => {
                self.line("unsafe {")?;
                self.indent();
                self.emit_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            StatementKind::Defer(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&format!("defer {};", e))?;
            }
            StatementKind::Yield(opt_expr) => {
                if let Some(e) = opt_expr {
                    let expr = self.emit_expr(e)?;
                    self.line(&format!("yield {};", expr))?;
                } else {
                    self.line("yield;")?;
                }
            }
            StatementKind::Loop(body) => {
                self.line("loop {")?;
                self.indent();
                self.emit_block(body)?;
                self.dedent();
                self.line("}")?;
            }
        }
        Ok(())
    }

    fn emit_if(&mut self, if_stmt: &ast::IfStatement) -> RustResult<()> {
        let cond = self.emit_expr(&if_stmt.condition)?;
        self.line(&format!("if {} {{", cond))?;
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

    fn emit_for(&mut self, for_stmt: &ast::ForStatement) -> RustResult<()> {
        let iterator = self.emit_expr(&for_stmt.iterator)?;
        let pattern_name = match &for_stmt.pattern {
            ast::Pattern::Variable(name) => name.clone(),
            ast::Pattern::Wildcard => "_".to_string(),
            _ => "item".to_string(),
        };

        self.line(&format!("for {} in {} {{", pattern_name, iterator))?;
        self.indent();
        self.emit_block(&for_stmt.body)?;
        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_match(&mut self, match_stmt: &ast::MatchStatement) -> RustResult<()> {
        let expr = self.emit_expr(&match_stmt.expression)?;
        self.line(&format!("match {} {{", expr))?;
        self.indent();

        for case in &match_stmt.cases {
            let pattern = self.emit_pattern(&case.pattern)?;
            let guard = if let Some(g) = &case.guard {
                format!(" if {}", self.emit_expr(g)?)
            } else {
                "".to_string()
            };
            self.line(&format!("{}{} => {{", pattern, guard))?;
            self.indent();
            self.emit_block(&case.body)?;
            self.dedent();
            self.line("}")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_pattern(&mut self, pattern: &ast::Pattern) -> RustResult<String> {
        match pattern {
            ast::Pattern::Variable(name) => Ok(name.clone()),
            ast::Pattern::Wildcard => Ok("_".to_string()),
            ast::Pattern::Literal(lit) => self.emit_literal_pattern(lit),
            ast::Pattern::Array(patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!("[{}]", pattern_strs.join(", ")))
            }
            ast::Pattern::Dictionary(entries) => {
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let k_str = self.emit_pattern(k)?;
                        let v_str = self.emit_pattern(v)?;
                        Ok(format!("{}: {}", k_str, v_str))
                    })
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!("{{{}}}", entries_str.join(", ")))
            }
            ast::Pattern::Record(name, fields) => {
                let fields_str: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| {
                        let p_str = self.emit_pattern(p)?;
                        Ok(format!("{}: {}", n, p_str))
                    })
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!("{} {{ {} }}", name, fields_str.join(", ")))
            }
            ast::Pattern::Tuple(patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!("({},)", pattern_strs.join(", ")))
            }
            ast::Pattern::Or(left, right) => {
                let left_str = self.emit_pattern(left)?;
                let right_str = self.emit_pattern(right)?;
                Ok(format!("{} | {}", left_str, right_str))
            }
            ast::Pattern::Guard(inner, cond) => {
                let inner_str = self.emit_pattern(inner)?;
                let cond_str = self.emit_expr(cond)?;
                Ok(format!("{} if {}", inner_str, cond_str))
            }
            ast::Pattern::EnumConstructor(type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_pattern(p))
                    .collect::<RustResult<Vec<_>>>()?;
                if patterns.is_empty() {
                    Ok(format!("{}::{}", type_name, variant_name))
                } else {
                    Ok(format!(
                        "{}::{}({})",
                        type_name,
                        variant_name,
                        pattern_strs.join(", ")
                    ))
                }
            }
        }
    }

    fn emit_literal_pattern(&mut self, lit: &ast::Literal) -> RustResult<String> {
        match lit {
            ast::Literal::Integer(n) => Ok(n.to_string()),
            ast::Literal::Float(f) => Ok(f.to_string()),
            ast::Literal::Boolean(b) => Ok(b.to_string()),
            ast::Literal::String(s) => Ok(format!("\"{}\"", s)),
            ast::Literal::Char(c) => Ok(format!("'{}'", c)),
            ast::Literal::Null | ast::Literal::None => Ok("None".to_string()),
            ast::Literal::Unit => Ok("()".to_string()),
        }
    }

    fn emit_try(&mut self, try_stmt: &ast::TryStatement) -> RustResult<()> {
        self.line("{")?;
        self.indent();
        self.line("let __result = (|| {")?;
        self.indent();
        self.emit_block(&try_stmt.body)?;
        self.line("Ok(())")?;
        self.dedent();
        self.line("})();")?;

        self.line("match __result {")?;
        self.indent();

        for cc in &try_stmt.catch_clauses {
            let err_type = cc
                .exception_type
                .as_ref()
                .map(|t| t.clone())
                .unwrap_or_else(|| "_".to_string());
            let var_name = cc.variable_name.as_ref().map(|n| n.as_str()).unwrap_or("_");

            self.line(&format!("Err({}: {}) => {{", var_name, err_type))?;
            self.indent();
            self.emit_block(&cc.body)?;
            self.dedent();
            self.line("}")?;
        }

        self.line("Ok(_) => {}")?;
        self.dedent();
        self.line("}")?;

        if let Some(finally) = &try_stmt.finally_block {
            self.emit_block(finally)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    fn emit_expr(&mut self, expr: &ast::Expression) -> RustResult<String> {
        match &expr.node {
            ExpressionKind::Literal(lit) => self.emit_literal(lit),
            ExpressionKind::Variable(name) => {
                // If the variable is a local/parameter, don't prefix with self
                if self.local_vars.contains(name) {
                    return Ok(name.clone());
                }
                // If we're inside a class and this variable is a field, prefix with self.
                if self.current_class_fields.contains(name) {
                    Ok(format!("self.{}", name))
                } else {
                    Ok(name.clone())
                }
            }
            ExpressionKind::Binary(op, left, right) => {
                let left_type = self.expr_type_name(left);
                let right_type = self.expr_type_name(right);

                // Special handling for string concatenation
                // String + anything should concatenate as strings
                if matches!(op, BinaryOp::Add | BinaryOp::Concat) {
                    let is_string_op = left_type == "string"
                        || right_type == "string"
                        || matches!(&left.node, ExpressionKind::Literal(ast::Literal::String(_)));

                    if is_string_op {
                        let left_str = self.emit_expr(left)?;
                        let right_str = self.emit_expr(right)?;

                        // Use format! for proper string concatenation
                        return Ok(format!(
                            "format!(\"{{}}{{}}\", {}, {})",
                            left_str, right_str
                        ));
                    }
                }

                // Check if we have mixed int/float arithmetic
                let is_mixed_numeric = (left_type == "int" && right_type == "float")
                    || (left_type == "float" && right_type == "int");

                let left_str = self.emit_expr(left)?;
                let right_str = self.emit_expr(right)?;
                let op_str = self.binary_op_to_rust(op);

                if is_mixed_numeric
                    && matches!(
                        op,
                        BinaryOp::Add
                            | BinaryOp::Sub
                            | BinaryOp::Mul
                            | BinaryOp::Div
                            | BinaryOp::Mod
                    )
                {
                    // Cast int to float for mixed arithmetic
                    if left_type == "int" {
                        Ok(format!("({} as f64) {} {}", left_str, op_str, right_str))
                    } else {
                        Ok(format!("{} {} ({} as f64)", left_str, op_str, right_str))
                    }
                } else {
                    Ok(format!("{} {} {}", left_str, op_str, right_str))
                }
            }
            ExpressionKind::Unary(op, e) => {
                let e_str = self.emit_expr(e)?;
                match op {
                    UnaryOp::Negate => Ok(format!("-{}", e_str)),
                    UnaryOp::Not => Ok(format!("!{}", e_str)),
                    UnaryOp::BitNot => Ok(format!("!{}", e_str)), // Rust uses ! for bitwise not
                    UnaryOp::Wait => Ok(format!("{}.await", e_str)),
                }
            }
            ExpressionKind::Cast(expr, ty) => {
                let expr_str = self.emit_expr(expr)?;
                let ty_str = self.emit_type(ty);
                Ok(format!("{} as {}", expr_str, ty_str))
            }
            ExpressionKind::Call(callee, args) => {
                // Handle `Type.null()` -> `std::ptr::null_mut::<Type>()`
                // This is when callee is Member(Pointer(Type), "null") or Member(Type, "null")
                if args.is_empty() {
                    if let ExpressionKind::Member(obj, name) = &callee.node {
                        if name == "null" {
                            // Check if obj is Pointer(T) or ConstPointer(T)
                            if let ExpressionKind::Call(inner_callee, inner_args) = &obj.node {
                                if let ExpressionKind::Variable(type_name) = &inner_callee.node {
                                    if type_name == "Pointer" && inner_args.len() == 1 {
                                        let inner_type = self.emit_expr(&inner_args[0])?;
                                        let rust_type = self.type_name_to_rust_type(&inner_type);
                                        return Ok(format!(
                                            "std::ptr::null_mut::<{}>()",
                                            rust_type
                                        ));
                                    }
                                    if type_name == "ConstPointer" && inner_args.len() == 1 {
                                        let inner_type = self.emit_expr(&inner_args[0])?;
                                        let rust_type = self.type_name_to_rust_type(&inner_type);
                                        return Ok(format!("std::ptr::null::<{}>()", rust_type));
                                    }
                                }
                            }
                            // Check if obj is a simple type name like Void, CLong, etc.
                            if let ExpressionKind::Variable(type_name) = &obj.node {
                                let rust_type = self.type_name_to_rust_type(type_name);
                                return Ok(format!("std::ptr::null_mut::<{}>()", rust_type));
                            }
                        }
                    }
                }

                // Special handling for println/print functions -> convert to Rust macros
                if let ExpressionKind::Variable(name) = &callee.node {
                    match name.as_str() {
                        "println" => {
                            if args.is_empty() {
                                return Ok("println!()".to_string());
                            } else {
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_expr(a))
                                    .collect::<RustResult<Vec<_>>>()?;
                                let format_args: Vec<String> =
                                    args_str.iter().map(|_| "{}".to_string()).collect();
                                return Ok(format!(
                                    "println!(\"{}\", {})",
                                    format_args.join(" "),
                                    args_str.join(", ")
                                ));
                            }
                        }
                        "print" => {
                            if args.is_empty() {
                                return Ok("print!()".to_string());
                            } else {
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_expr(a))
                                    .collect::<RustResult<Vec<_>>>()?;
                                let format_args: Vec<String> =
                                    args_str.iter().map(|_| "{}".to_string()).collect();
                                return Ok(format!(
                                    "print!(\"{}\", {})",
                                    format_args.join(" "),
                                    args_str.join(", ")
                                ));
                            }
                        }
                        "eprintln" => {
                            if args.is_empty() {
                                return Ok("eprintln!()".to_string());
                            } else {
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_expr(a))
                                    .collect::<RustResult<Vec<_>>>()?;
                                let format_args: Vec<String> =
                                    args_str.iter().map(|_| "{}".to_string()).collect();
                                return Ok(format!(
                                    "eprintln!(\"{}\", {})",
                                    format_args.join(" "),
                                    args_str.join(", ")
                                ));
                            }
                        }
                        "eprint" => {
                            if args.is_empty() {
                                return Ok("eprint!()".to_string());
                            } else {
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_expr(a))
                                    .collect::<RustResult<Vec<_>>>()?;
                                let format_args: Vec<String> =
                                    args_str.iter().map(|_| "{}".to_string()).collect();
                                return Ok(format!(
                                    "eprint!(\"{}\", {})",
                                    format_args.join(" "),
                                    args_str.join(", ")
                                ));
                            }
                        }
                        _ => {}
                    }
                }

                // Handle struct instantiation: TypeName() for empty struct
                // Heuristic: TypeName starts with uppercase and no args
                if let ExpressionKind::Variable(type_name) = &callee.node {
                    if type_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        if args.is_empty() {
                            // Empty struct instantiation
                            return Ok(format!("{} {{ }}", type_name));
                        }
                        // Could be a constructor call with positional args
                        // For now, emit as function call
                    }
                }

                // Handle type constructors like Pointer(Void), ConstPointer(T)
                if let ExpressionKind::Variable(type_name) = &callee.node {
                    match type_name.as_str() {
                        "Pointer" if args.len() == 1 => {
                            // Pointer(T) - emit as a type reference for FFI
                            let inner_type = self.emit_expr(&args[0])?;
                            let rust_type = self.type_name_to_rust_type(&inner_type);
                            // Return a marker that can be used in type position
                            return Ok(format!("*mut {}", rust_type));
                        }
                        "ConstPointer" if args.len() == 1 => {
                            let inner_type = self.emit_expr(&args[0])?;
                            let rust_type = self.type_name_to_rust_type(&inner_type);
                            return Ok(format!("*const {}", rust_type));
                        }
                        _ => {}
                    }
                }

                // Handle enum constructors: TypeName.Variant(args)
                // Heuristic: TypeName starts with uppercase
                if let ExpressionKind::Member(obj, variant_name) = &callee.node {
                    if let ExpressionKind::Variable(type_name) = &obj.node {
                        // Check if this looks like a type name (starts with uppercase)
                        if type_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                            // This looks like an enum constructor
                            let args_str: Vec<String> = args
                                .iter()
                                .map(|a| self.emit_expr(a))
                                .collect::<RustResult<Vec<_>>>()?;
                            return Ok(format!(
                                "{}::{}({})",
                                type_name,
                                variant_name,
                                args_str.join(", ")
                            ));
                        }
                    }
                }

                let callee_str = self.emit_expr(callee)?;
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_expr(a))
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!("{}({})", callee_str, args_str.join(", ")))
            }
            ExpressionKind::Member(obj, name) => {
                // Handle `this.field` -> `self.field`
                if let ExpressionKind::Variable(var_name) = &obj.node {
                    if var_name == "this" {
                        return Ok(format!("self.{}", name));
                    }
                    // Check if this looks like an enum unit variant (TypeName.Variant)
                    // Heuristic: Type names start with uppercase, and this is NOT part of a Call
                    // This is checked in the Call handler, so here we just emit as :: for uppercase types
                    if var_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        return Ok(format!("{}::{}", var_name, name));
                    }
                }

                let obj_str = self.emit_expr(obj)?;
                Ok(format!("{}.{}", obj_str, name))
            }
            ExpressionKind::Assign(target, value) => {
                let target_str = self.emit_expr(target)?;
                let value_str = self.emit_expr(value)?;
                Ok(format!("{} = {}", target_str, value_str))
            }
            ExpressionKind::If(cond, then, else_) => {
                let cond_str = self.emit_expr(cond)?;
                let then_str = self.emit_expr(then)?;
                let else_str = self.emit_expr(else_)?;
                Ok(format!(
                    "if {} {{ {} }} else {{ {} }}",
                    cond_str, then_str, else_str
                ))
            }
            ExpressionKind::Lambda(params, body) => {
                let params_str: Vec<String> = params
                    .iter()
                    .map(|p| {
                        let ty = if let Some(t) = &p.type_annot {
                            format!(": {}", self.emit_type(t))
                        } else {
                            "".to_string()
                        };
                        format!("{}{}", p.name, ty)
                    })
                    .collect();

                let saved_output = self.buffer.take();
                self.emit_block(body)?;
                let body_output = self.buffer.take();
                self.buffer = x_codegen::CodeBuffer::new();
                // write saved output back
                let _ = self.buffer.line(&saved_output);

                Ok(format!(
                    "|{}| {{ {} }}",
                    params_str.join(", "),
                    body_output.trim()
                ))
            }
            ExpressionKind::Array(elements) => {
                // Check if this is a mixed-type array
                let types: std::collections::HashSet<String> =
                    elements.iter().map(|e| self.expr_type_name(e)).collect();

                if types.len() > 1 || elements.iter().any(|e| self.expr_needs_dynamic(e)) {
                    // Generate as DynamicValue::Array
                    let elements_str: Vec<String> = elements
                        .iter()
                        .map(|e| self.emit_dynamic_value(e))
                        .collect::<RustResult<Vec<_>>>()?;
                    Ok(format!(
                        "DynamicValue::Array(vec![{}])",
                        elements_str.join(", ")
                    ))
                } else {
                    let elements_str: Vec<String> = elements
                        .iter()
                        .map(|e| self.emit_expr(e))
                        .collect::<RustResult<Vec<_>>>()?;
                    Ok(format!("vec![{}]", elements_str.join(", ")))
                }
            }
            ExpressionKind::Tuple(elements) => {
                // 元组作为数组处理
                let elements_str: Vec<String> = elements
                    .iter()
                    .map(|e| self.emit_expr(e))
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!("vec![{}]", elements_str.join(", ")))
            }
            ExpressionKind::Dictionary(entries) => {
                if entries.is_empty() {
                    return Ok("HashMap::new()".to_string());
                }

                // Check if this is a mixed-type dictionary
                let value_types: std::collections::HashSet<String> = entries
                    .iter()
                    .map(|(_, v)| self.expr_type_name(v))
                    .collect();
                let has_nested_dynamic = entries.iter().any(|(_, v)| self.expr_needs_dynamic(v));

                if value_types.len() > 1 || has_nested_dynamic {
                    // Generate as DynamicValue::Map
                    let entries_str: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| {
                            let k_str = self.emit_expr(k)?;
                            let v_str = self.emit_dynamic_value(v)?;
                            // k_str is already "key".to_string() from emit_literal
                            Ok(format!("({}, {})", k_str, v_str))
                        })
                        .collect::<RustResult<Vec<_>>>()?;
                    Ok(format!(
                        "DynamicValue::Map(vec![{}].into_iter().collect())",
                        entries_str.join(", ")
                    ))
                } else {
                    let entries_str: Vec<String> = entries
                        .iter()
                        .map(|(k, v)| {
                            let k_str = self.emit_expr(k)?;
                            let v_str = self.emit_expr(v)?;
                            Ok(format!("({}, {})", k_str, v_str))
                        })
                        .collect::<RustResult<Vec<_>>>()?;
                    Ok(format!(
                        "vec![{}].into_iter().collect()",
                        entries_str.join(", ")
                    ))
                }
            }
            ExpressionKind::Range(start, end, inclusive) => {
                let start_str = self.emit_expr(start)?;
                let end_str = self.emit_expr(end)?;
                if *inclusive {
                    Ok(format!("{}..={}", start_str, end_str))
                } else {
                    Ok(format!("{}..{}", start_str, end_str))
                }
            }
            ExpressionKind::Parenthesized(inner) => {
                let inner_str = self.emit_expr(inner)?;
                Ok(format!("({})", inner_str))
            }
            ExpressionKind::Record(name, fields) => {
                let fields_str: Vec<String> = fields
                    .iter()
                    .map(|(n, e)| {
                        let e_str = self.emit_expr(e)?;
                        Ok(format!("{}: {}", n, e_str))
                    })
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!("{} {{ {} }}", name, fields_str.join(", ")))
            }
            ExpressionKind::Pipe(input, stages) => {
                let mut result = self.emit_expr(input)?;
                for stage in stages {
                    let stage_str = self.emit_expr(stage)?;
                    result = format!("{}({})", stage_str, result);
                }
                Ok(result)
            }
            ExpressionKind::Wait(wait_type, exprs) => match wait_type {
                ast::WaitType::Single => {
                    if exprs.len() == 1 {
                        let expr_str = self.emit_expr(&exprs[0])?;
                        Ok(format!("{}.await", expr_str))
                    } else {
                        Err(x_codegen::CodeGenError::UnsupportedFeature(
                            "Wait single with multiple expressions".to_string(),
                        ))
                    }
                }
                ast::WaitType::Together => {
                    let exprs_str: Vec<String> = exprs
                        .iter()
                        .map(|e| self.emit_expr(e))
                        .collect::<RustResult<Vec<_>>>()?;
                    Ok(format!("futures::join!({})", exprs_str.join(", ")))
                }
                ast::WaitType::Race => {
                    let exprs_str: Vec<String> = exprs
                        .iter()
                        .enumerate()
                        .map(|(i, e)| {
                            let e_str = self.emit_expr(e)?;
                            Ok(format!("{} => __res{}_{}", e_str, i, i))
                        })
                        .collect::<RustResult<Vec<_>>>()?;
                    Ok(format!("futures::select!({})", exprs_str.join(", ")))
                }
                ast::WaitType::Timeout(_) => Err(x_codegen::CodeGenError::UnsupportedFeature(
                    "Wait with timeout".to_string(),
                )),
                ast::WaitType::Atomic => {
                    if exprs.len() == 1 {
                        let expr_str = self.emit_expr(&exprs[0])?;
                        Ok(format!("atomic {}; {}", expr_str, expr_str))
                    } else {
                        Err(x_codegen::CodeGenError::UnsupportedFeature(
                            "Wait atomic with multiple expressions".to_string(),
                        ))
                    }
                }
                ast::WaitType::Retry => {
                    if exprs.len() == 1 {
                        let expr_str = self.emit_expr(&exprs[0])?;
                        Ok(format!("// retry: {}", expr_str))
                    } else {
                        Err(x_codegen::CodeGenError::UnsupportedFeature(
                            "Wait retry with multiple expressions".to_string(),
                        ))
                    }
                }
            },
            ExpressionKind::Await(expr) => {
                let expr_str = self.emit_expr(expr)?;
                Ok(format!("{}.await", expr_str))
            }
            ExpressionKind::OptionalChain(base, member) => {
                let base_str = self.emit_expr(base)?;
                Ok(format!("{}?.{}", base_str, member))
            }
            ExpressionKind::NullCoalescing(left, right) => {
                let left_str = self.emit_expr(left)?;
                let right_str = self.emit_expr(right)?;
                Ok(format!("{}.unwrap_or({})", left_str, right_str))
            }
            ExpressionKind::Needs(name) => Ok(format!("/* needs: {} */", name)),
            ExpressionKind::Given(name, expr) => {
                let expr_str = self.emit_expr(expr)?;
                Ok(format!("/* given {} = {} */", name, expr_str))
            }
            ExpressionKind::Handle(expr, handlers) => {
                let expr_str = self.emit_expr(expr)?;
                let handlers_str: Vec<String> = handlers
                    .iter()
                    .map(|(name, handler)| {
                        let handler_str = self.emit_expr(handler)?;
                        Ok(format!("{} => {}", name, handler_str))
                    })
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!(
                    "handle {} with {{ {} }}",
                    expr_str,
                    handlers_str.join(", ")
                ))
            }
            ExpressionKind::TryPropagate(expr) => {
                let expr_str = self.emit_expr(expr)?;
                Ok(format!("{}?", expr_str))
            }
            ExpressionKind::Match(discriminant, match_cases) => {
                let discriminant_code = self.emit_expr(discriminant)?;
                let mut output = String::new();
                output.push_str(&format!("match {} {{\n", discriminant_code));
                self.indent();
                for case in match_cases {
                    let pattern = self.emit_pattern(&case.pattern)?;
                    let guard = if let Some(g) = &case.guard {
                        format!(" if {}", self.emit_expr(g)?)
                    } else {
                        String::new()
                    };
                    // Match body - since this is an expression match, the last expression is returned
                    let mut body_code = String::new();
                    let stmt_count = case.body.statements.len();
                    for (i, stmt) in case.body.statements.iter().enumerate() {
                        let is_last = i == stmt_count - 1;
                        if let StatementKind::Expression(expr) = &stmt.node {
                            if is_last {
                                // Last expression in match body is the result
                                body_code.push_str(&self.emit_expr(expr)?);
                            } else {
                                self.emit_statement(stmt)?;
                            }
                        } else {
                            self.emit_statement(stmt)?;
                        }
                    }
                    let indent = "    ".repeat(self.buffer.indent_level());
                    output.push_str(&format!("{}{}{} => {{\n", indent, pattern, guard));
                    self.indent();
                    if !body_code.is_empty() {
                        output.push_str(&format!(
                            "{}{}\n",
                            "    ".repeat(self.buffer.indent_level()),
                            body_code
                        ));
                    }
                    self.dedent();
                    output.push_str(&format!(
                        "{}}},\n",
                        "    ".repeat(self.buffer.indent_level())
                    ));
                }
                self.dedent();
                output.push_str(&format!("{}}}", "    ".repeat(self.buffer.indent_level())));
                Ok(output)
            }
        }
    }

    /// Emit an expression as a DynamicValue enum variant
    fn emit_dynamic_value(&mut self, expr: &ast::Expression) -> RustResult<String> {
        match &expr.node {
            ExpressionKind::Literal(lit) => match lit {
                ast::Literal::Integer(n) => Ok(format!("DynamicValue::Int({})", n)),
                ast::Literal::Float(f) => Ok(format!("DynamicValue::Float({}f64)", f)),
                ast::Literal::Boolean(b) => Ok(format!("DynamicValue::Bool({})", b)),
                ast::Literal::String(s) => Ok(format!(
                    "DynamicValue::String({}.to_string())",
                    format!("\"{}\"", s)
                )),
                ast::Literal::Char(c) => Ok(format!(
                    "DynamicValue::String({}.to_string())",
                    format!("'{}'", c)
                )),
                ast::Literal::Null | ast::Literal::None => Ok("DynamicValue::Null".to_string()),
                ast::Literal::Unit => Ok("DynamicValue::Null".to_string()),
            },
            ExpressionKind::Array(elements) => {
                let elements_str: Vec<String> = elements
                    .iter()
                    .map(|e| self.emit_dynamic_value(e))
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!(
                    "DynamicValue::Array(vec![{}])",
                    elements_str.join(", ")
                ))
            }
            ExpressionKind::Tuple(elements) => {
                let elements_str: Vec<String> = elements
                    .iter()
                    .map(|e| self.emit_dynamic_value(e))
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!(
                    "DynamicValue::Array(vec![{}])",
                    elements_str.join(", ")
                ))
            }
            ExpressionKind::Dictionary(entries) => {
                if entries.is_empty() {
                    return Ok("DynamicValue::Map(HashMap::new())".to_string());
                }
                let entries_str: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let k_str = self.emit_expr(k)?;
                        let v_str = self.emit_dynamic_value(v)?;
                        // k_str is already "key".to_string() from emit_literal
                        Ok(format!("({}, {})", k_str, v_str))
                    })
                    .collect::<RustResult<Vec<_>>>()?;
                Ok(format!(
                    "DynamicValue::Map(vec![{}].into_iter().collect())",
                    entries_str.join(", ")
                ))
            }
            ExpressionKind::Variable(name) => Ok(name.clone()),
            _ => {
                // For other expressions, emit normally (they should be DynamicValue already)
                self.emit_expr(expr)
            }
        }
    }

    fn emit_literal(&mut self, lit: &ast::Literal) -> RustResult<String> {
        match lit {
            ast::Literal::Integer(n) => Ok(n.to_string()),
            ast::Literal::Float(f) => Ok(format!("{}f64", f)),
            ast::Literal::Boolean(b) => Ok(b.to_string()),
            ast::Literal::String(s) => Ok(format!("\"{}\".to_string()", s)),
            ast::Literal::Char(c) => Ok(format!("'{}'", c)),
            ast::Literal::Null | ast::Literal::None => Ok("None".to_string()),
            ast::Literal::Unit => Ok("()".to_string()),
        }
    }

    fn binary_op_to_rust(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Pow => ".pow", // Rust doesn't have **, uses method
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::LeftShift => "<<",
            BinaryOp::RightShift => ">>",
            BinaryOp::Concat => "+", // String concat
            BinaryOp::RangeExclusive => "..",
            BinaryOp::RangeInclusive => "..=",
        }
    }

    fn emit_type(&self, ty: &ast::Type) -> String {
        match ty {
            ast::Type::Int => "i32".to_string(),
            ast::Type::UnsignedInt => "u32".to_string(),
            ast::Type::Float => "f64".to_string(),
            ast::Type::Bool => "bool".to_string(),
            ast::Type::String => "String".to_string(),
            ast::Type::Char => "char".to_string(),
            ast::Type::Unit => "()".to_string(),
            ast::Type::Never => "!".to_string(),
            ast::Type::Array(inner) => format!("Vec<{}>", self.emit_type(inner)),
            ast::Type::Dictionary(k, v) => {
                format!("HashMap<{}, {}>", self.emit_type(k), self.emit_type(v))
            }
            ast::Type::TypeConstructor(name, args) if name == "Option" && args.len() == 1 => {
                format!("Option<{}>", self.emit_type(&args[0]))
            }
            ast::Type::TypeConstructor(name, args) if name == "Result" && args.len() == 2 => {
                format!(
                    "Result<{}, {}>",
                    self.emit_type(&args[0]),
                    self.emit_type(&args[1])
                )
            }
            ast::Type::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.emit_type(t)).collect();
                format!("({},)", type_strs.join(", "))
            }
            ast::Type::Function(params, ret) => {
                let params_str: Vec<String> = params.iter().map(|t| self.emit_type(t)).collect();
                format!("fn({}) -> {}", params_str.join(", "), self.emit_type(ret))
            }
            ast::Type::Async(inner) => format!("impl Future<Output = {}>", self.emit_type(inner)),
            ast::Type::Generic(name) | ast::Type::Var(name) => name.clone(),
            ast::Type::TypeParam(name) => name.clone(),
            ast::Type::TypeConstructor(name, args) => {
                let args_str: Vec<String> = args.iter().map(|t| self.emit_type(t)).collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
            ast::Type::Union(name, _) => name.clone(),
            ast::Type::Record(name, _) => name.clone(),
            ast::Type::Dynamic => "Box<dyn std::any::Any>".to_string(),
            ast::Type::Reference(inner) => format!("&{}", self.emit_type(inner)),
            ast::Type::MutableReference(inner) => format!("&mut {}", self.emit_type(inner)),
            // FFI types
            ast::Type::Pointer(inner) => format!("*mut {}", self.emit_type(inner)),
            ast::Type::ConstPointer(inner) => format!("*const {}", self.emit_type(inner)),
            ast::Type::Void => "std::ffi::c_void".to_string(),
            // C FFI types
            ast::Type::CInt => "std::ffi::c_int".to_string(),
            ast::Type::CUInt => "std::ffi::c_uint".to_string(),
            ast::Type::CLong => "std::ffi::c_long".to_string(),
            ast::Type::CULong => "std::ffi::c_ulong".to_string(),
            ast::Type::CLongLong => "std::ffi::c_longlong".to_string(),
            ast::Type::CULongLong => "std::ffi::c_ulonglong".to_string(),
            ast::Type::CFloat => "std::ffi::c_float".to_string(),
            ast::Type::CDouble => "std::ffi::c_double".to_string(),
            ast::Type::CChar => "std::ffi::c_char".to_string(),
            ast::Type::CSize => "usize".to_string(),
            ast::Type::CString => "std::ffi::CString".to_string(),
        }
    }

    fn infer_type_from_expr(&self, expr: &ast::Expression) -> String {
        match &expr.node {
            ExpressionKind::Literal(lit) => match lit {
                ast::Literal::Integer(_) => "i32".to_string(),
                ast::Literal::Float(_) => "f64".to_string(),
                ast::Literal::Boolean(_) => "bool".to_string(),
                ast::Literal::String(_) => "String".to_string(),
                ast::Literal::Char(_) => "char".to_string(),
                ast::Literal::Null | ast::Literal::None => "Option<_>".to_string(),
                ast::Literal::Unit => "()".to_string(),
            },
            ExpressionKind::Array(_) => "Vec<_>".to_string(),
            ExpressionKind::Dictionary(_) => "HashMap<_, _>".to_string(),
            ExpressionKind::Lambda(_, _) => "impl Fn(_)".to_string(),
            ExpressionKind::Record(name, _) => name.clone(),
            ExpressionKind::Variable(name) => name.clone(),
            _ => "_".to_string(),
        }
    }

    /// Convert a type name string (from expression context) to Rust FFI type
    fn type_name_to_rust_type(&self, type_name: &str) -> String {
        match type_name {
            "Void" => "std::ffi::c_void".to_string(),
            "CInt" => "std::ffi::c_int".to_string(),
            "CUInt" => "std::ffi::c_uint".to_string(),
            "CLong" => "std::ffi::c_long".to_string(),
            "CULong" => "std::ffi::c_ulong".to_string(),
            "CLongLong" => "std::ffi::c_longlong".to_string(),
            "CULongLong" => "std::ffi::c_ulonglong".to_string(),
            "CFloat" => "std::ffi::c_float".to_string(),
            "CDouble" => "std::ffi::c_double".to_string(),
            "CChar" => "std::ffi::c_char".to_string(),
            "CSize" => "usize".to_string(),
            "CString" => "std::ffi::CString".to_string(),
            "Int" => "i32".to_string(),
            "UnsignedInt" | "UInt" => "u32".to_string(),
            "Float" => "f64".to_string(),
            "Bool" => "bool".to_string(),
            "String" => "String".to_string(),
            "Char" => "char".to_string(),
            _ => type_name.to_string(),
        }
    }
}

impl RustBackend {
    /// Generate Rust code from LIR (Low-level Intermediate Representation)
    /// This is the new code generation path that aligns with the compiler architecture:
    /// HIR -> MIR -> LIR -> Codegen
    pub fn generate_from_lir(&mut self, program: &LirProgram) -> RustResult<CodegenOutput> {
        // Start with prelude
        self.line("// Generated by X Language Rust Backend from LIR")?;
        self.line("// Warning: This file is automatically generated. DO NOT EDIT.")?;
        self.line("")?;

        // Add necessary imports that are commonly used
        self.line("use std::collections::HashMap;")?;
        self.line("use std::ffi::c_void;")?;
        self.line("use std::process;")?;
        self.line("")?;

        // Process all declarations
        for decl in &program.declarations {
            self.generate_lir_declaration(decl)?;
        }

        Ok(CodegenOutput {
            files: vec![OutputFile {
                path: PathBuf::from("main.rs"),
                content: self.output().to_string().into_bytes(),
                file_type: x_codegen::FileType::Rust,
            }],
            dependencies: Vec::new(),
        })
    }

    /// 使用 rustc 编译生成的 Rust 代码为可执行文件
    pub fn compile_rust(
        rust_code: &str,
        output_path: &std::path::Path,
    ) -> Result<std::path::PathBuf, x_codegen::CodeGenError> {
        // 创建临时目录存放 Rust 源文件
        let temp_dir = std::env::temp_dir().join("xlang_rust_build");
        let src_dir = temp_dir.join("src");
        std::fs::create_dir_all(&src_dir).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!(
                "Failed to create temp directory: {}",
                e
            ))
        })?;

        // 写入 src/main.rs
        let rs_path = src_dir.join("main.rs");
        std::fs::write(&rs_path, rust_code).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!(
                "Failed to write Rust source: {}",
                e
            ))
        })?;

        // 创建 Cargo.toml
        let cargo_toml = r#"[package]
name = "xlang_output"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "xlang_output"
path = "src/main.rs"

[dependencies]
"#;
        let cargo_path = temp_dir.join("Cargo.toml");
        std::fs::write(&cargo_path, cargo_toml).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!(
                "Failed to write Cargo.toml: {}",
                e
            ))
        })?;

        // 调用 cargo build
        let output_status = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(&cargo_path)
            .current_dir(&temp_dir)
            .output()
            .map_err(|e| {
                x_codegen::CodeGenError::GenerationError(format!(
                    "Failed to invoke cargo: {}. Is Rust installed?",
                    e
                ))
            })?;

        if !output_status.status.success() {
            let stderr = String::from_utf8_lossy(&output_status.stderr);
            let stdout = String::from_utf8_lossy(&output_status.stdout);
            return Err(x_codegen::CodeGenError::GenerationError(format!(
                "Rust compilation failed.\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout, stderr
            )));
        }

        // 找到生成的可执行文件
        let target_dir = temp_dir.join("target").join("release");
        let exe_name = if cfg!(windows) {
            "xlang_output.exe"
        } else {
            "xlang_output"
        };
        let exe_path = target_dir.join(exe_name);

        if !exe_path.exists() {
            return Err(x_codegen::CodeGenError::GenerationError(format!(
                "cargo build succeeded but output not found at {}",
                exe_path.display()
            )));
        }

        // 复制到目标位置
        std::fs::copy(&exe_path, output_path).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!(
                "Failed to copy executable: {}",
                e
            ))
        })?;

        // 设置可执行权限（非 Windows）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(output_path)
                .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))?
                .permissions();
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(output_path, perms)
                .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))?;
        }

        // 清理临时文件
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(output_path.to_path_buf())
    }

    /// Generate code for a LIR declaration
    fn generate_lir_declaration(&mut self, decl: &x_lir::Declaration) -> RustResult<()> {
        match decl {
            x_lir::Declaration::Import(import) => self.generate_lir_import(import)?,
            x_lir::Declaration::Function(func) => self.generate_lir_function(func)?,
            x_lir::Declaration::Global(global) => self.generate_lir_global(global)?,
            x_lir::Declaration::Struct(struct_) => self.generate_lir_struct(struct_)?,
            x_lir::Declaration::Class(class) => self.generate_lir_class(class)?,
            x_lir::Declaration::VTable(vtable) => self.generate_lir_vtable(vtable)?,
            x_lir::Declaration::Enum(enum_) => self.generate_lir_enum(enum_)?,
            x_lir::Declaration::TypeAlias(alias) => self.generate_lir_type_alias(alias)?,
            x_lir::Declaration::ExternFunction(ext) => self.generate_lir_extern_function(ext)?,
        }
        Ok(())
    }

    /// Generate import declaration
    fn generate_lir_import(&mut self, import: &x_lir::Import) -> RustResult<()> {
        if import.import_all {
            self.line(&format!("use {}::*;", import.module_path))?;
        } else if !import.symbols.is_empty() {
            let symbols: Vec<String> = import
                .symbols
                .iter()
                .map(|(name, alias)| {
                    if let Some(alias) = alias {
                        format!("{} as {}", name, alias)
                    } else {
                        name.clone()
                    }
                })
                .collect();
            self.line(&format!(
                "use {}::{{{}}};",
                import.module_path,
                symbols.join(", ")
            ))?;
        }
        self.line("")?;
        Ok(())
    }

    /// Generate function from LIR
    fn generate_lir_function(&mut self, func: &x_lir::Function) -> RustResult<()> {
        // Handle type parameters for generics
        let type_params = if func.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", func.type_params.join(", "))
        };

        // Build parameters
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|param| format!("{}: {}", param.name, self.lir_type_to_rust(&param.type_)))
            .collect();

        // main function must return (), convert i32 return to ()
        let is_main = func.name == "main";
        let return_type = if is_main {
            "()".to_string()
        } else {
            self.lir_type_to_rust(&func.return_type)
        };
        let is_async = false; // LIR doesn't carry async info yet
        let _async_keyword = if is_async { "async " } else { "" };

        self.line(&format!(
            "{}pub fn {}{}({}) -> {} {{",
            if func.is_static { "pub " } else { "" },
            func.name,
            type_params,
            params.join(", "),
            return_type
        ))?;
        self.indent();

        // For main function, we need special handling of return statements
        if is_main {
            self.generate_lir_block_for_main(&func.body)?;
        } else {
            self.generate_lir_block(&func.body)?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;

        Ok(())
    }

    /// Generate block for main function (handles return differently)
    fn generate_lir_block_for_main(&mut self, block: &x_lir::Block) -> RustResult<()> {
        for stmt in &block.statements {
            // Check if this is the last statement and it's a return
            let is_last_return = if let x_lir::Statement::Return(Some(_)) = stmt {
                block.statements.iter().last() == Some(stmt)
            } else {
                false
            };

            if is_last_return {
                // For main, use std::process::exit() to set exit code
                if let x_lir::Statement::Return(Some(expr)) = stmt {
                    let code = self.generate_lir_expression(expr)?;
                    self.line(&format!("std::process::exit({});", code))?;
                }
            } else {
                self.generate_lir_statement(stmt)?;
            }
        }
        Ok(())
    }

    /// Generate global variable
    fn generate_lir_global(&mut self, global: &x_lir::GlobalVar) -> RustResult<()> {
        let ty = self.lir_type_to_rust(&global.type_);
        // For global variables in X, use static (not pub)
        let prefix = if global.is_static { "static " } else { "static " };
        let pub_prefix = if global.is_static { "pub " } else { "" };
        let mut decl = format!(
            "{}{}{} : {}{}",
            prefix,
            pub_prefix,
            global.name,
            ty,
            if global.initializer.is_some() {
                " = "
            } else {
                ";"
            }
        );

        if let Some(init) = &global.initializer {
            let init_code = self.generate_lir_expression(init)?;
            decl.push_str(&init_code);
            decl.push(';');
        }

        self.line(&decl)?;
        self.line("")?;
        Ok(())
    }

    /// Generate struct definition
    fn generate_lir_struct(&mut self, struct_: &x_lir::Struct) -> RustResult<()> {
        self.line("#[derive(Debug, Clone, PartialEq)]")?;
        self.line(&format!("pub struct {} {{", struct_.name))?;
        self.indent();

        for field in &struct_.fields {
            let ty = self.lir_type_to_rust(&field.type_);
            self.line(&format!("pub {}: {},", field.name, ty))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate class definition
    fn generate_lir_class(&mut self, class: &x_lir::Class) -> RustResult<()> {
        self.line("#[derive(Debug, Clone)]")?;
        self.line(&format!("pub struct {} {{", class.name))?;
        self.indent();

        // If this class has a vtable, add it
        if class.has_vtable {
            self.line(&format!("vtable: *mut {}VTable,", class.name))?;
        }

        for field in &class.fields {
            let ty = self.lir_type_to_rust(&field.type_);
            self.line(&format!("pub {}: {},", field.name, ty))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate vtable definition
    fn generate_lir_vtable(&mut self, vtable: &x_lir::VTable) -> RustResult<()> {
        self.line(&format!("pub struct {}VTable {{", vtable.name))?;
        self.indent();

        for entry in &vtable.entries {
            let params: Vec<String> = entry
                .function_type
                .param_types
                .iter()
                .map(|ty| self.lir_type_to_rust(ty))
                .collect();
            let return_type = self.lir_type_to_rust(&entry.function_type.return_type);
            let fn_type = format!("fn({}) -> {}", params.join(", "), return_type);
            self.line(&format!("pub {}: {},", entry.method_name, fn_type))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate enum definition
    fn generate_lir_enum(&mut self, enum_: &x_lir::Enum) -> RustResult<()> {
        self.line("#[derive(Debug, Clone, PartialEq)]")?;
        self.line(&format!("pub enum {} {{", enum_.name))?;
        self.indent();

        for variant in &enum_.variants {
            if let Some(value) = variant.value {
                self.line(&format!("{} = {},", variant.name, value))?;
            } else {
                self.line(&format!("{},", variant.name))?;
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate type alias
    fn generate_lir_type_alias(&mut self, alias: &x_lir::TypeAlias) -> RustResult<()> {
        let ty = self.lir_type_to_rust(&alias.type_);
        self.line(&format!("pub type {} = {};", alias.name, ty))?;
        self.line("")?;
        Ok(())
    }

    /// Generate extern function declaration
    fn generate_lir_extern_function(&mut self, ext: &x_lir::ExternFunction) -> RustResult<()> {
        // Use uppercase "C" for Rust ABI
        let abi = ext.abi.clone().unwrap_or_else(|| "C".to_string());
        let abi_display = if abi.to_lowercase() == "c" { "C" } else { &abi };

        let type_params = if ext.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", ext.type_params.join(", "))
        };

        // Parameters are just types, generate with numbered names
        let params: Vec<String> = ext
            .parameters
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("arg{}: {}", i, self.lir_type_to_rust(ty)))
            .collect();

        let return_type = self.lir_type_to_rust(&ext.return_type);
        self.line(&format!("#[link(name = \"{}\")]", abi.to_lowercase()))?;
        self.line(&format!("extern \"{}\" {{", abi_display))?;
        self.indent();
        self.line(&format!(
            "fn {}{}({}) -> {};",
            ext.name,
            type_params,
            params.join(", "),
            return_type
        ))?;
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate a LIR basic block
    fn generate_lir_block(&mut self, block: &x_lir::Block) -> RustResult<()> {
        for stmt in &block.statements {
            self.generate_lir_statement(stmt)?;
        }
        Ok(())
    }

    /// Generate a LIR statement
    fn generate_lir_statement(&mut self, stmt: &x_lir::Statement) -> RustResult<()> {
        match stmt {
            x_lir::Statement::Expression(expr) => {
                let code = self.generate_lir_expression(expr)?;
                self.line(&format!("{};", code))?;
            }
            x_lir::Statement::Variable(var) => {
                let ty = self.lir_type_to_rust(&var.type_);
                let mut decl = if var.is_static {
                    format!("static {}: {}", var.name, ty)
                } else {
                    format!("let {}: {}", var.name, ty)
                };

                if var.is_extern {
                    decl.push_str(";");
                    let _ = self.line(&decl);
                } else if let Some(init) = &var.initializer {
                    let init_code = self.generate_lir_expression(init)?;
                    decl.push_str(&format!(" = {};", init_code));
                    let _ = self.line(&decl);
                } else {
                    decl.push_str(";");
                    let _ = self.line(&decl);
                }
            }
            x_lir::Statement::If(if_stmt) => {
                let cond = self.generate_lir_expression(&if_stmt.condition)?;
                self.line(&format!("if {} {{", cond))?;
                self.indent();
                self.generate_lir_statement(&if_stmt.then_branch)?;
                self.dedent();

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.line("} else {")?;
                    self.indent();
                    self.generate_lir_statement(else_branch)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            x_lir::Statement::While(while_stmt) => {
                let cond = self.generate_lir_expression(&while_stmt.condition)?;
                self.line(&format!("while {} {{", cond))?;
                self.indent();
                self.generate_lir_statement(&while_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::DoWhile(do_while) => {
                self.line("loop {")?;
                self.indent();
                self.generate_lir_statement(&do_while.body)?;
                let cond = self.generate_lir_expression(&do_while.condition)?;
                self.line(&format!("if !({}) {{ break; }}", cond))?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::For(for_stmt) => {
                self.line("for (")?;
                if let Some(init) = &for_stmt.initializer {
                    self.generate_lir_statement(init)?;
                }
                self.line(";")?;
                if let Some(cond) = &for_stmt.condition {
                    let cond_code = self.generate_lir_expression(cond)?;
                    self.line(&format!(" {}", cond_code))?;
                }
                self.line(";")?;
                if let Some(inc) = &for_stmt.increment {
                    let inc_code = self.generate_lir_expression(inc)?;
                    self.line(&format!(" {}", inc_code))?;
                }
                self.line(") {")?;
                self.indent();
                self.generate_lir_statement(&for_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let code = self.generate_lir_expression(expr)?;
                    self.line(&format!("return {};", code))?;
                } else {
                    self.line("return;")?;
                }
            }
            x_lir::Statement::Break => {
                self.line("break;")?;
            }
            x_lir::Statement::Continue => {
                self.line("continue;")?;
            }
            x_lir::Statement::Label(_name) => {
                // Rust doesn't support labels in this form, skip it
            }
            x_lir::Statement::Goto(target) => {
                self.line(&format!("goto {};", target))?;
            }
            x_lir::Statement::Compound(block) => {
                self.line("{")?;
                self.indent();
                self.generate_lir_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Empty => {}
            x_lir::Statement::Match(match_stmt) => {
                let expr = self.generate_lir_expression(&match_stmt.scrutinee)?;
                self.line(&format!("match {} {{", expr))?;
                self.indent();

                for case in &match_stmt.cases {
                    let pattern = self.generate_lir_pattern(&case.pattern)?;
                    let guard = if let Some(g) = &case.guard {
                        format!(" if {}", self.generate_lir_expression(g)?)
                    } else {
                        String::new()
                    };
                    self.line(&format!("{}{} => {{", pattern, guard))?;
                    self.indent();
                    self.generate_lir_block(&case.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Try(try_stmt) => {
                self.line("{")?;
                self.indent();
                self.line("let __result = (|| {")?;
                self.indent();
                self.generate_lir_block(&try_stmt.body)?;
                self.line("Ok(())")?;
                self.dedent();
                self.line("})();")?;
                self.line("match __result {")?;
                self.indent();

                for catch in &try_stmt.catch_clauses {
                    let var_name = catch.variable_name.as_deref().unwrap_or("_");
                    let ty = catch.exception_type.as_deref().unwrap_or("_");
                    self.line(&format!("Err({}: {}) => {{", var_name, ty))?;
                    self.indent();
                    self.generate_lir_block(&catch.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.line("Ok(_) => {},")?;
                self.dedent();
                self.line("}")?;

                if let Some(finally) = &try_stmt.finally_block {
                    self.generate_lir_block(finally)?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Declaration(_) => {
                // Already handled at top level
                // This shouldn't happen in LIR block anyway
            }
            x_lir::Statement::Switch(switch_stmt) => {
                self.generate_lir_switch(switch_stmt)?;
            }
        }
        Ok(())
    }

    /// Generate a switch statement
    fn generate_lir_switch(&mut self, switch_stmt: &x_lir::SwitchStatement) -> RustResult<()> {
        let expr = self.generate_lir_expression(&switch_stmt.expression)?;
        self.line(&format!("match {} {{", expr))?;
        self.indent();

        for case in &switch_stmt.cases {
            let value = self.generate_lir_expression(&case.value)?;
            self.line(&format!("{} => {{", value))?;
            self.indent();
            self.generate_lir_statement(&case.body)?;
            self.dedent();
            self.line("},")?;
        }

        if let Some(default_body) = &switch_stmt.default {
            self.line("_ => {")?;
            self.indent();
            self.generate_lir_statement(default_body)?;
            self.dedent();
            self.line("},")?;
        }

        self.dedent();
        self.line("}")?;

        Ok(())
    }

    /// Generate a LIR pattern
    fn generate_lir_pattern(&mut self, pattern: &x_lir::Pattern) -> RustResult<String> {
        let result = match pattern {
            x_lir::Pattern::Wildcard => Ok("_".to_string()),
            x_lir::Pattern::Variable(name) => Ok(name.clone()),
            x_lir::Pattern::Literal(lit) => Ok(self.generate_lir_literal(lit)),
            x_lir::Pattern::Constructor(name, patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.generate_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                if patterns.is_empty() {
                    Ok(format!("{}", name))
                } else {
                    Ok(format!("{}({})", name, pat_strs.join(", ")))
                }
            }
            x_lir::Pattern::Tuple(patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.generate_lir_pattern(p))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>(
                )?;
                Ok(format!("({},)", pat_strs.join(", ")))
            }
            x_lir::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| -> Result<String, x_codegen::CodeGenError> {
                        let p_str = self.generate_lir_pattern(p)?;
                        Ok(format!("{}: {}", n, p_str))
                    })
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(format!("{} {{ {} }}", name, field_strs.join(", ")))
            }
            x_lir::Pattern::Or(left, right) => {
                let left_str = self.generate_lir_pattern(left)?;
                let right_str = self.generate_lir_pattern(right)?;
                Ok(format!("{} | {}", left_str, right_str))
            }
        };
        result
    }

    /// Generate a LIR expression
    fn generate_lir_expression(&mut self, expr: &x_lir::Expression) -> RustResult<String> {
        let result = match expr {
            x_lir::Expression::Literal(lit) => Ok(self.generate_lir_literal(lit)),
            x_lir::Expression::Variable(name) => Ok(name.clone()),
            x_lir::Expression::Unary(op, inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                let op_str = match op {
                    x_lir::UnaryOp::Minus => "-",
                    x_lir::UnaryOp::Plus => "+",
                    x_lir::UnaryOp::Not => "!",
                    x_lir::UnaryOp::BitNot => "!",
                    x_lir::UnaryOp::PreIncrement => "++",
                    x_lir::UnaryOp::PreDecrement => "--",
                    x_lir::UnaryOp::PostIncrement => "++",
                    x_lir::UnaryOp::PostDecrement => "--",
                };
                let result = match op {
                    x_lir::UnaryOp::PostIncrement | x_lir::UnaryOp::PostDecrement => {
                        format!("{}{}", inner_code, op_str)
                    }
                    _ => format!("{}{}", op_str, inner_code),
                };
                Ok(result)
            }
            x_lir::Expression::Binary(op, left, right) => {
                let left_code = self.generate_lir_expression(left)?;
                let right_code = self.generate_lir_expression(right)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+",
                    x_lir::BinaryOp::Subtract => "-",
                    x_lir::BinaryOp::Multiply => "*",
                    x_lir::BinaryOp::Divide => "/",
                    x_lir::BinaryOp::Modulo => "%",
                    x_lir::BinaryOp::BitAnd => "&",
                    x_lir::BinaryOp::BitOr => "|",
                    x_lir::BinaryOp::BitXor => "^",
                    x_lir::BinaryOp::LeftShift => "<<",
                    x_lir::BinaryOp::RightShift => ">>",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>",
                    x_lir::BinaryOp::LessThan => "<",
                    x_lir::BinaryOp::LessThanEqual => "<=",
                    x_lir::BinaryOp::GreaterThan => ">",
                    x_lir::BinaryOp::GreaterThanEqual => ">=",
                    x_lir::BinaryOp::Equal => "==",
                    x_lir::BinaryOp::NotEqual => "!=",
                    x_lir::BinaryOp::LogicalAnd => "&&",
                    x_lir::BinaryOp::LogicalOr => "||",
                };
                Ok(format!("{} {} {}", left_code, op_str, right_code))
            }
            x_lir::Expression::Ternary(cond, then, else_) => {
                let cond_code = self.generate_lir_expression(cond)?;
                let then_code = self.generate_lir_expression(then)?;
                let else_code = self.generate_lir_expression(else_)?;
                Ok(format!("{} ? {} : {}", cond_code, then_code, else_code))
            }
            x_lir::Expression::Assign(target, value) => {
                // Check if the value is a void function call (println, print, etc.)
                if let x_lir::Expression::Call(callee, args) = value.as_ref() {
                    if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                        let name = fn_name.as_str();
                        // For void functions, just emit the call without assignment
                        if matches!(name, "println" | "print" | "eprintln" | "eprintln!" | "format") {
                            let args_code: Vec<String> = args
                                .iter()
                                .map(|arg| self.generate_lir_expression(arg))
                                .collect::<Result<_, _>>()?;
                            let call_str = match name {
                                "println" => format!("println!({})", args_code.join(", ")),
                                "print" => format!("print!({})", args_code.join(", ")),
                                "eprintln" | "eprintln!" => format!("eprintln!({})", args_code.join(", ")),
                                "format" => format!("format!({})", args_code.join(", ")),
                                _ => format!("{}({})", name, args_code.join(", ")),
                            };
                            return Ok(call_str);
                        }
                    }
                }
                let target_code = self.generate_lir_expression(target)?;
                let value_code = self.generate_lir_expression(value)?;
                Ok(format!("{} = {}", target_code, value_code))
            }
            x_lir::Expression::AssignOp(op, target, value) => {
                let target_code = self.generate_lir_expression(target)?;
                let value_code = self.generate_lir_expression(value)?;
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
                    x_lir::BinaryOp::RightShift => ">>=",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>=",
                    _ => "=", // fallback
                };
                Ok(format!("{} {} {}", target_code, op_str, value_code))
            }
            x_lir::Expression::Call(callee, args) => {
                let callee_code = self.generate_lir_expression(callee)?;
                let args_code: Vec<String> = args
                    .iter()
                    .map(|arg| self.generate_lir_expression(arg))
                    .collect::<Result<_, _>>()?;

                // Convert common X built-in functions to Rust equivalents
                let callee_str = callee_code.as_str();
                let result = if callee_str == "println" {
                    format!("println!({})", args_code.join(", "))
                } else if callee_str == "print" {
                    format!("print!({})", args_code.join(", "))
                } else {
                    format!("{}({})", callee_code, args_code.join(", "))
                };
                Ok(result)
            }
            x_lir::Expression::Index(base, index) => {
                let base_code = self.generate_lir_expression(base)?;
                let index_code = self.generate_lir_expression(index)?;
                Ok(format!("{}[{}]", base_code, index_code))
            }
            x_lir::Expression::Member(base, field) => {
                let base_code = self.generate_lir_expression(base)?;
                Ok(format!("{}.{}", base_code, field))
            }
            x_lir::Expression::PointerMember(base, field) => {
                let base_code = self.generate_lir_expression(base)?;
                Ok(format!("{}->{}", base_code, field))
            }
            x_lir::Expression::AddressOf(inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                Ok(format!("&{}", inner_code))
            }
            x_lir::Expression::Dereference(inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                Ok(format!("*{}", inner_code))
            }
            x_lir::Expression::Cast(ty, inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                let ty_str = self.lir_type_to_rust(ty);
                Ok(format!("{} as {}", inner_code, ty_str))
            }
            x_lir::Expression::SizeOf(ty) => {
                let ty_str = self.lir_type_to_rust(ty);
                Ok(format!("std::mem::size_of::<{}>()", ty_str))
            }
            x_lir::Expression::SizeOfExpr(expr) => {
                let expr_code = self.generate_lir_expression(expr)?;
                Ok(format!("std::mem::size_of_val(&{})", expr_code))
            }
            x_lir::Expression::AlignOf(ty) => {
                let ty_str = self.lir_type_to_rust(ty);
                Ok(format!("std::mem::align_of::<{}>()", ty_str))
            }
            x_lir::Expression::Comma(exprs) => {
                let expr_codes: Vec<String> = exprs
                    .iter()
                    .map(|e| self.generate_lir_expression(e))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(expr_codes.join(", "))
            }
            x_lir::Expression::Parenthesized(inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                Ok(format!("({})", inner_code))
            }
            x_lir::Expression::InitializerList(inits) => {
                let init_codes: Vec<String> = inits
                    .iter()
                    .map(|init| self.generate_lir_initializer(init))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(format!("{{{}}}", init_codes.join(", ")))
            }
            x_lir::Expression::CompoundLiteral(ty, inits) => {
                let ty_str = self.lir_type_to_rust(ty);
                let init_codes: Vec<String> = inits
                    .iter()
                    .map(|init| self.generate_lir_initializer(init))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(format!("{} {{ {} }}", ty_str, init_codes.join(", ")))
            }
        };
        result
    }

    /// Generate a LIR literal
    fn generate_lir_literal(&self, lit: &x_lir::Literal) -> String {
        match lit {
            x_lir::Literal::Integer(v) => v.to_string(),
            x_lir::Literal::UnsignedInteger(v) => format!("{}u", v),
            x_lir::Literal::Long(v) => format!("{}i64", v),
            x_lir::Literal::UnsignedLong(v) => format!("{}u64", v),
            x_lir::Literal::LongLong(v) => format!("{}i64", v),
            x_lir::Literal::UnsignedLongLong(v) => format!("{}u64", v),
            x_lir::Literal::Float(v) => format!("{}f32", v),
            x_lir::Literal::Double(v) => v.to_string(),
            x_lir::Literal::Bool(v) => v.to_string(),
            x_lir::Literal::Char(c) => format!("'{}'", c),
            x_lir::Literal::String(s) => format!("\"{}\"", s),
            x_lir::Literal::NullPointer => "std::ptr::null_mut()".to_string(),
        }
    }

    /// Generate a LIR initializer
    fn generate_lir_initializer(&mut self, init: &x_lir::Initializer) -> RustResult<String> {
        let result = match init {
            x_lir::Initializer::Expression(expr) => self.generate_lir_expression(expr),
            x_lir::Initializer::List(list) => {
                let items: Vec<String> = list
                    .iter()
                    .map(|i| self.generate_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", items.join(", ")))
            }
            x_lir::Initializer::Named(name, init) => {
                let init_code = self.generate_lir_initializer(init)?;
                Ok(format!(".{name} = {init_code}"))
            }
            x_lir::Initializer::Indexed(idx, init) => {
                let idx_code = self.generate_lir_expression(idx)?;
                let init_code = self.generate_lir_initializer(init)?;
                Ok(format!("[{idx_code}] = {init_code}"))
            }
        };
        result
    }

    /// Convert LIR type to Rust type string
    fn lir_type_to_rust(&self, ty: &x_lir::Type) -> String {
        match ty {
            x_lir::Type::Void => "()".to_string(),
            x_lir::Type::Bool => "bool".to_string(),
            x_lir::Type::Char => "char".to_string(),
            x_lir::Type::Schar => "i8".to_string(),
            x_lir::Type::Uchar => "u8".to_string(),
            x_lir::Type::Short => "i16".to_string(),
            x_lir::Type::Ushort => "u16".to_string(),
            x_lir::Type::Int => "i32".to_string(),
            x_lir::Type::Uint => "u32".to_string(),
            x_lir::Type::Long => "i64".to_string(),
            x_lir::Type::Ulong => "u64".to_string(),
            x_lir::Type::LongLong => "i64".to_string(),
            x_lir::Type::UlongLong => "u64".to_string(),
            x_lir::Type::Float => "f32".to_string(),
            x_lir::Type::Double => "f64".to_string(),
            x_lir::Type::LongDouble => "f128".to_string(),
            x_lir::Type::Size => "usize".to_string(),
            x_lir::Type::Ptrdiff => "isize".to_string(),
            x_lir::Type::Intptr => "isize".to_string(),
            x_lir::Type::Uintptr => "usize".to_string(),
            x_lir::Type::Pointer(inner) => {
                let inner_str = self.lir_type_to_rust(inner);
                format!("*mut {}", inner_str)
            }
            x_lir::Type::Array(inner, None) => {
                let inner_str = self.lir_type_to_rust(inner);
                format!("Vec<{}>", inner_str)
            }
            x_lir::Type::Array(inner, Some(size)) => {
                let inner_str = self.lir_type_to_rust(inner);
                format!("[{}; {}]", inner_str, size)
            }
            x_lir::Type::FunctionPointer(return_type, param_types) => {
                let params: Vec<String> = param_types
                    .iter()
                    .map(|t| self.lir_type_to_rust(t))
                    .collect();
                let ret = self.lir_type_to_rust(return_type);
                format!("fn({}) -> {}", params.join(", "), ret)
            }
            x_lir::Type::Named(name) => name.clone(),
            x_lir::Type::Qualified(quals, inner) => {
                let mut inner_str = self.lir_type_to_rust(inner);
                if quals.is_const {
                    inner_str = format!("const {}", inner_str);
                }
                inner_str
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_parser::parser::XParser;

    #[test]
    fn test_empty_program_generation() {
        let source = "";
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, std::path::PathBuf::from("output.rs"));
    }

    #[test]
    fn test_hello_world_generation() {
        let source = r#"
            function main() {
                print("Hello, World!")
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("fn main()"));
    }

    #[test]
    fn test_variable_generation() {
        let source = r#"
            function main() {
                let x = 42
                let mut y = 3.14
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("let x = 42"), "Content was: {}", content);
        assert!(
            content.contains("let mut y = 3.14"),
            "Content was: {}",
            content
        );
    }

    #[test]
    fn test_function_generation() {
        let source = r#"
            function add(a: Int, b: Int) -> Int {
                return a + b
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("fn add(a: i32, b: i32) -> i32"));
        assert!(content.contains("return a + b"));
    }

    #[test]
    fn test_extern_function_generation() {
        let source = r#"
            foreign "C" function puts(s: CString) -> CInt
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("extern \"C\""));
        assert!(content.contains("fn puts"));
    }

    #[test]
    fn test_unsafe_block_generation() {
        let source = r#"
            function main() {
                unsafe {
                    let x = 42
                }
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("unsafe {"));
        assert!(content.contains("let x = 42"));
    }

    #[test]
    fn test_config_default() {
        let config = RustBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
    }

    #[test]
    fn test_config_with_options() {
        let config = RustBackendConfig {
            output_dir: Some(std::path::PathBuf::from("/tmp")),
            optimize: true,
            debug_info: false,
        };
        assert!(config.optimize);
        assert!(!config.debug_info);
        assert!(config.output_dir.is_some());
    }

    #[test]
    fn test_if_statement_generation() {
        let source = r#"
            function main() {
                if true {
                    print("yes")
                } else {
                    print("no")
                }
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("if true"));
    }

    #[test]
    fn test_while_loop_generation() {
        let source = r#"
            function main() {
                while true {
                    print("loop")
                }
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("while true"));
    }

    #[test]
    fn test_for_loop_generation() {
        let source = r#"
            function main() {
                for i in range(5) {
                    print(i)
                }
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("for i in"));
    }

    #[test]
    fn test_array_generation() {
        let source = r#"
            function main() {
                let arr = [1, 2, 3]
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("vec!"));
    }

    #[test]
    fn test_class_generation() {
        // Rust backend uses struct for classes
        let source = r#"
            class Person {
                name: String
                age: Int
            }
        "#;
        let parser = XParser::new();
        let program = parser.parse(source).unwrap();

        let config = RustBackendConfig::default();
        let mut backend = RustBackend::new(config);
        let output = backend.generate_from_ast(&program).unwrap();

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("struct Person"));
    }
}
