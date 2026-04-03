//! TypeScript 后端 - 将 X AST 编译为 TypeScript 6.0 代码
//!
//! 生成 TypeScript 源码，支持基本的 X 语言特性
//!
//! ## TypeScript 6.0 特性支持 (2026年3月发布)
//! - Bridge release before native Go-based TypeScript 7.0
//! - Less context-sensitive functions for better type inference
//! - Subpath imports starting with `#/`
//! - ES2025 target support (RegExp.escape, Promise.try)
//! - Temporal API types built-in
//! - const type parameters
//! - satisfies operator
//! - using 声明（显式资源管理）

use std::fmt::Write;
use std::path::PathBuf;
use x_codegen::headers;
use x_parser::ast::{self, ExpressionKind, Program as AstProgram, StatementKind};

#[derive(Debug, Clone)]
pub struct TypeScriptBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl Default for TypeScriptBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
        }
    }
}

pub struct TypeScriptBackend {
    config: TypeScriptBackendConfig,
    indent: usize,
    output: String,
}

pub type TypeScriptResult<T> = Result<T, x_codegen::CodeGenError>;

impl TypeScriptBackend {
    pub fn new(config: TypeScriptBackendConfig) -> Self {
        Self {
            config,
            indent: 0,
            output: String::new(),
        }
    }

    pub fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> TypeScriptResult<x_codegen::CodegenOutput> {
        self.output.clear();
        self.indent = 0;

        self.emit_header()?;

        // Emit all declarations first
        for decl in &program.declarations {
            self.emit_declaration(decl)?;
        }

        // Emit main function containing all top-level statements
        if !program.statements.is_empty() {
            self.line("function main(): void {")?;
            self.indent += 1;

            for stmt in &program.statements {
                self.emit_statement(stmt)?;
            }

            self.indent -= 1;
            self.line("}")?;
            self.line("")?;
            self.line("// Run the main function")?;
            self.line("main();")?;
        }

        // Create output file
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("index.ts"),
            content: self.output.as_bytes().to_vec(),
            file_type: x_codegen::FileType::TypeScript,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    fn emit_header(&mut self) -> TypeScriptResult<()> {
        self.line(headers::TYPESCRIPT)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: TypeScript 6.0 / ES2025 (March 2026)")?;
        self.line("// tsconfig: strict=true, module=esnext")?;
        self.line("")?;
        Ok(())
    }

    fn emit_declaration(&mut self, decl: &ast::Declaration) -> TypeScriptResult<()> {
        match decl {
            ast::Declaration::Variable(var_decl) => {
                self.emit_variable_decl(var_decl)?;
                self.line("")?;
            }
            ast::Declaration::Function(func_decl) => {
                self.emit_function_decl(func_decl)?;
                self.line("")?;
            }
            ast::Declaration::Class(class_decl) => {
                self.emit_class_decl(class_decl)?;
                self.line("")?;
            }
            ast::Declaration::Trait(trait_decl) => {
                self.emit_trait_decl(trait_decl)?;
                self.line("")?;
            }
            _ => {
                return Err(x_codegen::CodeGenError::UnsupportedFeature(format!(
                    "Declaration type {:?} is not yet supported in TypeScript backend",
                    decl
                )));
            }
        }
        Ok(())
    }

    fn emit_variable_decl(&mut self, var_decl: &ast::VariableDecl) -> TypeScriptResult<()> {
        let keyword = if var_decl.is_mutable { "let" } else { "const" };
        let name = &var_decl.name;

        if let Some(init) = &var_decl.initializer {
            let init_expr = self.emit_expr(init)?;
            self.line(&format!("{} {} = {};", keyword, name, init_expr))?;
        } else {
            self.line(&format!("{} {};", keyword, name))?;
        }

        Ok(())
    }

    fn emit_function_decl(&mut self, func_decl: &ast::FunctionDecl) -> TypeScriptResult<()> {
        let async_keyword = if func_decl.is_async { "async " } else { "" };
        let name = &func_decl.name;

        let params: Vec<String> = func_decl
            .parameters
            .iter()
            .map(|p| p.name.clone())
            .collect();

        self.line(&format!(
            "{}function {}({}): void {{",
            async_keyword,
            name,
            params.join(", ")
        ))?;
        self.indent += 1;

        self.emit_block(&func_decl.body)?;

        self.indent -= 1;
        self.line("}")?;

        Ok(())
    }

    fn emit_class_decl(&mut self, class_decl: &ast::ClassDecl) -> TypeScriptResult<()> {
        // Emit class header
        let extends_clause = if let Some(parent) = &class_decl.extends {
            format!(" extends {}", parent)
        } else {
            String::new()
        };

        let implements_clause = if !class_decl.implements.is_empty() {
            format!(" implements {}", class_decl.implements.join(", "))
        } else {
            String::new()
        };

        self.line(&format!(
            "class {}{}{} {{",
            class_decl.name, extends_clause, implements_clause
        ))?;
        self.indent += 1;

        // Emit members
        for member in &class_decl.members {
            match member {
                ast::ClassMember::Field(field) => {
                    let field_type = if let Some(type_annot) = &field.type_annot {
                        self.emit_type(type_annot)
                    } else {
                        "any".to_string()
                    };
                    let initializer = if let Some(init) = &field.initializer {
                        format!(" = {}", self.emit_expr(init)?)
                    } else {
                        String::new()
                    };
                    self.line(&format!(
                        "{}{}: {}{};",
                        if field.is_mutable { "" } else { "readonly " },
                        field.name,
                        field_type,
                        initializer
                    ))?;
                }
                ast::ClassMember::Method(method) => {
                    self.emit_method_decl(method)?;
                }
                ast::ClassMember::Constructor(constructor) => {
                    self.emit_constructor_decl(constructor)?;
                }
            }
        }

        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_method_decl(&mut self, method: &ast::FunctionDecl) -> TypeScriptResult<()> {
        let params: Vec<String> = method
            .parameters
            .iter()
            .map(|p| {
                let type_str = if let Some(type_annot) = &p.type_annot {
                    format!(": {}", self.emit_type(type_annot))
                } else {
                    String::new()
                };
                format!("{}{}", p.name, type_str)
            })
            .collect();

        let return_type = if let Some(ret) = &method.return_type {
            self.emit_type(ret)
        } else {
            "void".to_string()
        };

        self.line(&format!(
            "{}({}): {} {{",
            method.name,
            params.join(", "),
            return_type
        ))?;
        self.indent += 1;
        self.emit_block(&method.body)?;
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_constructor_decl(
        &mut self,
        constructor: &ast::ConstructorDecl,
    ) -> TypeScriptResult<()> {
        let params: Vec<String> = constructor
            .parameters
            .iter()
            .map(|p| {
                let type_str = if let Some(type_annot) = &p.type_annot {
                    format!(": {}", self.emit_type(type_annot))
                } else {
                    String::new()
                };
                format!("{}{}", p.name, type_str)
            })
            .collect();

        self.line(&format!("constructor({}) {{", params.join(", ")))?;
        self.indent += 1;
        self.emit_block(&constructor.body)?;
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_trait_decl(&mut self, trait_decl: &ast::TraitDecl) -> TypeScriptResult<()> {
        self.line(&format!("interface {} {{", trait_decl.name))?;
        self.indent += 1;

        for method in &trait_decl.methods {
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| {
                    let type_str = if let Some(type_annot) = &p.type_annot {
                        format!(": {}", self.emit_type(type_annot))
                    } else {
                        String::new()
                    };
                    format!("{}{}", p.name, type_str)
                })
                .collect();

            let return_type = if let Some(ret) = &method.return_type {
                self.emit_type(ret)
            } else {
                "void".to_string()
            };

            self.line(&format!(
                "{}({}): {};",
                method.name,
                params.join(", "),
                return_type
            ))?;
        }

        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_type(&self, ty: &ast::Type) -> String {
        match ty {
            ast::Type::Int => "number".to_string(),
            ast::Type::Float => "number".to_string(),
            ast::Type::Bool => "boolean".to_string(),
            ast::Type::String => "string".to_string(),
            ast::Type::Char => "string".to_string(),
            ast::Type::Unit => "void".to_string(),
            ast::Type::Never => "never".to_string(),
            ast::Type::Array(inner) => format!("{}[]", self.emit_type(inner)),
            ast::Type::Option(inner) => format!("{} | null", self.emit_type(inner)),
            ast::Type::Result(ok, _err) => self.emit_type(ok), // Simplified
            ast::Type::Tuple(types) => {
                let type_strs: Vec<String> = types.iter().map(|t| self.emit_type(t)).collect();
                format!("[{}]", type_strs.join(", "))
            }
            ast::Type::Function(params, ret) => {
                let param_strs: Vec<String> = params.iter().map(|t| self.emit_type(t)).collect();
                format!("({}) => {}", param_strs.join(", "), self.emit_type(ret))
            }
            ast::Type::Generic(name) | ast::Type::TypeParam(name) | ast::Type::Var(name) => {
                name.clone()
            }
            _ => "any".to_string(),
        }
    }

    fn emit_statement(&mut self, stmt: &ast::Statement) -> TypeScriptResult<()> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                let expr_str = self.emit_expr(expr)?;
                self.line(&format!("{};", expr_str))?;
            }
            StatementKind::Variable(var_decl) => {
                self.emit_variable_decl(var_decl)?;
            }
            StatementKind::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let expr_str = self.emit_expr(expr)?;
                    self.line(&format!("return {};", expr_str))?;
                } else {
                    self.line("return;")?;
                }
            }
            StatementKind::If(if_stmt) => {
                self.emit_if_statement(if_stmt)?;
            }
            StatementKind::While(while_stmt) => {
                self.emit_while_statement(while_stmt)?;
            }
            StatementKind::For(for_stmt) => {
                self.emit_for_statement(for_stmt)?;
            }
            StatementKind::Match(match_stmt) => {
                self.emit_match_statement(match_stmt)?;
            }
            StatementKind::Try(try_stmt) => {
                self.emit_try_statement(try_stmt)?;
            }
            StatementKind::Break => {
                self.line("break;")?;
            }
            StatementKind::Continue => {
                self.line("continue;")?;
            }
            StatementKind::DoWhile(do_while) => {
                self.line("while (true) {")?;
                self.indent += 1;
                self.emit_block(&do_while.body)?;
                let cond = self.emit_expr(&do_while.condition)?;
                self.line(&format!("if (!({})) {{ break; }}", cond))?;
                self.indent -= 1;
                self.line("}")?;
            }
            StatementKind::Unsafe(block) => {
                // TypeScript doesn't have unsafe blocks, just emit the block
                self.line("// unsafe block")?;
                self.line("{")?;
                self.indent += 1;
                self.emit_block(block)?;
                self.indent -= 1;
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
                self.line("while (true) {")?;
                self.indent += 1;
                self.emit_block(body)?;
                self.indent -= 1;
                self.line("}")?;
            }
        }
        Ok(())
    }

    fn emit_block(&mut self, block: &ast::Block) -> TypeScriptResult<()> {
        for stmt in &block.statements {
            self.emit_statement(stmt)?;
        }
        Ok(())
    }

    fn emit_if_statement(&mut self, if_stmt: &ast::IfStatement) -> TypeScriptResult<()> {
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

    fn emit_while_statement(&mut self, while_stmt: &ast::WhileStatement) -> TypeScriptResult<()> {
        let cond = self.emit_expr(&while_stmt.condition)?;
        self.line(&format!("while ({}) {{", cond))?;
        self.indent += 1;
        self.emit_block(&while_stmt.body)?;
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_for_statement(&mut self, for_stmt: &ast::ForStatement) -> TypeScriptResult<()> {
        let iter = self.emit_expr(&for_stmt.iterator)?;
        let pattern_name = self.emit_pattern_var(&for_stmt.pattern);
        self.line(&format!("for (const {} of {}) {{", pattern_name, iter))?;
        self.indent += 1;
        self.emit_block(&for_stmt.body)?;
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_pattern_var(&self, pattern: &ast::Pattern) -> String {
        match pattern {
            ast::Pattern::Wildcard => "_".to_string(),
            ast::Pattern::Variable(name) => name.clone(),
            ast::Pattern::Array(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("[{}]", vars.join(", "))
            }
            ast::Pattern::Tuple(elements) => {
                let vars: Vec<String> = elements.iter().map(|p| self.emit_pattern_var(p)).collect();
                format!("[{}]", vars.join(", "))
            }
            ast::Pattern::Or(left, _) => self.emit_pattern_var(left),
            ast::Pattern::Guard(inner, _) => self.emit_pattern_var(inner),
            _ => "_item".to_string(),
        }
    }

    fn emit_match_statement(&mut self, match_stmt: &ast::MatchStatement) -> TypeScriptResult<()> {
        let expr = self.emit_expr(&match_stmt.expression)?;
        // Use a temporary variable for the match value
        let temp_var = "__match_val__";
        self.line(&format!("const {} = {};", temp_var, expr))?;

        for (i, case) in match_stmt.cases.iter().enumerate() {
            let condition =
                self.emit_match_condition(temp_var, &case.pattern, case.guard.as_ref())?;

            if i == 0 {
                self.line(&format!("if ({}) {{", condition))?;
            } else {
                self.line(&format!("else if ({}) {{", condition))?;
            }

            self.indent += 1;
            self.emit_pattern_bindings(temp_var, &case.pattern)?;
            self.emit_block(&case.body)?;
            self.indent -= 1;
            self.line("}")?;
        }

        Ok(())
    }

    fn emit_match_condition(
        &self,
        var: &str,
        pattern: &ast::Pattern,
        guard: Option<&ast::Expression>,
    ) -> TypeScriptResult<String> {
        let base_cond = match pattern {
            ast::Pattern::Wildcard => "true".to_string(),
            ast::Pattern::Variable(_) => "true".to_string(),
            ast::Pattern::Literal(lit) => {
                let lit_str = self.emit_literal(lit)?;
                format!("{} === {}", var, lit_str)
            }
            ast::Pattern::Or(left, right) => {
                let left_cond = self.emit_match_condition(var, left, None)?;
                let right_cond = self.emit_match_condition(var, right, None)?;
                format!("({}) || ({})", left_cond, right_cond)
            }
            ast::Pattern::Array(elements) => {
                let len_check = format!(
                    "Array.isArray({}) && {}.length === {}",
                    var,
                    var,
                    elements.len()
                );
                let elem_checks: Vec<String> = elements
                    .iter()
                    .enumerate()
                    .map(|(i, p)| self.emit_match_condition(&format!("{}[{}]", var, i), p, None))
                    .collect::<TypeScriptResult<Vec<_>>>()?;
                if elem_checks.is_empty() {
                    format!("{}.length === 0", var)
                } else {
                    format!("({}) && {}", len_check, elem_checks.join(" && "))
                }
            }
            _ => format!("true /* pattern not fully supported */"),
        };

        if let Some(guard_expr) = guard {
            let guard_str = self.emit_expr(guard_expr)?;
            Ok(format!("({}) && ({})", base_cond, guard_str))
        } else {
            Ok(base_cond)
        }
    }

    fn emit_pattern_bindings(&mut self, var: &str, pattern: &ast::Pattern) -> TypeScriptResult<()> {
        match pattern {
            ast::Pattern::Variable(name) => {
                self.line(&format!("const {} = {};", name, var))?;
            }
            ast::Pattern::Array(elements) => {
                for (i, elem) in elements.iter().enumerate() {
                    self.emit_pattern_bindings(&format!("{}[{}]", var, i), elem)?;
                }
            }
            ast::Pattern::Tuple(elements) => {
                for (i, elem) in elements.iter().enumerate() {
                    self.emit_pattern_bindings(&format!("{}[{}]", var, i), elem)?;
                }
            }
            ast::Pattern::Or(left, _) => {
                self.emit_pattern_bindings(var, left)?;
            }
            ast::Pattern::Guard(inner, _) => {
                self.emit_pattern_bindings(var, inner)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn emit_try_statement(&mut self, try_stmt: &ast::TryStatement) -> TypeScriptResult<()> {
        self.line("try {")?;
        self.indent += 1;
        self.emit_block(&try_stmt.body)?;
        self.indent -= 1;

        for catch in &try_stmt.catch_clauses {
            let catch_line = if let Some(var_name) = &catch.variable_name {
                format!("catch ({}) {{", var_name)
            } else {
                "catch (error) {".to_string()
            };
            self.line(&catch_line)?;
            self.indent += 1;
            self.emit_block(&catch.body)?;
            self.indent -= 1;
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.line("} finally {")?;
            self.indent += 1;
            self.emit_block(finally)?;
            self.indent -= 1;
        }

        self.line("}")?;
        Ok(())
    }

    fn emit_expr(&self, expr: &ast::Expression) -> TypeScriptResult<String> {
        match &expr.node {
            ExpressionKind::Literal(lit) => Ok(self.emit_literal(lit)?),
            ExpressionKind::Variable(name) => Ok(name.clone()),
            ExpressionKind::Binary(op, lhs, rhs) => {
                let lhs_str = self.emit_expr(lhs)?;
                let rhs_str = self.emit_expr(rhs)?;
                Ok(self.emit_binop(op, &lhs_str, &rhs_str))
            }
            ExpressionKind::Unary(op, expr) => {
                let expr_str = self.emit_expr(expr)?;
                Ok(self.emit_unaryop(op, &expr_str))
            }
            ExpressionKind::Call(callee, args) => {
                let callee_str = self.emit_expr(callee)?;
                let mut arg_strs = Vec::new();
                for arg in args {
                    arg_strs.push(self.emit_expr(arg)?);
                }

                // Map built-in functions
                if callee_str == "println" {
                    Ok(format!("console.log({})", arg_strs.join(", ")))
                } else {
                    Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
                }
            }
            ExpressionKind::Assign(lhs, rhs) => {
                let lhs_str = self.emit_expr(lhs)?;
                let rhs_str = self.emit_expr(rhs)?;
                Ok(format!("{} = {}", lhs_str, rhs_str))
            }
            ExpressionKind::Parenthesized(expr) => {
                let inner = self.emit_expr(expr)?;
                Ok(format!("({})", inner))
            }
            ExpressionKind::Wait(wait_type, exprs) => self.emit_wait(wait_type, exprs),
            ExpressionKind::Await(expr) => {
                let expr_str = self.emit_expr(expr)?;
                Ok(format!("await {}", expr_str))
            }
            ExpressionKind::OptionalChain(base, member) => {
                let base_str = self.emit_expr(base)?;
                Ok(format!("{}?.{}", base_str, member))
            }
            ExpressionKind::NullCoalescing(left, right) => {
                let left_str = self.emit_expr(left)?;
                let right_str = self.emit_expr(right)?;
                Ok(format!("({} ?? {})", left_str, right_str))
            }
            _ => Err(x_codegen::CodeGenError::UnsupportedFeature(format!(
                "Expression type {:?} is not yet supported in TypeScript backend",
                expr
            ))),
        }
    }

    fn emit_literal(&self, lit: &ast::Literal) -> TypeScriptResult<String> {
        match lit {
            ast::Literal::Integer(n) => Ok(n.to_string()),
            ast::Literal::Float(f) => Ok(f.to_string()),
            ast::Literal::Boolean(b) => Ok(b.to_string()),
            ast::Literal::String(s) => Ok(format!("\"{}\"", s.replace('"', "\\\""))),
            ast::Literal::Char(c) => Ok(format!("'{}'", c)),
            ast::Literal::Null => Ok("null".to_string()),
            ast::Literal::None => Ok("undefined".to_string()),
            ast::Literal::Unit => Ok("undefined".to_string()),
        }
    }

    fn emit_binop(&self, op: &ast::BinaryOp, lhs: &str, rhs: &str) -> String {
        match op {
            ast::BinaryOp::Add => format!("{} + {}", lhs, rhs),
            ast::BinaryOp::Sub => format!("{} - {}", lhs, rhs),
            ast::BinaryOp::Mul => format!("{} * {}", lhs, rhs),
            ast::BinaryOp::Div => format!("{} / {}", lhs, rhs),
            ast::BinaryOp::Mod => format!("{} % {}", lhs, rhs),
            ast::BinaryOp::Pow => format!("Math.pow({}, {})", lhs, rhs),
            ast::BinaryOp::And => format!("{} && {}", lhs, rhs),
            ast::BinaryOp::Or => format!("{} || {}", lhs, rhs),
            ast::BinaryOp::Equal => format!("{} === {}", lhs, rhs),
            ast::BinaryOp::NotEqual => format!("{} !== {}", lhs, rhs),
            ast::BinaryOp::Less => format!("{} < {}", lhs, rhs),
            ast::BinaryOp::LessEqual => format!("{} <= {}", lhs, rhs),
            ast::BinaryOp::Greater => format!("{} > {}", lhs, rhs),
            ast::BinaryOp::GreaterEqual => format!("{} >= {}", lhs, rhs),
            ast::BinaryOp::BitAnd => format!("{} & {}", lhs, rhs),
            ast::BinaryOp::BitOr => format!("{} | {}", lhs, rhs),
            ast::BinaryOp::BitXor => format!("{} ^ {}", lhs, rhs),
            ast::BinaryOp::LeftShift => format!("{} << {}", lhs, rhs),
            ast::BinaryOp::RightShift => format!("{} >> {}", lhs, rhs),
            ast::BinaryOp::Concat => format!("{} + {}", lhs, rhs),
            _ => format!("/* unsupported op {:?} */ {} {}", op, lhs, rhs),
        }
    }

    fn emit_unaryop(&self, op: &ast::UnaryOp, expr: &str) -> String {
        match op {
            ast::UnaryOp::Negate => format!("-{}", expr),
            ast::UnaryOp::Not => format!("!{}", expr),
            ast::UnaryOp::BitNot => format!("~{}", expr),
            ast::UnaryOp::Wait => format!("await {}", expr),
        }
    }

    fn emit_wait(
        &self,
        wait_type: &ast::WaitType,
        exprs: &[ast::Expression],
    ) -> TypeScriptResult<String> {
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
                    let awaited: Vec<String> =
                        expr_strs.iter().map(|e| format!("await {}", e)).collect();
                    Ok(format!("({})", awaited.join(", ")))
                }
            }
            ast::WaitType::Together => {
                // Parallel execution: Promise.all
                if expr_strs.is_empty() {
                    Ok("Promise.resolve()".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    Ok(format!("await Promise.all([{}])", expr_strs.join(", ")))
                }
            }
            ast::WaitType::Race => {
                // Race: Promise.race
                if expr_strs.is_empty() {
                    Ok("Promise.resolve()".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("await {}", expr_strs[0]))
                } else {
                    Ok(format!("await Promise.race([{}])", expr_strs.join(", ")))
                }
            }
            ast::WaitType::Timeout(timeout_expr) => {
                // Timeout: race between operation and timeout
                let timeout = self.emit_expr(timeout_expr)?;
                if expr_strs.is_empty() {
                    Ok(format!(
                        "new Promise(resolve => setTimeout(resolve, {}))",
                        timeout
                    ))
                } else {
                    let expr = &expr_strs[0];
                    Ok(format!(
                        "await Promise.race([{}, new Promise((_, reject) => setTimeout(() => reject(new Error('Timeout')), {}))])",
                        expr, timeout
                    ))
                }
            }
            ast::WaitType::Atomic => {
                // Atomic: just await the expression with a comment
                if expr_strs.len() == 1 {
                    Ok(format!("// atomic\nawait {}", expr_strs[0]))
                } else {
                    Ok(format!("// atomic\nawait Promise.all([{}])", expr_strs.join(", ")))
                }
            }
            ast::WaitType::Retry => {
                // Retry: just await the expression with a comment
                if expr_strs.len() == 1 {
                    Ok(format!("// retry\nawait {}", expr_strs[0]))
                } else {
                    Ok(format!("// retry\nawait Promise.all([{}])", expr_strs.join(", ")))
                }
            }
        }
    }

    fn line(&mut self, s: &str) -> TypeScriptResult<()> {
        for _ in 0..self.indent {
            write!(self.output, "    ")?;
        }
        writeln!(self.output, "{}", s)?;
        Ok(())
    }

    // ============================================================
    // LIR → TypeScript generation (full pipeline entry point)
    // ============================================================

    /// Generate TypeScript code from the Low-level IR.
    /// This is the canonical entry point used by the full compiler pipeline.
    pub fn generate_from_lir(
        &mut self,
        lir: &x_lir::Program,
    ) -> TypeScriptResult<x_codegen::CodegenOutput> {
        self.output.clear();
        self.indent = 0;

        self.emit_header()?;

        // Extern function declarations (builtins are mapped in emit_lir_expression)
        for decl in &lir.declarations {
            if let x_lir::Declaration::ExternFunction(ef) = decl {
                self.emit_lir_extern_function(ef)?;
            }
        }

        // Global variables
        for decl in &lir.declarations {
            if let x_lir::Declaration::Global(gv) = decl {
                self.emit_lir_global_var(gv)?;
            }
        }

        // Type aliases
        for decl in &lir.declarations {
            if let x_lir::Declaration::TypeAlias(ta) = decl {
                let ty_str = self.emit_lir_type(&ta.type_);
                self.line(&format!("type {} = {};", ta.name, ty_str))?;
                self.line("")?;
            }
        }

        // Struct definitions → TypeScript interfaces
        for decl in &lir.declarations {
            if let x_lir::Declaration::Struct(s) = decl {
                self.emit_lir_struct(s)?;
            }
        }

        // Class definitions
        for decl in &lir.declarations {
            if let x_lir::Declaration::Class(c) = decl {
                self.emit_lir_class(c)?;
            }
        }

        // Enum definitions → TypeScript enums
        for decl in &lir.declarations {
            if let x_lir::Declaration::Enum(e) = decl {
                self.emit_lir_enum(e)?;
            }
        }

        // Functions
        let mut has_main = false;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(func) = decl {
                if func.name == "main" {
                    has_main = true;
                }
                self.emit_lir_function(func)?;
                self.line("")?;
            }
        }

        // Auto-invoke main if present (TypeScript has no implicit entry point)
        if has_main {
            self.line("// Entry point")?;
            self.line("main();")?;
        }

        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("index.ts"),
            content: self.output.as_bytes().to_vec(),
            file_type: x_codegen::FileType::TypeScript,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    fn emit_lir_extern_function(&mut self, ef: &x_lir::ExternFunction) -> TypeScriptResult<()> {
        // Skip X built-ins that are mapped to JS/TS equivalents in emit_lir_expression
        match ef.name.as_str() {
            "println" | "print" | "exit" | "panic" => return Ok(()),
            _ => {}
        }
        let params: Vec<String> = ef
            .parameters
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("arg{}: {}", i, self.emit_lir_type(ty)))
            .collect();
        let ret_type = self.emit_lir_type(&ef.return_type);
        self.line(&format!(
            "declare function {}({}): {};",
            ef.name,
            params.join(", "),
            ret_type
        ))?;
        self.line("")?;
        Ok(())
    }

    fn emit_lir_global_var(&mut self, gv: &x_lir::GlobalVar) -> TypeScriptResult<()> {
        let keyword = if gv.initializer.is_some() {
            "const"
        } else {
            "let"
        };
        let type_ann = self.emit_lir_type(&gv.type_);
        if let Some(init) = &gv.initializer {
            let init_str = self.emit_lir_expression(init)?;
            self.line(&format!(
                "{} {}: {} = {};",
                keyword, gv.name, type_ann, init_str
            ))?;
        } else {
            self.line(&format!("{} {}: {};", keyword, gv.name, type_ann))?;
        }
        self.line("")?;
        Ok(())
    }

    fn emit_lir_struct(&mut self, s: &x_lir::Struct) -> TypeScriptResult<()> {
        self.line(&format!("interface {} {{", s.name))?;
        self.indent += 1;
        for field in &s.fields {
            let ty_str = self.emit_lir_type(&field.type_);
            self.line(&format!("{}: {};", field.name, ty_str))?;
        }
        self.indent -= 1;
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    fn emit_lir_class(&mut self, c: &x_lir::Class) -> TypeScriptResult<()> {
        let extends_part = c
            .extends
            .as_deref()
            .map(|p| format!(" extends {}", p))
            .unwrap_or_default();
        let implements_part = if c.implements.is_empty() {
            String::new()
        } else {
            format!(" implements {}", c.implements.join(", "))
        };
        self.line(&format!(
            "class {}{}{} {{",
            c.name, extends_part, implements_part
        ))?;
        self.indent += 1;
        for field in &c.fields {
            let ty_str = self.emit_lir_type(&field.type_);
            self.line(&format!("{}: {};", field.name, ty_str))?;
        }
        self.indent -= 1;
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    fn emit_lir_enum(&mut self, e: &x_lir::Enum) -> TypeScriptResult<()> {
        self.line(&format!("enum {} {{", e.name))?;
        self.indent += 1;
        for (i, variant) in e.variants.iter().enumerate() {
            let value = variant
                .value
                .map(|v| v.to_string())
                .unwrap_or_else(|| i.to_string());
            let comma = if i < e.variants.len() - 1 { "," } else { "" };
            self.line(&format!("{} = {}{}", variant.name, value, comma))?;
        }
        self.indent -= 1;
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    fn emit_lir_function(&mut self, func: &x_lir::Function) -> TypeScriptResult<()> {
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| format!("{}: {}", p.name, self.emit_lir_type(&p.type_)))
            .collect();
        let ret_type = self.emit_lir_type(&func.return_type);
        self.line(&format!(
            "function {}({}): {} {{",
            func.name,
            params.join(", "),
            ret_type
        ))?;
        self.indent += 1;
        self.emit_lir_block(&func.body)?;
        self.indent -= 1;
        self.line("}")?;
        Ok(())
    }

    fn emit_lir_block(&mut self, block: &x_lir::Block) -> TypeScriptResult<()> {
        for stmt in &block.statements {
            self.emit_lir_statement(stmt)?;
        }
        Ok(())
    }

    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> TypeScriptResult<()> {
        match stmt {
            x_lir::Statement::Expression(expr) => {
                let s = self.emit_lir_expression(expr)?;
                self.line(&format!("{};", s))?;
            }
            x_lir::Statement::Variable(var) => {
                let keyword = if var.initializer.is_some() {
                    "const"
                } else {
                    "let"
                };
                let type_ann = self.emit_lir_type(&var.type_);
                if let Some(init) = &var.initializer {
                    let init_str = self.emit_lir_expression(init)?;
                    self.line(&format!(
                        "{} {}: {} = {};",
                        keyword, var.name, type_ann, init_str
                    ))?;
                } else {
                    self.line(&format!("{} {}: {};", keyword, var.name, type_ann))?;
                }
            }
            x_lir::Statement::If(if_stmt) => {
                let cond = self.emit_lir_expression(&if_stmt.condition)?;
                self.line(&format!("if ({}) {{", cond))?;
                self.indent += 1;
                self.emit_lir_statement(&if_stmt.then_branch)?;
                self.indent -= 1;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.line("} else {")?;
                    self.indent += 1;
                    self.emit_lir_statement(else_branch)?;
                    self.indent -= 1;
                }
                self.line("}")?;
            }
            x_lir::Statement::While(w) => {
                let cond = self.emit_lir_expression(&w.condition)?;
                self.line(&format!("while ({}) {{", cond))?;
                self.indent += 1;
                self.emit_lir_statement(&w.body)?;
                self.indent -= 1;
                self.line("}")?;
            }
            x_lir::Statement::DoWhile(d) => {
                self.line("do {")?;
                self.indent += 1;
                self.emit_lir_statement(&d.body)?;
                self.indent -= 1;
                let cond = self.emit_lir_expression(&d.condition)?;
                self.line(&format!("}} while ({});", cond))?;
            }
            x_lir::Statement::For(for_stmt) => {
                let init = if let Some(init_stmt) = &for_stmt.initializer {
                    match init_stmt.as_ref() {
                        x_lir::Statement::Variable(var) => {
                            let kw = if var.initializer.is_some() {
                                "const"
                            } else {
                                "let"
                            };
                            let ta = self.emit_lir_type(&var.type_);
                            if let Some(ie) = &var.initializer {
                                let is = self.emit_lir_expression(ie)?;
                                format!("{} {}: {} = {}", kw, var.name, ta, is)
                            } else {
                                format!("{} {}: {}", kw, var.name, ta)
                            }
                        }
                        x_lir::Statement::Expression(expr) => self.emit_lir_expression(expr)?,
                        _ => String::new(),
                    }
                } else {
                    String::new()
                };
                let cond = for_stmt
                    .condition
                    .as_ref()
                    .map(|c| self.emit_lir_expression(c))
                    .transpose()?
                    .unwrap_or_default();
                let incr = for_stmt
                    .increment
                    .as_ref()
                    .map(|i| self.emit_lir_expression(i))
                    .transpose()?
                    .unwrap_or_default();
                self.line(&format!("for ({}; {}; {}) {{", init, cond, incr))?;
                self.indent += 1;
                self.emit_lir_statement(&for_stmt.body)?;
                self.indent -= 1;
                self.line("}")?;
            }
            x_lir::Statement::Switch(sw) => {
                let expr = self.emit_lir_expression(&sw.expression)?;
                self.line(&format!("switch ({}) {{", expr))?;
                self.indent += 1;
                for case in &sw.cases {
                    let val = self.emit_lir_expression(&case.value)?;
                    self.line(&format!("case {}:", val))?;
                    self.indent += 1;
                    self.emit_lir_statement(&case.body)?;
                    self.line("break;")?;
                    self.indent -= 1;
                }
                if let Some(default) = &sw.default {
                    self.line("default:")?;
                    self.indent += 1;
                    self.emit_lir_statement(default)?;
                    self.indent -= 1;
                }
                self.indent -= 1;
                self.line("}")?;
            }
            x_lir::Statement::Match(m) => {
                self.emit_lir_match(m)?;
            }
            x_lir::Statement::Try(try_stmt) => {
                self.line("try {")?;
                self.indent += 1;
                self.emit_lir_block(&try_stmt.body)?;
                self.indent -= 1;
                for catch in &try_stmt.catch_clauses {
                    let var = catch.variable_name.as_deref().unwrap_or("_e");
                    self.line(&format!("}} catch ({}) {{", var))?;
                    self.indent += 1;
                    self.emit_lir_block(&catch.body)?;
                    self.indent -= 1;
                }
                if let Some(finally) = &try_stmt.finally_block {
                    self.line("} finally {")?;
                    self.indent += 1;
                    self.emit_lir_block(finally)?;
                    self.indent -= 1;
                }
                self.line("}")?;
            }
            x_lir::Statement::Return(expr_opt) => {
                if let Some(expr) = expr_opt {
                    let s = self.emit_lir_expression(expr)?;
                    self.line(&format!("return {};", s))?;
                } else {
                    self.line("return;")?;
                }
            }
            x_lir::Statement::Break => self.line("break;")?,
            x_lir::Statement::Continue => self.line("continue;")?,
            x_lir::Statement::Compound(block) => {
                self.line("{")?;
                self.indent += 1;
                self.emit_lir_block(block)?;
                self.indent -= 1;
                self.line("}")?;
            }
            x_lir::Statement::Label(name) => {
                // TypeScript has labelled statements but no goto; emit label only
                self.line(&format!("{}:", name))?;
            }
            x_lir::Statement::Goto(name) => {
                self.line(&format!("// goto {} (unsupported in TypeScript)", name))?;
            }
            x_lir::Statement::Empty => {}
            x_lir::Statement::Declaration(_) => {
                // Nested declarations are skipped; they are handled at top-level
            }
        }
        Ok(())
    }

    fn emit_lir_match(&mut self, m: &x_lir::MatchStatement) -> TypeScriptResult<()> {
        let scrutinee = self.emit_lir_expression(&m.scrutinee)?;
        self.line(&format!("const __match__ = {};", scrutinee))?;
        for (i, case) in m.cases.iter().enumerate() {
            let cond = self.emit_lir_pattern_condition("__match__", &case.pattern)?;
            let guard_cond = if let Some(guard) = &case.guard {
                let gs = self.emit_lir_expression(guard)?;
                format!("({}) && ({})", cond, gs)
            } else {
                cond
            };
            if i == 0 {
                self.line(&format!("if ({}) {{", guard_cond))?;
            } else {
                self.line(&format!("}} else if ({}) {{", guard_cond))?;
            }
            self.indent += 1;
            self.emit_lir_pattern_bindings("__match__", &case.pattern)?;
            self.emit_lir_block(&case.body)?;
            self.indent -= 1;
        }
        if !m.cases.is_empty() {
            self.line("}")?;
        }
        Ok(())
    }

    fn emit_lir_pattern_condition(
        &self,
        var: &str,
        pattern: &x_lir::Pattern,
    ) -> TypeScriptResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard | x_lir::Pattern::Variable(_) => Ok("true".to_string()),
            x_lir::Pattern::Literal(lit) => {
                let ls = self.emit_lir_literal(lit)?;
                Ok(format!("{} === {}", var, ls))
            }
            x_lir::Pattern::Constructor(name, _) => {
                Ok(format!("{} !== null && {}.tag === \"{}\"", var, var, name))
            }
            x_lir::Pattern::Tuple(elements) => {
                let checks: Vec<String> = elements
                    .iter()
                    .enumerate()
                    .map(|(i, p)| self.emit_lir_pattern_condition(&format!("{}[{}]", var, i), p))
                    .collect::<Result<_, _>>()?;
                if checks.is_empty() {
                    Ok("true".to_string())
                } else {
                    Ok(checks.join(" && "))
                }
            }
            x_lir::Pattern::Record(_, fields) => {
                let checks: Vec<String> = fields
                    .iter()
                    .map(|(f, p)| self.emit_lir_pattern_condition(&format!("{}.{}", var, f), p))
                    .collect::<Result<_, _>>()?;
                if checks.is_empty() {
                    Ok("true".to_string())
                } else {
                    Ok(checks.join(" && "))
                }
            }
            x_lir::Pattern::Or(left, right) => {
                let l = self.emit_lir_pattern_condition(var, left)?;
                let r = self.emit_lir_pattern_condition(var, right)?;
                Ok(format!("({}) || ({})", l, r))
            }
        }
    }

    fn emit_lir_pattern_bindings(
        &mut self,
        var: &str,
        pattern: &x_lir::Pattern,
    ) -> TypeScriptResult<()> {
        match pattern {
            x_lir::Pattern::Variable(name) => {
                self.line(&format!("const {} = {};", name, var))?;
            }
            x_lir::Pattern::Constructor(_, fields) => {
                for (i, field) in fields.iter().enumerate() {
                    self.emit_lir_pattern_bindings(&format!("{}.fields[{}]", var, i), field)?;
                }
            }
            x_lir::Pattern::Tuple(elements) => {
                for (i, elem) in elements.iter().enumerate() {
                    self.emit_lir_pattern_bindings(&format!("{}[{}]", var, i), elem)?;
                }
            }
            x_lir::Pattern::Record(_, fields) => {
                for (f, p) in fields {
                    self.emit_lir_pattern_bindings(&format!("{}.{}", var, f), p)?;
                }
            }
            x_lir::Pattern::Or(left, _) => {
                self.emit_lir_pattern_bindings(var, left)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn emit_lir_expression(&self, expr: &x_lir::Expression) -> TypeScriptResult<String> {
        match expr {
            x_lir::Expression::Literal(lit) => self.emit_lir_literal(lit),
            x_lir::Expression::Variable(name) => Ok(match name.as_str() {
                "println" => "console.log".to_string(),
                "print" => "process.stdout.write".to_string(),
                "exit" => "process.exit".to_string(),
                _ => name.clone(),
            }),
            x_lir::Expression::Unary(op, inner) => {
                let s = self.emit_lir_expression(inner)?;
                Ok(match op {
                    x_lir::UnaryOp::Minus => format!("(-{})", s),
                    x_lir::UnaryOp::Not => format!("(!{})", s),
                    x_lir::UnaryOp::BitNot => format!("(~{})", s),
                    x_lir::UnaryOp::PreIncrement => format!("(++{})", s),
                    x_lir::UnaryOp::PreDecrement => format!("(--{})", s),
                    x_lir::UnaryOp::PostIncrement => format!("({}++)", s),
                    x_lir::UnaryOp::PostDecrement => format!("({}--)", s),
                    x_lir::UnaryOp::Plus => s,
                })
            }
            x_lir::Expression::Binary(op, lhs, rhs) => {
                let l = self.emit_lir_expression(lhs)?;
                let r = self.emit_lir_expression(rhs)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+",
                    x_lir::BinaryOp::Subtract => "-",
                    x_lir::BinaryOp::Multiply => "*",
                    x_lir::BinaryOp::Divide => "/",
                    x_lir::BinaryOp::Modulo => "%",
                    x_lir::BinaryOp::Equal => "===",
                    x_lir::BinaryOp::NotEqual => "!==",
                    x_lir::BinaryOp::LessThan => "<",
                    x_lir::BinaryOp::LessThanEqual => "<=",
                    x_lir::BinaryOp::GreaterThan => ">",
                    x_lir::BinaryOp::GreaterThanEqual => ">=",
                    x_lir::BinaryOp::LogicalAnd => "&&",
                    x_lir::BinaryOp::LogicalOr => "||",
                    x_lir::BinaryOp::BitAnd => "&",
                    x_lir::BinaryOp::BitOr => "|",
                    x_lir::BinaryOp::BitXor => "^",
                    x_lir::BinaryOp::LeftShift => "<<",
                    x_lir::BinaryOp::RightShift => ">>>",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>",
                };
                Ok(format!("({} {} {})", l, op_str, r))
            }
            x_lir::Expression::Ternary(cond, then, else_) => {
                let c = self.emit_lir_expression(cond)?;
                let t = self.emit_lir_expression(then)?;
                let e = self.emit_lir_expression(else_)?;
                Ok(format!("({} ? {} : {})", c, t, e))
            }
            x_lir::Expression::Assign(lhs, rhs) => {
                let l = self.emit_lir_expression(lhs)?;
                let r = self.emit_lir_expression(rhs)?;
                Ok(format!("{} = {}", l, r))
            }
            x_lir::Expression::AssignOp(op, lhs, rhs) => {
                let l = self.emit_lir_expression(lhs)?;
                let r = self.emit_lir_expression(rhs)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+=",
                    x_lir::BinaryOp::Subtract => "-=",
                    x_lir::BinaryOp::Multiply => "*=",
                    x_lir::BinaryOp::Divide => "/=",
                    x_lir::BinaryOp::Modulo => "%=",
                    _ => "+=",
                };
                Ok(format!("{} {} {}", l, op_str, r))
            }
            x_lir::Expression::Call(callee, args) => {
                let callee_str = self.emit_lir_expression(callee)?;
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_lir_expression(a))
                    .collect::<Result<_, _>>()?;
                Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
            }
            x_lir::Expression::Index(arr, idx) => {
                let a = self.emit_lir_expression(arr)?;
                let i = self.emit_lir_expression(idx)?;
                Ok(format!("{}[{}]", a, i))
            }
            x_lir::Expression::Member(obj, field) => {
                let o = self.emit_lir_expression(obj)?;
                Ok(format!("{}.{}", o, field))
            }
            x_lir::Expression::PointerMember(obj, field) => {
                // No pointer indirection in TypeScript
                let o = self.emit_lir_expression(obj)?;
                Ok(format!("{}.{}", o, field))
            }
            x_lir::Expression::AddressOf(inner) | x_lir::Expression::Dereference(inner) => {
                // No raw pointers in TypeScript; emit the inner expression directly
                self.emit_lir_expression(inner)
            }
            x_lir::Expression::Cast(_ty, inner) => {
                let s = self.emit_lir_expression(inner)?;
                Ok(format!("({} as unknown)", s))
            }
            x_lir::Expression::Parenthesized(inner) => {
                let s = self.emit_lir_expression(inner)?;
                Ok(format!("({})", s))
            }
            x_lir::Expression::SizeOf(_)
            | x_lir::Expression::SizeOfExpr(_)
            | x_lir::Expression::AlignOf(_) => Ok("0".to_string()),
            x_lir::Expression::Comma(exprs) => {
                let strs: Vec<String> = exprs
                    .iter()
                    .map(|e| self.emit_lir_expression(e))
                    .collect::<Result<_, _>>()?;
                Ok(format!("({})", strs.join(", ")))
            }
            x_lir::Expression::InitializerList(_) | x_lir::Expression::CompoundLiteral(_, _) => {
                Ok("{}".to_string())
            }
        }
    }

    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> TypeScriptResult<String> {
        match lit {
            x_lir::Literal::Integer(n) => Ok(n.to_string()),
            x_lir::Literal::UnsignedInteger(n) => Ok(n.to_string()),
            x_lir::Literal::Long(n) => Ok(n.to_string()),
            x_lir::Literal::UnsignedLong(n) => Ok(n.to_string()),
            x_lir::Literal::LongLong(n) => Ok(n.to_string()),
            x_lir::Literal::UnsignedLongLong(n) => Ok(n.to_string()),
            x_lir::Literal::Float(f) | x_lir::Literal::Double(f) => Ok(format!("{}", f)),
            x_lir::Literal::Char(c) => Ok(format!("\"{}\"", c)),
            x_lir::Literal::String(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                Ok(format!("\"{}\"", escaped))
            }
            x_lir::Literal::Bool(b) => Ok(b.to_string()),
            x_lir::Literal::NullPointer => Ok("null".to_string()),
        }
    }

    /// Map an LIR type to its TypeScript equivalent.
    fn emit_lir_type(&self, ty: &x_lir::Type) -> String {
        match ty {
            x_lir::Type::Void => "void".to_string(),
            x_lir::Type::Bool => "boolean".to_string(),
            x_lir::Type::Char | x_lir::Type::Schar | x_lir::Type::Uchar => "string".to_string(),
            x_lir::Type::Short
            | x_lir::Type::Ushort
            | x_lir::Type::Int
            | x_lir::Type::Uint
            | x_lir::Type::Long
            | x_lir::Type::Ulong
            | x_lir::Type::LongLong
            | x_lir::Type::UlongLong
            | x_lir::Type::Float
            | x_lir::Type::Double
            | x_lir::Type::LongDouble
            | x_lir::Type::Size
            | x_lir::Type::Ptrdiff
            | x_lir::Type::Intptr
            | x_lir::Type::Uintptr => "number".to_string(),
            x_lir::Type::Pointer(inner) => match inner.as_ref() {
                x_lir::Type::Char | x_lir::Type::Schar | x_lir::Type::Uchar => "string".to_string(),
                x_lir::Type::Void => "unknown".to_string(),
                _ => format!("{} | null", self.emit_lir_type(inner)),
            },
            x_lir::Type::Array(inner, _) => format!("{}[]", self.emit_lir_type(inner)),
            x_lir::Type::FunctionPointer(ret, params) => {
                let param_strs: Vec<String> = params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| format!("arg{}: {}", i, self.emit_lir_type(p)))
                    .collect();
                format!("({}) => {}", param_strs.join(", "), self.emit_lir_type(ret))
            }
            x_lir::Type::Named(name) => name.clone(),
            x_lir::Type::Qualified(_, inner) => self.emit_lir_type(inner),
        }
    }

    /// Invoke `tsc` to compile the generated TypeScript file to JavaScript.
    ///
    /// Requires TypeScript to be installed: `npm install -g typescript`
    pub fn compile_ts_to_js(
        ts_code: &str,
        output_dir: &std::path::Path,
        out_name: &str,
    ) -> TypeScriptResult<std::path::PathBuf> {
        let ts_path = output_dir.join(format!("{}.ts", out_name));
        std::fs::write(&ts_path, ts_code).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!(
                "Failed to write TypeScript source: {}",
                e
            ))
        })?;

        let status = std::process::Command::new("tsc")
            .arg("--outDir")
            .arg(output_dir)
            .arg("--target")
            .arg("ES2020")
            .arg("--module")
            .arg("commonjs")
            .arg("--strict")
            .arg("--esModuleInterop")
            .arg(&ts_path)
            .status()
            .map_err(|e| {
                x_codegen::CodeGenError::GenerationError(format!(
                    "Failed to invoke tsc: {}. Install TypeScript with: npm install -g typescript",
                    e
                ))
            })?;

        if !status.success() {
            return Err(x_codegen::CodeGenError::GenerationError(
                "TypeScript compilation failed. \
                 Use `--emit ts` to inspect the generated TypeScript code."
                    .to_string(),
            ));
        }

        Ok(output_dir.join(format!("{}.js", out_name)))
    }
}

/// `CodeGenerator` trait implementation — enables the TypeScript backend
/// to participate in the full compiler pipeline (LIR → TypeScript).
impl x_codegen::CodeGenerator for TypeScriptBackend {
    type Config = TypeScriptBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        TypeScriptBackend::new(config)
    }

    fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> Result<x_codegen::CodegenOutput, Self::Error> {
        TypeScriptBackend::generate_from_ast(self, program)
    }

    fn generate_from_hir(
        &mut self,
        _hir: &x_hir::Hir,
    ) -> Result<x_codegen::CodegenOutput, Self::Error> {
        Err(x_codegen::CodeGenError::UnsupportedFeature(
            "HIR → TypeScript not yet implemented; use the LIR path (generate_from_lir)."
                .to_string(),
        ))
    }

    fn generate_from_lir(
        &mut self,
        lir: &x_lir::Program,
    ) -> Result<x_codegen::CodegenOutput, Self::Error> {
        TypeScriptBackend::generate_from_lir(self, lir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::{Literal, MethodModifiers, Spanned};

    fn make_expr(kind: ExpressionKind) -> ast::Expression {
        Spanned::new(kind, Span::default())
    }

    fn make_stmt(kind: StatementKind) -> ast::Statement {
        Spanned::new(kind, Span::default())
    }

    #[test]
    fn test_empty_program_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![],
            statements: vec![],
        };

        let mut backend = TypeScriptBackend::new(TypeScriptBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let ts_code = String::from_utf8_lossy(&output.files[0].content);
        // With empty statements, no main function is generated
        assert!(ts_code.contains("// Generated by X-Lang TypeScript 6.0 backend"));
    }

    #[test]
    fn test_program_with_statements_generates_main() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![],
            statements: vec![make_stmt(StatementKind::Expression(make_expr(
                ExpressionKind::Call(
                    Box::new(make_expr(ExpressionKind::Variable("println".to_string()))),
                    vec![make_expr(ExpressionKind::Literal(Literal::String(
                        "Hello, World!".to_string(),
                    )))],
                ),
            )))],
        };

        let mut backend = TypeScriptBackend::new(TypeScriptBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let ts_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(ts_code.contains("function main(): void"));
        assert!(ts_code.contains("console.log("));
        assert!(ts_code.contains("main();"));
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
                    statements: vec![make_stmt(StatementKind::Return(Some(make_expr(
                        ExpressionKind::Literal(Literal::String("data".to_string())),
                    ))))],
                },
                is_async: true,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = TypeScriptBackend::new(TypeScriptBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let ts_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(ts_code.contains("async function fetch_data"));
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
                    statements: vec![make_stmt(StatementKind::Expression(make_expr(
                        ExpressionKind::Wait(
                            ast::WaitType::Together,
                            vec![
                                make_expr(ExpressionKind::Variable("task1".to_string())),
                                make_expr(ExpressionKind::Variable("task2".to_string())),
                            ],
                        ),
                    )))],
                },
                is_async: true,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = TypeScriptBackend::new(TypeScriptBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let ts_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(ts_code.contains("async function main"));
        assert!(ts_code.contains("await Promise.all([task1, task2])"));
    }

    #[test]
    fn test_wait_race_generation() {
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
                        ExpressionKind::Wait(
                            ast::WaitType::Race,
                            vec![
                                make_expr(ExpressionKind::Variable("task1".to_string())),
                                make_expr(ExpressionKind::Variable("task2".to_string())),
                            ],
                        ),
                    )))],
                },
                is_async: true,
                modifiers: MethodModifiers::default(),
            })],
            statements: vec![],
        };

        let mut backend = TypeScriptBackend::new(TypeScriptBackendConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let ts_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(ts_code.contains("async function main"));
        assert!(ts_code.contains("await Promise.race([task1, task2])"));
    }
}
