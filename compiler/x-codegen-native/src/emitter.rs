//! 二进制输出发射器
//!
//! 支持多种二进制输出格式：ELF, Mach-O, PE, Wasm

use std::collections::HashMap;
use std::io::{self, Write};

// ============================================================================
// 二进制格式
// ============================================================================

/// 支持的二进制输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    /// ELF (Linux, BSD)
    Elf,
    /// Mach-O (macOS, iOS)
    MachO,
    /// PE/COFF (Windows)
    PE,
    /// WebAssembly
    Wasm,
    /// 原始二进制
    Raw,
}

impl BinaryFormat {
    /// 获取格式的魔数
    pub fn magic(&self) -> &'static [u8] {
        match self {
            BinaryFormat::Elf => &[0x7f, b'E', b'L', b'F'],
            BinaryFormat::MachO => &[0xfe, 0xed, 0xfa, 0xce],
            BinaryFormat::PE => &[b'M', b'Z'],
            BinaryFormat::Wasm => &[0x00, 0x61, 0x73, 0x6d],
            BinaryFormat::Raw => &[],
        }
    }

    /// 获取默认文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            BinaryFormat::Elf => "o",
            BinaryFormat::MachO => "o",
            BinaryFormat::PE => "obj",
            BinaryFormat::Wasm => "wasm",
            BinaryFormat::Raw => "bin",
        }
    }
}

// ============================================================================
// 二进制发射器
// ============================================================================

/// 二进制文件发射器
///
/// 用于构建和输出可执行文件或目标文件
pub struct BinaryEmitter {
    /// 二进制格式
    format: BinaryFormat,
    /// 代码段
    text_section: Vec<u8>,
    /// 数据段
    data_section: Vec<u8>,
    /// 只读数据段
    rodata_section: Vec<u8>,
    /// BSS 段（未初始化数据）
    bss_size: usize,
    /// 符号表
    symbols: HashMap<String, SymbolInfo>,
    /// 重定位表
    relocations: Vec<Relocation>,
    /// 当前段偏移
    current_offset: usize,
}

/// 符号信息
#[derive(Debug, Clone)]
struct SymbolInfo {
    /// 符号名称
    name: String,
    /// 符号类型
    sym_type: SymbolType,
    /// 绑定类型
    binding: SymbolBinding,
    /// 所在段
    section: Section,
    /// 偏移量
    offset: usize,
    /// 大小
    size: usize,
}

/// 符号类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolType {
    /// 未定义
    Notype,
    /// 函数
    Func,
    /// 对象（变量）
    Object,
    /// 段
    Section,
    /// 文件
    File,
}

/// 符号绑定类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolBinding {
    /// 局部符号
    Local,
    /// 全局符号
    Global,
    /// 弱符号
    Weak,
}

/// 段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// 代码段
    Text,
    /// 数据段
    Data,
    /// 只读数据段
    Rodata,
    /// BSS 段
    Bss,
}

/// 重定位条目
#[derive(Debug, Clone)]
struct Relocation {
    /// 重定位偏移
    offset: usize,
    /// 重定位类型
    rel_type: RelocationType,
    /// 关联符号
    symbol: String,
    /// 添加值
    addend: i64,
}

/// 重定位类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelocationType {
    /// 64 位绝对地址
    Absolute64,
    /// 32 位相对地址
    Relative32,
    /// PLT 跳转
    Plt32,
    /// GOT 引用
    Got64,
}

impl BinaryEmitter {
    /// 创建新的二进制发射器
    pub fn new(format: BinaryFormat) -> Self {
        Self {
            format,
            text_section: Vec::new(),
            data_section: Vec::new(),
            rodata_section: Vec::new(),
            bss_size: 0,
            symbols: HashMap::new(),
            relocations: Vec::new(),
            current_offset: 0,
        }
    }

    /// 获取二进制格式
    pub fn format(&self) -> BinaryFormat {
        self.format
    }

    /// 向代码段写入字节
    pub fn emit_code(&mut self, bytes: &[u8]) {
        self.text_section.extend_from_slice(bytes);
        self.current_offset = self.text_section.len();
    }

    /// 写入单条指令
    pub fn emit_instruction(&mut self, opcode: u8, operands: &[u8]) {
        self.text_section.push(opcode);
        self.text_section.extend_from_slice(operands);
        self.current_offset = self.text_section.len();
    }

    /// 向数据段写入字节
    pub fn emit_data(&mut self, bytes: &[u8]) {
        self.data_section.extend_from_slice(bytes);
    }

    /// 向只读数据段写入字节
    pub fn emit_rodata(&mut self, bytes: &[u8]) {
        self.rodata_section.extend_from_slice(bytes);
    }

    /// 添加全局符号
    pub fn add_global_symbol(&mut self, name: &str, section: Section, offset: usize, size: usize) {
        self.symbols.insert(
            name.to_string(),
            SymbolInfo {
                name: name.to_string(),
                sym_type: SymbolType::Func,
                binding: SymbolBinding::Global,
                section,
                offset,
                size,
            },
        );
    }

    /// 添加局部符号
    pub fn add_local_symbol(&mut self, name: &str, section: Section, offset: usize, size: usize) {
        self.symbols.insert(
            name.to_string(),
            SymbolInfo {
                name: name.to_string(),
                sym_type: SymbolType::Object,
                binding: SymbolBinding::Local,
                section,
                offset,
                size,
            },
        );
    }

    /// 添加外部符号引用
    pub fn add_external_symbol(&mut self, name: &str) {
        self.symbols.insert(
            name.to_string(),
            SymbolInfo {
                name: name.to_string(),
                sym_type: SymbolType::Notype,
                binding: SymbolBinding::Global,
                section: Section::Text, // 未定义符号
                offset: 0,
                size: 0,
            },
        );
    }

    /// 添加重定位条目
    pub fn add_relocation(&mut self, offset: usize, rel_type: RelocationType, symbol: &str, addend: i64) {
        self.relocations.push(Relocation {
            offset,
            rel_type,
            symbol: symbol.to_string(),
            addend,
        });
    }

    /// 分配 BSS 空间
    pub fn allocate_bss(&mut self, size: usize) {
        self.bss_size += size;
    }

    /// 获取当前代码段偏移
    pub fn current_offset(&self) -> usize {
        self.current_offset
    }

    /// 获取代码段大小
    pub fn text_size(&self) -> usize {
        self.text_section.len()
    }

    /// 获取数据段大小
    pub fn data_size(&self) -> usize {
        self.data_section.len()
    }

    /// 生成完整的二进制输出
    pub fn emit(&self) -> io::Result<Vec<u8>> {
        match self.format {
            BinaryFormat::Elf => self.emit_elf(),
            BinaryFormat::MachO => self.emit_macho(),
            BinaryFormat::PE => self.emit_pe(),
            BinaryFormat::Wasm => self.emit_wasm(),
            BinaryFormat::Raw => self.emit_raw(),
        }
    }

    /// 生成 ELF 文件
    fn emit_elf(&self) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();

        // ELF 头部 (64-bit)
        output.write_all(&[0x7f, b'E', b'L', b'F'])?; // 魔数
        output.write_all(&[2])?; // 64-bit
        output.write_all(&[1])?; // 小端序
        output.write_all(&[1])?; // ELF 版本
        output.write_all(&[0; 9])?; // 填充

        // e_type: ET_EXEC = 2
        output.write_all(&2u16.to_le_bytes())?;
        // e_machine: EM_X86_64 = 62
        output.write_all(&62u16.to_le_bytes())?;
        // e_version
        output.write_all(&1u32.to_le_bytes())?;
        // e_entry: 入口点地址
        output.write_all(&0x400000u64.to_le_bytes())?;
        // e_phoff: 程序头偏移
        output.write_all(&64u64.to_le_bytes())?;
        // e_shoff: 段头偏移（后面填充）
        let shoff_offset = output.len();
        output.write_all(&0u64.to_le_bytes())?;
        // e_flags
        output.write_all(&0u32.to_le_bytes())?;
        // e_ehsize: ELF 头大小
        output.write_all(&64u16.to_le_bytes())?;
        // e_phentsize: 程序头条目大小
        output.write_all(&56u16.to_le_bytes())?;
        // e_phnum: 程序头数量
        output.write_all(&2u16.to_le_bytes())?;
        // e_shentsize: 段头条目大小
        output.write_all(&64u16.to_le_bytes())?;
        // e_shnum: 段头数量
        output.write_all(&4u16.to_le_bytes())?;
        // e_shstrndx: 段名字符串表索引
        output.write_all(&3u16.to_le_bytes())?;

        // 程序头 - 代码段
        // p_type: PT_LOAD = 1
        output.write_all(&1u32.to_le_bytes())?;
        // p_flags: PF_R | PF_X = 5
        output.write_all(&5u32.to_le_bytes())?;
        // p_offset
        output.write_all(&0u64.to_le_bytes())?;
        // p_vaddr
        output.write_all(&0x400000u64.to_le_bytes())?;
        // p_paddr
        output.write_all(&0x400000u64.to_le_bytes())?;
        // p_filesz
        output.write_all(&(self.text_section.len() as u64).to_le_bytes())?;
        // p_memsz
        output.write_all(&(self.text_section.len() as u64).to_le_bytes())?;
        // p_align
        output.write_all(&0x1000u64.to_le_bytes())?;

        // 程序头 - 数据段
        output.write_all(&1u32.to_le_bytes())?; // p_type
        output.write_all(&6u32.to_le_bytes())?; // p_flags: PF_R | PF_W
        output.write_all(&0u64.to_le_bytes())?; // p_offset（待修正）
        output.write_all(&0x600000u64.to_le_bytes())?; // p_vaddr
        output.write_all(&0x600000u64.to_le_bytes())?; // p_paddr
        output.write_all(&(self.data_section.len() as u64).to_le_bytes())?;
        output.write_all(&((self.data_section.len() + self.bss_size) as u64).to_le_bytes())?;
        output.write_all(&0x1000u64.to_le_bytes())?;

        // 代码段内容
        output.write_all(&self.text_section)?;

        // 数据段内容
        output.write_all(&self.data_section)?;

        Ok(output)
    }

    /// 生成 Mach-O 文件
    fn emit_macho(&self) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();

        // Mach-O 头部 (64-bit)
        output.write_all(&0xfeedfacfu32.to_le_bytes())?; // 魔数
        output.write_all(&0x01000007u32.to_le_bytes())?; // cputype: CPU_TYPE_X86_64
        output.write_all(&3u32.to_le_bytes())?; // cpusubtype: CPU_SUBTYPE_X86_64_ALL
        output.write_all(&2u32.to_le_bytes())?; // filetype: MH_EXECUTE
        output.write_all(&0u32.to_le_bytes())?; // ncmds（待填充）
        output.write_all(&0u32.to_le_bytes())?; // sizeofcmds（待填充）
        output.write_all(&0x0085u32.to_le_bytes())?; // flags
        output.write_all(&0u32.to_le_bytes())?; // reserved

        // LC_SEGMENT_64 命令
        // ...（简化实现）

        // 代码段
        output.write_all(&self.text_section)?;

        // 数据段
        output.write_all(&self.data_section)?;

        Ok(output)
    }

    /// 生成 PE/COFF 文件
    fn emit_pe(&self) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();

        // DOS 头部（简化）
        output.write_all(b"MZ")?; // DOS 签名
        output.write_all(&[0u8; 58])?; // DOS 头部其余部分
        output.write_all(&0x40u32.to_le_bytes())?; // e_lfanew: PE 头偏移

        // PE 签名
        output.write_all(b"PE\x00\x00")?;

        // COFF 文件头
        output.write_all(&0x8664u16.to_le_bytes())?; // Machine: AMD64
        output.write_all(&2u16.to_le_bytes())?; // NumberOfSections
        output.write_all(&0u32.to_le_bytes())?; // TimeDateStamp
        output.write_all(&0u32.to_le_bytes())?; // PointerToSymbolTable
        output.write_all(&0u32.to_le_bytes())?; // NumberOfSymbols
        output.write_all(&0u16.to_le_bytes())?; // SizeOfOptionalHeader
        output.write_all(&0x0022u16.to_le_bytes())?; // Characteristics

        // 段表（简化）
        // .text 段
        output.write_all(b".text\x00\x00\x00")?;
        output.write_all(&(self.text_section.len() as u32).to_le_bytes())?;
        output.write_all(&0u32.to_le_bytes())?; // VirtualSize
        output.write_all(&0u32.to_le_bytes())?; // VirtualAddress
        output.write_all(&(self.text_section.len() as u32).to_le_bytes())?; // SizeOfRawData
        output.write_all(&0u32.to_le_bytes())?; // PointerToRawData
        // ...

        // 代码段内容
        output.write_all(&self.text_section)?;

        // 数据段内容
        output.write_all(&self.data_section)?;

        Ok(output)
    }

    /// 生成 WebAssembly 模块
    fn emit_wasm(&self) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();

        // Wasm 魔数
        output.write_all(&[0x00, 0x61, 0x73, 0x6d])?;

        // 版本号
        output.write_all(&[0x01, 0x00, 0x00, 0x00])?;

        // 类型段（简化）
        output.write_all(&[0x01])?; // Section ID: Type
        output.write_all(&[0x05])?; // Section size
        output.write_all(&[0x01])?; // Number of types
        output.write_all(&[0x60])?; // Function type
        output.write_all(&[0x00])?; // No params
        output.write_all(&[0x01, 0x7f])?; // Result: i32

        // 函数段
        output.write_all(&[0x03])?; // Section ID: Function
        output.write_all(&[0x02])?; // Section size
        output.write_all(&[0x01])?; // Number of functions
        output.write_all(&[0x00])?; // Type index

        // 代码段
        output.write_all(&[0x0a])?; // Section ID: Code
        let code_size = self.text_section.len() + 2;
        output.write_all(&[code_size as u8])?; // Section size
        output.write_all(&[0x01])?; // Number of functions
        output.write_all(&[self.text_section.len() as u8])?; // Function body size
        output.write_all(&self.text_section)?;

        Ok(output)
    }

    /// 生成原始二进制
    fn emit_raw(&self) -> io::Result<Vec<u8>> {
        let mut output = Vec::new();
        output.extend_from_slice(&self.text_section);
        output.extend_from_slice(&self.data_section);
        Ok(output)
    }
}

// ============================================================================
// 辅助结构
// ============================================================================

/// ELF 段头构建器
pub struct SectionHeaderBuilder {
    pub name: String,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

impl SectionHeaderBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            sh_type: 0,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: 0,
            sh_size: 0,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        }
    }

    pub fn sh_type(mut self, t: u32) -> Self {
        self.sh_type = t;
        self
    }

    pub fn sh_flags(mut self, f: u64) -> Self {
        self.sh_flags = f;
        self
    }

    pub fn sh_addr(mut self, a: u64) -> Self {
        self.sh_addr = a;
        self
    }

    pub fn sh_offset(mut self, o: u64) -> Self {
        self.sh_offset = o;
        self
    }

    pub fn sh_size(mut self, s: u64) -> Self {
        self.sh_size = s;
        self
    }

    pub fn build(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0u32.to_le_bytes()); // sh_name (index into string table)
        buf.extend_from_slice(&self.sh_type.to_le_bytes());
        buf.extend_from_slice(&self.sh_flags.to_le_bytes());
        buf.extend_from_slice(&self.sh_addr.to_le_bytes());
        buf.extend_from_slice(&self.sh_offset.to_le_bytes());
        buf.extend_from_slice(&self.sh_size.to_le_bytes());
        buf.extend_from_slice(&self.sh_link.to_le_bytes());
        buf.extend_from_slice(&self.sh_info.to_le_bytes());
        buf.extend_from_slice(&self.sh_addralign.to_le_bytes());
        buf.extend_from_slice(&self.sh_entsize.to_le_bytes());
        buf
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_format_magic() {
        assert_eq!(BinaryFormat::Elf.magic(), &[0x7f, b'E', b'L', b'F']);
        assert_eq!(BinaryFormat::Wasm.magic(), &[0x00, 0x61, 0x73, 0x6d]);
        assert_eq!(BinaryFormat::Raw.magic(), &[]);
    }

    #[test]
    fn test_binary_format_extension() {
        assert_eq!(BinaryFormat::Elf.extension(), "o");
        assert_eq!(BinaryFormat::MachO.extension(), "o");
        assert_eq!(BinaryFormat::PE.extension(), "obj");
        assert_eq!(BinaryFormat::Wasm.extension(), "wasm");
        assert_eq!(BinaryFormat::Raw.extension(), "bin");
    }

    #[test]
    fn test_binary_emitter_creation() {
        let emitter = BinaryEmitter::new(BinaryFormat::Elf);
        assert_eq!(emitter.format(), BinaryFormat::Elf);
        assert_eq!(emitter.text_size(), 0);
        assert_eq!(emitter.data_size(), 0);
    }

    #[test]
    fn test_emit_code() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Raw);
        emitter.emit_code(&[0x48, 0x89, 0xe5]); // mov rbp, rsp
        assert_eq!(emitter.text_size(), 3);
        assert_eq!(emitter.current_offset(), 3);
    }

    #[test]
    fn test_emit_instruction() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Raw);
        emitter.emit_instruction(0xc3, &[]); // ret
        assert_eq!(emitter.text_size(), 1);
    }

    #[test]
    fn test_emit_data() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Raw);
        emitter.emit_data(&[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(emitter.data_size(), 4);
    }

    #[test]
    fn test_add_global_symbol() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Elf);
        emitter.add_global_symbol("main", Section::Text, 0, 100);
        assert!(emitter.symbols.contains_key("main"));
    }

    #[test]
    fn test_add_local_symbol() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Elf);
        emitter.add_local_symbol("local_var", Section::Data, 0, 8);
        assert!(emitter.symbols.contains_key("local_var"));
    }

    #[test]
    fn test_add_relocation() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Elf);
        emitter.add_relocation(0x10, RelocationType::Absolute64, "printf", 0);
        assert_eq!(emitter.relocations.len(), 1);
    }

    #[test]
    fn test_allocate_bss() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Elf);
        emitter.allocate_bss(1024);
        assert_eq!(emitter.bss_size, 1024);
    }

    #[test]
    fn test_emit_raw() {
        let mut emitter = BinaryEmitter::new(BinaryFormat::Raw);
        emitter.emit_code(&[0x48, 0x31, 0xc0]); // xor rax, rax
        emitter.emit_data(&[0x42]); // data

        let output = emitter.emit().unwrap();
        assert_eq!(output.len(), 4);
        assert_eq!(&output[..3], &[0x48, 0x31, 0xc0]);
    }

    #[test]
    fn test_emit_elf() {
        let emitter = BinaryEmitter::new(BinaryFormat::Elf);
        let output = emitter.emit().unwrap();

        // 验证 ELF 魔数
        assert_eq!(&output[0..4], &[0x7f, b'E', b'L', b'F']);
    }

    #[test]
    fn test_emit_wasm() {
        let emitter = BinaryEmitter::new(BinaryFormat::Wasm);
        let output = emitter.emit().unwrap();

        // 验证 Wasm 魔数
        assert_eq!(&output[0..4], &[0x00, 0x61, 0x73, 0x6d]);
        // 验证版本号
        assert_eq!(&output[4..8], &[0x01, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_section_header_builder() {
        let sh = SectionHeaderBuilder::new(".text")
            .sh_type(1) // SHT_PROGBITS
            .sh_flags(6) // SHF_ALLOC | SHF_EXECINSTR
            .sh_size(100)
            .build();

        assert_eq!(sh.len(), 64); // ELF64 section header size
    }
}
