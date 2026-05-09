//! Wasm 32 汇编生成器
//!
//! 支持 WebAssembly 文本格式 (WAT).
//!
//! # 支持的 ABI
//!
//! - WebAssembly 标准调用约定
//!
//! # 示例输出
//!
//! ```wat
//! (module
//!   (func $main (result i32)
//!     ;; ... function body ...
//!     i32.const 0
//!     return
//!   )
//!   (export "main" (func $main))
//! )
//! ```

use std::collections::HashMap;
use std::fmt::Write;

use crate::{NativeError, NativeResult, TargetArch, TargetOS};
use x_lir as lir;

use super::{AssemblyGenerator, GlobalInfo};

/// Wasm 32 汇编生成器
pub struct Wasm32AssemblyGenerator {
    /// 输出缓冲区
    output: String,
    /// 当前缩进级别
    indent: usize,
    /// 标签计数器
    label_counter: usize,
    /// 字符串字面量表
    string_literals: HashMap<String, String>,
    /// 各字符串字面量在线性内存 `.data` 中的起始偏移（与 `generate` 中 data 段布局一致）
    string_memory_offsets: HashMap<String, usize>,
    /// 各全局变量在线性内存 `.data` 中的起始偏移（与 `generate` 中 data 段布局一致）
    global_memory_offsets: HashMap<String, usize>,
    /// 全局变量表
    globals: HashMap<String, GlobalInfo>,
    /// 局部变量索引（Wasm 使用索引而非栈偏移）
    local_indices: HashMap<String, u32>,
    /// 当前局部变量计数
    local_count: u32,
    /// 当前函数名
    current_function: String,
    /// 循环标签栈 - (continue_label, break_label) for each nested loop
    loop_labels: Vec<(String, String)>,
    /// 字段偏移：`StructName::field` -> 字节偏移（同模块多 struct 时避免同名字段冲突）
    field_offsets: HashMap<String, usize>,
    /// 当前函数参数与局部变量的静态类型（用于解析 Member / PointerMember）
    local_and_param_types: HashMap<String, lir::Type>,
}

impl Wasm32AssemblyGenerator {
    /// 创建新的 Wasm 32 汇编生成器
    pub fn new(_os: TargetOS) -> Self {
        let _ = _os;
        Self {
            output: String::new(),
            indent: 0,
            label_counter: 0,
            string_literals: HashMap::new(),
            string_memory_offsets: HashMap::new(),
            global_memory_offsets: HashMap::new(),
            globals: HashMap::new(),
            local_indices: HashMap::new(),
            local_count: 0,
            current_function: String::new(),
            loop_labels: Vec::new(),
            field_offsets: HashMap::new(),
            local_and_param_types: HashMap::new(),
        }
    }

    /// 清空生成器状态
    pub fn clear(&mut self) {
        self.output.clear();
        self.indent = 0;
        self.label_counter = 0;
        self.string_literals.clear();
        self.string_memory_offsets.clear();
        self.global_memory_offsets.clear();
        self.globals.clear();
        self.local_indices.clear();
        self.local_count = 0;
        self.current_function.clear();
        self.loop_labels.clear();
        self.field_offsets.clear();
        self.local_and_param_types.clear();
    }

    /// 生成唯一标签名
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("L_{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// WAT 块/循环标签（`$` + 唯一 id，用于 `br` / `br_if`）
    fn wat_label(&mut self, prefix: &str) -> String {
        format!("${}", self.new_label(prefix))
    }

    /// 输出一行汇编
    fn emit_line(&mut self, line: &str) -> NativeResult<()> {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
        writeln!(self.output, "{}", line)?;
        Ok(())
    }

    /// 输出原始文本
    fn emit_raw(&mut self, text: &str) -> NativeResult<()> {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
        writeln!(self.output, "{}", text)?;
        Ok(())
    }

    /// WAT 中带标识的局部槽须使用 `local.get $name` / `local.set $name`（与 `(local $name i32)` 一致）。
    #[inline]
    fn emit_local_get_named(&mut self, name: &str) -> NativeResult<()> {
        self.emit_line(&format!("local.get ${}", name))
    }

    #[inline]
    fn emit_local_set_named(&mut self, name: &str) -> NativeResult<()> {
        self.emit_line(&format!("local.set ${}", name))
    }

    /// 按槽位序号访问（参数与用户 `(local ...)` 的顺序与 `local_indices` 一致）。
    #[inline]
    fn emit_local_get_index(&mut self, idx: u32) -> NativeResult<()> {
        self.emit_line(&format!("local.get {}", idx))
    }

    #[inline]
    fn emit_local_set_index(&mut self, idx: u32) -> NativeResult<()> {
        self.emit_line(&format!("local.set {}", idx))
    }

    /// 获取类型的字节大小
    fn size_of_ty(&self, ty: &lir::Type) -> usize {
        match ty {
            lir::Type::Void => 0,
            lir::Type::Bool => 1,
            lir::Type::Char => 4,
            lir::Type::Schar => 1,
            lir::Type::Uchar => 1,
            lir::Type::Short => 2,
            lir::Type::Ushort => 2,
            lir::Type::Int => 4,
            lir::Type::Uint => 4,
            lir::Type::Long => 8,
            lir::Type::Ulong => 8,
            lir::Type::LongLong => 8,
            lir::Type::UlongLong => 8,
            lir::Type::Float => 4,
            lir::Type::Double => 8,
            lir::Type::Size => 4,
            lir::Type::Ptrdiff => 4,
            lir::Type::Pointer(_) => 4,
            lir::Type::FunctionPointer(_, _) => 4,
            _ => 4, // default
        }
    }

    /// 获取类型的对齐要求（字节）
    fn align_of_ty(&self, ty: &lir::Type) -> usize {
        match ty {
            lir::Type::Void => 1,
            lir::Type::Bool => 1,
            lir::Type::Char => 4,
            lir::Type::Schar => 1,
            lir::Type::Uchar => 1,
            lir::Type::Short => 2,
            lir::Type::Ushort => 2,
            lir::Type::Int => 4,
            lir::Type::Uint => 4,
            lir::Type::Long => 8,
            lir::Type::Ulong => 8,
            lir::Type::LongLong => 8,
            lir::Type::UlongLong => 8,
            lir::Type::Float => 4,
            lir::Type::Double => 8,
            lir::Type::Size => 4,
            lir::Type::Ptrdiff => 4,
            lir::Type::Pointer(_) => 4,
            lir::Type::FunctionPointer(_, _) => 4,
            lir::Type::Array(elem, _) => self.align_of_ty(elem),
            lir::Type::Tuple(items) => items.iter().map(|item| self.align_of_ty(item)).max().unwrap_or(1),
            lir::Type::Qualified(_, ty) => self.align_of_ty(ty),
            _ => 4,
        }
    }

    /// WAT `data` 段中写入 `n` 个零字节（`\00` 转义序列拼接）
    fn wat_zero_bytes(n: usize) -> String {
        (0..n).map(|_| "\\00").collect()
    }

    /// WAT `data` 字符串体：每个源字节写成 `\hh`（两位十六进制），与线形内存内容一致且总可被 WAT 解析。
    ///
    /// 不使用 Rust `escape_default()`：其 `\xNN`、`\u{...}` 等形式不是 WAT 字符串合法转义。
    fn wat_escape_data_string(s: &str) -> String {
        s.bytes().map(|b| format!("\\{:02x}", b)).collect()
    }

    /// 将 LIR 类型转换为 Wasm 类型
    fn ty_to_wasm(&self, ty: &lir::Type) -> &'static str {
        match ty {
            lir::Type::Void => "empty",
            lir::Type::Bool => "i32",
            lir::Type::Char => "i32",
            lir::Type::Schar => "i32",
            lir::Type::Uchar => "i32",
            lir::Type::Short => "i32",
            lir::Type::Ushort => "i32",
            lir::Type::Int => "i32",
            lir::Type::Uint => "i32",
            lir::Type::Long => "i64",
            lir::Type::Ulong => "i64",
            lir::Type::LongLong => "i64",
            lir::Type::UlongLong => "i64",
            lir::Type::Float => "f32",
            lir::Type::Double => "f64",
            lir::Type::Size => "i32",
            lir::Type::Ptrdiff => "i32",
            lir::Type::Pointer(_) => "i32",
            lir::Type::FunctionPointer(_, _) => "i32",
            _ => "i32", // default
        }
    }

    /// 从线性内存加载全局 `name` 到 `result_local`（按标量宽度读取，8 字节时截断为 i32）。
    fn emit_global_load(&mut self, name: &str, result_local: &str) -> NativeResult<()> {
        let addr = self
            .global_memory_offsets
            .get(name)
            .copied()
            .ok_or_else(|| {
                NativeError::CodegenError(format!("Global `{}` has no linear memory layout", name))
            })?;
        let size = self.globals.get(name).map(|g| g.size).ok_or_else(|| {
            NativeError::CodegenError(format!("Global `{}` metadata missing", name))
        })?;

        if size == 8 {
            self.emit_line(&format!("i32.const {}", addr))?;
            self.emit_line("i64.load")?;
            self.emit_line("i32.wrap_i64")?;
            self.emit_local_set_named(result_local)?;
            return Ok(());
        }

        let op = match size {
            1 => "i32.load8_u",
            2 => "i32.load16_u",
            4 => "i32.load",
            _ => {
                self.emit_line(&format!(
                    ";; TODO wasm global `{}` load size {}",
                    name, size
                ))?;
                "i32.load"
            }
        };
        self.emit_line(&format!("i32.const {}", addr))?;
        self.emit_line(op)?;
        self.emit_local_set_named(result_local)?;
        Ok(())
    }

    /// 将 `temp` 中的 i32 写入全局 `name` 的线性内存槽位（8 字节时符号扩展到 i64）。
    fn emit_global_store(&mut self, name: &str) -> NativeResult<()> {
        let addr = self
            .global_memory_offsets
            .get(name)
            .copied()
            .ok_or_else(|| {
                NativeError::CodegenError(format!("Global `{}` has no linear memory layout", name))
            })?;
        let size = self.globals.get(name).map(|g| g.size).ok_or_else(|| {
            NativeError::CodegenError(format!("Global `{}` metadata missing", name))
        })?;

        if size == 8 {
            self.emit_line(&format!("i32.const {}", addr))?;
            self.emit_local_get_named("temp")?;
            self.emit_line("i64.extend_i32_s")?;
            self.emit_line("i64.store")?;
            return Ok(());
        }

        self.emit_line(&format!("i32.const {}", addr))?;
        self.emit_local_get_named("temp")?;
        match size {
            1 => self.emit_line("i32.store8")?,
            2 => self.emit_line("i32.store16")?,
            4 => self.emit_line("i32.store")?,
            _ => {
                self.emit_line(&format!(
                    ";; TODO wasm global `{}` store size {}",
                    name, size
                ))?;
                self.emit_line("i32.store")?;
            }
        }
        Ok(())
    }

    /// 加载立即数
    fn emit_load_immediate(&mut self, value: i64, result_reg: &str) -> NativeResult<()> {
        // result_reg：具名临时槽（如 `temp`），输出 `local.set $name`
        self.emit_line(&format!("i32.const {}", value))?;
        self.emit_local_set_named(result_reg)?;
        Ok(())
    }

    /// 收集局部变量
    fn collect_locals(stmt: &lir::Statement, locals: &mut Vec<(String, lir::Type)>) {
        match stmt {
            lir::Statement::Variable(var) => {
                locals.push((var.name.clone(), var.type_.clone()));
            }
            lir::Statement::Compound(block) => {
                for stmt in &block.statements {
                    Self::collect_locals(stmt, locals);
                }
            }
            lir::Statement::If(if_stmt) => {
                Self::collect_locals(&*if_stmt.then_branch, locals);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::collect_locals(&**else_branch, locals);
                }
            }
            lir::Statement::For(for_stmt) => {
                if let Some(init) = &for_stmt.initializer {
                    Self::collect_locals(&**init, locals);
                }
                Self::collect_locals(&*for_stmt.body, locals);
            }
            lir::Statement::While(while_stmt) => {
                Self::collect_locals(&*while_stmt.body, locals);
            }
            lir::Statement::DoWhile(do_while) => {
                Self::collect_locals(&*do_while.body, locals);
            }
            lir::Statement::Switch(sw) => {
                for c in &sw.cases {
                    Self::collect_locals(&c.body, locals);
                }
                if let Some(def) = &sw.default {
                    Self::collect_locals(&**def, locals);
                }
            }
            lir::Statement::Match(m) => {
                for case in &m.cases {
                    for s in &case.body.statements {
                        Self::collect_locals(s, locals);
                    }
                }
            }
            lir::Statement::Try(t) => {
                for s in &t.body.statements {
                    Self::collect_locals(s, locals);
                }
                for c in &t.catch_clauses {
                    for s in &c.body.statements {
                        Self::collect_locals(s, locals);
                    }
                }
                if let Some(fin) = &t.finally_block {
                    for s in &fin.statements {
                        Self::collect_locals(s, locals);
                    }
                }
            }
            lir::Statement::Declaration(lir::Declaration::Function(f)) => {
                for s in &f.body.statements {
                    Self::collect_locals(s, locals);
                }
            }
            _ => {}
        }
    }

    /// 收集参数与语句中的变量静态类型（与 `collect_locals` 遍历范围一致）
    fn collect_var_types_stmt(stmt: &lir::Statement, types: &mut HashMap<String, lir::Type>) {
        match stmt {
            lir::Statement::Variable(var) => {
                types.insert(var.name.clone(), var.type_.clone());
            }
            lir::Statement::Compound(block) => {
                for s in &block.statements {
                    Self::collect_var_types_stmt(s, types);
                }
            }
            lir::Statement::If(if_stmt) => {
                Self::collect_var_types_stmt(&*if_stmt.then_branch, types);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::collect_var_types_stmt(&**else_branch, types);
                }
            }
            lir::Statement::For(for_stmt) => {
                if let Some(init) = &for_stmt.initializer {
                    Self::collect_var_types_stmt(&**init, types);
                }
                Self::collect_var_types_stmt(&*for_stmt.body, types);
            }
            lir::Statement::While(while_stmt) => {
                Self::collect_var_types_stmt(&*while_stmt.body, types);
            }
            lir::Statement::DoWhile(do_while) => {
                Self::collect_var_types_stmt(&*do_while.body, types);
            }
            lir::Statement::Switch(sw) => {
                for c in &sw.cases {
                    Self::collect_var_types_stmt(&c.body, types);
                }
                if let Some(def) = &sw.default {
                    Self::collect_var_types_stmt(&**def, types);
                }
            }
            lir::Statement::Match(m) => {
                for case in &m.cases {
                    for s in &case.body.statements {
                        Self::collect_var_types_stmt(s, types);
                    }
                }
            }
            lir::Statement::Try(t) => {
                for s in &t.body.statements {
                    Self::collect_var_types_stmt(s, types);
                }
                for c in &t.catch_clauses {
                    for s in &c.body.statements {
                        Self::collect_var_types_stmt(s, types);
                    }
                }
                if let Some(fin) = &t.finally_block {
                    for s in &fin.statements {
                        Self::collect_var_types_stmt(s, types);
                    }
                }
            }
            lir::Statement::Declaration(lir::Declaration::Function(f)) => {
                for s in &f.body.statements {
                    Self::collect_var_types_stmt(s, types);
                }
            }
            _ => {}
        }
    }

    fn peel_qualified(ty: &lir::Type) -> &lir::Type {
        match ty {
            lir::Type::Qualified(_, inner) => Self::peel_qualified(inner),
            t => t,
        }
    }

    fn layout_key(struct_name: &str, field: &str) -> String {
        format!("{}::{}", struct_name, field)
    }

    /// `*Named` 或 `Pointer(Named)` -> 结构体名
    fn struct_name_from_pointer_type(ty: &lir::Type) -> Option<String> {
        let ty = Self::peel_qualified(ty);
        match ty {
            lir::Type::Pointer(inner) => {
                let inner = Self::peel_qualified(inner);
                if let lir::Type::Named(s) = inner {
                    Some(s.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn struct_name_from_aggregate_type(ty: &lir::Type) -> Option<String> {
        let ty = Self::peel_qualified(ty);
        match ty {
            lir::Type::Named(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn infer_pointee_struct_for_expr(&self, expr: &lir::Expression) -> Option<String> {
        match expr {
            lir::Expression::Variable(n) => self
                .local_and_param_types
                .get(n)
                .and_then(Self::struct_name_from_pointer_type),
            lir::Expression::Cast(ty, e) => Self::struct_name_from_pointer_type(ty)
                .or_else(|| self.infer_pointee_struct_for_expr(e)),
            lir::Expression::Parenthesized(e) => self.infer_pointee_struct_for_expr(e),
            _ => None,
        }
    }

    fn infer_aggregate_struct_for_expr(&self, expr: &lir::Expression) -> Option<String> {
        match expr {
            lir::Expression::Variable(n) => self
                .local_and_param_types
                .get(n)
                .and_then(Self::struct_name_from_aggregate_type),
            lir::Expression::Dereference(inner) => self.infer_pointee_struct_for_expr(inner),
            lir::Expression::Cast(ty, e) => Self::struct_name_from_aggregate_type(ty)
                .or_else(|| self.infer_aggregate_struct_for_expr(e)),
            lir::Expression::Parenthesized(e) => self.infer_aggregate_struct_for_expr(e),
            _ => None,
        }
    }

    /// 当无法从表达式推断类型时，若仅有一个 `*::field` 匹配则使用该偏移
    fn unique_layout_field_offset(&self, field: &str) -> Option<usize> {
        let suffix = format!("::{}", field);
        let mut found: Option<usize> = None;
        for (k, &off) in &self.field_offsets {
            if k.ends_with(&suffix) {
                if found.is_some() {
                    return None;
                }
                found = Some(off);
            }
        }
        found.or_else(|| self.field_offsets.get(field).copied())
    }

    fn resolve_field_offset(
        &self,
        base: &lir::Expression,
        field: &str,
        pointer_member: bool,
    ) -> Option<usize> {
        let by_type = if pointer_member {
            self.infer_pointee_struct_for_expr(base)
        } else {
            self.infer_aggregate_struct_for_expr(base)
        };
        if let Some(s) = by_type {
            let key = Self::layout_key(&s, field);
            if let Some(&o) = self.field_offsets.get(&key) {
                return Some(o);
            }
        }
        self.unique_layout_field_offset(field)
    }

    /// 处理 LIR 表达式，将结果放入指定局部变量
    fn emit_expression(&mut self, expr: &lir::Expression, result_local: &str) -> NativeResult<()> {
        match expr {
            lir::Expression::Literal(lit) => {
                match lit {
                    lir::Literal::Bool(b) => {
                        self.emit_line(&format!("i32.const {}", if *b { 1 } else { 0 }))?;
                        self.emit_local_set_named(result_local)?;
                    }
                    lir::Literal::Integer(i) => {
                        self.emit_load_immediate(*i, result_local)?;
                    }
                    lir::Literal::UnsignedInteger(u) => {
                        self.emit_load_immediate(*u as i64, result_local)?;
                    }
                    lir::Literal::Long(i) => {
                        self.emit_load_immediate(*i, result_local)?;
                    }
                    lir::Literal::UnsignedLong(u) => {
                        self.emit_load_immediate(*u as i64, result_local)?;
                    }
                    lir::Literal::LongLong(i) => {
                        self.emit_load_immediate(*i, result_local)?;
                    }
                    lir::Literal::UnsignedLongLong(u) => {
                        self.emit_load_immediate(*u as i64, result_local)?;
                    }
                    lir::Literal::Float(_f) => {
                        // TODO: 加载浮点数
                        self.emit_line(&format!(";; TODO: float literal"))?;
                        self.emit_line(&format!("i32.const 0"))?;
                        self.emit_local_set_named(result_local)?;
                    }
                    lir::Literal::Double(_d) => {
                        // TODO: 加载浮点数
                        self.emit_line(&format!(";; TODO: double literal"))?;
                        self.emit_line(&format!("i32.const 0"))?;
                        self.emit_local_set_named(result_local)?;
                    }
                    lir::Literal::Char(c) => {
                        self.emit_line(&format!("i32.const {}", *c as i32))?;
                        self.emit_local_set_named(result_local)?;
                    }
                    lir::Literal::String(s) => {
                        // 字符串在 data 段中的偏移在 `generate` 里已写入 `string_memory_offsets`
                        let addr = self.string_memory_offsets.get(s).copied().ok_or_else(|| {
                            NativeError::CodegenError(format!(
                                "String literal memory offset not computed: {:?}",
                                s
                            ))
                        })?;
                        self.emit_line(&format!("i32.const {}", addr))?;
                        self.emit_local_set_named(result_local)?;
                    }
                    lir::Literal::NullPointer => {
                        self.emit_line(&format!("i32.const 0"))?;
                        self.emit_local_set_named(result_local)?;
                    }
                }
                Ok(())
            }
            lir::Expression::Variable(name) => {
                if let Some(&idx) = self.local_indices.get(name) {
                    self.emit_local_get_index(idx)?;
                    self.emit_local_set_named(result_local)?;
                } else if self.global_memory_offsets.contains_key(name) {
                    self.emit_global_load(name, result_local)?;
                } else {
                    return Err(NativeError::CodegenError(format!(
                        "Variable not found: {}",
                        name
                    )));
                }
                Ok(())
            }
            lir::Expression::Member(base, field_name) => {
                // Get base address (pointer to struct) then add field offset
                let offset_opt = self.resolve_field_offset(base, field_name, false);
                self.emit_expression(base, "temp_base")?;
                if let Some(offset) = offset_opt {
                    self.emit_local_get_named("temp_base")?;
                    self.emit_line(&format!("i32.const {}", offset))?;
                    self.emit_line("i32.add")?;
                    // Now load from the resulting address
                    self.emit_line("i32.load")?;
                    self.emit_local_set_named(result_local)?;
                } else {
                    self.emit_line(&format!(";; TODO: field offset not found: {}", field_name))?;
                    self.emit_line("i32.const 0")?;
                    self.emit_local_set_named(result_local)?;
                }
                Ok(())
            }
            lir::Expression::PointerMember(base, field_name) => {
                // base is already a pointer to the struct, add field offset
                let offset_opt = self.resolve_field_offset(base, field_name, true);
                self.emit_expression(base, "temp_base")?;
                if let Some(offset) = offset_opt {
                    self.emit_local_get_named("temp_base")?;
                    self.emit_line(&format!("i32.const {}", offset))?;
                    self.emit_line("i32.add")?;
                    self.emit_line("i32.load")?;
                    self.emit_local_set_named(result_local)?;
                } else {
                    self.emit_line(&format!(";; TODO: field offset not found: {}", field_name))?;
                    self.emit_line("i32.const 0")?;
                    self.emit_local_set_named(result_local)?;
                }
                Ok(())
            }
            lir::Expression::Dereference(ptr) => {
                self.emit_expression(ptr, "temp")?;
                self.emit_line("i32.load")?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::AddressOf(expr) => {
                match expr.as_ref() {
                    lir::Expression::Variable(name) => {
                        if let Some(addr) = self.global_memory_offsets.get(name) {
                            self.emit_line(&format!("i32.const {}", addr))?;
                            self.emit_local_set_named(result_local)?;
                        } else {
                            // Wasm: 局部变量不能直接取地址
                            self.emit_line(&format!(
                                ";; TODO: address of local variable {}",
                                name
                            ))?;
                            self.emit_line("i32.const 0")?;
                            self.emit_local_set_named(result_local)?;
                        }
                    }
                    _ => {
                        self.emit_expression(expr, result_local)?;
                    }
                }
                Ok(())
            }
            lir::Expression::Unary(op, operand) => {
                self.emit_expression(operand, "temp")?;
                match op {
                    lir::UnaryOp::Minus => {
                        self.emit_line("i32.const 0")?;
                        self.emit_local_get_named("temp")?;
                        self.emit_line("i32.sub")?;
                    }
                    lir::UnaryOp::BitNot => {
                        self.emit_line("i32.const -1")?;
                        self.emit_local_get_named("temp")?;
                        self.emit_line("i32.xor")?;
                    }
                    lir::UnaryOp::Not => {
                        self.emit_local_get_named("temp")?;
                        self.emit_line("i32.eqz")?;
                    }
                    _ => {
                        self.emit_line(&format!(";; TODO: unary op {:?}", op))?;
                    }
                }
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::Binary(op, left, right) => {
                // 先计算左操作数到 temp0，右操作数到 temp1
                self.emit_expression(left, "temp0")?;
                self.emit_expression(right, "temp1")?;

                let wasm_op = match op {
                    lir::BinaryOp::Add => "i32.add",
                    lir::BinaryOp::Subtract => "i32.sub",
                    lir::BinaryOp::Multiply => "i32.mul",
                    lir::BinaryOp::Divide => "i32.div_s",
                    lir::BinaryOp::Modulo => "i32.rem_s",
                    lir::BinaryOp::LeftShift => "i32.shl",
                    lir::BinaryOp::RightShift => "i32.shr_s",
                    lir::BinaryOp::BitAnd => "i32.and",
                    lir::BinaryOp::BitXor => "i32.xor",
                    lir::BinaryOp::BitOr => "i32.or",
                    lir::BinaryOp::LessThan => "i32.lt_s",
                    lir::BinaryOp::LessThanEqual => "i32.le_s",
                    lir::BinaryOp::GreaterThan => "i32.gt_s",
                    lir::BinaryOp::GreaterThanEqual => "i32.ge_s",
                    lir::BinaryOp::Equal => "i32.eq",
                    lir::BinaryOp::NotEqual => "i32.ne",
                    _ => ";; TODO",
                };

                self.emit_local_get_named("temp0")?;
                self.emit_local_get_named("temp1")?;
                if !matches!(
                    op,
                    lir::BinaryOp::LessThan
                        | lir::BinaryOp::LessThanEqual
                        | lir::BinaryOp::GreaterThan
                        | lir::BinaryOp::GreaterThanEqual
                        | lir::BinaryOp::Equal
                        | lir::BinaryOp::NotEqual
                ) {
                    self.emit_line(wasm_op)?;
                } else {
                    // Comparison operations already produce 0/1 result
                    self.emit_line(wasm_op)?;
                }
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::Call(callee, args) => {
                // Wasm ABI: 参数压栈，调用后结果在栈上
                for (i, arg) in args.iter().enumerate() {
                    let temp = format!("temp{}", i);
                    self.emit_expression(arg, &temp)?;
                    self.emit_local_get_named(&temp)?;
                }

                match callee.as_ref() {
                    lir::Expression::Variable(name) => {
                        self.emit_line(&format!("call ${}", name))?;
                    }
                    _ => {
                        // 间接调用需要通过 table
                        self.emit_expression(callee, "temp_callee")?;
                        self.emit_local_get_named("temp_callee")?;
                        self.emit_line("call_indirect (type $func_ty)")?;
                    }
                }

                // 结果已经在栈上，保存到结果局部变量
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::Index(base, index) => {
                self.emit_expression(base, "temp0")?;
                self.emit_expression(index, "temp1")?;
                // Scale by 4 bytes (Wasm 32-bit pointer)
                self.emit_local_get_named("temp1")?;
                self.emit_line("i32.const 2")?; // shl by 2 = *4
                self.emit_line("i32.shl")?;
                self.emit_local_get_named("temp0")?;
                self.emit_line("i32.add")?;
                self.emit_line("i32.load")?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::Assign(_, _) => {
                self.emit_line(";; TODO: assignment expression")?;
                self.emit_line("i32.const 0")?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::AssignOp(_, _, _) => {
                self.emit_line(";; TODO: assignment op expression")?;
                self.emit_line("i32.const 0")?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::Cast(_ty, expr) => {
                // Wasm: most casts happen via convert instructions
                self.emit_expression(expr, result_local)?;
                // TODO: proper conversion
                Ok(())
            }
            lir::Expression::SizeOf(ty) => {
                let size = self.size_of_ty(ty);
                self.emit_line(&format!("i32.const {}", size))?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::SizeOfExpr(_expr) => {
                self.emit_line(&format!("i32.const 4"))?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::AlignOf(ty) => {
                let align = self.align_of_ty(ty);
                self.emit_line(&format!("i32.const {}", align))?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::Ternary(cond, then, else_) => {
                let merge = self.wat_label("ternary_merge");
                let to_else = self.wat_label("ternary_else");

                self.emit_line(&format!("(block {}", merge))?;
                self.indent += 1;
                self.emit_line(&format!("(block {}", to_else))?;
                self.indent += 1;

                self.emit_expression(cond, "temp_cond")?;
                self.emit_local_get_named("temp_cond")?;
                self.emit_line("i32.eqz")?;
                self.emit_line(&format!("br_if {}", to_else))?;

                self.emit_expression(then, result_local)?;
                self.emit_line(&format!("br {}", merge))?;

                self.indent -= 1;
                self.emit_line(")")?;

                self.indent += 1;
                self.emit_expression(else_, result_local)?;
                self.indent -= 1;

                self.indent -= 1;
                self.emit_line(")")?;
                Ok(())
            }
            lir::Expression::Comma(_) => {
                self.emit_line(";; TODO: comma expression")?;
                self.emit_line("i32.const 0")?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::Parenthesized(expr) => {
                self.emit_expression(expr, result_local)?;
                Ok(())
            }
            lir::Expression::InitializerList(_) => {
                self.emit_line(";; TODO: initializer list")?;
                self.emit_line("i32.const 0")?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
            lir::Expression::CompoundLiteral(_, _) => {
                self.emit_line(";; TODO: compound literal")?;
                self.emit_line("i32.const 0")?;
                self.emit_local_set_named(result_local)?;
                Ok(())
            }
        }
    }

    /// 处理赋值
    #[allow(dead_code)]
    fn emit_assign(
        &mut self,
        target: &lir::Expression,
        source: &lir::Expression,
    ) -> NativeResult<()> {
        self.emit_expression(source, "temp")?;

        match target {
            lir::Expression::Variable(name) => {
                if let Some(&idx) = self.local_indices.get(name) {
                    self.emit_local_get_named("temp")?;
                    self.emit_local_set_index(idx)?;
                } else if self.global_memory_offsets.contains_key(name) {
                    self.emit_global_store(name)?;
                } else {
                    return Err(NativeError::CodegenError(format!(
                        "Variable not found: {}",
                        name
                    )));
                }
                Ok(())
            }
            lir::Expression::Member(base, field) => {
                let offset_opt = self.resolve_field_offset(base, field, false);
                self.emit_expression(base, "t1")?;
                if let Some(offset) = offset_opt {
                    self.emit_local_get_named("t1")?;
                    self.emit_line(&format!("i32.const {}", offset))?;
                    self.emit_line("i32.add")?;
                    self.emit_local_get_named("temp")?;
                    self.emit_line("i32.store")?;
                } else {
                    self.emit_line(&format!(
                        ";; TODO: field offset not found for assignment: {}",
                        field
                    ))?;
                }
                Ok(())
            }
            lir::Expression::PointerMember(base, field) => {
                let offset_opt = self.resolve_field_offset(base, field, true);
                self.emit_expression(base, "t1")?;
                if let Some(offset) = offset_opt {
                    self.emit_local_get_named("t1")?;
                    self.emit_line(&format!("i32.const {}", offset))?;
                    self.emit_line("i32.add")?;
                    self.emit_local_get_named("temp")?;
                    self.emit_line("i32.store")?;
                } else {
                    self.emit_line(&format!(
                        ";; TODO: field offset not found for assignment: {}",
                        field
                    ))?;
                }
                Ok(())
            }
            lir::Expression::Dereference(ptr) => {
                self.emit_expression(ptr, "t1")?;
                self.emit_local_get_named("temp")?;
                self.emit_line("i32.store")?;
                Ok(())
            }
            lir::Expression::Index(base, idx) => {
                self.emit_expression(base, "t0")?;
                self.emit_expression(idx, "t1")?;
                self.emit_local_get_named("t1")?;
                self.emit_line("i32.const 2")?;
                self.emit_line("i32.shl")?;
                self.emit_local_get_named("t0")?;
                self.emit_line("i32.add")?;
                self.emit_local_get_named("temp")?;
                self.emit_line("i32.store")?;
                Ok(())
            }
            _ => Err(NativeError::CodegenError(format!(
                "Unsupported assignment target: {:?}",
                target
            ))),
        }
    }

    /// 处理单条语句
    fn emit_statement(&mut self, stmt: &lir::Statement) -> NativeResult<()> {
        match stmt {
            lir::Statement::Empty => Ok(()),
            lir::Statement::Expression(expr) => {
                // Evaluate and drop result
                self.emit_expression(expr, "_")?;
                Ok(())
            }
            lir::Statement::Variable(var) => {
                // 已经在函数开头分配了局部变量索引
                // 如果有初始化表达式，求值并存入
                if let Some(init) = &var.initializer {
                    let idx = self.local_indices[&var.name];
                    self.emit_expression(init, "temp_init")?;
                    self.emit_local_get_named("temp_init")?;
                    self.emit_local_set_index(idx)?;
                }
                Ok(())
            }
            lir::Statement::If(if_stmt) => {
                let merge = self.wat_label("if_merge");
                let to_else = self.wat_label("if_else");

                self.emit_line(&format!("(block {}", merge))?;
                self.indent += 1;
                self.emit_line(&format!("(block {}", to_else))?;
                self.indent += 1;

                self.emit_expression(&if_stmt.condition, "temp_cond")?;
                self.emit_local_get_named("temp_cond")?;
                self.emit_line("i32.eqz")?;
                self.emit_line(&format!("br_if {}", to_else))?;

                self.emit_statement(&*if_stmt.then_branch)?;
                self.emit_line(&format!("br {}", merge))?;

                self.indent -= 1;
                self.emit_line(")")?;

                self.indent += 1;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.emit_statement(&**else_branch)?;
                }
                self.indent -= 1;

                self.indent -= 1;
                self.emit_line(")")?;
                Ok(())
            }
            lir::Statement::For(for_stmt) => {
                let brk = self.wat_label("for_brk");
                let lp = self.wat_label("for_lp");
                let cont = self.wat_label("for_cont");

                self.loop_labels.push((cont.clone(), brk.clone()));

                if let Some(init) = &for_stmt.initializer {
                    self.emit_statement(init)?;
                }

                self.emit_line(&format!("(block {}", brk))?;
                self.indent += 1;
                self.emit_line(&format!("(loop {}", lp))?;
                self.indent += 1;

                if let Some(cond) = &for_stmt.condition {
                    self.emit_expression(cond, "temp_cond")?;
                    self.emit_local_get_named("temp_cond")?;
                    self.emit_line("i32.eqz")?;
                    self.emit_line(&format!("br_if {}", brk))?;
                }

                self.emit_line(&format!("(block {}", cont))?;
                self.indent += 1;
                self.emit_statement(&*for_stmt.body)?;
                self.indent -= 1;
                self.emit_line(")")?;

                if let Some(inc) = &for_stmt.increment {
                    self.emit_expression(inc, "_")?;
                }

                self.emit_line(&format!("br {}", lp))?;

                self.indent -= 1;
                self.emit_line(")")?;
                self.indent -= 1;
                self.emit_line(")")?;

                self.loop_labels.pop();
                Ok(())
            }
            lir::Statement::While(while_stmt) => {
                let brk = self.wat_label("while_brk");
                let lp = self.wat_label("while_lp");
                let cont = self.wat_label("while_cont");

                self.loop_labels.push((cont.clone(), brk.clone()));

                self.emit_line(&format!("(block {}", brk))?;
                self.indent += 1;
                self.emit_line(&format!("(loop {}", lp))?;
                self.indent += 1;

                self.emit_expression(&while_stmt.condition, "temp_cond")?;
                self.emit_local_get_named("temp_cond")?;
                self.emit_line("i32.eqz")?;
                self.emit_line(&format!("br_if {}", brk))?;

                self.emit_line(&format!("(block {}", cont))?;
                self.indent += 1;
                self.emit_statement(&*while_stmt.body)?;
                self.indent -= 1;
                self.emit_line(")")?;

                self.emit_line(&format!("br {}", lp))?;

                self.indent -= 1;
                self.emit_line(")")?;
                self.indent -= 1;
                self.emit_line(")")?;

                self.loop_labels.pop();
                Ok(())
            }
            lir::Statement::DoWhile(do_while) => {
                let brk = self.wat_label("do_brk");
                let lp = self.wat_label("do_lp");
                let body_exit = self.wat_label("do_after_body");

                // continue：跳到条件再判；break：跳出最外层 block
                self.loop_labels.push((body_exit.clone(), brk.clone()));

                self.emit_line(&format!("(block {}", brk))?;
                self.indent += 1;
                self.emit_line(&format!("(loop {}", lp))?;
                self.indent += 1;

                self.emit_line(&format!("(block {}", body_exit))?;
                self.indent += 1;
                self.emit_statement(&*do_while.body)?;
                self.indent -= 1;
                self.emit_line(")")?;

                self.emit_expression(&do_while.condition, "temp_cond")?;
                self.emit_local_get_named("temp_cond")?;
                self.emit_line(&format!("br_if {}", lp))?;

                self.indent -= 1;
                self.emit_line(")")?;
                self.indent -= 1;
                self.emit_line(")")?;

                self.loop_labels.pop();
                Ok(())
            }
            lir::Statement::Break => {
                if let Some((_continue_label, break_label)) = self.loop_labels.last() {
                    self.emit_line(&format!("br {}", break_label))?;
                } else {
                    self.emit_line(";; TODO: break outside loop")?;
                }
                Ok(())
            }
            lir::Statement::Continue => {
                // continue jumps to the current loop's continue target (condition check / loop start)
                if let Some((continue_label, _break_label)) = self.loop_labels.last() {
                    self.emit_line(&format!("br {}", continue_label))?;
                } else {
                    self.emit_line(";; TODO: continue outside loop")?;
                }
                Ok(())
            }
            lir::Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.emit_expression(expr, "ret_val")?;
                    self.emit_local_get_named("ret_val")?;
                }
                self.emit_line("return")?;
                Ok(())
            }
            lir::Statement::Switch(_) => {
                self.emit_line(";; TODO: switch statement")?;
                Ok(())
            }
            lir::Statement::Match(_) => {
                self.emit_line(";; TODO: match statement")?;
                Ok(())
            }
            lir::Statement::Try(_) => {
                self.emit_line(";; TODO: try statement")?;
                Ok(())
            }
            lir::Statement::Goto(label) => {
                let target = if label.starts_with('$') {
                    label.clone()
                } else {
                    format!("${}", label)
                };
                self.emit_line(&format!("br {}", target))?;
                Ok(())
            }
            lir::Statement::Label(label) => {
                self.emit_line(&format!(
                    ";; label `{}` — Wasm 需结构化控制流，裸标号无法作为 br 目标",
                    label
                ))?;
                Ok(())
            }
            lir::Statement::Declaration(_) => Ok(()),
            lir::Statement::Compound(block) => {
                self.indent += 1;
                for stmt in &block.statements {
                    self.emit_statement(stmt)?;
                }
                self.indent -= 1;
                Ok(())
            }
        }
    }

    /// 收集字符串字面量
    fn collect_string_literals(&mut self, expr: &lir::Expression) -> NativeResult<()> {
        match expr {
            lir::Expression::Literal(lir::Literal::String(s)) => {
                if !self.string_literals.contains_key(s) {
                    let label = format!("str_{}", self.string_literals.len());
                    self.string_literals.insert(s.clone(), label);
                }
            }
            lir::Expression::Literal(_) => {}
            lir::Expression::Unary(_, expr) => self.collect_string_literals(expr)?,
            lir::Expression::Binary(_, left, right) => {
                self.collect_string_literals(left)?;
                self.collect_string_literals(right)?;
            }
            lir::Expression::Ternary(c, t, f) => {
                self.collect_string_literals(c)?;
                self.collect_string_literals(t)?;
                self.collect_string_literals(f)?;
            }
            lir::Expression::Assign(target, source) => {
                self.collect_string_literals(target)?;
                self.collect_string_literals(source)?;
            }
            lir::Expression::AssignOp(_, target, source) => {
                self.collect_string_literals(target)?;
                self.collect_string_literals(source)?;
            }
            lir::Expression::Call(callee, args) => {
                self.collect_string_literals(callee)?;
                for arg in args {
                    self.collect_string_literals(arg)?;
                }
            }
            lir::Expression::Index(base, idx) => {
                self.collect_string_literals(base)?;
                self.collect_string_literals(idx)?;
            }
            lir::Expression::Member(base, _field) => self.collect_string_literals(base)?,
            lir::Expression::PointerMember(base, _field) => self.collect_string_literals(base)?,
            lir::Expression::AddressOf(expr) => self.collect_string_literals(expr)?,
            lir::Expression::Dereference(expr) => self.collect_string_literals(expr)?,
            lir::Expression::Cast(_ty, expr) => self.collect_string_literals(expr)?,
            lir::Expression::Comma(exprs) => {
                for expr in exprs {
                    self.collect_string_literals(expr)?;
                }
            }
            lir::Expression::Parenthesized(expr) => self.collect_string_literals(expr)?,
            lir::Expression::InitializerList(inits) => {
                for init in inits {
                    self.collect_string_literals_init(init)?;
                }
            }
            lir::Expression::CompoundLiteral(_ty, inits) => {
                for init in inits {
                    self.collect_string_literals_init(init)?;
                }
            }
            lir::Expression::SizeOf(_)
            | lir::Expression::SizeOfExpr(_)
            | lir::Expression::AlignOf(_) => {}
            lir::Expression::Variable(_) => {}
        }
        Ok(())
    }

    /// 收集字符串字面量从初始化器
    fn collect_string_literals_init(&mut self, init: &lir::Initializer) -> NativeResult<()> {
        match init {
            lir::Initializer::Expression(e) => self.collect_string_literals(e)?,
            lir::Initializer::List(list) => {
                for i in list {
                    self.collect_string_literals_init(i)?;
                }
            }
            lir::Initializer::Named(_, i) => self.collect_string_literals_init(i)?,
            lir::Initializer::Indexed(idx, i) => {
                self.collect_string_literals(idx)?;
                self.collect_string_literals_init(i)?;
            }
        }
        Ok(())
    }

    fn collect_string_literals_block(&mut self, block: &lir::Block) -> NativeResult<()> {
        for stmt in &block.statements {
            self.collect_string_literals_stmt(stmt)?;
        }
        Ok(())
    }

    fn collect_declaration_strings(&mut self, decl: &lir::Declaration) -> NativeResult<()> {
        match decl {
            lir::Declaration::Function(f) => self.collect_string_literals_block(&f.body),
            lir::Declaration::Global(g) => {
                if let Some(init) = &g.initializer {
                    self.collect_string_literals(init)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn collect_pattern_strings(&mut self, pat: &lir::Pattern) -> NativeResult<()> {
        use lir::{Literal, Pattern};
        match pat {
            Pattern::Literal(Literal::String(s)) => {
                if !self.string_literals.contains_key(s) {
                    let label = format!("str_{}", self.string_literals.len());
                    self.string_literals.insert(s.clone(), label);
                }
                Ok(())
            }
            Pattern::Literal(_) => Ok(()),
            Pattern::Constructor(_, ps) => {
                for p in ps {
                    self.collect_pattern_strings(p)?;
                }
                Ok(())
            }
            Pattern::Tuple(ps) => {
                for p in ps {
                    self.collect_pattern_strings(p)?;
                }
                Ok(())
            }
            Pattern::Record(_, fields) => {
                for (_, p) in fields {
                    self.collect_pattern_strings(p)?;
                }
                Ok(())
            }
            Pattern::Or(a, b) => {
                self.collect_pattern_strings(a)?;
                self.collect_pattern_strings(b)
            }
            Pattern::Wildcard | Pattern::Variable(_) => Ok(()),
        }
    }

    /// 收集字符串字面量从语句
    fn collect_string_literals_stmt(&mut self, stmt: &lir::Statement) -> NativeResult<()> {
        match stmt {
            lir::Statement::Expression(expr) => self.collect_string_literals(expr)?,
            lir::Statement::Declaration(decl) => {
                self.collect_declaration_strings(decl)?;
            }
            lir::Statement::Variable(var) => {
                if let Some(init) = &var.initializer {
                    self.collect_string_literals(init)?;
                }
            }
            lir::Statement::If(if_stmt) => {
                self.collect_string_literals(&if_stmt.condition)?;
                self.collect_string_literals_stmt(&*if_stmt.then_branch)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.collect_string_literals_stmt(&**else_branch)?;
                }
            }
            lir::Statement::For(for_stmt) => {
                if let Some(init) = &for_stmt.initializer {
                    self.collect_string_literals_stmt(init)?;
                }
                if let Some(cond) = &for_stmt.condition {
                    self.collect_string_literals(cond)?;
                }
                self.collect_string_literals_stmt(&*for_stmt.body)?;
                if let Some(inc) = &for_stmt.increment {
                    self.collect_string_literals(inc)?;
                }
            }
            lir::Statement::While(while_stmt) => {
                self.collect_string_literals(&while_stmt.condition)?;
                self.collect_string_literals_stmt(&*while_stmt.body)?;
            }
            lir::Statement::DoWhile(do_while) => {
                self.collect_string_literals(&do_while.condition)?;
                self.collect_string_literals_stmt(&*do_while.body)?;
            }
            lir::Statement::Switch(sw) => {
                self.collect_string_literals(&sw.expression)?;
                for c in &sw.cases {
                    self.collect_string_literals(&c.value)?;
                    self.collect_string_literals_stmt(&c.body)?;
                }
                if let Some(def) = &sw.default {
                    self.collect_string_literals_stmt(def)?;
                }
            }
            lir::Statement::Match(m) => {
                self.collect_string_literals(&m.scrutinee)?;
                for case in &m.cases {
                    self.collect_pattern_strings(&case.pattern)?;
                    if let Some(g) = &case.guard {
                        self.collect_string_literals(g)?;
                    }
                    self.collect_string_literals_block(&case.body)?;
                }
            }
            lir::Statement::Try(t) => {
                self.collect_string_literals_block(&t.body)?;
                for c in &t.catch_clauses {
                    self.collect_string_literals_block(&c.body)?;
                }
                if let Some(fin) = &t.finally_block {
                    self.collect_string_literals_block(fin)?;
                }
            }
            lir::Statement::Compound(block) => {
                for stmt in &block.statements {
                    self.collect_string_literals_stmt(stmt)?;
                }
            }
            lir::Statement::Return(Some(expr)) => self.collect_string_literals(expr)?,
            _ => {}
        }
        Ok(())
    }

    /// 处理一个函数
    fn emit_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        // 不可调用 self.clear()：会清空整个模块输出并重置 indent，破坏外层 generate() 状态。
        self.loop_labels.clear();
        self.current_function = func.name.clone();
        self.local_and_param_types.clear();
        for p in &func.parameters {
            self.local_and_param_types
                .insert(p.name.clone(), p.type_.clone());
        }
        for stmt in &func.body.statements {
            Self::collect_var_types_stmt(stmt, &mut self.local_and_param_types);
        }

        // 收集所有字符串字面量
        for stmt in &func.body.statements {
            self.collect_string_literals_stmt(stmt)?;
        }

        // 收集所有局部变量
        let mut locals = Vec::new();
        for (_i, param) in func.parameters.iter().enumerate() {
            locals.push((param.name.clone(), param.type_.clone()));
        }
        for stmt in &func.body.statements {
            Self::collect_locals(stmt, &mut locals);
        }

        // emit_expression / emit_statement 使用的隐式 i32 槽；WAT 要求预先 `(local $x i32)` 声明。
        // 若用户已声明同名局部变量，则不再追加（极少见；此时生成器仍复用该槽）。
        for i in 0..16 {
            let name = format!("temp{}", i);
            if !locals.iter().any(|(n, _)| n == &name) {
                locals.push((name, lir::Type::Int));
            }
        }
        for name in [
            "temp",
            "temp_base",
            "temp_cond",
            "temp_callee",
            "temp_init",
            "ret_val",
            "t0",
            "t1",
            "_",
        ] {
            if !locals.iter().any(|(n, _)| n == name) {
                locals.push((name.to_string(), lir::Type::Int));
            }
        }

        // 分配局部变量索引
        self.local_indices.clear();
        self.local_count = 0;
        for (name, _ty) in &locals {
            self.local_indices.insert(name.clone(), self.local_count);
            self.local_count += 1;
        }

        // 开始函数定义
        let mut params_wat = String::new();
        for (i, param) in func.parameters.iter().enumerate() {
            let wasm_ty = self.ty_to_wasm(&param.type_);
            if i > 0 {
                params_wat.push(' ');
            }
            params_wat.push_str(&format!("(param ${} {})", param.name, wasm_ty));
        }

        let result_ty = if func.return_type != lir::Type::Void {
            let wasm_ty = self.ty_to_wasm(&func.return_type);
            format!("(result {})", wasm_ty)
        } else {
            String::new()
        };

        self.emit_raw(&format!("(func ${} {} {}", func.name, params_wat, result_ty).trim_end())?;

        // 声明局部变量
        self.indent += 1;
        for (name, ty) in &locals {
            if func.parameters.iter().any(|p| &p.name == name) {
                continue; // parameters already declared in header
            }
            let wasm_ty = self.ty_to_wasm(ty);
            self.emit_line(&format!("(local ${} {})", name, wasm_ty))?;
        }

        // 生成函数体
        self.indent += 1;
        for stmt in &func.body.statements {
            self.emit_statement(stmt)?;
        }
        self.indent -= 1;

        // 如果没有返回语句且非void返回，隐式返回0
        if func.return_type != lir::Type::Void {
            // TODO: check if function already has return
            self.emit_line("i32.const 0")?;
            self.emit_line("return")?;
        }

        self.indent -= 1;
        self.emit_raw(")")?;

        // Export main function
        if func.name == "main" {
            self.emit_line(&format!("(export \"main\" (func ${}))", func.name))?;
        }

        Ok(())
    }
}

impl AssemblyGenerator for Wasm32AssemblyGenerator {
    fn generate(&mut self, lir: &lir::Program) -> NativeResult<String> {
        // Start module
        self.output.clear();
        self.string_literals.clear();
        self.string_memory_offsets.clear();
        self.global_memory_offsets.clear();
        self.globals.clear();
        // 每次生成独立模块：避免复用生成器时 struct 字段偏移、标号计数污染下一次输出
        self.field_offsets.clear();
        self.local_and_param_types.clear();
        self.loop_labels.clear();
        self.label_counter = 0;
        self.indent = 0;
        self.emit_raw("(module")?;
        self.indent += 1;

        // TODO: Type section for indirect calls
        self.emit_line(";; Type definitions for indirect calls")?;
        self.emit_line("(type $func_ty (func))")?;
        self.emit_line("")?;

        // Add memory
        self.emit_line(";; Memory")?;
        self.emit_line("(memory 1)")?;
        self.emit_line("(export \"memory\" (memory 0))")?;
        self.emit_line("")?;

        // First pass: collect all string literals from functions
        for decl in &lir.declarations {
            if let lir::Declaration::Function(func) = decl {
                for stmt in &func.body.statements {
                    self.collect_string_literals_stmt(stmt)?;
                }
            }
        }

        // Calculate struct field offsets with proper alignment
        for decl in &lir.declarations {
            if let lir::Declaration::Struct(struct_decl) = decl {
                let mut current_offset = 0;
                let mut max_align = 1;

                for field in &struct_decl.fields {
                    let align = self.align_of_ty(&field.type_);
                    max_align = max_align.max(align);

                    // Align current offset to field's required alignment
                    current_offset = if current_offset % align == 0 {
                        current_offset
                    } else {
                        current_offset + align - (current_offset % align)
                    };

                    self.field_offsets.insert(
                        Self::layout_key(&struct_decl.name, &field.name),
                        current_offset,
                    );
                    current_offset += self.size_of_ty(&field.type_);
                }

                let _ = current_offset.next_multiple_of(max_align.max(1));
            }
            // Also handle class declarations
            if let lir::Declaration::Class(class_decl) = decl {
                let mut current_offset = 0;
                let mut max_align = 1;

                for field in &class_decl.fields {
                    let align = self.align_of_ty(&field.type_);
                    max_align = max_align.max(align);

                    // Align current offset to field's required alignment
                    current_offset = if current_offset % align == 0 {
                        current_offset
                    } else {
                        current_offset + align - (current_offset % align)
                    };

                    self.field_offsets.insert(
                        Self::layout_key(&class_decl.name, &field.name),
                        current_offset,
                    );
                    current_offset += self.size_of_ty(&field.type_);
                }

                let _ = current_offset.next_multiple_of(max_align.max(1));
            }
        }

        // 全局变量元数据（线性内存中的静态区域）
        for decl in &lir.declarations {
            if let lir::Declaration::Global(global) = decl {
                let size = self.size_of_ty(&global.type_);
                self.globals.insert(
                    global.name.clone(),
                    GlobalInfo {
                        size,
                        initialized: global.initializer.is_some(),
                        align: self.align_of_ty(&global.type_),
                        initializer: global.initializer.clone(),
                    },
                );
            }
        }

        // 字符串与全局数据：使用递增偏移，避免多个 `(data (i32.const 0) ...)` 互相覆盖
        let mut data_offset: usize = 0;
        if !self.string_literals.is_empty() {
            let mut strings: Vec<(String, String)> = self
                .string_literals
                .iter()
                .map(|(s, label)| (s.clone(), label.clone()))
                .collect();
            strings.sort_by(|a, b| a.1.cmp(&b.1));
            for (s, _label) in strings {
                self.string_memory_offsets.insert(s.clone(), data_offset);
                let payload = Self::wat_escape_data_string(&s);
                self.emit_line(&format!(
                    ";; String @{}: {}",
                    data_offset,
                    s.escape_default()
                ))?;
                self.emit_line(&format!(
                    "(data (i32.const {}) \"{}\")",
                    data_offset, payload
                ))?;
                data_offset += s.len();
                data_offset = data_offset.next_multiple_of(4);
            }
            self.emit_line("")?;
        }

        if !self.globals.is_empty() {
            let mut globals: Vec<(String, GlobalInfo)> = self
                .globals
                .iter()
                .map(|(n, i)| (n.clone(), i.clone()))
                .collect();
            globals.sort_by(|a, b| a.0.cmp(&b.0));
            for (name, info) in globals {
                data_offset = data_offset.next_multiple_of(info.align.max(1));
                self.global_memory_offsets.insert(name.clone(), data_offset);
                self.emit_line(&format!(
                    ";; Global `{}` @{} size {}",
                    name, data_offset, info.size
                ))?;
                let payload = Self::wat_zero_bytes(info.size);
                self.emit_line(&format!(
                    "(data (i32.const {}) \"{}\")",
                    data_offset, payload
                ))?;
                data_offset += info.size;
            }
            self.emit_line("")?;
        }

        // Generate all functions
        for decl in &lir.declarations {
            if let lir::Declaration::Function(func) = decl {
                self.emit_function(func)?;
                self.emit_line("")?;
            }
        }

        // End module
        self.indent -= 1;
        self.emit_raw(")")?;

        Ok(std::mem::take(&mut self.output))
    }

    fn arch(&self) -> TargetArch {
        TargetArch::Wasm32
    }
}
