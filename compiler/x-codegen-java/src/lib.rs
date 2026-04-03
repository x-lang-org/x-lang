//! Java 后端 - 生成 Java 25 LTS 源代码
//!
//! 面向 JVM 平台，生成 Java 源代码
//!
//! ## Java 25 LTS 特性支持 (2025年9月发布)
//! - Records（记录类）
//! - Pattern matching for switch（switch 模式匹配）
//! - Sealed classes（密封类）
//! - Text blocks（文本块）
//! - Virtual threads（虚拟线程）
//! - Structured concurrency（结构化并发）
//! - Scoped values
//! - Pattern matching for switch with primitives

use std::fmt::Write;
use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, OutputFile, FileType};
use x_lir::Program as LirProgram;
use x_parser::ast::{self, ExpressionKind, StatementKind, Program as AstProgram, Type};

/// Java 后端配置
#[derive(Debug, Clone)]
pub struct JavaConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    /// 生成的 Java 类名（默认为 "Main"）
    pub class_name: String,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            class_name: "Main".to_string(),
        }
    }
}

/// Java 后端
pub struct JavaBackend {
    config: JavaConfig,
    /// 代码缓冲区（统一管理输出和缩进）
    buffer: x_codegen::CodeBuffer,
}

pub type JavaResult<T> = Result<T, x_codegen::CodeGenError>;

impl JavaBackend {
    pub fn new(config: JavaConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
        }
    }

    /// 输出一行代码
    fn line(&mut self, s: &str) -> JavaResult<()> {
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

    /// 从 AST 生成 Java 代码
    pub fn generate_from_ast(
        &mut self,
        program: &AstProgram,
    ) -> JavaResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // 开始类定义
        self.line(&format!("public class {} {{", self.config.class_name))?;
        self.indent();

        // Single pass to categorize declarations
        let mut classes = Vec::new();
        let mut global_vars = Vec::new();
        let mut functions = Vec::new();

        for decl in &program.declarations {
            match decl {
                ast::Declaration::Class(class) => classes.push(class),
                ast::Declaration::Variable(v) => global_vars.push(v),
                ast::Declaration::Function(f) => functions.push(f),
                _ => {}
            }
        }

        // Emit class fields (from class declarations)
        for class in &classes {
            self.emit_x_class(class)?;
        }

        // Emit global variables as static fields
        for v in &global_vars {
            self.emit_static_field(v)?;
        }

        // Emit functions as static methods
        let mut user_main = None;
        for f in &functions {
            if f.name == "main" {
                user_main = Some(f.clone());
            } else {
                self.emit_function(f)?;
                self.line("")?;
            }
        }

        // Emit the user's main function (renamed to avoid conflict)
        if let Some(user_main_func) = &user_main {
            // Emit the user's main as a renamed function
            self.line(&format!("// User-defined main function"))?;
            self.emit_function(user_main_func)?;
            self.line("")?;
        }

        // Always emit the Java main method entry point
        self.emit_main_method(user_main.is_some())?;

        self.dedent();
        self.line("}")?;

        // Create output file
        let output_file = OutputFile {
            path: PathBuf::from(format!("{}.java", self.config.class_name)),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::Java,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// Emit file header with package and imports (Java 25 LTS)
    fn emit_header(&mut self) -> JavaResult<()> {
        self.line(headers::JAVA)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: Java 25 LTS (September 2025)")?;
        self.line("")?;
        // Java 25 标准库导入
        self.line("import java.util.*;")?;
        self.line("import java.util.concurrent.*;")?;
        self.line("import java.util.concurrent.Flow.*;")?;
        self.line("import java.util.concurrent.StructuredTaskScope;")?;
        self.line("")?;
        Ok(())
    }

    /// Emit an X class as a Java class (Java 21: supports sealed, records)
    fn emit_x_class(&mut self, class: &ast::ClassDecl) -> JavaResult<()> {
        // Check if this is a data class (all fields final, no methods except constructor)
        // In Java 21, we could emit as record, but for compatibility we use regular class

        // Class modifiers (sealed/non-sealed for Java 17+)
        let sealed_mod = if class.modifiers.is_final {
            "final "
        } else if class.modifiers.is_abstract {
            "abstract "
        } else {
            ""
        };

        // Extends clause
        let extends = if let Some(parent) = &class.extends {
            format!(" extends {}", parent)
        } else {
            "".to_string()
        };

        // Implements clause
        let implements = if class.implements.is_empty() {
            "".to_string()
        } else {
            format!(" implements {}", class.implements.join(", "))
        };

        self.line(&format!("{}public static class {}{}{} {{", sealed_mod, class.name, extends, implements))?;
        self.indent();

        // Emit fields
        for member in &class.members {
            if let ast::ClassMember::Field(field) = member {
                let type_str = self.map_type(&field.type_annot);
                let init = if let Some(expr) = &field.initializer {
                    format!(" = {}", self.emit_expr(expr)?)
                } else {
                    "".to_string()
                };
                self.line(&format!("public {} {}{};", type_str, field.name, init))?;
            }
        }

        // Emit constructor if present
        for member in &class.members {
            if let ast::ClassMember::Constructor(constructor) = member {
                self.line("")?;
                self.emit_constructor(&class.name, constructor)?;
            }
        }

        // Emit methods
        for member in &class.members {
            if let ast::ClassMember::Method(method) = member {
                self.line("")?;
                self.emit_method(method)?;
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Emit a Java constructor
    fn emit_constructor(&mut self, class_name: &str, constructor: &ast::ConstructorDecl) -> JavaResult<()> {
        let params: Vec<String> = constructor
            .parameters
            .iter()
            .map(|p| {
                let type_str = self.map_type(&p.type_annot);
                format!("{} {}", type_str, p.name)
            })
            .collect();

        self.line(&format!("public {}({}) {{", class_name, params.join(", ")))?;
        self.indent();

        for stmt in &constructor.body.statements {
            self.emit_statement(stmt)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit a Java method
    fn emit_method(&mut self, method: &ast::FunctionDecl) -> JavaResult<()> {
        let return_type = method.return_type.as_ref()
            .map(|t| self.map_type_from_type(t))
            .unwrap_or_else(|| "void".to_string());

        let params: Vec<String> = method
            .parameters
            .iter()
            .map(|p| {
                let type_str = self.map_type(&p.type_annot);
                format!("{} {}", type_str, p.name)
            })
            .collect();

        let async_modifier = if method.is_async { "async " } else { "" };
        self.line(&format!("public {}{} {}({}) {{", async_modifier, return_type, method.name, params.join(", ")))?;
        self.indent();

        self.emit_block(&method.body)?;

        // Add default return if needed
        if method.return_type.is_some() {
            self.line("return null;")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit a static field for global variables
    fn emit_static_field(&mut self, v: &ast::VariableDecl) -> JavaResult<()> {
        let type_str = self.map_type(&v.type_annot);
        let init = if let Some(expr) = &v.initializer {
            format!(" = {}", self.emit_expr(expr)?)
        } else {
            "".to_string()
        };
        self.line(&format!("private static {} {}{};", type_str, v.name, init))?;
        Ok(())
    }

    /// Emit a function as a static method
    fn emit_function(&mut self, f: &ast::FunctionDecl) -> JavaResult<()> {
        let return_type = f.return_type.as_ref()
            .map(|t| self.map_type_from_type(t))
            .unwrap_or_else(|| "void".to_string());

        let params: Vec<String> = f
            .parameters
            .iter()
            .map(|p| {
                let type_str = self.map_type(&p.type_annot);
                format!("{} {}", type_str, p.name)
            })
            .collect();

        let async_modifier = if f.is_async { "async " } else { "" };
        self.line(&format!("public static {}{} {}({}) {{", async_modifier, return_type, f.name, params.join(", ")))?;
        self.indent();

        self.emit_block(&f.body)?;

        // Add default return if needed
        if f.return_type.is_some() {
            self.line("return null;")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit main method
    fn emit_main_method(&mut self, has_user_main: bool) -> JavaResult<()> {
        self.line("public static void main(String[] args) {")?;
        self.indent();
        if has_user_main {
            // Call the user's main function
            self.line("main();")?;
        } else {
            self.line("System.out.println(\"Hello from Java backend!\");")?;
        }
        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit a block of statements
    fn emit_block(&mut self, block: &ast::Block) -> JavaResult<()> {
        for stmt in &block.statements {
            self.emit_statement(stmt)?;
        }
        Ok(())
    }

    /// Emit a statement
    fn emit_statement(&mut self, stmt: &ast::Statement) -> JavaResult<()> {
        match &stmt.node {
            StatementKind::Expression(expr) => {
                let e = self.emit_expr(expr)?;
                self.line(&format!("{};", e))?;
            }
            StatementKind::Variable(v) => {
                let type_str = self.map_type(&v.type_annot);
                let init = if let Some(expr) = &v.initializer {
                    format!(" = {}", self.emit_expr(expr)?)
                } else {
                    "".to_string()
                };
                self.line(&format!("{} {}{};", type_str, v.name, init))?;
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
                // Java doesn't have unsafe blocks, just emit the block
                self.line("// unsafe block")?;
                self.emit_block(block)?;
            }
            StatementKind::Defer(expr) => {
                // Java doesn't have built-in defer, emit as comment
                let e = self.emit_expr(expr)?;
                self.line(&format!("// defer: {};", e))?;
            }
            StatementKind::Yield(opt_expr) => {
                // Java doesn't have yield (for generators), just comment
                if let Some(expr) = opt_expr {
                    let e = self.emit_expr(expr)?;
                    self.line(&format!("// yield: {};", e))?;
                } else {
                    self.line("// yield;")?;
                }
            }
            StatementKind::Loop(block) => {
                self.line("while (true) {")?;
                self.indent();
                self.emit_block(block)?;
                self.dedent();
                self.line("}")?;
            }
        }
        Ok(())
    }

    /// Emit if statement
    fn emit_if(&mut self, if_stmt: &ast::IfStatement) -> JavaResult<()> {
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

    /// Emit for statement
    fn emit_for(&mut self, for_stmt: &ast::ForStatement) -> JavaResult<()> {
        let iter = self.emit_expr(&for_stmt.iterator)?;
        let pattern_var = self.emit_pattern_var(&for_stmt.pattern);

        // Java enhanced for loop
        self.line(&format!("for (var {} : {}) {{", pattern_var, iter))?;
        self.indent();
        self.emit_block(&for_stmt.body)?;
        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit pattern variable name
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

    /// Emit literal for pattern
    fn emit_literal_for_pattern(&self, lit: &ast::Literal) -> String {
        match lit {
            ast::Literal::Integer(n) => format!("{}", n),
            ast::Literal::Float(f) => format!("{}", f),
            ast::Literal::Boolean(b) => format!("{}", b),
            ast::Literal::String(s) => format!("\"{}\"", s),
            ast::Literal::Char(c) => format!("'{}'", c),
            ast::Literal::Null | ast::Literal::None | ast::Literal::Unit => "null".to_string(),
        }
    }

    /// Emit match statement (using if-else chain)
    fn emit_match(&mut self, match_stmt: &ast::MatchStatement) -> JavaResult<()> {
        let expr = self.emit_expr(&match_stmt.expression)?;

        // Store the expression in a temporary variable for complex expressions
        let temp_var = "__matchVal";
        self.line(&format!("var {} = {};", temp_var, expr))?;

        for (i, case) in match_stmt.cases.iter().enumerate() {
            let condition = self.emit_match_condition(temp_var, &case.pattern, case.guard.as_ref())?;

            if i == 0 {
                self.line(&format!("if ({}) {{", condition))?;
            } else {
                self.line(&format!("}} else if ({}) {{", condition))?;
            }

            self.indent();
            self.emit_pattern_bindings(temp_var, &case.pattern)?;
            self.emit_block(&case.body)?;
            self.dedent();
        }

        self.line("}")?;
        Ok(())
    }

    /// Emit match condition
    fn emit_match_condition(&self, var: &str, pattern: &ast::Pattern, guard: Option<&ast::Expression>) -> JavaResult<String> {
        let base_cond = match pattern {
            ast::Pattern::Wildcard => "true".to_string(),
            ast::Pattern::Variable(_) => "true".to_string(),
            ast::Pattern::Literal(lit) => {
                let lit_str = self.emit_literal_for_pattern(lit);
                format!("{}.equals({})", var, lit_str)
            }
            ast::Pattern::Or(left, right) => {
                let left_cond = self.emit_match_condition(var, left, None)?;
                let right_cond = self.emit_match_condition(var, right, None)?;
                format!("({}) || ({})", left_cond, right_cond)
            }
            ast::Pattern::Guard(inner, cond_expr) => {
                let inner_cond = self.emit_match_condition(var, inner, None)?;
                let guard_cond = self.emit_expr(cond_expr)?;
                format!("({}) && ({})", inner_cond, guard_cond)
            }
            ast::Pattern::Array(elements) => {
                let len_check = format!("{}.length == {}", var, elements.len());
                let elem_checks: Vec<String> = elements.iter().enumerate()
                    .map(|(i, p)| self.emit_match_condition(&format!("{}[{}]", var, i), p, None))
                    .collect::<JavaResult<Vec<_>>>()?;
                if elem_checks.is_empty() {
                    format!("{}.length == 0", var)
                } else {
                    format!("({}) && ({})", len_check, elem_checks.join(" && "))
                }
            }
            ast::Pattern::Tuple(elements) => {
                // Java tuples are not native, assume using a Tuple class
                let elem_checks: Vec<String> = elements.iter().enumerate()
                    .map(|(i, p)| self.emit_match_condition(&format!("{}._{}", var, i), p, None))
                    .collect::<JavaResult<Vec<_>>>()?;
                elem_checks.join(" && ")
            }
            ast::Pattern::Record(_, _) | ast::Pattern::Dictionary(_) => {
                format!("true // pattern matching for records/dicts not fully implemented")
            }
            ast::Pattern::EnumConstructor(_type_name, variant_name, patterns) => {
                if patterns.is_empty() {
                    format!("{} instanceof {}", var, variant_name)
                } else {
                    format!("{} instanceof {} // enum constructor pattern not fully implemented", var, variant_name)
                }
            }
        };

        if let Some(guard_expr) = guard {
            let guard_str = self.emit_expr(guard_expr)?;
            Ok(format!("({}) && ({})", base_cond, guard_str))
        } else {
            Ok(base_cond)
        }
    }

    /// Emit pattern bindings
    fn emit_pattern_bindings(&mut self, var: &str, pattern: &ast::Pattern) -> JavaResult<()> {
        match pattern {
            ast::Pattern::Variable(name) => {
                self.line(&format!("var {} = {};", name, var))?;
            }
            ast::Pattern::Array(elements) => {
                for (i, elem) in elements.iter().enumerate() {
                    self.emit_pattern_bindings(&format!("{}[{}]", var, i), elem)?;
                }
            }
            ast::Pattern::Tuple(elements) => {
                for (i, elem) in elements.iter().enumerate() {
                    self.emit_pattern_bindings(&format!("{}._{}", var, i), elem)?;
                }
            }
            ast::Pattern::Or(left, _) => {
                self.emit_pattern_bindings(var, left)?;
            }
            ast::Pattern::Guard(inner, _) => {
                self.emit_pattern_bindings(var, inner)?;
            }
            ast::Pattern::Dictionary(entries) => {
                for (key, val_pattern) in entries {
                    let key_str = self.emit_pattern_var(key);
                    self.emit_pattern_bindings(&format!("{}.get({})", var, key_str), val_pattern)?;
                }
            }
            ast::Pattern::Record(_, fields) => {
                for (field, val_pattern) in fields {
                    self.emit_pattern_bindings(&format!("{}.{}", var, field), val_pattern)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Emit try statement
    fn emit_try(&mut self, try_stmt: &ast::TryStatement) -> JavaResult<()> {
        self.line("try {")?;
        self.indent();
        self.emit_block(&try_stmt.body)?;
        self.dedent();

        for catch in &try_stmt.catch_clauses {
            let catch_line = if let Some(var_name) = &catch.variable_name {
                if let Some(exc_type) = &catch.exception_type {
                    format!("catch ({} {}) {{", exc_type, var_name)
                } else {
                    format!("catch (Exception {}) {{", var_name)
                }
            } else if let Some(exc_type) = &catch.exception_type {
                format!("catch ({}) {{", exc_type)
            } else {
                "catch (Exception e) {".to_string()
            };

            self.line(&catch_line)?;
            self.indent();
            self.emit_block(&catch.body)?;
            self.dedent();
            self.line("}")?;
        }

        if let Some(finally) = &try_stmt.finally_block {
            self.line("finally {")?;
            self.indent();
            self.emit_block(finally)?;
            self.dedent();
            self.line("}")?;
        }

        Ok(())
    }

    /// Map optional type annotation to Java type
    fn map_type(&self, type_ann: &Option<Type>) -> String {
        match type_ann {
            Some(t) => self.map_type_from_type(t),
            None => "var".to_string(),
        }
    }

    /// Map X Type to Java type
    fn map_type_from_type(&self, t: &Type) -> String {
        match t {
            Type::Int => "int".to_string(),
            Type::UnsignedInt => "int".to_string(),
            Type::Float => "double".to_string(),
            Type::Bool => "boolean".to_string(),
            Type::String => "String".to_string(),
            Type::Char => "char".to_string(),
            Type::Unit => "void".to_string(),
            Type::Void => "void".to_string(),
            Type::Never => "void".to_string(),
            Type::Dynamic => "Object".to_string(),
            Type::Array(inner) => {
                format!("List<{}>", self.map_type_from_type(inner))
            }
            Type::Dictionary(key, value) => {
                format!("Map<{}, {}>", self.map_type_from_type(key), self.map_type_from_type(value))
            }
            Type::Option(inner) => {
                format!("Optional<{}>", self.map_type_from_type(inner))
            }
            Type::Result(ok, err) => {
                format!("Result<{}, {}>", self.map_type_from_type(ok), self.map_type_from_type(err))
            }
            Type::Function(params, ret) => {
                if params.len() == 1 {
                    format!("Function<{}, {}>", self.map_type_from_type(&params[0]), self.map_type_from_type(ret))
                } else if params.is_empty() {
                    format!("Supplier<{}>", self.map_type_from_type(ret))
                } else {
                    let param_types: Vec<String> = params.iter().map(|p| self.map_type_from_type(p)).collect();
                    format!("Function<{}, {}>", param_types.join(", "), self.map_type_from_type(ret))
                }
            }
            Type::Tuple(elements) => {
                if elements.is_empty() {
                    "void".to_string()
                } else if elements.len() == 2 {
                    format!("Pair<{}, {}>",
                        self.map_type_from_type(&elements[0]),
                        self.map_type_from_type(&elements[1]))
                } else {
                    format!("Tuple{}<{}>", elements.len(),
                        elements.iter().map(|e| self.map_type_from_type(e)).collect::<Vec<_>>().join(", "))
                }
            }
            Type::Async(inner) => {
                format!("CompletableFuture<{}>", self.map_type_from_type(inner))
            }
            Type::Record(name, _) => name.clone(),
            Type::Union(name, _) => name.clone(),
            // References in Java are just normal object references
            Type::Reference(inner) => self.map_type_from_type(inner),
            Type::MutableReference(inner) => self.map_type_from_type(inner),
            Type::Pointer(inner) => {
                // Java doesn't have pointers, use Long for native pointer
                format!("/* pointer to {} */ long", self.map_type_from_type(inner))
            }
            Type::ConstPointer(inner) => {
                format!("/* const pointer to {} */ long", self.map_type_from_type(inner))
            }
            Type::CInt => "int".to_string(),
            Type::CUInt => "int".to_string(),
            Type::CLong => "long".to_string(),
            Type::CULong => "long".to_string(),
            Type::CLongLong => "long".to_string(),
            Type::CULongLong => "long".to_string(),
            Type::CFloat => "float".to_string(),
            Type::CDouble => "double".to_string(),
            Type::CChar => "char".to_string(),
            Type::CSize => "long".to_string(),
            Type::CString => "String".to_string(),
            Type::Generic(name) => format!("{}<?>", name),
            Type::TypeParam(name) => format!("Object /* type param {} */", name),
            Type::TypeConstructor(name, args) => {
                let args_java: Vec<String> = args.iter().map(|a| self.map_type_from_type(a)).collect();
                format!("{}<{}>", name, args_java.join(", "))
            }
            Type::Var(name) => format!("Object /* type var {} */", name),
        }
    }

    /// Emit an expression
    fn emit_expr(&self, expr: &ast::Expression) -> JavaResult<String> {
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
                // Java ternary operator
                Ok(format!("({}) ? ({}) : ({})", c, t, e))
            }
            ExpressionKind::Member(obj, field) => {
                let o = self.emit_expr(obj)?;
                Ok(format!("{}.{}", o, field))
            }
            ExpressionKind::Wait(wait_type, exprs) => {
                self.emit_wait(wait_type, exprs)
            }
            ExpressionKind::Dictionary(entries) => {
                self.emit_dict_literal(entries)
            }
            ExpressionKind::Record(name, fields) => {
                self.emit_record_literal(name, fields)
            }
            ExpressionKind::Range(start, end, inclusive) => {
                self.emit_range_literal(start, end, *inclusive)
            }
            ExpressionKind::Pipe(input, funcs) => {
                self.emit_pipe(input, funcs)
            }
            ExpressionKind::Lambda(params, body) => {
                self.emit_lambda(params, body)
            }
            ExpressionKind::Match(expr, cases) => {
                self.emit_match_expr(expr, cases)
            }
            ExpressionKind::Needs(effect) => {
                Ok(format!("/* needs {} */ null", effect))
            }
            ExpressionKind::Given(name, value) => {
                let v = self.emit_expr(value)?;
                Ok(format!("/* given {} = {} */ null", name, v))
            }
            ExpressionKind::Handle(expr, handlers) => {
                let e = self.emit_expr(expr)?;
                Ok(format!("/* handle {} with {} handlers */", e, handlers.len()))
            }
            ExpressionKind::TryPropagate(expr) => {
                let e = self.emit_expr(expr)?;
                Ok(format!("{}.orElseThrow()", e))
            }
            ExpressionKind::Cast(expr, ty) => {
                let e = self.emit_expr(expr)?;
                let type_str = self.map_type_from_type(ty);
                Ok(format!("({}) {}", type_str, e))
            }
            ExpressionKind::Await(expr) => {
                let e = self.emit_expr(expr)?;
                Ok(format!("{}.get()", e))
            }
            ExpressionKind::OptionalChain(base_expr, member) => {
                let base = self.emit_expr(base_expr)?;
                Ok(format!("{} != null ? {}.{} : null", base, base, member))
            }
            ExpressionKind::NullCoalescing(left, right) => {
                let l = self.emit_expr(left)?;
                let r = self.emit_expr(right)?;
                Ok(format!("{} != null ? {} : {}", l, l, r))
            }
            ExpressionKind::Tuple(elements) => {
                let elems: Vec<String> = elements
                    .iter()
                    .map(|e| self.emit_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("Tuple.of({})", elems.join(", ")))
            }
        }
    }

    /// Emit a literal
    fn emit_literal(&self, lit: &ast::Literal) -> JavaResult<String> {
        match lit {
            ast::Literal::Integer(n) => Ok(format!("{}", n)),
            ast::Literal::Float(f) => Ok(format!("{}f", f)),
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
            ast::Literal::None => Ok("Optional.empty()".to_string()),
            ast::Literal::Unit => Ok("null".to_string()),
        }
    }

    /// Emit a binary operation
    fn emit_binop(&self, op: &ast::BinaryOp, l: &str, r: &str) -> String {
        match op {
            ast::BinaryOp::Add => format!("{} + {}", l, r),
            ast::BinaryOp::Sub => format!("{} - {}", l, r),
            ast::BinaryOp::Mul => format!("{} * {}", l, r),
            ast::BinaryOp::Div => format!("{} / {}", l, r),
            ast::BinaryOp::Mod => format!("{} % {}", l, r),
            ast::BinaryOp::Equal => format!("Objects.equals({}, {})", l, r),
            ast::BinaryOp::NotEqual => format!("!Objects.equals({}, {})", l, r),
            ast::BinaryOp::Less => format!("{} < {}", l, r),
            ast::BinaryOp::LessEqual => format!("{} <= {}", l, r),
            ast::BinaryOp::Greater => format!("{} > {}", l, r),
            ast::BinaryOp::GreaterEqual => format!("{} >= {}", l, r),
            ast::BinaryOp::And => format!("{} && {}", l, r),
            ast::BinaryOp::Or => format!("{} || {}", l, r),
            ast::BinaryOp::Pow => format!("Math.pow({}, {})", l, r),
            _ => format!("/* unsupported binop {:?} */", op),
        }
    }

    /// Emit a unary operation
    fn emit_unaryop(&self, op: &ast::UnaryOp, e: &str) -> String {
        match op {
            ast::UnaryOp::Negate => format!("-{}", e),
            ast::UnaryOp::Not => format!("!{}", e),
            ast::UnaryOp::Wait => format!("{}.get()", e), // Wait becomes CompletableFuture.get()
            _ => format!("/* unsupported unary {:?} */", op),
        }
    }

    /// Emit wait expression
    fn emit_wait(&self, wait_type: &ast::WaitType, exprs: &[ast::Expression]) -> JavaResult<String> {
        let expr_strs: Vec<String> = exprs
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<Result<Vec<_>, _>>()?;

        match wait_type {
            ast::WaitType::Single => {
                if expr_strs.len() == 1 {
                    Ok(format!("{}.get()", expr_strs[0]))
                } else {
                    let awaited: Vec<String> = expr_strs.iter().map(|e| format!("{}.get()", e)).collect();
                    Ok(format!("List.of({})", awaited.join(", ")))
                }
            }
            ast::WaitType::Together => {
                if expr_strs.is_empty() {
                    Ok("CompletableFuture.completedFuture(null)".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("{}.get()", expr_strs[0]))
                } else {
                    Ok(format!("CompletableFuture.allOf({}).join()", expr_strs.join(", ")))
                }
            }
            ast::WaitType::Race => {
                if expr_strs.is_empty() {
                    Ok("CompletableFuture.completedFuture(null)".to_string())
                } else if expr_strs.len() == 1 {
                    Ok(format!("{}.get()", expr_strs[0]))
                } else {
                    Ok(format!("CompletableFuture.anyOf({}).join()", expr_strs.join(", ")))
                }
            }
            ast::WaitType::Timeout(timeout_expr) => {
                let timeout = self.emit_expr(timeout_expr)?;
                if expr_strs.is_empty() {
                    Ok(format!("Thread.sleep({})", timeout))
                } else {
                    let expr = &expr_strs[0];
                    Ok(format!("{}.get({}, TimeUnit.MILLISECONDS)", expr, timeout))
                }
            }
            ast::WaitType::Atomic => {
                Ok(format!("/* atomic wait: {} */ {}.get()", expr_strs.join(", "), expr_strs[0]))
            }
            ast::WaitType::Retry => {
                Ok(format!("/* retry wait: {} */ {}.get()", expr_strs.join(", "), expr_strs[0]))
            }
        }
    }

    /// Emit a function call
    fn emit_call(
        &self,
        callee: &ast::Expression,
        args: &[ast::Expression],
    ) -> JavaResult<String> {
        // Handle special built-in functions
        if let ExpressionKind::Variable(name) = &callee.node {
            match name.as_str() {
                "print" | "println" => {
                    let arg_strs: Vec<String> = args.iter()
                        .map(|a| self.emit_expr(a))
                        .collect::<JavaResult<Vec<_>>>()?;
                    return Ok(format!("System.out.println({})", arg_strs.join(" + ")));
                }
                "assert" => {
                    let arg_strs: Vec<String> = args.iter()
                        .map(|a| self.emit_expr(a))
                        .collect::<JavaResult<Vec<_>>>()?;
                    return Ok(format!("assert {};", arg_strs.join(", ")));
                }
                _ => {}
            }
        }

        let callee_str = self.emit_expr(callee)?;
        let arg_strs: Vec<String> = args
            .iter()
            .map(|a| self.emit_expr(a))
            .collect::<JavaResult<Vec<_>>>()?;
        Ok(format!("{}({})", callee_str, arg_strs.join(", ")))
    }

    /// Emit an assignment expression
    fn emit_assign(
        &self,
        target: &ast::Expression,
        value: &ast::Expression,
    ) -> JavaResult<String> {
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

    /// Emit an array literal
    fn emit_array_literal(&self, elements: &[ast::Expression]) -> JavaResult<String> {
        if elements.is_empty() {
            return Ok("new Object[0]".to_string());
        }
        let elem_strs: Vec<String> = elements
            .iter()
            .map(|e| self.emit_expr(e))
            .collect::<JavaResult<Vec<_>>>()?;
        Ok(format!("new Object[]{{ {} }}", elem_strs.join(", ")))
    }

    /// Emit a dictionary literal
    fn emit_dict_literal(&self, entries: &[(ast::Expression, ast::Expression)]) -> JavaResult<String> {
        if entries.is_empty() {
            return Ok("new HashMap<>()".to_string());
        }
        let entry_strs: Vec<String> = entries.iter()
            .map(|(k, v)| {
                let k_str = self.emit_expr(k)?;
                let v_str = self.emit_expr(v)?;
                Ok(format!(".put({}, {})", k_str, v_str))
            })
            .collect::<JavaResult<Vec<_>>>()?;
        Ok(format!("new HashMap<>(){{ {} }}", entry_strs.join("")))
    }

    /// Emit a record literal
    fn emit_record_literal(&self, name: &str, fields: &[(String, ast::Expression)]) -> JavaResult<String> {
        let field_strs: Vec<String> = fields.iter()
            .map(|(k, v)| {
                let v_str = self.emit_expr(v)?;
                Ok(format!("{}.set{}({})", k, k, v_str))
            })
            .collect::<JavaResult<Vec<_>>>()?;
        Ok(format!("new {}(){{ {} }}", name, field_strs.join("")))
    }

    /// Emit a range literal
    fn emit_range_literal(&self, start: &ast::Expression, end: &ast::Expression, inclusive: bool) -> JavaResult<String> {
        let s = self.emit_expr(start)?;
        let e = self.emit_expr(end)?;
        if inclusive {
            Ok(format!("IntStream.rangeClosed({}, {})", s, e))
        } else {
            Ok(format!("IntStream.range({}, {})", s, e))
        }
    }

    /// Emit a pipe expression
    fn emit_pipe(&self, input: &ast::Expression, funcs: &[Box<ast::Expression>]) -> JavaResult<String> {
        let mut result = self.emit_expr(input)?;
        for func in funcs {
            let f = self.emit_expr(func)?;
            result = format!("{}.apply({})", f, result);
        }
        Ok(result)
    }

    /// Emit a lambda expression
    fn emit_lambda(&self, params: &[ast::Parameter], body: &ast::Block) -> JavaResult<String> {
        let param_strs: Vec<String> = params.iter()
            .map(|p| p.name.clone())
            .collect();

        // For simple lambdas with a single expression
        if body.statements.len() == 1 {
            if let StatementKind::Expression(expr) = &body.statements[0].node {
                let body_expr = self.emit_expr(expr)?;
                return Ok(format!("({}) -> {}", param_strs.join(", "), body_expr));
            }
        }

        // For complex lambdas, emit as a block
        // Note: This is a simplified version; real implementation would need more work
        Ok(format!("({}) -> {{ /* lambda body */ }}", param_strs.join(", ")))
    }

    /// Emit a match expression
    fn emit_match_expr(&self, expr: &ast::Expression, cases: &[ast::MatchCase]) -> JavaResult<String> {
        let e = self.emit_expr(expr)?;
        let _case_count = cases.len();
        // Simplified match expression - real implementation would need more complex code
        Ok(format!("/* match {} with {} cases */ null", e, _case_count))
    }

    /// 映射 LIR 类型到 Java 类型
    fn lir_type_to_java(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "void".to_string(),
            Bool => "boolean".to_string(),
            Char => "char".to_string(),
            Schar | Short => "short".to_string(),
            Uchar | Ushort | Int | Uint => "int".to_string(),
            Long | Ulong | LongLong | UlongLong => "long".to_string(),
            Float => "float".to_string(),
            Double | LongDouble => "double".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "long".to_string(),
            Pointer(inner) => format!("{}[]", self.lir_type_to_java(inner)),
            Array(inner, _) => format!("{}[]", self.lir_type_to_java(inner)),
            Named(n) => n.clone(),
            FunctionPointer(_, _) => "java.util.function.Function".to_string(),
            Qualified(_, inner) => self.lir_type_to_java(inner),
        }
    }

    /// 发射 LIR 语句
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> JavaResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{};", s))?;
            }
            Variable(v) => {
                let ty = self.lir_type_to_java(&v.type_);
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("{} {} = {};", ty, v.name, init_str))?;
                } else {
                    self.line(&format!("{} {};", ty, v.name))?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("if ({}) {{", cond))?;
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
                self.line(&format!("while ({}) {{", cond))?;
                self.indent();
                self.emit_lir_statement(&w.body)?;
                self.dedent();
                self.line("}")?;
            }
            Return(r) => {
                if let Some(e) = r {
                    let val = self.emit_lir_expr(e)?;
                    self.line(&format!("return {};", val))?;
                } else {
                    self.line("return;")?;
                }
            }
            Break => self.line("break;")?,
            Continue => self.line("continue;")?,
            _ => self.line("// unsupported statement")?,
        }
        Ok(())
    }

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> JavaResult<String> {
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
            _ => Ok("null".to_string()),
        }
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> JavaResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(format!("{}L", n)),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!("\"{}\"", s)),
            Char(c) => Ok(format!("'{}'", c)),
            Bool(b) => Ok(b.to_string()),
            NullPointer => Ok("null".to_string()),
        }
    }

    /// 映射 LIR 二元运算符
    fn map_lir_binop(&self, op: &x_lir::BinaryOp) -> String {
        use x_lir::BinaryOp::*;
        match op {
            Add => "+",
            Subtract => "-",
            Multiply => "*",
            Divide => "/",
            Modulo => "%",
            LeftShift => "<<",
            RightShift => ">>",
            RightShiftArithmetic => ">>>",
            LessThan => "<",
            LessThanEqual => "<=",
            GreaterThan => ">",
            GreaterThanEqual => ">=",
            Equal => "==",
            NotEqual => "!=",
            BitAnd => "&",
            BitOr => "|",
            BitXor => "^",
            LogicalAnd => "&&",
            LogicalOr => "||",
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
            // Increment/decrement are handled at the statement level
            PreIncrement => "++".to_string(),
            PreDecrement => "--".to_string(),
            PostIncrement => "++".to_string(),
            PostDecrement => "--".to_string(),
        }
    }
}

impl CodeGenerator for JavaBackend {
    type Config = JavaConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_ast(&mut self, program: &AstProgram) -> Result<CodegenOutput, Self::Error> {
        self.generate_from_ast(program)
    }

    fn generate_from_hir(&mut self, _hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error> {
        // TODO: 实现 HIR -> Java 源码生成
        Err(x_codegen::CodeGenError::Unimplemented("Java 后端 HIR 生成尚未实现".to_string()))
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        // 简单的 LIR -> Java 代码生成
        self.buffer.clear();

        self.emit_header().map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;

        // 开始类定义
        self.line(&format!("public class {} {{", self.config.class_name)).map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;
        self.indent();

        // 收集函数
        let mut has_main = false;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    has_main = true;
                }
                // 发射函数签名
                let ret = self.lir_type_to_java(&f.return_type);
                let params: Vec<String> = f.parameters.iter()
                    .map(|p| format!("{} {}", self.lir_type_to_java(&p.type_), p.name))
                    .collect();
                self.line(&format!("    public static {} {}({}) {{", ret, f.name, params.join(", "))).map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;
                self.indent();

                // 发射函数体
                for stmt in &f.body.statements {
                    self.emit_lir_statement(stmt).map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;
                }

                self.dedent();
                self.line("    }").map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;
                self.line("").map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;
            }
        }

        // main 方法
        self.line("    public static void main(String[] args) {").map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;
        self.indent();
        if has_main {
            self.line("        main(args);").map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;
        }
        self.dedent();
        self.line("    }").map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;

        self.dedent();
        self.line("}").map_err(|e| x_codegen::CodeGenError::Unimplemented(e.to_string()))?;

        let output_file = OutputFile {
            path: PathBuf::from(format!("{}.java", self.config.class_name)),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::Java,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

// 保持向后兼容的别名
pub type JavaCodeGenerator = JavaBackend;
pub type JavaCodeGenError = x_codegen::CodeGenError;

#[cfg(test)]
mod tests {
    use super::*;
    use x_lexer::span::Span;
    use x_parser::ast::{Spanned, MethodModifiers};

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
                name: "main".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                return_type: None,
                effects: vec![],
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
                span: Span::default(),
            })],
            statements: vec![],
        };

        let mut backend = JavaBackend::new(JavaConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let java_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(java_code.contains("System.out.println(\"Hello, World!\")"));
        assert!(java_code.contains("public static void main(String[] args)"));
    }

    #[test]
    fn test_empty_program_generates_default_main() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![],
            statements: vec![],
        };

        let mut backend = JavaBackend::new(JavaConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let java_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(java_code.contains("public class Main"));
        assert!(java_code.contains("public static void main(String[] args)"));
        assert!(java_code.contains("System.out.println(\"Hello from Java backend!\")"));
    }

    #[test]
    fn test_function_with_parameters() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                name: "add".to_string(),
                type_parameters: vec![],
                parameters: vec![
                    ast::Parameter {
                        name: "a".to_string(),
                        type_annot: Some(Type::Int),
                        default: None,
                        span: Span::default(),
                    },
                    ast::Parameter {
                        name: "b".to_string(),
                        type_annot: Some(Type::Int),
                        default: None,
                        span: Span::default(),
                    },
                ],
                effects: vec![],
                return_type: Some(Type::Int),
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::Return(Some(make_expr(
                        ExpressionKind::Binary(
                            ast::BinaryOp::Add,
                            Box::new(make_expr(ExpressionKind::Variable("a".to_string()))),
                            Box::new(make_expr(ExpressionKind::Variable("b".to_string()))),
                        ),
                    ))))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
                span: Span::default(),
            })],
            statements: vec![],
        };

        let mut backend = JavaBackend::new(JavaConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let java_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(java_code.contains("public static int add(int a, int b)"));
        assert!(java_code.contains("return a + b"));
    }

    #[test]
    fn test_class_generation() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Class(ast::ClassDecl {
                name: "Point".to_string(),
                type_parameters: vec![],
                extends: None,
                implements: vec![],
                members: vec![
                    ast::ClassMember::Field(ast::VariableDecl {
                        name: "x".to_string(),
                        is_mutable: true,
                        is_constant: false,
                        type_annot: Some(Type::Int),
                        initializer: None,
                        visibility: ast::Visibility::Public,
                        span: Span::default(),
                    }),
                    ast::ClassMember::Field(ast::VariableDecl {
                        name: "y".to_string(),
                        is_mutable: true,
                        is_constant: false,
                        type_annot: Some(Type::Int),
                        initializer: None,
                        visibility: ast::Visibility::Public,
                        span: Span::default(),
                    }),
                ],
                modifiers: ast::ClassModifiers::default(),
                span: Span::default(),
            })],
            statements: vec![],
        };

        let mut backend = JavaBackend::new(JavaConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let java_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(java_code.contains("public static class Point"));
        assert!(java_code.contains("public int x"));
        assert!(java_code.contains("public int y"));
    }

    #[test]
    fn test_if_statement() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                name: "test_if".to_string(),
                type_parameters: vec![],
                parameters: vec![],
                return_type: None,
                effects: vec![],
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::If(ast::IfStatement {
                        condition: make_expr(ExpressionKind::Literal(ast::Literal::Boolean(true))),
                        then_block: ast::Block {
                            statements: vec![make_stmt(StatementKind::Expression(make_expr(
                                ExpressionKind::Call(
                                    Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                                    vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                        "true".to_string(),
                                    )))],
                                ),
                            )))],
                        },
                        else_block: Some(ast::Block {
                            statements: vec![make_stmt(StatementKind::Expression(make_expr(
                                ExpressionKind::Call(
                                    Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                                    vec![make_expr(ExpressionKind::Literal(ast::Literal::String(
                                        "false".to_string(),
                                    )))],
                                ),
                            )))],
                        }),
                    }))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
                span: Span::default(),
            })],
            statements: vec![],
        };

        let mut backend = JavaBackend::new(JavaConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let java_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(java_code.contains("if (true) {"));
        assert!(java_code.contains("} else {"));
    }

    #[test]
    fn test_while_statement() {
        let program = AstProgram {
            span: Span::default(),
            declarations: vec![ast::Declaration::Function(ast::FunctionDecl {
                name: "count_down".to_string(),
                type_parameters: vec![],
                parameters: vec![ast::Parameter {
                    name: "n".to_string(),
                    type_annot: Some(Type::Int),
                    default: None,
                    span: Span::default(),
                }],
                return_type: None,
                effects: vec![],
                body: ast::Block {
                    statements: vec![make_stmt(StatementKind::While(ast::WhileStatement {
                        condition: make_expr(ExpressionKind::Binary(
                            ast::BinaryOp::Greater,
                            Box::new(make_expr(ExpressionKind::Variable("n".to_string()))),
                            Box::new(make_expr(ExpressionKind::Literal(ast::Literal::Integer(0)))),
                        )),
                        body: ast::Block {
                            statements: vec![
                                make_stmt(StatementKind::Expression(make_expr(ExpressionKind::Call(
                                    Box::new(make_expr(ExpressionKind::Variable("print".to_string()))),
                                    vec![make_expr(ExpressionKind::Variable("n".to_string()))],
                                )))),
                                make_stmt(StatementKind::Expression(make_expr(ExpressionKind::Assign(
                                    Box::new(make_expr(ExpressionKind::Variable("n".to_string()))),
                                    Box::new(make_expr(ExpressionKind::Binary(
                                        ast::BinaryOp::Sub,
                                        Box::new(make_expr(ExpressionKind::Variable("n".to_string()))),
                                        Box::new(make_expr(ExpressionKind::Literal(ast::Literal::Integer(1)))),
                                    ))),
                                )))),
                            ],
                        },
                    }))],
                },
                is_async: false,
                modifiers: MethodModifiers::default(),
                span: Span::default(),
            })],
            statements: vec![],
        };

        let mut backend = JavaBackend::new(JavaConfig::default());
        let output = backend.generate_from_ast(&program).unwrap();
        let java_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(java_code.contains("while (n > 0)"));
        assert!(java_code.contains("n = n - 1"));
    }

    #[test]
    fn test_type_mapping() {
        let backend = JavaBackend::new(JavaConfig::default());

        // Test basic types
        assert_eq!(backend.map_type_from_type(&Type::Int), "int");
        assert_eq!(backend.map_type_from_type(&Type::Float), "double");
        assert_eq!(backend.map_type_from_type(&Type::Bool), "boolean");
        assert_eq!(backend.map_type_from_type(&Type::String), "String");

        // Test generic types
        assert_eq!(backend.map_type_from_type(&Type::Array(Box::new(Type::Int))), "List<int>");

        // Test optional
        assert_eq!(backend.map_type_from_type(&Type::Option(Box::new(Type::Int))), "Optional<int>");

        // Test none type
        assert_eq!(backend.map_type(&None), "var");
    }
}
