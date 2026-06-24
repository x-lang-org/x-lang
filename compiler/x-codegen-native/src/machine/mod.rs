//! 直出机器码层
//!
//! 仿照 Zig 自举后端：把 LIR 直接降级为机器码字节，边发射边登记
//! 标签修补（函数内跳转/调用）与重定位（外部符号、字符串/数据引用），
//! 最终交给 emitter 写出可重定位 ELF 目标文件，再由系统链接器链接。
//!
//! 当前实现目标：x86_64 Linux (System V AMD64 ABI)。

pub mod x86_64;

pub use x86_64::MachineCodeGen;

/// 目标 section 类别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecKind {
    Text,
    Rodata,
    Data,
    Bss,
}

/// 重定位类型（x86_64 ELF）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelKind {
    /// R_X86_64_PC32 = 2
    Pc32,
    /// R_X86_64_PLT32 = 4
    Plt32,
    /// R_X86_64_64 = 1
    Abs64,
}

impl RelKind {
    pub fn elf_type(self) -> u32 {
        match self {
            RelKind::Abs64 => 1,
            RelKind::Pc32 => 2,
            RelKind::Plt32 => 4,
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
