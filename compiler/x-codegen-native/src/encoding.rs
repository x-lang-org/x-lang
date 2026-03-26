//! 机器码编码器
//!
//! 将汇编指令编码为机器码字节序列

use crate::arch::{X86Register, TargetArch};
use crate::emitter::BinaryEmitter;

// ============================================================================
// 编码器 trait
// ============================================================================

/// 机器码编码器 trait
pub trait MachineCodeEncoder {
    /// 获取目标架构
    fn arch(&self) -> TargetArch;

    /// 编码指令并返回字节序列
    fn encode(&mut self) -> Vec<u8>;
}

// ============================================================================
// x86-64 编码器
// ============================================================================

/// x86-64 机器码编码器
///
/// 支持基本指令的编码，遵循 Intel 64 和 AMD64 架构手册
pub struct X86_64Encoder {
    /// 输出缓冲区
    buffer: Vec<u8>,
    /// REX 前缀
    rex: Option<u8>,
    /// 是否需要 67 前缀（地址大小覆盖）
    address_size_override: bool,
    /// 是否需要 48 前缀（操作数大小覆盖）
    operand_size_override: bool,
}

impl X86_64Encoder {
    /// 创建新的 x86-64 编码器
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            rex: None,
            address_size_override: false,
            operand_size_override: false,
        }
    }

    /// 清空编码器状态
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.rex = None;
        self.address_size_override = false;
        self.operand_size_override = false;
    }

    /// 获取编码结果
    pub fn result(&self) -> &[u8] {
        &self.buffer
    }

    /// 写入字节
    pub fn emit_byte(&mut self, b: u8) -> &mut Self {
        self.buffer.push(b);
        self
    }

    /// 写入字（16 位）
    pub fn emit_word(&mut self, w: u16) -> &mut Self {
        self.buffer.extend_from_slice(&w.to_le_bytes());
        self
    }

    /// 写入双字（32 位）
    pub fn emit_dword(&mut self, d: u32) -> &mut Self {
        self.buffer.extend_from_slice(&d.to_le_bytes());
        self
    }

    /// 写入四字（64 位）
    pub fn emit_qword(&mut self, q: u64) -> &mut Self {
        self.buffer.extend_from_slice(&q.to_le_bytes());
        self
    }

    /// 设置 REX 前缀
    pub fn set_rex(&mut self, w: bool, r: bool, x: bool, b: bool) -> &mut Self {
        let mut rex = 0x40; // REX 前缀基础值
        if w {
            rex |= 0x08; // REX.W
        }
        if r {
            rex |= 0x04; // REX.R
        }
        if x {
            rex |= 0x02; // REX.X
        }
        if b {
            rex |= 0x01; // REX.B
        }
        self.rex = Some(rex);
        self
    }

    /// 设置 REX.W 前缀（64 位操作数）
    pub fn set_rex_w(&mut self) -> &mut Self {
        self.set_rex(true, false, false, false)
    }

    /// 发射前缀
    pub fn emit_prefixes(&mut self) -> &mut Self {
        if self.operand_size_override {
            self.emit_byte(0x66);
        }
        if self.address_size_override {
            self.emit_byte(0x67);
        }
        if let Some(rex) = self.rex {
            self.emit_byte(rex);
        }
        self
    }

    /// 编码 ModR/M 字节
    ///
    /// ModR/M 字节格式：
    /// - Mod (2 bits): 寻址模式
    /// - Reg/Opcode (3 bits): 寄存器或操作码扩展
    /// - R/M (3 bits): 寄存器/内存操作数
    pub fn modrm(&self, mod_: u8, reg: u8, rm: u8) -> u8 {
        ((mod_ & 0x03) << 6) | ((reg & 0x07) << 3) | (rm & 0x07)
    }

    /// 编码 SIB 字节
    ///
    /// SIB 字节格式：
    /// - Scale (2 bits): 比例因子 (1, 2, 4, 8)
    /// - Index (3 bits): 索引寄存器
    /// - Base (3 bits): 基址寄存器
    pub fn sib(&self, scale: u8, index: u8, base: u8) -> u8 {
        let scale_bits = match scale {
            1 => 0,
            2 => 1,
            4 => 2,
            8 => 3,
            _ => 0,
        };
        ((scale_bits & 0x03) << 6) | ((index & 0x07) << 3) | (base & 0x07)
    }

    /// 获取 x86-64 寄存器编号 (0-15)
    pub fn reg_number(reg: X86Register) -> (u8, bool) {
        let (num, needs_rex) = match reg {
            X86Register::Rax | X86Register::Eax | X86Register::Ax | X86Register::Al => (0, false),
            X86Register::Rbx | X86Register::Ebx | X86Register::Bx | X86Register::Bl => (3, false),
            X86Register::Rcx | X86Register::Ecx | X86Register::Cx | X86Register::Cl => (1, false),
            X86Register::Rdx | X86Register::Edx | X86Register::Dx | X86Register::Dl => (2, false),
            X86Register::Rsi | X86Register::Esi | X86Register::Si | X86Register::Sil => (6, false),
            X86Register::Rdi | X86Register::Edi | X86Register::Di | X86Register::Dil => (7, false),
            X86Register::Rbp | X86Register::Ebp | X86Register::Bp | X86Register::Bpl => (5, false),
            X86Register::Rsp | X86Register::Esp | X86Register::Sp | X86Register::Spl => (4, false),
            X86Register::R8 | X86Register::R8d | X86Register::R8w | X86Register::R8b => (8, true),
            X86Register::R9 | X86Register::R9d | X86Register::R9w | X86Register::R9b => (9, true),
            X86Register::R10 | X86Register::R10d | X86Register::R10w | X86Register::R10b => {
                (10, true)
            }
            X86Register::R11 | X86Register::R11d | X86Register::R11w | X86Register::R11b => {
                (11, true)
            }
            X86Register::R12 | X86Register::R12d | X86Register::R12w | X86Register::R12b => {
                (12, true)
            }
            X86Register::R13 | X86Register::R13d | X86Register::R13w | X86Register::R13b => {
                (13, true)
            }
            X86Register::R14 | X86Register::R14d | X86Register::R14w | X86Register::R14b => {
                (14, true)
            }
            X86Register::R15 | X86Register::R15d | X86Register::R15w | X86Register::R15b => {
                (15, true)
            }
            _ => (0, false),
        };
        (num, needs_rex)
    }

    // ========================================================================
    // 基本指令编码
    // ========================================================================

    /// 编码 MOV r64, imm64
    ///
    /// 操作码: REX.W + B8+ rd
    pub fn mov_reg_imm64(&mut self, dest: X86Register, imm: u64) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(dest);
        if needs_rex {
            self.set_rex(true, false, false, num >= 8);
        } else {
            self.set_rex_w();
        }
        self.emit_prefixes();
        self.emit_byte(0xB8 + (num & 0x07));
        self.emit_qword(imm);
        self
    }

    /// 编码 MOV r64, r64
    ///
    /// 操作码: REX.W + 89 /r
    pub fn mov_reg_reg(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, dest_rex) = Self::reg_number(dest);
        let (src_num, src_rex) = Self::reg_number(src);
        self.set_rex(true, src_num >= 8, false, dest_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x89);
        self.emit_byte(self.modrm(3, src_num & 0x07, dest_num & 0x07));
        self
    }

    /// 编码 MOV r64, [r64 + disp32]
    ///
    /// 操作码: REX.W + 8B /r
    pub fn mov_reg_mem(&mut self, dest: X86Register, base: X86Register, disp: i32) -> &mut Self {
        let (dest_num, dest_rex) = Self::reg_number(dest);
        let (base_num, base_rex) = Self::reg_number(base);
        self.set_rex(true, dest_num >= 8, false, base_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x8B);
        self.emit_byte(self.modrm(2, dest_num & 0x07, base_num & 0x07));
        self.emit_dword(disp as u32);
        let _ = (dest_rex, base_rex);
        self
    }

    /// 编码 MOV [r64 + disp32], r64
    pub fn mov_mem_reg(&mut self, base: X86Register, disp: i32, src: X86Register) -> &mut Self {
        let (src_num, _) = Self::reg_number(src);
        let (base_num, _) = Self::reg_number(base);
        self.set_rex(true, src_num >= 8, false, base_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x89);
        self.emit_byte(self.modrm(2, src_num & 0x07, base_num & 0x07));
        self.emit_dword(disp as u32);
        self
    }

    /// 编码 PUSH r64
    ///
    /// 操作码: 50+ rd
    pub fn push_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, true);
            self.emit_prefixes();
        }
        self.emit_byte(0x50 + (num & 0x07));
        self
    }

    /// 编码 POP r64
    ///
    /// 操作码: 58+ rd
    pub fn pop_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, true);
            self.emit_prefixes();
        }
        self.emit_byte(0x58 + (num & 0x07));
        self
    }

    /// 编码 ADD r64, r64
    ///
    /// 操作码: REX.W + 01 /r
    pub fn add_reg_reg(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (src_num, _) = Self::reg_number(src);
        self.set_rex(true, src_num >= 8, false, dest_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x01);
        self.emit_byte(self.modrm(3, src_num & 0x07, dest_num & 0x07));
        self
    }

    /// 编码 SUB r64, r64
    ///
    /// 操作码: REX.W + 29 /r
    pub fn sub_reg_reg(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (src_num, _) = Self::reg_number(src);
        self.set_rex(true, src_num >= 8, false, dest_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x29);
        self.emit_byte(self.modrm(3, src_num & 0x07, dest_num & 0x07));
        self
    }

    /// 编码 IMUL r64, r64
    ///
    /// 操作码: REX.W + 0F AF /r
    pub fn imul_reg_reg(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (src_num, _) = Self::reg_number(src);
        self.set_rex(true, dest_num >= 8, false, src_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x0F);
        self.emit_byte(0xAF);
        self.emit_byte(self.modrm(3, dest_num & 0x07, src_num & 0x07));
        self
    }

    /// 编码 XOR r64, r64 (常用于清零)
    ///
    /// 操作码: REX.W + 31 /r
    pub fn xor_reg_reg(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (src_num, _) = Self::reg_number(src);
        self.set_rex(true, src_num >= 8, false, dest_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x31);
        self.emit_byte(self.modrm(3, src_num & 0x07, dest_num & 0x07));
        self
    }

    /// 编码 CMP r64, r64
    ///
    /// 操作码: REX.W + 39 /r
    pub fn cmp_reg_reg(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (src_num, _) = Self::reg_number(src);
        self.set_rex(true, src_num >= 8, false, dest_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x39);
        self.emit_byte(self.modrm(3, src_num & 0x07, dest_num & 0x07));
        self
    }

    /// 编码 TEST r64, r64
    ///
    /// 操作码: REX.W + 85 /r
    pub fn test_reg_reg(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (src_num, _) = Self::reg_number(src);
        self.set_rex(true, src_num >= 8, false, dest_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x85);
        self.emit_byte(self.modrm(3, src_num & 0x07, dest_num & 0x07));
        self
    }

    /// 编码 CALL rel32
    ///
    /// 操作码: E8 cd
    pub fn call_rel32(&mut self, offset: i32) -> &mut Self {
        self.emit_byte(0xE8);
        self.emit_dword(offset as u32);
        self
    }

    /// 编码 CALL r64 (间接调用)
    ///
    /// 操作码: REX.W + FF /2
    pub fn call_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, num >= 8);
            self.emit_prefixes();
        }
        self.emit_byte(0xFF);
        self.emit_byte(self.modrm(3, 2, num & 0x07));
        self
    }

    /// 编码 RET
    ///
    /// 操作码: C3
    pub fn ret(&mut self) -> &mut Self {
        self.emit_byte(0xC3);
        self
    }

    /// 编码 NOP
    ///
    /// 操作码: 90
    pub fn nop(&mut self) -> &mut Self {
        self.emit_byte(0x90);
        self
    }

    /// 编码 LEA r64, [r64 + disp32]
    ///
    /// 操作码: REX.W + 8D /r
    pub fn lea_reg_mem(&mut self, dest: X86Register, base: X86Register, disp: i32) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (base_num, _) = Self::reg_number(base);
        self.set_rex(true, dest_num >= 8, false, base_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x8D);
        self.emit_byte(self.modrm(2, dest_num & 0x07, base_num & 0x07));
        self.emit_dword(disp as u32);
        self
    }

    /// 编码条件跳转 (Jcc)
    ///
    /// 操作码: 0F 8x cd
    pub fn jcc(&mut self, cond: Condition, offset: i32) -> &mut Self {
        self.emit_byte(0x0F);
        self.emit_byte(0x80 | (cond as u8));
        self.emit_dword(offset as u32);
        self
    }

    /// 编码 JMP rel32
    ///
    /// 操作码: E9 cd
    pub fn jmp_rel32(&mut self, offset: i32) -> &mut Self {
        self.emit_byte(0xE9);
        self.emit_dword(offset as u32);
        self
    }

    /// 编码 SETcc r8
    ///
    /// 操作码: 0F 9x /r
    pub fn setcc(&mut self, cond: Condition, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, num >= 8);
            self.emit_prefixes();
        }
        self.emit_byte(0x0F);
        self.emit_byte(0x90 | (cond as u8));
        self.emit_byte(self.modrm(3, 0, num & 0x07));
        self
    }

    /// 编码 MOVZX r64, r8
    ///
    /// 操作码: REX.W + 0F B6 /r
    pub fn movzx_r64_r8(&mut self, dest: X86Register, src: X86Register) -> &mut Self {
        let (dest_num, _) = Self::reg_number(dest);
        let (src_num, _) = Self::reg_number(src);
        self.set_rex(true, dest_num >= 8, false, src_num >= 8);
        self.emit_prefixes();
        self.emit_byte(0x0F);
        self.emit_byte(0xB6);
        self.emit_byte(self.modrm(3, dest_num & 0x07, src_num & 0x07));
        self
    }

    /// 编码 INC r64
    ///
    /// 操作码: REX.W + FF /0
    pub fn inc_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, num >= 8);
            self.emit_prefixes();
        }
        self.emit_byte(0xFF);
        self.emit_byte(self.modrm(3, 0, num & 0x07));
        self
    }

    /// 编码 DEC r64
    ///
    /// 操作码: REX.W + FF /1
    pub fn dec_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, num >= 8);
            self.emit_prefixes();
        }
        self.emit_byte(0xFF);
        self.emit_byte(self.modrm(3, 1, num & 0x07));
        self
    }

    /// 编码 NEG r64
    ///
    /// 操作码: REX.W + F7 /3
    pub fn neg_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, num >= 8);
            self.emit_prefixes();
        }
        self.emit_byte(0xF7);
        self.emit_byte(self.modrm(3, 3, num & 0x07));
        self
    }

    /// 编码 NOT r64
    ///
    /// 操作码: REX.W + F7 /2
    pub fn not_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, num >= 8);
            self.emit_prefixes();
        }
        self.emit_byte(0xF7);
        self.emit_byte(self.modrm(3, 2, num & 0x07));
        self
    }

    /// 编码 IDIV r64
    ///
    /// 操作码: REX.W + F7 /7
    pub fn idiv_reg(&mut self, reg: X86Register) -> &mut Self {
        let (num, needs_rex) = Self::reg_number(reg);
        if needs_rex {
            self.set_rex(false, false, false, num >= 8);
            self.emit_prefixes();
        }
        self.emit_byte(0xF7);
        self.emit_byte(self.modrm(3, 7, num & 0x07));
        self
    }

    /// 编码 CQO (Convert Quadword to Octaword)
    ///
    /// 操作码: REX.W + 99
    pub fn cqo(&mut self) -> &mut Self {
        self.set_rex_w();
        self.emit_prefixes();
        self.emit_byte(0x99);
        self
    }
}

impl Default for X86_64Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl MachineCodeEncoder for X86_64Encoder {
    fn arch(&self) -> TargetArch {
        TargetArch::X86_64
    }

    fn encode(&mut self) -> Vec<u8> {
        self.buffer.clone()
    }
}

// ============================================================================
// 条件码
// ============================================================================

/// x86 条件码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Condition {
    /// 等于 (ZF=1) / 零标志
    E = 0x04,
    /// 不等于 (ZF=0) / 非零标志
    NE = 0x05,
    /// 小于 (SF!=OF)
    L = 0x0C,
    /// 小于等于 (ZF=1 or SF!=OF)
    LE = 0x0E,
    /// 大于 (ZF=0 and SF=OF)
    G = 0x0F,
    /// 大于等于 (SF=OF)
    GE = 0x0D,
    /// 无符号小于 (CF=1)
    B = 0x02,
    /// 无符号小于等于 (CF=1 or ZF=1)
    BE = 0x06,
    /// 无符号大于 (CF=0 and ZF=0)
    A = 0x07,
    /// 无符号大于等于 (CF=0)
    AE = 0x03,
    /// 符号标志
    S = 0x08,
    /// 非符号标志
    NS = 0x09,
    /// 溢出标志
    O = 0x00,
    /// 非溢出标志
    NO = 0x01,
}

// ============================================================================
// AArch64 编码器（简化实现）
// ============================================================================

/// AArch64 机器码编码器
pub struct AArch64Encoder {
    buffer: Vec<u8>,
}

impl AArch64Encoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    /// 获取编码结果
    pub fn result(&self) -> &[u8] {
        &self.buffer
    }

    /// 发射 32 位指令
    pub fn emit_instruction(&mut self, instr: u32) -> &mut Self {
        self.buffer.extend_from_slice(&instr.to_le_bytes());
        self
    }

    /// 编码 ADD Xd, Xn, Xm
    pub fn add_x(&mut self, dest: u8, src1: u8, src2: u8) -> &mut Self {
        let instr = 0x8B00_0000 | ((src2 as u32) << 16) | ((src1 as u32) << 5) | (dest as u32);
        self.emit_instruction(instr)
    }

    /// 编码 SUB Xd, Xn, Xm
    pub fn sub_x(&mut self, dest: u8, src1: u8, src2: u8) -> &mut Self {
        let instr = 0xCB00_0000 | ((src2 as u32) << 16) | ((src1 as u32) << 5) | (dest as u32);
        self.emit_instruction(instr)
    }

    /// 编码 MOV Xd, Xm (ORR Xd, XZR, Xm)
    pub fn mov_x(&mut self, dest: u8, src: u8) -> &mut Self {
        let instr = 0xAA00_03E0 | ((src as u32) << 16) | (dest as u32);
        self.emit_instruction(instr)
    }

    /// 编码 LDR Xt, [Xn, #imm]
    pub fn ldr_x(&mut self, dest: u8, base: u8, offset: u16) -> &mut Self {
        let instr = 0xF940_0000 | ((offset as u32 / 8) << 10) | ((base as u32) << 5) | (dest as u32);
        self.emit_instruction(instr)
    }

    /// 编码 STR Xt, [Xn, #imm]
    pub fn str_x(&mut self, src: u8, base: u8, offset: u16) -> &mut Self {
        let instr = 0xF900_0000 | ((offset as u32 / 8) << 10) | ((base as u32) << 5) | (src as u32);
        self.emit_instruction(instr)
    }

    /// 编码 BL (带链接的分支)
    pub fn bl(&mut self, offset: i32) -> &mut Self {
        let imm26 = (offset / 4) as u32 & 0x03FF_FFFF;
        let instr = 0x9400_0000 | imm26;
        self.emit_instruction(instr)
    }

    /// 编码 RET
    pub fn ret(&mut self) -> &mut Self {
        self.emit_instruction(0xD65F_03C0)
    }

    /// 编码 NOP
    pub fn nop(&mut self) -> &mut Self {
        self.emit_instruction(0xD503_201F)
    }
}

impl Default for AArch64Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl MachineCodeEncoder for AArch64Encoder {
    fn arch(&self) -> TargetArch {
        TargetArch::AArch64
    }

    fn encode(&mut self) -> Vec<u8> {
        self.buffer.clone()
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x86_64_encoder_creation() {
        let encoder = X86_64Encoder::new();
        assert!(encoder.result().is_empty());
    }

    #[test]
    fn test_mov_reg_imm64() {
        let mut encoder = X86_64Encoder::new();
        encoder.mov_reg_imm64(X86Register::Rax, 0x1234567890ABCDEF);
        let result = encoder.result();

        // REX.W + B8 + imm64
        assert_eq!(result[0], 0x48); // REX.W
        assert_eq!(result[1], 0xB8); // MOV rax, imm64 opcode
    }

    #[test]
    fn test_mov_reg_reg() {
        let mut encoder = X86_64Encoder::new();
        encoder.mov_reg_reg(X86Register::Rax, X86Register::Rbx);
        let result = encoder.result();

        assert_eq!(result[0], 0x48); // REX.W
        assert_eq!(result[1], 0x89); // MOV opcode
    }

    #[test]
    fn test_push_pop() {
        let mut encoder = X86_64Encoder::new();
        encoder.push_reg(X86Register::Rbp);
        encoder.pop_reg(X86Register::Rbp);
        let result = encoder.result();

        assert_eq!(result[0], 0x55); // PUSH rbp
        assert_eq!(result[1], 0x5D); // POP rbp
    }

    #[test]
    fn test_ret() {
        let mut encoder = X86_64Encoder::new();
        encoder.ret();
        assert_eq!(encoder.result(), &[0xC3]);
    }

    #[test]
    fn test_nop() {
        let mut encoder = X86_64Encoder::new();
        encoder.nop();
        assert_eq!(encoder.result(), &[0x90]);
    }

    #[test]
    fn test_xor_reg_reg() {
        let mut encoder = X86_64Encoder::new();
        encoder.xor_reg_reg(X86Register::Rax, X86Register::Rax);
        let result = encoder.result();

        assert_eq!(result[0], 0x48); // REX.W
        assert_eq!(result[1], 0x31); // XOR opcode
    }

    #[test]
    fn test_add_sub() {
        let mut encoder = X86_64Encoder::new();
        encoder.add_reg_reg(X86Register::Rax, X86Register::Rbx);
        let add_result = encoder.result().to_vec();

        encoder.clear();
        encoder.sub_reg_reg(X86Register::Rax, X86Register::Rbx);
        let sub_result = encoder.result().to_vec();

        assert_eq!(add_result[1], 0x01); // ADD opcode
        assert_eq!(sub_result[1], 0x29); // SUB opcode
    }

    #[test]
    fn test_call_jmp() {
        let mut encoder = X86_64Encoder::new();
        encoder.call_rel32(0x100);
        let call_result = encoder.result().to_vec();

        encoder.clear();
        encoder.jmp_rel32(0x200);
        let jmp_result = encoder.result().to_vec();

        assert_eq!(call_result[0], 0xE8); // CALL opcode
        assert_eq!(jmp_result[0], 0xE9); // JMP opcode
    }

    #[test]
    fn test_jcc() {
        let mut encoder = X86_64Encoder::new();
        encoder.jcc(Condition::E, 0x100);
        let result = encoder.result();

        assert_eq!(result[0], 0x0F);
        assert_eq!(result[1], 0x84); // JE opcode
    }

    #[test]
    fn test_setcc() {
        let mut encoder = X86_64Encoder::new();
        encoder.setcc(Condition::E, X86Register::Al);
        let result = encoder.result();

        assert_eq!(result[0], 0x0F);
        assert_eq!(result[1], 0x94); // SETE opcode
    }

    #[test]
    fn test_reg_number() {
        assert_eq!(X86_64Encoder::reg_number(X86Register::Rax), (0, false));
        assert_eq!(X86_64Encoder::reg_number(X86Register::Rbx), (3, false));
        assert_eq!(X86_64Encoder::reg_number(X86Register::R8), (8, true));
        assert_eq!(X86_64Encoder::reg_number(X86Register::R15), (15, true));
    }

    #[test]
    fn test_modrm() {
        let encoder = X86_64Encoder::new();
        // Mod = 3 (register), Reg = 0 (rax), R/M = 3 (rbx)
        let modrm = encoder.modrm(3, 0, 3);
        assert_eq!(modrm, 0b11_000_011);
    }

    #[test]
    fn test_aarch64_encoder() {
        let mut encoder = AArch64Encoder::new();
        encoder.ret();
        let result = encoder.result();

        // RET 指令应该是 4 字节
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_aarch64_add() {
        let mut encoder = AArch64Encoder::new();
        encoder.add_x(0, 1, 2); // ADD X0, X1, X2
        let result = encoder.result();

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_condition_values() {
        assert_eq!(Condition::E as u8, 0x04);
        assert_eq!(Condition::NE as u8, 0x05);
        assert_eq!(Condition::L as u8, 0x0C);
        assert_eq!(Condition::G as u8, 0x0F);
    }

    #[test]
    fn test_full_function() {
        // 编码一个简单的函数：返回 42
        let mut encoder = X86_64Encoder::new();
        encoder
            .push_reg(X86Register::Rbp)        // push rbp
            .mov_reg_reg(X86Register::Rbp, X86Register::Rsp) // mov rbp, rsp
            .mov_reg_imm64(X86Register::Rax, 42) // mov rax, 42
            .pop_reg(X86Register::Rbp)         // pop rbp
            .ret();                            // ret

        let result = encoder.result();
        assert!(!result.is_empty());

        // 验证最后的 ret 指令
        assert_eq!(*result.last().unwrap(), 0xC3);
    }
}
