//! AArch64 (ARM64) 直出机器码生成器 (AAPCS64, Linux)
//!
//! 把 LIR 直接降级为 `.text` 机器码（定长 32 位指令），收集字符串/全局数据、
//! 符号表与重定位。函数内分支以 fixup 回填；外部调用与全局/字符串引用以
//! `R_AARCH64_*` 重定位交给链接器。
//!
//! 值模型：标量结果置于 `x0`。临时值通过 `str x0,[sp,#-16]!` /
//! `ldr x0,[sp],#16` 入/出栈（每个临时占 16 字节，保持 sp 16 字节对齐）。
//! 当前覆盖：整型/指针标量、控制流、调用（≤8 整型实参）、println/print、
//! 全局 .data/.bss、字符串。浮点与聚合体暂以整型路径近似处理。

use std::collections::{HashMap, HashSet};

use crate::{NativeError, NativeResult, TargetOS};
use x_lir as lir;

use super::{MachineObject, ObjReloc, ObjSymbol, RelKind, RelTarget, SecKind};

const ARG_REGS: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
const SP: u8 = 31;
const FP: u8 = 29;
const LR: u8 = 30;

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Cond {
    Eq = 0,
    Ne = 1,
    Mi = 4,
    Hi = 8,
    Ls = 9,
    Ge = 10,
    Lt = 11,
    Gt = 12,
    Le = 13,
}

/// 打印种类
#[derive(Clone, Copy, PartialEq)]
enum PK {
    Int,
    Str,
    Char,
    Bool,
    Ptr,
}
impl PK {
    fn spec(self) -> &'static str {
        match self {
            PK::Int => "%lld",
            PK::Str => "%s",
            PK::Char => "%c",
            PK::Bool => "%s",
            PK::Ptr => "%p",
        }
    }
    fn from_type(ty: &lir::Type) -> Self {
        use lir::Type;
        let ty = match ty {
            Type::Qualified(_, i) => i.as_ref(),
            t => t,
        };
        match ty {
            Type::Bool => PK::Bool,
            Type::Char | Type::Schar | Type::Uchar => PK::Char,
            Type::Pointer(inner) => {
                let inner = match inner.as_ref() {
                    Type::Qualified(_, i) => i.as_ref(),
                    t => t,
                };
                if matches!(inner, Type::Char | Type::Schar | Type::Uchar) {
                    PK::Str
                } else {
                    PK::Ptr
                }
            }
            _ => PK::Int,
        }
    }
}

struct JumpFixup {
    field: usize,
    label: String,
    is_cond: bool,
}
struct CallFixup {
    field: usize,
    callee: String,
}
struct GlobalSlot {
    offset: u64,
    section: SecKind,
}
struct FuncSym {
    name: String,
    offset: u64,
    size: u64,
}

pub struct Aarch64CodeGen {
    #[allow(dead_code)]
    os: TargetOS,
    text: Vec<u8>,
    rodata: Vec<u8>,
    data: Vec<u8>,
    bss_size: u64,
    string_offsets: HashMap<String, u64>,
    globals: HashMap<String, GlobalSlot>,
    global_types: HashMap<String, lir::Type>,
    field_offsets: HashMap<String, usize>,
    field_types: HashMap<String, lir::Type>,
    local_offsets: HashMap<String, i32>,
    local_and_param_types: HashMap<String, lir::Type>,
    stack_size: usize,
    stack_depth: i32,
    frame: i32,
    labels: HashMap<String, usize>,
    label_counter: usize,
    jump_fixups: Vec<JumpFixup>,
    call_fixups: Vec<CallFixup>,
    loop_labels: Vec<(String, String)>,
    func_syms: Vec<FuncSym>,
    external_syms: Vec<String>,
    defined_functions: HashSet<String>,
    relocations: Vec<ObjReloc>,
}

impl Aarch64CodeGen {
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
            field_offsets: HashMap::new(),
            field_types: HashMap::new(),
            local_offsets: HashMap::new(),
            local_and_param_types: HashMap::new(),
            stack_size: 0,
            stack_depth: 0,
            frame: 0,
            labels: HashMap::new(),
            label_counter: 0,
            jump_fixups: Vec::new(),
            call_fixups: Vec::new(),
            loop_labels: Vec::new(),
            func_syms: Vec::new(),
            external_syms: Vec::new(),
            defined_functions: HashSet::new(),
            relocations: Vec::new(),
        }
    }

    fn w(&mut self, word: u32) {
        self.text.extend_from_slice(&word.to_le_bytes());
    }

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

    // ====================== 指令编码 ======================
    fn emit_mov_imm(&mut self, rd: u8, val: i64) {
        let u = val as u64;
        let parts = [
            (u & 0xffff) as u32,
            ((u >> 16) & 0xffff) as u32,
            ((u >> 32) & 0xffff) as u32,
            ((u >> 48) & 0xffff) as u32,
        ];
        // movz rd, parts[0]
        self.w(0xD2800000 | (parts[0] << 5) | rd as u32);
        for hw in 1..4u32 {
            if parts[hw as usize] != 0 {
                self.w(0xF2800000 | (hw << 21) | (parts[hw as usize] << 5) | rd as u32);
            }
        }
    }
    fn emit_mov_reg(&mut self, rd: u8, rm: u8) {
        // orr rd, xzr, rm
        self.w(0xAA0003E0 | ((rm as u32) << 16) | rd as u32);
    }
    fn alu_rrr(&mut self, base: u32, rd: u8, rn: u8, rm: u8) {
        self.w(base | ((rm as u32) << 16) | ((rn as u32) << 5) | rd as u32);
    }
    fn emit_add(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0x8B000000, rd, rn, rm);
    }
    fn emit_sub(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0xCB000000, rd, rn, rm);
    }
    fn emit_mul(&mut self, rd: u8, rn: u8, rm: u8) {
        self.w(0x9B007C00 | ((rm as u32) << 16) | ((rn as u32) << 5) | rd as u32);
    }
    fn emit_sdiv(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0x9AC00C00, rd, rn, rm);
    }
    fn emit_and(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0x8A000000, rd, rn, rm);
    }
    fn emit_orr(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0xAA000000, rd, rn, rm);
    }
    fn emit_eor(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0xCA000000, rd, rn, rm);
    }
    fn emit_lslv(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0x9AC02000, rd, rn, rm);
    }
    fn emit_lsrv(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0x9AC02400, rd, rn, rm);
    }
    fn emit_asrv(&mut self, rd: u8, rn: u8, rm: u8) {
        self.alu_rrr(0x9AC02800, rd, rn, rm);
    }
    fn emit_add_imm(&mut self, rd: u8, rn: u8, imm: u32) {
        self.w(0x91000000 | ((imm & 0xFFF) << 10) | ((rn as u32) << 5) | rd as u32);
    }

    // ── 浮点（double，scalar）────────────────────────────────────────────
    /// fmov d<dd>, x<xn>（GPR 位 → 浮点寄存器）
    fn emit_fmov_d_from_x(&mut self, dd: u8, xn: u8) {
        self.w(0x9E670000 | ((xn as u32) << 5) | dd as u32);
    }
    /// fmov x<xd>, d<dn>（浮点寄存器 → GPR 位）
    fn emit_fmov_x_from_d(&mut self, xd: u8, dn: u8) {
        self.w(0x9E660000 | ((dn as u32) << 5) | xd as u32);
    }
    /// scvtf d<dd>, x<xn>（有符号整数 → double）
    fn emit_scvtf(&mut self, dd: u8, xn: u8) {
        self.w(0x9E620000 | ((xn as u32) << 5) | dd as u32);
    }
    fn emit_fadd(&mut self, dd: u8, dn: u8, dm: u8) {
        self.w(0x1E602800 | ((dm as u32) << 16) | ((dn as u32) << 5) | dd as u32);
    }
    fn emit_fsub(&mut self, dd: u8, dn: u8, dm: u8) {
        self.w(0x1E603800 | ((dm as u32) << 16) | ((dn as u32) << 5) | dd as u32);
    }
    fn emit_fmul(&mut self, dd: u8, dn: u8, dm: u8) {
        self.w(0x1E600800 | ((dm as u32) << 16) | ((dn as u32) << 5) | dd as u32);
    }
    fn emit_fdiv(&mut self, dd: u8, dn: u8, dm: u8) {
        self.w(0x1E601800 | ((dm as u32) << 16) | ((dn as u32) << 5) | dd as u32);
    }
    fn emit_fcmp(&mut self, dn: u8, dm: u8) {
        self.w(0x1E602000 | ((dm as u32) << 16) | ((dn as u32) << 5));
    }
    #[allow(dead_code)]
    fn emit_sub_imm(&mut self, rd: u8, rn: u8, imm: u32) {
        self.w(0xD1000000 | ((imm & 0xFFF) << 10) | ((rn as u32) << 5) | rd as u32);
    }
    fn emit_cmp(&mut self, rn: u8, rm: u8) {
        // subs xzr, rn, rm
        self.w(0xEB000000 | ((rm as u32) << 16) | ((rn as u32) << 5) | 31);
    }
    fn emit_cset(&mut self, rd: u8, cond: Cond) {
        let inv = (cond as u32) ^ 1;
        self.w(0x9A9F07E0 | (inv << 12) | rd as u32);
    }
    fn emit_neg(&mut self, rd: u8, rm: u8) {
        self.w(0xCB0003E0 | ((rm as u32) << 16) | rd as u32);
    }
    fn emit_mvn(&mut self, rd: u8, rm: u8) {
        self.w(0xAA2003E0 | ((rm as u32) << 16) | rd as u32);
    }
    /// ldr xt,[xn,#off]（off 必须为 8 的倍数且非负，使用无符号缩放偏移）
    fn emit_ldr(&mut self, rt: u8, rn: u8, off: i32) {
        if off >= 0 && off % 8 == 0 && (off / 8) < 4096 {
            let imm12 = (off / 8) as u32;
            self.w(0xF9400000 | (imm12 << 10) | ((rn as u32) << 5) | rt as u32);
        } else {
            // LDUR（simm9）
            let imm9 = (off as u32) & 0x1FF;
            self.w(0xF8400000 | (imm9 << 12) | ((rn as u32) << 5) | rt as u32);
        }
    }
    fn emit_str(&mut self, rt: u8, rn: u8, off: i32) {
        if off >= 0 && off % 8 == 0 && (off / 8) < 4096 {
            let imm12 = (off / 8) as u32;
            self.w(0xF9000000 | (imm12 << 10) | ((rn as u32) << 5) | rt as u32);
        } else {
            let imm9 = (off as u32) & 0x1FF;
            self.w(0xF8000000 | (imm9 << 12) | ((rn as u32) << 5) | rt as u32);
        }
    }
    /// ldr xt,[xn] (off=0)
    fn emit_ldr0(&mut self, rt: u8, rn: u8) {
        self.w(0xF9400000 | ((rn as u32) << 5) | rt as u32);
    }
    fn emit_str0(&mut self, rt: u8, rn: u8) {
        self.w(0xF9000000 | ((rn as u32) << 5) | rt as u32);
    }
    fn push(&mut self, rt: u8) {
        // str rt,[sp,#-16]!
        let imm9 = (-16i32 as u32) & 0x1FF;
        self.w(0xF8000C00 | (imm9 << 12) | ((SP as u32) << 5) | rt as u32);
        self.stack_depth += 1;
    }
    fn pop(&mut self, rt: u8) {
        // ldr rt,[sp],#16
        let imm9 = (16u32) & 0x1FF;
        self.w(0xF8400400 | (imm9 << 12) | ((SP as u32) << 5) | rt as u32);
        self.stack_depth -= 1;
    }
    fn emit_stp_fp_lr_pre(&mut self, imm: i64) {
        let imm7 = ((imm / 8) as u32) & 0x7F;
        self.w(0xA9800000 | (imm7 << 15) | ((LR as u32) << 10) | ((SP as u32) << 5) | FP as u32);
    }
    fn emit_ldp_fp_lr_post(&mut self, imm: i64) {
        let imm7 = ((imm / 8) as u32) & 0x7F;
        self.w(0xA8C00000 | (imm7 << 15) | ((LR as u32) << 10) | ((SP as u32) << 5) | FP as u32);
    }
    fn emit_mov_fp_sp(&mut self) {
        // add x29, sp, #0
        self.w(0x910003FD);
    }
    fn emit_mov_sp_fp(&mut self) {
        // add sp, x29, #0
        self.w(0x910003BF);
    }
    fn emit_ret(&mut self) {
        self.w(0xD65F03C0);
    }
    fn emit_blr(&mut self, rn: u8) {
        self.w(0xD63F0000 | ((rn as u32) << 5));
    }

    // ====================== 标签/分支/调用 fixup ======================
    fn new_label(&mut self, prefix: &str) -> String {
        let l = format!("L_{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        l
    }
    fn define_label(&mut self, name: &str) {
        self.labels.insert(name.to_string(), self.text.len());
    }
    fn emit_b(&mut self, label: &str) {
        let field = self.text.len();
        self.w(0x14000000);
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
            is_cond: false,
        });
    }
    fn emit_bcond(&mut self, cond: Cond, label: &str) {
        let field = self.text.len();
        self.w(0x54000000 | (cond as u32));
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
            is_cond: true,
        });
    }
    fn emit_bl(&mut self, callee: &str) {
        let field = self.text.len();
        self.w(0x94000000);
        self.call_fixups.push(CallFixup {
            field,
            callee: callee.to_string(),
        });
    }

    fn resolve_jumps(&mut self) -> NativeResult<()> {
        let jumps = std::mem::take(&mut self.jump_fixups);
        for jf in jumps {
            let target = *self
                .labels
                .get(&jf.label)
                .ok_or_else(|| NativeError::CodegenError(format!("未定义标签: {}", jf.label)))?;
            let rel = (target as i64 - jf.field as i64) / 4;
            let word = u32::from_le_bytes(self.text[jf.field..jf.field + 4].try_into().unwrap());
            let patched = if jf.is_cond {
                (word & !(0x7FFFF << 5)) | (((rel as u32) & 0x7FFFF) << 5)
            } else {
                (word & !0x3FFFFFF) | ((rel as u32) & 0x3FFFFFF)
            };
            self.text[jf.field..jf.field + 4].copy_from_slice(&patched.to_le_bytes());
        }
        Ok(())
    }

    fn resolve_calls(&mut self) -> NativeResult<()> {
        let calls = std::mem::take(&mut self.call_fixups);
        for cf in calls {
            if let Some(f) = self.func_syms.iter().find(|f| f.name == cf.callee) {
                let rel = (f.offset as i64 - cf.field as i64) / 4;
                let word =
                    u32::from_le_bytes(self.text[cf.field..cf.field + 4].try_into().unwrap());
                let patched = (word & !0x3FFFFFF) | ((rel as u32) & 0x3FFFFFF);
                self.text[cf.field..cf.field + 4].copy_from_slice(&patched.to_le_bytes());
            } else {
                if !self.external_syms.iter().any(|s| s == &cf.callee) {
                    self.external_syms.push(cf.callee.clone());
                }
                self.relocations.push(ObjReloc {
                    offset: cf.field as u64,
                    target: RelTarget::Symbol(cf.callee.clone()),
                    kind: RelKind::Aarch64Call26,
                    addend: 0,
                });
            }
        }
        Ok(())
    }

    fn intern_string(&mut self, s: &str) -> u64 {
        if let Some(&o) = self.string_offsets.get(s) {
            return o;
        }
        let o = self.rodata.len() as u64;
        self.rodata.extend_from_slice(s.as_bytes());
        self.rodata.push(0);
        self.string_offsets.insert(s.to_string(), o);
        o
    }

    /// 计算 section 内某偏移的地址到 rd：adrp + add（两条重定位）
    fn emit_section_addr(&mut self, rd: u8, section: SecKind, offset: u64) {
        let adrp_field = self.text.len();
        self.w(0x90000000 | rd as u32); // adrp rd, 0
        self.relocations.push(ObjReloc {
            offset: adrp_field as u64,
            target: RelTarget::Section(section),
            kind: RelKind::Aarch64AdrPrelPgHi21,
            addend: offset as i64,
        });
        let add_field = self.text.len();
        self.emit_add_imm(rd, rd, 0); // add rd, rd, #:lo12:
        self.relocations.push(ObjReloc {
            offset: add_field as u64,
            target: RelTarget::Section(section),
            kind: RelKind::Aarch64AddAbsLo12Nc,
            addend: offset as i64,
        });
    }

    fn emit_lea_string(&mut self, rd: u8, s: &str) {
        let off = self.intern_string(s);
        self.emit_section_addr(rd, SecKind::Rodata, off);
    }

    fn emit_lea_global(&mut self, rd: u8, name: &str) -> NativeResult<()> {
        let (off, sec) = self
            .globals
            .get(name)
            .map(|g| (g.offset, g.section))
            .ok_or_else(|| NativeError::CodegenError(format!("未知全局变量: {}", name)))?;
        self.emit_section_addr(rd, sec, off);
        Ok(())
    }

    fn store_local(&mut self, off: i32, rt: u8) {
        self.emit_str(rt, FP, off);
    }
    fn load_local(&mut self, rt: u8, off: i32) {
        self.emit_ldr(rt, FP, off);
    }

    // ====================== 布局 ======================
    fn collect_layout(&mut self, program: &lir::Program) {
        for decl in &program.declarations {
            match decl {
                lir::Declaration::Global(g) => {
                    let size = self.type_size(&g.type_).max(8);
                    let slot = round_up(size, 8) as u64;
                    self.global_types.insert(g.name.clone(), g.type_.clone());
                    let init = g
                        .initializer
                        .as_ref()
                        .and_then(|e| self.const_init_bytes(&g.type_, e));
                    if let Some(mut bytes) = init {
                        let offset = round_up(self.data.len(), 8) as u64;
                        while (self.data.len() as u64) < offset {
                            self.data.push(0);
                        }
                        bytes.resize(slot as usize, 0);
                        self.data.extend_from_slice(&bytes);
                        self.globals.insert(
                            g.name.clone(),
                            GlobalSlot {
                                offset,
                                section: SecKind::Data,
                            },
                        );
                    } else {
                        let offset = round_up(self.bss_size as usize, 8) as u64;
                        self.globals.insert(
                            g.name.clone(),
                            GlobalSlot {
                                offset,
                                section: SecKind::Bss,
                            },
                        );
                        self.bss_size = offset + slot;
                    }
                }
                lir::Declaration::Struct(s) => self.collect_fields(&s.name, &s.fields),
                lir::Declaration::Class(c) => self.collect_fields(&c.name, &c.fields),
                _ => {}
            }
        }
    }

    fn collect_fields(&mut self, name: &str, fields: &[lir::Field]) {
        let mut off = 0usize;
        for f in fields {
            let align = self.type_size(&f.type_).max(1);
            if off % align != 0 {
                off += align - (off % align);
            }
            self.field_offsets
                .insert(format!("{}::{}", name, f.name), off);
            self.field_types
                .insert(format!("{}::{}", name, f.name), f.type_.clone());
            off += self.type_size(&f.type_);
        }
    }

    fn const_init_bytes(&self, ty: &lir::Type, e: &lir::Expression) -> Option<Vec<u8>> {
        use lir::{Expression, Literal};
        let e = match e {
            Expression::Parenthesized(i) | Expression::Cast(_, i) => i.as_ref(),
            o => o,
        };
        let lit = match e {
            Expression::Literal(l) => l,
            _ => return None,
        };
        let size = self.type_size(ty).max(1);
        let ib = |v: i64, n: usize| v.to_le_bytes()[..n.min(8)].to_vec();
        Some(match lit {
            Literal::Integer(v) | Literal::Long(v) | Literal::LongLong(v) => ib(*v, size),
            Literal::UnsignedInteger(v)
            | Literal::UnsignedLong(v)
            | Literal::UnsignedLongLong(v) => ib(*v as i64, size),
            Literal::Bool(b) => vec![*b as u8],
            Literal::Char(c) => ib(*c as i64, size),
            Literal::Float(f) => (*f as f64).to_le_bytes().to_vec(),
            Literal::Double(f) => f.to_le_bytes().to_vec(),
            Literal::NullPointer => ib(0, size),
            Literal::String(_) => return None,
        })
    }

    // ====================== 函数 ======================
    fn gen_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        let start = self.text.len() as u64;
        self.labels.clear();
        self.jump_fixups.clear();
        self.local_offsets.clear();
        self.local_and_param_types.clear();
        self.loop_labels.clear();
        self.stack_depth = 0;

        for p in &func.parameters {
            self.local_and_param_types
                .insert(p.name.clone(), p.type_.clone());
        }
        for s in &func.body.statements {
            Self::collect_var_types(s, &mut self.local_and_param_types);
        }

        // 局部布局：[x29 + 16 + k*8]
        let mut disp = 16i32;
        for p in &func.parameters {
            self.local_offsets.insert(p.name.clone(), disp);
            disp += 8;
        }
        let mut varnames = Vec::new();
        for s in &func.body.statements {
            Self::collect_var_names(s, &mut varnames);
        }
        for v in varnames {
            self.local_offsets.entry(v).or_insert_with(|| {
                let d = disp;
                disp += 8;
                d
            });
        }
        self.stack_size = disp as usize;
        let frame = round_up(self.stack_size, 16) as i32;
        self.frame = frame;

        self.emit_stp_fp_lr_pre(-(frame as i64));
        self.emit_mov_fp_sp();

        for (i, p) in func.parameters.iter().enumerate().take(ARG_REGS.len()) {
            let off = self.local_offsets[&p.name];
            self.store_local(off, ARG_REGS[i]);
        }

        self.gen_block(&func.body)?;
        self.gen_epilogue();
        self.resolve_jumps()?;

        let size = self.text.len() as u64 - start;
        self.func_syms.push(FuncSym {
            name: func.name.clone(),
            offset: start,
            size,
        });
        Ok(())
    }

    fn gen_epilogue(&mut self) {
        self.emit_mov_sp_fp();
        self.emit_ldp_fp_lr_post(self.frame as i64);
        self.emit_ret();
    }

    fn gen_block(&mut self, block: &lir::Block) -> NativeResult<()> {
        for s in &block.statements {
            self.gen_statement(s)?;
        }
        Ok(())
    }

    fn gen_statement(&mut self, stmt: &lir::Statement) -> NativeResult<()> {
        use lir::Statement;
        match stmt {
            Statement::Expression(e) => {
                self.gen_expr(e)?;
            }
            Statement::Variable(v) => {
                if let Some(init) = &v.initializer {
                    self.gen_expr(init)?;
                    let off = self.local_offsets[&v.name];
                    self.store_local(off, 0);
                }
            }
            Statement::Return(Some(e)) => {
                self.gen_expr(e)?;
                self.gen_epilogue();
            }
            Statement::Return(None) => {
                self.emit_mov_imm(0, 0);
                self.gen_epilogue();
            }
            Statement::If(s) => {
                let else_l = self.new_label("else");
                let end_l = self.new_label("endif");
                self.gen_expr(&s.condition)?;
                self.emit_cmp_zero_branch_eq(&else_l);
                self.gen_statement(&s.then_branch)?;
                self.emit_b(&end_l);
                self.define_label(&else_l);
                if let Some(eb) = &s.else_branch {
                    self.gen_statement(eb)?;
                }
                self.define_label(&end_l);
            }
            Statement::While(s) => {
                let start_l = self.new_label("wstart");
                let end_l = self.new_label("wend");
                self.loop_labels.push((start_l.clone(), end_l.clone()));
                self.define_label(&start_l);
                self.gen_expr(&s.condition)?;
                self.emit_cmp_zero_branch_eq(&end_l);
                self.gen_statement(&s.body)?;
                self.emit_b(&start_l);
                self.define_label(&end_l);
                self.loop_labels.pop();
            }
            Statement::DoWhile(s) => {
                let start_l = self.new_label("dstart");
                let cond_l = self.new_label("dcond");
                let end_l = self.new_label("dend");
                self.loop_labels.push((cond_l.clone(), end_l.clone()));
                self.define_label(&start_l);
                self.gen_statement(&s.body)?;
                self.define_label(&cond_l);
                self.gen_expr(&s.condition)?;
                self.emit_cmp_zero_branch_ne(&start_l);
                self.define_label(&end_l);
                self.loop_labels.pop();
            }
            Statement::For(s) => {
                let start_l = self.new_label("fstart");
                let end_l = self.new_label("fend");
                if let Some(init) = &s.initializer {
                    self.gen_statement(init)?;
                }
                self.loop_labels.push((start_l.clone(), end_l.clone()));
                self.define_label(&start_l);
                if let Some(c) = &s.condition {
                    self.gen_expr(c)?;
                    self.emit_cmp_zero_branch_eq(&end_l);
                }
                self.gen_statement(&s.body)?;
                if let Some(inc) = &s.increment {
                    self.gen_expr(inc)?;
                }
                self.emit_b(&start_l);
                self.define_label(&end_l);
                self.loop_labels.pop();
            }
            Statement::Compound(b) => self.gen_block(b)?,
            Statement::Break => {
                if let Some((_, e)) = self.loop_labels.last() {
                    let e = e.clone();
                    self.emit_b(&e);
                }
            }
            Statement::Continue => {
                if let Some((c, _)) = self.loop_labels.last() {
                    let c = c.clone();
                    self.emit_b(&c);
                }
            }
            Statement::Label(n) => self.define_label(n),
            Statement::Goto(n) => {
                let n = n.clone();
                self.emit_b(&n);
            }
            Statement::Switch(sw) => self.gen_switch(sw)?,
            Statement::Empty => {}
            _ => {
                log::debug!("aarch64: 未支持的语句 {:?}", std::mem::discriminant(stmt));
            }
        }
        Ok(())
    }

    /// x0==0 则跳 label（cmp x0,#0 等价：cbz x0,label）
    fn emit_cmp_zero_branch_eq(&mut self, label: &str) {
        // cbz x0, label
        let field = self.text.len();
        self.w(0xB4000000); // cbz x0
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
            is_cond: true,
        });
    }
    fn emit_cmp_zero_branch_ne(&mut self, label: &str) {
        // cbnz x0, label
        let field = self.text.len();
        self.w(0xB5000000);
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
            is_cond: true,
        });
    }

    fn gen_switch(&mut self, sw: &lir::SwitchStatement) -> NativeResult<()> {
        let end = self.new_label("swend");
        self.loop_labels.push((end.clone(), end.clone()));
        self.gen_expr(&sw.expression)?;
        self.push(0); // 被测值入栈
        let case_labels: Vec<String> = (0..sw.cases.len())
            .map(|_| self.new_label("case"))
            .collect();
        let default_l = self.new_label("swdefault");
        for (i, c) in sw.cases.iter().enumerate() {
            self.gen_expr(&c.value)?; // x0 = case value
            self.emit_mov_reg(1, 0); // x1 = value
            self.emit_ldr(0, SP, 0); // x0 = 被测值（栈顶）
            self.emit_cmp(0, 1);
            self.emit_bcond(Cond::Eq, &case_labels[i]);
        }
        self.emit_b(&default_l);
        for (i, c) in sw.cases.iter().enumerate() {
            self.define_label(&case_labels[i]);
            self.gen_statement(&c.body)?;
        }
        self.define_label(&default_l);
        if let Some(d) = &sw.default {
            self.gen_statement(d)?;
        }
        self.define_label(&end);
        self.pop(0);
        self.loop_labels.pop();
        Ok(())
    }

    // ====================== 表达式（结果入 x0） ======================
    fn gen_expr(&mut self, expr: &lir::Expression) -> NativeResult<()> {
        use lir::{Expression, UnaryOp};
        match expr {
            Expression::Literal(l) => self.gen_literal(l),
            Expression::Variable(name) => {
                if let Some(&off) = self.local_offsets.get(name) {
                    self.load_local(0, off);
                } else if self.globals.contains_key(name) {
                    self.emit_lea_global(0, name)?;
                    self.emit_ldr0(0, 0);
                } else {
                    return Err(NativeError::CodegenError(format!("未定义变量: {}", name)));
                }
                Ok(())
            }
            Expression::Unary(op, e) => {
                match op {
                    UnaryOp::Minus => {
                        self.gen_expr(e)?;
                        self.emit_neg(0, 0);
                    }
                    UnaryOp::BitNot => {
                        self.gen_expr(e)?;
                        self.emit_mvn(0, 0);
                    }
                    UnaryOp::Plus => self.gen_expr(e)?,
                    UnaryOp::Not => {
                        self.gen_expr(e)?;
                        self.emit_cmp_imm0(0);
                        self.emit_cset(0, Cond::Eq);
                    }
                    UnaryOp::Reference | UnaryOp::MutableReference => {
                        return self.gen_expr(&Expression::AddressOf(e.clone()));
                    }
                    UnaryOp::PreIncrement | UnaryOp::PreDecrement => {
                        let d = if matches!(op, UnaryOp::PreIncrement) {
                            1
                        } else {
                            -1
                        };
                        self.gen_inc_dec(e, d, true)?;
                    }
                    UnaryOp::PostIncrement | UnaryOp::PostDecrement => {
                        let d = if matches!(op, UnaryOp::PostIncrement) {
                            1
                        } else {
                            -1
                        };
                        self.gen_inc_dec(e, d, false)?;
                    }
                }
                Ok(())
            }
            Expression::Binary(op, l, r) => self.gen_binary(*op, l, r),
            Expression::Call(f, args) => self.gen_call(f, args),
            Expression::Assign(t, v) => self.gen_assign(t, v),
            Expression::AssignOp(op, t, v) => {
                let be =
                    lir::Expression::Binary(*op, Box::new((**t).clone()), Box::new((**v).clone()));
                self.gen_assign(t, &be)
            }
            Expression::Cast(_, inner) => self.gen_expr(inner),
            Expression::Parenthesized(inner) => self.gen_expr(inner),
            Expression::AddressOf(inner) => match inner.as_ref() {
                Expression::Variable(name) => {
                    if let Some(&off) = self.local_offsets.get(name) {
                        self.emit_add_imm(0, FP, off as u32);
                        Ok(())
                    } else if self.globals.contains_key(name) {
                        self.emit_lea_global(0, name)
                    } else {
                        self.gen_expr(inner)
                    }
                }
                _ => self.gen_expr(inner),
            },
            Expression::Dereference(inner) => {
                self.gen_expr(inner)?;
                self.emit_ldr0(0, 0);
                Ok(())
            }
            Expression::Index(arr, idx) => {
                let elem_ty = self.index_elem_type(arr);
                let esz = elem_ty.as_ref().map(|t| self.type_size(t)).unwrap_or(8) as i64;
                self.gen_expr(arr)?;
                self.push(0);
                self.gen_expr(idx)?;
                self.emit_mov_imm(2, esz);
                self.emit_mul(1, 0, 2); // x1 = idx*esz
                self.pop(0); // x0 = base
                self.emit_add(0, 0, 1);
                self.emit_ldr0(0, 0);
                Ok(())
            }
            Expression::Member(obj, field) => {
                self.gen_expr(obj)?;
                let off = self.resolve_field_offset(obj, field, false).unwrap_or(0);
                if off > 0 {
                    self.emit_add_imm(0, 0, off as u32);
                }
                self.emit_ldr0(0, 0);
                Ok(())
            }
            Expression::PointerMember(obj, field) => {
                self.gen_expr(obj)?;
                self.emit_ldr0(0, 0);
                let off = self.resolve_field_offset(obj, field, true).unwrap_or(0);
                if off > 0 {
                    self.emit_add_imm(0, 0, off as u32);
                }
                self.emit_ldr0(0, 0);
                Ok(())
            }
            Expression::Ternary(c, t, e) => {
                let else_l = self.new_label("tern_else");
                let end_l = self.new_label("tern_end");
                self.gen_expr(c)?;
                self.emit_cmp_zero_branch_eq(&else_l);
                self.gen_expr(t)?;
                self.emit_b(&end_l);
                self.define_label(&else_l);
                self.gen_expr(e)?;
                self.define_label(&end_l);
                Ok(())
            }
            Expression::SizeOf(ty) => {
                let s = self.type_size(ty) as i64;
                self.emit_mov_imm(0, s);
                Ok(())
            }
            Expression::Comma(exprs) => {
                for e in exprs {
                    self.gen_expr(e)?;
                }
                Ok(())
            }
            _ => {
                log::debug!("aarch64: 未支持的表达式 {:?}", std::mem::discriminant(expr));
                self.emit_mov_imm(0, 0);
                Ok(())
            }
        }
    }

    fn emit_cmp_imm0(&mut self, rn: u8) {
        // subs xzr, rn, #0  => cmp rn,#0
        self.w(0xF100001F | ((rn as u32) << 5));
    }

    fn gen_literal(&mut self, lit: &lir::Literal) -> NativeResult<()> {
        use lir::Literal;
        match lit {
            Literal::Integer(n) | Literal::Long(n) | Literal::LongLong(n) => {
                self.emit_mov_imm(0, *n)
            }
            Literal::UnsignedInteger(n)
            | Literal::UnsignedLong(n)
            | Literal::UnsignedLongLong(n) => self.emit_mov_imm(0, *n as i64),
            Literal::Bool(b) => self.emit_mov_imm(0, *b as i64),
            Literal::Char(c) => self.emit_mov_imm(0, *c as i64),
            Literal::Float(f) => self.emit_mov_imm(0, (*f as f64).to_bits() as i64),
            Literal::Double(f) => self.emit_mov_imm(0, f.to_bits() as i64),
            Literal::String(s) => {
                let s = s.clone();
                self.emit_lea_string(0, &s);
            }
            Literal::NullPointer => self.emit_mov_imm(0, 0),
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
        match op {
            BinaryOp::LogicalAnd => {
                let end = self.new_label("land_end");
                self.gen_expr(left)?;
                self.emit_cmp_zero_branch_eq(&end);
                self.gen_expr(right)?;
                self.emit_cmp_imm0(0);
                self.emit_cset(0, Cond::Ne);
                self.define_label(&end);
                return Ok(());
            }
            BinaryOp::LogicalOr => {
                let set1 = self.new_label("lor_one");
                let end = self.new_label("lor_end");
                self.gen_expr(left)?;
                self.emit_cmp_zero_branch_ne(&set1);
                self.gen_expr(right)?;
                self.emit_cmp_zero_branch_ne(&set1);
                self.emit_mov_imm(0, 0);
                self.emit_b(&end);
                self.define_label(&set1);
                self.emit_mov_imm(0, 1);
                self.define_label(&end);
                return Ok(());
            }
            _ => {}
        }

        // 浮点运算：任一操作数为浮点则走 FP 路径
        let lf = self.expr_is_float(left);
        let rf = self.expr_is_float(right);
        if lf || rf {
            return self.gen_binary_float(op, left, right, lf, rf);
        }

        self.gen_expr(left)?;
        self.push(0);
        self.gen_expr(right)?;
        self.emit_mov_reg(1, 0); // x1 = right
        self.pop(0); // x0 = left

        match op {
            BinaryOp::Add => self.emit_add(0, 0, 1),
            BinaryOp::Subtract => self.emit_sub(0, 0, 1),
            BinaryOp::Multiply => self.emit_mul(0, 0, 1),
            BinaryOp::Divide => self.emit_sdiv(0, 0, 1),
            BinaryOp::Modulo => {
                self.emit_sdiv(2, 0, 1); // x2 = x0/x1
                self.emit_mul(2, 2, 1); // x2 = (x0/x1)*x1
                self.emit_sub(0, 0, 2); // x0 = x0 - x2
            }
            BinaryOp::BitAnd => self.emit_and(0, 0, 1),
            BinaryOp::BitOr => self.emit_orr(0, 0, 1),
            BinaryOp::BitXor => self.emit_eor(0, 0, 1),
            BinaryOp::LeftShift => self.emit_lslv(0, 0, 1),
            BinaryOp::RightShift => self.emit_lsrv(0, 0, 1),
            BinaryOp::RightShiftArithmetic => self.emit_asrv(0, 0, 1),
            BinaryOp::Equal => self.cmp_set(Cond::Eq),
            BinaryOp::NotEqual => self.cmp_set(Cond::Ne),
            BinaryOp::LessThan => self.cmp_set(Cond::Lt),
            BinaryOp::LessThanEqual => self.cmp_set(Cond::Le),
            BinaryOp::GreaterThan => self.cmp_set(Cond::Gt),
            BinaryOp::GreaterThanEqual => self.cmp_set(Cond::Ge),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => unreachable!(),
        }
        Ok(())
    }

    fn cmp_set(&mut self, cond: Cond) {
        self.emit_cmp(0, 1);
        self.emit_cset(0, cond);
    }

    /// 表达式静态类型是否为浮点
    fn expr_is_float(&self, e: &lir::Expression) -> bool {
        matches!(
            self.infer_expr_type(e),
            Some(lir::Type::Float) | Some(lir::Type::Double) | Some(lir::Type::LongDouble)
        )
    }

    /// 浮点二元运算。值模型同整数：8 字节位存于 GPR。计算时搬入 d0/d1，
    /// 必要时把整数操作数用 scvtf 提升为 double；算术结果位回到 x0，
    /// 比较结果为 x0 中的 0/1。
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
        self.push(0);
        self.gen_expr(right)?;
        self.emit_mov_reg(1, 0); // x1 = right 位/整数
        self.pop(0); // x0 = left 位/整数

        // d0 = left, d1 = right（必要时 int->double）
        if lf {
            self.emit_fmov_d_from_x(0, 0);
        } else {
            self.emit_scvtf(0, 0);
        }
        if rf {
            self.emit_fmov_d_from_x(1, 1);
        } else {
            self.emit_scvtf(1, 1);
        }

        match op {
            BinaryOp::Add => self.emit_fadd(0, 0, 1),
            BinaryOp::Subtract => self.emit_fsub(0, 0, 1),
            BinaryOp::Multiply => self.emit_fmul(0, 0, 1),
            BinaryOp::Divide => self.emit_fdiv(0, 0, 1),
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::LessThan
            | BinaryOp::LessThanEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterThanEqual => {
                self.emit_fcmp(0, 1);
                let cond = match op {
                    BinaryOp::Equal => Cond::Eq,
                    BinaryOp::NotEqual => Cond::Ne,
                    BinaryOp::LessThan => Cond::Mi,
                    BinaryOp::LessThanEqual => Cond::Ls,
                    BinaryOp::GreaterThan => Cond::Gt,
                    BinaryOp::GreaterThanEqual => Cond::Ge,
                    _ => unreachable!(),
                };
                self.emit_cset(0, cond);
                return Ok(());
            }
            _ => self.emit_fadd(0, 0, 1),
        }
        self.emit_fmov_x_from_d(0, 0); // x0 = 结果位
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
                if let Some(&off) = self.local_offsets.get(name) {
                    self.store_local(off, 0);
                } else if self.globals.contains_key(name) {
                    self.emit_mov_reg(1, 0); // x1 = value
                    self.emit_lea_global(0, name)?;
                    self.emit_str0(1, 0);
                } else {
                    return Err(NativeError::CodegenError(format!(
                        "未定义赋值目标: {}",
                        name
                    )));
                }
            }
            Expression::Dereference(ptr) => {
                self.gen_expr(value)?;
                self.push(0);
                self.gen_expr(ptr)?;
                self.pop(1); // x1 = value
                self.emit_str0(1, 0);
            }
            Expression::Member(obj, field) => {
                self.gen_expr(value)?;
                self.push(0);
                self.gen_expr(obj)?;
                let off = self.resolve_field_offset(obj, field, false).unwrap_or(0);
                if off > 0 {
                    self.emit_add_imm(0, 0, off as u32);
                }
                self.pop(1);
                self.emit_str0(1, 0);
            }
            Expression::Index(arr, idx) => {
                let elem_ty = self.index_elem_type(arr);
                let esz = elem_ty.as_ref().map(|t| self.type_size(t)).unwrap_or(8) as i64;
                self.gen_expr(value)?;
                self.push(0);
                self.gen_expr(arr)?;
                self.push(0);
                self.gen_expr(idx)?;
                self.emit_mov_imm(2, esz);
                self.emit_mul(0, 0, 2);
                self.pop(1); // x1 = base
                self.emit_add(0, 1, 0); // x0 = base + idx*esz
                self.pop(1); // x1 = value
                self.emit_str0(1, 0);
            }
            _ => {
                self.gen_expr(value)?;
                log::debug!("aarch64: 未支持的赋值目标");
            }
        }
        Ok(())
    }

    fn gen_inc_dec(&mut self, target: &lir::Expression, delta: i64, pre: bool) -> NativeResult<()> {
        use lir::Expression;
        if let Expression::Variable(name) = target {
            if let Some(&off) = self.local_offsets.get(name) {
                self.load_local(0, off);
                if !pre {
                    self.push(0);
                }
                self.emit_mov_imm(1, delta.abs());
                if delta >= 0 {
                    self.emit_add(0, 0, 1);
                } else {
                    self.emit_sub(0, 0, 1);
                }
                self.store_local(off, 0);
                if !pre {
                    self.pop(0);
                }
                return Ok(());
            }
        }
        self.gen_expr(target)?;
        self.emit_mov_imm(1, delta.abs());
        if delta >= 0 {
            self.emit_add(0, 0, 1);
        } else {
            self.emit_sub(0, 0, 1);
        }
        Ok(())
    }

    fn gen_call(&mut self, func: &lir::Expression, args: &[lir::Expression]) -> NativeResult<()> {
        use lir::Expression;
        let direct = match func {
            Expression::Variable(n)
                if !self.local_offsets.contains_key(n) && !self.globals.contains_key(n) =>
            {
                Some(n.clone())
            }
            _ => None,
        };
        if let Some(name) = &direct {
            if matches!(
                name.as_str(),
                "println" | "print" | "print_inline" | "eprintln" | "eprint"
            ) {
                return self.gen_print_call(name, args);
            }
        }
        // 求值实参 → 入栈（逆序，使栈顶为 arg0）→ 按 AAPCS64 分发到
        // 通用寄存器 x0..x7（整型/指针）或浮点寄存器 d0..d7（浮点）。
        let n = args.len().min(ARG_REGS.len());
        let floats: Vec<bool> = args.iter().take(n).map(|a| self.expr_is_float(a)).collect();
        for a in args.iter().take(n).rev() {
            self.gen_expr(a)?;
            self.push(0);
        }
        // 间接调用：先把函数指针求到 x9（在弹参之前，避免占用参数寄存器）
        let indirect = direct.is_none();
        if indirect {
            // 函数指针当前不可能是浮点；先弹出所有参数再求 func 会破坏栈布局，
            // 因此这里在参数仍在栈上时求值 func 到 x9。
            self.gen_expr(func)?;
            self.emit_mov_reg(9, 0);
        }
        let mut ngrn = 0usize; // 下一个通用参数寄存器
        let mut nsrn = 0usize; // 下一个浮点参数寄存器
        for i in 0..n {
            self.pop(10); // x10 = arg i 位
            if floats[i] {
                self.emit_fmov_d_from_x(nsrn as u8, 10);
                nsrn += 1;
            } else {
                self.emit_mov_reg(ARG_REGS[ngrn], 10);
                ngrn += 1;
            }
        }
        match direct {
            Some(name) => self.emit_bl(&name),
            None => self.emit_blr(9),
        }
        Ok(())
    }

    fn gen_print_call(&mut self, name: &str, args: &[lir::Expression]) -> NativeResult<()> {
        let newline = !matches!(name, "print_inline" | "eprint");
        let to_stderr = matches!(name, "eprintln" | "eprint");
        let kinds: Vec<PK> = args.iter().map(|a| self.print_kind(a)).collect();
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
        let callee = if to_stderr { "dprintf" } else { "printf" };
        let lead = if to_stderr { 2 } else { 1 };

        // 实参逆序入栈： [..args.rev(), fmt, (fd?)]
        for (idx, a) in args.iter().enumerate().rev() {
            if kinds[idx] == PK::Bool {
                self.gen_bool_to_cstr(a)?;
            } else {
                self.gen_expr(a)?;
            }
            self.push(0);
        }
        self.emit_lea_string(0, &fmt);
        self.push(0);
        if to_stderr {
            self.emit_mov_imm(0, 2);
            self.push(0);
        }
        // 弹入寄存器：x0 = (fd 或 fmt), ...
        let total = lead + args.len();
        let nreg = total.min(ARG_REGS.len());
        for i in 0..nreg {
            self.pop(ARG_REGS[i]);
        }
        self.emit_bl(callee);
        Ok(())
    }

    fn gen_bool_to_cstr(&mut self, e: &lir::Expression) -> NativeResult<()> {
        self.gen_expr(e)?;
        let lt = self.new_label("bt");
        let le = self.new_label("be");
        self.emit_cmp_zero_branch_ne(&lt);
        self.emit_lea_string(0, "false");
        self.emit_b(&le);
        self.define_label(&lt);
        self.emit_lea_string(0, "true");
        self.define_label(&le);
        Ok(())
    }

    fn print_kind(&self, e: &lir::Expression) -> PK {
        match self.infer_expr_type(e) {
            Some(t) => PK::from_type(&t),
            None => PK::Int,
        }
    }

    // ====================== 类型推断/字段 ======================
    fn infer_expr_type(&self, e: &lir::Expression) -> Option<lir::Type> {
        use lir::{BinaryOp, Expression, Type};
        match e {
            Expression::Literal(l) => Some(match l {
                lir::Literal::Bool(_) => Type::Bool,
                lir::Literal::Char(_) => Type::Char,
                lir::Literal::String(_) => Type::Pointer(Box::new(Type::Char)),
                lir::Literal::Float(_) => Type::Float,
                lir::Literal::Double(_) => Type::Double,
                lir::Literal::NullPointer => Type::Pointer(Box::new(Type::Void)),
                _ => Type::Long,
            }),
            Expression::Variable(n) => self
                .local_and_param_types
                .get(n)
                .cloned()
                .or_else(|| self.global_types.get(n).cloned()),
            Expression::Cast(ty, _) => Some(ty.clone()),
            Expression::Parenthesized(i) => self.infer_expr_type(i),
            Expression::Binary(op, l, _) => {
                if matches!(
                    op,
                    BinaryOp::Equal
                        | BinaryOp::NotEqual
                        | BinaryOp::LessThan
                        | BinaryOp::LessThanEqual
                        | BinaryOp::GreaterThan
                        | BinaryOp::GreaterThanEqual
                ) {
                    Some(Type::Bool)
                } else {
                    self.infer_expr_type(l)
                }
            }
            Expression::Index(a, _) => self.index_elem_type(a),
            Expression::Member(o, f) => self.member_type(o, f, false),
            Expression::PointerMember(o, f) => self.member_type(o, f, true),
            _ => None,
        }
    }

    fn index_elem_type(&self, arr: &lir::Expression) -> Option<lir::Type> {
        match self.infer_expr_type(arr) {
            Some(lir::Type::Pointer(p)) | Some(lir::Type::Array(p, _)) => Some(*p),
            _ => None,
        }
    }

    fn member_type(&self, obj: &lir::Expression, field: &str, via_ptr: bool) -> Option<lir::Type> {
        let sname = self.struct_name(obj, via_ptr)?;
        self.field_types
            .get(&format!("{}::{}", sname, field))
            .cloned()
    }

    fn struct_name(&self, obj: &lir::Expression, via_ptr: bool) -> Option<String> {
        let ty = self.infer_expr_type(obj)?;
        let ty = peel(&ty);
        match (via_ptr, ty) {
            (true, lir::Type::Pointer(inner)) => match peel(inner) {
                lir::Type::Named(s) => Some(s.clone()),
                _ => None,
            },
            (false, lir::Type::Named(s)) => Some(s.clone()),
            _ => None,
        }
    }

    fn resolve_field_offset(
        &self,
        obj: &lir::Expression,
        field: &str,
        via_ptr: bool,
    ) -> Option<usize> {
        let sname = self.struct_name(obj, via_ptr)?;
        self.field_offsets
            .get(&format!("{}::{}", sname, field))
            .copied()
    }

    fn type_size(&self, ty: &lir::Type) -> usize {
        use lir::Type;
        match ty {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            Type::Float | Type::Long | Type::Ulong | Type::Double | Type::Pointer(_) => 8,
            Type::LongLong | Type::UlongLong | Type::LongDouble => 16,
            Type::Qualified(_, i) => self.type_size(i),
            _ => 8,
        }
    }

    fn collect_var_types(stmt: &lir::Statement, types: &mut HashMap<String, lir::Type>) {
        match stmt {
            lir::Statement::Variable(v) => {
                types.insert(v.name.clone(), v.type_.clone());
            }
            lir::Statement::Compound(b) => {
                for s in &b.statements {
                    Self::collect_var_types(s, types);
                }
            }
            lir::Statement::If(s) => {
                Self::collect_var_types(&s.then_branch, types);
                if let Some(e) = &s.else_branch {
                    Self::collect_var_types(e, types);
                }
            }
            lir::Statement::For(s) => {
                if let Some(i) = &s.initializer {
                    Self::collect_var_types(i, types);
                }
                Self::collect_var_types(&s.body, types);
            }
            lir::Statement::While(s) => Self::collect_var_types(&s.body, types),
            lir::Statement::DoWhile(s) => Self::collect_var_types(&s.body, types),
            _ => {}
        }
    }

    fn collect_var_names(stmt: &lir::Statement, names: &mut Vec<String>) {
        match stmt {
            lir::Statement::Variable(v) => names.push(v.name.clone()),
            lir::Statement::Compound(b) => {
                for s in &b.statements {
                    Self::collect_var_names(s, names);
                }
            }
            lir::Statement::If(s) => {
                Self::collect_var_names(&s.then_branch, names);
                if let Some(e) = &s.else_branch {
                    Self::collect_var_names(e, names);
                }
            }
            lir::Statement::For(s) => {
                if let Some(i) = &s.initializer {
                    Self::collect_var_names(i, names);
                }
                Self::collect_var_names(&s.body, names);
            }
            lir::Statement::While(s) => Self::collect_var_names(&s.body, names),
            lir::Statement::DoWhile(s) => Self::collect_var_names(&s.body, names),
            lir::Statement::Switch(s) => {
                for c in &s.cases {
                    Self::collect_var_names(&c.body, names);
                }
                if let Some(d) = &s.default {
                    Self::collect_var_names(d, names);
                }
            }
            _ => {}
        }
    }
}

fn peel(ty: &lir::Type) -> &lir::Type {
    match ty {
        lir::Type::Qualified(_, i) => peel(i),
        t => t,
    }
}

fn round_up(n: usize, a: usize) -> usize {
    if a == 0 {
        return n;
    }
    n.div_ceil(a) * a
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lir::{self as lir, Expression, Literal, Statement, Type};

    fn gen(program: &lir::Program) -> MachineObject {
        Aarch64CodeGen::new(TargetOS::Linux)
            .generate(program)
            .unwrap()
    }

    #[test]
    fn test_return_const_has_text_and_symbol() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::Literal(
                Literal::Integer(42),
            ))));
        program.add(lir::Declaration::Function(func));
        let obj = gen(&program);
        assert!(!obj.text.is_empty());
        assert!(obj.text.len() % 4 == 0);
        assert!(obj.symbols.iter().any(|s| s.name == "main" && s.is_func));
        // ret = 0xD65F03C0
        let last = u32::from_le_bytes(obj.text[obj.text.len() - 4..].try_into().unwrap());
        assert_eq!(last, 0xD65F03C0);
    }

    #[test]
    fn test_external_call_creates_reloc() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Expression(Expression::Call(
                Box::new(Expression::Variable("println".into())),
                vec![Expression::Literal(Literal::String("hi".into()))],
            )));
        func.body
            .statements
            .push(Statement::Return(Some(Expression::Literal(
                Literal::Integer(0),
            ))));
        program.add(lir::Declaration::Function(func));
        let obj = gen(&program);
        assert!(obj
            .relocations
            .iter()
            .any(|r| matches!(r.kind, RelKind::Aarch64Call26)));
        assert!(obj.relocations.iter().any(|r| matches!(
            r.kind,
            RelKind::Aarch64AdrPrelPgHi21 | RelKind::Aarch64AddAbsLo12Nc
        )));
        assert!(obj.rodata.windows(2).any(|w| w == b"hi"));
    }
}
