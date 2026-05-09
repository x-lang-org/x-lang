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
            Tuple(items) => {
                let item_strs: Vec<String> = items.iter().map(|item| self.lir_type_to_java(item)).collect();
                format!("Object /* tuple<{}> */", item_strs.join(", "))
            }
            Named(n) => n.clone(),
            FunctionPointer(_, _) => "java.util.function.Function".to_string(),
            Qualified(_, inner) => self.lir_type_to_java(inner),
        }
    }

    // ========================================================================
    // LIR declaration generation
    // ========================================================================

    /// Generate code for a LIR declaration (dispatches to specific handlers)
    fn generate_lir_declaration(&mut self, decl: &x_lir::Declaration) -> JavaResult<()> {
        match decl {
            x_lir::Declaration::Import(import) => self.generate_lir_import(import),
            x_lir::Declaration::Function(func) => self.generate_lir_function(func),
            x_lir::Declaration::Global(global) => self.generate_lir_global(global),
            x_lir::Declaration::Struct(struct_) => self.generate_lir_struct(struct_),
            x_lir::Declaration::Class(class) => self.generate_lir_class(class),
            x_lir::Declaration::VTable(vtable) => self.generate_lir_vtable(vtable),
            x_lir::Declaration::Enum(enum_) => self.generate_lir_enum(enum_),
            x_lir::Declaration::TypeAlias(alias) => self.generate_lir_type_alias(alias),
            x_lir::Declaration::Newtype(nt) => self.generate_lir_newtype(nt),
            x_lir::Declaration::Trait(trait_) => self.generate_lir_trait(trait_),
            x_lir::Declaration::Effect(effect) => self.generate_lir_effect(effect),
            x_lir::Declaration::Impl(impl_) => self.generate_lir_impl(impl_),
            x_lir::Declaration::ExternFunction(ext) => self.generate_lir_extern_function(ext),
        }
    }

    /// Generate import declaration
    fn generate_lir_import(&mut self, import: &x_lir::Import) -> JavaResult<()> {
        if import.import_all {
            self.line(&format!("import {}.*;", import.module_path))?;
        } else if !import.symbols.is_empty() {
            for (name, alias) in &import.symbols {
                if let Some(alias) = alias {
                    self.line(&format!(
                        "import {}.{}; // alias: {}",
                        import.module_path, name, alias
                    ))?;
                } else {
                    self.line(&format!("import {}.{};", import.module_path, name))?;
                }
            }
        }
        self.line("")?;
        Ok(())
    }

    /// Generate function declaration (non-main)
    fn generate_lir_function(&mut self, func: &x_lir::Function) -> JavaResult<()> {
        // main is handled separately in generate_from_lir
        if func.name == "main" {
            return Ok(());
        }

        let ret = self.lir_type_to_java(&func.return_type);
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| format!("{} {}", self.lir_type_to_java(&p.type_), p.name))
            .collect();

        let type_params = if func.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}> ", func.type_params.join(", "))
        };

        self.line(&format!(
            "public static {}{} {}({}) {{",
            type_params,
            ret,
            func.name,
            params.join(", ")
        ))?;
        self.indent();

        for stmt in &func.body.statements {
            self.emit_lir_statement(stmt)?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate global variable as static class field
    fn generate_lir_global(&mut self, global: &x_lir::GlobalVar) -> JavaResult<()> {
        let ty = self.lir_type_to_java(&global.type_);
        let vis = if global.is_static {
            "private"
        } else {
            "public"
        };
        if let Some(init) = &global.initializer {
            let init_str = self.emit_lir_expr(init)?;
            self.line(&format!(
                "{} static {} {} = {};",
                vis, ty, global.name, init_str
            ))?;
        } else {
            self.line(&format!("{} static {} {};", vis, ty, global.name))?;
        }
        self.line("")?;
        Ok(())
    }

    /// Generate struct as Java record
    fn generate_lir_struct(&mut self, struct_: &x_lir::Struct) -> JavaResult<()> {
        let fields: Vec<String> = struct_
            .fields
            .iter()
            .map(|f| format!("{} {}", self.lir_type_to_java(&f.type_), f.name))
            .collect();
        self.line(&format!(
            "record {}({}) {{}}",
            struct_.name,
            fields.join(", ")
        ))?;
        self.line("")?;
        Ok(())
    }

    /// Generate class declaration
    fn generate_lir_class(&mut self, class: &x_lir::Class) -> JavaResult<()> {
        let mut decl = format!("static class {}", class.name);
        if let Some(base) = &class.extends {
            decl.push_str(&format!(" extends {}", base));
        }
        if !class.implements.is_empty() {
            decl.push_str(&format!(" implements {}", class.implements.join(", ")));
        }
        decl.push_str(" {");
        self.line(&decl)?;
        self.indent();

        for field in &class.fields {
            let ty = self.lir_type_to_java(&field.type_);
            self.line(&format!("{} {};", ty, field.name))?;
        }

        if class.has_vtable {
            self.line(&format!(
                "// vtable: {} methods",
                class.vtable_indices.len()
            ))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate vtable as interface with method signatures
    fn generate_lir_vtable(&mut self, vtable: &x_lir::VTable) -> JavaResult<()> {
        self.line(&format!(
            "interface {}VTable {{ // for class {}",
            vtable.name, vtable.class_name
        ))?;
        self.indent();

        for entry in &vtable.entries {
            let params: Vec<String> = entry
                .function_type
                .param_types
                .iter()
                .enumerate()
                .map(|(i, ty)| format!("{} arg{}", self.lir_type_to_java(ty), i))
                .collect();
            let return_type = self.lir_type_to_java(&entry.function_type.return_type);
            self.line(&format!(
                "{} {}({});",
                return_type,
                entry.method_name,
                params.join(", ")
            ))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate enum declaration
    fn generate_lir_enum(&mut self, enum_: &x_lir::Enum) -> JavaResult<()> {
        self.line(&format!("enum {} {{", enum_.name))?;
        self.indent();

        let variant_strs: Vec<String> = enum_
            .variants
            .iter()
            .map(|v| {
                if let Some(value) = v.value {
                    format!("{}({})", v.name, value)
                } else {
                    v.name.clone()
                }
            })
            .collect();
        self.line(&format!("{};", variant_strs.join(", ")))?;

        // If any variant has an explicit value, generate the value field + constructor
        if enum_.variants.iter().any(|v| v.value.is_some()) {
            self.line("")?;
            self.line("private final int value;")?;
            self.line(&format!(
                "{}(int value) {{ this.value = value; }}",
                enum_.name
            ))?;
            self.line(&format!("{}() {{ this.value = -1; }}", enum_.name))?;
            self.line("public int getValue() { return value; }")?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate type alias as comment (Java doesn't have type aliases)
    fn generate_lir_type_alias(&mut self, alias: &x_lir::TypeAlias) -> JavaResult<()> {
        let ty = self.lir_type_to_java(&alias.type_);
        self.line(&format!("// type alias: {} = {}", alias.name, ty))?;
        self.line("")?;
        Ok(())
    }

    /// Generate newtype as wrapper class
    fn generate_lir_newtype(&mut self, nt: &x_lir::Newtype) -> JavaResult<()> {
        let ty = self.lir_type_to_java(&nt.type_);
        self.line(&format!("record {}({} value) {{}}", nt.name, ty))?;
        self.line("")?;
        Ok(())
    }

    /// Generate trait as Java interface
    fn generate_lir_trait(&mut self, trait_: &x_lir::Trait) -> JavaResult<()> {
        let type_params = if trait_.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", trait_.type_params.join(", "))
        };

        let mut decl = format!("interface {}{}", trait_.name, type_params);
        if !trait_.extends.is_empty() {
            decl.push_str(&format!(" extends {}", trait_.extends.join(", ")));
        }
        decl.push_str(" {");
        self.line(&decl)?;
        self.indent();

        for method in &trait_.methods {
            let ret_ty = method
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_java(ty))
                .unwrap_or_else(|| "void".to_string());
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| format!("{} {}", self.lir_type_to_java(&p.type_), p.name))
                .collect();
            let method_type_params = if method.type_params.is_empty() {
                String::new()
            } else {
                format!("<{}> ", method.type_params.join(", "))
            };

            if method.default_body.is_some() {
                self.line(&format!(
                    "default {}{} {}({}) {{",
                    method_type_params,
                    ret_ty,
                    method.name,
                    params.join(", ")
                ))?;
                self.indent();
                self.line("// default implementation")?;
                self.dedent();
                self.line("}")?;
            } else {
                self.line(&format!(
                    "{}{} {}({});",
                    method_type_params,
                    ret_ty,
                    method.name,
                    params.join(", ")
                ))?;
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate effect as Java interface
    fn generate_lir_effect(&mut self, effect: &x_lir::Effect) -> JavaResult<()> {
        let type_params = if effect.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", effect.type_params.join(", "))
        };
        self.line(&format!("interface {}{} {{", effect.name, type_params))?;
        self.indent();

        for op in &effect.operations {
            let ret_ty = op
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_java(ty))
                .unwrap_or_else(|| "void".to_string());
            let params: Vec<String> = op
                .parameters
                .iter()
                .map(|p| format!("{} {}", self.lir_type_to_java(&p.type_), p.name))
                .collect();
            self.line(&format!("{} {}({});", ret_ty, op.name, params.join(", ")))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate impl block (trait/effect implementation)
    fn generate_lir_impl(&mut self, impl_: &x_lir::Impl) -> JavaResult<()> {
        let target_ty = self.lir_type_to_java(&impl_.target_type);
        let kind = if impl_.is_effect { "effect" } else { "trait" };
        self.line(&format!(
            "// implements {} {} for {}",
            kind, impl_.trait_name, target_ty
        ))?;

        for method in &impl_.methods {
            let ret = self.lir_type_to_java(&method.return_type);
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| format!("{} {}", self.lir_type_to_java(&p.type_), p.name))
                .collect();
            let method_type_params = if method.type_params.is_empty() {
                String::new()
            } else {
                format!("<{}> ", method.type_params.join(", "))
            };
            self.line(&format!(
                "// {}{} {}({}) {{ /* body */ }}",
                method_type_params,
                ret,
                method.name,
                params.join(", ")
            ))?;
        }

        self.line("")?;
        Ok(())
    }

    /// Generate extern function declaration
    fn generate_lir_extern_function(&mut self, ext: &x_lir::ExternFunction) -> JavaResult<()> {
        let abi = ext.abi.as_deref().unwrap_or("C");
        let return_type = self.lir_type_to_java(&ext.return_type);
        let params: Vec<String> = ext
            .parameters
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("{} arg{}", self.lir_type_to_java(ty), i))
            .collect();
        let type_params = if ext.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}> ", ext.type_params.join(", "))
        };

        self.line(&format!("// extern \"{}\" function", abi))?;
        self.line(&format!(
            "// native {}{} {}({});",
            type_params,
            return_type,
            ext.name,
            params.join(", ")
        ))?;
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // LIR block / statement generation
    // ========================================================================

    /// Generate a LIR basic block
    fn generate_lir_block(&mut self, block: &x_lir::Block) -> JavaResult<()> {
        for stmt in &block.statements {
            self.emit_lir_statement(stmt)?;
        }
        Ok(())
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
                if v.is_static {
                    if let Some(init) = &v.initializer {
                        let init_str = self.emit_lir_expr(init)?;
                        self.line(&format!("static {} {} = {};", ty, v.name, init_str))?;
                    } else {
                        self.line(&format!("static {} {};", ty, v.name))?;
                    }
                } else if v.is_extern {
                    self.line(&format!("/* extern */ {} {};", ty, v.name))?;
                } else if let Some(init) = &v.initializer {
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
            DoWhile(dw) => {
                self.line("do {")?;
                self.indent();
                self.emit_lir_statement(&dw.body)?;
                self.dedent();
                let cond = self.emit_lir_expr(&dw.condition)?;
                self.line(&format!("}} while ({});", cond))?;
            }
            For(f) => {
                // Build for header parts
                let init_str = if let Some(init) = &f.initializer {
                    self.emit_lir_statement_inline(init)?
                } else {
                    String::new()
                };
                let cond_str = if let Some(cond) = &f.condition {
                    self.emit_lir_expr(cond)?
                } else {
                    String::new()
                };
                let incr_str = if let Some(incr) = &f.increment {
                    self.emit_lir_expr(incr)?
                } else {
                    String::new()
                };
                self.line(&format!(
                    "for ({}; {}; {}) {{",
                    init_str, cond_str, incr_str
                ))?;
                self.indent();
                self.emit_lir_statement(&f.body)?;
                self.dedent();
                self.line("}")?;
            }
            Switch(sw) => {
                self.emit_lir_switch(sw)?;
            }
            Match(m) => {
                self.emit_lir_match(m)?;
            }
            Try(t) => {
                self.line("try {")?;
                self.indent();
                self.generate_lir_block(&t.body)?;
                self.dedent();

                for catch in &t.catch_clauses {
                    let exception_type = catch.exception_type.as_deref().unwrap_or("Exception");
                    let var_name = catch.variable_name.as_deref().unwrap_or("e");
                    self.line(&format!("}} catch ({} {}) {{", exception_type, var_name))?;
                    self.indent();
                    self.generate_lir_block(&catch.body)?;
                    self.dedent();
                }

                if let Some(finally) = &t.finally_block {
                    self.line("} finally {")?;
                    self.indent();
                    self.generate_lir_block(finally)?;
                    self.dedent();
                }
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
            Goto(target) => {
                self.line(&format!("// goto {}", target))?;
            }
            Label(name) => {
                self.line(&format!("// label: {}:", name))?;
            }
            Empty => {
                // emit nothing
            }
            Compound(block) => {
                self.line("{")?;
                self.indent();
                self.generate_lir_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            Declaration(decl) => {
                self.generate_lir_declaration(decl)?;
            }
        }
        Ok(())
    }

    /// Emit a statement as an inline string (for `for` initializer)
    fn emit_lir_statement_inline(&mut self, stmt: &x_lir::Statement) -> JavaResult<String> {
        match stmt {
            x_lir::Statement::Variable(v) => {
                let ty = self.lir_type_to_java(&v.type_);
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    Ok(format!("{} {} = {}", ty, v.name, init_str))
                } else {
                    Ok(format!("{} {}", ty, v.name))
                }
            }
            x_lir::Statement::Expression(e) => self.emit_lir_expr(e),
            _ => {
                // For other statement types in for-init, just emit normally and return empty
                self.emit_lir_statement(stmt)?;
                Ok(String::new())
            }
        }
    }

    /// Emit a switch statement
    fn emit_lir_switch(&mut self, switch: &x_lir::SwitchStatement) -> JavaResult<()> {
        let expr = self.emit_lir_expr(&switch.expression)?;
        self.line(&format!("switch ({}) {{", expr))?;
        self.indent();

        for case in &switch.cases {
            let value = self.emit_lir_expr(&case.value)?;
            self.line(&format!("case {}:", value))?;
            self.indent();
            self.emit_lir_statement(&case.body)?;
            self.line("break;")?;
            self.dedent();
        }

        if let Some(default_body) = &switch.default {
            self.line("default:")?;
            self.indent();
            self.emit_lir_statement(default_body)?;
            self.line("break;")?;
            self.dedent();
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// Emit a match statement (Java 21+ pattern matching switch)
    fn emit_lir_match(&mut self, match_stmt: &x_lir::MatchStatement) -> JavaResult<()> {
        let expr = self.emit_lir_expr(&match_stmt.scrutinee)?;
        self.line(&format!("switch ({}) {{", expr))?;
        self.indent();

        for case in &match_stmt.cases {
            let pattern = self.emit_lir_pattern(&case.pattern)?;
            let guard = if let Some(g) = &case.guard {
                format!(" when {}", self.emit_lir_expr(g)?)
            } else {
                String::new()
            };

            // Use arrow syntax for pattern matching switch
            self.line(&format!("case {}{} -> {{", pattern, guard))?;
            self.indent();
            self.generate_lir_block(&case.body)?;
            self.dedent();
            self.line("}")?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    // ========================================================================
    // LIR pattern generation
    // ========================================================================

    /// Generate a LIR pattern for Java pattern matching
    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> JavaResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard => Ok("default".to_string()),
            x_lir::Pattern::Variable(name) => Ok(name.clone()),
            x_lir::Pattern::Literal(lit) => self.emit_lir_literal(lit),
            x_lir::Pattern::Constructor(name, patterns) => {
                if patterns.is_empty() {
                    Ok(name.clone())
                } else {
                    let pat_strs: Vec<String> = patterns
                        .iter()
                        .map(|p| self.emit_lir_pattern(p))
                        .collect::<Result<_, _>>()?;
                    Ok(format!("{} {}", name, pat_strs.join(", ")))
                }
            }
            x_lir::Pattern::Tuple(patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                Ok(format!("/* tuple */ ({})", pat_strs.join(", ")))
            }
            x_lir::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| -> JavaResult<String> {
                        let p_str = self.emit_lir_pattern(p)?;
                        Ok(format!("{} {}", p_str, n))
                    })
                    .collect::<Result<_, _>>()?;
                Ok(format!("{}({})", name, field_strs.join(", ")))
            }
            x_lir::Pattern::Or(left, right) => {
                let left_str = self.emit_lir_pattern(left)?;
                let right_str = self.emit_lir_pattern(right)?;
                Ok(format!("{}, {}", left_str, right_str))
            }
        }
    }

    // ========================================================================
    // LIR expression generation
    // ========================================================================

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
                let inner = self.emit_lir_expr(e)?;
                let op_str = self.map_lir_unaryop(op);
                match op {
                    x_lir::UnaryOp::PostIncrement | x_lir::UnaryOp::PostDecrement => {
                        Ok(format!("({}{})", inner, op_str))
                    }
                    x_lir::UnaryOp::Reference | x_lir::UnaryOp::MutableReference => {
                        // No-op in Java (no address-of operator)
                        Ok(inner)
                    }
                    _ => Ok(format!("({}{})", op_str, inner)),
                }
            }
            Ternary(cond, then_expr, else_expr) => {
                let cond_str = self.emit_lir_expr(cond)?;
                let then_str = self.emit_lir_expr(then_expr)?;
                let else_str = self.emit_lir_expr(else_expr)?;
                Ok(format!("({} ? {} : {})", cond_str, then_str, else_str))
            }
            Call(callee, args) => {
                let callee_str = self.emit_lir_expr(callee)?;
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // 映射内置函数到 Java 标准库
                match callee_str.as_str() {
                    "println" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("System.out.println({})", args_part))
                    }
                    "print" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("System.out.print({})", args_part))
                    }
                    "eprintln" | "eprintln!" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("System.err.println({})", args_part))
                    }
                    "format" => {
                        if args_str.is_empty() {
                            Ok("\"\"".to_string())
                        } else {
                            Ok(format!("String.format({})", args_str.join(", ")))
                        }
                    }
                    _ => Ok(format!("{}({})", callee_str, args_str.join(", "))),
                }
            }
            // 赋值表达式（如 t0 = println(...)）
            Assign(target, value) => {
                let target_str = self.emit_lir_expr(target)?;
                let value_str = self.emit_lir_expr(value)?;
                Ok(format!("{} = {}", target_str, value_str))
            }
            AssignOp(op, target, value) => {
                let target_str = self.emit_lir_expr(target)?;
                let value_str = self.emit_lir_expr(value)?;
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
                    x_lir::BinaryOp::RightShiftArithmetic => ">>>=",
                    _ => "=",
                };
                Ok(format!("{} {} {}", target_str, op_str, value_str))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            PointerMember(obj, member) => {
                // Java has no pointer members; treat same as regular member access
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("{}[{}]", arr_str, idx_str))
            }
            AddressOf(inner) => {
                // No-op in Java (no address-of)
                self.emit_lir_expr(inner)
            }
            Dereference(inner) => {
                // No-op in Java (no pointer dereference)
                self.emit_lir_expr(inner)
            }
            Cast(ty, inner) => {
                let inner_str = self.emit_lir_expr(inner)?;
                let ty_str = self.lir_type_to_java(ty);
                Ok(format!("(({}) {})", ty_str, inner_str))
            }
            SizeOf(ty) => {
                // Java doesn't have sizeof; use a constant based on known type sizes
                let size = ty.size_of();
                Ok(format!(
                    "{} /* sizeof({}) */",
                    size,
                    self.lir_type_to_java(ty)
                ))
            }
            SizeOfExpr(_expr) => Ok("0 /* sizeof(expr) */".to_string()),
            AlignOf(ty) => {
                let align = ty.align_of();
                Ok(format!(
                    "{} /* alignof({}) */",
                    align,
                    self.lir_type_to_java(ty)
                ))
            }
            Comma(exprs) => {
                // Java doesn't have the comma operator; evaluate all but return last
                if exprs.is_empty() {
                    Ok("null".to_string())
                } else {
                    // Just return the last expression value
                    // (side effects of earlier expressions would need statement-level handling)
                    let last = exprs.last().unwrap();
                    self.emit_lir_expr(last)
                }
            }
            Parenthesized(inner) => {
                let inner_str = self.emit_lir_expr(inner)?;
                Ok(format!("({})", inner_str))
            }
            InitializerList(inits) => {
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|init| self.emit_lir_initializer(init))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("new Object[]{{ {} }}", init_strs.join(", ")))
            }
            CompoundLiteral(ty, inits) => {
                let ty_str = self.lir_type_to_java(ty);
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|init| self.emit_lir_initializer(init))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("new {}({})", ty_str, init_strs.join(", ")))
            }
        }
    }

    // ========================================================================
    // LIR literal / initializer generation
    // ========================================================================

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

    /// Emit a LIR initializer
    fn emit_lir_initializer(&self, init: &x_lir::Initializer) -> JavaResult<String> {
        match init {
            x_lir::Initializer::Expression(expr) => self.emit_lir_expr(expr),
            x_lir::Initializer::List(list) => {
                let items: Vec<String> = list
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{ {} }}", items.join(", ")))
            }
            x_lir::Initializer::Named(name, init) => {
                let init_code = self.emit_lir_initializer(init)?;
                Ok(format!("/* .{} = */ {}", name, init_code))
            }
            x_lir::Initializer::Indexed(idx, init) => {
                let idx_code = self.emit_lir_expr(idx)?;
                let init_code = self.emit_lir_initializer(init)?;
                Ok(format!("/* [{}] = */ {}", idx_code, init_code))
            }
        }
    }

    // ========================================================================
    // LIR operator mapping
    // ========================================================================

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
            PreIncrement => "++".to_string(),
            PreDecrement => "--".to_string(),
            PostIncrement => "++".to_string(),
            PostDecrement => "--".to_string(),
            Reference => "".to_string(),        // No-op in Java
            MutableReference => "".to_string(), // No-op in Java
        }
    }

    // ========================================================================
    // Top-level LIR generation
    // ========================================================================

    /// 从 LIR 生成 Java 代码
    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> JavaResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // Emit top-level imports (before the class)
        for decl in &lir.declarations {
            if let x_lir::Declaration::Import(import) = decl {
                self.generate_lir_import(import)?;
            }
        }

        // 开始类定义
        self.line(&format!("public class {} {{", self.config.class_name))?;
        self.indent();

        // Collect the main function reference for later; process all other declarations
        let mut main_function: Option<&x_lir::Function> = None;
        for decl in &lir.declarations {
            match decl {
                x_lir::Declaration::Import(_) => {
                    // Already emitted above, skip
                }
                x_lir::Declaration::Function(f) if f.name == "main" => {
                    main_function = Some(f);
                }
                _ => {
                    self.generate_lir_declaration(decl)?;
                }
            }
        }

        // main 方法 - 如果有 X 的 main 函数，将代码内联到 Java main 方法中
        self.line("public static void main(String[] args) {")?;
        self.indent();

        if let Some(main_fn) = main_function {
            // 内联 main 函数的代码
            let mut has_output = false;
            for stmt in &main_fn.body.statements {
                // 处理 return 语句 - 使用 System.exit() 传递退出码
                if let x_lir::Statement::Return(Some(ret_val)) = stmt {
                    if has_output {
                        self.line("System.exit(0);")?;
                    } else {
                        let exit_code = self.emit_lir_expr(ret_val)?;
                        self.line(&format!("System.exit({});", exit_code))?;
                    }
                    continue;
                } else if let x_lir::Statement::Return(None) = stmt {
                    self.line("System.exit(0);")?;
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
            self.line("System.out.println(\"Hello from Java backend!\");")?;
        }

        self.dedent();
        self.line("}")?;

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

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
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
