//! RISC-V 64 (RV64IM) 直出机器码生成器 (Linux, lp64d)
//!
//! 把 LIR 直接降级为 RV64 机器码（定长 32 位指令）。值模型：标量结果置于
//! `a0`(x10)。临时值通过 `addi sp,sp,-16; sd a0,0(sp)` / `ld a0,0(sp);
//! addi sp,sp,16` 入/出栈。覆盖整型/指针标量、控制流、调用（≤8 整型实参）、
//! println/print、全局 .data/.bss、字符串。浮点与聚合体暂以整型路径近似处理。
//!
//! 引用寻址采用绝对模式（非 PIE，静态链接）：`lui+addi` 配 `R_RISCV_HI20`/
//! `R_RISCV_LO12_I`；外部调用用 `auipc+jalr` 配 `R_RISCV_CALL_PLT`。

use std::collections::{HashMap, HashSet};

use crate::{NativeError, NativeResult, TargetOS};
use x_lir as lir;

use super::{MachineObject, ObjReloc, ObjSymbol, RelKind, RelTarget, SecKind};

const ZERO: u8 = 0;
const RA: u8 = 1;
const SP: u8 = 2;
const FP: u8 = 8; // s0
const A0: u8 = 10;
const T0: u8 = 5;
const T1: u8 = 6;
const T2: u8 = 7;

#[inline]
fn fits_imm12(off: i32) -> bool {
    (-2048..=2047).contains(&off)
}
const ARG_REGS: [u8; 8] = [10, 11, 12, 13, 14, 15, 16, 17];

// opcodes
const OP: u32 = 0x33;
const OP_IMM: u32 = 0x13;
const LOAD: u32 = 0x03;
const STORE: u32 = 0x23;
const BRANCH: u32 = 0x63;
const JAL: u32 = 0x6f;
const JALR: u32 = 0x67;
const LUI: u32 = 0x37;
const AUIPC: u32 = 0x17;
const OP_FP: u32 = 0x53;
// 浮点参数寄存器 fa0..fa7 = f10..f17（lp64d ABI）
const FARG_REGS: [u8; 8] = [10, 11, 12, 13, 14, 15, 16, 17];

struct JumpFixup {
    field: usize,
    label: String,
    is_branch: bool,
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

pub struct RiscV64CodeGen {
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

impl RiscV64CodeGen {
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
    fn r_type(&mut self, funct7: u32, rs2: u8, rs1: u8, funct3: u32, rd: u8, opcode: u32) {
        self.w((funct7 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | (funct3 << 12)
            | ((rd as u32) << 7)
            | opcode);
    }
    fn i_type(&mut self, imm: i32, rs1: u8, funct3: u32, rd: u8, opcode: u32) {
        let imm = (imm as u32) & 0xfff;
        self.w((imm << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | ((rd as u32) << 7) | opcode);
    }
    fn s_type(&mut self, imm: i32, rs2: u8, rs1: u8, funct3: u32, opcode: u32) {
        let imm = imm as u32;
        let imm11_5 = (imm >> 5) & 0x7f;
        let imm4_0 = imm & 0x1f;
        self.w((imm11_5 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | (funct3 << 12)
            | (imm4_0 << 7)
            | opcode);
    }
    fn u_type(&mut self, imm: u32, rd: u8, opcode: u32) {
        self.w((imm & 0xffff_f000) | ((rd as u32) << 7) | opcode);
    }

    fn emit_addi(&mut self, rd: u8, rs1: u8, imm: i32) {
        self.i_type(imm, rs1, 0x0, rd, OP_IMM);
    }
    fn emit_mv(&mut self, rd: u8, rs: u8) {
        self.emit_addi(rd, rs, 0);
    }
    fn emit_slli(&mut self, rd: u8, rs1: u8, sh: u32) {
        // RV64: shamt 6 位
        self.w(((sh & 0x3f) << 20)
            | ((rs1 as u32) << 15)
            | (0x1 << 12)
            | ((rd as u32) << 7)
            | OP_IMM);
    }
    /// 加载任意 64 位常量到 rd（递归 lui/addi/slli）
    fn emit_li(&mut self, rd: u8, val: i64) {
        let lo12 = ((val & 0xfff) as i32) << 20 >> 20; // 符号扩展低 12 位
        let hi = (val - lo12 as i64) >> 12;
        if hi == 0 {
            self.emit_addi(rd, ZERO, lo12);
        } else {
            self.emit_li(rd, hi);
            self.emit_slli(rd, rd, 12);
            if lo12 != 0 {
                self.emit_addi(rd, rd, lo12);
            }
        }
    }

    // ── 浮点（double，D 扩展）─────────────────────────────────────────────
    /// fmv.d.x f<fd>, x<xs>（GPR 位 → 浮点寄存器）
    fn emit_fmv_d_x(&mut self, fd: u8, xs: u8) {
        self.r_type(0x79, 0, xs, 0x0, fd, OP_FP);
    }
    /// fmv.x.d x<xd>, f<fs>（浮点寄存器 → GPR 位）
    fn emit_fmv_x_d(&mut self, xd: u8, fs: u8) {
        self.r_type(0x71, 0, fs, 0x0, xd, OP_FP);
    }
    /// fcvt.d.l f<fd>, x<xs>（有符号 64 位整数 → double）
    fn emit_fcvt_d_l(&mut self, fd: u8, xs: u8) {
        self.r_type(0x69, 2, xs, 0x0, fd, OP_FP);
    }
    fn emit_fadd_d(&mut self, fd: u8, fs1: u8, fs2: u8) {
        self.r_type(0x01, fs2, fs1, 0x0, fd, OP_FP);
    }
    fn emit_fsub_d(&mut self, fd: u8, fs1: u8, fs2: u8) {
        self.r_type(0x05, fs2, fs1, 0x0, fd, OP_FP);
    }
    fn emit_fmul_d(&mut self, fd: u8, fs1: u8, fs2: u8) {
        self.r_type(0x09, fs2, fs1, 0x0, fd, OP_FP);
    }
    fn emit_fdiv_d(&mut self, fd: u8, fs1: u8, fs2: u8) {
        self.r_type(0x0D, fs2, fs1, 0x0, fd, OP_FP);
    }
    /// feq.d x<xd>, f<fs1>, f<fs2>
    fn emit_feq_d(&mut self, xd: u8, fs1: u8, fs2: u8) {
        self.r_type(0x51, fs2, fs1, 0x2, xd, OP_FP);
    }
    /// flt.d x<xd>, f<fs1>, f<fs2>
    fn emit_flt_d(&mut self, xd: u8, fs1: u8, fs2: u8) {
        self.r_type(0x51, fs2, fs1, 0x1, xd, OP_FP);
    }
    /// fle.d x<xd>, f<fs1>, f<fs2>
    fn emit_fle_d(&mut self, xd: u8, fs1: u8, fs2: u8) {
        self.r_type(0x51, fs2, fs1, 0x0, xd, OP_FP);
    }

    fn emit_ld(&mut self, rd: u8, rs1: u8, off: i32) {
        self.i_type(off, rs1, 0x3, rd, LOAD);
    }
    fn emit_sd(&mut self, rs2: u8, rs1: u8, off: i32) {
        self.s_type(off, rs2, rs1, 0x3, STORE);
    }
    fn push(&mut self, r: u8) {
        self.emit_addi(SP, SP, -16);
        self.emit_sd(r, SP, 0);
    }
    fn pop(&mut self, r: u8) {
        self.emit_ld(r, SP, 0);
        self.emit_addi(SP, SP, 16);
    }

    fn store_local(&mut self, off: i32, r: u8) {
        self.emit_sd_off(r, FP, off);
    }
    fn load_local(&mut self, r: u8, off: i32) {
        self.emit_ld_off(r, FP, off);
    }

    /// sd 支持任意偏移：超出 12 位立即数范围时用 T0 计算有效地址。
    fn emit_sd_off(&mut self, rs2: u8, base: u8, off: i32) {
        if fits_imm12(off) {
            self.emit_sd(rs2, base, off);
        } else {
            self.emit_li(T0, off as i64);
            self.r_type(0x0, T0, base, 0x0, T0, OP); // add t0, base, t0
            self.emit_sd(rs2, T0, 0);
        }
    }
    /// ld 支持任意偏移：超出 12 位立即数范围时用 T0 计算有效地址。
    fn emit_ld_off(&mut self, rd: u8, base: u8, off: i32) {
        if fits_imm12(off) {
            self.emit_ld(rd, base, off);
        } else {
            self.emit_li(T0, off as i64);
            self.r_type(0x0, T0, base, 0x0, T0, OP); // add t0, base, t0
            self.emit_ld(rd, T0, 0);
        }
    }
    /// 调整 sp：超出 12 位立即数范围时用 T0 完成。
    fn adjust_sp(&mut self, delta: i32) {
        if fits_imm12(delta) {
            self.emit_addi(SP, SP, delta);
        } else {
            self.emit_li(T0, delta as i64);
            self.r_type(0x0, T0, SP, 0x0, SP, OP); // add sp, sp, t0
        }
    }

    fn emit_ret(&mut self) {
        // jalr x0, ra, 0
        self.i_type(0, RA, 0x0, ZERO, JALR);
    }

    // ====================== 标签/分支/调用 ======================
    fn new_label(&mut self, prefix: &str) -> String {
        let l = format!("L_{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        l
    }
    fn define_label(&mut self, name: &str) {
        self.labels.insert(name.to_string(), self.text.len());
    }
    /// beq/bne rs1,rs2,label
    fn emit_branch(&mut self, funct3: u32, rs1: u8, rs2: u8, label: &str) {
        let field = self.text.len();
        // 占位（imm=0）
        self.w(((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (funct3 << 12) | BRANCH);
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
            is_branch: true,
        });
    }
    fn emit_beqz(&mut self, rs: u8, label: &str) {
        self.emit_branch(0x0, rs, ZERO, label);
    }
    fn emit_bnez(&mut self, rs: u8, label: &str) {
        self.emit_branch(0x1, rs, ZERO, label);
    }
    fn emit_j(&mut self, label: &str) {
        let field = self.text.len();
        self.w(JAL); // jal x0, 0
        self.jump_fixups.push(JumpFixup {
            field,
            label: label.to_string(),
            is_branch: false,
        });
    }
    fn emit_call(&mut self, callee: &str) {
        let field = self.text.len();
        // auipc ra, 0 ; jalr ra, ra, 0
        self.u_type(0, RA, AUIPC);
        self.i_type(0, RA, 0x0, RA, JALR);
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
            let rel = (target as i64 - jf.field as i64) as i32;
            let word = u32::from_le_bytes(self.text[jf.field..jf.field + 4].try_into().unwrap());
            let patched = if jf.is_branch {
                word | encode_b_imm(rel)
            } else {
                word | encode_j_imm(rel)
            };
            self.text[jf.field..jf.field + 4].copy_from_slice(&patched.to_le_bytes());
        }
        Ok(())
    }

    fn resolve_calls(&mut self) -> NativeResult<()> {
        let calls = std::mem::take(&mut self.call_fixups);
        for cf in calls {
            if let Some(f) = self.func_syms.iter().find(|f| f.name == cf.callee) {
                // 内部调用：用 pc 相对填充 auipc+jalr
                let rel = f.offset as i64 - cf.field as i64;
                let hi = (((rel + 0x800) >> 12) as i32) as u32;
                let lo = (rel - ((hi as i64) << 12)) as i32;
                let auipc =
                    u32::from_le_bytes(self.text[cf.field..cf.field + 4].try_into().unwrap());
                let auipc = (auipc & 0xfff) | (hi << 12);
                self.text[cf.field..cf.field + 4].copy_from_slice(&auipc.to_le_bytes());
                let jf = cf.field + 4;
                let jalr = u32::from_le_bytes(self.text[jf..jf + 4].try_into().unwrap());
                let jalr = (jalr & 0x000f_ffff) | (((lo as u32) & 0xfff) << 20);
                self.text[jf..jf + 4].copy_from_slice(&jalr.to_le_bytes());
            } else {
                if !self.external_syms.iter().any(|s| s == &cf.callee) {
                    self.external_syms.push(cf.callee.clone());
                }
                self.relocations.push(ObjReloc {
                    offset: cf.field as u64,
                    target: RelTarget::Symbol(cf.callee.clone()),
                    kind: RelKind::RiscvCallPlt,
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

    /// 绝对寻址：lui rd,%hi(sym) ; addi rd,rd,%lo(sym)
    fn emit_section_addr(&mut self, rd: u8, section: SecKind, offset: u64) {
        let lui_field = self.text.len();
        self.u_type(0, rd, LUI);
        self.relocations.push(ObjReloc {
            offset: lui_field as u64,
            target: RelTarget::Section(section),
            kind: RelKind::RiscvHi20,
            addend: offset as i64,
        });
        let addi_field = self.text.len();
        self.i_type(0, rd, 0x0, rd, OP_IMM);
        self.relocations.push(ObjReloc {
            offset: addi_field as u64,
            target: RelTarget::Section(section),
            kind: RelKind::RiscvLo12I,
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

        for p in &func.parameters {
            self.local_and_param_types
                .insert(p.name.clone(), p.type_.clone());
        }
        for s in &func.body.statements {
            Self::collect_var_types(s, &mut self.local_and_param_types);
        }

        // 局部在 [s0 + disp]，disp 从 0 起；保存的 ra/s0 在帧顶。
        let mut disp = 0i32;
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
        let frame = round_up(self.stack_size + 16, 16) as i32;
        self.frame = frame;

        // prologue
        self.adjust_sp(-frame);
        self.emit_sd_off(RA, SP, frame - 8);
        self.emit_sd_off(FP, SP, frame - 16);
        self.emit_mv(FP, SP);

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
        let frame = self.frame;
        self.emit_mv(SP, FP);
        self.emit_ld_off(RA, SP, frame - 8);
        self.emit_ld_off(FP, SP, frame - 16);
        self.adjust_sp(frame);
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
                    self.store_local(off, A0);
                }
            }
            Statement::Return(Some(e)) => {
                self.gen_expr(e)?;
                self.gen_epilogue();
            }
            Statement::Return(None) => {
                self.emit_li(A0, 0);
                self.gen_epilogue();
            }
            Statement::If(s) => {
                let else_l = self.new_label("else");
                let end_l = self.new_label("endif");
                self.gen_expr(&s.condition)?;
                self.emit_beqz(A0, &else_l);
                self.gen_statement(&s.then_branch)?;
                self.emit_j(&end_l);
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
                self.emit_beqz(A0, &end_l);
                self.gen_statement(&s.body)?;
                self.emit_j(&start_l);
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
                self.emit_bnez(A0, &start_l);
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
                    self.emit_beqz(A0, &end_l);
                }
                self.gen_statement(&s.body)?;
                if let Some(inc) = &s.increment {
                    self.gen_expr(inc)?;
                }
                self.emit_j(&start_l);
                self.define_label(&end_l);
                self.loop_labels.pop();
            }
            Statement::Compound(b) => self.gen_block(b)?,
            Statement::Break => {
                if let Some((_, e)) = self.loop_labels.last() {
                    let e = e.clone();
                    self.emit_j(&e);
                }
            }
            Statement::Continue => {
                if let Some((c, _)) = self.loop_labels.last() {
                    let c = c.clone();
                    self.emit_j(&c);
                }
            }
            Statement::Label(n) => self.define_label(n),
            Statement::Goto(n) => {
                let n = n.clone();
                self.emit_j(&n);
            }
            Statement::Switch(sw) => self.gen_switch(sw)?,
            Statement::Empty => {}
            _ => {
                log::debug!("riscv64: 未支持的语句 {:?}", std::mem::discriminant(stmt));
            }
        }
        Ok(())
    }

    fn gen_switch(&mut self, sw: &lir::SwitchStatement) -> NativeResult<()> {
        let end = self.new_label("swend");
        self.loop_labels.push((end.clone(), end.clone()));
        self.gen_expr(&sw.expression)?;
        self.push(A0);
        let case_labels: Vec<String> = (0..sw.cases.len())
            .map(|_| self.new_label("case"))
            .collect();
        let default_l = self.new_label("swdefault");
        for (i, c) in sw.cases.iter().enumerate() {
            self.gen_expr(&c.value)?; // a0 = case value
            self.emit_mv(T1, A0);
            self.emit_ld(A0, SP, 0); // 被测值
            self.emit_branch(0x0, A0, T1, &case_labels[i]); // beq
        }
        self.emit_j(&default_l);
        for (i, c) in sw.cases.iter().enumerate() {
            self.define_label(&case_labels[i]);
            self.gen_statement(&c.body)?;
        }
        self.define_label(&default_l);
        if let Some(d) = &sw.default {
            self.gen_statement(d)?;
        }
        self.define_label(&end);
        self.pop(A0);
        self.loop_labels.pop();
        Ok(())
    }

    // ====================== 表达式（结果入 a0） ======================
    fn gen_expr(&mut self, expr: &lir::Expression) -> NativeResult<()> {
        use lir::{Expression, UnaryOp};
        match expr {
            Expression::Literal(l) => self.gen_literal(l),
            Expression::Variable(name) => {
                if let Some(&off) = self.local_offsets.get(name) {
                    self.load_local(A0, off);
                } else if self.globals.contains_key(name) {
                    self.emit_lea_global(A0, name)?;
                    self.emit_ld(A0, A0, 0);
                } else {
                    return Err(NativeError::CodegenError(format!("未定义变量: {}", name)));
                }
                Ok(())
            }
            Expression::Unary(op, e) => {
                match op {
                    UnaryOp::Minus => {
                        self.gen_expr(e)?;
                        self.r_type(0x20, A0, ZERO, 0x0, A0, OP); // sub a0, zero, a0
                    }
                    UnaryOp::BitNot => {
                        self.gen_expr(e)?;
                        self.i_type(-1, A0, 0x4, A0, OP_IMM); // xori a0,a0,-1
                    }
                    UnaryOp::Plus => self.gen_expr(e)?,
                    UnaryOp::Not => {
                        self.gen_expr(e)?;
                        self.i_type(1, A0, 0x3, A0, OP_IMM); // sltiu a0,a0,1
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
                        self.emit_addi(A0, FP, off);
                        Ok(())
                    } else if self.globals.contains_key(name) {
                        self.emit_lea_global(A0, name)
                    } else {
                        self.gen_expr(inner)
                    }
                }
                _ => self.gen_expr(inner),
            },
            Expression::Dereference(inner) => {
                self.gen_expr(inner)?;
                self.emit_ld(A0, A0, 0);
                Ok(())
            }
            Expression::Index(arr, idx) => {
                let elem_ty = self.index_elem_type(arr);
                let esz = elem_ty.as_ref().map(|t| self.type_size(t)).unwrap_or(8) as i64;
                self.gen_expr(arr)?;
                self.push(A0);
                self.gen_expr(idx)?;
                self.emit_li(T2, esz);
                self.r_type(0x01, T2, A0, 0x0, A0, OP); // mul a0, a0, t2
                self.pop(T1); // base
                self.r_type(0x0, A0, T1, 0x0, A0, OP); // add a0, t1, a0
                self.emit_ld(A0, A0, 0);
                Ok(())
            }
            Expression::Member(obj, field) => {
                self.gen_expr(obj)?;
                let off = self.resolve_field_offset(obj, field, false).unwrap_or(0);
                if off > 0 {
                    self.emit_addi(A0, A0, off as i32);
                }
                self.emit_ld(A0, A0, 0);
                Ok(())
            }
            Expression::PointerMember(obj, field) => {
                self.gen_expr(obj)?;
                self.emit_ld(A0, A0, 0);
                let off = self.resolve_field_offset(obj, field, true).unwrap_or(0);
                if off > 0 {
                    self.emit_addi(A0, A0, off as i32);
                }
                self.emit_ld(A0, A0, 0);
                Ok(())
            }
            Expression::Ternary(c, t, e) => {
                let else_l = self.new_label("tern_else");
                let end_l = self.new_label("tern_end");
                self.gen_expr(c)?;
                self.emit_beqz(A0, &else_l);
                self.gen_expr(t)?;
                self.emit_j(&end_l);
                self.define_label(&else_l);
                self.gen_expr(e)?;
                self.define_label(&end_l);
                Ok(())
            }
            Expression::SizeOf(ty) => {
                let s = self.type_size(ty) as i64;
                self.emit_li(A0, s);
                Ok(())
            }
            Expression::Comma(exprs) => {
                for e in exprs {
                    self.gen_expr(e)?;
                }
                Ok(())
            }
            _ => {
                log::debug!("riscv64: 未支持的表达式 {:?}", std::mem::discriminant(expr));
                self.emit_li(A0, 0);
                Ok(())
            }
        }
    }

    fn gen_literal(&mut self, lit: &lir::Literal) -> NativeResult<()> {
        use lir::Literal;
        match lit {
            Literal::Integer(n) | Literal::Long(n) | Literal::LongLong(n) => self.emit_li(A0, *n),
            Literal::UnsignedInteger(n)
            | Literal::UnsignedLong(n)
            | Literal::UnsignedLongLong(n) => self.emit_li(A0, *n as i64),
            Literal::Bool(b) => self.emit_li(A0, *b as i64),
            Literal::Char(c) => self.emit_li(A0, *c as i64),
            Literal::Float(f) => self.emit_li(A0, (*f as f64).to_bits() as i64),
            Literal::Double(f) => self.emit_li(A0, f.to_bits() as i64),
            Literal::String(s) => {
                let s = s.clone();
                self.emit_lea_string(A0, &s);
            }
            Literal::NullPointer => self.emit_li(A0, 0),
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
                self.emit_beqz(A0, &end);
                self.gen_expr(right)?;
                self.r_type(0x0, A0, ZERO, 0x3, A0, OP); // sltu a0, zero, a0 → a0 = (a0!=0)
                self.define_label(&end);
                return Ok(());
            }
            BinaryOp::LogicalOr => {
                let set1 = self.new_label("lor_one");
                let end = self.new_label("lor_end");
                self.gen_expr(left)?;
                self.emit_bnez(A0, &set1);
                self.gen_expr(right)?;
                self.emit_bnez(A0, &set1);
                self.emit_li(A0, 0);
                self.emit_j(&end);
                self.define_label(&set1);
                self.emit_li(A0, 1);
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
        self.push(A0);
        self.gen_expr(right)?;
        self.emit_mv(T1, A0); // t1 = right
        self.pop(A0); // a0 = left

        match op {
            BinaryOp::Add => self.r_type(0x0, T1, A0, 0x0, A0, OP),
            BinaryOp::Subtract => self.r_type(0x20, T1, A0, 0x0, A0, OP),
            BinaryOp::Multiply => self.r_type(0x01, T1, A0, 0x0, A0, OP),
            BinaryOp::Divide => self.r_type(0x01, T1, A0, 0x4, A0, OP),
            BinaryOp::Modulo => self.r_type(0x01, T1, A0, 0x6, A0, OP),
            BinaryOp::BitAnd => self.r_type(0x0, T1, A0, 0x7, A0, OP),
            BinaryOp::BitOr => self.r_type(0x0, T1, A0, 0x6, A0, OP),
            BinaryOp::BitXor => self.r_type(0x0, T1, A0, 0x4, A0, OP),
            BinaryOp::LeftShift => self.r_type(0x0, T1, A0, 0x1, A0, OP),
            BinaryOp::RightShift => self.r_type(0x0, T1, A0, 0x5, A0, OP), // srl
            BinaryOp::RightShiftArithmetic => self.r_type(0x20, T1, A0, 0x5, A0, OP), // sra
            BinaryOp::Equal => {
                self.r_type(0x20, T1, A0, 0x0, A0, OP); // sub
                self.i_type(1, A0, 0x3, A0, OP_IMM); // sltiu a0,a0,1
            }
            BinaryOp::NotEqual => {
                self.r_type(0x20, T1, A0, 0x0, A0, OP); // sub
                self.r_type(0x0, A0, ZERO, 0x3, A0, OP); // sltu a0, zero, a0
            }
            BinaryOp::LessThan => self.r_type(0x0, T1, A0, 0x2, A0, OP), // slt a0,a0,t1
            BinaryOp::GreaterThan => self.r_type(0x0, A0, T1, 0x2, A0, OP), // slt a0,t1,a0
            BinaryOp::LessThanEqual => {
                self.r_type(0x0, A0, T1, 0x2, A0, OP); // slt a0,t1,a0  (t1<a0)
                self.i_type(1, A0, 0x4, A0, OP_IMM); // xori a0,a0,1
            }
            BinaryOp::GreaterThanEqual => {
                self.r_type(0x0, T1, A0, 0x2, A0, OP); // slt a0,a0,t1  (a0<t1)
                self.i_type(1, A0, 0x4, A0, OP_IMM); // xori a0,a0,1
            }
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => unreachable!(),
        }
        Ok(())
    }

    /// 表达式静态类型是否为浮点
    fn expr_is_float(&self, e: &lir::Expression) -> bool {
        matches!(
            self.infer_expr_type(e),
            Some(lir::Type::Float) | Some(lir::Type::Double) | Some(lir::Type::LongDouble)
        )
    }

    /// 浮点二元运算。值模型同整数：8 字节位存于 GPR/栈。计算时搬入 f0/f1，
    /// 必要时把整数操作数用 fcvt.d.l 提升为 double；算术结果位回到 a0，
    /// 比较结果为 a0 中的 0/1。
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
        self.push(A0);
        self.gen_expr(right)?;
        self.emit_mv(T1, A0); // t1 = right 位/整数
        self.pop(A0); // a0 = left 位/整数

        // f0 = left, f1 = right（必要时 int->double）
        if lf {
            self.emit_fmv_d_x(0, A0);
        } else {
            self.emit_fcvt_d_l(0, A0);
        }
        if rf {
            self.emit_fmv_d_x(1, T1);
        } else {
            self.emit_fcvt_d_l(1, T1);
        }

        match op {
            BinaryOp::Add => {
                self.emit_fadd_d(0, 0, 1);
                self.emit_fmv_x_d(A0, 0);
            }
            BinaryOp::Subtract => {
                self.emit_fsub_d(0, 0, 1);
                self.emit_fmv_x_d(A0, 0);
            }
            BinaryOp::Multiply => {
                self.emit_fmul_d(0, 0, 1);
                self.emit_fmv_x_d(A0, 0);
            }
            BinaryOp::Divide => {
                self.emit_fdiv_d(0, 0, 1);
                self.emit_fmv_x_d(A0, 0);
            }
            BinaryOp::Equal => self.emit_feq_d(A0, 0, 1),
            BinaryOp::NotEqual => {
                self.emit_feq_d(A0, 0, 1);
                self.i_type(1, A0, 0x4, A0, OP_IMM); // xori a0,a0,1
            }
            BinaryOp::LessThan => self.emit_flt_d(A0, 0, 1),
            BinaryOp::LessThanEqual => self.emit_fle_d(A0, 0, 1),
            BinaryOp::GreaterThan => self.emit_flt_d(A0, 1, 0),
            BinaryOp::GreaterThanEqual => self.emit_fle_d(A0, 1, 0),
            _ => {
                self.emit_fadd_d(0, 0, 1);
                self.emit_fmv_x_d(A0, 0);
            }
        }
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
                    self.store_local(off, A0);
                } else if self.globals.contains_key(name) {
                    self.emit_mv(T1, A0);
                    self.emit_lea_global(A0, name)?;
                    self.emit_sd(T1, A0, 0);
                } else {
                    return Err(NativeError::CodegenError(format!(
                        "未定义赋值目标: {}",
                        name
                    )));
                }
            }
            Expression::Dereference(ptr) => {
                self.gen_expr(value)?;
                self.push(A0);
                self.gen_expr(ptr)?;
                self.pop(T1);
                self.emit_sd(T1, A0, 0);
            }
            Expression::Member(obj, field) => {
                self.gen_expr(value)?;
                self.push(A0);
                self.gen_expr(obj)?;
                let off = self.resolve_field_offset(obj, field, false).unwrap_or(0);
                if off > 0 {
                    self.emit_addi(A0, A0, off as i32);
                }
                self.pop(T1);
                self.emit_sd(T1, A0, 0);
            }
            Expression::Index(arr, idx) => {
                let elem_ty = self.index_elem_type(arr);
                let esz = elem_ty.as_ref().map(|t| self.type_size(t)).unwrap_or(8) as i64;
                self.gen_expr(value)?;
                self.push(A0);
                self.gen_expr(arr)?;
                self.push(A0);
                self.gen_expr(idx)?;
                self.emit_li(T2, esz);
                self.r_type(0x01, T2, A0, 0x0, A0, OP); // mul a0,a0,t2
                self.pop(T1); // base
                self.r_type(0x0, A0, T1, 0x0, A0, OP); // add a0,t1,a0
                self.pop(T1); // value
                self.emit_sd(T1, A0, 0);
            }
            _ => {
                self.gen_expr(value)?;
                log::debug!("riscv64: 未支持的赋值目标");
            }
        }
        Ok(())
    }

    fn gen_inc_dec(&mut self, target: &lir::Expression, delta: i32, pre: bool) -> NativeResult<()> {
        use lir::Expression;
        if let Expression::Variable(name) = target {
            if let Some(&off) = self.local_offsets.get(name) {
                self.load_local(A0, off);
                if !pre {
                    self.push(A0);
                }
                self.emit_addi(A0, A0, delta);
                self.store_local(off, A0);
                if !pre {
                    self.pop(A0);
                }
                return Ok(());
            }
        }
        self.gen_expr(target)?;
        self.emit_addi(A0, A0, delta);
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
        // 求值实参 → 入栈（逆序，栈顶为 arg0）→ 按 lp64d ABI 分发到
        // 整型寄存器 a0..a7 或浮点寄存器 fa0..fa7。
        let n = args.len().min(ARG_REGS.len());
        let floats: Vec<bool> = args.iter().take(n).map(|a| self.expr_is_float(a)).collect();
        for a in args.iter().take(n).rev() {
            self.gen_expr(a)?;
            self.push(A0);
        }
        let indirect = direct.is_none();
        if indirect {
            self.gen_expr(func)?;
            self.emit_mv(T2, A0);
        }
        let mut ngrn = 0usize;
        let mut nsrn = 0usize;
        for i in 0..n {
            self.pop(T1); // t1 = arg i 位
            if floats[i] {
                self.emit_fmv_d_x(FARG_REGS[nsrn], T1);
                nsrn += 1;
            } else {
                self.emit_mv(ARG_REGS[ngrn], T1);
                ngrn += 1;
            }
        }
        match direct {
            Some(name) => self.emit_call(&name),
            None => self.i_type(0, T2, 0x0, RA, JALR), // jalr ra, t2, 0
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

        for (idx, a) in args.iter().enumerate().rev() {
            if kinds[idx] == PK::Bool {
                self.gen_bool_to_cstr(a)?;
            } else {
                self.gen_expr(a)?;
            }
            self.push(A0);
        }
        self.emit_lea_string(A0, &fmt);
        self.push(A0);
        if to_stderr {
            self.emit_li(A0, 2);
            self.push(A0);
        }
        let total = lead + args.len();
        let nreg = total.min(ARG_REGS.len());
        for i in 0..nreg {
            self.pop(ARG_REGS[i]);
        }
        self.emit_call(callee);
        Ok(())
    }

    fn gen_bool_to_cstr(&mut self, e: &lir::Expression) -> NativeResult<()> {
        self.gen_expr(e)?;
        let lt = self.new_label("bt");
        let le = self.new_label("be");
        self.emit_bnez(A0, &lt);
        self.emit_lea_string(A0, "false");
        self.emit_j(&le);
        self.define_label(&lt);
        self.emit_lea_string(A0, "true");
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

/// 打印种类（与 aarch64 共用语义）
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

/// 仅生成 B 型立即数的位（其它字段保持 0）
fn encode_b_imm(imm: i32) -> u32 {
    let imm = imm as u32;
    let b12 = (imm >> 12) & 1;
    let b10_5 = (imm >> 5) & 0x3f;
    let b4_1 = (imm >> 1) & 0xf;
    let b11 = (imm >> 11) & 1;
    (b12 << 31) | (b10_5 << 25) | (b4_1 << 8) | (b11 << 7)
}

/// 仅生成 J 型立即数的位
fn encode_j_imm(imm: i32) -> u32 {
    let imm = imm as u32;
    let b20 = (imm >> 20) & 1;
    let b10_1 = (imm >> 1) & 0x3ff;
    let b11 = (imm >> 11) & 1;
    let b19_12 = (imm >> 12) & 0xff;
    (b20 << 31) | (b10_1 << 21) | (b11 << 20) | (b19_12 << 12)
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_lir::{self as lir, Expression, Literal, Statement, Type};

    fn gen(program: &lir::Program) -> MachineObject {
        RiscV64CodeGen::new(TargetOS::Linux)
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
        assert!(obj.text.len() % 4 == 0); // 定长 32 位指令
        assert!(obj.symbols.iter().any(|s| s.name == "main" && s.is_func));
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
        // 应有对 printf 的调用重定位与字符串数据
        assert!(obj
            .relocations
            .iter()
            .any(|r| matches!(r.kind, RelKind::RiscvCallPlt)));
        assert!(obj.rodata.windows(2).any(|w| w == b"hi"));
    }

    #[test]
    fn test_li_roundtrip_small() {
        // 间接验证 emit_li 不 panic 且产生指令
        let mut g = RiscV64CodeGen::new(TargetOS::Linux);
        g.emit_li(A0, 0x1234_5678_9abc_def0u64 as i64);
        assert!(!g.text.is_empty());
    }
}
