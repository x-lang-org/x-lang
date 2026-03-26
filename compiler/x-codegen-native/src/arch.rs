//! 架构抽象层
//!
//! 定义不同目标架构的统一接口，包括寄存器、指令和内存操作数。

use std::fmt::{self, Display};

// ============================================================================
// 目标架构
// ============================================================================

/// 目标架构
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetArch {
    /// x86-64 (AMD64)
    #[default]
    X86_64,
    /// ARM64 (AArch64)
    AArch64,
    /// RISC-V 64-bit
    RiscV64,
    /// WebAssembly 32-bit
    Wasm32,
}

impl TargetArch {
    /// 获取架构名称
    pub fn name(&self) -> &'static str {
        match self {
            TargetArch::X86_64 => "x86_64",
            TargetArch::AArch64 => "aarch64",
            TargetArch::RiscV64 => "riscv64",
            TargetArch::Wasm32 => "wasm32",
        }
    }

    /// 获取指针大小（字节）
    pub fn pointer_size(&self) -> usize {
        match self {
            TargetArch::X86_64 => 8,
            TargetArch::AArch64 => 8,
            TargetArch::RiscV64 => 8,
            TargetArch::Wasm32 => 4,
        }
    }

    /// 获取默认对齐
    pub fn default_align(&self) -> usize {
        self.pointer_size()
    }

    /// 是否为大端序
    pub fn is_big_endian(&self) -> bool {
        match self {
            TargetArch::X86_64 => false,
            TargetArch::AArch64 => false, // ARM64 通常运行在小端模式
            TargetArch::RiscV64 => false,
            TargetArch::Wasm32 => false,
        }
    }
}

impl Display for TargetArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// ============================================================================
// 寄存器
// ============================================================================

/// 通用寄存器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    // x86-64 寄存器
    X86(X86Register),
    // AArch64 寄存器
    AArch64(AArch64Register),
    // RISC-V 寄存器
    RiscV(RiscVRegister),
    // Wasm 局部变量索引
    WasmLocal(u32),
}

impl Register {
    /// 创建 x86-64 寄存器
    pub fn x86(reg: X86Register) -> Self {
        Register::X86(reg)
    }

    /// 创建 AArch64 寄存器
    pub fn aarch64(reg: AArch64Register) -> Self {
        Register::AArch64(reg)
    }

    /// 创建 RISC-V 寄存器
    pub fn riscv(reg: RiscVRegister) -> Self {
        Register::RiscV(reg)
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::X86(r) => write!(f, "{}", r),
            Register::AArch64(r) => write!(f, "{}", r),
            Register::RiscV(r) => write!(f, "{}", r),
            Register::WasmLocal(i) => write!(f, "local{}", i),
        }
    }
}

// ============================================================================
// x86-64 寄存器
// ============================================================================

/// x86-64 通用寄存器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum X86Register {
    // 64-bit 通用寄存器
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    // 32-bit 寄存器
    Eax,
    Ebx,
    Ecx,
    Edx,
    Esi,
    Edi,
    Ebp,
    Esp,
    R8d,
    R9d,
    R10d,
    R11d,
    R12d,
    R13d,
    R14d,
    R15d,
    // 16-bit 寄存器
    Ax,
    Bx,
    Cx,
    Dx,
    Si,
    Di,
    Bp,
    Sp,
    R8w,
    R9w,
    R10w,
    R11w,
    R12w,
    R13w,
    R14w,
    R15w,
    // 8-bit 寄存器
    Al,
    Bl,
    Cl,
    Dl,
    Sil,
    Dil,
    Bpl,
    Spl,
    R8b,
    R9b,
    R10b,
    R11b,
    R12b,
    R13b,
    R14b,
    R15b,
    Ah,
    Bh,
    Ch,
    Dh,
}

impl X86Register {
    /// 是否为被调用者保存寄存器
    pub fn is_callee_saved(&self) -> bool {
        matches!(
            self,
            X86Register::Rbx
                | X86Register::Rbp
                | X86Register::R12
                | X86Register::R13
                | X86Register::R14
                | X86Register::R15
                | X86Register::Ebx
                | X86Register::Ebp
                | X86Register::R12d
                | X86Register::R13d
                | X86Register::R14d
                | X86Register::R15d
        )
    }

    /// 是否为参数寄存器（System V ABI）
    pub fn is_argument_sysv(&self) -> bool {
        matches!(
            self,
            X86Register::Rdi
                | X86Register::Rsi
                | X86Register::Rdx
                | X86Register::Rcx
                | X86Register::R8
                | X86Register::R9
        )
    }

    /// 是否为参数寄存器（Windows x64 ABI）
    pub fn is_argument_windows(&self) -> bool {
        matches!(
            self,
            X86Register::Rcx | X86Register::Rdx | X86Register::R8 | X86Register::R9
        )
    }
}

impl Display for X86Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            X86Register::Rax => "rax",
            X86Register::Rbx => "rbx",
            X86Register::Rcx => "rcx",
            X86Register::Rdx => "rdx",
            X86Register::Rsi => "rsi",
            X86Register::Rdi => "rdi",
            X86Register::Rbp => "rbp",
            X86Register::Rsp => "rsp",
            X86Register::R8 => "r8",
            X86Register::R9 => "r9",
            X86Register::R10 => "r10",
            X86Register::R11 => "r11",
            X86Register::R12 => "r12",
            X86Register::R13 => "r13",
            X86Register::R14 => "r14",
            X86Register::R15 => "r15",
            X86Register::Eax => "eax",
            X86Register::Ebx => "ebx",
            X86Register::Ecx => "ecx",
            X86Register::Edx => "edx",
            X86Register::Esi => "esi",
            X86Register::Edi => "edi",
            X86Register::Ebp => "ebp",
            X86Register::Esp => "esp",
            X86Register::R8d => "r8d",
            X86Register::R9d => "r9d",
            X86Register::R10d => "r10d",
            X86Register::R11d => "r11d",
            X86Register::R12d => "r12d",
            X86Register::R13d => "r13d",
            X86Register::R14d => "r14d",
            X86Register::R15d => "r15d",
            X86Register::Ax => "ax",
            X86Register::Bx => "bx",
            X86Register::Cx => "cx",
            X86Register::Dx => "dx",
            X86Register::Si => "si",
            X86Register::Di => "di",
            X86Register::Bp => "bp",
            X86Register::Sp => "sp",
            X86Register::R8w => "r8w",
            X86Register::R9w => "r9w",
            X86Register::R10w => "r10w",
            X86Register::R11w => "r11w",
            X86Register::R12w => "r12w",
            X86Register::R13w => "r13w",
            X86Register::R14w => "r14w",
            X86Register::R15w => "r15w",
            X86Register::Al => "al",
            X86Register::Bl => "bl",
            X86Register::Cl => "cl",
            X86Register::Dl => "dl",
            X86Register::Sil => "sil",
            X86Register::Dil => "dil",
            X86Register::Bpl => "bpl",
            X86Register::Spl => "spl",
            X86Register::R8b => "r8b",
            X86Register::R9b => "r9b",
            X86Register::R10b => "r10b",
            X86Register::R11b => "r11b",
            X86Register::R12b => "r12b",
            X86Register::R13b => "r13b",
            X86Register::R14b => "r14b",
            X86Register::R15b => "r15b",
            X86Register::Ah => "ah",
            X86Register::Bh => "bh",
            X86Register::Ch => "ch",
            X86Register::Dh => "dh",
        };
        write!(f, "{}", name)
    }
}

// ============================================================================
// AArch64 寄存器
// ============================================================================

/// AArch64 通用寄存器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AArch64Register {
    // 64-bit 通用寄存器 X0-X30
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29, // Frame Pointer (FP)
    X30, // Link Register (LR)
    // 特殊寄存器
    Sp, // Stack Pointer
    Pc, // Program Counter
    // 32-bit 寄存器 W0-W30
    W0,
    W1,
    W2,
    W3,
    W4,
    W5,
    W6,
    W7,
    W8,
    W9,
    W10,
    W11,
    W12,
    W13,
    W14,
    W15,
    W16,
    W17,
    W18,
    W19,
    W20,
    W21,
    W22,
    W23,
    W24,
    W25,
    W26,
    W27,
    W28,
    W29,
    W30,
    Wsp,
}

impl AArch64Register {
    /// 是否为被调用者保存寄存器
    pub fn is_callee_saved(&self) -> bool {
        matches!(
            self,
            AArch64Register::X19
                | AArch64Register::X20
                | AArch64Register::X21
                | AArch64Register::X22
                | AArch64Register::X23
                | AArch64Register::X24
                | AArch64Register::X25
                | AArch64Register::X26
                | AArch64Register::X27
                | AArch64Register::X28
                | AArch64Register::X29
                | AArch64Register::W19
                | AArch64Register::W20
                | AArch64Register::W21
                | AArch64Register::W22
                | AArch64Register::W23
                | AArch64Register::W24
                | AArch64Register::W25
                | AArch64Register::W26
                | AArch64Register::W27
                | AArch64Register::W28
                | AArch64Register::W29
        )
    }

    /// 是否为参数寄存器
    pub fn is_argument(&self) -> bool {
        matches!(
            self,
            AArch64Register::X0
                | AArch64Register::X1
                | AArch64Register::X2
                | AArch64Register::X3
                | AArch64Register::X4
                | AArch64Register::X5
                | AArch64Register::X6
                | AArch64Register::X7
                | AArch64Register::W0
                | AArch64Register::W1
                | AArch64Register::W2
                | AArch64Register::W3
                | AArch64Register::W4
                | AArch64Register::W5
                | AArch64Register::W6
                | AArch64Register::W7
        )
    }
}

impl Display for AArch64Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AArch64Register::X0 => "x0",
            AArch64Register::X1 => "x1",
            AArch64Register::X2 => "x2",
            AArch64Register::X3 => "x3",
            AArch64Register::X4 => "x4",
            AArch64Register::X5 => "x5",
            AArch64Register::X6 => "x6",
            AArch64Register::X7 => "x7",
            AArch64Register::X8 => "x8",
            AArch64Register::X9 => "x9",
            AArch64Register::X10 => "x10",
            AArch64Register::X11 => "x11",
            AArch64Register::X12 => "x12",
            AArch64Register::X13 => "x13",
            AArch64Register::X14 => "x14",
            AArch64Register::X15 => "x15",
            AArch64Register::X16 => "x16",
            AArch64Register::X17 => "x17",
            AArch64Register::X18 => "x18",
            AArch64Register::X19 => "x19",
            AArch64Register::X20 => "x20",
            AArch64Register::X21 => "x21",
            AArch64Register::X22 => "x22",
            AArch64Register::X23 => "x23",
            AArch64Register::X24 => "x24",
            AArch64Register::X25 => "x25",
            AArch64Register::X26 => "x26",
            AArch64Register::X27 => "x27",
            AArch64Register::X28 => "x28",
            AArch64Register::X29 => "x29",
            AArch64Register::X30 => "x30",
            AArch64Register::Sp => "sp",
            AArch64Register::Pc => "pc",
            AArch64Register::W0 => "w0",
            AArch64Register::W1 => "w1",
            AArch64Register::W2 => "w2",
            AArch64Register::W3 => "w3",
            AArch64Register::W4 => "w4",
            AArch64Register::W5 => "w5",
            AArch64Register::W6 => "w6",
            AArch64Register::W7 => "w7",
            AArch64Register::W8 => "w8",
            AArch64Register::W9 => "w9",
            AArch64Register::W10 => "w10",
            AArch64Register::W11 => "w11",
            AArch64Register::W12 => "w12",
            AArch64Register::W13 => "w13",
            AArch64Register::W14 => "w14",
            AArch64Register::W15 => "w15",
            AArch64Register::W16 => "w16",
            AArch64Register::W17 => "w17",
            AArch64Register::W18 => "w18",
            AArch64Register::W19 => "w19",
            AArch64Register::W20 => "w20",
            AArch64Register::W21 => "w21",
            AArch64Register::W22 => "w22",
            AArch64Register::W23 => "w23",
            AArch64Register::W24 => "w24",
            AArch64Register::W25 => "w25",
            AArch64Register::W26 => "w26",
            AArch64Register::W27 => "w27",
            AArch64Register::W28 => "w28",
            AArch64Register::W29 => "w29",
            AArch64Register::W30 => "w30",
            AArch64Register::Wsp => "wsp",
        };
        write!(f, "{}", name)
    }
}

// ============================================================================
// RISC-V 寄存器
// ============================================================================

/// RISC-V 通用寄存器
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiscVRegister {
    // 整数寄存器 X0-X31
    Zero, // x0, always 0
    Ra,   // x1, return address
    Sp,   // x2, stack pointer
    Gp,   // x3, global pointer
    Tp,   // x4, thread pointer
    T0,   // x5-x7, temporaries
    T1,
    T2,
    S0, // x8, saved register (frame pointer)
    S1, // x9, saved register
    A0, // x10-x11, function arguments / return values
    A1,
    A2, // x12-x17, function arguments
    A3,
    A4,
    A5,
    A6,
    A7,
    S2, // x18-x27, saved registers
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3, // x28-x31, temporaries
    T4,
    T5,
    T6,
}

impl RiscVRegister {
    /// 是否为被调用者保存寄存器
    pub fn is_callee_saved(&self) -> bool {
        matches!(
            self,
            RiscVRegister::S0
                | RiscVRegister::S1
                | RiscVRegister::S2
                | RiscVRegister::S3
                | RiscVRegister::S4
                | RiscVRegister::S5
                | RiscVRegister::S6
                | RiscVRegister::S7
                | RiscVRegister::S8
                | RiscVRegister::S9
                | RiscVRegister::S10
                | RiscVRegister::S11
        )
    }

    /// 是否为参数寄存器
    pub fn is_argument(&self) -> bool {
        matches!(
            self,
            RiscVRegister::A0
                | RiscVRegister::A1
                | RiscVRegister::A2
                | RiscVRegister::A3
                | RiscVRegister::A4
                | RiscVRegister::A5
                | RiscVRegister::A6
                | RiscVRegister::A7
        )
    }

    /// 获取寄存器编号
    pub fn number(&self) -> u8 {
        match self {
            RiscVRegister::Zero => 0,
            RiscVRegister::Ra => 1,
            RiscVRegister::Sp => 2,
            RiscVRegister::Gp => 3,
            RiscVRegister::Tp => 4,
            RiscVRegister::T0 => 5,
            RiscVRegister::T1 => 6,
            RiscVRegister::T2 => 7,
            RiscVRegister::S0 => 8,
            RiscVRegister::S1 => 9,
            RiscVRegister::A0 => 10,
            RiscVRegister::A1 => 11,
            RiscVRegister::A2 => 12,
            RiscVRegister::A3 => 13,
            RiscVRegister::A4 => 14,
            RiscVRegister::A5 => 15,
            RiscVRegister::A6 => 16,
            RiscVRegister::A7 => 17,
            RiscVRegister::S2 => 18,
            RiscVRegister::S3 => 19,
            RiscVRegister::S4 => 20,
            RiscVRegister::S5 => 21,
            RiscVRegister::S6 => 22,
            RiscVRegister::S7 => 23,
            RiscVRegister::S8 => 24,
            RiscVRegister::S9 => 25,
            RiscVRegister::S10 => 26,
            RiscVRegister::S11 => 27,
            RiscVRegister::T3 => 28,
            RiscVRegister::T4 => 29,
            RiscVRegister::T5 => 30,
            RiscVRegister::T6 => 31,
        }
    }
}

impl Display for RiscVRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RiscVRegister::Zero => "zero",
            RiscVRegister::Ra => "ra",
            RiscVRegister::Sp => "sp",
            RiscVRegister::Gp => "gp",
            RiscVRegister::Tp => "tp",
            RiscVRegister::T0 => "t0",
            RiscVRegister::T1 => "t1",
            RiscVRegister::T2 => "t2",
            RiscVRegister::S0 => "s0",
            RiscVRegister::S1 => "s1",
            RiscVRegister::A0 => "a0",
            RiscVRegister::A1 => "a1",
            RiscVRegister::A2 => "a2",
            RiscVRegister::A3 => "a3",
            RiscVRegister::A4 => "a4",
            RiscVRegister::A5 => "a5",
            RiscVRegister::A6 => "a6",
            RiscVRegister::A7 => "a7",
            RiscVRegister::S2 => "s2",
            RiscVRegister::S3 => "s3",
            RiscVRegister::S4 => "s4",
            RiscVRegister::S5 => "s5",
            RiscVRegister::S6 => "s6",
            RiscVRegister::S7 => "s7",
            RiscVRegister::S8 => "s8",
            RiscVRegister::S9 => "s9",
            RiscVRegister::S10 => "s10",
            RiscVRegister::S11 => "s11",
            RiscVRegister::T3 => "t3",
            RiscVRegister::T4 => "t4",
            RiscVRegister::T5 => "t5",
            RiscVRegister::T6 => "t6",
        };
        write!(f, "{}", name)
    }
}

// ============================================================================
// 内存操作数
// ============================================================================

/// 内存操作数
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryOperand {
    /// x86-64 内存操作数: [base + index * scale + displacement]
    X86Memory {
        base: Option<X86Register>,
        index: Option<X86Register>,
        scale: u8,
        displacement: i32,
    },
    /// AArch64 内存操作数: [Xn, #offset]
    AArch64Memory {
        base: AArch64Register,
        offset: i32,
    },
    /// RISC-V 内存操作数: offset(reg)
    RiscVMemory {
        base: RiscVRegister,
        offset: i32,
    },
    /// Wasm 内存操作数
    WasmMemory { offset: u32 },
}

impl MemoryOperand {
    /// 创建 x86-64 内存操作数
    pub fn x86(base: X86Register, displacement: i32) -> Self {
        MemoryOperand::X86Memory {
            base: Some(base),
            index: None,
            scale: 1,
            displacement,
        }
    }

    /// 创建 x86-64 带索引的内存操作数
    pub fn x86_indexed(base: X86Register, index: X86Register, scale: u8, displacement: i32) -> Self {
        MemoryOperand::X86Memory {
            base: Some(base),
            index: Some(index),
            scale,
            displacement,
        }
    }

    /// 创建 AArch64 内存操作数
    pub fn aarch64(base: AArch64Register, offset: i32) -> Self {
        MemoryOperand::AArch64Memory { base, offset }
    }

    /// 创建 RISC-V 内存操作数
    pub fn riscv(base: RiscVRegister, offset: i32) -> Self {
        MemoryOperand::RiscVMemory { base, offset }
    }
}

impl Display for MemoryOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryOperand::X86Memory {
                base,
                index,
                scale,
                displacement,
            } => {
                write!(f, "[")?;
                if let Some(b) = base {
                    write!(f, "{}", b)?;
                }
                if let Some(i) = index {
                    if base.is_some() {
                        write!(f, " + ")?;
                    }
                    write!(f, "{}*{}", i, scale)?;
                }
                if *displacement != 0 {
                    if base.is_some() || index.is_some() {
                        if *displacement > 0 {
                            write!(f, " + {}", displacement)?;
                        } else {
                            write!(f, " - {}", displacement.abs())?;
                        }
                    } else {
                        write!(f, "{}", displacement)?;
                    }
                }
                write!(f, "]")
            }
            MemoryOperand::AArch64Memory { base, offset } => {
                if *offset == 0 {
                    write!(f, "[{}]", base)
                } else {
                    write!(f, "[{}, #{}]", base, offset)
                }
            }
            MemoryOperand::RiscVMemory { base, offset } => {
                write!(f, "{}({})", offset, base)
            }
            MemoryOperand::WasmMemory { offset } => {
                write!(f, "[{}]", offset)
            }
        }
    }
}

// ============================================================================
// 指令 trait
// ============================================================================

/// 机器指令 trait
///
/// 所有架构的指令都要实现这个 trait
pub trait Instruction: Display {
    /// 获取指令所属架构
    fn arch(&self) -> TargetArch;

    /// 获取指令的字节大小
    fn size(&self) -> usize;

    /// 编码为机器码
    fn encode(&self) -> Vec<u8>;
}

// ============================================================================
// x86-64 指令
// ============================================================================

/// x86-64 指令
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X86Instruction {
    // 数据传送
    Mov { dest: Operand, src: Operand },
    Movzx { dest: Operand, src: Operand },
    Movsx { dest: Operand, src: Operand },
    Lea { dest: Operand, src: MemoryOperand },
    Push(Operand),
    Pop(Operand),
    Xchg { op1: Operand, op2: Operand },

    // 算术运算
    Add { dest: Operand, src: Operand },
    Sub { dest: Operand, src: Operand },
    Imul { dest: Operand, src: Operand },
    Imul3 { dest: Operand, src: Operand, imm: i32 },
    Idiv(Operand),
    Inc(Operand),
    Dec(Operand),
    Neg(Operand),

    // 逻辑运算
    And { dest: Operand, src: Operand },
    Or { dest: Operand, src: Operand },
    Xor { dest: Operand, src: Operand },
    Not(Operand),
    Shl { dest: Operand, count: Operand },
    Shr { dest: Operand, count: Operand },
    Sar { dest: Operand, count: Operand },

    // 比较和测试
    Cmp { op1: Operand, op2: Operand },
    Test { op1: Operand, op2: Operand },

    // 条件设置
    Sete(Operand),
    Setne(Operand),
    Setl(Operand),
    Setle(Operand),
    Setg(Operand),
    Setge(Operand),

    // 控制转移
    Jmp(String),
    Je(String),
    Jne(String),
    Jz(String),
    Jnz(String),
    Jl(String),
    Jle(String),
    Jg(String),
    Jge(String),
    Call(String),
    Ret,

    // 栈操作
    Enter { alloc: u16, nesting: u8 },
    Leave,

    // 其他
    Nop,
    Cqo,
}

/// x86-64 操作数
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Register(X86Register),
    Memory(MemoryOperand),
    Immediate(i64),
    Label(String),
}

impl Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(r) => write!(f, "{}", r),
            Operand::Memory(m) => write!(f, "{}", m),
            Operand::Immediate(n) => write!(f, "{}", n),
            Operand::Label(l) => write!(f, "{}", l),
        }
    }
}

impl Display for X86Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            X86Instruction::Mov { dest, src } => write!(f, "mov {}, {}", dest, src),
            X86Instruction::Movzx { dest, src } => write!(f, "movzx {}, {}", dest, src),
            X86Instruction::Movsx { dest, src } => write!(f, "movsx {}, {}", dest, src),
            X86Instruction::Lea { dest, src } => write!(f, "lea {}, {}", dest, src),
            X86Instruction::Push(op) => write!(f, "push {}", op),
            X86Instruction::Pop(op) => write!(f, "pop {}", op),
            X86Instruction::Xchg { op1, op2 } => write!(f, "xchg {}, {}", op1, op2),
            X86Instruction::Add { dest, src } => write!(f, "add {}, {}", dest, src),
            X86Instruction::Sub { dest, src } => write!(f, "sub {}, {}", dest, src),
            X86Instruction::Imul { dest, src } => write!(f, "imul {}, {}", dest, src),
            X86Instruction::Imul3 { dest, src, imm } => write!(f, "imul {}, {}, {}", dest, src, imm),
            X86Instruction::Idiv(op) => write!(f, "idiv {}", op),
            X86Instruction::Inc(op) => write!(f, "inc {}", op),
            X86Instruction::Dec(op) => write!(f, "dec {}", op),
            X86Instruction::Neg(op) => write!(f, "neg {}", op),
            X86Instruction::And { dest, src } => write!(f, "and {}, {}", dest, src),
            X86Instruction::Or { dest, src } => write!(f, "or {}, {}", dest, src),
            X86Instruction::Xor { dest, src } => write!(f, "xor {}, {}", dest, src),
            X86Instruction::Not(op) => write!(f, "not {}", op),
            X86Instruction::Shl { dest, count } => write!(f, "shl {}, {}", dest, count),
            X86Instruction::Shr { dest, count } => write!(f, "shr {}, {}", dest, count),
            X86Instruction::Sar { dest, count } => write!(f, "sar {}, {}", dest, count),
            X86Instruction::Cmp { op1, op2 } => write!(f, "cmp {}, {}", op1, op2),
            X86Instruction::Test { op1, op2 } => write!(f, "test {}, {}", op1, op2),
            X86Instruction::Sete(op) => write!(f, "sete {}", op),
            X86Instruction::Setne(op) => write!(f, "setne {}", op),
            X86Instruction::Setl(op) => write!(f, "setl {}", op),
            X86Instruction::Setle(op) => write!(f, "setle {}", op),
            X86Instruction::Setg(op) => write!(f, "setg {}", op),
            X86Instruction::Setge(op) => write!(f, "setge {}", op),
            X86Instruction::Jmp(label) => write!(f, "jmp {}", label),
            X86Instruction::Je(label) => write!(f, "je {}", label),
            X86Instruction::Jne(label) => write!(f, "jne {}", label),
            X86Instruction::Jz(label) => write!(f, "jz {}", label),
            X86Instruction::Jnz(label) => write!(f, "jnz {}", label),
            X86Instruction::Jl(label) => write!(f, "jl {}", label),
            X86Instruction::Jle(label) => write!(f, "jle {}", label),
            X86Instruction::Jg(label) => write!(f, "jg {}", label),
            X86Instruction::Jge(label) => write!(f, "jge {}", label),
            X86Instruction::Call(label) => write!(f, "call {}", label),
            X86Instruction::Ret => write!(f, "ret"),
            X86Instruction::Enter { alloc, nesting } => write!(f, "enter {}, {}", alloc, nesting),
            X86Instruction::Leave => write!(f, "leave"),
            X86Instruction::Nop => write!(f, "nop"),
            X86Instruction::Cqo => write!(f, "cqo"),
        }
    }
}

impl Instruction for X86Instruction {
    fn arch(&self) -> TargetArch {
        TargetArch::X86_64
    }

    fn size(&self) -> usize {
        // 简化实现，返回指令长度的估计值
        match self {
            X86Instruction::Mov { .. } => 7,       // 平均长度
            X86Instruction::Push(_) | X86Instruction::Pop(_) => 2,
            X86Instruction::Ret => 1,
            X86Instruction::Nop => 1,
            X86Instruction::Call(_) => 5,
            X86Instruction::Jmp(_) => 5,
            _ => 4, // 默认长度
        }
    }

    fn encode(&self) -> Vec<u8> {
        // 简化实现，返回占位符
        // 实际实现需要完整的 x86-64 编码器
        match self {
            X86Instruction::Ret => vec![0xC3],
            X86Instruction::Nop => vec![0x90],
            _ => vec![0x90; self.size()], // NOP 填充
        }
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_arch() {
        assert_eq!(TargetArch::X86_64.name(), "x86_64");
        assert_eq!(TargetArch::AArch64.name(), "aarch64");
        assert_eq!(TargetArch::RiscV64.name(), "riscv64");
        assert_eq!(TargetArch::Wasm32.name(), "wasm32");
    }

    #[test]
    fn test_pointer_size() {
        assert_eq!(TargetArch::X86_64.pointer_size(), 8);
        assert_eq!(TargetArch::AArch64.pointer_size(), 8);
        assert_eq!(TargetArch::RiscV64.pointer_size(), 8);
        assert_eq!(TargetArch::Wasm32.pointer_size(), 4);
    }

    #[test]
    fn test_x86_register_display() {
        assert_eq!(format!("{}", X86Register::Rax), "rax");
        assert_eq!(format!("{}", X86Register::Eax), "eax");
        assert_eq!(format!("{}", X86Register::Ax), "ax");
        assert_eq!(format!("{}", X86Register::Al), "al");
    }

    #[test]
    fn test_x86_callee_saved() {
        assert!(X86Register::Rbx.is_callee_saved());
        assert!(X86Register::Rbp.is_callee_saved());
        assert!(X86Register::R12.is_callee_saved());
        assert!(!X86Register::Rax.is_callee_saved());
        assert!(!X86Register::Rcx.is_callee_saved());
    }

    #[test]
    fn test_x86_argument_sysv() {
        assert!(X86Register::Rdi.is_argument_sysv());
        assert!(X86Register::Rsi.is_argument_sysv());
        assert!(X86Register::R8.is_argument_sysv());
        assert!(!X86Register::Rax.is_argument_sysv());
    }

    #[test]
    fn test_x86_argument_windows() {
        assert!(X86Register::Rcx.is_argument_windows());
        assert!(X86Register::Rdx.is_argument_windows());
        assert!(X86Register::R8.is_argument_windows());
        assert!(!X86Register::Rdi.is_argument_windows());
    }

    #[test]
    fn test_aarch64_register_display() {
        assert_eq!(format!("{}", AArch64Register::X0), "x0");
        assert_eq!(format!("{}", AArch64Register::X29), "x29");
        assert_eq!(format!("{}", AArch64Register::W0), "w0");
    }

    #[test]
    fn test_aarch64_argument() {
        assert!(AArch64Register::X0.is_argument());
        assert!(AArch64Register::X7.is_argument());
        assert!(!AArch64Register::X8.is_argument());
    }

    #[test]
    fn test_riscv_register_display() {
        assert_eq!(format!("{}", RiscVRegister::Zero), "zero");
        assert_eq!(format!("{}", RiscVRegister::Ra), "ra");
        assert_eq!(format!("{}", RiscVRegister::A0), "a0");
    }

    #[test]
    fn test_riscv_register_number() {
        assert_eq!(RiscVRegister::Zero.number(), 0);
        assert_eq!(RiscVRegister::Ra.number(), 1);
        assert_eq!(RiscVRegister::A0.number(), 10);
        assert_eq!(RiscVRegister::T6.number(), 31);
    }

    #[test]
    fn test_memory_operand_display() {
        let mem = MemoryOperand::x86(X86Register::Rbp, -8);
        assert_eq!(format!("{}", mem), "[rbp - 8]");

        let mem = MemoryOperand::x86_indexed(X86Register::Rax, X86Register::Rcx, 4, 0);
        assert_eq!(format!("{}", mem), "[rax + rcx*4]");

        let mem = MemoryOperand::aarch64(AArch64Register::X29, 16);
        assert_eq!(format!("{}", mem), "[x29, #16]");

        let mem = MemoryOperand::riscv(RiscVRegister::S0, -8);
        assert_eq!(format!("{}", mem), "-8(s0)");
    }

    #[test]
    fn test_x86_instruction_display() {
        let mov = X86Instruction::Mov {
            dest: Operand::Register(X86Register::Rax),
            src: Operand::Immediate(42),
        };
        assert_eq!(format!("{}", mov), "mov rax, 42");

        let push = X86Instruction::Push(Operand::Register(X86Register::Rbp));
        assert_eq!(format!("{}", push), "push rbp");

        let ret = X86Instruction::Ret;
        assert_eq!(format!("{}", ret), "ret");
    }

    #[test]
    fn test_x86_instruction_encode() {
        let ret = X86Instruction::Ret;
        assert_eq!(ret.encode(), vec![0xC3]);

        let nop = X86Instruction::Nop;
        assert_eq!(nop.encode(), vec![0x90]);
    }
}
