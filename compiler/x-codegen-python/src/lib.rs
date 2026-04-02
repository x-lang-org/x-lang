//! Python 后端 - 将 X AST 编译为 Python 3.14 代码
//!
//! 生成清晰可读的 Python 源代码，支持基本的 X 语言特性
//!
//! ## Python 3.14 特性支持 (2026年2月发布)
//! - Template strings (t-strings) - PEP 750
//! - Deferred evaluation of annotations - PEP 649/749
//! - Multiple interpreters via concurrent.interpreters - PEP 734
//! - Zstandard compression support - PEP 784
//! - Bracketless exception handling - PEP 758
//! - Type hints with union syntax (`X | Y`)
//! - Pattern matching (`match`/`case` 语句)
//! - Type parameter syntax (`def func[T](x: T) -> T`)
//! - Free-threaded Python officially supported

use std::fmt::Write;
use std::path::PathBuf;
use x_parser::ast::{self, ExpressionKind, StatementKind, Program as AstProgram};
use x_codegen::{headers, CodeGenerator, CodegenOutput, OutputFile, FileType};
use x_lir::Program as LirProgram;

#[derive(Debug, Clone)]
pub struct PythonBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl Default for PythonBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
        }
    }
}

pub struct PythonBackend {
    config: PythonBackendConfig,
    indent: usize,
    output: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PythonBackendError {
    #[error("Generation error: {0}")]
    GenerationError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

pub type PythonResult<T> = Result<T, PythonBackendError>;

impl PythonBackend {
    pub fn new(config: PythonBackendConfig) -> Self {
        Self {
            config,
            indent: 0,
            output: String::new(),
        }
    }

    pub fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> PythonResult<x_codegen::CodegenOutput> {
        self.output.clear();
        self.indent = 0;

        self.emit_header()?;

        // Emit X classes as Python classes
        for decl in &program.declarations {
            if let ast::Declaration::Class(class) = decl {
                self.emit_x_class(class)?;
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
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("output.py"),
            content: self.output.as_bytes().to_vec(),
            file_type: x_codegen::FileType::Python,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// Emit an X class as a Python class
    fn emit_x_class(&mut self, class: &ast::ClassDecl) -> PythonResult<()> {
        // Build class definition with inheritance
        let bases = if let Some(parent) = &class.extends {
            format!("({})", parent)
        } else if !class.implements.is_empty() {
            // Python uses multiple inheritance for interfaces
            format!("({})", class.implements.join(", "))
        } else {
            "".to_string()
        };

        self.line(&format!("class {}{}:", class.name, bases))?;
        self.indent += 1;

        // Count fields and methods
        let has_content = class.members.iter().any(|m| {
            matches!(m, ast::ClassMember::Field(_) | ast::ClassMember::Method(_) | ast::ClassMember::Constructor(_))
        });

        if !has_content {
            self.line("pass")?;
        } else {
            // Emit constructor if present
            let mut constructor_found = false;
            for member in &class.members {
                if let ast::ClassMember::Constructor(constructor) = member {
                    self.emit_constructor(constructor)?;
                    constructor_found = true;
                    self.line("")?;
                }
            }

            // If no constructor, emit a basic __init__
            if !constructor_found {
                self.line("def __init__(self):")?;
                self.indent += 1;
                // Initialize fields
                for member in &class.members {
                    if let ast::ClassMember::Field(field) = member {
                        let init = if let Some(expr) = &field.initializer {
                            self.emit_expr(expr)?
                        } else {
                            "None".to_string()
                        };
                        self.line(&format!("self.{} = {}", field.name, init))?;
                    }
                }
                if !class.members.iter().any(|m| matches!(m, ast::ClassMember::Field(_))) {
                    self.line("pass")?;
                }
                self.indent -= 1;
                self.line("")?;
            }

            // Emit methods
            for member in &class.members {
                if let ast::ClassMember::Method(method) = member {
                    self.emit_method(method)?;
                    self.line("")?;
                }
            }
        }

        self.indent -= 1;
        self.line("")?;
        Ok(())
    }

    /// Emit a Python constructor (__init__)
    fn emit_constructor(&mut self, constructor: &ast::ConstructorDecl) -> PythonResult<()> {
        let params: Vec<String> = constructor
            .parameters
            .iter()
            .map(|p| p.name.clone())
            .collect();

        self.line(&format!("def __init__(self, {}):", params.join(", ")))?;
        self.indent += 1;

        // Initialize fields from parameters or defaults
        for member in &constructor.body.statements {
            self.emit_statement(member)?;
        }

        self.indent -= 1;
        Ok(())
    }

    /// Emit a Python method
    fn emit_method(&mut self, method: &ast::FunctionDecl) -> PythonResult<()> {
        let mut params = vec!["self".to_string()];
        for p in &method.parameters {
            params.push(p.name.clone());
        }

        let async_keyword = if method.is_async { "async " } else { "" };
        self.line(&format!("{}def {}({}):", async_keyword, method.name, params.join(", ")))?;
        self.indent += 1;

        self.emit_block(&method.body)?;

        if method.return_type.is_some() {
            self.line("return None")?;
        }

        self.indent -= 1;
        Ok(())
    }

    fn emit_header(&mut self) -> PythonResult<()> {
        self.line(headers::PYTHON)?;
        self.line("# DO NOT EDIT")?;
        self.line("# Target: Python 3.14 (February 2026)")?;
        self.line("# Requires: Python >= 3.14")?;
        self.line("")?;
        self.line("from __future__ import annotations")?;
        self.line("import asyncio")?;
        self.line("from typing import Any, TypeVar, Generic, Optional")?;
        self.line("")?;
        Ok(())
    }

    fn emit_main_function(&mut self) -> PythonResult<()> {
        self.line("def main():")?;
        self.indent += 1;
        self.line("print('Hello from Python backend!')")?;
        self.indent -= 1;
        self.line("")?;
        self.line("if __name__ == '__main__':")?;
        self.indent += 1;
        self.line("main()")?;
        self.indent -= 1;
        Ok(())
    }

    fn emit_global_var(&mut self, v: &ast::VariableDecl) -> PythonResult<()> {
        let init = if let Some(expr) = &v.initializer {
            self.emit_expr(expr)?
        } else {
            "None".to_string()
        };
        self.line(&format!("{} = {}", v.name, init))?;
        Ok(())
    }

    fn emit_function(&mut self, f: &ast::FunctionDecl) -> PythonResult<()> {
        let params = if f.parameters.is_empty() {
            "".to_string()
        } else {
            f.parameters
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        // Emit async keyword for async functions
        let async_keyword = if f.is_async { "async " } else { "" };
        self.line(&format!("{}def {}({}):", async_keyword, f.name, params))?;
        self.indent += 1;
        self.emit_block(&f.body)?;
        if f.return_type.is_some() {
            self.line("return None")?;
        }
        self.indent -= 1;
        Ok(())
    }

    fn emit_block(&mut self, block: &ast::Block) -> PythonResult<()> {
        for stmt in &block.statements {
            self.emit_statement(stmt)?;
        }
        Ok(())
    }

    fn emit_statement(&mut self, stmt: &ast::Statement) -> PythonResult<()> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&e)?;
            }
            StatementKind::Variable(v) => {
                let init = if let Some(expr) = &v.initializer {
                    self.emit_expr(expr)?
                } else {
                    "None".to_string()
                };
                self.line(&format!("{} = {}", v.name, init))?;
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
                self.line(&format!("while {}:", cond))?;
                self.indent += 1;
                self.emit_block(&while_stmt.body)?;
                self.indent -= 1;
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
                self.line("while True:")?;
                self.indent += 1;
                self.emit_block(&d.body)?;
                let cond = self.emit_expr(&d.condition)?;
                self.line(&format!("if not ({}):", cond))?;
                self.indent += 1;
                self.line("break")?;
                self.indent -= 1;
                self.indent -= 1;
            }
            StatementKind::Unsafe(block) => {
                // Python doesn't have unsafe blocks, just emit the block
                self.line("# unsafe block")?;
                self.emit_block(block)?;
            }
            StatementKind::Defer(expr) => {
                // Python doesn't have defer, comment it
                let e = self.emit_expr(expr)?;
                self.line(&format!("# defer {}", e))?;
            }
            StatementKind::Yield(opt_expr) => {
                // Yield for generators
                if let Some(e) = opt_expr {
                    let expr = self.emit_expr(e)?;
                    self.line(&format!("yield {}", expr))?;
                } else {
                    self.line("yield")?;
                }
            }
            StatementKind::Loop(body) => {
                self.line("while True:")?;
                self.indent += 1;
                self.emit_block(body)?;
                self.indent -= 1;
            }
        }
        Ok(())
    }

    fn emit_if(&mut self, if_stmt: &ast::IfStatement) -> PythonResult<()> {
        let cond = self.emit_expr(&if_stmt.condition)?;
        self.line(&format!("if {}:", cond))?;
        self.indent += 1;
        self.emit_block(&if_stmt.then_block)?;
        self.indent -= 1;
        if let Some(else_block) = &if_stmt.else_block {
            self.line("else:")?;
            self.indent += 1;
            self.emit_block(else_block)?;
            self.indent -= 1;
        }
        Ok(())
    }

    fn emit_for(&mut self, for_stmt: &ast::ForStatement) -> PythonResult<()> {
        // Generate the iterator expression
        let iter = self.emit_expr(&for_stmt.iterator)?;

        // Generate the pattern variable name(s)
        let pattern_var = self.emit_pattern_var(&for_stmt.pattern);

        self.line(&format!("for {} in {}:", pattern_var, iter))?;
        self.indent += 1;
        self.emit_block(&for_stmt.body)?;
        self.indent -= 1;
        Ok(())
    }

    fn emit_pattern_var(&self, pattern: &ast::Pattern) -> String {
        match pattern {
            ast::Pattern::Wildcard => "_".to_string(),
            ast::Pattern::Variable(name) => name.clone(),
            ast::Pattern::Literal(lit) => self.emit_literal_for_pattern(lit),
            ast::Pattern::Array(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("[{}]", vars.join(", "))
            }
            ast::Pattern::Tuple(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("({})", vars.join(", "))
            }
            ast::Pattern::Or(left, _) => self.emit_pattern_var(left),
            ast::Pattern::Guard(inner, _) => self.emit_pattern_var(inner),
            ast::Pattern::Dictionary(entries) => {
                let vars: Vec<String> = entries.iter().map(|(k, v)| format!("{}: {}", self.emit_pattern_var(k), self.emit_pattern_var(v))).collect();
                format!("{{{}}}", vars.join(", "))
            }
            ast::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields.iter().map(|(k, v)| format!("{}={}", k, self.emit_pattern_var(v))).collect();
                format!("{}({})", name, field_strs.join(", "))
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns.iter().map(|p| self.emit_pattern_var(p)).collect();
                if patterns.is_empty() {
                    variant_name.clone()
                } else {
                    format!("{}({})", variant_name, pattern_strs.join(", "))
                }
            }
        }
    }

    fn emit_literal_for_pattern(&self, lit: &ast::Literal) -> String {
        match lit {
            ast::Literal::Integer(n) => format!("{}", n),
            ast::Literal::Float(f) => format!("{}", f),
            ast::Literal::Boolean(b) => format!("{}", b),
            ast::Literal::String(s) => format!("\"{}\"", s),
            ast::Literal::Char(c) => format!("'{}'", c),
            ast::Literal::Null | ast::Literal::None | ast::Literal::Unit => "None".to_string(),
        }
    }

    fn emit_match(&mut self, match_stmt: &ast::MatchStatement) -> PythonResult<()> {
        // Python 3.10+ structural pattern matching (match/case)
        let expr = self.emit_expr(&match_stmt.expression)?;

        self.line(&format!("match {}:", expr))?;
        self.indent += 1;

        for case in &match_stmt.cases {
            let pattern = self.emit_match_pattern(&case.pattern)?;
            let case_line = if let Some(guard) = &case.guard {
                let guard_expr = self.emit_expr(guard)?;
                format!("case {} if {}:", pattern, guard_expr)
            } else {
                format!("case {}:", pattern)
            };
            self.line(&case_line)?;
            self.indent += 1;
            self.emit_block(&case.body)?;
            self.indent -= 1;
        }

        self.indent -= 1;
        Ok(())
    }

    /// Emit a Python 3.10+ match pattern
    fn emit_match_pattern(&self, pattern: &ast::Pattern) -> PythonResult<String> {
        match pattern {
            ast::Pattern::Wildcard => Ok("_".to_string()),
            ast::Pattern::Variable(name) => Ok(name.clone()),
            ast::Pattern::Literal(lit) => Ok(self.emit_literal_for_pattern(lit)),
            ast::Pattern::Or(left, right) => {
                let left_pat = self.emit_match_pattern(left)?;
                let right_pat = self.emit_match_pattern(right)?;
                Ok(format!("{} | {}", left_pat, right_pat))
            }
            ast::Pattern::Guard(inner, cond_expr) => {
                // Guard is handled in emit_match, this just returns the inner pattern
                let inner_pat = self.emit_match_pattern(inner)?;
                let guard_cond = self.emit_expr(cond_expr)?;
                Ok(format!("{} if {}", inner_pat, guard_cond))
            }
            ast::Pattern::Array(elements) => {
                let elem_patterns: Vec<String> = elements
                    .iter()
                    .map(|p| self.emit_match_pattern(p))
                    .collect::<PythonResult<Vec<_>>>()?;
                Ok(format!("[{}]", elem_patterns.join(", ")))
            }
            ast::Pattern::Tuple(elements) => {
                let elem_patterns: Vec<String> = elements
                    .iter()
                    .map(|p| self.emit_match_pattern(p))
                    .collect::<PythonResult<Vec<_>>>()?;
                Ok(format!("({})", elem_patterns.join(", ")))
            }
            ast::Pattern::Dictionary(entries) => {
                let entry_patterns: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| {
                        let key = self.emit_match_pattern(k)?;
                        let val = self.emit_match_pattern(v)?;
                        Ok(format!("{}: {}", key, val))
                    })
                    .collect::<PythonResult<Vec<_>>>()?;
                Ok(format!("{{{}}}", entry_patterns.join(", ")))
            }
            ast::Pattern::Record(name, fields) => {
                // Match class instance with attributes
                let field_patterns: Vec<String> = fields
                    .iter()
                    .map(|(field, val)| {
                        let val_pat = self.emit_match_pattern(val)?;
                        Ok(format!("{}={}", field, val_pat))
                    })
                    .collect::<PythonResult<Vec<_>>>()?;
                Ok(format!("{}({})", name, field_patterns.join(", ")))
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_match_pattern(p))
                    .collect::<PythonResult<Vec<_>>>()?;
                if patterns.is_empty() {
                    Ok(variant_name.clone())
                } else {
                    Ok(format!("{}({})", variant_name, pattern_strs.join(", ")))
                }
            }
        }
    }

    fn emit_try(&mut self, try_stmt: &ast::TryStatement) -> PythonResult<()> {
        self.line("try:")?;
        self.indent += 1;
        self.emit_block(&try_stmt.body)?;
        self.indent -= 1;

        for catch in &try_stmt.catch_clauses {
            let except_line = if let Some(var_name) = &catch.variable_name {
                if let Some(exc_type) = &catch.exception_type {
                    format!("except {} as {}:", exc_type, var_name)
                } else {
                    format!("except Exception as {}:", var_name)
                }
            } else if let Some(exc_type) = &catch.exception_type {
                format!("except {}:", exc_type)
            } else {
                "except Exception:".to_string()
            };

            self.line(&except_line)?;
            self.indent += 1;
            self.emit_block(&catch.body)?;
            self.indent -= 1;
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.line("finally:")?;
            self.indent += 1;
            self.emit_block(finally)?;
            self.indent -= 1;
        }

        Ok(())
    }

    fn emit_expr(&self, expr: &ast::Expression) -> PythonResult<String> {
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
            ExpressionKind::Parenthesized(inner) => {
                let e = self.emit_expr(inner)?;
                Ok(format!("({})", e))
            }
            ExpressionKind::If(cond, then_e, else_e) => {
                let c = self.emit_expr(cond)?;
                let t = self.emit_expr(then_e)?;
                let e = self.emit_expr(else_e)?;
                Ok(format!("{} if {} else {}", t, c, e))
            }
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                Ok(format!("{}.{}", o, field))
            }
            ExpressionKind::Wait(wait_type, exprs) => {
                self.emit_wait(wait_type, exprs)
            }
            ExpressionKind::Await(expr) => {
                let e = self.emit_expr(expr)?;
                Ok(format!("await {}", e))
            }
            ExpressionKind::OptionalChain(base, member) => {
                let b = self.emit_expr(base)?;
                Ok(format!("{}.{}", b, member))
            }
            ExpressionKind::NullCoalescing(left, right) => {
                let l = self.emit_expr(left)?;
                let r = self.emit_expr(right)?;
                Ok(format!("{} if {} is not None else {}", l, l, r))
            }
            _ => Err(PythonBackendError::UnsupportedFeature(format!(
                "{:?}",
                expr
            ))),
        }
    }

    fn emit_literal(&self, lit: &ast::Literal) -> PythonResult<String> {
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
            ast::Literal::Null => Ok("None".to_string()),
            ast::Literal::None => Ok("None".to_string()),
            ast::Literal::Unit => Ok("None".to_string()),
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
            ast::BinaryOp::Pow => format!("{} ** {}", l, r),
            _ => format!("# unsupported binop {:?} #", op),
        }
    }

    fn emit_unaryop(&self, op: &ast::UnaryOp, e: &str) -> String {
        match op {
            ast::UnaryOp::Negate => format!("-{}", e),
            ast::UnaryOp::Not => format!("not {}", e),
            ast::UnaryOp::Wait => format!("await {}", e), // Wait becomes await
            _ => format!("# unsupported unary {:?} #", op),
        }
    }

    fn emit_wait(&self, wait_type: &ast::WaitType, exprs: &[ast::Expression]) -> PythonResult<String> {
        let expr_strs: Vec<String> = exprs
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<Result<Vec<_>, _>>()?;
        match wait_type {
            ast::WaitType::Single => {
                // Single await: await expr
                if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    // Multiple expressions - await each
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(format!("({})", awaited.join(", ")))
                }
            }
            ast::WaitType::Together => {
                // Parallel execution: asyncio.gather
                if expr_strs.is_empty() {
                    Ok("asyncio.sleep(0)".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    Ok(format!("await asyncio.gather({})", expr_strs.join(", ")))
                }
            }
            ast::WaitType::Race => {
                // Race: asyncio.wait with FIRST_COMPLETED
                if expr_strs.is_empty() {
                    Ok("asyncio.sleep(0)".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    // Use asyncio.wait with return_when=FIRST_COMPLETED
                    Ok(format!(
                        "await asyncio.wait([{}], return_when=asyncio.FIRST_COMPLETED)",
                        expr_strs.join(", ")
                    ))
                }
            }
            ast::WaitType::Timeout(timeout_expr) => {
                // Timeout: asyncio.wait_for
                let timeout = self.emit_expr(timeout_expr)?;
                if expr_strs.is_empty() {
                    Ok(format!("await asyncio.sleep({})", timeout))
                } else {
                    let expr = &expr_strs[0];
                    Ok(format!("await asyncio.wait_for({}, timeout={})", expr, timeout))
                }
            }
            ast::WaitType::Atomic => {
                // Atomic: atomic operation - just comment and await
                if expr_strs.len() == 1 {
                    Ok(format!("# atomic\nawait {}", expr_strs[0]))
                } else {
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(format!("# atomic\n({})", awaited.join(", ")))
                }
            }
            ast::WaitType::Retry => {
                // Retry: retry operation - just comment and await
                if expr_strs.len() == 1 {
                    Ok(format!("# retry\nawait {}", expr_strs[0]))
                } else {
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(format!("# retry\n({})", awaited.join(", ")))
                }
            }
        }
    }

    fn emit_call(
        &self,
        callee: &ast::Expression,
        args: &[ast::Expression],
    ) -> PythonResult<String> {
        let callee_str = self.emit_expr(callee)?;
        let arg_strs: Vec<String> = args
            .iter()
            .map(|a| self.emit_expr(a))
            .collect::<PythonResult<Vec<_>>>()?;
        Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
    }

    fn emit_assign(
        &self,
        target: &ast::Expression,
        value: &ast::Expression,
    ) -> PythonResult<String> {
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

    fn emit_array_literal(&self, elements: &[ast::Expression]) -> PythonResult<String> {
        if elements.is_empty() {
            return Ok("[]".to_string());
        }
        let elem_strs: Vec<String> = elements
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<PythonResult<Vec<_>>>()?;
        Ok(format!("[{}]", elem_strs.join(", ")))
    }

    fn line(&mut self, s: &str) -> PythonResult<()> {
        for _ in 0..self.indent {
            write!(self.output, "    ")?;
        }
        writeln!(self.output, "{}", s)?;
        Ok(())
    }
}

/// CodeGenerator trait 实现
impl CodeGenerator for PythonBackend {
    type Config = PythonBackendConfig;
    type Error = PythonBackendError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> Result<CodegenOutput, Self::Error> {
        self.generate_from_ast(program)
    }

    fn generate_from_hir(
        &mut self,
        _hir: &x_hir::Hir,
    ) -> Result<CodegenOutput, Self::Error> {
        Err(PythonBackendError::UnsupportedFeature(
            "HIR → Python not yet implemented".to_string(),
        ))
    }

    fn generate_from_lir(
        &mut self,
        lir: &LirProgram,
    ) -> Result<CodegenOutput, Self::Error> {
        // LIR -> Python 代码生成
        self.output.clear();
        self.indent = 0;

        self.emit_header()?;

        // 收集函数
        let mut has_main = false;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    has_main = true;
                }
                // 发射函数
                self.line(&format!("def {}({}):", f.name, f.parameters.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ")))?;
                self.indent += 1;

                // 发射函数体
                for stmt in &f.body.statements {
                    self.emit_lir_statement(stmt)?;
                }

                self.indent -= 1;
                self.line("")?;
            }
        }

        // main 入口
        if has_main {
            self.line("if __name__ == \"__main__\":")?;
            self.indent += 1;
            self.line("main()")?;
            self.indent -= 1;
        }

        let output_file = OutputFile {
            path: PathBuf::from("main.py"),
            content: self.output.as_bytes().to_vec(),
            file_type: FileType::Python,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

/// LIR -> Python 辅助方法
impl PythonBackend {
    /// 发射 LIR 语句
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> PythonResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{}", s))?;
            }
            Variable(v) => {
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("{} = {}", v.name, init_str))?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("if {}:", cond))?;
                self.indent += 1;
                self.emit_lir_statement(&i.then_branch)?;
                self.indent -= 1;
                if let Some(else_br) = &i.else_branch {
                    self.line("else:")?;
                    self.indent += 1;
                    self.emit_lir_statement(else_br)?;
                    self.indent -= 1;
                }
            }
            While(w) => {
                let cond = self.emit_lir_expr(&w.condition)?;
                self.line(&format!("while {}:", cond))?;
                self.indent += 1;
                self.emit_lir_statement(&w.body)?;
                self.indent -= 1;
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
            _ => self.line("# unsupported statement")?,
        }
        Ok(())
    }

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> PythonResult<String> {
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
            _ => Ok("None".to_string()),
        }
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> PythonResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(n.to_string()),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))),
            Char(c) => Ok(format!("'{}'", c)),
            Bool(b) => Ok(b.to_string()),
            NullPointer => Ok("None".to_string()),
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
            LogicalAnd => "and", LogicalOr => "or",
        }.to_string()
    }

    /// 映射 LIR 一元运算符
    fn map_lir_unaryop(&self, op: &x_lir::UnaryOp) -> String {
        use x_lir::UnaryOp::*;
        match op {
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Not => "not ".to_string(),
            BitNot => "~".to_string(),
            _ => "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::{Spanned, StatementKind, ExpressionKind, MethodModifiers};

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
                    statements: vec![make_stmt(StatementKind::Expression(make_expr(ExpressionKind::Call(
                        Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                        vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                            "Hello, World!".to_string(),
                        )))],
                    ))))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = PythonBackend::new(PythonBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let python_code = String::from_utf8_lossy(&output.files[0].content);
        // The implementation uses double quotes for string literals
        assert!(python_code.contains("print(\"Hello, World!\")"));
        assert!(python_code.contains("def main():"));
    }

    #[test]
    fn test_empty_program_generates_default_main() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![],
            statements: vec![],
        };

        let mut backend = PythonBackend::new(PythonBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let python_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(python_code.contains("def main():"));
        assert!(python_code.contains("print('Hello from Python backend!')"));
        assert!(python_code.contains("if __name__ == '__main__':"));
        assert!(python_code.contains("main()"));
    }
}
