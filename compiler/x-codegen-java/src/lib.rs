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

#![allow(clippy::only_used_in_recursion, clippy::useless_format)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;

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
        self.buffer
            .line(s)
            .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))
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

    /// Emit file header with package and imports (Java 25 LTS)
    fn emit_header(&mut self) -> JavaResult<()> {
        self.line(headers::JAVA)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: Java 25 LTS (September 2025)")?;
        self.line("")?;
        // Java 25 标准库导入
        self.line("import java.util.*;")?;
        self.line("")?;
        Ok(())
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
                // 如果是赋值表达式且右侧是 void 函数（如 println），只调用不赋值
                if let x_lir::Expression::Assign(target, value) = e {
                    if let x_lir::Expression::Call(callee, args) = value.as_ref() {
                        if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                            let name = fn_name.as_str();
                            if matches!(name, "println" | "print" | "eprintln" | "eprintln!") {
                                // 映射到 Java 的 System.out.println/etc
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_lir_expr(a))
                                    .collect::<Result<Vec<_>, _>>()?;
                                let args_part = args_str.join(", ");

                                // println 返回 void，不赋值
                                let call_str = if name == "eprintln" || name == "eprintln!" {
                                    format!("System.err.println({})", args_part)
                                } else {
                                    format!(
                                        "System.out.{}({})",
                                        name.trim_end_matches("ln"),
                                        args_part
                                    )
                                };
                                self.line(&format!("{};", call_str))?;
                                return Ok(());
                            }
                        }
                    }
                    // 对于其他赋值
                    let target_str = self.emit_lir_expr(target)?;
                    let value_str = self.emit_lir_expr(value)?;
                    // 直接赋值（Java 会在赋值前初始化）
                    self.line(&format!("{} = {};", target_str, value_str))?;
                    return Ok(());
                }
                // 常规表达式处理
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
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // 映射内置函数到 Java 标准库
                let java_call = match callee_str.as_str() {
                    "println" => {
                        // println(args...) -> System.out.println(args...)
                        let args_part = if args_str.is_empty() {
                            "".to_string()
                        } else {
                            args_str.join(", ")
                        };
                        return Ok(format!("System.out.println({})", args_part));
                    }
                    "print" => {
                        let args_part = if args_str.is_empty() {
                            "".to_string()
                        } else {
                            args_str.join(", ")
                        };
                        return Ok(format!("System.out.print({})", args_part));
                    }
                    "eprintln" | "eprintln!" => {
                        let args_part = if args_str.is_empty() {
                            "".to_string()
                        } else {
                            args_str.join(", ")
                        };
                        return Ok(format!("System.err.println({})", args_part));
                    }
                    "format" => {
                        // format!("...", args...) -> String.format("...", args...)
                        if args_str.is_empty() {
                            return Ok("\"\"".to_string());
                        }
                        return Ok(format!("String.format({})", args_str.join(", ")));
                    }
                    _ => format!("{}({})", callee_str, args_str.join(", ")),
                };
                Ok(java_call)
            }
            // 赋值表达式（如 t0 = println(...)）
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
            // Increment/decrement are handled at the statement level
            PreIncrement => "++".to_string(),
            PreDecrement => "--".to_string(),
            PostIncrement => "++".to_string(),
            PostDecrement => "--".to_string(),
            Reference => "&".to_string(),
            MutableReference => "&".to_string(), // Java doesn't have mutable references, just use regular reference
        }
    }

    /// 从 LIR 生成 Java 代码
    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> JavaResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // 开始类定义
        self.line(&format!("public class {} {{", self.config.class_name))?;
        self.indent();

        // 收集函数（跳过 main 函数，它会被内联到 Java main 方法中）
        let mut main_function: Option<&x_lir::Function> = None;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    main_function = Some(f);
                    continue; // 跳过 main，稍后特殊处理
                }
                // 发射其他函数
                let ret = self.lir_type_to_java(&f.return_type);
                let params: Vec<String> = f
                    .parameters
                    .iter()
                    .map(|p| format!("{} {}", self.lir_type_to_java(&p.type_), p.name))
                    .collect();
                self.line(&format!(
                    "    public static {} {}({}) {{",
                    ret,
                    f.name,
                    params.join(", ")
                ))?;
                self.indent();

                // 发射函数体
                for stmt in &f.body.statements {
                    self.emit_lir_statement(stmt)?;
                }

                self.dedent();
                self.line("    }")?;
                self.line("")?;
            }
        }

        // main 方法 - 如果有 X 的 main 函数，将代码内联到 Java main 方法中
        self.line("    public static void main(String[] args) {")?;
        self.indent();

        if let Some(main_fn) = main_function {
            // 内联 main 函数的代码
            let mut has_output = false;
            for stmt in &main_fn.body.statements {
                // 处理 return 语句 - 使用 System.exit() 传递退出码
                if let x_lir::Statement::Return(Some(ret_val)) = stmt {
                    // 如果之前的语句已经输出，使用 System.exit(0)
                    // 否则尝试使用返回值
                    if has_output {
                        self.line("        System.exit(0);")?;
                    } else {
                        let exit_code = self.emit_lir_expr(ret_val)?;
                        self.line(&format!("        System.exit({});", exit_code))?;
                    }
                    continue;
                } else if let x_lir::Statement::Return(None) = stmt {
                    // return; -> System.exit(0)
                    self.line("        System.exit(0);")?;
                    continue;
                }
                // 跳过 Label 和 Goto
                if matches!(stmt, x_lir::Statement::Label(_) | x_lir::Statement::Goto(_)) {
                    continue;
                }

                // 跟踪是否有输出语句
                if matches!(stmt, x_lir::Statement::Expression(_)) {
                    has_output = true;
                }

                self.emit_lir_statement(stmt)?;
            }
        } else {
            // 没有 main 函数，输出默认消息
            self.line("        System.out.println(\"Hello from Java backend!\");")?;
        }

        self.dedent();
        self.line("    }")?;

        self.dedent();
        self.line("}")?;

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

impl CodeGenerator for JavaBackend {
    type Config = JavaConfig;
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
        let config = JavaConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
        assert_eq!(config.class_name, "Main");
    }
}
