//! Swift 后端 - 生成 Swift 6 源代码
//!
//! 面向 Apple 生态（iOS、macOS、watchOS、tvOS）及 Linux
//!
//! ## Swift 6.x 特性支持
//! - Data-race safety in concurrent code
//! - Typed throws
//! - Non-copyable types with generics
//! - 128-bit integer types
//! - Embedded Swift for microcontrollers
//! - Swift Testing library
//! - C++ interoperability improvements
//! - Strict concurrency by default

#![allow(clippy::only_used_in_recursion, clippy::useless_format)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;

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

// 保持向后兼容的别名
pub type SwiftCodeGenerator = SwiftBackend;
pub type SwiftCodeGenError = x_codegen::CodeGenError;

impl SwiftBackend {
    pub fn new(config: SwiftBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
        }
    }

    fn line(&mut self, s: &str) -> SwiftResult<()> {
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

    /// Emit file header (Swift 6)
    fn emit_header(&mut self) -> SwiftResult<()> {
        self.line(headers::SWIFT)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: Swift 6")?;
        self.line("")?;
        self.line("import Foundation")?;
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // LIR type mapping
    // ========================================================================

    /// Map LIR type to Swift type
    fn lir_type_to_swift(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "Void".to_string(),
            Bool => "Bool".to_string(),
            Char => "Character".to_string(),
            Schar | Short => "Int16".to_string(),
            Uchar | Ushort => "UInt16".to_string(),
            Int => "Int".to_string(),
            Uint => "UInt".to_string(),
            Long | LongLong => "Int64".to_string(),
            Ulong | UlongLong => "UInt64".to_string(),
            Float => "Float".to_string(),
            Double | LongDouble => "Double".to_string(),
            Size | Uintptr => "UInt".to_string(),
            Ptrdiff | Intptr => "Int".to_string(),
            Pointer(inner) => format!("UnsafeMutablePointer<{}>", self.lir_type_to_swift(inner)),
            Array(inner, _) => format!("[{}]", self.lir_type_to_swift(inner)),
            FunctionPointer(ret, params) => {
                let param_strs: Vec<String> =
                    params.iter().map(|p| self.lir_type_to_swift(p)).collect();
                let ret_str = self.lir_type_to_swift(ret);
                format!("({}) -> {}", param_strs.join(", "), ret_str)
            }
            Named(n) => n.clone(),
            Qualified(_, inner) => self.lir_type_to_swift(inner),
        }
    }

    // ========================================================================
    // LIR statement emission
    // ========================================================================

    /// Emit a LIR statement
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> SwiftResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                // Handle assignment expressions where rhs is a print call
                if let x_lir::Expression::Assign(target, value) = e {
                    if let x_lir::Expression::Call(callee, args) = value.as_ref() {
                        if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                            let name = fn_name.as_str();
                            if matches!(name, "println" | "print" | "eprintln" | "eprintln!") {
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_lir_expr(a))
                                    .collect::<Result<Vec<_>, _>>()?;
                                let args_part = args_str.join(", ");

                                let call_str = if name == "eprintln" || name == "eprintln!" {
                                    format!(
                                        "fputs({} + \"\\n\", stderr)",
                                        if args_part.is_empty() {
                                            "\"\"".to_string()
                                        } else {
                                            args_part
                                        }
                                    )
                                } else if name == "println" {
                                    format!("print({})", args_part)
                                } else {
                                    // "print" without newline
                                    format!("print({}, terminator: \"\")", args_part)
                                };
                                self.line(&call_str)?;
                                return Ok(());
                            }
                        }
                    }
                    // General assignment
                    let target_str = self.emit_lir_expr(target)?;
                    let value_str = self.emit_lir_expr(value)?;
                    self.line(&format!("{} = {}", target_str, value_str))?;
                    return Ok(());
                }
                // Regular expression statement
                let s = self.emit_lir_expr(e)?;
                self.line(&s)?;
            }
            Variable(v) => {
                let ty = self.lir_type_to_swift(&v.type_);
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("var {}: {} = {}", v.name, ty, init_str))?;
                } else {
                    // Swift requires initialization; declare with default
                    self.line(&format!("var {}: {}", v.name, ty))?;
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
            DoWhile(d) => {
                self.line("repeat {")?;
                self.indent();
                self.emit_lir_statement(&d.body)?;
                self.dedent();
                let cond = self.emit_lir_expr(&d.condition)?;
                self.line(&format!("}} while {}", cond))?;
            }
            For(f) => {
                // Swift doesn't have C-style for loops; emit as while
                if let Some(init) = &f.initializer {
                    self.emit_lir_statement(init)?;
                }
                let cond = if let Some(c) = &f.condition {
                    self.emit_lir_expr(c)?
                } else {
                    "true".to_string()
                };
                self.line(&format!("while {} {{", cond))?;
                self.indent();
                self.emit_lir_statement(&f.body)?;
                if let Some(inc) = &f.increment {
                    let inc_str = self.emit_lir_expr(inc)?;
                    self.line(&inc_str)?;
                }
                self.dedent();
                self.line("}")?;
            }
            Switch(s) => {
                let expr = self.emit_lir_expr(&s.expression)?;
                self.line(&format!("switch {} {{", expr))?;
                self.indent();
                for case in &s.cases {
                    let val = self.emit_lir_expr(&case.value)?;
                    self.line(&format!("case {}:", val))?;
                    self.indent();
                    self.emit_lir_statement(&case.body)?;
                    self.dedent();
                }
                if let Some(default) = &s.default {
                    self.line("default:")?;
                    self.indent();
                    self.emit_lir_statement(default)?;
                    self.dedent();
                }
                self.dedent();
                self.line("}")?;
            }
            Match(m) => {
                let scrutinee = self.emit_lir_expr(&m.scrutinee)?;
                self.line(&format!("switch {} {{", scrutinee))?;
                self.indent();
                for case in &m.cases {
                    let pattern = self.emit_lir_pattern(&case.pattern);
                    if let Some(guard) = &case.guard {
                        let guard_str = self.emit_lir_expr(guard)?;
                        self.line(&format!("case {} where {}:", pattern, guard_str))?;
                    } else {
                        self.line(&format!("case {}:", pattern))?;
                    }
                    self.indent();
                    for stmt in &case.body.statements {
                        self.emit_lir_statement(stmt)?;
                    }
                    self.dedent();
                }
                self.dedent();
                self.line("}")?;
            }
            Try(t) => {
                self.line("do {")?;
                self.indent();
                for stmt in &t.body.statements {
                    self.emit_lir_statement(stmt)?;
                }
                self.dedent();
                for catch in &t.catch_clauses {
                    let catch_line = if let Some(var_name) = &catch.variable_name {
                        if let Some(exc_type) = &catch.exception_type {
                            format!("}} catch let {} as {} {{", var_name, exc_type)
                        } else {
                            format!("}} catch let {} {{", var_name)
                        }
                    } else if let Some(exc_type) = &catch.exception_type {
                        format!("}} catch let error as {} {{", exc_type)
                    } else {
                        "} catch {".to_string()
                    };
                    self.line(&catch_line)?;
                    self.indent();
                    for stmt in &catch.body.statements {
                        self.emit_lir_statement(stmt)?;
                    }
                    self.dedent();
                }
                if let Some(finally) = &t.finally_block {
                    // Swift doesn't have finally; use defer semantics in a comment
                    self.line("} // finally:")?;
                    for stmt in &finally.statements {
                        self.emit_lir_statement(stmt)?;
                    }
                } else {
                    self.line("}")?;
                }
            }
            Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let val = self.emit_lir_expr(expr)?;
                    self.line(&format!("return {}", val))?;
                } else {
                    self.line("return")?;
                }
            }
            Break => self.line("break")?,
            Continue => self.line("continue")?,
            Label(name) => {
                self.line(&format!("// label: {}", name))?;
            }
            Goto(name) => {
                self.line(&format!("// goto: {}", name))?;
            }
            Empty => {}
            Compound(block) => {
                for stmt in &block.statements {
                    self.emit_lir_statement(stmt)?;
                }
            }
            Declaration(_) => {
                self.line("// nested declaration")?;
            }
        }
        Ok(())
    }

    // ========================================================================
    // LIR expression emission
    // ========================================================================

    /// Emit a LIR expression
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
                let inner = self.emit_lir_expr(e)?;
                let op_str = self.map_lir_unaryop(op);
                // Post-increment/decrement go after the expression
                match op {
                    x_lir::UnaryOp::PostIncrement | x_lir::UnaryOp::PostDecrement => {
                        Ok(format!("({}{})", inner, op_str))
                    }
                    _ => Ok(format!("({}{})", op_str, inner)),
                }
            }
            Ternary(cond, then_e, else_e) => {
                let c = self.emit_lir_expr(cond)?;
                let t = self.emit_lir_expr(then_e)?;
                let e = self.emit_lir_expr(else_e)?;
                Ok(format!("({} ? {} : {})", c, t, e))
            }
            Assign(target, value) => {
                let t = self.emit_lir_expr(target)?;
                let v = self.emit_lir_expr(value)?;
                Ok(format!("{} = {}", t, v))
            }
            AssignOp(op, target, value) => {
                let t = self.emit_lir_expr(target)?;
                let v = self.emit_lir_expr(value)?;
                let op_str = self.map_lir_binop(op);
                Ok(format!("{} {}= {}", t, op_str, v))
            }
            Call(callee, args) => {
                let callee_str = self.emit_lir_expr(callee)?;
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // Map built-in functions to Swift equivalents
                match callee_str.as_str() {
                    "println" => {
                        let args_part = args_str.join(", ");
                        return Ok(format!("print({})", args_part));
                    }
                    "print" => {
                        let args_part = args_str.join(", ");
                        return Ok(format!("print({}, terminator: \"\")", args_part));
                    }
                    "eprintln" | "eprintln!" => {
                        let args_part = if args_str.is_empty() {
                            "\"\"".to_string()
                        } else {
                            args_str.join(", ")
                        };
                        return Ok(format!("fputs({} + \"\\n\", stderr)", args_part));
                    }
                    "format" => {
                        if args_str.is_empty() {
                            return Ok("\"\"".to_string());
                        }
                        // Swift string interpolation
                        return Ok(format!("\"\\({})\"", args_str.join(", ")));
                    }
                    _ => {}
                }
                Ok(format!("{}({})", callee_str, args_str.join(", ")))
            }
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("{}[{}]", arr_str, idx_str))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            PointerMember(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.pointee.{}", obj_str, member))
            }
            AddressOf(e) => {
                let inner = self.emit_lir_expr(e)?;
                Ok(format!("&{}", inner))
            }
            Dereference(e) => {
                let inner = self.emit_lir_expr(e)?;
                Ok(format!("{}.pointee", inner))
            }
            Cast(ty, e) => {
                let inner = self.emit_lir_expr(e)?;
                let swift_type = self.lir_type_to_swift(ty);
                Ok(format!("{}({})  ", swift_type, inner))
            }
            SizeOf(ty) => {
                let swift_type = self.lir_type_to_swift(ty);
                Ok(format!("MemoryLayout<{}>.size", swift_type))
            }
            SizeOfExpr(e) => {
                let inner = self.emit_lir_expr(e)?;
                Ok(format!("MemoryLayout.size(ofValue: {})", inner))
            }
            AlignOf(ty) => {
                let swift_type = self.lir_type_to_swift(ty);
                Ok(format!("MemoryLayout<{}>.alignment", swift_type))
            }
            Comma(exprs) => {
                // Swift doesn't have the comma operator; emit last expression
                if let Some(last) = exprs.last() {
                    self.emit_lir_expr(last)
                } else {
                    Ok("()".to_string())
                }
            }
            Parenthesized(e) => {
                let inner = self.emit_lir_expr(e)?;
                Ok(format!("({})", inner))
            }
            InitializerList(inits) => {
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", init_strs.join(", ")))
            }
            CompoundLiteral(ty, inits) => {
                let swift_type = self.lir_type_to_swift(ty);
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{}({})", swift_type, init_strs.join(", ")))
            }
        }
    }

    /// Emit a LIR literal
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> SwiftResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(n.to_string()),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!(
                "\"{}\"",
                s.replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
            )),
            Char(c) => {
                // Swift Character is created from a String literal
                Ok(format!("Character(\"{}\")", c))
            }
            Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            NullPointer => Ok("nil".to_string()),
        }
    }

    /// Emit a LIR initializer
    fn emit_lir_initializer(&self, init: &x_lir::Initializer) -> SwiftResult<String> {
        match init {
            x_lir::Initializer::Expression(e) => self.emit_lir_expr(e),
            x_lir::Initializer::List(items) => {
                let strs: Vec<String> = items
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", strs.join(", ")))
            }
            x_lir::Initializer::Named(name, inner) => {
                let val = self.emit_lir_initializer(inner)?;
                Ok(format!("{}: {}", name, val))
            }
            x_lir::Initializer::Indexed(idx, inner) => {
                let idx_str = self.emit_lir_expr(idx)?;
                let val = self.emit_lir_initializer(inner)?;
                Ok(format!("/* [{}] */ {}", idx_str, val))
            }
        }
    }

    /// Emit a LIR pattern for switch/match
    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> String {
        match pattern {
            x_lir::Pattern::Wildcard => "_".to_string(),
            x_lir::Pattern::Variable(name) => format!("let {}", name),
            x_lir::Pattern::Literal(lit) => self
                .emit_lir_literal(lit)
                .unwrap_or_else(|_| "_".to_string()),
            x_lir::Pattern::Constructor(name, pats) => {
                let pat_strs: Vec<String> = pats.iter().map(|p| self.emit_lir_pattern(p)).collect();
                if pats.is_empty() {
                    format!(".{}", name)
                } else {
                    format!(".{}({})", name, pat_strs.join(", "))
                }
            }
            x_lir::Pattern::Tuple(pats) => {
                let pat_strs: Vec<String> = pats.iter().map(|p| self.emit_lir_pattern(p)).collect();
                format!("({})", pat_strs.join(", "))
            }
            x_lir::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.emit_lir_pattern(v)))
                    .collect();
                format!("{}({})", name, field_strs.join(", "))
            }
            x_lir::Pattern::Or(left, right) => {
                format!(
                    "{}, {}",
                    self.emit_lir_pattern(left),
                    self.emit_lir_pattern(right)
                )
            }
        }
    }

    // ========================================================================
    // Operator mapping
    // ========================================================================

    /// Map LIR binary operator to Swift operator string
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
            RightShiftArithmetic => ">>", // Swift >> is arithmetic for signed types
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
        }
        .to_string()
    }

    /// Map LIR unary operator to Swift operator string
    fn map_lir_unaryop(&self, op: &x_lir::UnaryOp) -> String {
        use x_lir::UnaryOp::*;
        match op {
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Not => "!".to_string(),
            BitNot => "~".to_string(),
            PreIncrement => "/* ++pre */ ".to_string(),
            PreDecrement => "/* --pre */ ".to_string(),
            PostIncrement => " /* post++ */".to_string(),
            PostDecrement => " /* post-- */".to_string(),
            Reference => "&".to_string(),
            MutableReference => "&".to_string(),
        }
    }

    // ========================================================================
    // Top-level LIR -> Swift code generation
    // ========================================================================

    /// Generate Swift source from a LIR program
    pub fn generate_from_lir_impl(&mut self, lir: &LirProgram) -> SwiftResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // Collect and emit functions
        let mut has_main = false;

        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                let ret = self.lir_type_to_swift(&f.return_type);
                let params: Vec<String> = f
                    .parameters
                    .iter()
                    .map(|p| format!("_ {}: {}", p.name, self.lir_type_to_swift(&p.type_)))
                    .collect();

                if f.name == "main" {
                    has_main = true;
                    // Swift uses top-level code; emit main body directly
                    // or wrap it in a func main() and call it
                    self.line("func main() {")?;
                } else {
                    self.line(&format!(
                        "func {}({}) -> {} {{",
                        f.name,
                        params.join(", "),
                        ret
                    ))?;
                }
                self.indent();

                for stmt in &f.body.statements {
                    // Skip `return 0` in main (Swift main is Void)
                    if f.name == "main" {
                        if let x_lir::Statement::Return(Some(expr)) = stmt {
                            if let x_lir::Expression::Literal(x_lir::Literal::Integer(0)) = expr {
                                continue;
                            }
                        }
                    }
                    self.emit_lir_statement(stmt)?;
                }

                self.dedent();
                self.line("}")?;
                self.line("")?;
            }
        }

        // Call main if it exists; otherwise emit a default
        if has_main {
            self.line("main()")?;
        } else {
            self.line("func main() {")?;
            self.indent();
            self.line("print(\"Hello from Swift!\")")?;
            self.dedent();
            self.line("}")?;
            self.line("")?;
            self.line("main()")?;
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

impl CodeGenerator for SwiftBackend {
    type Config = SwiftBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        self.generate_from_lir_impl(lir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SwiftBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
        assert_eq!(config.target, SwiftTarget::MacOS);
    }

    #[test]
    fn test_lir_type_mapping() {
        let backend = SwiftBackend::new(SwiftBackendConfig::default());
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Int), "Int");
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Uint), "UInt");
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Long), "Int64");
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Float), "Float");
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Double), "Double");
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Bool), "Bool");
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Char), "Character");
        assert_eq!(backend.lir_type_to_swift(&x_lir::Type::Void), "Void");
    }

    #[test]
    fn test_empty_lir_program() {
        let lir = LirProgram {
            declarations: vec![],
        };
        let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
        let output = backend.generate_from_lir_impl(&lir).unwrap();
        let swift_code = String::from_utf8_lossy(&output.files[0].content);

        assert!(swift_code.contains("func main()"));
        assert!(swift_code.contains("print(\"Hello from Swift!\")"));
        assert!(swift_code.contains("main()"));
    }

    #[test]
    fn test_lir_binop_mapping() {
        let backend = SwiftBackend::new(SwiftBackendConfig::default());
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::Add), "+");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::Subtract), "-");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::Multiply), "*");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::Equal), "==");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::LogicalAnd), "&&");
    }
}
