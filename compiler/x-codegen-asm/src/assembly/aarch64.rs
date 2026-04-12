//! AArch64 汇编生成器
//!
//! 支持 GNU 汇编语法（Linux/macOS/Windows）。
//!
//! # 支持的 ABI
//!
//! - System V AAPCS ABI (Linux)
//! - Apple ABI (macOS)
//! - Microsoft ARM64 ABI (Windows)
//!
//! # 示例输出
//!
//! ```asm
//! .global main
//! main:
//!     stp x29, x30, [sp, #-16]!
//!     mov x29, sp
//!     sub sp, sp, #32
//!     ; ... function body ...
//!     mov sp, x29
//!     ldp x29, x30, [sp], #16
//!     ret
//! ```

use std::collections::HashMap;
use std::fmt::Write;

use crate::{NativeError, NativeResult, TargetOS};
use x_lir as lir;

use super::{AssemblyGenerator, GlobalInfo};

/// AArch64 汇编生成器
pub struct AArch64AssemblyGenerator {
    /// 目标操作系统
    os: TargetOS,
    /// 输出缓冲区
    output: String,
    /// 当前缩进级别
    indent: usize,
    /// 标签计数器
    label_counter: usize,
    /// 字符串字面量表
    string_literals: HashMap<String, String>,
    /// 全局变量表
    globals: HashMap<String, GlobalInfo>,
    /// 全局变量声明顺序（保持顺序很重要）
    global_order: Vec<String>,
    /// 局部变量栈偏移
    local_offsets: HashMap<String, i32>,
    /// 当前栈帧大小
    stack_size: usize,
    /// 当前函数名
    current_function: String,
    /// 循环标签栈 - (continue_label, break_label) for each nested loop
    loop_labels: Vec<(String, String)>,
    /// 字段偏移：`StructName::field` -> 字节偏移
    field_offsets: HashMap<String, usize>,
    /// 当前函数参数与局部变量的静态类型（解析 Member / PointerMember）
    local_and_param_types: HashMap<String, lir::Type>,
}

impl AArch64AssemblyGenerator {
    /// 创建新的 AArch64 汇编生成器
    pub fn new(os: TargetOS) -> Self {
        Self {
            os,
            output: String::new(),
            indent: 0,
            label_counter: 0,
            string_literals: HashMap::new(),
            globals: HashMap::new(),
            global_order: Vec::new(),
            local_offsets: HashMap::new(),
            stack_size: 0,
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
        self.globals.clear();
        self.global_order.clear();
        self.local_offsets.clear();
        self.stack_size = 0;
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

    /// 输出一行汇编
    fn emit_line(&mut self, line: &str) -> NativeResult<()> {
        writeln!(self.output, "{}{}", "    ".repeat(self.indent), line)?;
        Ok(())
    }

    /// 输出原始文本（无缩进）
    fn emit_raw(&mut self, text: &str) -> NativeResult<()> {
        writeln!(self.output, "{}", text)?;
        Ok(())
    }

    /// 外部 C / 系统库符号名（macOS 需前导 `_`；部分 X 内建映射到 libc）
    fn extern_branch_target(&self, name: &str) -> String {
        let mapped = match name {
            // X 运行时常见内建：映射到 libSystem / libc，便于裸汇编链接
            // println 和 print 都使用 printf，以便支持整数和字符串参数
            "println" => "printf",
            "print" => "printf",
            _ => name,
        };
        match self.os {
            TargetOS::MacOS => format!("_{}", mapped),
            _ => mapped.to_string(),
        }
    }

    /// 生成内存操作数语法
    fn mem_operand(&self, base: &str, offset: i32) -> String {
        if offset == 0 {
            format!("[{}]", base)
        } else if offset & 0xF == 0 {
            format!("[{}, #{:#x}]", base, offset)
        } else {
            format!("[{}, #{}]", base, offset)
        }
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
            lir::Type::LongDouble => 8,
            lir::Type::Size => 8,
            lir::Type::Ptrdiff => 8,
            lir::Type::Intptr => 8,
            lir::Type::Uintptr => 8,
            lir::Type::Pointer(_) => 8,
            lir::Type::Array(ty, len) => {
                let len = len.unwrap_or(1);
                self.size_of_ty(ty) * (len as usize)
            }
            lir::Type::FunctionPointer(_, _) => 8,
            lir::Type::Named(_) => 8,
            lir::Type::Qualified(_, ty) => self.size_of_ty(ty),
        }
    }

    fn layout_key(struct_name: &str, field: &str) -> String {
        format!("{}::{}", struct_name, field)
    }

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

    fn peel_qualified_ty(ty: &lir::Type) -> &lir::Type {
        match ty {
            lir::Type::Qualified(_, inner) => Self::peel_qualified_ty(inner),
            t => t,
        }
    }

    fn struct_name_from_pointer_type(ty: &lir::Type) -> Option<String> {
        let ty = Self::peel_qualified_ty(ty);
        match ty {
            lir::Type::Pointer(inner) => {
                let inner = Self::peel_qualified_ty(inner);
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
        let ty = Self::peel_qualified_ty(ty);
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

    /// 将整数加载到寄存器
    fn emit_load_immediate(&mut self, value: i64, reg: &str) -> NativeResult<()> {
        if value >= 0 && value <= 0xFFFF {
            self.emit_line(&format!("mov {}, #{}", reg, value))?;
        } else {
            // 对于大立即数，需要多步加载
            let low = value & 0xFFFF;
            let high = (value >> 16) & 0xFFFF;
            if high != 0 {
                self.emit_line(&format!("movz {}, #{}", reg, low))?;
                self.emit_line(&format!("movk {}, #{} lsl 16", reg, high))?;
                let high2 = (value >> 32) & 0xFFFF;
                if high2 != 0 {
                    self.emit_line(&format!("movk {}, #{} lsl 32", reg, high2))?;
                }
                let high3 = (value >> 48) & 0xFFFF;
                if high3 != 0 {
                    self.emit_line(&format!("movk {}, #{} lsl 48", reg, high3))?;
                }
            } else {
                self.emit_line(&format!("mov {}, #{}", reg, low))?;
            }
        }
        Ok(())
    }

    fn count_flat_initializer_slots(init: &lir::Initializer) -> usize {
        match init {
            lir::Initializer::Expression(_) => 1,
            lir::Initializer::List(list) => {
                list.iter().map(Self::count_flat_initializer_slots).sum()
            }
            lir::Initializer::Named(_, inner) => Self::count_flat_initializer_slots(inner),
            lir::Initializer::Indexed(_, inner) => Self::count_flat_initializer_slots(inner),
        }
    }

    fn count_flat_initializer_list_slots(items: &[lir::Initializer]) -> usize {
        items.iter().map(Self::count_flat_initializer_slots).sum()
    }

    fn emit_flat_initializer_on_stack_aarch64(
        &mut self,
        init: &lir::Initializer,
        slot: &mut usize,
    ) -> NativeResult<()> {
        match init {
            lir::Initializer::Expression(e) => {
                self.emit_expression(e, "x9")?;
                self.emit_line(&format!("str x9, [sp, #{}]", *slot * 8))?;
                *slot += 1;
            }
            lir::Initializer::List(list) => {
                for i in list {
                    self.emit_flat_initializer_on_stack_aarch64(i, slot)?;
                }
            }
            lir::Initializer::Named(_, inner) => {
                self.emit_flat_initializer_on_stack_aarch64(inner, slot)?;
            }
            lir::Initializer::Indexed(idx, inner) => {
                self.emit_expression(idx, "x9")?;
                self.emit_flat_initializer_on_stack_aarch64(inner, slot)?;
            }
        }
        Ok(())
    }

    fn emit_initializer_list_on_stack_aarch64(
        &mut self,
        items: &[lir::Initializer],
        result_reg: &str,
    ) -> NativeResult<()> {
        let num_slots = Self::count_flat_initializer_list_slots(items);
        let size_bytes = (num_slots * 8).next_multiple_of(16);
        self.emit_line(&format!("sub sp, sp, #{}", size_bytes))?;
        let mut slot = 0usize;
        for item in items {
            self.emit_flat_initializer_on_stack_aarch64(item, &mut slot)?;
        }
        debug_assert_eq!(slot, num_slots);
        self.emit_line(&format!("mov {}, sp", result_reg))?;
        Ok(())
    }

    /// 处理 LIR 表达式，将结果放入指定寄存器
    fn emit_expression(&mut self, expr: &lir::Expression, result_reg: &str) -> NativeResult<()> {
        match expr {
            lir::Expression::Literal(lit) => {
                match lit {
                    lir::Literal::Bool(b) => {
                        self.emit_line(&format!(
                            "mov {}, #{}",
                            result_reg,
                            if *b { 1 } else { 0 }
                        ))?;
                    }
                    lir::Literal::Integer(i) => {
                        self.emit_load_immediate(*i, result_reg)?;
                    }
                    lir::Literal::UnsignedInteger(u) => {
                        self.emit_load_immediate(*u as i64, result_reg)?;
                    }
                    lir::Literal::Long(i) => {
                        self.emit_load_immediate(*i, result_reg)?;
                    }
                    lir::Literal::UnsignedLong(u) => {
                        self.emit_load_immediate(*u as i64, result_reg)?;
                    }
                    lir::Literal::LongLong(i) => {
                        self.emit_load_immediate(*i, result_reg)?;
                    }
                    lir::Literal::UnsignedLongLong(u) => {
                        self.emit_load_immediate(*u as i64, result_reg)?;
                    }
                    lir::Literal::Float(f) => {
                        // TODO: 加载浮点数到 SIMD 寄存器
                        self.emit_line(&format!("// TODO: float literal {}", f))?;
                    }
                    lir::Literal::Double(f) => {
                        // TODO: 加载浮点数到 SIMD 寄存器
                        self.emit_line(&format!("// TODO: double literal {}", f))?;
                    }
                    lir::Literal::Char(c) => {
                        self.emit_line(&format!("mov {}, #{}", result_reg, *c as i64))?;
                    }
                    lir::Literal::String(s) => {
                        // 字符串字面量存在 rodata，将地址加载到寄存器
                        let label = self.string_literals.get(s).cloned().ok_or_else(|| {
                            NativeError::CodegenError(format!("String literal not found: {}", s))
                        })?;
                        match self.os {
                            TargetOS::MacOS => {
                                // 本地 .cstring 符号：用 PAGE/PAGEOFF（:got: 仅适用于经动态链接的符号）
                                self.emit_line(&format!("adrp {}, {}@PAGE", result_reg, label))?;
                                self.emit_line(&format!(
                                    "add {}, {}, {}@PAGEOFF",
                                    result_reg, result_reg, label
                                ))?;
                            }
                            _ => {
                                self.emit_line(&format!("ldr {}, ={}", result_reg, label))?;
                            }
                        }
                    }
                    lir::Literal::NullPointer => {
                        self.emit_line(&format!("mov {}, #0", result_reg))?;
                    }
                }
                Ok(())
            }
            lir::Expression::Variable(name) => {
                // 首先检查局部变量
                if let Some(offset) = self.local_offsets.get(name) {
                    self.emit_line(&format!(
                        "ldr {}, {}",
                        result_reg,
                        self.mem_operand("x29", *offset)
                    ))?;
                } else if self.globals.contains_key(name) {
                    // 全局符号：先取地址，再从内存加载（与 x86_64 的 mov rax, [sym] 对应）
                    let sym = name.as_str();
                    match self.os {
                        TargetOS::MacOS => {
                            self.emit_line(&format!("adrp {}, {}@PAGE", result_reg, sym))?;
                            self.emit_line(&format!(
                                "add {}, {}, {}@PAGEOFF",
                                result_reg, result_reg, sym
                            ))?;
                            self.emit_line(&format!("ldr {}, [{}]", result_reg, result_reg))?;
                        }
                        _ => {
                            self.emit_line(&format!("ldr {}, ={}", result_reg, sym))?;
                            self.emit_line(&format!("ldr {}, [{}]", result_reg, result_reg))?;
                        }
                    }
                } else {
                    return Err(NativeError::CodegenError(format!(
                        "Variable not found: {}",
                        name
                    )));
                }
                Ok(())
            }
            lir::Expression::Member(base, field) => {
                self.emit_expression(base, result_reg)?;
                let offset = self.resolve_field_offset(base, field, false).unwrap_or(0);
                if offset > 0 {
                    self.emit_line(&format!("add {}, {}, #{}", result_reg, result_reg, offset))?;
                }
                self.emit_line(&format!("ldr {}, [{}]", result_reg, result_reg))?;
                Ok(())
            }
            lir::Expression::PointerMember(base, field) => {
                self.emit_expression(base, result_reg)?;
                self.emit_line(&format!("ldr {}, [{}]", result_reg, result_reg))?;
                let offset = self.resolve_field_offset(base, field, true).unwrap_or(0);
                if offset > 0 {
                    self.emit_line(&format!("add {}, {}, #{}", result_reg, result_reg, offset))?;
                }
                self.emit_line(&format!("ldr {}, [{}]", result_reg, result_reg))?;
                Ok(())
            }
            lir::Expression::Dereference(ptr) => {
                self.emit_expression(ptr, result_reg)?;
                self.emit_line(&format!("ldr {}, [{}]", result_reg, result_reg))?;
                Ok(())
            }
            lir::Expression::AddressOf(expr) => {
                match expr.as_ref() {
                    lir::Expression::Variable(name) => {
                        let offset = self.local_offsets.get(name).ok_or_else(|| {
                            NativeError::CodegenError(format!("Variable not found: {}", name))
                        })?;
                        // 栈上偏移多为负数：AArch64 的 add 立即数非负，用 sub #|off|
                        if *offset >= 0 {
                            self.emit_line(&format!("add {}, x29, #{}", result_reg, offset))?;
                        } else {
                            self.emit_line(&format!("sub {}, x29, #{}", result_reg, -offset))?;
                        }
                    }
                    lir::Expression::Member(base, field) => {
                        self.emit_expression(base, result_reg)?;
                        let offset = self.resolve_field_offset(base, field, false).unwrap_or(0);
                        if offset > 0 {
                            self.emit_line(&format!(
                                "add {}, {}, #{}",
                                result_reg, result_reg, offset
                            ))?;
                        }
                    }
                    lir::Expression::PointerMember(base, field) => {
                        self.emit_expression(base, result_reg)?;
                        self.emit_line(&format!("ldr {}, [{}]", result_reg, result_reg))?;
                        let offset = self.resolve_field_offset(base, field, true).unwrap_or(0);
                        if offset > 0 {
                            self.emit_line(&format!(
                                "add {}, {}, #{}",
                                result_reg, result_reg, offset
                            ))?;
                        }
                    }
                    _ => {
                        // TODO: 处理其他情况
                        self.emit_expression(expr, result_reg)?;
                    }
                }
                Ok(())
            }
            lir::Expression::Unary(op, operand) => {
                self.emit_expression(operand, "x9")?;
                match op {
                    lir::UnaryOp::Minus => {
                        self.emit_line(&format!("neg {}, x9", result_reg))?;
                    }
                    lir::UnaryOp::BitNot => {
                        self.emit_line(&format!("mvn {}, x9", result_reg))?;
                    }
                    lir::UnaryOp::Not => {
                        // Logical not - 结果为 0 或 1
                        self.emit_line("cmp x9, #0")?;
                        self.emit_line(&format!("cset {}, eq", result_reg))?;
                    }
                    _ => {
                        // TODO: increment/decrement
                        self.emit_line(&format!("// TODO: unary op {:?}", op))?;
                    }
                }
                Ok(())
            }
            lir::Expression::Binary(op, left, right) => {
                // 先计算左操作数到 x9，右操作数到 x10
                self.emit_expression(left, "x9")?;
                self.emit_expression(right, "x10")?;

                let need_cset = matches!(
                    op,
                    lir::BinaryOp::LessThan
                        | lir::BinaryOp::LessThanEqual
                        | lir::BinaryOp::GreaterThan
                        | lir::BinaryOp::GreaterThanEqual
                        | lir::BinaryOp::Equal
                        | lir::BinaryOp::NotEqual
                );

                let op_name = match op {
                    lir::BinaryOp::Add => "add",
                    lir::BinaryOp::Subtract => "sub",
                    lir::BinaryOp::Multiply => "mul",
                    lir::BinaryOp::Divide => "sdiv",
                    lir::BinaryOp::Modulo => "srem",
                    lir::BinaryOp::LeftShift => "lsl",
                    lir::BinaryOp::RightShift => "lsr",
                    lir::BinaryOp::RightShiftArithmetic => "asr",
                    lir::BinaryOp::BitAnd => "and",
                    lir::BinaryOp::BitXor => "eor",
                    lir::BinaryOp::BitOr => "orr",
                    lir::BinaryOp::LessThan => "cmp",
                    lir::BinaryOp::LessThanEqual => "cmp",
                    lir::BinaryOp::GreaterThan => "cmp",
                    lir::BinaryOp::GreaterThanEqual => "cmp",
                    lir::BinaryOp::Equal => "cmp",
                    lir::BinaryOp::NotEqual => "cmp",
                    _ => "// TODO",
                };

                if !need_cset {
                    self.emit_line(&format!("{} {}, x9, x10", op_name, result_reg))?;
                } else {
                    self.emit_line(&format!("{} x9, x10", op_name))?;
                    let cset_cc = match op {
                        lir::BinaryOp::Equal => "eq",
                        lir::BinaryOp::NotEqual => "ne",
                        lir::BinaryOp::LessThan => "lt",
                        lir::BinaryOp::LessThanEqual => "le",
                        lir::BinaryOp::GreaterThan => "gt",
                        lir::BinaryOp::GreaterThanEqual => "ge",
                        _ => unreachable!(),
                    };
                    self.emit_line(&format!("cset {}, {}", result_reg, cset_cc))?;
                }

                Ok(())
            }
            lir::Expression::Call(callee, args) => {
                // AAPCS: X0-X7 用于参数，结果在 X0
                // 检测是否是 println/print 调用，并根据参数类型添加格式字符串
                let needs_format_string = match callee.as_ref() {
                    lir::Expression::Variable(name) => {
                        (name == "println" || name == "print") && !args.is_empty()
                    }
                    _ => false,
                };

                // 如果是 println/print，检测第一个参数是否为字符串字面量
                let is_string_arg = if needs_format_string {
                    matches!(
                        args.first(),
                        Some(lir::Expression::Literal(lir::Literal::String(_)))
                    )
                } else {
                    false
                };

                // 计算参数偏移：如果是 println 且参数不是字符串，需要在 x0 放置格式字符串
                let mut arg_offset = 0;

                // 如果是 println 且参数不是字符串，生成格式字符串
                if needs_format_string && !is_string_arg {
                    // 生成格式字符串标签
                    let fmt_str = "%d\\n";
                    let label = if !self.string_literals.contains_key(fmt_str) {
                        let l = format!("LC{}", self.string_literals.len());
                        self.string_literals.insert(fmt_str.to_string(), l.clone());
                        l
                    } else {
                        self.string_literals
                            .get(fmt_str)
                            .cloned()
                            .unwrap_or_default()
                    };

                    // 将格式字符串地址加载到 x0（第一个参数位置）
                    match self.os {
                        TargetOS::MacOS => {
                            self.emit_line(&format!("adrp x0, {}@PAGE", label))?;
                            self.emit_line(&format!("add x0, x0, {}@PAGEOFF", label))?;
                        }
                        _ => {
                            self.emit_line(&format!("ldr x0, ={}", label))?;
                        }
                    }
                    arg_offset = 1;
                }

                // 先将参数加载到寄存器（从正确的偏移开始）
                for (i, arg) in args.iter().enumerate() {
                    let reg_idx = i + arg_offset;
                    if reg_idx < 8 {
                        let arg_reg = format!("x{}", reg_idx);
                        self.emit_expression(arg, &arg_reg)?;
                    } else {
                        // TODO: 栈参数
                        self.emit_line(&format!("// TODO: parameter {} on stack", i))?;
                    }
                }

                // 调用函数：直接调用或间接调用
                match callee.as_ref() {
                    lir::Expression::Variable(name) => {
                        let target = self.extern_branch_target(name);
                        self.emit_line(&format!("bl {}", target))?;
                    }
                    _ => {
                        // 间接调用，函数指针在表达式结果中
                        self.emit_expression(callee, "x9")?;
                        self.emit_line("blr x9")?;
                    }
                }

                // 结果已经在 x0，移动到目标寄存器
                if result_reg != "x0" {
                    self.emit_line(&format!("mov {}, x0", result_reg))?;
                }
                Ok(())
            }
            lir::Expression::Index(base, index) => {
                self.emit_expression(base, "x10")?;
                self.emit_expression(index, "x11")?;
                // Scale by 8 bytes (assuming 64-bit)
                self.emit_line("lsl x11, x11, #3")?;
                self.emit_line("add x10, x10, x11")?;
                self.emit_line(&format!("ldr {}, [x10]", result_reg))?;
                Ok(())
            }
            lir::Expression::Assign(target, value) => {
                self.emit_assign(target, value)?;
                // 对于函数调用（result_reg 可能是 x0），不要移动 x9 的值
                // 因为函数调用的结果会在 x0 中返回
                if result_reg != "x9" && result_reg != "x0" {
                    self.emit_line(&format!("mov {}, x9", result_reg))?;
                }
                Ok(())
            }
            lir::Expression::AssignOp(op, target, value) => {
                self.emit_expression(target, "x10")?;
                self.emit_expression(value, "x11")?;
                match op {
                    lir::BinaryOp::Add => self.emit_line("add x9, x10, x11")?,
                    lir::BinaryOp::Subtract => self.emit_line("sub x9, x10, x11")?,
                    lir::BinaryOp::Multiply => self.emit_line("mul x9, x10, x11")?,
                    lir::BinaryOp::Divide => {
                        self.emit_line("sdiv x9, x10, x11")?;
                    }
                    lir::BinaryOp::Modulo => {
                        self.emit_line("sdiv x12, x10, x11")?;
                        self.emit_line("msub x9, x12, x11, x10")?;
                    }
                    lir::BinaryOp::BitAnd => self.emit_line("and x9, x10, x11")?,
                    lir::BinaryOp::BitOr => self.emit_line("orr x9, x10, x11")?,
                    lir::BinaryOp::BitXor => self.emit_line("eor x9, x10, x11")?,
                    lir::BinaryOp::LeftShift => self.emit_line("lsl x9, x10, x11")?,
                    lir::BinaryOp::RightShift => self.emit_line("lsr x9, x10, x11")?,
                    lir::BinaryOp::RightShiftArithmetic => self.emit_line("asr x9, x10, x11")?,
                    _ => {
                        return Err(NativeError::CodegenError(format!(
                            "Unsupported compound assignment operator: {:?}",
                            op
                        )));
                    }
                }
                self.store_lvalue(target)?;
                if result_reg != "x9" {
                    self.emit_line(&format!("mov {}, x9", result_reg))?;
                }
                Ok(())
            }
            lir::Expression::Cast(_ty, expr) => {
                // AArch64 大部分 cast 直接通过寄存器传递即可
                self.emit_expression(expr, result_reg)?;
                Ok(())
            }
            lir::Expression::SizeOf(ty) => {
                let size = self.size_of_ty(ty);
                self.emit_line(&format!("mov {}, #{}", result_reg, size))?;
                Ok(())
            }
            lir::Expression::SizeOfExpr(_expr) => {
                // TODO: 计算表达式类型大小
                self.emit_line(&format!("mov {}, #8", result_reg))?;
                Ok(())
            }
            lir::Expression::AlignOf(ty) => {
                let align = self.type_align(ty);
                self.emit_line(&format!("mov {}, #{}", result_reg, align))?;
                Ok(())
            }
            lir::Expression::Ternary(cond, then, else_) => {
                let else_label = self.new_label("ternary_else");
                let end_label = self.new_label("ternary_end");

                self.emit_expression(cond, result_reg)?;
                self.emit_line(&format!("cmp {}, #0", result_reg))?;
                self.emit_line(&format!("b.eq {}", else_label))?;

                self.emit_expression(then, result_reg)?;
                self.emit_line(&format!("b {}", end_label))?;

                self.emit_raw(&format!("{}:", else_label))?;
                self.emit_expression(else_, result_reg)?;

                self.emit_raw(&format!("{}:", end_label))?;
                Ok(())
            }
            lir::Expression::Comma(exprs) => {
                // Comma expression: evaluate all expressions in order,
                // result is the value of the last one
                for (i, expr) in exprs.iter().enumerate() {
                    if i == exprs.len() - 1 {
                        // Last one - result goes to result_reg
                        self.emit_expression(expr, result_reg)?;
                    } else {
                        // Evaluate but discard result
                        self.emit_expression(expr, "x9")?;
                    }
                }
                Ok(())
            }
            lir::Expression::Parenthesized(expr) => {
                self.emit_expression(expr, result_reg)?;
                Ok(())
            }
            lir::Expression::InitializerList(items) => {
                self.emit_initializer_list_on_stack_aarch64(items, result_reg)?;
                Ok(())
            }
            lir::Expression::CompoundLiteral(_, items) => {
                self.emit_initializer_list_on_stack_aarch64(items, result_reg)?;
                Ok(())
            }
        }
    }

    /// 将 x9 中的值存储到赋值目标（与 emit_assign 共用）
    fn store_lvalue(&mut self, target: &lir::Expression) -> NativeResult<()> {
        match target {
            lir::Expression::Variable(name) => {
                // 首先检查局部变量
                if let Some(offset) = self.local_offsets.get(name) {
                    self.emit_line(&format!("str x9, {}", self.mem_operand("x29", *offset)))?;
                    return Ok(());
                }
                // 然后检查全局变量
                if self.globals.contains_key(name) {
                    let sym = name.as_str();
                    match self.os {
                        TargetOS::MacOS => {
                            self.emit_line(&format!("adrp x10, {}@PAGE", sym))?;
                            self.emit_line(&format!("add x10, x10, {}@PAGEOFF", sym))?;
                            self.emit_line("str x9, [x10]")?;
                        }
                        _ => {
                            self.emit_line(&format!("ldr x10, ={}", sym))?;
                            self.emit_line("str x9, [x10]")?;
                        }
                    }
                    return Ok(());
                }
                Err(NativeError::CodegenError(format!(
                    "Variable not found: {}",
                    name
                )))
            }
            lir::Expression::Member(base, field) => {
                self.emit_expression(base, "x10")?;
                let offset = self.resolve_field_offset(base, field, false).unwrap_or(0);
                if offset > 0 {
                    if offset <= 4095 {
                        self.emit_line(&format!("str x9, [x10, #{}, lsl #0]", offset))?;
                    } else {
                        self.emit_line(&format!("add x10, x10, #{}", offset))?;
                        self.emit_line("str x9, [x10]")?;
                    }
                } else {
                    self.emit_line("str x9, [x10]")?;
                }
                Ok(())
            }
            lir::Expression::PointerMember(base, field) => {
                self.emit_expression(base, "x10")?;
                self.emit_line("ldr x10, [x10]")?;
                let offset = self.resolve_field_offset(base, field, true).unwrap_or(0);
                if offset > 0 {
                    if offset <= 4095 {
                        self.emit_line(&format!("str x9, [x10, #{}, lsl #0]", offset))?;
                    } else {
                        self.emit_line(&format!("add x10, x10, #{}", offset))?;
                        self.emit_line("str x9, [x10]")?;
                    }
                } else {
                    self.emit_line("str x9, [x10]")?;
                }
                Ok(())
            }
            lir::Expression::Dereference(ptr) => {
                self.emit_expression(ptr, "x10")?;
                self.emit_line("str x9, [x10]")?;
                Ok(())
            }
            lir::Expression::Index(base, index) => {
                self.emit_expression(base, "x10")?;
                self.emit_expression(index, "x11")?;
                // Scale by 8 bytes (assuming 64-bit)
                self.emit_line("lsl x11, x11, #3")?;
                self.emit_line("add x10, x10, x11")?;
                self.emit_line("str x9, [x10]")?;
                Ok(())
            }
            _ => Err(NativeError::CodegenError(format!(
                "Unsupported assignment target: {:?}",
                target
            ))),
        }
    }

    /// 求值 `source` 并写入 `target`（对应 LIR 的 `Expression::Assign` 等）
    fn emit_assign(
        &mut self,
        target: &lir::Expression,
        source: &lir::Expression,
    ) -> NativeResult<()> {
        self.emit_expression(source, "x9")?;
        self.store_lvalue(target)
    }

    /// 处理单条语句
    fn emit_statement(&mut self, stmt: &lir::Statement) -> NativeResult<()> {
        match stmt {
            lir::Statement::Empty => Ok(()),
            lir::Statement::Expression(expr) => {
                // 计算表达式但丢弃结果
                // 对于 main 函数中的函数调用，将结果放到 x9 而不是 x0，避免覆盖返回值
                if self.current_function == "main" {
                    if matches!(expr, lir::Expression::Call(_, _)) {
                        self.emit_expression(expr, "x9")?;
                    } else {
                        self.emit_expression(expr, "x0")?;
                    }
                } else {
                    self.emit_expression(expr, "x0")?;
                }
                Ok(())
            }
            lir::Statement::Variable(var) => {
                // 局部变量声明已经在函数序言处理过
                // 如果有初始化表达式，对它求值并存储
                if let Some(init) = &var.initializer {
                    let offset = self.local_offsets[&var.name];
                    self.emit_expression(init, "x9")?;
                    self.emit_line(&format!("str x9, {}", self.mem_operand("x29", offset)))?;
                }
                Ok(())
            }
            lir::Statement::If(if_stmt) => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("if_end");

                self.emit_expression(&if_stmt.condition, "x9")?;
                self.emit_line("cmp x9, #0")?;
                self.emit_line(&format!("b.eq {}", else_label))?;

                self.indent += 1;
                self.emit_statement(&*if_stmt.then_branch)?;
                self.indent -= 1;

                self.emit_line(&format!("b {}", end_label))?;
                self.emit_raw(&format!("{}:", else_label))?;

                self.indent += 1;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.emit_statement(&**else_branch)?;
                }
                self.indent -= 1;

                self.emit_raw(&format!("{}:", end_label))?;
                Ok(())
            }
            lir::Statement::While(while_stmt) => {
                let start_label = self.new_label("while_start"); // continue jumps here (recheck condition)
                let end_label = self.new_label("while_end"); // break jumps here

                // Push to label stack for break/continue
                self.loop_labels
                    .push((start_label.clone(), end_label.clone()));

                self.emit_raw(&format!("{}:", start_label))?;

                self.indent += 1;
                self.emit_expression(&while_stmt.condition, "x9")?;
                self.emit_line("cmp x9, #0")?;
                self.emit_line(&format!("b.eq {}", end_label))?;

                self.emit_statement(&*while_stmt.body)?;
                self.emit_line(&format!("b {}", start_label))?;
                self.indent -= 1;

                self.emit_raw(&format!("{}:", end_label))?;

                // Pop from stack
                self.loop_labels.pop();
                Ok(())
            }
            lir::Statement::DoWhile(do_while) => {
                let start_label = self.new_label("do_start");
                let cond_label = self.new_label("do_cond");
                let end_label = self.new_label("do_end");

                // continue -> go to condition check (then back to start if condition true)
                // break -> exit to end_label
                self.loop_labels
                    .push((cond_label.clone(), end_label.clone()));

                self.emit_raw(&format!("{}:", start_label))?;

                self.indent += 1;
                self.emit_statement(&*do_while.body)?;
                self.emit_line(&format!("b {}", cond_label))?;
                self.emit_raw(&format!("{}:", cond_label))?;
                self.emit_expression(&do_while.condition, "x9")?;
                self.emit_line("cmp x9, #0")?;
                self.emit_line(&format!("b.ne {}", start_label))?;
                self.indent -= 1;

                self.emit_raw(&format!("{}:", end_label))?;

                // Pop from stack
                self.loop_labels.pop();
                Ok(())
            }
            lir::Statement::For(for_stmt) => {
                let start_label = self.new_label("for_start");
                let end_label = self.new_label("for_end");
                // continue -> go to condition check (then loop if true)
                // break -> exit to end_label

                // initializer
                if let Some(init) = &for_stmt.initializer {
                    self.emit_statement(init)?;
                }

                // Push to label stack
                self.loop_labels
                    .push((start_label.clone(), end_label.clone()));

                self.emit_raw(&format!("{}:", start_label))?;

                // condition check
                self.indent += 1;
                if let Some(cond) = &for_stmt.condition {
                    self.emit_expression(cond, "x9")?;
                    self.emit_line("cmp x9, #0")?;
                    self.emit_line(&format!("b.eq {}", end_label))?;
                }

                // loop body
                self.emit_statement(&for_stmt.body)?;

                // increment
                if let Some(inc) = &for_stmt.increment {
                    self.emit_expression(inc, "x9")?;
                }

                self.emit_line(&format!("b {}", start_label))?;
                self.indent -= 1;

                self.emit_raw(&format!("{}:", end_label))?;

                // Pop from stack
                self.loop_labels.pop();
                Ok(())
            }
            lir::Statement::Break => {
                // break jumps to the current loop's break target
                if let Some((_continue_label, break_label)) = self.loop_labels.last() {
                    self.emit_line(&format!("b {}", break_label))?;
                } else {
                    self.emit_line("// TODO: break outside loop")?;
                }
                Ok(())
            }
            lir::Statement::Continue => {
                // continue jumps to the current loop's continue target (condition check / loop start)
                if let Some((continue_label, _break_label)) = self.loop_labels.last() {
                    self.emit_line(&format!("b {}", continue_label))?;
                } else {
                    self.emit_line("// TODO: continue outside loop")?;
                }
                Ok(())
            }
            lir::Statement::Return(expr) => {
                // 对于 main 函数，也应该评估 return 表达式
                if let Some(expr) = expr {
                    self.emit_expression(expr, "x0")?;
                } else {
                    // 只有当没有 return 表达式时才返回 0
                    self.emit_line("mov x0, #0")?;
                }
                // 恢复栈帧并返回
                self.emit_line("mov sp, x29")?;
                self.emit_line("ldp x29, x30, [sp], #16")?;
                self.emit_line("ret")?;
                Ok(())
            }
            lir::Statement::Switch(_) => {
                // TODO: 实现 switch
                self.emit_line("// TODO: switch statement")?;
                Ok(())
            }
            lir::Statement::Match(_) => {
                // TODO: 实现 match
                self.emit_line("// TODO: match statement")?;
                Ok(())
            }
            lir::Statement::Try(_) => {
                // TODO: 实现 try
                self.emit_line("// TODO: try statement")?;
                Ok(())
            }
            lir::Statement::Goto(label) => {
                self.emit_line(&format!("b {}", label))?;
                Ok(())
            }
            lir::Statement::Label(label) => {
                self.emit_raw(&format!("{}:", label))?;
                Ok(())
            }
            lir::Statement::Declaration(_) => {
                // 声明已经在函数开头处理
                Ok(())
            }
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

    /// 从语句块收集字符串字面量（须在发出代码前调用，供 .rodata 与 adr/ldr 使用）
    fn collect_string_literals(&mut self, block: &lir::Block) -> NativeResult<()> {
        for stmt in &block.statements {
            self.collect_stmt_strings(stmt)?;
        }
        Ok(())
    }

    fn collect_declaration_strings(&mut self, decl: &lir::Declaration) -> NativeResult<()> {
        match decl {
            lir::Declaration::Function(func) => self.collect_string_literals(&func.body),
            lir::Declaration::Global(g) => {
                if let Some(init) = &g.initializer {
                    self.collect_expr_strings(init)?;
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
                    let label = format!("LC{}", self.string_literals.len());
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

    fn collect_stmt_strings(&mut self, stmt: &lir::Statement) -> NativeResult<()> {
        use lir::Statement;
        match stmt {
            Statement::Expression(expr) => self.collect_expr_strings(expr),
            Statement::Declaration(decl) => self.collect_declaration_strings(decl),
            Statement::Variable(var) => {
                if let Some(init) = &var.initializer {
                    self.collect_expr_strings(init)
                } else {
                    Ok(())
                }
            }
            Statement::If(if_stmt) => {
                self.collect_expr_strings(&if_stmt.condition)?;
                self.collect_stmt_strings(&if_stmt.then_branch)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.collect_stmt_strings(else_branch)?;
                }
                Ok(())
            }
            Statement::While(while_stmt) => {
                self.collect_expr_strings(&while_stmt.condition)?;
                self.collect_stmt_strings(&while_stmt.body)
            }
            Statement::DoWhile(do_while) => {
                self.collect_stmt_strings(&*do_while.body)?;
                self.collect_expr_strings(&do_while.condition)
            }
            Statement::For(for_stmt) => {
                if let Some(init) = &for_stmt.initializer {
                    self.collect_stmt_strings(init)?;
                }
                if let Some(cond) = &for_stmt.condition {
                    self.collect_expr_strings(cond)?;
                }
                if let Some(inc) = &for_stmt.increment {
                    self.collect_expr_strings(inc)?;
                }
                self.collect_stmt_strings(&for_stmt.body)
            }
            Statement::Switch(sw) => {
                self.collect_expr_strings(&sw.expression)?;
                for c in &sw.cases {
                    self.collect_expr_strings(&c.value)?;
                    self.collect_stmt_strings(&c.body)?;
                }
                if let Some(def) = &sw.default {
                    self.collect_stmt_strings(def)?;
                }
                Ok(())
            }
            Statement::Match(m) => {
                self.collect_expr_strings(&m.scrutinee)?;
                for case in &m.cases {
                    self.collect_pattern_strings(&case.pattern)?;
                    if let Some(g) = &case.guard {
                        self.collect_expr_strings(g)?;
                    }
                    self.collect_string_literals(&case.body)?;
                }
                Ok(())
            }
            Statement::Try(t) => {
                self.collect_string_literals(&t.body)?;
                for c in &t.catch_clauses {
                    self.collect_string_literals(&c.body)?;
                }
                if let Some(fin) = &t.finally_block {
                    self.collect_string_literals(fin)?;
                }
                Ok(())
            }
            Statement::Return(Some(expr)) => self.collect_expr_strings(expr),
            Statement::Compound(block) => self.collect_string_literals(block),
            _ => Ok(()),
        }
    }

    fn collect_initializer_strings(&mut self, init: &lir::Initializer) -> NativeResult<()> {
        match init {
            lir::Initializer::Expression(e) => self.collect_expr_strings(e),
            lir::Initializer::List(items) => {
                for i in items {
                    self.collect_initializer_strings(i)?;
                }
                Ok(())
            }
            lir::Initializer::Named(_, inner) => self.collect_initializer_strings(inner),
            lir::Initializer::Indexed(idx, inner) => {
                self.collect_expr_strings(idx)?;
                self.collect_initializer_strings(inner)
            }
        }
    }

    fn collect_expr_strings(&mut self, expr: &lir::Expression) -> NativeResult<()> {
        use lir::{Expression, Literal};
        match expr {
            Expression::Literal(Literal::String(s)) => {
                if !self.string_literals.contains_key(s) {
                    let label = format!("LC{}", self.string_literals.len());
                    self.string_literals.insert(s.clone(), label);
                }
                Ok(())
            }
            Expression::Literal(_) | Expression::Variable(_) => Ok(()),
            Expression::Unary(_, e) => self.collect_expr_strings(e),
            Expression::Binary(_, left, right) => {
                self.collect_expr_strings(left)?;
                self.collect_expr_strings(right)
            }
            Expression::Ternary(c, t, e) => {
                self.collect_expr_strings(c)?;
                self.collect_expr_strings(t)?;
                self.collect_expr_strings(e)
            }
            Expression::Call(f, args) => {
                self.collect_expr_strings(f)?;
                for a in args {
                    self.collect_expr_strings(a)?;
                }
                Ok(())
            }
            Expression::Assign(t, v) => {
                self.collect_expr_strings(t)?;
                self.collect_expr_strings(v)
            }
            Expression::AssignOp(_, t, v) => {
                self.collect_expr_strings(t)?;
                self.collect_expr_strings(v)
            }
            Expression::Index(a, i) => {
                self.collect_expr_strings(a)?;
                self.collect_expr_strings(i)
            }
            Expression::Member(o, _) => self.collect_expr_strings(o),
            Expression::PointerMember(o, _) => self.collect_expr_strings(o),
            Expression::Dereference(e) => self.collect_expr_strings(e),
            Expression::AddressOf(e) => self.collect_expr_strings(e),
            Expression::Cast(_, e) => self.collect_expr_strings(e),
            Expression::Comma(exprs) => {
                for e in exprs {
                    self.collect_expr_strings(e)?;
                }
                Ok(())
            }
            Expression::Parenthesized(e) => self.collect_expr_strings(e),
            Expression::InitializerList(items) => {
                for item in items {
                    self.collect_initializer_strings(item)?;
                }
                Ok(())
            }
            Expression::CompoundLiteral(_, items) => {
                for item in items {
                    self.collect_initializer_strings(item)?;
                }
                Ok(())
            }
            Expression::SizeOf(_) | Expression::AlignOf(_) => Ok(()),
            Expression::SizeOfExpr(e) => self.collect_expr_strings(e),
        }
    }

    /// 处理一个函数
    fn emit_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        // 不可 clear()：会抹掉已生成的 .text 与跨函数共享的 string_literals / field_offsets。
        self.loop_labels.clear();
        self.local_offsets.clear();
        self.stack_size = 0;
        self.current_function = func.name.clone();
        self.local_and_param_types.clear();
        for p in &func.parameters {
            self.local_and_param_types
                .insert(p.name.clone(), p.type_.clone());
        }
        for stmt in &func.body.statements {
            Self::collect_var_types_stmt(stmt, &mut self.local_and_param_types);
        }

        // 收集参数与函数体中的局部变量（参数须参与栈布局，否则 `local_offsets[&param.name]` 会失败）
        let mut locals = Vec::new();
        for p in &func.parameters {
            locals.push((p.name.clone(), p.type_.clone()));
        }
        fn collect_locals(stmt: &lir::Statement, locals: &mut Vec<(String, lir::Type)>) {
            match stmt {
                lir::Statement::Variable(var) => {
                    locals.push((var.name.clone(), var.type_.clone()));
                }
                lir::Statement::Compound(block) => {
                    for stmt in &block.statements {
                        collect_locals(stmt, locals);
                    }
                }
                lir::Statement::If(if_stmt) => {
                    collect_locals(&*if_stmt.then_branch, locals);
                    if let Some(else_branch) = &if_stmt.else_branch {
                        collect_locals(&**else_branch, locals);
                    }
                }
                lir::Statement::While(while_stmt) => {
                    collect_locals(&*while_stmt.body, locals);
                }
                lir::Statement::DoWhile(do_while) => {
                    collect_locals(&*do_while.body, locals);
                }
                lir::Statement::For(for_stmt) => {
                    if let Some(init) = &for_stmt.initializer {
                        collect_locals(&**init, locals);
                    }
                    collect_locals(&*for_stmt.body, locals);
                }
                lir::Statement::Switch(sw) => {
                    for c in &sw.cases {
                        collect_locals(&c.body, locals);
                    }
                    if let Some(def) = &sw.default {
                        collect_locals(&**def, locals);
                    }
                }
                lir::Statement::Match(m) => {
                    for case in &m.cases {
                        for s in &case.body.statements {
                            collect_locals(s, locals);
                        }
                    }
                }
                lir::Statement::Try(t) => {
                    for s in &t.body.statements {
                        collect_locals(s, locals);
                    }
                    for c in &t.catch_clauses {
                        for s in &c.body.statements {
                            collect_locals(s, locals);
                        }
                    }
                    if let Some(fin) = &t.finally_block {
                        for s in &fin.statements {
                            collect_locals(s, locals);
                        }
                    }
                }
                lir::Statement::Declaration(lir::Declaration::Function(f)) => {
                    for s in &f.body.statements {
                        collect_locals(s, locals);
                    }
                }
                _ => {}
            }
        }
        for stmt in &func.body.statements {
            collect_locals(stmt, &mut locals);
        }

        // 声明全局符号
        match self.os {
            TargetOS::MacOS => {
                self.emit_raw(&format!(".global _{}", func.name))?;
                self.emit_raw(&format!("_{}:", func.name))?;
            }
            _ => {
                self.emit_raw(&format!(".global {}", func.name))?;
                self.emit_raw(&format!(".type {}, %function", func.name))?;
                self.emit_raw(&format!("{}:", func.name))?;
            }
        }

        // 函数序言：保存帧指针和链接寄存器
        self.emit_line("stp x29, x30, [sp, #-16]!")?;
        self.emit_line("mov x29, sp")?;

        // 计算栈帧大小并分配栈空间
        // 计算所有局部变量的总大小并对齐到 16 字节
        let mut total_stack = 0usize;
        for (_, ty) in &locals {
            let size = self.size_of_ty(ty);
            let align = if size >= 8 { 8 } else { size };
            total_stack = (total_stack + size + align - 1) & !(align - 1);
        }
        self.stack_size = (total_stack + 15) & !15; // 16 字节对齐

        if self.stack_size > 0 {
            self.emit_line(&format!("sub sp, sp, #{}", self.stack_size))?;
        }

        // 计算局部变量的栈偏移（x29 指向帧指针，从负偏移向下增长）
        let mut current_offset: i32 = -16; // x29 - 16 开始（已经保存了 x29, x30）
        for (name, ty) in &locals {
            let size = self.size_of_ty(&ty);
            let align = if size >= 8 { 8 } else { size };
            // 对齐调整
            let adjustment = ((size + align - 1) & !(align - 1)) as i32;
            current_offset -= adjustment;
            self.local_offsets.insert(name.clone(), current_offset);
        }

        // 将参数从参数寄存器(x0-x7)存储到栈上的局部变量
        // 参数按照顺序分配给前 8 个参数位置
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 8 {
                let offset = self.local_offsets[&param.name];
                self.emit_line(&format!("str x{}, {}", i, self.mem_operand("x29", offset)))?;
            } else {
                // 参数超过 8 个已经在栈上，从 x29 + 16 开始读取
                // TODO: 处理超过 8 个参数的情况
                self.emit_line(&format!("// TODO: parameter {} on stack", i))?;
            }
        }

        // 发射函数体
        self.indent += 1;
        for stmt in &func.body.statements {
            self.emit_statement(stmt)?;
        }
        self.indent -= 1;

        // 如果函数没有显式返回，我们需要添加一个默认返回
        if let Some(last) = func.body.statements.last() {
            if !matches!(last, lir::Statement::Return(_)) {
                // 对于 main 函数，总是返回 0
                self.emit_line("mov x0, #0")?;
                self.emit_line("mov sp, x29")?;
                self.emit_line("ldp x29, x30, [sp], #16")?;
                self.emit_line("ret")?;
            }
        } else {
            // 空函数，直接返回
            if self.current_function == "main" {
                self.emit_line("mov x0, #0")?;
            }
            self.emit_line("mov sp, x29")?;
            self.emit_line("ldp x29, x30, [sp], #16")?;
            self.emit_line("ret")?;
        }

        // 函数大小声明 - 仅在 Linux 上需要，macOS 使用 Mach-O 格式不需要
        if matches!(self.os, TargetOS::Linux) {
            self.emit_raw(&format!(".size {}, .-{}", func.name, func.name))?;
        }
        self.emit_raw("")?;

        Ok(())
    }

    /// 获取类型大小（字节）- AArch64 ABI
    fn type_size(&self, ty: &lir::Type) -> usize {
        use lir::Type;
        match ty {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            Type::Long | Type::Ulong => 8,
            Type::LongLong | Type::UlongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::LongDouble => 16,
            Type::Size | Type::Ptrdiff | Type::Intptr | Type::Uintptr => 8,
            Type::Pointer(_) => 8,
            Type::FunctionPointer(_, _) => 8,
            Type::Array(elem, Some(len)) => elem.size_of() * (*len as usize),
            Type::Array(elem, None) => elem.size_of(),
            Type::Named(_) => 0,
            Type::Qualified(_, ty) => ty.size_of(),
        }
    }

    /// 获取类型对齐（字节）- AArch64 ABI
    fn type_align(&self, ty: &lir::Type) -> usize {
        use lir::Type;
        match ty {
            Type::Void => 1,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            Type::Long | Type::Ulong => 8,
            Type::LongLong | Type::UlongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::LongDouble => 16,
            Type::Size | Type::Ptrdiff | Type::Intptr | Type::Uintptr => 8,
            Type::Pointer(_) => 8,
            Type::FunctionPointer(_, _) => 8,
            Type::Array(elem, _) => elem.align_of(),
            Type::Named(_) => 8,
            Type::Qualified(_, ty) => ty.align_of(),
        }
    }
}

impl AssemblyGenerator for AArch64AssemblyGenerator {
    fn generate(&mut self, lir: &lir::Program) -> NativeResult<String> {
        self.clear();

        // Calculate field offsets for all structs and classes
        for decl in &lir.declarations {
            match decl {
                lir::Declaration::Struct(strct) => {
                    // Calculate field offsets with proper alignment
                    let mut current_offset = 0;
                    let mut max_alignment = 1;

                    for field in &strct.fields {
                        let align = self.type_align(&field.type_);
                        max_alignment = max_alignment.max(align);

                        // Align current offset to field alignment requirement
                        if current_offset % align != 0 {
                            current_offset += align - (current_offset % align);
                        }

                        let size = self.type_size(&field.type_);
                        self.field_offsets
                            .insert(Self::layout_key(&strct.name, &field.name), current_offset);
                        current_offset += size;
                    }

                    // Padded aggregate size (for future layout); field offsets already final
                    let _ = current_offset.next_multiple_of(max_alignment.max(1));
                }
                lir::Declaration::Class(cls) => {
                    // Calculate field offsets with proper alignment - same as struct
                    let mut current_offset = 0;
                    let mut max_alignment = 1;

                    for field in &cls.fields {
                        let align = self.type_align(&field.type_);
                        max_alignment = max_alignment.max(align);

                        // Align current offset to field alignment requirement
                        if current_offset % align != 0 {
                            current_offset += align - (current_offset % align);
                        }

                        let size = self.type_size(&field.type_);
                        self.field_offsets
                            .insert(Self::layout_key(&cls.name, &field.name), current_offset);
                        current_offset += size;
                    }

                    let _ = current_offset.next_multiple_of(max_alignment.max(1));
                }
                lir::Declaration::Global(global) => {
                    let size = self.type_size(&global.type_);
                    self.globals.insert(
                        global.name.clone(),
                        GlobalInfo {
                            size,
                            initialized: global.initializer.is_some(),
                            align: self.type_align(&global.type_),
                            initializer: global.initializer.clone(),
                        },
                    );
                    // 保持声明顺序
                    self.global_order.push(global.name.clone());
                }
                lir::Declaration::Function(func) => {
                    self.collect_string_literals(&func.body)?;
                }
                _ => {}
            }
        }

        // 文件开头 - 代码段（Mach-O 需显式段属性）
        match self.os {
            TargetOS::MacOS => {
                self.emit_raw(".section __TEXT,__text,regular,pure_instructions")?;
            }
            _ => {
                self.emit_raw(".text")?;
            }
        }
        self.emit_raw("")?;

        // 生成所有函数
        for decl in &lir.declarations {
            if let lir::Declaration::Function(func) = decl {
                self.emit_function(func)?;
            }
        }

        // 生成字符串字面量只读数据段
        if !self.string_literals.is_empty() {
            self.emit_raw("")?;
            match self.os {
                TargetOS::MacOS => {
                    // Mach-O：使用 cstring 段，ELF 风格的 .section .rodata 会被 clang 拒绝
                    self.emit_raw(".section __TEXT,__cstring,cstring_literals")?;
                }
                _ => {
                    self.emit_raw(".section .rodata")?;
                }
            }
            let strings: Vec<(String, String)> = self
                .string_literals
                .iter()
                .map(|(s, l)| (s.clone(), l.clone()))
                .collect();
            for (s, label) in strings {
                self.emit_raw(&format!(".p2align 3"))?;
                self.emit_raw(&format!("{}:", label))?;
                self.indent += 1;
                // 正确处理字符串中的转义序列：\n -> 换行符，\\ -> \
                let escaped = s
                    .replace("\\n", "\n")
                    .replace("\\t", "\t")
                    .replace("\\r", "\r")
                    .replace("\\\\", "\\");
                self.emit_line(&format!(".asciz \"{}\"", escaped.escape_debug()))?;
                self.indent -= 1;
            }
        }

        // 生成全局变量
        if !self.globals.is_empty() {
            self.emit_raw("")?;
            match self.os {
                TargetOS::MacOS => {
                    self.emit_raw(".section __DATA,__data")?;
                }
                _ => {
                    self.emit_raw(".data")?;
                }
            }
            // 使用 global_order 保持声明顺序
            let globals: Vec<(String, GlobalInfo)> = self
                .global_order
                .iter()
                .filter_map(|n: &String| self.globals.get(n).map(|i| (n.clone(), i.clone())))
                .collect();
            for (name, info) in globals {
                // GAS .align n 为 2^n 字节边界；align 为 0 时 trailing_zeros 无意义
                let align_pow2 = info.align.max(1).trailing_zeros();
                self.emit_raw(&format!(".align {}", align_pow2))?;
                self.emit_raw(&format!(".global {}", name))?;
                self.emit_raw(&format!("{}:", name))?;
                self.indent += 1;
                if let Some(init) = &info.initializer {
                    // 从 Expression::Literal 中提取值
                    if let lir::Expression::Literal(lit) = init {
                        match lit {
                            lir::Literal::Integer(n) => {
                                self.emit_line(&format!(".word {}", n))?;
                            }
                            lir::Literal::Float(f) => {
                                // 将浮点数转换为位表示
                                let bits: u64 = f.to_bits();
                                self.emit_line(&format!(".word {}", bits))?;
                            }
                            lir::Literal::Double(d) => {
                                let bits: u64 = d.to_bits();
                                self.emit_line(&format!(".word {}", bits))?;
                            }
                            lir::Literal::String(_) | lir::Literal::NullPointer => {
                                // 字符串和空指针用 .zero
                                self.emit_line(&format!(".zero {}", info.size))?;
                            }
                            _ => {
                                self.emit_line(&format!(".zero {}", info.size))?;
                            }
                        }
                    } else {
                        self.emit_line(&format!(".zero {}", info.size))?;
                    }
                } else if info.initialized {
                    self.emit_line(&format!(".space {}", info.size))?;
                } else {
                    self.emit_line(&format!(".zero {}", info.size))?;
                }
                self.indent -= 1;
            }
        }

        Ok(self.output.clone())
    }

    fn arch(&self) -> crate::TargetArch {
        crate::TargetArch::AArch64
    }
}

#[cfg(test)]
mod aarch64_field_tests {
    use super::*;

    #[test]
    fn test_two_structs_same_field_name_pointer_member() {
        let mut program = lir::Program::new();
        program.add(lir::Declaration::Struct(lir::Struct {
            name: "A".into(),
            fields: vec![lir::Field {
                name: "x".into(),
                type_: lir::Type::Int,
            }],
        }));
        program.add(lir::Declaration::Struct(lir::Struct {
            name: "B".into(),
            fields: vec![
                lir::Field {
                    name: "pad".into(),
                    type_: lir::Type::Int,
                },
                lir::Field {
                    name: "x".into(),
                    type_: lir::Type::Int,
                },
            ],
        }));
        let mut func = lir::Function::new("main", lir::Type::Int).param(
            "pb",
            lir::Type::Pointer(Box::new(lir::Type::Named("B".into()))),
        );
        func.body.statements.push(lir::Statement::Return(Some(
            lir::Expression::PointerMember(Box::new(lir::Expression::var("pb")), "x".into()),
        )));
        program.add(lir::Declaration::Function(func));

        let mut gen = AArch64AssemblyGenerator::new(TargetOS::Linux);
        let asm = gen.generate(&program).unwrap();
        assert!(
            asm.contains("add x0, x0, #4") || asm.contains("add x0, x0, #0x4"),
            "应按 *B 使用 `B::x` 偏移 4: {asm}"
        );
    }

    #[test]
    fn test_nested_initializer_list_sub_sp_flat_slots() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", lir::Type::Int);
        func.body.statements.push(lir::Statement::Return(Some(
            lir::Expression::InitializerList(vec![lir::Initializer::List(vec![
                lir::Initializer::Expression(lir::Expression::int(1)),
                lir::Initializer::Expression(lir::Expression::int(2)),
            ])]),
        )));
        program.add(lir::Declaration::Function(func));

        let mut gen = AArch64AssemblyGenerator::new(TargetOS::Linux);
        let asm = gen.generate(&program).unwrap();
        assert!(
            asm.lines().any(|l| l.trim() == "sub sp, sp, #16"),
            "嵌套 List 应 sub sp 分配 16 字节（对齐）: {asm}"
        );
        assert!(
            !asm.contains("TODO: initializer list") && !asm.contains("TODO: compound literal"),
            "不应再保留 initializer/compound 的 TODO: {asm}"
        );
    }
}
