//! Python backend - compiles X LIR to Python 3.14 source code
//!
//! Generates clean, readable Python source code from LIR.
//!
//! ## Python 3.14 features (February 2026)
//! - Template strings (t-strings) - PEP 750
//! - Deferred evaluation of annotations - PEP 649/749
//! - Bracketless exception handling - PEP 758
//! - Type hints with union syntax (`X | Y`)
//! - Pattern matching (`match`/`case`)

#![allow(clippy::only_used_in_recursion, clippy::useless_format)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
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
    #[allow(dead_code)]
    config: PythonBackendConfig,
    /// Code buffer
    buffer: x_codegen::CodeBuffer,
}

pub type PythonResult<T> = Result<T, x_codegen::CodeGenError>;

// Backward-compatible aliases
pub type PythonCodeGenerator = PythonBackend;
pub type PythonCodeGenError = x_codegen::CodeGenError;

impl PythonBackend {
    pub fn new(config: PythonBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
        }
    }

    fn line(&mut self, s: &str) -> PythonResult<()> {
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

    fn sanitize_identifier(&self, name: &str) -> String {
        match name {
            "False" | "None" | "True" | "and" | "as" | "assert" | "async" | "await"
            | "break" | "class" | "continue" | "def" | "del" | "elif" | "else"
            | "except" | "finally" | "for" | "from" | "global" | "if" | "import"
            | "in" | "is" | "lambda" | "nonlocal" | "not" | "or" | "pass"
            | "raise" | "return" | "try" | "while" | "with" | "yield" | "match"
            | "case" => format!("{}_x", name),
            _ => name.to_string(),
        }
    }

    fn sanitize_parameters(&self, params: &[x_lir::Parameter]) -> String {
        params
            .iter()
            .map(|p| self.sanitize_identifier(&p.name))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Emit file header with imports (Python 3.14)
    fn emit_header(&mut self) -> PythonResult<()> {
        self.line(headers::PYTHON)?;
        self.line("# DO NOT EDIT")?;
        self.line("# Target: Python 3.14 (February 2026)")?;
        self.line("# Requires: Python >= 3.14")?;
        self.line("")?;
        self.line("from __future__ import annotations")?;
        self.line("import builtins")?;
        self.line("import sys")?;
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // LIR type mapping
    // ========================================================================

    /// Map LIR type to Python type annotation string
    fn lir_type_to_python(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "None".to_string(),
            Bool => "bool".to_string(),
            Char => "str".to_string(),
            Schar | Short | Int | Uint | Long | Ulong | LongLong | UlongLong | Uchar | Ushort => {
                "int".to_string()
            }
            Float | Double | LongDouble => "float".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "int".to_string(),
            Pointer(inner) => format!("list[{}]", self.lir_type_to_python(inner)),
            Array(inner, _) => format!("list[{}]", self.lir_type_to_python(inner)),
            Tuple(items) => {
                let item_strs: Vec<String> = items.iter().map(|item| self.lir_type_to_python(item)).collect();
                format!("tuple[{}]", item_strs.join(", "))
            }
            Named(n) => n.clone(),
            FunctionPointer(_, _) => "callable".to_string(),
            Qualified(_, inner) => self.lir_type_to_python(inner),
        }
    }

    // ========================================================================
    // LIR statement emission
    // ========================================================================

    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> PythonResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                // Handle assignment with void-returning calls (println, etc.)
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
                                    if args_part.is_empty() {
                                        "builtins.print(file=sys.stderr)".to_string()
                                    } else {
                                        format!("builtins.print({}, file=sys.stderr)", args_part)
                                    }
                                } else {
                                    format!("builtins.print({})", args_part)
                                };
                                self.line(&call_str)?;
                                return Ok(());
                            }
                            if name == "panic" {
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_lir_expr(a))
                                    .collect::<Result<Vec<_>, _>>()?;
                                let panic_stmt = if args_str.is_empty() {
                                    "raise Exception()".to_string()
                                } else {
                                    format!("raise Exception({})", args_str.join(", "))
                                };
                                self.line(&panic_stmt)?;
                                return Ok(());
                            }
                        }
                    }
                    // Other assignments
                    let target_str = self.emit_lir_expr(target)?;
                    let value_str = self.emit_lir_expr(value)?;
                    if value_str.starts_with("raise ") {
                        self.line(&value_str)?;
                        return Ok(());
                    }
                    self.line(&format!("{} = {}", target_str, value_str))?;
                    return Ok(());
                }
                let s = self.emit_lir_expr(e)?;
                self.line(&s)?;
            }
            Variable(v) => {
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("{} = {}", self.sanitize_identifier(&v.name), init_str))?;
                } else {
                    // Python variables need a value; use None as default
                    self.line(&format!("{} = None", self.sanitize_identifier(&v.name)))?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("if {}:", cond))?;
                self.indent();
                self.emit_lir_statement(&i.then_branch)?;
                self.dedent();
                if let Some(else_br) = &i.else_branch {
                    self.line("else:")?;
                    self.indent();
                    self.emit_lir_statement(else_br)?;
                    self.dedent();
                }
            }
            While(w) => {
                let cond = self.emit_lir_expr(&w.condition)?;
                self.line(&format!("while {}:", cond))?;
                self.indent();
                self.emit_lir_statement(&w.body)?;
                self.dedent();
            }
            DoWhile(d) => {
                self.line("while True:")?;
                self.indent();
                self.emit_lir_statement(&d.body)?;
                let cond = self.emit_lir_expr(&d.condition)?;
                self.line(&format!("if not ({}):", cond))?;
                self.indent();
                self.line("break")?;
                self.dedent();
                self.dedent();
            }
            For(f) => {
                // LIR for-loops map to Python while-loops
                if let Some(init) = &f.initializer {
                    self.emit_lir_statement(init)?;
                }
                let cond = if let Some(c) = &f.condition {
                    self.emit_lir_expr(c)?
                } else {
                    "True".to_string()
                };
                self.line(&format!("while {}:", cond))?;
                self.indent();
                self.emit_lir_statement(&f.body)?;
                if let Some(inc) = &f.increment {
                    let inc_str = self.emit_lir_expr(inc)?;
                    self.line(&inc_str)?;
                }
                self.dedent();
            }
            Switch(sw) => {
                // Python 3.10+ match/case
                let expr = self.emit_lir_expr(&sw.expression)?;
                self.line(&format!("match {}:", expr))?;
                self.indent();
                for case in &sw.cases {
                    let val = self.emit_lir_expr(&case.value)?;
                    self.line(&format!("case {}:", val))?;
                    self.indent();
                    self.emit_lir_statement(&case.body)?;
                    self.dedent();
                }
                if let Some(default) = &sw.default {
                    self.line("case _:")?;
                    self.indent();
                    self.emit_lir_statement(default)?;
                    self.dedent();
                }
                self.dedent();
            }
            Match(m) => {
                let expr = self.emit_lir_expr(&m.scrutinee)?;
                self.line(&format!("match {}:", expr))?;
                self.indent();
                for case in &m.cases {
                    let pattern = self.emit_lir_pattern(&case.pattern);
                    if let Some(guard) = &case.guard {
                        let guard_str = self.emit_lir_expr(guard)?;
                        self.line(&format!("case {} if {}:", pattern, guard_str))?;
                    } else {
                        self.line(&format!("case {}:", pattern))?;
                    }
                    self.indent();
                    for s in &case.body.statements {
                        self.emit_lir_statement(s)?;
                    }
                    self.dedent();
                }
                self.dedent();
            }
            Try(t) => {
                self.line("try:")?;
                self.indent();
                for s in &t.body.statements {
                    self.emit_lir_statement(s)?;
                }
                self.dedent();

                for catch in &t.catch_clauses {
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
                    self.indent();
                    for s in &catch.body.statements {
                        self.emit_lir_statement(s)?;
                    }
                    self.dedent();
                }

                if let Some(finally) = &t.finally_block {
                    self.line("finally:")?;
                    self.indent();
                    for s in &finally.statements {
                        self.emit_lir_statement(s)?;
                    }
                    self.dedent();
                }
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
            Label(_) => { /* labels have no Python equivalent, skip */ }
            Goto(_) => { /* goto has no Python equivalent, skip */ }
            Empty => self.line("pass")?,
            Compound(block) => {
                for s in &block.statements {
                    self.emit_lir_statement(s)?;
                }
            }
            Declaration(_) => {
                self.line("pass  # nested declaration")?;
            }
        }
        Ok(())
    }

    // ========================================================================
    // LIR pattern emission (for match/case)
    // ========================================================================

    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> String {
        use x_lir::Pattern::*;
        match pattern {
            Wildcard => "_".to_string(),
            Variable(name) => self.sanitize_identifier(name),
            Literal(lit) => self
                .emit_lir_literal(lit)
                .unwrap_or_else(|_| "None".to_string()),
            Constructor(name, pats) => {
                if pats.is_empty() {
                    self.sanitize_identifier(name)
                } else {
                    let args: Vec<String> = pats.iter().map(|p| self.emit_lir_pattern(p)).collect();
                    format!("{}({})", self.sanitize_identifier(name), args.join(", "))
                }
            }
            Tuple(pats) => {
                let args: Vec<String> = pats.iter().map(|p| self.emit_lir_pattern(p)).collect();
                format!("({})", args.join(", "))
            }
            Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, self.emit_lir_pattern(v)))
                    .collect();
                format!("{}({})", self.sanitize_identifier(name), field_strs.join(", "))
            }
            Or(left, right) => {
                let l = self.emit_lir_pattern(left);
                let r = self.emit_lir_pattern(right);
                format!("{} | {}", l, r)
            }
        }
    }

    // ========================================================================
    // LIR expression emission
    // ========================================================================

    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> PythonResult<String> {
        use x_lir::Expression::*;
        match expr {
            Literal(l) => self.emit_lir_literal(l),
            Variable(n) => Ok(self.sanitize_identifier(n)),
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
            Ternary(cond, then_e, else_e) => {
                let c = self.emit_lir_expr(cond)?;
                let t = self.emit_lir_expr(then_e)?;
                let e = self.emit_lir_expr(else_e)?;
                Ok(format!("({} if {} else {})", t, c, e))
            }
            Call(callee, args) => {
                let callee_str = self.emit_lir_expr(callee)?;
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // Map X built-in functions to Python equivalents
                let func_name = match callee_str.as_str() {
                    "println" | "print" => "builtins.print",
                    "eprintln" | "eprintln!" => {
                        let args_part = args_str.join(", ");
                        if args_part.is_empty() {
                            return Ok("builtins.print(file=sys.stderr)".to_string());
                        } else {
                            return Ok(format!("builtins.print({}, file=sys.stderr)", args_part));
                        }
                    }
                    "panic" => {
                        if args_str.is_empty() {
                            return Ok("raise Exception()".to_string());
                        } else {
                            return Ok(format!("raise Exception({})", args_str.join(", ")));
                        }
                    }
                    "format" => {
                        if args_str.is_empty() {
                            return Ok("\"\"".to_string());
                        }
                        // Use f-string style or str.format
                        return Ok(format!("str.format({})", args_str.join(", ")));
                    }
                    "string" => "str",
                    "len" => "len",
                    "typeof" => "type",
                    "clone" => "copy.deepcopy",
                    other => other,
                };

                Ok(format!("{}({})", func_name, args_str.join(", ")))
            }
            Assign(target, value) => {
                let target_str = self.emit_lir_expr(target)?;
                let value_str = self.emit_lir_expr(value)?;
                Ok(format!("{} = {}", target_str, value_str))
            }
            AssignOp(op, target, value) => {
                let target_str = self.emit_lir_expr(target)?;
                let value_str = self.emit_lir_expr(value)?;
                let op_str = self.map_lir_binop(op);
                Ok(format!("{} {}= {}", target_str, op_str, value_str))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                if member == "length" {
                    return Ok(format!("len({})", obj_str));
                }
                Ok(format!("{}.{}", obj_str, member))
            }
            PointerMember(obj, member) => {
                // Python doesn't have pointer access, treat like member
                let obj_str = self.emit_lir_expr(obj)?;
                if member == "length" {
                    return Ok(format!("len({})", obj_str));
                }
                Ok(format!("{}.{}", obj_str, member))
            }
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("{}[{}]", arr_str, idx_str))
            }
            AddressOf(e) => {
                // No address-of in Python; just emit the expression
                self.emit_lir_expr(e)
            }
            Dereference(e) => {
                // No dereference in Python; just emit the expression
                self.emit_lir_expr(e)
            }
            Cast(_ty, e) => {
                // Python doesn't do runtime casts; just emit the inner expression
                self.emit_lir_expr(e)
            }
            SizeOf(_ty) => {
                // No direct equivalent; use sys.getsizeof as approximation
                Ok("0".to_string())
            }
            SizeOfExpr(e) => {
                let e_str = self.emit_lir_expr(e)?;
                Ok(format!("sys.getsizeof({})", e_str))
            }
            AlignOf(_ty) => Ok("0".to_string()),
            Comma(exprs) => {
                // Comma expressions: emit all, return last
                if exprs.is_empty() {
                    return Ok("None".to_string());
                }
                let strs: Vec<String> = exprs
                    .iter()
                    .map(|e| self.emit_lir_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                // In Python, a tuple of expressions evaluates all and returns the tuple.
                // For comma-expression semantics (return last), use the last value.
                Ok(strs.last().unwrap().clone())
            }
            Parenthesized(inner) => {
                let e = self.emit_lir_expr(inner)?;
                Ok(format!("({})", e))
            }
            InitializerList(inits) => {
                let items: Vec<String> = inits
                    .iter()
                    .filter_map(|init| match init {
                        x_lir::Initializer::Expression(e) => self.emit_lir_expr(e).ok(),
                        _ => None,
                    })
                    .collect();
                Ok(format!("[{}]", items.join(", ")))
            }
            CompoundLiteral(_ty, inits) => {
                let items: Vec<String> = inits
                    .iter()
                    .filter_map(|init| match init {
                        x_lir::Initializer::Expression(e) => self.emit_lir_expr(e).ok(),
                        _ => None,
                    })
                    .collect();
                Ok(format!("[{}]", items.join(", ")))
            }
        }
    }

    // ========================================================================
    // LIR literal emission
    // ========================================================================

    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> PythonResult<String> {
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
            Char(c) => Ok(format!("'{}'", c)),
            Bool(b) => {
                if *b {
                    Ok("True".to_string())
                } else {
                    Ok("False".to_string())
                }
            }
            NullPointer => Ok("None".to_string()),
        }
    }

    // ========================================================================
    // LIR operator mapping
    // ========================================================================

    fn map_lir_binop(&self, op: &x_lir::BinaryOp) -> String {
        use x_lir::BinaryOp::*;
        match op {
            Add => "+",
            Subtract => "-",
            Multiply => "*",
            Divide => "/",
            Modulo => "%",
            LessThan => "<",
            LessThanEqual => "<=",
            GreaterThan => ">",
            GreaterThanEqual => ">=",
            Equal => "==",
            NotEqual => "!=",
            BitAnd => "&",
            BitOr => "|",
            BitXor => "^",
            LeftShift => "<<",
            RightShift => ">>",
            RightShiftArithmetic => ">>",
            LogicalAnd => "and",
            LogicalOr => "or",
        }
        .to_string()
    }

    fn map_lir_unaryop(&self, op: &x_lir::UnaryOp) -> String {
        use x_lir::UnaryOp::*;
        match op {
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Not => "not ".to_string(),
            BitNot => "~".to_string(),
            PreIncrement => "/* ++pre */ ".to_string(),
            PreDecrement => "/* --pre */ ".to_string(),
            PostIncrement => " /* post++ */".to_string(),
            PostDecrement => " /* post-- */".to_string(),
            Reference => "".to_string(),
            MutableReference => "".to_string(),
        }
    }

    // ========================================================================
    // Declaration dispatch
    // ========================================================================

    fn emit_lir_declaration(&mut self, decl: &x_lir::Declaration) -> PythonResult<()> {
        match decl {
            x_lir::Declaration::Function(f) => {
                let params = self.sanitize_parameters(&f.parameters);
                self.line(&format!("def {}({}):", self.sanitize_identifier(&f.name), params))?;
                self.indent();

                if f.body.statements.is_empty() {
                    self.line("pass")?;
                } else {
                    for stmt in &f.body.statements {
                        self.emit_lir_statement(stmt)?;
                    }
                }

                self.dedent();
                self.line("")?;
            }
            x_lir::Declaration::Global(v) => {
                let ty = self.lir_type_to_python(&v.type_);
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("{}: {} = {}", self.sanitize_identifier(&v.name), ty, init_str))?;
                } else {
                    self.line(&format!("{}: {} = None", self.sanitize_identifier(&v.name), ty))?;
                }
            }
            x_lir::Declaration::Struct(s) => {
                self.line("@dataclass")?;
                self.line(&format!("class {}:", s.name))?;
                self.indent();
                if s.fields.is_empty() {
                    self.line("pass")?;
                } else {
                    for field in &s.fields {
                        let ty = self.lir_type_to_python(&field.type_);
                        self.line(&format!("{}: {}", field.name, ty))?;
                    }
                }
                self.dedent();
                self.line("")?;
            }
            x_lir::Declaration::Class(c) => {
                let base = if let Some(ext) = &c.extends {
                    ext.clone()
                } else if !c.implements.is_empty() {
                    c.implements.join(", ")
                } else {
                    String::new()
                };
                if base.is_empty() {
                    self.line(&format!("class {}:", c.name))?;
                } else {
                    self.line(&format!("class {}({}):", c.name, base))?;
                }
                self.indent();
                if c.fields.is_empty() {
                    self.line("pass")?;
                } else {
                    // Generate __init__ with fields
                    let field_params: Vec<String> = c
                        .fields
                        .iter()
                        .map(|f| {
                            let ty = self.lir_type_to_python(&f.type_);
                            format!("{}: {}", f.name, ty)
                        })
                        .collect();
                    self.line(&format!("def __init__(self, {}):", field_params.join(", ")))?;
                    self.indent();
                    for field in &c.fields {
                        self.line(&format!("self.{} = {}", field.name, field.name))?;
                    }
                    self.dedent();
                }
                self.dedent();
                self.line("")?;
            }
            x_lir::Declaration::VTable(vt) => {
                self.line(&format!("# VTable {} for class {}", vt.name, vt.class_name))?;
            }
            x_lir::Declaration::Enum(e) => {
                self.line(&format!("class {}(Enum):", e.name))?;
                self.indent();
                if e.variants.is_empty() {
                    self.line("pass")?;
                } else {
                    for variant in &e.variants {
                        if let Some(val) = variant.value {
                            self.line(&format!("{} = {}", variant.name, val))?;
                        } else {
                            self.line(&format!("{} = auto()", variant.name))?;
                        }
                    }
                }
                self.dedent();
                self.line("")?;
            }
            x_lir::Declaration::TypeAlias(ta) => {
                let ty = self.lir_type_to_python(&ta.type_);
                self.line(&format!("{} = {}", ta.name, ty))?;
            }
            x_lir::Declaration::Newtype(nt) => {
                let inner_ty = self.lir_type_to_python(&nt.type_);
                self.line(&format!("class {}:", nt.name))?;
                self.indent();
                self.line(&format!("def __init__(self, value: {}):", inner_ty))?;
                self.indent();
                self.line("self.value = value")?;
                self.dedent();
                self.dedent();
                self.line("")?;
            }
            x_lir::Declaration::Trait(t) => {
                if t.extends.is_empty() {
                    self.line(&format!("class {}(ABC):", t.name))?;
                } else {
                    let bases = t.extends.join(", ");
                    self.line(&format!("class {}({}, ABC):", t.name, bases))?;
                }
                self.indent();
                if t.methods.is_empty() {
                    self.line("pass")?;
                } else {
                    for method in &t.methods {
                        self.line("@abstractmethod")?;
                        let params: Vec<String> = std::iter::once("self".to_string())
                            .chain(method.parameters.iter().map(|p| p.name.clone()))
                            .collect();
                        self.line(&format!("def {}({}):", method.name, params.join(", ")))?;
                        self.indent();
                        if let Some(body) = &method.default_body {
                            for stmt in &body.statements {
                                self.emit_lir_statement(stmt)?;
                            }
                        } else {
                            self.line("...")?;
                        }
                        self.dedent();
                    }
                }
                self.dedent();
                self.line("")?;
            }
            x_lir::Declaration::Effect(eff) => {
                self.line(&format!("class {}(ABC):", eff.name))?;
                self.indent();
                if eff.operations.is_empty() {
                    self.line("pass")?;
                } else {
                    for op in &eff.operations {
                        self.line("@abstractmethod")?;
                        let params: Vec<String> = std::iter::once("self".to_string())
                            .chain(op.parameters.iter().map(|p| p.name.clone()))
                            .collect();
                        self.line(&format!("def {}({}):", op.name, params.join(", ")))?;
                        self.indent();
                        self.line("...")?;
                        self.dedent();
                    }
                }
                self.dedent();
                self.line("")?;
            }
            x_lir::Declaration::Impl(imp) => {
                let target = self.lir_type_to_python(&imp.target_type);
                self.line(&format!("# impl {} for {}", imp.trait_name, target))?;
                for method in &imp.methods {
                    let params: Vec<String> = std::iter::once("self".to_string())
                        .chain(method.parameters.iter().map(|p| p.name.clone()))
                        .collect();
                    self.line(&format!("# def {}({}):", method.name, params.join(", ")))?;
                }
            }
            x_lir::Declaration::ExternFunction(ef) => {
                self.line(&format!("# extern function {}", ef.name))?;
            }
            x_lir::Declaration::Import(imp) => {
                if imp.module_path.starts_with("std.") {
                    if imp.import_all {
                        self.line(&format!("# import {}.*", imp.module_path))?;
                    } else if imp.symbols.is_empty() {
                        self.line(&format!("# import {}", imp.module_path))?;
                    } else {
                        let syms: Vec<String> = imp
                            .symbols
                            .iter()
                            .map(|(name, alias)| {
                                if let Some(a) = alias {
                                    format!("{} as {}", name, a)
                                } else {
                                    name.clone()
                                }
                            })
                            .collect();
                        self.line(&format!("# from {} import {}", imp.module_path, syms.join(", ")))?;
                    }
                } else if imp.import_all {
                    self.line(&format!("from {} import *", imp.module_path))?;
                } else if imp.symbols.is_empty() {
                    self.line(&format!("import {}", imp.module_path))?;
                } else {
                    let syms: Vec<String> = imp
                        .symbols
                        .iter()
                        .map(|(name, alias)| {
                            if let Some(a) = alias {
                                format!("{} as {}", name, a)
                            } else {
                                name.clone()
                            }
                        })
                        .collect();
                    self.line(&format!(
                        "from {} import {}",
                        imp.module_path,
                        syms.join(", ")
                    ))?;
                }
            }
        }
        Ok(())
    }

    // ========================================================================
    // LIR -> Python code generation (public entry point)
    // ========================================================================

    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> PythonResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // Emit all declarations
        let mut has_main = false;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    has_main = true;
                }
            }
            self.emit_lir_declaration(decl)?;
        }

        // Emit main entry point
        if has_main {
            self.line("if __name__ == \"__main__\":")?;
            self.indent();
            self.line("main()")?;
            self.dedent();
        }

        let output_file = OutputFile {
            path: PathBuf::from("main.py"),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::Python,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

// ============================================================================
// CodeGenerator trait implementation
// ============================================================================

impl CodeGenerator for PythonBackend {
    type Config = PythonBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        Self::generate_from_lir(self, lir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = PythonBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
    }

    #[test]
    fn test_config_with_options() {
        let config = PythonBackendConfig {
            output_dir: Some(std::path::PathBuf::from("/tmp")),
            optimize: true,
            debug_info: false,
        };
        assert!(config.optimize);
        assert!(!config.debug_info);
        assert!(config.output_dir.is_some());
    }

    #[test]
    fn test_line_output() {
        let mut backend = PythonBackend::new(PythonBackendConfig::default());
        backend.line("test line").unwrap();
        let output = backend.output();
        assert_eq!(output, "test line\n");
    }

    #[test]
    fn test_multiple_lines() {
        let mut backend = PythonBackend::new(PythonBackendConfig::default());
        backend.line("line1").unwrap();
        backend.line("line2").unwrap();
        backend.line("line3").unwrap();
        let output = backend.output();
        assert_eq!(output, "line1\nline2\nline3\n");
    }

    #[test]
    fn test_buffer_indent_dedent() {
        let mut backend = PythonBackend::new(PythonBackendConfig::default());
        backend.line("line1").unwrap();
        backend.indent();
        backend.line("line2").unwrap();
        backend.dedent();
        backend.line("line3").unwrap();
        let code = backend.output();
        assert!(code.contains("line1"));
        assert!(code.contains("    line2")); // indented
        assert!(code.contains("line3"));
    }

    #[test]
    fn test_header_includes_imports() {
        let mut backend = PythonBackend::new(PythonBackendConfig::default());
        backend.emit_header().unwrap();
        let code = backend.output();
        assert!(code.contains("from __future__ import annotations"));
        assert!(code.contains("import sys"));
    }

    #[test]
    fn test_lir_literal_emission() {
        let backend = PythonBackend::new(PythonBackendConfig::default());

        assert_eq!(
            backend
                .emit_lir_literal(&x_lir::Literal::Integer(42))
                .unwrap(),
            "42"
        );
        assert_eq!(
            backend
                .emit_lir_literal(&x_lir::Literal::Float(3.14))
                .unwrap(),
            "3.14"
        );
        assert_eq!(
            backend
                .emit_lir_literal(&x_lir::Literal::Bool(true))
                .unwrap(),
            "True"
        );
        assert_eq!(
            backend
                .emit_lir_literal(&x_lir::Literal::Bool(false))
                .unwrap(),
            "False"
        );
        assert_eq!(
            backend
                .emit_lir_literal(&x_lir::Literal::String("hello".to_string()))
                .unwrap(),
            "\"hello\""
        );
        assert_eq!(
            backend
                .emit_lir_literal(&x_lir::Literal::NullPointer)
                .unwrap(),
            "None"
        );
    }

    #[test]
    fn test_lir_binop_mapping() {
        let backend = PythonBackend::new(PythonBackendConfig::default());

        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::Add), "+");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::Subtract), "-");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::Multiply), "*");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::LogicalAnd), "and");
        assert_eq!(backend.map_lir_binop(&x_lir::BinaryOp::LogicalOr), "or");
    }

    #[test]
    fn test_lir_unaryop_mapping() {
        let backend = PythonBackend::new(PythonBackendConfig::default());

        assert_eq!(backend.map_lir_unaryop(&x_lir::UnaryOp::Minus), "-");
        assert_eq!(backend.map_lir_unaryop(&x_lir::UnaryOp::Not), "not ");
        assert_eq!(backend.map_lir_unaryop(&x_lir::UnaryOp::BitNot), "~");
    }

    #[test]
    fn test_lir_type_mapping() {
        let backend = PythonBackend::new(PythonBackendConfig::default());

        assert_eq!(backend.lir_type_to_python(&x_lir::Type::Int), "int");
        assert_eq!(backend.lir_type_to_python(&x_lir::Type::Float), "float");
        assert_eq!(backend.lir_type_to_python(&x_lir::Type::Bool), "bool");
        assert_eq!(backend.lir_type_to_python(&x_lir::Type::Char), "str");
        assert_eq!(backend.lir_type_to_python(&x_lir::Type::Void), "None");
    }

    #[test]
    fn test_sanitize_python_reserved_identifier() {
        let backend = PythonBackend::new(PythonBackendConfig::default());

        assert_eq!(backend.sanitize_identifier("assert"), "assert_x");
        assert_eq!(backend.sanitize_identifier("main"), "main");
    }
}
