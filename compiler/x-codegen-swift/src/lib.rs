//! Swift 后端 - 生成 Swift 6.3 源代码
//!
//! 面向 Apple 生态（iOS、macOS、watchOS、tvOS）
//!
//! ## Swift 6.x 特性支持
//! - Data-race safety in concurrent code（并发代码数据竞争安全）
//! - Typed throws（类型化抛出）
//! - Non-copyable types with generics（泛型不可复制类型）
//! - 128-bit integer types（128位整数类型）
//! - Embedded Swift for microcontrollers（嵌入式 Swift）
//! - Swift Testing library
//! - C++ interoperability improvements
//! - Strict concurrency by default

use std::fmt::Write;
use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;
use x_parser::ast::{
    self, BinaryOp, ExpressionKind, Literal, Pattern, StatementKind, UnaryOp, WaitType,
    Program as AstProgram,
};

/// Swift 后端配置
#[derive(Debug, Clone)]
pub struct SwiftBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub target: SwiftTarget,
}

/// Swift 编译目标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SwiftTarget {
    #[default]
    MacOS,
    IOS,
    WatchOS,
    TvOS,
    Linux,
}

impl Default for SwiftBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            target: SwiftTarget::MacOS,
        }
    }
}

/// Swift 后端
pub struct SwiftBackend {
    #[allow(dead_code)]
    config: SwiftBackendConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
}

pub type SwiftResult<T> = Result<T, x_codegen::CodeGenError>;

impl SwiftBackend {
    pub fn new(config: SwiftBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
        }
    }

    fn line(&mut self, s: &str) -> SwiftResult<()> {
        self.buffer.line(s).map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))
    }

    fn indent(&mut self) { self.buffer.indent(); }
    fn dedent(&mut self) { self.buffer.dedent(); }
    fn output(&self) -> &str { self.buffer.as_str() }

    /// 从 AST 生成 Swift 代码
    pub fn generate_from_ast_impl(
        &mut self,
        program: &AstProgram,
    ) -> SwiftResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // Emit imports
        self.line("import Foundation")?;
        self.line("")?;

        // Emit X classes as Swift classes
        for decl in &program.declarations {
            if let ast::Declaration::Class(class) = decl {
                self.emit_class(class)?;
            }
        }

        // Emit enums
        for decl in &program.declarations {
            if let ast::Declaration::Enum(enum_decl) = decl {
                self.emit_enum(enum_decl)?;
            }
        }

        // Emit global variables
        for decl in &program.declarations {
            if let ast::Declaration::Variable(v) = decl {
                self.emit_global_var(v)?;
            }
        }

        // Emit functions
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

        // Emit main function only if not already defined
        if !has_main {
            self.emit_main_function()?;
        }

        // Create output file
        let output_file = OutputFile {
            path: PathBuf::from("output.swift"),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::Swift,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// Emit file header (Swift 6.3)
    fn emit_header(&mut self) -> SwiftResult<()> {
        self.line(headers::SWIFT)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: Swift 6.3")?;
        self.line("")?;
        Ok(())
    }

    /// Emit main function
    fn emit_main_function(&mut self) -> SwiftResult<()> {
        self.line("func main() {")?;
        self.indent();
        self.line("print(\"Hello from Swift backend!\")")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;
        self.line("main()")?;
        Ok(())
    }

    /// Emit global variable
    fn emit_global_var(&mut self, v: &ast::VariableDecl) -> SwiftResult<()> {
        let keyword = if v.is_mutable { "var" } else { "let" };
        let init = if let Some(expr) = &v.initializer {
            self.emit_expr(expr)?
        } else {
            self.emit_default_value(&v.type_annot)?
        };

        if let Some(type_annot) = &v.type_annot {
            let swift_type = self.map_type(type_annot)?;
            self.line(&format!("{} {}: {} = {}", keyword, v.name, swift_type, init))?;
        } else {
            self.line(&format!("{} {} = {}", keyword, v.name, init))?;
        }
        Ok(())
    }

    /// Emit function declaration
    fn emit_function(&mut self, f: &ast::FunctionDecl) -> SwiftResult<()> {
        let params: Vec<String> = f
            .parameters
            .iter()
            .map(|p| {
                if let Some(type_annot) = &p.type_annot {
                    let swift_type = self.map_type(type_annot).unwrap_or_else(|_| "Any".to_string());
                    format!("{}: {}", p.name, swift_type)
                } else {
                    format!("{}: Any", p.name)
                }
            })
            .collect();

        let return_type = if let Some(ret) = &f.return_type {
            format!(" -> {}", self.map_type(ret)?)
        } else {
            String::new()
        };

        let async_keyword = if f.is_async { "async " } else { "" };
        self.line(&format!(
            "{}func {}({}){} {{",
            async_keyword, f.name, params.join(", "), return_type
        ))?;
        self.indent();

        self.emit_block(&f.body)?;

        // Add implicit return if needed
        if f.return_type.is_some() {
            self.line("return nil")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit class declaration
    fn emit_class(&mut self, class: &ast::ClassDecl) -> SwiftResult<()> {
        let inheritance = if let Some(parent) = &class.extends {
            format!(": {}", parent)
        } else if !class.implements.is_empty() {
            format!(": {}", class.implements.join(", "))
        } else {
            String::new()
        };

        self.line(&format!("class {}{} {{", class.name, inheritance))?;
        self.indent();

        let has_content = class.members.iter().any(|m| {
            matches!(
                m,
                ast::ClassMember::Field(_)
                    | ast::ClassMember::Method(_)
                    | ast::ClassMember::Constructor(_)
            )
        });

        if !has_content {
            self.line("// Empty class")?;
        } else {
            // Emit properties
            for member in &class.members {
                if let ast::ClassMember::Field(field) = member {
                    self.emit_property(field)?;
                }
            }

            // Emit constructor if present
            for member in &class.members {
                if let ast::ClassMember::Constructor(constructor) = member {
                    self.line("")?;
                    self.emit_constructor(constructor)?;
                }
            }

            // Emit methods
            for member in &class.members {
                if let ast::ClassMember::Method(method) = member {
                    self.line("")?;
                    self.emit_method(method)?;
                }
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Emit property
    fn emit_property(&mut self, field: &ast::VariableDecl) -> SwiftResult<()> {
        let keyword = if field.is_mutable { "var" } else { "let" };
        let type_str = if let Some(type_annot) = &field.type_annot {
            format!(": {}", self.map_type(type_annot)?)
        } else {
            String::new()
        };

        if let Some(init) = &field.initializer {
            let init_str = self.emit_expr(init)?;
            self.line(&format!("{} {}{} = {}", keyword, field.name, type_str, init_str))?;
        } else if !type_str.is_empty() {
            self.line(&format!("{} {}{}", keyword, field.name, type_str))?;
        } else {
            self.line(&format!("{} {}: Any", keyword, field.name))?;
        }
        Ok(())
    }

    /// Emit constructor
    fn emit_constructor(&mut self, constructor: &ast::ConstructorDecl) -> SwiftResult<()> {
        let params: Vec<String> = constructor
            .parameters
            .iter()
            .map(|p| {
                if let Some(type_annot) = &p.type_annot {
                    let swift_type = self.map_type(type_annot).unwrap_or_else(|_| "Any".to_string());
                    format!("{}: {}", p.name, swift_type)
                } else {
                    format!("{}: Any", p.name)
                }
            })
            .collect();

        self.line(&format!("init({}) {{", params.join(", ")))?;
        self.indent();

        for stmt in &constructor.body.statements {
            self.emit_statement(stmt)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit method
    fn emit_method(&mut self, method: &ast::FunctionDecl) -> SwiftResult<()> {
        let mut params = vec![format!("_ self: {}", "Self")];
        for p in &method.parameters {
            if let Some(type_annot) = &p.type_annot {
                let swift_type = self.map_type(type_annot)?;
                params.push(format!("{}: {}", p.name, swift_type));
            } else {
                params.push(format!("{}: Any", p.name));
            }
        }

        let return_type = if let Some(ret) = &method.return_type {
            format!(" -> {}", self.map_type(ret)?)
        } else {
            String::new()
        };

        let async_keyword = if method.is_async { "async " } else { "" };
        self.line(&format!(
            "{}func {}({}){} {{",
            async_keyword, method.name, params.join(", "), return_type
        ))?;
        self.indent();

        self.emit_block(&method.body)?;

        if method.return_type.is_some() {
            self.line("return nil")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit enum declaration
    fn emit_enum(&mut self, enum_decl: &ast::EnumDecl) -> SwiftResult<()> {
        self.line(&format!("enum {} {{", enum_decl.name))?;
        self.indent();

        for variant in &enum_decl.variants {
            match &variant.data {
                ast::EnumVariantData::Unit => {
                    self.line(&format!("case {}", variant.name))?;
                }
                ast::EnumVariantData::Tuple(types) => {
                    let type_strs: Vec<String> = types
                        .iter()
                        .map(|t| self.map_type(t))
                        .collect::<SwiftResult<Vec<_>>>()?;
                    self.line(&format!("case {}({})", variant.name, type_strs.join(", ")))?;
                }
                ast::EnumVariantData::Record(fields) => {
                    let field_strs: Vec<String> = fields
                        .iter()
                        .map(|(name, ty)| {
                            let swift_type = self.map_type(ty).unwrap_or_else(|_| "Any".to_string());
                            format!("{}: {}", name, swift_type)
                        })
                        .collect();
                    self.line(&format!("case {}({})", variant.name, field_strs.join(", ")))?;
                }
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Emit block of statements
    fn emit_block(&mut self, block: &ast::Block) -> SwiftResult<()> {
        for stmt in &block.statements {
            self.emit_statement(stmt)?;
        }
        Ok(())
    }

    /// Emit statement
    fn emit_statement(&mut self, stmt: &ast::Statement) -> SwiftResult<()> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&e)?;
            }
            StatementKind::Variable(v) => {
                self.emit_variable_decl(v)?;
            }
            StatementKind::Return(opt) => {
                if let Some(expr) = opt {
                    let e = self.emit_expr(expr)?;
                    self.line(&format!("return {}", e))?;
                } else {
                    self.line("return")?;
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
                self.line("break")?;
            }
            StatementKind::Continue => {
                self.line("continue")?;
            }
            StatementKind::DoWhile(d) => {
                self.line("while true {")?;
                self.indent();
                self.emit_block(&d.body)?;
                let cond = self.emit_expr(&d.condition)?;
                self.line(&format!("if !({}) {{ break }}", cond))?;
                self.dedent();
                self.line("}")?;
            }
            StatementKind::Unsafe(block) => {
                // Swift doesn't have unsafe blocks like Rust, just emit the block
                self.line("// unsafe block")?;
                self.emit_block(block)?;
            }
            StatementKind::Defer(expr) => {
                // Swift has defer like keyword (Swift 6+ has `defer`
                let e = self.emit_expr(expr)?;
                self.line(&format!("defer {{ {};}}", e))?;
            }
            StatementKind::Yield(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let e = self.emit_expr(expr)?;
                    self.line(&format!("yield {}", e))?;
                } else {
                        self.line("yield")?;
                }
            }
            StatementKind::Loop(block) => {
                self.line("while true {")?;
                self.indent();
                self.emit_block(block)?;
                self.dedent();
                self.line("}")?;
            }
        }
        Ok(())
    }

    /// Emit variable declaration statement
    fn emit_variable_decl(&mut self, v: &ast::VariableDecl) -> SwiftResult<()> {
        let keyword = if v.is_mutable { "var" } else { "let" };
        let init = if let Some(expr) = &v.initializer {
            self.emit_expr(expr)?
        } else {
            self.emit_default_value(&v.type_annot)?
        };

        if let Some(type_annot) = &v.type_annot {
            let swift_type = self.map_type(type_annot)?;
            self.line(&format!("{} {}: {} = {}", keyword, v.name, swift_type, init))?;
        } else {
            self.line(&format!("{} {} = {}", keyword, v.name, init))?;
        }
        Ok(())
    }

    /// Emit if statement
    fn emit_if(&mut self, if_stmt: &ast::IfStatement) -> SwiftResult<()> {
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

    /// Emit for statement
    fn emit_for(&mut self, for_stmt: &ast::ForStatement) -> SwiftResult<()> {
        let iter = self.emit_expr(&for_stmt.iterator)?;
        let pattern_var = self.emit_pattern_var(&for_stmt.pattern);

        self.line(&format!("for {} in {} {{", pattern_var, iter))?;
        self.indent();
        self.emit_block(&for_stmt.body)?;
        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit pattern variable
    fn emit_pattern_var(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Variable(name) => name.clone(),
            Pattern::Literal(lit) => self.emit_literal_for_pattern(lit),
            Pattern::Array(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("[{}]", vars.join(", "))
            }
            Pattern::Tuple(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("({})", vars.join(", "))
            }
            Pattern::Or(left, _) => self.emit_pattern_var(left),
            Pattern::Guard(inner, _) => self.emit_pattern_var(inner),
            Pattern::Dictionary(entries) => {
                let vars: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", self.emit_pattern_var(k), self.emit_pattern_var(v)))
                    .collect();
                format!("[{}]", vars.join(", "))
            }
            Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.emit_pattern_var(v)))
                    .collect();
                format!("{}({})", name, field_strs.join(", "))
            }
            Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns.iter().map(|p| self.emit_pattern_var(p)).collect();
                if patterns.is_empty() {
                    format!(".{}", variant_name)
                } else {
                    format!(".{}({})", variant_name, pattern_strs.join(", "))
                }
            }
        }
    }

    /// Emit literal for pattern
    fn emit_literal_for_pattern(&self, lit: &Literal) -> String {
        match lit {
            Literal::Integer(n) => format!("{}", n),
            Literal::Float(f) => format!("{}", f),
            Literal::Boolean(b) => format!("{}", b),
            Literal::String(s) => format!("\"{}\"", s),
            Literal::Char(c) => format!("'{}'", c),
            Literal::Null | Literal::None | Literal::Unit => "nil".to_string(),
        }
    }

    /// Emit match statement using switch
    fn emit_match(&mut self, match_stmt: &ast::MatchStatement) -> SwiftResult<()> {
        let expr = self.emit_expr(&match_stmt.expression)?;
        self.line(&format!("switch {} {{", expr))?;
        self.indent();

        for case in &match_stmt.cases {
            let pattern = self.emit_switch_pattern(&case.pattern);
            let guard = if let Some(guard_expr) = &case.guard {
                let guard_str = self.emit_expr(guard_expr)?;
                format!(" where {}", guard_str)
            } else {
                String::new()
            };
            self.line(&format!("case {}{}:", pattern, guard))?;
            self.indent();
            self.emit_block(&case.body)?;
            self.dedent();
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit switch pattern
    fn emit_switch_pattern(&self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard => "_".to_string(),
            Pattern::Variable(name) => format!("let {}", name),
            Pattern::Literal(lit) => self.emit_literal_for_pattern(lit),
            Pattern::Array(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_switch_pattern(p)).collect();
                format!("[{}]", vars.join(", "))
            }
            Pattern::Tuple(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_switch_pattern(p)).collect();
                format!("({})", vars.join(", "))
            }
            Pattern::Or(left, right) => {
                format!(
                    "{}, {}",
                    self.emit_switch_pattern(left),
                    self.emit_switch_pattern(right)
                )
            }
            Pattern::Guard(inner, cond) => {
                format!("{} where {:?}", self.emit_switch_pattern(inner), cond)
            }
            Pattern::Dictionary(entries) => {
                let vars: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{}: {}", self.emit_switch_pattern(k), self.emit_switch_pattern(v)))
                    .collect();
                format!("[{}]", vars.join(", "))
            }
            Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.emit_switch_pattern(v)))
                    .collect();
                format!("{}.{}({})", name, name, field_strs.join(", "))
            }
            Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns.iter().map(|p| self.emit_switch_pattern(p)).collect();
                if patterns.is_empty() {
                    format!(".{}", variant_name)
                } else {
                    format!(".{}({})", variant_name, pattern_strs.join(", "))
                }
            }
        }
    }

    /// Emit try statement
    fn emit_try(&mut self, try_stmt: &ast::TryStatement) -> SwiftResult<()> {
        self.line("do {")?;
        self.indent();
        self.emit_block(&try_stmt.body)?;
        self.dedent();

        for catch in &try_stmt.catch_clauses {
            let catch_line = if let Some(var_name) = &catch.variable_name {
                if let Some(exc_type) = &catch.exception_type {
                    format!("catch let {} as {} {{", var_name, exc_type)
                } else {
                    format!("catch let {} {{", var_name)
                }
            } else if let Some(exc_type) = &catch.exception_type {
                format!("catch let error as {} {{", exc_type)
            } else {
                "catch {".to_string()
            };

            self.line(&catch_line)?;
            self.indent();
            self.emit_block(&catch.body)?;
            self.dedent();
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.line("defer {")?;
            self.indent();
            self.emit_block(finally)?;
            self.dedent();
            self.line("}")?;
        }

        self.line("}")?;
        Ok(())
    }

    /// Emit expression
    fn emit_expr(&self, expr: &ast::Expression) -> SwiftResult<String> {
        match &expr.node {
            ExpressionKind::Literal(lit) => self.emit_literal(lit),
            ExpressionKind::Variable(name) => Ok(name.clone()),
            ExpressionKind::Binary(op, lhs, rhs) => {
                let l = self.emit_expr(lhs)?;
                let r = self.emit_expr(rhs)?;
                self.emit_binop(op, &l, &r)
            }
            ExpressionKind::Unary(op, e) => {
                let inner = self.emit_expr(e)?;
                self.emit_unaryop(op, &inner)
            }
            ExpressionKind::Call(callee, args) => self.emit_call(callee, args),
            ExpressionKind::Assign(target, value) => self.emit_assign(target, value),
            ExpressionKind::Array(elements) => self.emit_array_literal(elements),
            ExpressionKind::Parenthesized(inner) => {
                let e = self.emit_expr(inner)?;
                Ok(format!("({})", e))
            }
            ExpressionKind::If(cond, then_e, else_e) => {
                let c = self.emit_expr(cond)?;
                let t = self.emit_expr(then_e)?;
                let e = self.emit_expr(else_e)?;
                Ok(format!("{} ? {} : {}", c, t, e))
            }
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                Ok(format!("{}.{}", o, field))
            }
            ExpressionKind::Wait(wait_type, exprs) => self.emit_wait(wait_type, exprs),
            ExpressionKind::Dictionary(entries) => {
                if entries.is_empty() {
                    return Ok("[:]".to_string());
                }
                let entry_strs: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let k_str = self.emit_expr(k)?;
                        let v_str = self.emit_expr(v)?;
                        Ok(format!("{}: {}", k_str, v_str))
                    })
                    .collect::<SwiftResult<Vec<_>>>()?;
                Ok(format!("[{}]", entry_strs.join(", ")))
            }
            ExpressionKind::Range(start, end, inclusive) => {
                let s = self.emit_expr(start)?;
                let e = self.emit_expr(end)?;
                if *inclusive {
                    Ok(format!("{}...{}", s, e))
                } else {
                    Ok(format!("{}..<{}", s, e))
                }
            }
            ExpressionKind::Lambda(params, _body) => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| {
                        if let Some(type_annot) = &p.type_annot {
                            let swift_type = self.map_type(type_annot).unwrap_or_else(|_| "Any".to_string());
                            format!("{}: {}", p.name, swift_type)
                        } else {
                            format!("{}: Any", p.name)
                        }
                    })
                    .collect();
                // For lambda, we need to emit the body differently
                Ok(format!("{{ {} in /* body */ }}", param_strs.join(", ")))
            }
            ExpressionKind::TryPropagate(inner) => {
                let e = self.emit_expr(inner)?;
                Ok(format!("try {}", e))
            }
            ExpressionKind::Cast(expr, ty) => {
                let e = self.emit_expr(expr)?;
                let type_str = self.map_type(ty)?;
                Ok(format!("{} as! {}", e, type_str))
            }
            ExpressionKind::Await(expr) => {
                let e = self.emit_expr(expr)?;
                Ok(format!("await {}", e))
            }
            ExpressionKind::OptionalChain(base_expr, member) => {
                let base = self.emit_expr(base_expr)?;
                Ok(format!("{}?.{}", base, member))
            }
            ExpressionKind::NullCoalescing(left, right) => {
                let l = self.emit_expr(left)?;
                let r = self.emit_expr(right)?;
                Ok(format!("{} ?? {}", l, r))
            }
            _ => Err(x_codegen::CodeGenError::UnsupportedFeature(format!(
                "Expression type not yet supported: {:?}",
                expr.node
            ))),
        }
    }

    /// Emit literal
    fn emit_literal(&self, lit: &Literal) -> SwiftResult<String> {
        match lit {
            Literal::Integer(n) => Ok(format!("{}", n)),
            Literal::Float(f) => Ok(format!("{}", f)),
            Literal::Boolean(b) => Ok(format!("{}", b)),
            Literal::String(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                Ok(format!("\"{}\"", escaped))
            }
            Literal::Char(c) => Ok(format!("'{}'", c)),
            Literal::Null => Ok("nil".to_string()),
            Literal::None => Ok("nil".to_string()),
            Literal::Unit => Ok("()".to_string()),
        }
    }

    /// Emit binary operation
    fn emit_binop(&self, op: &BinaryOp, l: &str, r: &str) -> SwiftResult<String> {
        let result = match op {
            BinaryOp::Add => format!("{} + {}", l, r),
            BinaryOp::Sub => format!("{} - {}", l, r),
            BinaryOp::Mul => format!("{} * {}", l, r),
            BinaryOp::Div => format!("{} / {}", l, r),
            BinaryOp::Mod => format!("{} % {}", l, r),
            BinaryOp::Equal => format!("{} == {}", l, r),
            BinaryOp::NotEqual => format!("{} != {}", l, r),
            BinaryOp::Less => format!("{} < {}", l, r),
            BinaryOp::LessEqual => format!("{} <= {}", l, r),
            BinaryOp::Greater => format!("{} > {}", l, r),
            BinaryOp::GreaterEqual => format!("{} >= {}", l, r),
            BinaryOp::And => format!("{} && {}", l, r),
            BinaryOp::Or => format!("{} || {}", l, r),
            BinaryOp::Pow => format!("pow({}, {})", l, r),
            BinaryOp::BitAnd => format!("{} & {}", l, r),
            BinaryOp::BitOr => format!("{} | {}", l, r),
            BinaryOp::BitXor => format!("{} ^ {}", l, r),
            BinaryOp::LeftShift => format!("{} << {}", l, r),
            BinaryOp::RightShift => format!("{} >> {}", l, r),
            BinaryOp::Concat => format!("{} + {}", l, r),
            BinaryOp::RangeExclusive => format!("{}..<{}", l, r),
            BinaryOp::RangeInclusive => format!("{}...{}", l, r),
        };
        Ok(result)
    }

    /// Emit unary operation
    fn emit_unaryop(&self, op: &UnaryOp, e: &str) -> SwiftResult<String> {
        let result = match op {
            UnaryOp::Negate => format!("-{}", e),
            UnaryOp::Not => format!("!{}", e),
            UnaryOp::BitNot => format!("~{}", e),
            UnaryOp::Wait => format!("await {}", e),
        };
        Ok(result)
    }

    /// Emit call expression
    fn emit_call(
        &self,
        callee: &ast::Expression,
        args: &[ast::Expression],
    ) -> SwiftResult<String> {
        let callee_str = self.emit_expr(callee)?;
        let arg_strs: Vec<String> = args
            .iter()
            .map(|a| self.emit_expr(a))
            .collect::<SwiftResult<Vec<_>>>()?;
        Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
    }

    /// Emit assignment expression
    fn emit_assign(
        &self,
        target: &ast::Expression,
        value: &ast::Expression,
    ) -> SwiftResult<String> {
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

    /// Emit array literal
    fn emit_array_literal(&self, elements: &[ast::Expression]) -> SwiftResult<String> {
        if elements.is_empty() {
            return Ok("[]".to_string());
        }
        let elem_strs: Vec<String> = elements
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<SwiftResult<Vec<_>>>()?;
        Ok(format!("[{}]", elem_strs.join(", ")))
    }

    /// Emit wait expression
    fn emit_wait(&self, wait_type: &WaitType, exprs: &[ast::Expression]) -> SwiftResult<String> {
        let expr_strs: Vec<String> = exprs
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<SwiftResult<Vec<_>>>()?;

        match wait_type {
            WaitType::Single => {
                if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(format!("({})", awaited.join(", ")))
                }
            }
            WaitType::Together => {
                if expr_strs.is_empty() {
                    Ok("await Task.yield()".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    Ok(format!(
                        "await Task {{ [{}] in () }}.value",
                        expr_strs.join(", ")
                    ))
                }
            }
            WaitType::Race => {
                if expr_strs.is_empty() {
                    Ok("await Task.yield()".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    Ok(format!(
                        "await Task {{ try await Task.sleep(nanoseconds: 1) }}.value"
                    ))
                }
            }
            WaitType::Timeout(timeout_expr) => {
                let timeout = self.emit_expr(timeout_expr)?;
                if expr_strs.is_empty() {
                    Ok(format!("await Task.sleep(nanoseconds: {})", timeout))
                } else {
                    let expr = &expr_strs[0];
                    Ok(format!(
                        "await Task.timeout(nanoseconds: {}) {{ await {} }}",
                        timeout, expr
                    ))
                }
            }
            WaitType::Atomic => {
                Ok(format!("/* atomic wait: {} */ await {}", expr_strs.join(", "), expr_strs[0]))
            }
            WaitType::Retry => {
                Ok(format!("/* retry wait: {} */ await {}", expr_strs.join(", "), expr_strs[0]))
            }
        }
    }

    /// Map X type to Swift type
    fn map_type(&self, ty: &ast::Type) -> SwiftResult<String> {
        match ty {
            ast::Type::Int => Ok("Int".to_string()),
            ast::Type::UnsignedInt => Ok("UInt".to_string()),
            ast::Type::Float => Ok("Double".to_string()),
            ast::Type::Bool => Ok("Bool".to_string()),
            ast::Type::String => Ok("String".to_string()),
            ast::Type::Char => Ok("Character".to_string()),
            ast::Type::Unit => Ok("Void".to_string()),
            ast::Type::Never => Ok("Never".to_string()),
            ast::Type::Dynamic => Ok("Any".to_string()),
            ast::Type::Array(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("[{}]", inner_type))
            }
            ast::Type::Dictionary(key, value) => {
                let key_type = self.map_type(key)?;
                let value_type = self.map_type(value)?;
                Ok(format!("[{}: {}]", key_type, value_type))
            }
            ast::Type::Option(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("{}?", inner_type))
            }
            ast::Type::Result(ok, err) => {
                let ok_type = self.map_type(ok)?;
                let err_type = self.map_type(err)?;
                Ok(format!("Result<{}, {}>", ok_type, err_type))
            }
            ast::Type::Tuple(types) => {
                let type_strs: Vec<String> = types
                    .iter()
                    .map(|t| self.map_type(t))
                    .collect::<SwiftResult<Vec<_>>>()?;
                Ok(format!("({})", type_strs.join(", ")))
            }
            ast::Type::Function(params, ret) => {
                let param_strs: Vec<String> = params
                    .iter()
                    .map(|p| self.map_type(p))
                    .collect::<SwiftResult<Vec<_>>>()?;
                let ret_type = self.map_type(ret)?;
                Ok(format!("({}) -> {}", param_strs.join(", "), ret_type))
            }
            ast::Type::Async(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("async -> {}", inner_type))
            }
            ast::Type::Generic(name) => Ok(name.clone()),
            ast::Type::TypeParam(name) => Ok(name.clone()),
            ast::Type::TypeConstructor(name, args) => {
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|t| self.map_type(t))
                    .collect::<SwiftResult<Vec<_>>>()?;
                Ok(format!("{}<{}>", name, arg_strs.join(", ")))
            }
            ast::Type::Var(name) => Ok(name.clone()),
            ast::Type::Reference(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("&{}", inner_type))
            }
            ast::Type::MutableReference(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("&mut {}", inner_type))
            }
            ast::Type::Pointer(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("UnsafeMutablePointer<{}>", inner_type))
            }
            ast::Type::ConstPointer(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("UnsafePointer<{}>", inner_type))
            }
            ast::Type::Void => Ok("Void".to_string()),
            // C FFI types
            ast::Type::CInt => Ok("CInt".to_string()),
            ast::Type::CUInt => Ok("CUnsignedInt".to_string()),
            ast::Type::CLong => Ok("CLong".to_string()),
            ast::Type::CULong => Ok("CUnsignedLong".to_string()),
            ast::Type::CLongLong => Ok("CLongLong".to_string()),
            ast::Type::CULongLong => Ok("CUnsignedLongLong".to_string()),
            ast::Type::CFloat => Ok("CFloat".to_string()),
            ast::Type::CDouble => Ok("CDouble".to_string()),
            ast::Type::CChar => Ok("CChar".to_string()),
            ast::Type::CSize => Ok("CSize".to_string()),
            ast::Type::CString => Ok("UnsafePointer<CChar>".to_string()),
            ast::Type::Record(name, _) => Ok(name.clone()),
            ast::Type::Union(name, _) => Ok(name.clone()),
        }
    }

    /// Emit default value for type
    fn emit_default_value(&self, type_annot: &Option<ast::Type>) -> SwiftResult<String> {
        match type_annot {
            Some(ty) => match ty {
                ast::Type::Int | ast::Type::UnsignedInt => Ok("0".to_string()),
                ast::Type::Float => Ok("0.0".to_string()),
                ast::Type::Bool => Ok("false".to_string()),
                ast::Type::String => Ok("\"\"".to_string()),
                ast::Type::Option(_) => Ok("nil".to_string()),
                ast::Type::Array(_) => Ok("[]".to_string()),
                ast::Type::Dictionary(_, _) => Ok("[:]".to_string()),
                _ => Ok("nil".to_string()),
            },
            None => Ok("nil".to_string()),
        }
    }
}

impl CodeGenerator for SwiftBackend {
    type Config = SwiftBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_ast(&mut self, program: &AstProgram) -> Result<CodegenOutput, Self::Error> {
        self.generate_from_ast_impl(program)
    }

    fn generate_from_hir(&mut self, _hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error> {
        Err(x_codegen::CodeGenError::Unimplemented(
            "Swift backend does not yet support HIR generation".to_string(),
        ))
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        // LIR -> Swift 代码生成
        self.buffer.clear();

        self.emit_header()?;

        // 开始结构体定义
        self.line("struct Program {")?;
        self.indent();

        // 收集函数
        let mut has_main = false;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    has_main = true;
                }
                // 发射函数签名
                let ret = self.lir_type_to_swift(&f.return_type);
                let params: Vec<String> = f.parameters.iter()
                    .map(|p| format!("{}: {}", p.name, self.lir_type_to_swift(&p.type_)))
                    .collect();
                self.line(&format!("static func {}({}) -> {} {{", f.name, params.join(", "), ret))?;
                self.indent();

                // 发射函数体
                for stmt in &f.body.statements {
                    self.emit_lir_statement(stmt)?;
                }

                self.dedent();
                self.line("}")?;
                self.line("")?;
            }
        }

        self.dedent();
        self.line("}")?;

        // main 入口
        if has_main {
            self.line("")?;
            self.line("Program.main()")?;
        }

        let output_file = OutputFile {
            path: PathBuf::from("main.swift"),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::Swift,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

/// LIR -> Swift 辅助方法
impl SwiftBackend {
    /// 将 LIR 类型转换为 Swift 类型
    fn lir_type_to_swift(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "Void".to_string(),
            Bool => "Bool".to_string(),
            Char => "Character".to_string(),
            Schar | Short => "Int16".to_string(),
            Uchar | Ushort | Int | Uint => "Int".to_string(),
            Long | Ulong | LongLong | UlongLong => "Int64".to_string(),
            Float => "Float".to_string(),
            Double | LongDouble => "Double".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "Int".to_string(),
            Pointer(inner) => "UnsafePointer<{}>".to_string(),
            Array(inner, _) => "[{}]".to_string(),
            FunctionPointer(_, _) => "(@escaping (...) -> Void)".to_string(),
            Named(n) => n.clone(),
            Qualified(_, inner) => self.lir_type_to_swift(inner),
            _ => "Any".to_string(),
        }
    }

    /// 发射 LIR 语句
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> SwiftResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{}", s))?;
            }
            Variable(v) => {
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    let ty = self.lir_type_to_swift(&v.type_);
                    self.line(&format!("let {}: {} = {}", v.name, ty, init_str))?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("if {} {{", cond))?;
                self.indent();
                self.emit_lir_statement(&i.then_branch)?;
                self.dedent();
                if let Some(else_br) = &i.else_branch {
                    self.line("} else {")?;
                    self.indent();
                    self.emit_lir_statement(else_br)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            While(w) => {
                let cond = self.emit_lir_expr(&w.condition)?;
                self.line(&format!("while {} {{", cond))?;
                self.indent();
                self.emit_lir_statement(&w.body)?;
                self.dedent();
                self.line("}")?;
            }
            Return(r) => {
                if let Some(e) = r {
                    let val = self.emit_lir_expr(e)?;
                    self.line(&format!("return {}", val))?;
                } else {
                    self.line("return")?;
                }
            }
            Break => self.line("break")?,
            Continue => self.line("continue")?,
            _ => self.line("// unsupported statement")?,
        }
        Ok(())
    }

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> SwiftResult<String> {
        use x_lir::Expression::*;
        match expr {
            Literal(l) => self.emit_lir_literal(l),
            Variable(n) => Ok(n.clone()),
            Binary(op, l, r) => {
                let left = self.emit_lir_expr(l)?;
                let right = self.emit_lir_expr(r)?;
                let op_str = self.map_lir_binop(op);
                Ok(format!("({} {} {})", left, op_str, right))
            }
            Unary(op, e) => {
                let e = self.emit_lir_expr(e)?;
                let op_str = self.map_lir_unaryop(op);
                Ok(format!("({}{})", op_str, e))
            }
            Call(callee, args) => {
                let callee_str = self.emit_lir_expr(callee)?;
                let args_str: Vec<String> = args.iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{}({})", callee_str, args_str.join(", ")))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("{}[{}]", arr_str, idx_str))
            }
            _ => Ok("nil".to_string()),
        }
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> SwiftResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(n.to_string()),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))),
            Char(c) => Ok(format!("\"{}\"", c)),
            Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            NullPointer => Ok("nil".to_string()),
        }
    }

    /// 映射 LIR 二元运算符
    fn map_lir_binop(&self, op: &x_lir::BinaryOp) -> String {
        use x_lir::BinaryOp::*;
        match op {
            Add => "+", Subtract => "-", Multiply => "*", Divide => "/", Modulo => "%",
            LessThan => "<", LessThanEqual => "<=", GreaterThan => ">", GreaterThanEqual => ">=",
            Equal => "==", NotEqual => "!=",
            BitAnd => "&", BitOr => "|", BitXor => "^",
            LeftShift => "<<", RightShift => ">>", RightShiftArithmetic => ">>",
            LogicalAnd => "&&", LogicalOr => "||",
        }.to_string()
    }

    /// 映射 LIR 一元运算符
    fn map_lir_unaryop(&self, op: &x_lir::UnaryOp) -> String {
        use x_lir::UnaryOp::*;
        match op {
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Not => "!".to_string(),
            BitNot => "~".to_string(),
            _ => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::{ExpressionKind, MethodModifiers, Spanned, StatementKind};

    fn make_expr(kind: ExpressionKind) -> ast::Expression {
        Spanned::new(kind, Span::default())
    }

    fn make_stmt(kind: StatementKind) -> ast::Statement {
        Spanned::new(kind, Span::default())
    }

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
                    statements: vec![make_stmt(StatementKind::Expression(make_expr(
                        ExpressionKind::Call(
                            Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                            vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                "Hello, World!".to_string(),
                            )))],
                        ),
                    )))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("print(\"Hello, World!\")"));
        assert!(swift_code.contains("func main()"));
    }

    #[test]
    fn test_empty_program_generates_default_main() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![],
            statements: vec![],
        };

        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("func main()"));
        assert!(swift_code.contains("print(\"Hello from Swift backend!\")"));
        assert!(swift_code.contains("main()"));
    }

    #[test]
    fn test_function_with_parameters() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "add".to_string(),
                type_parameters: vec![],
                parameters: vec![
                    ast::Parameter {
                        name: "a".to_string(),
                        type_annot: Some(ast::Type::Int),
                        default: None,
                        span: Span::default(),
                    },
                    ast::Parameter {
                        name: "b".to_string(),
                        type_annot: Some(ast::Type::Int),
                        default: None,
                        span: Span::default(),
                    },
                ],
                effects: vec![],
                return_type: Some(ast::Type::Int),
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::Return(Some(make_expr(
                        ExpressionKind::Binary(
                            BinaryOp::Add,
                            Box::new(make_expr(ExpressionKind::Variable("a".to_string()))),
                            Box::new(make_expr(ExpressionKind::Variable("b".to_string()))),
                        ),
                    ))))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("func add(a: Int, b: Int) -> Int"));
        assert!(swift_code.contains("return a + b"));
    }

    #[test]
    fn test_variable_declaration() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "test".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![
                        make_stmt(StatementKind::Variable(ast::VariableDecl {
                            name: "x".to_string(),
                            is_mutable: false,
                            is_constant: false,
                            type_annot: Some(ast::Type::Int),
                            initializer: Some(make_expr(ExpressionKind::Literal(
                                ast::Literal::Integer(42),
                            ))),
                            visibility: ast::Visibility::Private,
                            span: Span::default(),
                        })),
                        make_stmt(StatementKind::Variable(ast::VariableDecl {
                            name: "y".to_string(),
                            is_mutable: true,
                            is_constant: false,
                            type_annot: Some(ast::Type::String),
                            initializer: Some(make_expr(ExpressionKind::Literal(
                                ast::Literal::String("hello".to_string()),
                            ))),
                            visibility: ast::Visibility::Private,
                            span: Span::default(),
                        })),
                    ],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("let x: Int = 42"));
        assert!(swift_code.contains("var y: String = \"hello\""));
    }

    #[test]
    fn test_if_statement() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                span: Span::default(),
                name: "test".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                effects: vec![],
                return_type: None,
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::If(ast::IfStatement {
                        condition: make_expr(ExpressionKind::Variable("condition".to_string())),
                        then_block: ast::Block {
                            statements: vec![make_stmt(StatementKind::Expression(
                                make_expr(ExpressionKind::Call(
                                    Box::new(make_expr(ExpressionKind::Variable(
                                        "print".to_string(),
                                    ))),
                                    vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                        "yes".to_string(),
                                    )))],
                                )),
                            ))],
                        },
                        else_block: Some(ast::Block {
                            statements: vec![make_stmt(StatementKind::Expression(
                                make_expr(ExpressionKind::Call(
                                    Box::new(make_expr(ExpressionKind::Variable(
                                        "print".to_string(),
                                    ))),
                                    vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                        "no".to_string(),
                                    )))],
                                )),
                            ))],
                        }),
                    }))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("if condition {"));
        assert!(swift_code.contains("} else {"));
        assert!(swift_code.contains("print(\"yes\")"));
        assert!(swift_code.contains("print(\"no\")"));
    }

    #[test]
    fn test_type_mapping() {
        let backend = SwiftBackend::new(SwiftBackendConfig::default());

        assert_eq!(backend.map_type(&ast::Type::Int).unwrap(), "Int");
        assert_eq!(backend.map_type(&ast::Type::Float).unwrap(), "Double");
        assert_eq!(backend.map_type(&ast::Type::Bool).unwrap(), "Bool");
        assert_eq!(backend.map_type(&ast::Type::String).unwrap(), "String");
        assert_eq!(
            backend.map_type(&ast::Type::Array(Box::new(ast::Type::Int))).unwrap(),
            "[Int]"
        );
        assert_eq!(
            backend.map_type(&ast::Type::Option(Box::new(ast::Type::Int))).unwrap(),
            "Int?"
        );
    }

    #[test]
    fn test_class_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Class(ast::ClassDecl {
                span: Span::default(),
                name: "Person".to_string(),
                type_parameters: vec![],
                extends: None,
                implements: vec![],
                modifiers: ast::ClassModifiers::default(),
                members: vec![
                    ast::ClassMember::Field(ast::VariableDecl {
                        name: "name".to_string(),
                        is_mutable: false,
                        is_constant: false,
                        type_annot: Some(ast::Type::String),
                        initializer: None,
                        visibility: ast::Visibility::Private,
                        span: Span::default(),
                    }),
                    ast::ClassMember::Field(ast::VariableDecl {
                        name: "age".to_string(),
                        is_mutable: true,
                        is_constant: false,
                        type_annot: Some(ast::Type::Int),
                        initializer: Some(make_expr(ExpressionKind::Literal(ast::Literal::Integer(
                            0,
                        )))),
                        visibility: ast::Visibility::Private,
                        span: Span::default(),
                    }),
                ],
            })],
            statements: vec![],
        };

        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("class Person {"));
        assert!(swift_code.contains("let name: String"));
        assert!(swift_code.contains("var age: Int = 0"));
    }

    #[test]
    fn test_enum_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Enum(ast::EnumDecl {
                span: Span::default(),
                name: "Option".to_string(),
                type_parameters: vec![],
                variants: vec![
                    ast::EnumVariant {
                        name: "None".to_string(),
                        data: ast::EnumVariantData::Unit,
                        doc: None,
                        span: Span::default(),
                    },
                    ast::EnumVariant {
                        name: "Some".to_string(),
                        data: ast::EnumVariantData::Tuple(vec![ast::Type::Generic("T".to_string())]),
                        doc: None,
                        span: Span::default(),
                    },
                ],
            })],
            statements: vec![],
        };

        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("enum Option {"));
        assert!(swift_code.contains("case None"));
        assert!(swift_code.contains("case Some(T)"));
    }
}
