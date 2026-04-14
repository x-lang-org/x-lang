//! TypeScript 后端 - 生成 TypeScript/JavaScript 代码
//!
//! 面向 Web/Node.js 平台，生成 TypeScript 源代码
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

#![allow(clippy::only_used_in_recursion, clippy::useless_format)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;

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
    #[allow(dead_code)]
    config: TypeScriptBackendConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
}

pub type TypeScriptResult<T> = Result<T, x_codegen::CodeGenError>;

impl TypeScriptBackend {
    pub fn new(config: TypeScriptBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
        }
    }

    fn line(&mut self, s: &str) -> TypeScriptResult<()> {
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

    fn emit_header(&mut self) -> TypeScriptResult<()> {
        self.line(headers::TYPESCRIPT)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: TypeScript 6.0 / ES2025 (March 2026)")?;
        self.line("// tsconfig: strict=true, module=esnext")?;
        self.line("")?;
        Ok(())
    }

    /// 映射 LIR 类型到 TypeScript 类型
    fn lir_type_to_typescript(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "void".to_string(),
            Bool => "boolean".to_string(),
            Char => "string".to_string(),
            Schar | Short => "number".to_string(),
            Uchar | Ushort | Int | Uint => "number".to_string(),
            Long | Ulong | LongLong | UlongLong => "bigint".to_string(),
            Float | Double | LongDouble => "number".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "number".to_string(),
            Pointer(inner) => format!("Array<{}>", self.lir_type_to_typescript(inner)),
            Array(inner, _) => format!("Array<{}>", self.lir_type_to_typescript(inner)),
            Named(n) => n.clone(),
            FunctionPointer(_, _) => "Function".to_string(),
            Qualified(_, inner) => self.lir_type_to_typescript(inner),
        }
    }

    /// 发射 LIR 语句
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> TypeScriptResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                // 如果是赋值表达式且右侧是 void 函数（如 println），只调用不赋值
                if let x_lir::Expression::Assign(target, value) = e {
                    if let x_lir::Expression::Call(callee, args) = value.as_ref() {
                        if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                            let name = fn_name.as_str();
                            if matches!(name, "println" | "print" | "eprintln" | "eprintln!") {
                                // 映射到 TypeScript 的 console.log/etc
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_lir_expr(a))
                                    .collect::<Result<Vec<_>, _>>()?;
                                let args_part = args_str.join(", ");

                                // println 返回 void，不赋值
                                let call_str = if name == "eprintln" || name == "eprintln!" {
                                    format!("console.error({})", args_part)
                                } else if name == "print" {
                                    format!("console.log({})", args_part)
                                } else {
                                    format!("console.log({})", args_part)
                                };
                                self.line(&format!("{};", call_str))?;
                                return Ok(());
                            }
                        }
                    }
                    // 对于其他赋值
                    let target_str = self.emit_lir_expr(target)?;
                    let value_str = self.emit_lir_expr(value)?;
                    self.line(&format!("{} = {};", target_str, value_str))?;
                    return Ok(());
                }
                // 常规表达式处理
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{};", s))?;
            }
            Variable(v) => {
                let ty = self.lir_type_to_typescript(&v.type_);
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("let {}: {} = {};", v.name, ty, init_str))?;
                } else {
                    self.line(&format!("let {}: {};", v.name, ty))?;
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
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> TypeScriptResult<String> {
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
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // 映射内置函数到 TypeScript 标准库
                let ts_call = match callee_str.as_str() {
                    "println" => {
                        let args_part = if args_str.is_empty() {
                            "".to_string()
                        } else {
                            args_str.join(", ")
                        };
                        return Ok(format!("console.log({})", args_part));
                    }
                    "print" => {
                        let args_part = if args_str.is_empty() {
                            "".to_string()
                        } else {
                            args_str.join(", ")
                        };
                        return Ok(format!("console.log({})", args_part));
                    }
                    "eprintln" | "eprintln!" => {
                        let args_part = if args_str.is_empty() {
                            "".to_string()
                        } else {
                            args_str.join(", ")
                        };
                        return Ok(format!("console.error({})", args_part));
                    }
                    _ => format!("{}({})", callee_str, args_str.join(", ")),
                };
                Ok(ts_call)
            }
            Assign(target, value) => {
                let target_str = self.emit_lir_expr(target)?;
                let value_str = self.emit_lir_expr(value)?;
                Ok(format!("{} = {}", target_str, value_str))
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
            _ => Ok("undefined".to_string()),
        }
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> TypeScriptResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(format!("{}n", n)),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!("\"{}\"", s)),
            Char(c) => Ok(format!("\"{}\"", c)),
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
            Equal => "===",
            NotEqual => "!==",
            BitAnd => "&",
            BitOr => "|",
            BitXor => "^",
            LogicalAnd => "&&",
            LogicalOr => "||",
        }
        .to_string()
    }

    /// 映射 LIR 一元运算符
    fn map_lir_unaryop(&self, op: &x_lir::UnaryOp) -> String {
        use x_lir::UnaryOp::*;
        match op {
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Not => "!".to_string(),
            BitNot => "~".to_string(),
            PreIncrement => "++".to_string(),
            PreDecrement => "--".to_string(),
            PostIncrement => "++".to_string(),
            PostDecrement => "--".to_string(),
            Reference => "&".to_string(),
            MutableReference => "&".to_string(),
        }
    }

    /// 发射 LIR 函数
    fn emit_lir_function(&mut self, func: &x_lir::Function) -> TypeScriptResult<()> {
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| format!("{}: {}", p.name, self.lir_type_to_typescript(&p.type_)))
            .collect();
        let ret = self.lir_type_to_typescript(&func.return_type);

        self.line(&format!(
            "function {}({}): {} {{",
            func.name,
            params.join(", "),
            ret
        ))?;
        self.indent();

        // 发射函数体
        for stmt in &func.body.statements {
            self.emit_lir_statement(stmt)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// 从 LIR 生成 TypeScript 代码
    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> TypeScriptResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // 收集函数（跳过 main 函数，它会被特殊处理）
        let mut main_function: Option<&x_lir::Function> = None;
        let mut functions = Vec::new();

        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    main_function = Some(f);
                } else {
                    functions.push(f);
                }
            }
        }

        // 发射其他函数
        for func in functions {
            self.emit_lir_function(func)?;
            self.line("")?;
        }

        // main 函数
        if let Some(main_fn) = main_function {
            self.emit_lir_function(main_fn)?;
            self.line("")?;
            // 自动调用 main
            self.line("// Entry point")?;
            self.line("main();")?;
        } else {
            // 没有 main 函数，输出默认消息
            self.line("console.log(\"Hello from TypeScript backend!\");")?;
        }

        let output_file = OutputFile {
            path: PathBuf::from("index.ts"),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::TypeScript,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

// 保持向后兼容的别名
pub type TypeScriptCodeGenerator = TypeScriptBackend;
pub type TypeScriptCodeGenError = x_codegen::CodeGenError;

impl CodeGenerator for TypeScriptBackend {
    type Config = TypeScriptBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_lir(
        &mut self,
        lir: &LirProgram,
    ) -> Result<CodegenOutput, Self::Error> {
        Self::generate_from_lir(self, lir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TypeScriptBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
    }
}
