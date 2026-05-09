//! Erlang 后端 - 生成 Erlang OTP 28 源代码
//!
//! 面向并发、分布式系统、高可用场景
//!
//! ## Erlang OTP 28 特性支持
//! - Gradual set-theoretic types（渐进式集合论类型）
//! - maybe 类型
//! - JSON 支持
//! - Process labels（进程标签）
//! - Improved maps（改进的映射）
//! - Map comprehensions
//! - Improved ETS and dialyzer
//!
//! ## Erlang 语法特点
//! - 变量以大写字母或下划线开头
//! - 原子以小写字母开头
//! - 函数定义使用 `->` 和 `.`
//! - 模式匹配使用 `case ... of ... end`
//! - 输出使用 `io:format/2`

#![allow(
    clippy::if_same_then_else,
    clippy::only_used_in_recursion,
    clippy::useless_format
)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;

/// Erlang 后端配置
#[derive(Debug, Clone)]
pub struct ErlangBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub module_name: Option<String>,
}

impl Default for ErlangBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            module_name: None,
        }
    }
}

/// Erlang 后端
pub struct ErlangBackend {
    #[allow(dead_code)]
    config: ErlangBackendConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
    module_name: String,
    exports: Vec<String>,
    /// 用于生成唯一的 while/do-while/loop 辅助函数名
    loop_counter: usize,
}

pub type ErlangResult<T> = Result<T, x_codegen::CodeGenError>;

// 保持向后兼容的别名
pub type ErlangCodeGenerator = ErlangBackend;
pub type ErlangCodeGenError = x_codegen::CodeGenError;

impl ErlangBackend {
    pub fn new(config: ErlangBackendConfig) -> Self {
        let module_name = config
            .module_name
            .clone()
            .unwrap_or_else(|| "x_module".to_string());
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
            module_name,
            exports: Vec::new(),
            loop_counter: 0,
        }
    }

    fn next_loop_id(&mut self) -> usize {
        self.loop_counter += 1;
        self.loop_counter
    }

    fn line(&mut self, s: &str) -> ErlangResult<()> {
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

    /// 发射模块头 (Erlang OTP 28)
    fn emit_header(&mut self) -> ErlangResult<()> {
        self.line(headers::ERLANG)?;
        self.line("%%% DO NOT EDIT")?;
        self.line("%%% Target: Erlang OTP 28")?;
        self.line("")?;
        self.line(&format!("-module({}).", self.module_name))?;
        self.line("")?;

        if !self.exports.is_empty() {
            let exports: String = self.exports.join(", ");
            self.line(&format!("-export([{}]).", exports))?;
            self.line("")?;
        }

        Ok(())
    }

    /// 发射默认 main 函数
    fn emit_default_main(&mut self) -> ErlangResult<()> {
        self.line("main() ->")?;
        self.indent();
        self.line("io:format(\"Hello from Erlang backend!~n\", []).")?;
        self.dedent();
        Ok(())
    }

    /// 字段名映射为 Erlang map 键（小写原子，必要时引号）
    fn erlang_field_atom(&self, field: &str) -> String {
        if field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && !field.is_empty() {
            field.to_lowercase()
        } else {
            format!("'{}'", field.replace('\\', "\\\\").replace('\'', "\\'"))
        }
    }

    /// 将 X 变量名转换为 Erlang 变量名
    /// Erlang 变量必须以大写字母或下划线开头
    fn erlang_variable(&self, name: &str) -> String {
        if name.starts_with('_') {
            name.to_string()
        } else if name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
        {
            name.to_string()
        } else {
            // 首字母大写
            let mut chars: Vec<char> = name.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            chars.into_iter().collect()
        }
    }
}

impl CodeGenerator for ErlangBackend {
    type Config = ErlangBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        Self::generate_from_lir(self, lir)
    }
}

/// LIR -> Erlang 辅助方法
impl ErlangBackend {
    /// 将 LIR 类型转换为 Erlang 类型（用于 `-spec` 等扩展）
    fn lir_type_to_erlang(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "ok".to_string(),
            Bool => "boolean()".to_string(),
            Char => "char()".to_string(),
            Schar | Short | Int | Uint => "integer()".to_string(),
            Uchar | Ushort | Long | Ulong | LongLong | UlongLong => "non_neg_integer()".to_string(),
            Float | Double | LongDouble => "float()".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "integer()".to_string(),
            Pointer(_) => "term()".to_string(),
            Array(_, _) => "[term()]".to_string(),
            Tuple(items) => {
                let item_strs: Vec<String> = items.iter().map(|item| self.lir_type_to_erlang(item)).collect();
                format!("{{{}}}", item_strs.join(", "))
            }
            FunctionPointer(_, _) => "fun()".to_string(),
            Named(n) => n.clone(),
            Qualified(_, inner) => self.lir_type_to_erlang(inner),
        }
    }

    /// 将名称转为小写 Erlang atom（用于 module/record/variant 等）
    fn erlang_atom(&self, name: &str) -> String {
        let lower = name.to_lowercase();
        // Erlang atoms: start with lowercase letter or single-quoted
        if lower
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false)
            && lower.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            lower
        } else {
            format!("'{}'", lower.replace('\\', "\\\\").replace('\'', "\\'"))
        }
    }

    /// 从 LIR 生成 Erlang 代码
    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> ErlangResult<CodegenOutput> {
        self.buffer.clear();
        self.exports.clear();
        self.loop_counter = 0;

        // First pass: collect functions for export list and check for main
        let mut has_main = false;
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                self.exports
                    .push(format!("{}/{}", f.name, f.parameters.len()));
                if f.name == "main" {
                    has_main = true;
                }
            }
        }
        if !has_main {
            self.exports.push("main/0".to_string());
        }

        self.emit_header()?;

        // Process all declarations in order
        for decl in &lir.declarations {
            self.emit_lir_declaration(decl)?;
        }

        if !has_main {
            self.emit_default_main()?;
        }

        let output_file = OutputFile {
            path: PathBuf::from(format!("{}.erl", self.module_name)),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::Erlang,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// Emit a LIR declaration
    fn emit_lir_declaration(&mut self, decl: &x_lir::Declaration) -> ErlangResult<()> {
        match decl {
            x_lir::Declaration::Function(f) => {
                self.emit_lir_function(f)?;
                self.line("")?;
            }
            x_lir::Declaration::Global(global) => {
                self.emit_lir_global(global)?;
                self.line("")?;
            }
            x_lir::Declaration::Struct(s) => {
                self.emit_lir_struct(s)?;
                self.line("")?;
            }
            x_lir::Declaration::Class(c) => {
                self.emit_lir_class(c)?;
                self.line("")?;
            }
            x_lir::Declaration::VTable(vt) => {
                self.emit_lir_vtable(vt)?;
                self.line("")?;
            }
            x_lir::Declaration::Enum(e) => {
                self.emit_lir_enum(e)?;
                self.line("")?;
            }
            x_lir::Declaration::TypeAlias(alias) => {
                self.emit_lir_type_alias(alias)?;
                self.line("")?;
            }
            x_lir::Declaration::Newtype(nt) => {
                self.emit_lir_newtype(nt)?;
                self.line("")?;
            }
            x_lir::Declaration::Trait(t) => {
                self.emit_lir_trait(t)?;
                self.line("")?;
            }
            x_lir::Declaration::Effect(eff) => {
                self.emit_lir_effect(eff)?;
                self.line("")?;
            }
            x_lir::Declaration::Impl(impl_) => {
                self.emit_lir_impl(impl_)?;
                self.line("")?;
            }
            x_lir::Declaration::ExternFunction(ext) => {
                self.emit_lir_extern_function(ext)?;
                self.line("")?;
            }
            x_lir::Declaration::Import(import) => {
                self.emit_lir_import(import)?;
                self.line("")?;
            }
        }
        Ok(())
    }

    /// Emit global variable as a zero-arity function returning the constant
    fn emit_lir_global(&mut self, global: &x_lir::GlobalVar) -> ErlangResult<()> {
        let name = self.erlang_atom(&global.name);
        let init = global
            .initializer
            .as_ref()
            .map(|e| self.emit_lir_expr(e))
            .transpose()?
            .unwrap_or_else(|| "undefined".to_string());
        self.line(&format!("%% global: {}", global.name))?;
        self.line(&format!("{}() -> {}.", name, init))?;
        Ok(())
    }

    /// Emit struct as Erlang record definition
    fn emit_lir_struct(&mut self, s: &x_lir::Struct) -> ErlangResult<()> {
        let name = self.erlang_atom(&s.name);
        if s.fields.is_empty() {
            self.line(&format!("-record({}, {{}}).", name))?;
        } else {
            let fields: Vec<String> = s
                .fields
                .iter()
                .map(|f| self.erlang_field_atom(&f.name))
                .collect();
            self.line(&format!("-record({}, {{{}}}).", name, fields.join(", ")))?;
        }
        Ok(())
    }

    /// Emit class as record + comment about vtable/inheritance
    fn emit_lir_class(&mut self, class: &x_lir::Class) -> ErlangResult<()> {
        let name = self.erlang_atom(&class.name);
        self.line(&format!("%% class: {}", class.name))?;
        if let Some(parent) = &class.extends {
            self.line(&format!("%% extends: {}", parent))?;
        }
        if !class.implements.is_empty() {
            self.line(&format!("%% implements: {}", class.implements.join(", ")))?;
        }
        let mut fields: Vec<String> = Vec::new();
        if class.has_vtable {
            fields.push("__vtable".to_string());
        }
        for f in &class.fields {
            fields.push(self.erlang_field_atom(&f.name));
        }
        if fields.is_empty() {
            self.line(&format!("-record({}, {{}}).", name))?;
        } else {
            self.line(&format!("-record({}, {{{}}}).", name, fields.join(", ")))?;
        }
        Ok(())
    }

    /// Emit vtable as a comment (Erlang has no vtable concept)
    fn emit_lir_vtable(&mut self, vtable: &x_lir::VTable) -> ErlangResult<()> {
        self.line(&format!(
            "%% vtable: {} for class {}",
            vtable.name, vtable.class_name
        ))?;
        for entry in &vtable.entries {
            let params: Vec<String> = entry
                .function_type
                .param_types
                .iter()
                .map(|ty| self.lir_type_to_erlang(ty))
                .collect();
            let ret = self.lir_type_to_erlang(&entry.function_type.return_type);
            self.line(&format!(
                "%%   {} :: fun(({}) -> {})",
                entry.method_name,
                params.join(", "),
                ret
            ))?;
        }
        Ok(())
    }

    /// Emit enum as -define macros (atoms with optional integer values)
    fn emit_lir_enum(&mut self, enum_: &x_lir::Enum) -> ErlangResult<()> {
        self.line(&format!("%% enum: {}", enum_.name))?;
        for (idx, variant) in enum_.variants.iter().enumerate() {
            let val = variant.value.unwrap_or(idx as i64);
            let macro_name = format!(
                "{}_{}",
                enum_.name.to_uppercase(),
                variant.name.to_uppercase()
            );
            self.line(&format!("-define({}, {}).", macro_name, val))?;
        }
        Ok(())
    }

    /// Emit type alias
    fn emit_lir_type_alias(&mut self, alias: &x_lir::TypeAlias) -> ErlangResult<()> {
        let name = self.erlang_atom(&alias.name);
        let ty = self.lir_type_to_erlang(&alias.type_);
        self.line(&format!("-type {}() :: {}.", name, ty))?;
        Ok(())
    }

    /// Emit newtype as a tagged tuple type
    fn emit_lir_newtype(&mut self, nt: &x_lir::Newtype) -> ErlangResult<()> {
        let name = self.erlang_atom(&nt.name);
        let inner = self.lir_type_to_erlang(&nt.type_);
        self.line(&format!("-type {}() :: {{{}, {}}}.", name, name, inner))?;
        Ok(())
    }

    /// Emit trait as Erlang behaviour with -callback declarations
    fn emit_lir_trait(&mut self, trait_: &x_lir::Trait) -> ErlangResult<()> {
        self.line(&format!("%% trait: {}", trait_.name))?;
        if !trait_.extends.is_empty() {
            self.line(&format!("%% extends: {}", trait_.extends.join(", ")))?;
        }
        for method in &trait_.methods {
            let ret_ty = method
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_erlang(ty))
                .unwrap_or_else(|| "ok".to_string());
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| self.lir_type_to_erlang(&p.type_))
                .collect();
            let params_str = if params.is_empty() {
                String::new()
            } else {
                params.join(", ")
            };
            self.line(&format!(
                "-callback {}({}) -> {}.",
                method.name, params_str, ret_ty
            ))?;
        }
        Ok(())
    }

    /// Emit effect as Erlang behaviour callbacks
    fn emit_lir_effect(&mut self, effect: &x_lir::Effect) -> ErlangResult<()> {
        self.line(&format!("%% effect: {}", effect.name))?;
        for op in &effect.operations {
            let ret_ty = op
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_erlang(ty))
                .unwrap_or_else(|| "ok".to_string());
            let params: Vec<String> = op
                .parameters
                .iter()
                .map(|p| self.lir_type_to_erlang(&p.type_))
                .collect();
            let params_str = if params.is_empty() {
                String::new()
            } else {
                params.join(", ")
            };
            self.line(&format!(
                "-callback {}({}) -> {}.",
                op.name, params_str, ret_ty
            ))?;
        }
        Ok(())
    }

    /// Emit trait/effect implementation as Erlang functions
    fn emit_lir_impl(&mut self, impl_: &x_lir::Impl) -> ErlangResult<()> {
        let target = self.lir_type_to_erlang(&impl_.target_type);
        self.line(&format!("%% impl {} for {}", impl_.trait_name, target))?;
        for method in &impl_.methods {
            self.emit_lir_function(method)?;
            self.line("")?;
        }
        Ok(())
    }

    /// Emit extern function as a comment (Erlang uses NIFs or port drivers)
    fn emit_lir_extern_function(&mut self, ext: &x_lir::ExternFunction) -> ErlangResult<()> {
        let abi = ext.abi.as_deref().unwrap_or("C");
        let params: Vec<String> = ext
            .parameters
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let ty_str = self.lir_type_to_erlang(ty);
                format!("_Arg{} :: {}", i, ty_str)
            })
            .collect();
        let ret = self.lir_type_to_erlang(&ext.return_type);
        self.line(&format!("%% extern \"{}\" function: {}", abi, ext.name))?;
        self.line(&format!(
            "%% -spec {}({}) -> {}.",
            ext.name,
            params.join(", "),
            ret
        ))?;
        self.line(&format!(
            "{}({}) -> erlang:nif_error(not_loaded).",
            ext.name,
            (0..ext.parameters.len())
                .map(|i| format!("_Arg{}", i))
                .collect::<Vec<_>>()
                .join(", ")
        ))?;
        Ok(())
    }

    /// Emit import declaration
    fn emit_lir_import(&mut self, import: &x_lir::Import) -> ErlangResult<()> {
        let module = self.erlang_atom(&import.module_path);
        if import.import_all {
            self.line(&format!("%% import all from {}", import.module_path))?;
            self.line(&format!("%% -import({}, [...]).", module))?;
        } else if !import.symbols.is_empty() {
            let funcs: Vec<String> = import
                .symbols
                .iter()
                .map(|(name, _alias)| {
                    // We don't know arity from just a name, default to 0
                    format!("{}/0", name)
                })
                .collect();
            self.line(&format!("-import({}, [{}]).", module, funcs.join(", ")))?;
        }
        Ok(())
    }

    fn emit_lir_function(&mut self, f: &x_lir::Function) -> ErlangResult<()> {
        let ret_spec = self.lir_type_to_erlang(&f.return_type);
        let param_specs: Vec<String> = f
            .parameters
            .iter()
            .map(|p| self.lir_type_to_erlang(&p.type_))
            .collect();
        let spec_params = if param_specs.is_empty() {
            "()".to_string()
        } else {
            format!("({})", param_specs.join(", "))
        };
        self.line(&format!("-spec {}{} -> {}.", f.name, spec_params, ret_spec))?;
        let params: Vec<String> = f
            .parameters
            .iter()
            .map(|p| self.erlang_variable(&p.name))
            .collect();
        self.line(&format!("{}({}) ->", f.name, params.join(", ")))?;
        self.indent();
        let n = f.body.statements.len();
        if n == 0 {
            self.line("ok.")?;
        } else {
            for (i, stmt) in f.body.statements.iter().enumerate() {
                self.emit_lir_statement_seq(stmt, i + 1 == n)?;
            }
        }
        self.dedent();
        Ok(())
    }

    fn emit_lir_branch_boxed(&mut self, stmt: &x_lir::Statement) -> ErlangResult<()> {
        match stmt {
            x_lir::Statement::Compound(b) => {
                let n = b.statements.len();
                for (j, s) in b.statements.iter().enumerate() {
                    self.emit_lir_statement_seq(s, j + 1 == n)?;
                }
                Ok(())
            }
            s => self.emit_lir_statement_seq(s, true),
        }
    }

    fn emit_lir_loop_body(&mut self, stmt: &x_lir::Statement, label: &str) -> ErlangResult<()> {
        match stmt {
            x_lir::Statement::Compound(b) => {
                for s in &b.statements {
                    self.emit_lir_statement_seq(s, false)?;
                }
                self.line(&format!("{}();", label))?;
                Ok(())
            }
            s => {
                self.emit_lir_statement_seq(s, false)?;
                self.line(&format!("{}();", label))?;
                Ok(())
            }
        }
    }

    fn emit_lir_loop_body_postcond(
        &mut self,
        stmt: &x_lir::Statement,
        label: &str,
        cond: &str,
    ) -> ErlangResult<()> {
        match stmt {
            x_lir::Statement::Compound(b) => {
                for s in &b.statements {
                    self.emit_lir_statement_seq(s, false)?;
                }
            }
            s => {
                self.emit_lir_statement_seq(s, false)?;
            }
        }
        self.line(&format!("case {} of", cond))?;
        self.indent();
        self.line(&format!("true -> {}();", label))?;
        self.line("false -> ok")?;
        self.dedent();
        self.line("end.")?;
        Ok(())
    }

    fn emit_lir_statement_seq(
        &mut self,
        stmt: &x_lir::Statement,
        is_last: bool,
    ) -> ErlangResult<()> {
        use x_lir::Statement::*;
        let end = if is_last { "." } else { "," };
        match stmt {
            Expression(e) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{}{}", s, end))?;
            }
            Return(Some(e)) => {
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{}{}", s, end))?;
            }
            Return(None) => {
                self.line(&format!("ok{}", end))?;
            }
            Variable(v) => {
                let init = v
                    .initializer
                    .as_ref()
                    .map(|e| self.emit_lir_expr(e))
                    .transpose()?
                    .unwrap_or_else(|| "undefined".to_string());
                let name = self.erlang_variable(&v.name);
                self.line(&format!("{} = {}{}", name, init, end))?;
            }
            Compound(b) => {
                let m = b.statements.len();
                for (j, inner) in b.statements.iter().enumerate() {
                    self.emit_lir_statement_seq(inner, is_last && j + 1 == m)?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("case {} of", cond))?;
                self.indent();
                self.line("true ->")?;
                self.indent();
                self.emit_lir_branch_boxed(&i.then_branch)?;
                self.dedent();
                self.line(";")?;
                self.line("false ->")?;
                self.indent();
                match &i.else_branch {
                    Some(el) => self.emit_lir_branch_boxed(el)?,
                    None => self.line("ok")?,
                }
                self.dedent();
                self.dedent();
                self.line(&format!("end{}", end))?;
            }
            While(w) => {
                let id = self.next_loop_id();
                let label = format!("__lir_while_{}", id);
                let c = self.emit_lir_expr(&w.condition)?;
                self.line(&format!("{}() ->", label))?;
                self.indent();
                self.line(&format!("case {} of", c))?;
                self.indent();
                self.line("true ->")?;
                self.indent();
                self.emit_lir_loop_body(&w.body, &label)?;
                self.dedent();
                self.line("false -> ok")?;
                self.dedent();
                self.dedent();
                self.line("end.")?;
                self.dedent();
                self.line(&format!("{}(){}", label, end))?;
            }
            DoWhile(d) => {
                let id = self.next_loop_id();
                let label = format!("__lir_dowhile_{}", id);
                let c = self.emit_lir_expr(&d.condition)?;
                self.line(&format!("{}() ->", label))?;
                self.indent();
                self.emit_lir_loop_body_postcond(&d.body, &label, &c)?;
                self.dedent();
                self.line(&format!("{}(){}", label, end))?;
            }
            Empty => {}
            Break | Continue => {
                self.line(&format!("ok{}", end))?;
            }
            For(f) => {
                // Convert for loop to a recursive helper function, like while
                // for (init; cond; incr) body  =>
                //   init,
                //   __lir_for_N() ->
                //       case Cond of
                //           true -> Body, Incr, __lir_for_N();
                //           false -> ok
                //       end.
                //   __lir_for_N()
                if let Some(init) = &f.initializer {
                    self.emit_lir_statement_seq(init, false)?;
                }
                let id = self.next_loop_id();
                let label = format!("__lir_for_{}", id);
                let cond_str = f
                    .condition
                    .as_ref()
                    .map(|c| self.emit_lir_expr(c))
                    .transpose()?
                    .unwrap_or_else(|| "true".to_string());
                self.line(&format!("{}() ->", label))?;
                self.indent();
                self.line(&format!("case {} of", cond_str))?;
                self.indent();
                self.line("true ->")?;
                self.indent();
                // Emit body
                match f.body.as_ref() {
                    x_lir::Statement::Compound(b) => {
                        for s in &b.statements {
                            self.emit_lir_statement_seq(s, false)?;
                        }
                    }
                    s => {
                        self.emit_lir_statement_seq(s, false)?;
                    }
                }
                // Emit increment
                if let Some(incr) = &f.increment {
                    let incr_str = self.emit_lir_expr(incr)?;
                    self.line(&format!("{},", incr_str))?;
                }
                self.line(&format!("{}();", label))?;
                self.dedent();
                self.line("false -> ok")?;
                self.dedent();
                self.dedent();
                self.line("end.")?;
                self.dedent();
                self.line(&format!("{}(){}", label, end))?;
            }
            Switch(sw) => {
                let expr = self.emit_lir_expr(&sw.expression)?;
                self.line(&format!("case {} of", expr))?;
                self.indent();
                for (i, case) in sw.cases.iter().enumerate() {
                    let val = self.emit_lir_expr(&case.value)?;
                    self.line(&format!("{} ->", val))?;
                    self.indent();
                    self.emit_lir_branch_boxed(&case.body)?;
                    self.dedent();
                    // Separate cases with semicolon, except after the last one
                    // if there's a default or more cases
                    if i + 1 < sw.cases.len() || sw.default.is_some() {
                        self.line(";")?;
                    }
                }
                if let Some(default_body) = &sw.default {
                    self.line("_ ->")?;
                    self.indent();
                    self.emit_lir_branch_boxed(default_body)?;
                    self.dedent();
                }
                self.dedent();
                self.line(&format!("end{}", end))?;
            }
            Match(m) => {
                let scrutinee = self.emit_lir_expr(&m.scrutinee)?;
                self.line(&format!("case {} of", scrutinee))?;
                self.indent();
                for (i, case) in m.cases.iter().enumerate() {
                    let pat = self.emit_lir_pattern(&case.pattern)?;
                    let guard = if let Some(g) = &case.guard {
                        let g_str = self.emit_lir_expr(g)?;
                        format!(" when {}", g_str)
                    } else {
                        String::new()
                    };
                    self.line(&format!("{}{} ->", pat, guard))?;
                    self.indent();
                    let n_body = case.body.statements.len();
                    if n_body == 0 {
                        self.line("ok")?;
                    } else {
                        for (j, s) in case.body.statements.iter().enumerate() {
                            self.emit_lir_statement_seq(s, j + 1 == n_body)?;
                        }
                    }
                    self.dedent();
                    if i + 1 < m.cases.len() {
                        self.line(";")?;
                    }
                }
                self.dedent();
                self.line(&format!("end{}", end))?;
            }
            Try(t) => {
                self.line("try")?;
                self.indent();
                let n_body = t.body.statements.len();
                if n_body == 0 {
                    self.line("ok")?;
                } else {
                    for (j, s) in t.body.statements.iter().enumerate() {
                        self.emit_lir_statement_seq(s, j + 1 == n_body)?;
                    }
                }
                self.dedent();
                if !t.catch_clauses.is_empty() {
                    self.line("catch")?;
                    self.indent();
                    for (i, catch) in t.catch_clauses.iter().enumerate() {
                        let exc_type = catch.exception_type.as_deref().unwrap_or("_");
                        let var_name = catch
                            .variable_name
                            .as_ref()
                            .map(|n| self.erlang_variable(n))
                            .unwrap_or_else(|| "_".to_string());
                        self.line(&format!("{}:{} ->", exc_type, var_name))?;
                        self.indent();
                        let n_catch_body = catch.body.statements.len();
                        if n_catch_body == 0 {
                            self.line("ok")?;
                        } else {
                            for (j, s) in catch.body.statements.iter().enumerate() {
                                self.emit_lir_statement_seq(s, j + 1 == n_catch_body)?;
                            }
                        }
                        self.dedent();
                        if i + 1 < t.catch_clauses.len() {
                            self.line(";")?;
                        }
                    }
                    self.dedent();
                }
                if let Some(finally) = &t.finally_block {
                    self.line("after")?;
                    self.indent();
                    let n_fin = finally.statements.len();
                    if n_fin == 0 {
                        self.line("ok")?;
                    } else {
                        for (j, s) in finally.statements.iter().enumerate() {
                            self.emit_lir_statement_seq(s, j + 1 == n_fin)?;
                        }
                    }
                    self.dedent();
                }
                self.line(&format!("end{}", end))?;
            }
            Goto(target) => {
                self.line(&format!("% goto {}{}", target, end))?;
            }
            Label(name) => {
                self.line(&format!("% label: {}{}", name, end))?;
            }
            Declaration(decl) => {
                // Nested declaration - emit inline
                self.emit_lir_declaration(decl)?;
            }
        }
        Ok(())
    }

    /// Emit a LIR pattern for use in case clauses
    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> ErlangResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard => Ok("_".to_string()),
            x_lir::Pattern::Variable(name) => Ok(self.erlang_variable(name)),
            x_lir::Pattern::Literal(lit) => self.emit_lir_literal(lit),
            x_lir::Pattern::Constructor(name, patterns) => {
                if patterns.is_empty() {
                    Ok(self.erlang_atom(name))
                } else {
                    let pat_strs: Vec<String> = patterns
                        .iter()
                        .map(|p| self.emit_lir_pattern(p))
                        .collect::<ErlangResult<Vec<_>>>()?;
                    let atom = self.erlang_atom(name);
                    Ok(format!("{{{}, {}}}", atom, pat_strs.join(", ")))
                }
            }
            x_lir::Pattern::Tuple(patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("{{{}}}", pat_strs.join(", ")))
            }
            x_lir::Pattern::Record(name, fields) => {
                let rec_name = self.erlang_atom(name);
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(fname, fpat)| {
                        let fpat_str = self.emit_lir_pattern(fpat)?;
                        let fatom = self.erlang_field_atom(fname);
                        Ok(format!("{} = {}", fatom, fpat_str))
                    })
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("#{}{{ {} }}", rec_name, field_strs.join(", ")))
            }
            x_lir::Pattern::Or(left, right) => {
                // Erlang doesn't support or-patterns directly in a single clause.
                // We emit a comment noting this; callers using Match should
                // duplicate the clause. For inline usage, we use the left pattern.
                let left_str = self.emit_lir_pattern(left)?;
                let right_str = self.emit_lir_pattern(right)?;
                // Return a comment-annotated form; the Match handler above
                // uses case clauses separated by `;` which can support this.
                Ok(format!("{} %% | {}", left_str, right_str))
            }
        }
    }

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> ErlangResult<String> {
        use x_lir::Expression::*;
        match expr {
            Literal(l) => self.emit_lir_literal(l),
            Variable(n) => Ok(self.erlang_variable(n)),
            Binary(op, l, r) => {
                let left = self.emit_lir_expr(l)?;
                let right = self.emit_lir_expr(r)?;
                let op_str = self.map_lir_binop(op);
                Ok(format!("({} {} {})", left, op_str, right))
            }
            Unary(op, e) => {
                let e = self.emit_lir_expr(e)?;
                use x_lir::UnaryOp::*;
                match op {
                    Plus => Ok(format!("(+{})", e)),
                    Minus => Ok(format!("(-{})", e)),
                    Not => Ok(format!("(not {})", e)),
                    BitNot => Ok(format!("(bnot {})", e)),
                    PreIncrement | PostIncrement => Ok(format!("({} + 1)", e)),
                    PreDecrement | PostDecrement => Ok(format!("({} - 1)", e)),
                    Reference => Ok(format!("{}", e)),
                    MutableReference => Ok(format!("{}", e)),
                }
            }
            Ternary(c, t, el) => {
                let c = self.emit_lir_expr(c)?;
                let th = self.emit_lir_expr(t)?;
                let e = self.emit_lir_expr(el)?;
                Ok(format!(
                    "case ({} =/= 0) of true -> {}; false -> {} end",
                    c, th, e
                ))
            }
            Assign(t, v) => {
                let val = self.emit_lir_expr(v)?;
                match t.as_ref() {
                    x_lir::Expression::Variable(n) => {
                        let nv = self.erlang_variable(n);
                        Ok(format!("begin {} = {}, {} end", nv, val, nv))
                    }
                    _ => {
                        let ts = self.emit_lir_expr(t)?;
                        Ok(format!("begin _ = {}, {} end", val, ts))
                    }
                }
            }
            AssignOp(op, t, v) => {
                let tv = self.emit_lir_expr(t)?;
                let vv = self.emit_lir_expr(v)?;
                let sop = self.map_lir_binop(op);
                match t.as_ref() {
                    x_lir::Expression::Variable(n) => {
                        let nv = self.erlang_variable(n);
                        Ok(format!(
                            "begin {} = ({} {} {}), {} end",
                            nv, tv, sop, vv, nv
                        ))
                    }
                    _ => Ok(format!("({} {} {})", tv, sop, vv)),
                }
            }
            Call(callee, args) => self.emit_lir_call(callee, args),
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                Ok(format!("lists:nth(({}) + 1, {})", idx_str, arr_str))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                let atom = self.erlang_field_atom(member);
                Ok(format!("maps:get({}, {})", atom, obj_str))
            }
            PointerMember(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                let atom = self.erlang_field_atom(member);
                Ok(format!("maps:get({}, {})", atom, obj_str))
            }
            AddressOf(inner) => {
                let _ = self.emit_lir_expr(inner)?;
                Ok("% addressof".to_string())
            }
            Dereference(e) => self.emit_lir_expr(e),
            Cast(_ty, e) => self.emit_lir_expr(e),
            SizeOf(_ty) => Ok("8".to_string()),
            SizeOfExpr(_e) => Ok("8".to_string()),
            AlignOf(_ty) => Ok("8".to_string()),
            Comma(exprs) => {
                let parts: Vec<String> = exprs
                    .iter()
                    .map(|ex| self.emit_lir_expr(ex))
                    .collect::<ErlangResult<Vec<_>>>()?;
                Ok(format!("begin {} end", parts.join(", ")))
            }
            Parenthesized(e) => self.emit_lir_expr(e),
            InitializerList(inits) | CompoundLiteral(_, inits) => {
                let mut parts = Vec::new();
                for init in inits {
                    self.push_lir_init_expr(init, &mut parts)?;
                }
                Ok(format!("[{}]", parts.join(", ")))
            }
        }
    }

    fn push_lir_init_expr(
        &self,
        init: &x_lir::Initializer,
        out: &mut Vec<String>,
    ) -> ErlangResult<()> {
        match init {
            x_lir::Initializer::Expression(e) => {
                out.push(self.emit_lir_expr(e)?);
                Ok(())
            }
            x_lir::Initializer::List(list) => {
                for i in list {
                    self.push_lir_init_expr(i, out)?;
                }
                Ok(())
            }
            x_lir::Initializer::Named(_, inner) => self.push_lir_init_expr(inner, out),
            x_lir::Initializer::Indexed(_, inner) => self.push_lir_init_expr(inner, out),
        }
    }

    fn emit_lir_call(
        &self,
        callee: &x_lir::Expression,
        args: &[x_lir::Expression],
    ) -> ErlangResult<String> {
        if let x_lir::Expression::Variable(name) = callee {
            match name.as_str() {
                "print" | "println" => {
                    if args.is_empty() {
                        return Ok("io:format(\"~n\", [])".to_string());
                    }
                    let arg = self.emit_lir_expr(&args[0])?;
                    return Ok(format!("io:format(\"~p~n\", [{}])", arg));
                }
                "printf" => {
                    if args.is_empty() {
                        return Ok("io:format(\"\", [])".to_string());
                    }
                    let fmt = self.emit_lir_expr(&args[0])?;
                    let rest: Vec<String> = args[1..]
                        .iter()
                        .map(|a| self.emit_lir_expr(a))
                        .collect::<ErlangResult<Vec<_>>>()?;
                    return Ok(format!("io:format({}, [{}])", fmt, rest.join(", ")));
                }
                _ => {}
            }
        }
        let callee_str = self.emit_lir_expr(callee)?;
        let args_str: Vec<String> = args
            .iter()
            .map(|a| self.emit_lir_expr(a))
            .collect::<ErlangResult<Vec<_>>>()?;
        Ok(format!("{}({})", callee_str, args_str.join(", ")))
    }

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> ErlangResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(n.to_string()),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!(
                "\"{}\"",
                s.replace('\\', "\\\\").replace('"', "\\\"")
            )),
            Char(c) => Ok(format!("\"{}\"", c)),
            Bool(b) => Ok(if *b { "true" } else { "false" }.to_string()),
            NullPointer => Ok("undefined".to_string()),
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
            Modulo => "rem",
            LessThan => "<",
            LessThanEqual => "=<",
            GreaterThan => ">",
            GreaterThanEqual => ">=",
            Equal => "=:=",
            NotEqual => "/=",
            BitAnd => "band",
            BitOr => "bor",
            BitXor => "bxor",
            LeftShift => "bsl",
            RightShift => "bsr",
            RightShiftArithmetic => "bsr",
            LogicalAnd => "andalso",
            LogicalOr => "orelse",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ErlangBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
        assert!(config.module_name.is_none());
    }

    #[test]
    fn test_variable_naming() {
        let backend = ErlangBackend::new(ErlangBackendConfig::default());

        // 测试小写变量名转换
        assert_eq!(backend.erlang_variable("x"), "X");
        assert_eq!(backend.erlang_variable("myVar"), "MyVar");

        // 测试大写变量名保持不变
        assert_eq!(backend.erlang_variable("X"), "X");
        assert_eq!(backend.erlang_variable("MyVar"), "MyVar");

        // 测试下划线开头的变量保持不变
        assert_eq!(backend.erlang_variable("_temp"), "_temp");
    }

    #[test]
    fn test_lir_main_return_integer() {
        let mut prog = x_lir::Program::new();
        let mut main = x_lir::Function::new("main", x_lir::Type::Int);
        main.body
            .statements
            .push(x_lir::Statement::Return(Some(x_lir::Expression::int(42))));
        prog.add(x_lir::Declaration::Function(main));

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let out = backend.generate_from_lir(&prog).unwrap();
        let code = String::from_utf8_lossy(&out.files[0].content);

        assert!(code.contains("-module(x_module)."));
        assert!(code.contains("-export([main/0])."));
        assert!(code.contains("-spec main() -> integer()."));
        assert!(code.contains("main() ->"));
        assert!(code.contains("42."));
    }

    #[test]
    fn test_lir_empty_program_generates_default_main() {
        let prog = x_lir::Program::new();

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let out = backend.generate_from_lir(&prog).unwrap();
        let code = String::from_utf8_lossy(&out.files[0].content);

        assert!(code.contains("main() ->"));
        assert!(code.contains("Hello from Erlang backend!"));
    }

    #[test]
    fn test_lir_custom_module_name() {
        let config = ErlangBackendConfig {
            module_name: Some("my_custom_module".to_string()),
            ..Default::default()
        };
        let mut backend = ErlangBackend::new(config);

        let prog = x_lir::Program::new();
        let out = backend.generate_from_lir(&prog).unwrap();
        let code = String::from_utf8_lossy(&out.files[0].content);

        assert!(code.contains("-module(my_custom_module)."));
        assert!(out.files[0]
            .path
            .to_string_lossy()
            .contains("my_custom_module.erl"));
    }

    #[test]
    fn test_lir_function_with_parameters() {
        let mut prog = x_lir::Program::new();
        let mut add_fn = x_lir::Function::new("add", x_lir::Type::Int);
        add_fn.parameters.push(x_lir::Parameter {
            name: "a".to_string(),
            type_: x_lir::Type::Int,
        });
        add_fn.parameters.push(x_lir::Parameter {
            name: "b".to_string(),
            type_: x_lir::Type::Int,
        });
        add_fn
            .body
            .statements
            .push(x_lir::Statement::Return(Some(x_lir::Expression::Binary(
                x_lir::BinaryOp::Add,
                Box::new(x_lir::Expression::Variable("a".to_string())),
                Box::new(x_lir::Expression::Variable("b".to_string())),
            ))));
        prog.add(x_lir::Declaration::Function(add_fn));

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let out = backend.generate_from_lir(&prog).unwrap();
        let code = String::from_utf8_lossy(&out.files[0].content);

        assert!(code.contains("add(A, B) ->"));
        assert!(code.contains("(A + B)"));
        assert!(code.contains("-spec add(integer(), integer()) -> integer()."));
    }

    #[test]
    fn test_lir_println_call() {
        let mut prog = x_lir::Program::new();
        let mut main = x_lir::Function::new("main", x_lir::Type::Void);
        main.body
            .statements
            .push(x_lir::Statement::Expression(x_lir::Expression::Call(
                Box::new(x_lir::Expression::Variable("println".to_string())),
                vec![x_lir::Expression::Literal(x_lir::Literal::String(
                    "Hello, World!".to_string(),
                ))],
            )));
        prog.add(x_lir::Declaration::Function(main));

        let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
        let out = backend.generate_from_lir(&prog).unwrap();
        let code = String::from_utf8_lossy(&out.files[0].content);

        assert!(code.contains("io:format"));
        assert!(code.contains("Hello, World!"));
    }
}
