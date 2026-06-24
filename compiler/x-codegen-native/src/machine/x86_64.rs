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

/// 全局变量槽（当前一律放入 .bss）
struct GlobalSlot {
    offset: u64,
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
    bss_size: u64,

    /// 字符串 -> .rodata 偏移
    string_offsets: HashMap<String, u64>,
    /// 全局变量名 -> .bss 槽
    globals: HashMap<String, GlobalSlot>,

    /// 本函数局部/参数 -> [rbp - offset]（offset 为正）
    local_offsets: HashMap<String, i32>,
    /// 当前栈帧已用字节
    stack_size: usize,
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
            bss_size: 0,
            string_offsets: HashMap::new(),
            globals: HashMap::new(),
            local_offsets: HashMap::new(),
            stack_size: 0,
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
            data: Vec::new(),
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
                    let offset = round_up(self.bss_size as usize, 8) as u64;
                    self.globals
                        .insert(global.name.clone(), GlobalSlot { offset });
                    self.bss_size = offset + slot;
                }
                lir::Declaration::Struct(strct) => {
                    self.collect_fields(&strct.name, &strct.fields);
                }
                lir::Declaration::Class(cls) => {
                    self.collect_fields(&cls.name, &cls.fields);
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

    /// lea reg, [rip + global]，登记指向 .bss 的 PC32 重定位
    fn emit_lea_global(&mut self, dest: X86Register, name: &str) -> NativeResult<()> {
        let off = self
            .globals
            .get(name)
            .map(|g| g.offset)
            .ok_or_else(|| NativeError::CodegenError(format!("未知全局变量: {}", name)))?;
        emit!(self, lea_reg_rip(dest, 0));
        let field = self.text.len() - 4;
        self.relocations.push(ObjReloc {
            offset: field as u64,
            target: RelTarget::Section(SecKind::Bss),
            kind: RelKind::Pc32,
            addend: off as i64 - 4,
        });
        Ok(())
    }

    fn write_i32(&mut self, at: usize, val: i32) {
        self.text[at..at + 4].copy_from_slice(&val.to_le_bytes());
    }

    /// 在函数布局完成后回填函数内跳转（标签为函数局部，避免跨函数串名）
    fn resolve_jumps(&mut self) -> NativeResult<()> {
        let jumps = std::mem::take(&mut self.jump_fixups);
        for jf in jumps {
            let target = *self.labels.get(&jf.label).ok_or_else(|| {
                NativeError::CodegenError(format!("未定义标签: {}", jf.label))
            })?;
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

    // ========================================================================
    // 函数
    // ========================================================================

    fn gen_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        let start = self.text.len() as u64;
        self.func_offsets.insert(func.name.clone(), start);

        self.local_offsets.clear();
        self.stack_size = 0;
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

        // 参数落栈
        let nparams = func.parameters.len().min(ARG_REGS_SYSV.len());
        self.stack_size = nparams * 8;
        for (i, param) in func.parameters.iter().enumerate().take(nparams) {
            let offset = ((i + 1) * 8) as i32;
            self.local_offsets.insert(param.name.clone(), offset);
            self.store_local(offset, ARG_REGS_SYSV[i]);
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
            Statement::Empty => {}
            _ => {
                log::debug!("native: 未支持的语句 {:?}", std::mem::discriminant(stmt));
            }
        }
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
                    return Err(NativeError::CodegenError(format!(
                        "未定义变量: {}",
                        name
                    )));
                }
                Ok(())
            }
            Expression::Unary(op, e) => {
                self.gen_expr(e)?;
                match op {
                    UnaryOp::Minus => emit!(self, neg_reg(X86Register::Rax)),
                    UnaryOp::BitNot => emit!(self, not_reg(X86Register::Rax)),
                    UnaryOp::Not => {
                        emit!(self, test_reg_reg(X86Register::Rax, X86Register::Rax));
                        emit!(self, setcc(Condition::E, X86Register::Al));
                        emit!(self, movzx_r64_r8(X86Register::Rax, X86Register::Al));
                    }
                    _ => {}
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
            Expression::Cast(_, inner) => self.gen_expr(inner),
            Expression::Parenthesized(inner) => self.gen_expr(inner),
            Expression::AddressOf(inner) => match inner.as_ref() {
                Expression::Variable(name) => {
                    if let Some(&offset) = self.local_offsets.get(name) {
                        emit!(self, lea_reg_mem(X86Register::Rax, X86Register::Rbp, -offset));
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
                self.gen_expr(arr)?;
                emit!(self, push_reg(X86Register::Rax));
                self.gen_expr(idx)?;
                emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
                emit!(self, pop_reg(X86Register::Rax));
                emit!(self, shl_reg_imm8(X86Register::Rcx, 3));
                emit!(self, add_reg_reg(X86Register::Rax, X86Register::Rcx));
                emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
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
            Literal::Float(f) => emit!(self, mov_reg_imm(X86Register::Rax, *f as i64)),
            Literal::Double(f) => emit!(self, mov_reg_imm(X86Register::Rax, *f as i64)),
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

        self.gen_expr(left)?;
        emit!(self, push_reg(X86Register::Rax));
        self.gen_expr(right)?;
        emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
        emit!(self, pop_reg(X86Register::Rax));

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
                emit!(self, push_reg(X86Register::Rax));
                self.gen_expr(ptr)?;
                emit!(self, pop_reg(X86Register::Rcx));
                emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
            }
            Expression::Member(obj, field) => {
                self.gen_expr(value)?;
                emit!(self, push_reg(X86Register::Rax));
                self.gen_expr(obj)?;
                let offset = self.resolve_field_offset(obj, field, false).unwrap_or(0);
                if offset > 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, offset as i32));
                }
                emit!(self, pop_reg(X86Register::Rcx));
                emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
            }
            Expression::PointerMember(obj, field) => {
                self.gen_expr(value)?;
                emit!(self, push_reg(X86Register::Rax));
                self.gen_expr(obj)?;
                emit!(self, mov_reg_mem0(X86Register::Rax, X86Register::Rax));
                let offset = self.resolve_field_offset(obj, field, true).unwrap_or(0);
                if offset > 0 {
                    emit!(self, add_reg_imm32(X86Register::Rax, offset as i32));
                }
                emit!(self, pop_reg(X86Register::Rcx));
                emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
            }
            Expression::Index(arr, idx) => {
                self.gen_expr(value)?;
                emit!(self, push_reg(X86Register::Rax));
                self.gen_expr(arr)?;
                emit!(self, push_reg(X86Register::Rax));
                self.gen_expr(idx)?;
                emit!(self, shl_reg_imm8(X86Register::Rax, 3));
                emit!(self, mov_reg_reg(X86Register::Rcx, X86Register::Rax));
                emit!(self, pop_reg(X86Register::Rax));
                emit!(self, add_reg_reg(X86Register::Rax, X86Register::Rcx));
                emit!(self, pop_reg(X86Register::Rcx));
                emit!(self, mov_mem0_reg(X86Register::Rax, X86Register::Rcx));
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
        let binexpr = lir::Expression::Binary(
            op,
            Box::new(target.clone()),
            Box::new(value.clone()),
        );
        self.gen_assign(target, &binexpr)
    }

    fn gen_call(&mut self, func: &lir::Expression, args: &[lir::Expression]) -> NativeResult<()> {
        use lir::Expression;
        let n = args.len().min(ARG_REGS_SYSV.len());

        // 仅当被调用者是“具名函数符号”（内部定义或外部 libc）时走直接调用；
        // 若被调用者是局部/全局变量（函数指针值），则走间接调用。
        let direct_name = match func {
            Expression::Variable(name)
                if !self.local_offsets.contains_key(name)
                    && !self.globals.contains_key(name) =>
            {
                Some(name.clone())
            }
            _ => None,
        };

        match direct_name {
            Some(name) => {
                for arg in args.iter().take(n).rev() {
                    self.gen_expr(arg)?;
                    emit!(self, push_reg(X86Register::Rax));
                }
                for i in 0..n {
                    emit!(self, pop_reg(ARG_REGS_SYSV[i]));
                }
                // 可变参数 ABI：al = 使用的向量寄存器个数（这里置 0）
                emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax));
                self.emit_call_named(&name);
            }
            None => {
                // 间接调用：先求函数指针入栈，再求参数，最后 call r11
                self.gen_expr(func)?;
                emit!(self, push_reg(X86Register::Rax));
                for arg in args.iter().take(n).rev() {
                    self.gen_expr(arg)?;
                    emit!(self, push_reg(X86Register::Rax));
                }
                for i in 0..n {
                    emit!(self, pop_reg(ARG_REGS_SYSV[i]));
                }
                emit!(self, pop_reg(X86Register::R11));
                emit!(self, xor_reg_reg(X86Register::Rax, X86Register::Rax));
                emit!(self, call_reg(X86Register::R11));
            }
        }
        Ok(())
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
            Type::Int | Type::Uint | Type::Float => 4,
            Type::Long | Type::Ulong | Type::Double | Type::Pointer(_) => 8,
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
        assert!(obj.symbols.iter().any(|s| s.name == "puts" && s.section.is_none()));
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
        assert!(!obj.symbols.iter().any(|s| s.name == "helper" && s.section.is_none()));
        assert!(obj.relocations.is_empty());
    }
}
