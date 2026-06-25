//! 直出机器码层
//!
//! 仿照 Zig 自举后端：把 LIR 直接降级为机器码字节，边发射边登记
//! 标签修补（函数内跳转/调用）与重定位（外部符号、字符串/数据引用），
//! 最终交给 emitter 写出可重定位 ELF 目标文件，再由系统链接器链接。
//!
//! 当前实现目标：x86_64 Linux (System V AMD64 ABI)。

pub mod x86_64;

pub mod aarch64;
pub mod riscv64;

pub use x86_64::MachineCodeGen;

use crate::arch::TargetArch;

/// 目标 section 类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecKind {
    Text,
    Rodata,
    Data,
    Bss,
}

/// 重定位类型（架构中立的语义；由各 emitter 映射到具体 r_type）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelKind {
    // ---- x86_64 ----
    /// R_X86_64_PC32 = 2
    Pc32,
    /// R_X86_64_PLT32 = 4
    Plt32,
    /// R_X86_64_64 = 1 / 其它架构的 64 位绝对
    Abs64,
    // ---- aarch64 ----
    /// R_AARCH64_CALL26 = 283
    Aarch64Call26,
    /// R_AARCH64_ADR_PREL_PG_HI21 = 275
    Aarch64AdrPrelPgHi21,
    /// R_AARCH64_ADD_ABS_LO12_NC = 277
    Aarch64AddAbsLo12Nc,
    // ---- riscv64 ----
    /// R_RISCV_CALL_PLT = 19
    RiscvCallPlt,
    /// R_RISCV_PCREL_HI20 = 23
    RiscvPcRelHi20,
    /// R_RISCV_PCREL_LO12_I = 25
    RiscvPcRelLo12I,
    /// R_RISCV_RELAX = 51（伴随放松，作用于同一偏移）
    RiscvRelax,
    /// R_RISCV_HI20 = 26（绝对寻址 lui 高 20 位）
    RiscvHi20,
    /// R_RISCV_LO12_I = 27（绝对寻址 I 型指令低 12 位）
    RiscvLo12I,
}

impl RelKind {
    /// 映射到指定架构的 ELF 重定位类型号
    pub fn elf_type(self, arch: TargetArch) -> u32 {
        match (arch, self) {
            // x86_64
            (TargetArch::X86_64, RelKind::Abs64) => 1,
            (TargetArch::X86_64, RelKind::Pc32) => 2,
            (TargetArch::X86_64, RelKind::Plt32) => 4,
            // aarch64
            (TargetArch::AArch64, RelKind::Abs64) => 257, // R_AARCH64_ABS64
            (TargetArch::AArch64, RelKind::Aarch64Call26) => 283,
            (TargetArch::AArch64, RelKind::Aarch64AdrPrelPgHi21) => 275,
            (TargetArch::AArch64, RelKind::Aarch64AddAbsLo12Nc) => 277,
            // riscv64
            (TargetArch::RiscV64, RelKind::Abs64) => 2, // R_RISCV_64
            (TargetArch::RiscV64, RelKind::RiscvCallPlt) => 19,
            (TargetArch::RiscV64, RelKind::RiscvPcRelHi20) => 23,
            (TargetArch::RiscV64, RelKind::RiscvPcRelLo12I) => 25,
            (TargetArch::RiscV64, RelKind::RiscvRelax) => 51,
            (TargetArch::RiscV64, RelKind::RiscvHi20) => 26,
            (TargetArch::RiscV64, RelKind::RiscvLo12I) => 27,
            // 未知组合按 0（NONE），不应发生
            _ => 0,
        }
    }
}

/// 重定位目标：要么是某个 section（用 section 符号），要么是具名符号
#[derive(Debug, Clone)]
pub enum RelTarget {
    Section(SecKind),
    Symbol(String),
}

/// 一条重定位记录（作用于 .text）
#[derive(Debug, Clone)]
pub struct ObjReloc {
    /// 在 .text 内的字节偏移（被修补的 32 位字段位置）
    pub offset: u64,
    pub target: RelTarget,
    pub kind: RelKind,
    pub addend: i64,
}

/// 一个符号
#[derive(Debug, Clone)]
pub struct ObjSymbol {
    pub name: String,
    /// None 表示未定义（外部）符号
    pub section: Option<SecKind>,
    /// 在所属 section 内的偏移
    pub value: u64,
    pub size: u64,
    pub is_func: bool,
    pub is_global: bool,
}

/// 直出机器码的完整产物
#[derive(Debug, Clone, Default)]
pub struct MachineObject {
    pub text: Vec<u8>,
    pub rodata: Vec<u8>,
    pub data: Vec<u8>,
    pub bss_size: u64,
    pub symbols: Vec<ObjSymbol>,
    pub relocations: Vec<ObjReloc>,
}
