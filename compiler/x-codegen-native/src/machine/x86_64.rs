//! x86-64 直出机器码生成器 (System V AMD64 ABI, Linux)
//!
//! 把 LIR 直接降级为 `.text` 机器码字节，并收集字符串/全局数据、符号表与
//! 重定位。函数内跳转/调用以 fixup 记录、布局完成后回填；外部调用与
//! 字符串/全局引用以 ELF 重定位交给链接器。

use std::collections::{HashMap, HashSet};

use crate::arch::X86Register;
use crate::encoding::{Condition, X86_64Encoder};
use crate::{NativeError, NativeResult, TargetOS};
use x_lir as lir;

use super::{MachineObject, ObjReloc, ObjSymbol, RelKind, RelTarget, SecKind};

/// 发射一条指令的字节到 `.text`
macro_rules! emit {
    ($self:ident, $($call:tt)*) => {{
        let mut __enc = X86_64Encoder::new();
        __enc.$($call)*;
        $self.text.extend_from_slice(__enc.result());
    }};
}

/// 实参寄存器类别（System V AMD64）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArgClass {
    Int,
    Sse,
}

/// SSE 参数寄存器编号（xmm0..xmm7）
const SSE_ARG_REGS: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

/// printf 打印种类（按实参静态类型选择）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrintKind {
    Int,
    Float,
    Str,
    Char,
    Bool,
    Ptr,
}

impl PrintKind {
    fn spec(self) -> &'static str {
        match self {
            PrintKind::Int => "%lld",
            PrintKind::Float => "%g",
            PrintKind::Str => "%s",
            PrintKind::Char => "%c",
            PrintKind::Bool => "%s",
            PrintKind::Ptr => "%p",
        }
    }

    fn from_type(ty: &lir::Type) -> Self {
        use lir::Type;
        let ty = match ty {
            Type::Qualified(_, inner) => inner.as_ref(),
            t => t,
        };
        match ty {
            Type::Bool => PrintKind::Bool,
            Type::Float | Type::Double | Type::LongDouble => PrintKind::Float,
            Type::Char | Type::Schar | Type::Uchar => PrintKind::Char,
            Type::Pointer(inner) => {
                let inner = match inner.as_ref() {
                    Type::Qualified(_, i) => i.as_ref(),
                    t => t,
                };
                if matches!(inner, Type::Char | Type::Schar | Type::Uchar) {
                    PrintKind::Str
                } else {
                    PrintKind::Ptr
                }
            }
            Type::Int
            | Type::Uint
            | Type::Short
            | Type::Ushort
            | Type::Long
            | Type::Ulong
            | Type::LongLong
            | Type::UlongLong
            | Type::Size
            | Type::Ptrdiff
            | Type::Intptr
            | Type::Uintptr => PrintKind::Int,
            _ => PrintKind::Int,
        }
    }
}

const ARG_REGS_SYSV: [X86Register; 6] = [
    X86Register::Rdi,
    X86Register::Rsi,
    X86Register::Rdx,
    X86Register::Rcx,
    X86Register::R8,
    X86Register::R9,
];

/// 函数内跳转修补（rel32 指向某个标签）
struct JumpFixup {
    field: usize,
    label: String,
}

/// 调用修补（rel32 指向某个函数，可能是内部或外部）
struct CallFixup {
    field: usize,
    callee: String,
}

/// 全局变量槽（位于 .data 或 .bss）
struct GlobalSlot {
    offset: u64,
    section: SecKind,
}

/// 已定义函数符号（.text 内）
struct FuncSym {
    name: String,
    offset: u64,
    size: u64,
}

pub struct MachineCodeGen {
    #[allow(dead_code)]
    os: TargetOS,

    text: Vec<u8>,
    rodata: Vec<u8>,
    data: Vec<u8>,
    bss_size: u64,

    /// 字符串 -> .rodata 偏移
    string_offsets: HashMap<String, u64>,
    /// 全局变量名 -> .bss 槽
    globals: HashMap<String, GlobalSlot>,
    /// 全局变量名 -> 静态类型
    global_types: HashMap<String, lir::Type>,
    /// 函数名 -> 返回类型（用于浮点返回值搬运）
    func_return_types: HashMap<String, lir::Type>,
    /// 字段静态类型 StructName::field -> 类型
    field_types: HashMap<String, lir::Type>,

    /// 本函数局部/参数 -> [rbp - offset]（offset 为正）
    local_offsets: HashMap<String, i32>,
    /// 当前栈帧已用字节
    stack_size: usize,
    /// 当前临时压栈深度（8 字节槽数），用于在 call 前对齐 rsp 到 16 字节
    stack_depth: i32,
    /// 当前函数返回类型是否为浮点（决定返回值是否搬到 xmm0）
    current_ret_float: bool,
    /// 当前函数参数与局部变量静态类型
    local_and_param_types: HashMap<String, lir::Type>,
    /// 字段偏移 StructName::field -> 字节
    field_offsets: HashMap<String, usize>,

    /// 标签 -> .text 绝对偏移
    labels: HashMap<String, usize>,
    label_counter: usize,
    jump_fixups: Vec<JumpFixup>,
    call_fixups: Vec<CallFixup>,
    loop_labels: Vec<(String, String)>,

    /// 本程序定义的函数名集合
    defined_functions: HashSet<String>,
    func_offsets: HashMap<String, u64>,
    func_syms: Vec<FuncSym>,

    relocations: Vec<ObjReloc>,
    external_syms: Vec<String>,
}

impl MachineCodeGen {
    pub fn new(os: TargetOS) -> Self {
        Self {
            os,
            text: Vec::new(),
            rodata: Vec::new(),
            data: Vec::new(),
            bss_size: 0,
            string_offsets: HashMap::new(),
            globals: HashMap::new(),
            global_types: HashMap::new(),
            func_return_types: HashMap::new(),
            field_types: HashMap::new(),
            local_offsets: HashMap::new(),
            stack_size: 0,
            stack_depth: 0,
            current_ret_float: false,
            local_and_param_types: HashMap::new(),
            field_offsets: HashMap::new(),
            labels: HashMap::new(),
            label_counter: 0,
            jump_fixups: Vec::new(),
            call_fixups: Vec::new(),
            loop_labels: Vec::new(),
            defined_functions: HashSet::new(),
            func_offsets: HashMap::new(),
            func_syms: Vec::new(),
            relocations: Vec::new(),
            external_syms: Vec::new(),
        }
    }

    /// 主入口：生成完整的机器码对象
    pub fn generate(&mut self, program: &lir::Program) -> NativeResult<MachineObject> {
        self.collect_layout(program);

        for decl in &program.declarations {
            if let lir::Declaration::Function(func) = decl {
                self.defined_functions.insert(func.name.clone());
            }
        }

        for decl in &program.declarations {
            if let lir::Declaration::Function(func) = decl {
                self.gen_function(func)?;
            }
        }

        self.resolve_calls()?;

        let mut symbols: Vec<ObjSymbol> = Vec::new();
        for f in &self.func_syms {
            symbols.push(ObjSymbol {
                name: f.name.clone(),
                section: Some(SecKind::Text),
                value: f.offset,
                size: f.size,
                is_func: true,
                is_global: true,
            });
        }
        for ext in &self.external_syms {
            symbols.push(ObjSymbol {
                name: ext.clone(),
                section: None,
                value: 0,
                size: 0,
                is_func: false,
                is_global: true,
            });
        }

        Ok(MachineObject {
            text: std::mem::take(&mut self.text),
            rodata: std::mem::take(&mut self.rodata),
            data: std::mem::take(&mut self.data),
            bss_size: self.bss_size,
            symbols,
            relocations: std::mem::take(&mut self.relocations),
        })
    }

    // ========================================================================
    // 布局收集：全局变量 (.bss)、结构体字段偏移
    // ========================================================================

    fn collect_layout(&mut self, program: &lir::Program) {
        for decl in &program.declarations {
            match decl {
                lir::Declaration::Global(global) => {
                    let size = self.type_size(&global.type_).max(8);
                    let slot = round_up(size, 8) as u64;
                    self.global_types
                        .insert(global.name.clone(), global.type_.clone());

                    // 常量初始化器 → .data；否则 → .bss（零初始化）
                    let init_bytes = global
                        .initializer
                        .as_ref()
                        .and_then(|e| self.const_init_bytes(&global.type_, e));

                    if let Some(mut bytes) = init_bytes {
                        let offset = round_up(self.data.len(), 8) as u64;
                        while (self.data.len() as u64) < offset {
                            self.data.push(0);
                        }
                        bytes.resize(slot as usize, 0);
                        self.data.extend_from_slice(&bytes);
                        self.globals.insert(
                            global.name.clone(),
                            GlobalSlot {
                                offset,
                                section: SecKind::Data,
                            },
                        );
                    } else {
                        let offset = round_up(self.bss_size as usize, 8) as u64;
                        self.globals.insert(
                            global.name.clone(),
                            GlobalSlot {
                                offset,
                                section: SecKind::Bss,
                            },
                        );
                        self.bss_size = offset + slot;
                    }
                }
                lir::Declaration::Struct(strct) => {
                    self.collect_fields(&strct.name, &strct.fields);
                }
                lir::Declaration::Class(cls) => {
                    self.collect_fields(&cls.name, &cls.fields);
                }
                lir::Declaration::Function(f) => {
                    self.func_return_types
                        .insert(f.name.clone(), f.return_type.clone());
                }
                lir::Declaration::ExternFunction(f) => {
                    self.func_return_types
                        .insert(f.name.clone(), f.return_type.clone());
                }
                _ => {}
            }
        }
    }

    fn collect_fields(&mut self, name: &str, fields: &[lir::Field]) {
        let mut current_offset = 0usize;
        for field in fields {
            let align = self.type_align(&field.type_);
            if current_offset % align != 0 {
                current_offset += align - (current_offset % align);
            }
            let size = self.type_size(&field.type_);
            self.field_offsets
                .insert(format!("{}::{}", name, field.name), current_offset);
            self.field_types
                .insert(format!("{}::{}", name, field.name), field.type_.clone());
            current_offset += size;
        }
    }

    // ========================================================================
    // 字符串与数据
    // ========================================================================

    fn intern_string(&mut self, s: &str) -> u64 {
        if let Some(&off) = self.string_offsets.get(s) {
            return off;
        }
        let off = self.rodata.len() as u64;
        self.rodata.extend_from_slice(s.as_bytes());
        self.rodata.push(0);
        self.string_offsets.insert(s.to_string(), off);
        off
    }

    // ========================================================================
    // 标签 / fixup / 重定位
    // ========================================================================

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("L_{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn define_label(&mut self, label: &str) {
        self.labels.insert(label.to_string(), self.text.len());
    }

    fn emit_jmp(&mut self, label: &str) {
        emit!(self, jmp_rel32(0));
        let field = self.text.len() - 4;
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
        });
    }

    fn emit_jcc(&mut self, cond: Condition, label: &str) {
        emit!(self, jcc(cond, 0));
        let field = self.text.len() - 4;
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
        });
    }

    /// 发射 call rel32（占位），交由 finalize 决定内部回填还是外部重定位
    fn emit_call_named(&mut self, callee: &str) {
        emit!(self, call_rel32(0));
        let field = self.text.len() - 4;
        self.call_fixups.push(CallFixup {
            field,
            callee: callee.to_string(),
        });
    }

    /// lea reg, [rip + str]，登记指向 .rodata 的 PC32 重定位
    fn emit_lea_string(&mut self, dest: X86Register, s: &str) {
        let off = self.intern_string(s);
        emit!(self, lea_reg_rip(dest, 0));
        let field = self.text.len() - 4;
        self.relocations.push(ObjReloc {
            offset: field as u64,
            target: RelTarget::Section(SecKind::Rodata),
            kind: RelKind::Pc32,
            addend: off as i64 - 4,
        });
    }

    /// lea reg, [rip + global]，登记指向 .data/.bss 的 PC32 重定位
    fn emit_lea_global(&mut self, dest: X86Register, name: &str) -> NativeResult<()> {
        let (off, section) = self
            .globals
            .get(name)
            .map(|g| (g.offset, g.section))
            .ok_or_else(|| NativeError::CodegenError(format!("未知全局变量: {}", name)))?;
        emit!(self, lea_reg_rip(dest, 0));
        let field = self.text.len() - 4;
        self.relocations.push(ObjReloc {
            offset: field as u64,
            target: RelTarget::Section(section),
            kind: RelKind::Pc32,
            addend: off as i64 - 4,
        });
        Ok(())
    }

    /// 尝试把全局变量的常量初始化器编码为字节（小端）。仅支持标量字面量。
    fn const_init_bytes(&self, ty: &lir::Type, e: &lir::Expression) -> Option<Vec<u8>> {
        use lir::{Expression, Literal};
        let e = match e {
            Expression::Parenthesized(i) => i.as_ref(),
            Expression::Cast(_, i) => i.as_ref(),
            other => other,
        };
        let lit = match e {
            Expression::Literal(l) => l,
            _ => return None,
        };
        let size = self.type_size(ty).max(1);
        let int_bytes = |v: i64, n: usize| -> Vec<u8> {
            let le = v.to_le_bytes();
            le[..n.min(8)].to_vec()
        };
        let bytes = match lit {
            Literal::Integer(v) | Literal::Long(v) | Literal::LongLong(v) => int_bytes(*v, size),
            Literal::UnsignedInteger(v)
            | Literal::UnsignedLong(v)
            | Literal::UnsignedLongLong(v) => int_bytes(*v as i64, size),
            Literal::Bool(b) => vec![*b as u8],
            Literal::Char(c) => int_bytes(*c as i64, size),
            // 原生后端统一以 8 字节 f64 存储浮点
            Literal::Float(f) => {
                let _ = ty;
                (*f as f64).to_le_bytes().to_vec()
            }
            Literal::Double(f) => f.to_le_bytes().to_vec(),
            Literal::NullPointer => int_bytes(0, size),
            Literal::String(_) => return None,
        };
        Some(bytes)
    }

    fn write_i32(&mut self, at: usize, val: i32) {
        self.text[at..at + 4].copy_from_slice(&val.to_le_bytes());
    }

    /// 在函数布局完成后回填函数内跳转（标签为函数局部，避免跨函数串名）
    fn resolve_jumps(&mut self) -> NativeResult<()> {
        let jumps = std::mem::take(&mut self.jump_fixups);
        for jf in jumps {
            let target = *self
                .labels
                .get(&jf.label)
                .ok_or_else(|| NativeError::CodegenError(format!("未定义标签: {}", jf.label)))?;
            let rel = target as i64 - (jf.field as i64 + 4);
            self.write_i32(jf.field, rel as i32);
        }
        Ok(())
    }

    fn resolve_calls(&mut self) -> NativeResult<()> {
        let calls = std::mem::take(&mut self.call_fixups);
        for cf in calls {
            if let Some(&off) = self.func_offsets.get(&cf.callee) {
                let rel = off as i64 - (cf.field as i64 + 4);
                self.write_i32(cf.field, rel as i32);
            } else {
                if !self.external_syms.iter().any(|s| s == &cf.callee) {
                    self.external_syms.push(cf.callee.clone());
                }
                self.relocations.push(ObjReloc {
                    offset: cf.field as u64,
                    target: RelTarget::Symbol(cf.callee.clone()),
                    kind: RelKind::Plt32,
                    addend: -4,
                });
            }
        }
        Ok(())
    }

    // ========================================================================
    // 栈帧辅助
    // ========================================================================

    fn store_local(&mut self, offset: i32, src: X86Register) {
        emit!(self, mov_mem_reg(X86Register::Rbp, -offset, src));
    }

    fn load_local(&mut self, dest: X86Register, offset: i32) {
        emit!(self, mov_reg_mem(dest, X86Register::Rbp, -offset));
    }

    /// 压入一个临时值（跟踪栈深度，供 call 对齐）
    fn push(&mut self, reg: X86Register) {
        emit!(self, push_reg(reg));
        self.stack_depth += 1;
    }

    /// 弹出一个临时值
    fn pop(&mut self, reg: X86Register) {
        emit!(self, pop_reg(reg));
        self.stack_depth -= 1;
    }

    // ========================================================================
    // 函数
    // ========================================================================

    fn gen_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        let start = self.text.len() as u64;
        self.func_offsets.insert(func.name.clone(), start);

        self.local_offsets.clear();
        self.stack_size = 0;
        self.stack_depth = 0;
        self.current_ret_float = matches!(
            Self::peel_qualified_ty(&func.return_type),
            lir::Type::Float | lir::Type::Double | lir::Type::LongDouble
        );
        self.local_and_param_types.clear();
        self.loop_labels.clear();
        // 标签是函数局部的：清空，避免跨函数同名（如 "bb0"）冲突
        self.labels.clear();
        self.jump_fixups.clear();

        for p in &func.parameters {
            self.local_and_param_types
                .insert(p.name.clone(), p.type_.clone());
        }
        for stmt in &func.body.statements {
            Self::collect_var_types_stmt(stmt, &mut self.local_and_param_types);
        }

        // 序言
        emit!(self, push_reg(X86Register::Rbp));
        emit!(self, mov_reg_reg(X86Register::Rbp, X86Register::Rsp));
        emit!(self, sub_reg_imm32(X86Register::Rsp, 0));
        let frame_patch = self.text.len() - 4;

        // 参数落栈：整型来自 GPR(rdi..)，浮点来自 XMM(xmm0..)
        let nparams = func.parameters.len();
        self.stack_size = nparams * 8;
        let mut gi = 0usize;
        let mut si = 0usize;
        for (i, param) in func.parameters.iter().enumerate() {
            let offset = ((i + 1) * 8) as i32;
            self.local_offsets.insert(param.name.clone(), offset);
            let is_float = matches!(
                Self::peel_qualified_ty(&param.type_),
                lir::Type::Float | lir::Type::Double | lir::Type::LongDouble
            );
            if is_float {
                if si < SSE_ARG_REGS.len() {
                    emit!(self, movq_gpr_xmm(X86Register::Rax, SSE_ARG_REGS[si]));
                    self.store_local(offset, X86Register::Rax);
                }
                si += 1;
            } else {
                if gi < ARG_REGS_SYSV.len() {
                    self.store_local(offset, ARG_REGS_SYSV[gi]);
                }
                gi += 1;
            }
        }

        // 函数体
        self.gen_block(&func.body)?;

        // 兜底尾声（落空时）
        self.gen_epilogue();

        // 函数内跳转回填（标签局部）
        self.resolve_jumps()?;

        // 回填栈帧大小（16 字节对齐）
        let frame = round_up(self.stack_size, 16) as i32;
        self.write_i32(frame_patch, frame);

        let size = self.text.len() as u64 - start;
        self.func_syms.push(FuncSym {
            name: func.name.clone(),
            offset: start,
            size,
        });
        Ok(())
    }

    fn gen_epilogue(&mut self) {
        emit!(self, mov_reg_reg(X86Register::Rsp, X86Register::Rbp));
        emit!(self, pop_reg(X86Register::Rbp));
        emit!(self, ret());
    }

    fn gen_block(&mut self, block: &lir::Block) -> NativeResult<()> {
        for stmt in &block.statements {
            self.gen_statement(stmt)?;
        }
        Ok(())
    }

    fn gen_statement(&mut self, stmt: &lir::Statement) -> NativeResult<()> {
        use lir::Statement;
        match stmt {
            Statement::Expression(expr) => {
                self.gen_expr(expr)?;
            }
            Statement::Variable(var) => {
                self.stack_size += 8;
                let offset = self.stack_size as i32;
                self.local_offsets.insert(var.name.clone(), offset);
                if let Some(init) = &var.initializer {
                    self.gen_expr(init)?;
                } else {
                    emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax));
                }
                self.store_local(offset, X86Register::Rax);
            }
            Statement::Return(Some(expr)) => {
                self.gen_expr(expr)?;
                if self.current_ret_float {
                    emit!(self, movq_xmm_gpr(0, X86Register::Rax));
                }
                self.gen_epilogue();
            }
            Statement::Return(None) => {
                emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax));
                self.gen_epilogue();
            }
            Statement::If(if_stmt) => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");
                self.gen_expr(&if_stmt.condition)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::E, &else_label);
                self.gen_statement(&if_stmt.then_branch)?;
                self.emit_jmp(&end_label);
                self.define_label(&else_label);
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.gen_statement(else_branch)?;
                }
                self.define_label(&end_label);
            }
            Statement::While(while_stmt) => {
                let start_label = self.new_label("while_start");
                let end_label = self.new_label("while_end");
                self.loop_labels
                    .push((start_label.clone(), end_label.clone()));
                self.define_label(&start_label);
                self.gen_expr(&while_stmt.condition)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::E, &end_label);
                self.gen_statement(&while_stmt.body)?;
                self.emit_jmp(&start_label);
                self.define_label(&end_label);
                self.loop_labels.pop();
            }
            Statement::DoWhile(do_while) => {
                let start_label = self.new_label("do_start");
                let cond_label = self.new_label("do_cond");
                let end_label = self.new_label("do_end");
                self.loop_labels
                    .push((cond_label.clone(), end_label.clone()));
                self.define_label(&start_label);
                self.gen_statement(&do_while.body)?;
                self.define_label(&cond_label);
                self.gen_expr(&do_while.condition)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::NE, &start_label);
                self.define_label(&end_label);
                self.loop_labels.pop();
            }
            Statement::For(for_stmt) => {
                let start_label = self.new_label("for_start");
                let end_label = self.new_label("for_end");
                if let Some(init) = &for_stmt.initializer {
                    self.gen_statement(init)?;
                }
                self.loop_labels
                    .push((start_label.clone(), end_label.clone()));
                self.define_label(&start_label);
                if let Some(cond) = &for_stmt.condition {
                    self.gen_expr(cond)?;
                    emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                    self.emit_jcc(Condition::E, &end_label);
                }
                self.gen_statement(&for_stmt.body)?;
                if let Some(inc) = &for_stmt.increment {
                    self.gen_expr(inc)?;
                }
                self.emit_jmp(&start_label);
                self.define_label(&end_label);
                self.loop_labels.pop();
            }
            Statement::Compound(block) => {
                self.gen_block(block)?;
            }
            Statement::Break => {
                if let Some((_c, b)) = self.loop_labels.last() {
                    let b = b.clone();
                    self.emit_jmp(&b);
                }
            }
            Statement::Continue => {
                if let Some((c, _b)) = self.loop_labels.last() {
                    let c = c.clone();
                    self.emit_jmp(&c);
                }
            }
            Statement::Label(name) => {
                self.define_label(name);
            }
            Statement::Goto(name) => {
                let name = name.clone();
                self.emit_jmp(&name);
            }
            Statement::Switch(sw) => {
                self.gen_switch(sw)?;
            }
            Statement::Match(m) => {
                self.gen_match(m)?;
            }
            Statement::Declaration(_) => {
                log::debug!("native: 暂不支持函数内嵌套声明");
            }
            Statement::Empty => {}
            _ => {
                log::debug!("native: 未支持的语句 {:?}", std::mem::discriminant(stmt));
            }
        }
        Ok(())
    }

    /// switch 语句：整型比较链。
    ///
    /// MIR 生成的 switch 各分支体是 `goto <block>`，会直接跳出 switch，
    /// 因此不能把被测值长期压栈（跳出时不会执行配对的 pop，导致栈失衡 →
    /// 后续调用栈未对齐而崩溃）。这里改为每个分支各自独立地求值被测值与
    /// 分支常量并比较，push/pop 在单次比较内严格配对，不跨越任何跳转。
    fn gen_switch(&mut self, sw: &lir::SwitchStatement) -> NativeResult<()> {
        let end = self.new_label("switch_end");
        self.loop_labels.push((end.clone(), end.clone()));

        let case_labels: Vec<String> = (0..sw.cases.len())
            .map(|_| self.new_label("case"))
            .collect();
        let default_label = self.new_label("switch_default");

        for (i, case) in sw.cases.iter().enumerate() {
            // rcx = 分支常量
            self.gen_expr(&case.value)?;
            self.push(X86Register::Rax);
            // rax = 被测值（重新求值，避免跨跳转的栈残留）
            self.gen_expr(&sw.expression)?;
            self.pop(X86Register::Rcx);
            emit!(self, cmp_reg_reg(X86Register::Rax, X86Register::Rcx));
            self.emit_jcc(Condition::E, &case_labels[i]);
        }
        self.emit_jmp(&default_label);

        for (i, case) in sw.cases.iter().enumerate() {
            self.define_label(&case_labels[i]);
            self.gen_statement(&case.body)?;
        }
        self.define_label(&default_label);
        if let Some(def) = &sw.default {
            self.gen_statement(def)?;
        }
        self.define_label(&end);
        self.loop_labels.pop();
        Ok(())
    }

    /// match 语句：支持 Wildcard / Variable(绑定到被测值) / Literal 模式。
    /// Constructor/Tuple/Record 等需要 ADT 运行时表示，暂不支持。
    fn gen_match(&mut self, m: &lir::MatchStatement) -> NativeResult<()> {
        use lir::Pattern;
        let end = self.new_label("match_end");
        self.gen_expr(&m.scrutinee)?;
        self.push(X86Register::Rax); // 栈顶保存被测值

        for case in &m.cases {
            let next = self.new_label("match_next");
            match &case.pattern {
                Pattern::Wildcard => {}
                Pattern::Variable(name) => {
                    // 绑定被测值到该变量（若已有槽）
                    if let Some(&off) = self.local_offsets.get(name) {
                        emit!(self, mov_reg_mem(X86Register::Rax, X86Register::Rsp, 0));
                        self.store_local(off, X86Register::Rax);
                    }
                }
                Pattern::Literal(lit) => {
                    self.gen_literal(lit)?;
                    emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
                    emit!(self, mov_reg_mem(X86Register::Rax, X86Register::Rsp, 0));
                    emit!(self, cmp_reg_reg(X86Register::Rax, X86Register::Rcx));
                    self.emit_jcc(Condition::NE, &next);
                }
                _ => {
                    // 不支持的模式：跳过该分支
                    self.emit_jmp(&next);
                }
            }
            if let Some(guard) = &case.guard {
                self.gen_expr(guard)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::E, &next);
            }
            self.gen_block(&case.body)?;
            self.emit_jmp(&end);
            self.define_label(&next);
        }
        self.define_label(&end);
        self.pop(X86Register::Rax);
        Ok(())
    }

    // ========================================================================
    // 表达式（结果入 rax）
    // ========================================================================

    fn gen_expr(&mut self, expr: &lir::Expression) -> NativeResult<()> {
        use lir::{Expression, UnaryOp};
        match expr {
            Expression::Literal(lit) => self.gen_literal(lit),
            Expression::Variable(name) => {
                if let Some(&offset) = self.local_offsets.get(name) {
                    self.load_local(X86Register::Rax, offset);
                } else if self.globals.contains_key(name) {
                    self.emit_lea_global(X86Register::Rax, name)?;
                    emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
                } else {
                    return Err(NativeError::CodegenError(format!("未定义变量: {}", name)));
                }
                Ok(())
            }
            Expression::Unary(op, e) => {
                match op {
                    UnaryOp::Minus => {
                        self.gen_expr(e)?;
                        emit!(self, neg_reg(X86Register::Rax));
                    }
                    UnaryOp::BitNot => {
                        self.gen_expr(e)?;
                        emit!(self, not_reg(X86Register::Rax));
                    }
                    UnaryOp::Plus => {
                        self.gen_expr(e)?;
                    }
                    UnaryOp::Not => {
                        self.gen_expr(e)?;
                        emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                        emit!(self, setcc(Condition::E, X86Register::Al));
                        emit!(self, movzx_r64_r8(X86Register::Rax, X86Register::Al));
                    }
                    UnaryOp::Reference | UnaryOp::MutableReference => {
                        return self.gen_expr(&Expression::AddressOf(e.clone()));
                    }
                    UnaryOp::PreIncrement | UnaryOp::PreDecrement => {
                        let delta = if matches!(op, UnaryOp::PreIncrement) {
                            1
                        } else {
                            -1
                        };
                        self.gen_inc_dec(e, delta, true)?;
                    }
                    UnaryOp::PostIncrement | UnaryOp::PostDecrement => {
                        let delta = if matches!(op, UnaryOp::PostIncrement) {
                            1
                        } else {
                            -1
                        };
                        self.gen_inc_dec(e, delta, false)?;
                    }
                }
                Ok(())
            }
            Expression::Binary(op, left, right) => self.gen_binary(*op, left, right),
            Expression::Call(func, args) => self.gen_call(func, args),
            Expression::Assign(target, value) => self.gen_assign(target, value),
            Expression::AssignOp(op, target, value) => self.gen_assign_op(*op, target, value),
            Expression::SizeOf(ty) => {
                let size = self.type_size(ty) as i64;
                emit!(self, mov_reg_imm(X86Register::Rax, size));
                Ok(())
            }
            Expression::SizeOfExpr(_) => {
                emit!(self, mov_reg_imm(X86Register::Rax, 8));
                Ok(())
            }
            Expression::AlignOf(ty) => {
                let align = self.type_align(ty) as i64;
                emit!(self, mov_reg_imm(X86Register::Rax, align));
                Ok(())
            }
            Expression::Ternary(cond, then, else_) => {
                let else_label = self.new_label("tern_else");
                let end_label = self.new_label("tern_end");
                self.gen_expr(cond)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::E, &else_label);
                self.gen_expr(then)?;
                self.emit_jmp(&end_label);
                self.define_label(&else_label);
                self.gen_expr(else_)?;
                self.define_label(&end_label);
                Ok(())
            }
            Expression::Cast(target_ty, inner) => {
                self.gen_expr(inner)?;
                let to_float = matches!(
                    Self::peel_qualified_ty(target_ty),
                    lir::Type::Float | lir::Type::Double | lir::Type::LongDouble
                );
                let from_float = self.expr_is_float(inner);
                if to_float && !from_float {
                    // int -> double
                    emit!(self, cvtsi2sd(0, X86Register::Rax));
                    emit!(self, movq_gpr_xmm(X86Register::Rax, 0));
                } else if !to_float && from_float {
                    // double -> int（截断）
                    emit!(self, movq_xmm_gpr(0, X86Register::Rax));
                    emit!(self, cvttsd2si(X86Register::Rax, 0));
                }
                Ok(())
            }
            Expression::Parenthesized(inner) => self.gen_expr(inner),
            Expression::AddressOf(inner) => match inner.as_ref() {
                Expression::Variable(name) => {
                    if let Some(&offset) = self.local_offsets.get(name) {
                        emit!(
                            self,
                            lea_reg_mem(X86Register::Rax, X86Register::Rbp, -offset)
                        );
                        Ok(())
                    } else if self.globals.contains_key(name) {
                        self.emit_lea_global(X86Register::Rax, name)
                    } else {
                        self.gen_expr(inner)
                    }
                }
                _ => self.gen_expr(inner),
            },
            Expression::Dereference(inner) => {
                self.gen_expr(inner)?;
                emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
                Ok(())
            }
            Expression::Index(arr, idx) => {
                let elem_ty = self.index_elem_type(arr);
                let elem_size = elem_ty.as_ref().map(|t| self.type_size(t)).unwrap_or(8);
                self.gen_expr(arr)?;
                self.push(X86Register::Rax);
                self.gen_expr(idx)?;
                emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
                self.pop(X86Register::Rax);
                self.scale_reg(X86Register::Rcx, elem_size);
                emit!(self, add_reg_reg(X86Register::Rax, X86Register::Rcx));
                self.emit_sized_load(elem_ty.as_ref(), elem_size);
                Ok(())
            }
            Expression::Member(obj, field) => {
                self.gen_expr(obj)?;
                let offset = self.resolve_field_offset(obj, field, false).unwrap_or(0);
                if offset > 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, offset as i32));
                }
                emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
                Ok(())
            }
            Expression::PointerMember(obj, field) => {
                self.gen_expr(obj)?;
                emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
                let offset = self.resolve_field_offset(obj, field, true).unwrap_or(0);
                if offset > 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, offset as i32));
                }
                emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
                Ok(())
            }
            Expression::Comma(exprs) => {
                for e in exprs {
                    self.gen_expr(e)?;
                }
                Ok(())
            }
            _ => {
                log::debug!("native: 未支持的表达式 {:?}", std::mem::discriminant(expr));
                emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax));
                Ok(())
            }
        }
    }

    fn gen_literal(&mut self, lit: &lir::Literal) -> NativeResult<()> {
        use lir::Literal;
        match lit {
            Literal::Integer(n) => emit!(self, mov_reg_imm(X86Register::Rax, *n)),
            Literal::UnsignedInteger(n) => {
                emit!(self, mov_reg_imm64(X86Register::Rax, *n as u64))
            }
            Literal::Long(n) => emit!(self, mov_reg_imm(X86Register::Rax, *n)),
            Literal::UnsignedLong(n) => {
                emit!(self, mov_reg_imm64(X86Register::Rax, *n as u64))
            }
            Literal::LongLong(n) => emit!(self, mov_reg_imm(X86Register::Rax, *n as i64)),
            Literal::UnsignedLongLong(n) => {
                emit!(self, mov_reg_imm64(X86Register::Rax, *n as u64))
            }
            Literal::Bool(b) => {
                emit!(self, mov_reg_imm(X86Register::Rax, if *b { 1 } else { 0 }))
            }
            // 浮点字面量：把 f64 位模式装入 rax（值模型为“rax 中的 8 字节位”）
            Literal::Float(f) => {
                emit!(self, mov_reg_imm64(X86Register::Rax, (*f as f64).to_bits()))
            }
            Literal::Double(f) => {
                emit!(self, mov_reg_imm64(X86Register::Rax, f.to_bits()))
            }
            Literal::Char(c) => emit!(self, mov_reg_imm(X86Register::Rax, *c as i64)),
            Literal::String(s) => {
                let s = s.clone();
                self.emit_lea_string(X86Register::Rax, &s);
            }
            Literal::NullPointer => {
                emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax))
            }
        }
        Ok(())
    }

    fn gen_binary(
        &mut self,
        op: lir::BinaryOp,
        left: &lir::Expression,
        right: &lir::Expression,
    ) -> NativeResult<()> {
        use lir::BinaryOp;

        // 短路运算单独处理
        match op {
            BinaryOp::LogicalAnd => {
                let end = self.new_label("land_end");
                self.gen_expr(left)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::E, &end);
                self.gen_expr(right)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                emit!(self, setcc(Condition::NE, X86Register::Al));
                emit!(self, movzx_r64_r8(X86Register::Rax, X86Register::Al));
                self.define_label(&end);
                return Ok(());
            }
            BinaryOp::LogicalOr => {
                let set_one = self.new_label("lor_one");
                let end = self.new_label("lor_end");
                self.gen_expr(left)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::NE, &set_one);
                self.gen_expr(right)?;
                emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_jcc(Condition::NE, &set_one);
                emit!(self, mov_reg_imm(X86Register::Rax, 0));
                self.emit_jmp(&end);
                self.define_label(&set_one);
                emit!(self, mov_reg_imm(X86Register::Rax, 1));
                self.define_label(&end);
                return Ok(());
            }
            _ => {}
        }

        let lf = self.expr_is_float(left);
        let rf = self.expr_is_float(right);
        if lf || rf {
            return self.gen_binary_float(op, left, right, lf, rf);
        }

        self.gen_expr(left)?;
        self.push(X86Register::Rax);
        self.gen_expr(right)?;
        emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
        self.pop(X86Register::Rax);

        match op {
            BinaryOp::Add => emit!(self, add_reg_reg(X86Register::Rax, X86Register::Rcx)),
            BinaryOp::Subtract => emit!(self, sub_reg_reg(X86Register::Rax, X86Register::Rcx)),
            BinaryOp::Multiply => emit!(self, imul_reg_reg(X86Register::Rax, X86Register::Rcx)),
            BinaryOp::Divide => {
                emit!(self, cqo());
                emit!(self, idiv_reg(X86Register::Rcx));
            }
            BinaryOp::Modulo => {
                emit!(self, cqo());
                emit!(self, idiv_reg(X86Register::Rcx));
                emit!(self, mov_reg_reg(X86Register::Rax, X86Register::Rdx));
            }
            BinaryOp::BitAnd => emit!(self, and_reg_reg(X86Register::Rax, X86Register::Rcx)),
            BinaryOp::BitOr => emit!(self, or_reg_reg(X86Register::Rax, X86Register::Rcx)),
            BinaryOp::BitXor => emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rcx)),
            BinaryOp::LeftShift => emit!(self, shl_reg_cl(X86Register::Rax)),
            BinaryOp::RightShift => emit!(self, shr_reg_cl(X86Register::Rax)),
            BinaryOp::RightShiftArithmetic => emit!(self, sar_reg_cl(X86Register::Rax)),
            BinaryOp::Equal => self.gen_cmp_set(Condition::E),
            BinaryOp::NotEqual => self.gen_cmp_set(Condition::NE),
            BinaryOp::LessThan => self.gen_cmp_set(Condition::L),
            BinaryOp::LessThanEqual => self.gen_cmp_set(Condition::LE),
            BinaryOp::GreaterThan => self.gen_cmp_set(Condition::G),
            BinaryOp::GreaterThanEqual => self.gen_cmp_set(Condition::GE),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => unreachable!(),
        }
        Ok(())
    }

    fn gen_cmp_set(&mut self, cond: Condition) {
        emit!(self, cmp_reg_reg(X86Register::Rax, X86Register::Rcx));
        emit!(self, setcc(cond, X86Register::Al));
        emit!(self, movzx_r64_r8(X86Register::Rax, X86Register::Al));
    }

    /// 表达式静态类型是否为浮点
    fn expr_is_float(&self, e: &lir::Expression) -> bool {
        matches!(
            self.infer_expr_type(e),
            Some(lir::Type::Float) | Some(lir::Type::Double) | Some(lir::Type::LongDouble)
        )
    }

    /// 浮点二元运算。值模型：操作数为 rax/rcx 中的 8 字节位（浮点为 f64 位，
    /// 整数为普通整数，需要时用 cvtsi2sd 提升）。结果（算术）放回 rax 的 f64 位；
    /// 比较结果为 rax 中的 0/1。
    fn gen_binary_float(
        &mut self,
        op: lir::BinaryOp,
        left: &lir::Expression,
        right: &lir::Expression,
        lf: bool,
        rf: bool,
    ) -> NativeResult<()> {
        use lir::BinaryOp;
        self.gen_expr(left)?;
        self.push(X86Register::Rax);
        self.gen_expr(right)?;
        emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax)); // rcx = right bits/int
        self.pop(X86Register::Rax); // rax = left bits/int

        // 装入 xmm0=left, xmm1=right（必要时 int->double）
        if lf {
            emit!(self, movq_xmm_gpr(0, X86Register::Rax));
        } else {
            emit!(self, cvtsi2sd(0, X86Register::Rax));
        }
        if rf {
            emit!(self, movq_xmm_gpr(1, X86Register::Rcx));
        } else {
            emit!(self, cvtsi2sd(1, X86Register::Rcx));
        }

        match op {
            BinaryOp::Add => emit!(self, addsd(0, 1)),
            BinaryOp::Subtract => emit!(self, subsd(0, 1)),
            BinaryOp::Multiply => emit!(self, mulsd(0, 1)),
            BinaryOp::Divide => emit!(self, divsd(0, 1)),
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::LessThan
            | BinaryOp::LessThanEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanEqual => {
                // ucomisd 设置 CF/ZF/PF（类似无符号比较）
                emit!(self, ucomisd(0, 1));
                let cond = match op {
                    BinaryOp::Equal => Condition::E,
                    BinaryOp::NotEqual => Condition::NE,
                    BinaryOp::LessThan => Condition::B,
                    BinaryOp::LessThanEqual => Condition::BE,
                    BinaryOp::GreaterThan => Condition::A,
                    BinaryOp::GreaterThanEqual => Condition::AE,
                    _ => unreachable!(),
                };
                emit!(self, setcc(cond, X86Register::Al));
                emit!(self, movzx_r64_r8(X86Register::Rax, X86Register::Al));
                return Ok(());
            }
            _ => {
                // 取模等浮点未支持的运算：退化为加法语义占位
                emit!(self, addsd(0, 1));
            }
        }
        // 算术结果回到 rax 的 f64 位
        emit!(self, movq_gpr_xmm(X86Register::Rax, 0));
        Ok(())
    }

    fn gen_assign(
        &mut self,
        target: &lir::Expression,
        value: &lir::Expression,
    ) -> NativeResult<()> {
        use lir::Expression;
        match target {
            Expression::Variable(name) => {
                self.gen_expr(value)?;
                if let Some(&offset) = self.local_offsets.get(name) {
                    self.store_local(offset, X86Register::Rax);
                } else if self.globals.contains_key(name) {
                    emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
                    self.emit_lea_global(X86Register::Rax, name)?;
                    emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
                } else {
                    return Err(NativeError::CodegenError(format!(
                        "未定义赋值目标: {}",
                        name
                    )));
                }
            }
            Expression::Dereference(ptr) => {
                self.gen_expr(value)?;
                self.push(X86Register::Rax);
                self.gen_expr(ptr)?;
                self.pop(X86Register::Rcx);
                emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
            }
            Expression::Member(obj, field) => {
                self.gen_expr(value)?;
                self.push(X86Register::Rax);
                self.gen_expr(obj)?;
                let offset = self.resolve_field_offset(obj, field, false).unwrap_or(0);
                if offset > 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, offset as i32));
                }
                self.pop(X86Register::Rcx);
                emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
            }
            Expression::PointerMember(obj, field) => {
                self.gen_expr(value)?;
                self.push(X86Register::Rax);
                self.gen_expr(obj)?;
                emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
                let offset = self.resolve_field_offset(obj, field, true).unwrap_or(0);
                if offset > 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, offset as i32));
                }
                self.pop(X86Register::Rcx);
                emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
            }
            Expression::Index(arr, idx) => {
                let elem_ty = self.index_elem_type(arr);
                let elem_size = elem_ty.as_ref().map(|t| self.type_size(t)).unwrap_or(8);
                self.gen_expr(value)?;
                self.push(X86Register::Rax);
                self.gen_expr(arr)?;
                self.push(X86Register::Rax);
                self.gen_expr(idx)?;
                self.scale_reg(X86Register::Rax, elem_size);
                emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
                self.pop(X86Register::Rax);
                emit!(self, add_reg_reg(X86Register::Rax, X86Register::Rcx));
                self.pop(X86Register::Rcx);
                self.emit_sized_store(elem_size);
            }
            _ => {
                self.gen_expr(value)?;
                log::debug!("native: 未支持的赋值目标");
            }
        }
        Ok(())
    }

    fn gen_assign_op(
        &mut self,
        op: lir::BinaryOp,
        target: &lir::Expression,
        value: &lir::Expression,
    ) -> NativeResult<()> {
        // 形如 x op= v，当前仅完整支持局部/全局变量目标
        let binexpr =
            lir::Expression::Binary(op, Box::new(target.clone()), Box::new(value.clone()));
        self.gen_assign(target, &binexpr)
    }

    fn gen_call(&mut self, func: &lir::Expression, args: &[lir::Expression]) -> NativeResult<()> {
        use lir::Expression;

        // 仅当被调用者是“具名函数符号”（内部定义或外部 libc）时走直接调用；
        // 若被调用者是局部/全局变量（函数指针值），则走间接调用。
        let direct_name = match func {
            Expression::Variable(name)
                if !self.local_offsets.contains_key(name) && !self.globals.contains_key(name) =>
            {
                Some(name.clone())
            }
            _ => None,
        };

        // 内建打印函数：基于实参静态类型生成 printf 风格调用
        if let Some(name) = &direct_name {
            if matches!(
                name.as_str(),
                "println" | "print" | "print_inline" | "eprintln" | "eprint"
            ) {
                return self.gen_print_call(name, args);
            }
        }

        match direct_name {
            Some(name) => {
                let nargs = args.len();
                let classes: Vec<ArgClass> = args
                    .iter()
                    .map(|a| {
                        if self.expr_is_float(a) {
                            ArgClass::Sse
                        } else {
                            ArgClass::Int
                        }
                    })
                    .collect();
                let ret_float = matches!(
                    self.func_return_types.get(&name),
                    Some(lir::Type::Float) | Some(lir::Type::Double) | Some(lir::Type::LongDouble)
                );
                if classes.iter().all(|c| *c == ArgClass::Int) {
                    // 纯整型：支持 >6 栈实参
                    self.emit_named_call_values(&name, nargs, 0, |s, i| s.gen_expr(&args[i]))?;
                } else {
                    self.emit_named_call_classified(&name, &classes, |s, i| s.gen_expr(&args[i]))?;
                }
                if ret_float {
                    // 浮点返回值在 xmm0，搬到 rax 的 f64 位
                    emit!(self, movq_gpr_xmm(X86Register::Rax, 0));
                }
            }
            None => {
                // 间接调用（函数指针），限定 ≤6 个参数
                let n = args.len().min(ARG_REGS_SYSV.len());
                let pad = if self.stack_depth % 2 == 1 { 8 } else { 0 };
                if pad != 0 {
                    emit!(self, sub_reg_imm32(X86Register::Rsp, 8));
                }
                self.gen_expr(func)?;
                self.push(X86Register::Rax);
                for arg in args.iter().take(n).rev() {
                    self.gen_expr(arg)?;
                    self.push(X86Register::Rax);
                }
                for i in 0..n {
                    self.pop(ARG_REGS_SYSV[i]);
                }
                self.pop(X86Register::R11);
                emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax));
                emit!(self, call_reg(X86Register::R11));
                if pad != 0 {
                    emit!(self, add_reg_imm32(X86Register::Rsp, 8));
                }
            }
        }
        Ok(())
    }

    /// 发射对具名函数的调用：参数值由 `emit_arg(self, i)` 依次产生于 rax。
    /// 负责寄存器/栈分配、16 字节栈对齐、可变参数的 al(向量寄存器数)。
    fn emit_named_call_values<F>(
        &mut self,
        name: &str,
        nargs: usize,
        vec_count: u8,
        mut emit_arg: F,
    ) -> NativeResult<()>
    where
        F: FnMut(&mut Self, usize) -> NativeResult<()>,
    {
        let residual = nargs.saturating_sub(ARG_REGS_SYSV.len());
        let pad = if (self.stack_depth as usize + residual) % 2 == 1 {
            8
        } else {
            0
        };
        if pad != 0 {
            emit!(self, sub_reg_imm32(X86Register::Rsp, 8));
        }
        // 逆序压栈全部实参
        for i in (0..nargs).rev() {
            emit_arg(self, i)?;
            self.push(X86Register::Rax);
        }
        // 前 6 个弹入寄存器，其余留在栈上（顺序已正确）
        let nreg = nargs.min(ARG_REGS_SYSV.len());
        for i in 0..nreg {
            self.pop(ARG_REGS_SYSV[i]);
        }
        if vec_count == 0 {
            emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax));
        } else {
            emit!(self, mov_reg_imm(X86Register::Rax, vec_count as i64));
        }
        self.emit_call_named(name);
        let cleanup = (residual * 8 + pad) as i32;
        if cleanup != 0 {
            emit!(self, add_reg_imm32(X86Register::Rsp, cleanup));
        }
        self.stack_depth -= residual as i32;
        Ok(())
    }

    /// 带寄存器类别（Int/Sse）的具名调用，支持浮点实参（限定寄存器内：≤6 整型、≤8 浮点）。
    /// `vec_count`(al) 自动按 Sse 实参数设置（可变参数 ABI 需要）。
    fn emit_named_call_classified<F>(
        &mut self,
        name: &str,
        classes: &[ArgClass],
        mut emit_arg: F,
    ) -> NativeResult<()>
    where
        F: FnMut(&mut Self, usize) -> NativeResult<()>,
    {
        let nargs = classes.len();
        let pad = if self.stack_depth % 2 == 1 { 8 } else { 0 };
        if pad != 0 {
            emit!(self, sub_reg_imm32(X86Register::Rsp, 8));
        }
        // 逆序压栈全部实参
        for i in (0..nargs).rev() {
            emit_arg(self, i)?;
            self.push(X86Register::Rax);
        }
        // 顺序弹出并按类别分配到 GPR / XMM
        let mut gi = 0usize;
        let mut si = 0usize;
        for &class in classes.iter() {
            self.pop(X86Register::Rax);
            match class {
                ArgClass::Int => {
                    if gi < ARG_REGS_SYSV.len() {
                        emit!(self, mov_reg_reg(ARG_REGS_SYSV[gi], X86Register::Rax));
                    }
                    gi += 1;
                }
                ArgClass::Sse => {
                    if si < SSE_ARG_REGS.len() {
                        emit!(self, movq_xmm_gpr(SSE_ARG_REGS[si], X86Register::Rax));
                    }
                    si += 1;
                }
            }
        }
        emit!(self, mov_reg_imm(X86Register::Rax, si as i64)); // al = 向量寄存器数
        self.emit_call_named(name);
        if pad != 0 {
            emit!(self, add_reg_imm32(X86Register::Rsp, 8));
        }
        Ok(())
    }

    /// 生成 println/print/eprintln 等的 printf/dprintf 调用。
    fn gen_print_call(&mut self, name: &str, args: &[lir::Expression]) -> NativeResult<()> {
        let newline = !matches!(name, "print_inline" | "eprint");
        let to_stderr = matches!(name, "eprintln" | "eprint");

        // 计算每个实参的打印种类与格式串
        let kinds: Vec<PrintKind> = args.iter().map(|a| self.print_kind_of(a)).collect();
        let mut fmt = String::new();
        for (i, k) in kinds.iter().enumerate() {
            if i > 0 {
                fmt.push(' ');
            }
            fmt.push_str(k.spec());
        }
        if newline {
            fmt.push('\n');
        }

        // 组织 printf 实参： [fmt] + args  （eprintln 用 dprintf： [2, fmt] + args）
        let callee = if to_stderr { "dprintf" } else { "printf" };
        let lead = if to_stderr { 2 } else { 1 }; // 引导实参数（fd?+fmt）

        // 实参类别：fd/fmt 为 Int，浮点实参为 Sse，其余 Int
        let mut classes: Vec<ArgClass> = Vec::with_capacity(lead + args.len());
        for _ in 0..lead {
            classes.push(ArgClass::Int);
        }
        for k in &kinds {
            classes.push(if matches!(k, PrintKind::Float) {
                ArgClass::Sse
            } else {
                ArgClass::Int
            });
        }

        let fmt_clone = fmt.clone();
        let kinds_ref = &kinds;
        self.emit_named_call_classified(callee, &classes, |s, i| {
            if to_stderr && i == 0 {
                emit!(s, mov_reg_imm(X86Register::Rax, 2));
                Ok(())
            } else if i == lead - 1 {
                s.emit_lea_string(X86Register::Rax, &fmt_clone);
                Ok(())
            } else {
                let arg = &args[i - lead];
                match kinds_ref[i - lead] {
                    PrintKind::Bool => s.gen_bool_to_cstr(arg),
                    _ => s.gen_expr(arg),
                }
            }
        })
    }

    /// 计算实参在 printf 中应使用的打印种类
    fn print_kind_of(&self, e: &lir::Expression) -> PrintKind {
        match self.infer_expr_type(e) {
            Some(ty) => PrintKind::from_type(&ty),
            None => PrintKind::Int,
        }
    }

    /// 把布尔值（rax 中 0/1）转换为指向 "true"/"false" 的 C 字符串指针（结果在 rax）
    fn gen_bool_to_cstr(&mut self, e: &lir::Expression) -> NativeResult<()> {
        self.gen_expr(e)?;
        let lt = self.new_label("bt");
        let le = self.new_label("be");
        emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
        self.emit_jcc(Condition::NE, &lt);
        self.emit_lea_string(X86Register::Rax, "false");
        self.emit_jmp(&le);
        self.define_label(&lt);
        self.emit_lea_string(X86Register::Rax, "true");
        self.define_label(&le);
        Ok(())
    }

    // ========================================================================
    // 静态类型推断（尽力而为，用于打印/sized 访问）
    // ========================================================================

    fn lit_type(lit: &lir::Literal) -> lir::Type {
        use lir::{Literal, Type};
        match lit {
            Literal::Integer(_) | Literal::Long(_) | Literal::LongLong(_) => Type::Long,
            Literal::UnsignedInteger(_)
            | Literal::UnsignedLong(_)
            | Literal::UnsignedLongLong(_) => Type::Ulong,
            Literal::Float(_) => Type::Float,
            Literal::Double(_) => Type::Double,
            Literal::Char(_) => Type::Char,
            Literal::String(_) => Type::Pointer(Box::new(Type::Char)),
            Literal::Bool(_) => Type::Bool,
            Literal::NullPointer => Type::Pointer(Box::new(Type::Void)),
        }
    }

    fn infer_expr_type(&self, e: &lir::Expression) -> Option<lir::Type> {
        use lir::{BinaryOp, Expression, Type, UnaryOp};
        match e {
            Expression::Literal(l) => Some(Self::lit_type(l)),
            Expression::Variable(n) => self
                .local_and_param_types
                .get(n)
                .cloned()
                .or_else(|| self.global_types.get(n).cloned()),
            Expression::Cast(ty, _) => Some(ty.clone()),
            Expression::Parenthesized(i) => self.infer_expr_type(i),
            Expression::Unary(op, i) => match op {
                UnaryOp::Not => Some(Type::Bool),
                UnaryOp::Reference | UnaryOp::MutableReference => Some(Type::Pointer(Box::new(
                    self.infer_expr_type(i).unwrap_or(Type::Void),
                ))),
                _ => self.infer_expr_type(i),
            },
            Expression::Binary(op, l, _) => {
                if matches!(
                    op,
                    BinaryOp::Equal
                        | BinaryOp::NotEqual
                        | BinaryOp::LessThan
                        | BinaryOp::LessThanEqual
                        | BinaryOp::GreaterThan
                        | BinaryOp::GreaterThanEqual
                        | BinaryOp::LogicalAnd
                        | BinaryOp::LogicalOr
                ) {
                    Some(Type::Bool)
                } else {
                    self.infer_expr_type(l)
                }
            }
            Expression::AddressOf(i) => Some(Type::Pointer(Box::new(
                self.infer_expr_type(i).unwrap_or(Type::Void),
            ))),
            Expression::Dereference(i) => match self.infer_expr_type(i) {
                Some(Type::Pointer(p)) => Some(*p),
                Some(Type::Array(p, _)) => Some(*p),
                _ => None,
            },
            Expression::Index(a, _) => self.index_elem_type(a),
            Expression::Ternary(_, t, _) => self.infer_expr_type(t),
            Expression::Assign(t, _) => self.infer_expr_type(t),
            Expression::Member(o, f) => self.member_field_type(o, f, false),
            Expression::PointerMember(o, f) => self.member_field_type(o, f, true),
            _ => None,
        }
    }

    fn index_elem_type(&self, arr: &lir::Expression) -> Option<lir::Type> {
        match self.infer_expr_type(arr) {
            Some(lir::Type::Pointer(p)) => Some(*p),
            Some(lir::Type::Array(p, _)) => Some(*p),
            _ => None,
        }
    }

    fn member_field_type(
        &self,
        obj: &lir::Expression,
        field: &str,
        via_ptr: bool,
    ) -> Option<lir::Type> {
        let sname = if via_ptr {
            self.infer_pointee_struct_for_expr(obj)
        } else {
            self.infer_aggregate_struct_for_expr(obj)
        }?;
        self.field_types
            .get(&format!("{}::{}", sname, field))
            .cloned()
    }

    /// 把 reg *= size（用于 Index 偏移计算）
    fn scale_reg(&mut self, reg: X86Register, size: usize) {
        match size {
            0 | 1 => {}
            2 => emit!(self, shl_reg_imm8(reg, 1)),
            4 => emit!(self, shl_reg_imm8(reg, 2)),
            8 => emit!(self, shl_reg_imm8(reg, 3)),
            16 => emit!(self, shl_reg_imm8(reg, 4)),
            n if n.is_power_of_two() => {
                emit!(self, shl_reg_imm8(reg, n.trailing_zeros() as u8))
            }
            n => emit!(self, imul_reg_imm32(reg, reg, n as i32)),
        }
    }

    /// 从 [rax] 按元素大小加载到 rax（带符号/零扩展）
    fn emit_sized_load(&mut self, elem_ty: Option<&lir::Type>, size: usize) {
        let signed = elem_ty.map(Self::is_signed_int).unwrap_or(true);
        match size {
            1 => emit!(self, movzx_r64_m8(X86Register::Rax, X86Register::Rax)),
            2 => emit!(self, movzx_r64_m16(X86Register::Rax, X86Register::Rax)),
            4 => {
                if signed {
                    emit!(self, movsxd_r64_m32(X86Register::Rax, X86Register::Rax));
                } else {
                    emit!(self, mov_r32_m32(X86Register::Rax, X86Register::Rax));
                }
            }
            _ => emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax)),
        }
    }

    /// 把 rcx 中的值按大小存入 [rax]
    fn emit_sized_store(&mut self, size: usize) {
        match size {
            1 => emit!(self, mov_mem0_reg8(X86Register::Rax, X86Register::Rcx)),
            2 => emit!(self, mov_mem0_reg16(X86Register::Rax, X86Register::Rcx)),
            4 => emit!(self, mov_mem0_reg32(X86Register::Rax, X86Register::Rcx)),
            _ => emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx)),
        }
    }

    fn is_signed_int(ty: &lir::Type) -> bool {
        use lir::Type;
        let ty = match ty {
            Type::Qualified(_, i) => i.as_ref(),
            t => t,
        };
        matches!(
            ty,
            Type::Char
                | Type::Schar
                | Type::Short
                | Type::Int
                | Type::Long
                | Type::LongLong
                | Type::Ptrdiff
                | Type::Intptr
        )
    }

    /// 前/后自增自减： `pre` 为真返回新值，否则返回旧值。当前支持变量目标。
    fn gen_inc_dec(&mut self, target: &lir::Expression, delta: i64, pre: bool) -> NativeResult<()> {
        use lir::Expression;
        match target {
            Expression::Variable(name) if self.local_offsets.contains_key(name) => {
                let offset = self.local_offsets[name];
                self.load_local(X86Register::Rax, offset);
                if !pre {
                    self.push(X86Register::Rax); // 保存旧值
                }
                if delta >= 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, delta as i32));
                } else {
                    emit!(self, sub_reg_imm32(X86Register::Rax, (-delta) as i32));
                }
                self.store_local(offset, X86Register::Rax);
                if !pre {
                    self.pop(X86Register::Rax); // 恢复旧值作为结果
                }
                Ok(())
            }
            _ => {
                // 退化：求值一次（不保证语义完整）
                self.gen_expr(target)?;
                if delta >= 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, delta as i32));
                } else {
                    emit!(self, sub_reg_imm32(X86Register::Rax, (-delta) as i32));
                }
                Ok(())
            }
        }
    }

    // ========================================================================
    // 类型与字段偏移（移植自原汇编生成器的纯逻辑）
    // ========================================================================

    fn type_size(&self, ty: &lir::Type) -> usize {
        use lir::Type;
        match ty {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            // 原生后端内部统一以 8 字节双精度处理浮点
            Type::Float | Type::Long | Type::Ulong | Type::Double | Type::Pointer(_) => 8,
            Type::LongLong | Type::UlongLong | Type::LongDouble => 16,
            Type::Tuple(items) => items.iter().map(Type::size_of).sum(),
            _ => 8,
        }
    }

    fn type_align(&self, ty: &lir::Type) -> usize {
        self.type_size(ty).max(1)
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
                Self::collect_var_types_stmt(&if_stmt.then_branch, types);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::collect_var_types_stmt(else_branch, types);
                }
            }
            lir::Statement::For(for_stmt) => {
                if let Some(init) = &for_stmt.initializer {
                    Self::collect_var_types_stmt(init, types);
                }
                Self::collect_var_types_stmt(&for_stmt.body, types);
            }
            lir::Statement::While(while_stmt) => {
                Self::collect_var_types_stmt(&while_stmt.body, types);
            }
            lir::Statement::DoWhile(do_while) => {
                Self::collect_var_types_stmt(&do_while.body, types);
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
            let key = format!("{}::{}", s, field);
            if let Some(&o) = self.field_offsets.get(&key) {
                return Some(o);
            }
        }
        self.unique_layout_field_offset(field)
    }
}

fn round_up(n: usize, align: usize) -> usize {
    if align == 0 {
        return n;
    }
    n.div_ceil(align) * align
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lir::{self as lir, Expression, Statement, Type};

    fn gen(program: &lir::Program) -> MachineObject {
        let mut g = MachineCodeGen::new(TargetOS::Linux);
        g.generate(program).unwrap()
    }

    #[test]
    fn test_return_const_has_prologue_and_ret() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(42))));
        program.add(lir::Declaration::Function(func));

        let obj = gen(&program);
        // push rbp; mov rbp,rsp; sub rsp, imm32
        assert_eq!(obj.text[0], 0x55); // push rbp
                                       // 末字节应为 ret
        assert_eq!(*obj.text.last().unwrap(), 0xC3);
        // main 符号存在
        assert!(obj.symbols.iter().any(|s| s.name == "main" && s.is_func));
    }

    #[test]
    fn test_external_call_creates_plt_reloc_and_symbol() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body.statements.push(Statement::Expression(
            Expression::var("puts").call(vec![Expression::string("hi")]),
        ));
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(0))));
        program.add(lir::Declaration::Function(func));

        let obj = gen(&program);
        assert!(obj
            .symbols
            .iter()
            .any(|s| s.name == "puts" && s.section.is_none()));
        assert!(obj
            .relocations
            .iter()
            .any(|r| matches!(&r.target, RelTarget::Symbol(n) if n == "puts")
                && r.kind == RelKind::Plt32));
        // 字符串进入 .rodata
        assert!(obj.rodata.starts_with(b"hi\0"));
        // 存在指向 .rodata 的 PC32 重定位
        assert!(obj
            .relocations
            .iter()
            .any(|r| matches!(r.target, RelTarget::Section(SecKind::Rodata))
                && r.kind == RelKind::Pc32));
    }

    #[test]
    fn test_global_const_init_goes_to_data() {
        let mut program = lir::Program::new();
        program.add(lir::Declaration::Global(lir::GlobalVar {
            name: "g".into(),
            type_: Type::Long,
            initializer: Some(Expression::Literal(lir::Literal::Long(7))),
            is_static: true,
        }));
        let mut main = lir::Function::new("main", Type::Int);
        main.body
            .statements
            .push(Statement::Return(Some(Expression::var("g"))));
        program.add(lir::Declaration::Function(main));

        let obj = gen(&program);
        // 常量初始化写入 .data 的前 8 字节
        assert_eq!(&obj.data[0..8], &7i64.to_le_bytes());
        // 引用全局产生指向 .data 的 PC32 重定位
        assert!(obj
            .relocations
            .iter()
            .any(|r| matches!(r.target, RelTarget::Section(SecKind::Data))));
    }

    #[test]
    fn test_float_add_uses_sse() {
        let mut program = lir::Program::new();
        let mut main = lir::Function::new("main", Type::Double);
        main.body
            .statements
            .push(Statement::Return(Some(Expression::Binary(
                lir::BinaryOp::Add,
                Box::new(Expression::Literal(lir::Literal::Double(1.5))),
                Box::new(Expression::Literal(lir::Literal::Double(2.5))),
            ))));
        program.add(lir::Declaration::Function(main));
        let obj = gen(&program);
        // 含 addsd 前缀 F2 0F 58
        assert!(obj.text.windows(3).any(|w| w == [0xF2, 0x0F, 0x58]));
    }

    #[test]
    fn test_switch_emits_compare_chain() {
        let mut program = lir::Program::new();
        let mut main = lir::Function::new("main", Type::Int);
        main.body
            .statements
            .push(Statement::Switch(lir::SwitchStatement {
                expression: Expression::int(1),
                cases: vec![lir::SwitchCase {
                    value: Expression::int(1),
                    body: Box::new(Statement::Break),
                }],
                default: None,
            }));
        main.body
            .statements
            .push(Statement::Return(Some(Expression::int(0))));
        program.add(lir::Declaration::Function(main));
        let obj = gen(&program);
        // 生成成功且含 cmp (0x39) 指令
        assert!(obj.text.contains(&0x39));
        assert!(obj.symbols.iter().any(|s| s.name == "main"));
    }

    #[test]
    fn test_internal_call_is_resolved_directly() {
        let mut program = lir::Program::new();
        let mut helper = lir::Function::new("helper", Type::Int);
        helper
            .body
            .statements
            .push(Statement::Return(Some(Expression::int(7))));
        program.add(lir::Declaration::Function(helper));

        let mut main = lir::Function::new("main", Type::Int);
        main.body.statements.push(Statement::Return(Some(
            Expression::var("helper").call(vec![]),
        )));
        program.add(lir::Declaration::Function(main));

        let obj = gen(&program);
        // 内部调用不产生重定位、也不产生外部符号
        assert!(!obj
            .symbols
            .iter()
            .any(|s| s.name == "helper" && s.section.is_none()));
        assert!(obj.relocations.is_empty());
    }
}
