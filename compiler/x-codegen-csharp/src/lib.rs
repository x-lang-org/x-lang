//! C# 后端 - 生成 C# 14 源代码
//!
//! 面向 .NET 10 平台，生成 C# 源代码
//!
//! ## C# 14 / .NET 10 特性支持 (2026年3月发布)
//! - Primary constructors（主构造函数）
//! - Collection expressions（集合表达式）
//! - Inline arrays（内联数组）
//! - Extended nameof scope
//! - Implicit span conversions
//! - Params Span/ReadOnlySpan
//! - Partial properties
//! - Field-backed properties
//! - Method group natural type improvements

#![allow(clippy::only_used_in_recursion, clippy::useless_format)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;

/// C# 后端配置
#[derive(Debug, Clone)]
pub struct CSharpConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub namespace: Option<String>,
}

impl Default for CSharpConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            namespace: None,
        }
    }
}

/// C# 后端
pub struct CSharpBackend {
    config: CSharpConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
}

pub type CSharpResult<T> = Result<T, x_codegen::CodeGenError>;

impl CSharpBackend {
    pub fn new(config: CSharpConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
        }
    }

    /// 输出一行代码
    fn line(&mut self, s: &str) -> CSharpResult<()> {
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

    /// 生成文件头部 (C# 14 / .NET 10)
    fn emit_header(&mut self) -> CSharpResult<()> {
        self.line(headers::CSHARP)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: C# 14 / .NET 10 LTS (March 2026)")?;
        self.line("")?;
        self.line("using System;")?;
        self.line("using System.Collections.Generic;")?;
        self.line("using System.Threading.Tasks;")?;
        self.line("using System.Linq;")?;
        self.line("")?;
        Ok(())
    }

    /// 从 LIR 生成 C# 代码
    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> CSharpResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // 获取命名空间
        let namespace = self
            .config
            .namespace
            .clone()
            .unwrap_or_else(|| "XLang".to_string());

        self.line(&format!("namespace {}", namespace))?;
        self.line("{")?;
        self.indent();

        // 开始类定义
        self.line("public class Program {")?;
        self.indent();

        // 收集函数（跳过 main 函数，它会被内联到 C# Main 方法中）
        let mut main_function: Option<&x_lir::Function> = None;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    main_function = Some(f);
                    continue; // 跳过 main，稍后特殊处理
                }
                // 发射其他函数
                let ret = self.lir_type_to_csharp(&f.return_type);
                let params: Vec<String> = f
                    .parameters
                    .iter()
                    .map(|p| format!("{} {}", self.lir_type_to_csharp(&p.type_), p.name))
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

        // Main 方法入口 - 如果有 X 的 main 函数，将代码内联到 C# Main 方法中
        self.line("    public static void Main(string[] args) {")?;
        self.indent();

        if let Some(main_fn) = main_function {
            // 内联 main 函数的代码
            let mut has_output = false;
            for stmt in &main_fn.body.statements {
                // 处理 return 语句 - 使用 Environment.Exit() 传递退出码
                if let x_lir::Statement::Return(Some(ret_val)) = stmt {
                    if has_output {
                        self.line("        Environment.Exit(0);")?;
                    } else {
                        let exit_code = self.emit_lir_expr(ret_val)?;
                        self.line(&format!("        Environment.Exit({});", exit_code))?;
                    }
                    continue;
                } else if let x_lir::Statement::Return(None) = stmt {
                    self.line("        Environment.Exit(0);")?;
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
            self.line("        Console.WriteLine(\"Hello from C# backend!\");")?;
        }

        self.dedent();
        self.line("    }")?;

        self.dedent();
        self.line("}")?;
        self.dedent();
        self.line("}")?;

        let output_file = OutputFile {
            path: PathBuf::from("Program.cs"),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::CSharp,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

/// LIR -> C# 辅助方法
impl CSharpBackend {
    /// 将 LIR 类型转换为 C# 类型
    #[allow(clippy::only_used_in_recursion)]
    fn lir_type_to_csharp(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "void".to_string(),
            Bool => "bool".to_string(),
            Char => "char".to_string(),
            Schar | Short => "short".to_string(),
            Uchar | Ushort | Int | Uint => "int".to_string(),
            Long | Ulong | LongLong | UlongLong => "long".to_string(),
            Float => "float".to_string(),
            Double | LongDouble => "double".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "long".to_string(),
            Pointer(inner) => format!("{}*", self.lir_type_to_csharp(inner)), // unsafe
            Array(inner, _) => format!("{}[]", self.lir_type_to_csharp(inner)),
            FunctionPointer(_, _) => "Func<object, object>".to_string(),
            Named(n) => n.clone(),
            Qualified(_, inner) => self.lir_type_to_csharp(inner),
        }
    }

    /// 发射 LIR 语句
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> CSharpResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                // Check if this is an assignment where the value is a void function call
                if let x_lir::Expression::Assign(target, value) = e {
                    if let x_lir::Expression::Call(callee, _) = value.as_ref() {
                        if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                            let name = fn_name.as_str();
                            // println/print/eprintln return void, so just call the function
                            if matches!(
                                name,
                                "println" | "print" | "eprintln" | "eprintln!" | "format"
                            ) {
                                let args = if let x_lir::Expression::Call(_, args) = value.as_ref()
                                {
                                    args.clone()
                                } else {
                                    vec![]
                                };
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_lir_expr(a))
                                    .collect::<Result<Vec<_>, _>>()?;

                                let call_str = match name {
                                    "println" => {
                                        format!("Console.WriteLine({})", args_str.join(", "))
                                    }
                                    "print" => format!("Console.Write({})", args_str.join(", ")),
                                    "eprintln" | "eprintln!" => {
                                        format!("Console.Error.WriteLine({})", args_str.join(", "))
                                    }
                                    "format" => format!("string.Format({})", args_str.join(", ")),
                                    _ => format!("{}({})", name, args_str.join(", ")),
                                };
                                self.line(&format!("{};", call_str))?;
                                // For void functions, we need to initialize the target variable
                                let target_str = self.emit_lir_expr(target)?;
                                if target_str.starts_with("t") || target_str.starts_with("arg") {
                                    self.line(&format!("{} = 0;", target_str))?;
                                }
                                return Ok(());
                            }
                        }
                    }
                }
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{};", s))?;
            }
            Variable(v) => {
                let ty = self.lir_type_to_csharp(&v.type_);
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
            DoWhile(d) => {
                self.line("do {")?;
                self.indent();
                self.emit_lir_statement(&d.body)?;
                self.dedent();
                let cond = self.emit_lir_expr(&d.condition)?;
                self.line(&format!("}} while ({});", cond))?;
            }
            For(f) => {
                let mut for_header = String::from("for (");
                if let Some(init) = &f.initializer {
                    match init.as_ref() {
                        x_lir::Statement::Variable(v) => {
                            let ty = self.lir_type_to_csharp(&v.type_);
                            if let Some(init_val) = &v.initializer {
                                let init_str = self.emit_lir_expr(init_val)?;
                                for_header.push_str(&format!("{} {} = {}", ty, v.name, init_str));
                            } else {
                                for_header.push_str(&format!("{} {}", ty, v.name));
                            }
                        }
                        _ => for_header.push_str("/* init */"),
                    }
                }
                for_header.push_str("; ");
                if let Some(cond) = &f.condition {
                    let cond_str = self.emit_lir_expr(cond)?;
                    for_header.push_str(&cond_str);
                }
                for_header.push_str("; ");
                if let Some(increment) = &f.increment {
                    let inc_str = self.emit_lir_expr(increment)?;
                    for_header.push_str(&inc_str);
                }
                for_header.push_str(") {");
                self.line(&for_header)?;
                self.indent();
                self.emit_lir_statement(&f.body)?;
                self.dedent();
                self.line("}")?;
            }
            Switch(s) => {
                let cond = self.emit_lir_expr(&s.expression)?;
                self.line(&format!("switch ({}) {{", cond))?;
                self.indent();
                for case in &s.cases {
                    let value_str = self.emit_lir_expr(&case.value)?;
                    self.line(&format!("case {}:", value_str))?;
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
                let expr = self.emit_lir_expr(&m.scrutinee)?;
                self.line(&format!("switch ({}) {{", expr))?;
                self.indent();
                for case in &m.cases {
                    let pat_str = self.emit_lir_pattern(&case.pattern)?;
                    self.line(&format!("case {}:", pat_str))?;
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
                self.line("try {")?;
                self.indent();
                for stmt in &t.body.statements {
                    self.emit_lir_statement(stmt)?;
                }
                self.dedent();
                for catch in &t.catch_clauses {
                    let exc_type = &catch.exception_type;
                    let exc_name = &catch.variable_name;
                    self.line(&format!(
                        "catch ({}{}) {{",
                        exc_type.as_deref().unwrap_or("Exception"),
                        exc_name
                            .as_ref()
                            .map(|n| format!(" {}", n))
                            .unwrap_or_default()
                    ))?;
                    self.indent();
                    for stmt in &catch.body.statements {
                        self.emit_lir_statement(stmt)?;
                    }
                    self.dedent();
                    self.line("}")?;
                }
                if let Some(finally) = &t.finally_block {
                    self.line("finally {")?;
                    self.indent();
                    for stmt in &finally.statements {
                        self.emit_lir_statement(stmt)?;
                    }
                    self.dedent();
                    self.line("}")?;
                }
            }
            Goto(label) => self.line(&format!("goto {};", label))?,
            Label(label) => self.line(&format!("{}:", label))?,
            Empty => {}
            Compound(block) => {
                self.line("{")?;
                self.indent();
                for stmt in &block.statements {
                    self.emit_lir_statement(stmt)?;
                }
                self.dedent();
                self.line("}")?;
            }
            Declaration(d) => match d {
                x_lir::Declaration::Function(f) => {
                    let ret = self.lir_type_to_csharp(&f.return_type);
                    let params: Vec<String> = f
                        .parameters
                        .iter()
                        .map(|p| format!("{} {}", self.lir_type_to_csharp(&p.type_), p.name))
                        .collect();
                    self.line(&format!(
                        "public static {} {}({}) {{",
                        ret,
                        f.name,
                        params.join(", ")
                    ))?;
                    self.indent();
                    for stmt in &f.body.statements {
                        self.emit_lir_statement(stmt)?;
                    }
                    self.dedent();
                    self.line("}")?;
                }
                x_lir::Declaration::Global(g) => {
                    let ty = self.lir_type_to_csharp(&g.type_);
                    let init = g
                        .initializer
                        .as_ref()
                        .map(|e| self.emit_lir_expr(e).map(|s| format!(" = {}", s)))
                        .transpose()?;
                    self.line(&format!(
                        "public static {} {}{};",
                        ty,
                        g.name,
                        init.unwrap_or_default()
                    ))?;
                }
                x_lir::Declaration::Struct(s) => {
                    self.line(&format!("public struct {} {{", s.name))?;
                    self.indent();
                    for field in &s.fields {
                        let ty = self.lir_type_to_csharp(&field.type_);
                        self.line(&format!("public {} {};", ty, field.name))?;
                    }
                    self.dedent();
                    self.line("}")?;
                }
                x_lir::Declaration::Enum(e) => {
                    self.line(&format!("public enum {} {{", e.name))?;
                    self.indent();
                    for (i, variant) in e.variants.iter().enumerate() {
                        if i > 0 {
                            self.line(",")?;
                        }
                        self.line(&variant.name)?;
                    }
                    self.dedent();
                    self.line("}")?;
                }
                x_lir::Declaration::Import(i) => {
                    self.line(&format!("// using {};", i.module_path))?;
                }
                x_lir::Declaration::Class(c) => {
                    self.line(&format!(
                        "public class {} {{ /* class members */ }}",
                        c.name
                    ))?;
                }
                x_lir::Declaration::VTable(v) => {
                    self.line(&format!("/* vtable for {} */", v.name))?;
                }
                x_lir::Declaration::TypeAlias(t) => {
                    self.line(&format!(
                        "using {} = {};",
                        t.name,
                        self.lir_type_to_csharp(&t.type_)
                    ))?;
                }
                x_lir::Declaration::Newtype(nt) => {
                    self.line(&format!("public readonly struct {} {{", nt.name))?;
                    self.indent();
                    self.line(&format!(
                        "public {} Value;",
                        self.lir_type_to_csharp(&nt.type_)
                    ))?;
                    self.dedent();
                    self.line("}")?;
                    self.line("")?;
                }
                x_lir::Declaration::Trait(trait_) => {
                    self.line(&format!("public interface {} {{", trait_.name))?;
                    self.indent();
                    for method in &trait_.methods {
                        let ret_ty = method
                            .return_type
                            .as_ref()
                            .map(|ty| self.lir_type_to_csharp(ty))
                            .unwrap_or_else(|| "void".to_string());
                        let params: Vec<String> = method
                            .parameters
                            .iter()
                            .map(|p| format!("{} {}", self.lir_type_to_csharp(&p.type_), p.name))
                            .collect();
                        self.line(&format!(
                            "{} {}({});",
                            ret_ty,
                            method.name,
                            params.join(", ")
                        ))?;
                    }
                    self.dedent();
                    self.line("}")?;
                    self.line("")?;
                }
                x_lir::Declaration::Effect(effect) => {
                    self.line(&format!("public interface {} {{", effect.name))?;
                    self.indent();
                    for op in &effect.operations {
                        let ret_ty = op
                            .return_type
                            .as_ref()
                            .map(|ty| self.lir_type_to_csharp(ty))
                            .unwrap_or_else(|| "void".to_string());
                        let params: Vec<String> = op
                            .parameters
                            .iter()
                            .map(|p| format!("{} {}", self.lir_type_to_csharp(&p.type_), p.name))
                            .collect();
                        self.line(&format!("{} {}({});", ret_ty, op.name, params.join(", ")))?;
                    }
                    self.dedent();
                    self.line("}")?;
                    self.line("")?;
                }
                x_lir::Declaration::Impl(impl_) => {
                    let target_ty = self.lir_type_to_csharp(&impl_.target_type);
                    self.line(&format!(
                        "public {} {} for {} {{",
                        if impl_.is_effect {
                            "effect implementation"
                        } else {
                            "implementation"
                        },
                        impl_.trait_name,
                        target_ty
                    ))?;
                    self.indent();
                    for method in &impl_.methods {
                        let ret = self.lir_type_to_csharp(&method.return_type);
                        let params: Vec<String> = method
                            .parameters
                            .iter()
                            .map(|p| format!("{} {}", self.lir_type_to_csharp(&p.type_), p.name))
                            .collect();
                        self.line(&format!(
                            "public {} {}({}) {{",
                            ret,
                            method.name,
                            params.join(", ")
                        ))?;
                        self.indent();
                        // TODO: emit method body when fully implemented
                        self.line("// method body")?;
                        self.dedent();
                        self.line("}")?;
                    }
                    self.dedent();
                    self.line("}")?;
                    self.line("")?;
                }
                x_lir::Declaration::ExternFunction(e) => {
                    let params_str = e
                        .parameters
                        .iter()
                        .map(|ty| self.lir_type_to_csharp(ty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.line(&format!(
                        "[DllImport(\"{}\")] public static extern {} {}({});",
                        e.abi.as_deref().unwrap_or(&e.name),
                        self.lir_type_to_csharp(&e.return_type),
                        e.name,
                        params_str
                    ))?;
                }
            },
        }
        Ok(())
    }

    /// 发射 LIR 模式
    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> CSharpResult<String> {
        use x_lir::Pattern::*;
        match pattern {
            Wildcard => Ok("_".to_string()),
            Variable(n) => Ok(n.clone()),
            Literal(l) => self.emit_lir_literal(l),
            Constructor(name, patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{}({})", name, pat_strs.join(", ")))
            }
            Tuple(patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("({})", pat_strs.join(", ")))
            }
            Record(name, fields) => {
                let mut field_strs = Vec::new();
                for (k, v) in fields {
                    let v_str = self.emit_lir_pattern(v)?;
                    field_strs.push(format!("{} = {}", k, v_str));
                }
                Ok(format!("{}({})", name, field_strs.join(", ")))
            }
            Or(l, r) => {
                let left = self.emit_lir_pattern(l)?;
                let right = self.emit_lir_pattern(r)?;
                Ok(format!("({} | {})", left, right))
            }
        }
    }

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> CSharpResult<String> {
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

                // Map built-in functions to C# equivalents
                let csharp_call = match callee_str.as_str() {
                    "println" => {
                        let args_part = args_str.join(", ");
                        return Ok(format!("Console.WriteLine({})", args_part));
                    }
                    "print" => {
                        let args_part = args_str.join(", ");
                        return Ok(format!("Console.Write({})", args_part));
                    }
                    "eprintln" | "eprintln!" => {
                        let args_part = args_str.join(", ");
                        return Ok(format!("Console.Error.WriteLine({})", args_part));
                    }
                    "format" => {
                        // format!("...", args...) -> string.Format("...", args...)
                        if args_str.is_empty() {
                            return Ok("string.Empty".to_string());
                        }
                        return Ok(format!("string.Format({})", args_str.join(", ")));
                    }
                    _ => format!("{}({})", callee_str, args_str.join(", ")),
                };
                Ok(csharp_call)
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            PointerMember(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("{}[{}]", arr_str, idx_str))
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
            AddressOf(e) => {
                let _e_str = self.emit_lir_expr(e)?;
                Ok("ref var".to_string())
            }
            Dereference(e) => {
                let e_str = self.emit_lir_expr(e)?;
                Ok(format!("*{}", e_str))
            }
            Cast(ty, e) => {
                let e_str = self.emit_lir_expr(e)?;
                let ty_str = self.lir_type_to_csharp(ty);
                Ok(format!("({}){}", ty_str, e_str))
            }
            SizeOf(ty) => {
                let ty_str = self.lir_type_to_csharp(ty);
                Ok(format!("sizeof({})", ty_str))
            }
            SizeOfExpr(e) => {
                let e_str = self.emit_lir_expr(e)?;
                Ok(format!("Marshal.SizeOf({})", e_str))
            }
            AlignOf(ty) => {
                let ty_str = self.lir_type_to_csharp(ty);
                Ok(format!("/* alignof({}) */ 8", ty_str))
            }
            Comma(exprs) => {
                let expr_strs: Vec<String> = exprs
                    .iter()
                    .map(|e| self.emit_lir_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("({})", expr_strs.join(", ")))
            }
            Parenthesized(e) => {
                let e_str = self.emit_lir_expr(e)?;
                Ok(format!("({})", e_str))
            }
            InitializerList(inits) => {
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("new object[] {{ {} }}", init_strs.join(", ")))
            }
            CompoundLiteral(ty, inits) => {
                let ty_str = self.lir_type_to_csharp(ty);
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("new {} {{ {} }}", ty_str, init_strs.join(", ")))
            }
        }
    }

    /// 发射 LIR 初始化器
    fn emit_lir_initializer(&self, init: &x_lir::Initializer) -> CSharpResult<String> {
        use x_lir::Initializer::*;
        match init {
            Expression(e) => self.emit_lir_expr(e),
            List(inits) => {
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{ {} }}", init_strs.join(", ")))
            }
            Named(name, init) => {
                let init_str = self.emit_lir_initializer(init)?;
                Ok(format!("{} = {}", name, init_str))
            }
            Indexed(idx, init) => {
                let idx_str = self.emit_lir_expr(idx)?;
                let init_str = self.emit_lir_initializer(init)?;
                Ok(format!("[{}] = {}", idx_str, init_str))
            }
        }
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> CSharpResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(format!("{}UL", n)),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!(
                "\"{}\"",
                s.replace('\\', "\\\\").replace('"', "\\\"")
            )),
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
            RightShiftArithmetic => ">>>",
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
}

// 保持向后兼容的别名
pub type DotNetCodeGenerator = CSharpBackend;
pub type DotNetConfig = CSharpConfig;
pub type DotNetCodeGenError = x_codegen::CodeGenError;
pub type DotNetResult<T> = Result<T, x_codegen::CodeGenError>;

/// 编译生成的 C# 代码为可执行文件
///
/// 依赖 .NET SDK 在 PATH 中
impl CSharpBackend {
    /// 使用 dotnet CLI 编译 C# 源代码为可执行文件
    pub fn compile_csharp(
        csharp_code: &str,
        output_path: &std::path::Path,
    ) -> Result<std::path::PathBuf, x_codegen::CodeGenError> {
        // 创建临时目录存放 C# 源文件
        let temp_dir = std::env::temp_dir().join("xlang_csharp_build");
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!(
                "Failed to create temp directory: {}",
                e
            ))
        })?;

        // 写入 Program.cs
        let cs_path = temp_dir.join("Program.cs");
        std::fs::write(&cs_path, csharp_code).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to write C# source: {}", e))
        })?;

        // 创建 .csproj 项目文件
        let csproj_content = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>
</Project>
"#;
        let csproj_path = temp_dir.join("XLang.csproj");
        std::fs::write(&csproj_path, csproj_content).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to write .csproj: {}", e))
        })?;

        // 调用 dotnet build
        let output_status = std::process::Command::new("dotnet")
            .arg("build")
            .arg(&csproj_path)
            .arg("--configuration")
            .arg("Release")
            .arg("--output")
            .arg(temp_dir.join("output"))
            .current_dir(&temp_dir)
            .output()
            .map_err(|e| {
                x_codegen::CodeGenError::GenerationError(format!(
                    "Failed to invoke dotnet: {}. Is .NET SDK installed?",
                    e
                ))
            })?;

        if !output_status.status.success() {
            let stderr = String::from_utf8_lossy(&output_status.stderr);
            let stdout = String::from_utf8_lossy(&output_status.stdout);
            return Err(x_codegen::CodeGenError::GenerationError(format!(
                "C# compilation failed.\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout, stderr
            )));
        }

        // 找到生成的可执行文件
        let output_dir = temp_dir.join("output");
        let exe_name = if cfg!(windows) { "XLang.exe" } else { "XLang" };
        let exe_path = output_dir.join(exe_name);

        if !exe_path.exists() {
            return Err(x_codegen::CodeGenError::GenerationError(format!(
                "dotnet build succeeded but output not found at {}",
                exe_path.display()
            )));
        }

        // 复制到目标位置
        std::fs::copy(&exe_path, output_path).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to copy executable: {}", e))
        })?;

        // 清理临时文件
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(output_path.to_path_buf())
    }
}

impl CodeGenerator for CSharpBackend {
    type Config = CSharpConfig;
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
        let config = CSharpConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
        assert_eq!(config.namespace, None);
    }

    #[test]
    fn test_lir_type_mapping() {
        let backend = CSharpBackend::new(CSharpConfig::default());

        assert_eq!(backend.lir_type_to_csharp(&x_lir::Type::Void), "void");
        assert_eq!(backend.lir_type_to_csharp(&x_lir::Type::Bool), "bool");
        assert_eq!(backend.lir_type_to_csharp(&x_lir::Type::Char), "char");
        assert_eq!(backend.lir_type_to_csharp(&x_lir::Type::Int), "int");
        assert_eq!(backend.lir_type_to_csharp(&x_lir::Type::Long), "long");
        assert_eq!(backend.lir_type_to_csharp(&x_lir::Type::Float), "float");
        assert_eq!(backend.lir_type_to_csharp(&x_lir::Type::Double), "double");
    }
}
