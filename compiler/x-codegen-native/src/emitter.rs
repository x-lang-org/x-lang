//! 可重定位 ELF64 目标文件写出器
//!
//! 把 [`crate::machine::MachineObject`]（机器码字节 + 符号 + 重定位）写成
//! x86_64 的 `ET_REL` 目标文件，供系统链接器（cc/ld）链接为可执行文件。

use std::collections::HashMap;

use crate::machine::{MachineObject, ObjSymbol, RelTarget, SecKind};
use crate::NativeResult;

// ELF 常量
const ELFCLASS64: u8 = 2;
const ELFDATA2LSB: u8 = 1;
const EV_CURRENT: u8 = 1;
const ET_REL: u16 = 1;
const EM_X86_64: u16 = 62;

// section 头类型
const SHT_PROGBITS: u32 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_NOBITS: u32 = 8;

// section 标志
const SHF_WRITE: u64 = 0x1;
const SHF_ALLOC: u64 = 0x2;
const SHF_EXECINSTR: u64 = 0x4;

// 符号绑定/类型
const STB_LOCAL: u8 = 0;
const STB_GLOBAL: u8 = 1;
const STT_NOTYPE: u8 = 0;
const STT_FUNC: u8 = 2;
const STT_SECTION: u8 = 3;

const SHN_UNDEF: u16 = 0;

/// 字符串表构建器
#[derive(Default)]
struct StrTab {
    buf: Vec<u8>,
    map: HashMap<String, u32>,
}

impl StrTab {
    fn new() -> Self {
        // 索引 0 固定为空串
        StrTab {
            buf: vec![0],
            map: HashMap::new(),
        }
    }

    fn add(&mut self, s: &str) -> u32 {
        if s.is_empty() {
            return 0;
        }
        if let Some(&off) = self.map.get(s) {
            return off;
        }
        let off = self.buf.len() as u32;
        self.buf.extend_from_slice(s.as_bytes());
        self.buf.push(0);
        self.map.insert(s.to_string(), off);
        off
    }
}

/// section 索引（固定布局）
mod sec {
    pub const TEXT: usize = 1;
    pub const RODATA: usize = 2;
    pub const DATA: usize = 3;
    pub const BSS: usize = 4;
    pub const SYMTAB: usize = 5;
    pub const STRTAB: usize = 6;
    pub const SHSTRTAB: usize = 7;
    // 8 = .rela.text, 9 = .note.GNU-stack
    pub const COUNT: usize = 10;
}

fn sec_to_index(s: SecKind) -> usize {
    match s {
        SecKind::Text => sec::TEXT,
        SecKind::Rodata => sec::RODATA,
        SecKind::Data => sec::DATA,
        SecKind::Bss => sec::BSS,
    }
}

struct Sym {
    name_off: u32,
    info: u8,
    shndx: u16,
    value: u64,
    size: u64,
}

/// 写出可重定位 ELF64 目标文件
pub fn write_relocatable_elf(obj: &MachineObject) -> NativeResult<Vec<u8>> {
    let mut strtab = StrTab::new();

    // ---- 构建符号表 ----
    // 顺序：NULL，section 符号（local），具名 local（无），具名 global。
    let mut syms: Vec<Sym> = Vec::new();
    // 名称 -> 符号下标（供重定位解析）
    let mut sym_index: HashMap<String, usize> = HashMap::new();
    // section -> 该 section 符号下标
    let mut section_sym: HashMap<usize, usize> = HashMap::new();

    // 0: NULL
    syms.push(Sym {
        name_off: 0,
        info: 0,
        shndx: 0,
        value: 0,
        size: 0,
    });

    // section 符号（STT_SECTION, LOCAL）
    for sk in [SecKind::Text, SecKind::Rodata, SecKind::Data, SecKind::Bss] {
        let idx = sec_to_index(sk);
        section_sym.insert(idx, syms.len());
        syms.push(Sym {
            name_off: 0,
            info: (STB_LOCAL << 4) | STT_SECTION,
            shndx: idx as u16,
            value: 0,
            size: 0,
        });
    }

    let first_global = syms.len();

    // 具名 global 符号（已定义函数 + 外部未定义）
    for s in &obj.symbols {
        let ObjSymbol {
            name,
            section,
            value,
            size,
            is_func,
            is_global,
        } = s;
        let binding = if *is_global { STB_GLOBAL } else { STB_LOCAL };
        let typ = if *is_func { STT_FUNC } else { STT_NOTYPE };
        let shndx = match section {
            Some(sk) => sec_to_index(*sk) as u16,
            None => SHN_UNDEF,
        };
        let name_off = strtab.add(name);
        sym_index.insert(name.clone(), syms.len());
        syms.push(Sym {
            name_off,
            info: (binding << 4) | typ,
            shndx,
            value: *value,
            size: *size,
        });
    }

    // ---- 构建 .rela.text ----
    // Elf64_Rela: r_offset(8) r_info(8) r_addend(8)
    let mut rela: Vec<u8> = Vec::new();
    for r in &obj.relocations {
        let sym_idx = match &r.target {
            RelTarget::Section(sk) => *section_sym.get(&sec_to_index(*sk)).unwrap(),
            RelTarget::Symbol(name) => *sym_index.get(name).ok_or_else(|| {
                crate::NativeError::CodegenError(format!("重定位引用未知符号: {}", name))
            })?,
        };
        let r_info = ((sym_idx as u64) << 32) | (r.kind.elf_type() as u64);
        rela.extend_from_slice(&r.offset.to_le_bytes());
        rela.extend_from_slice(&r_info.to_le_bytes());
        rela.extend_from_slice(&(r.addend).to_le_bytes());
    }

    // ---- 序列化 symtab ----
    // Elf64_Sym: name(4) info(1) other(1) shndx(2) value(8) size(8) = 24 字节
    let mut symtab: Vec<u8> = Vec::new();
    for s in &syms {
        symtab.extend_from_slice(&s.name_off.to_le_bytes());
        symtab.push(s.info);
        symtab.push(0); // st_other
        symtab.extend_from_slice(&s.shndx.to_le_bytes());
        symtab.extend_from_slice(&s.value.to_le_bytes());
        symtab.extend_from_slice(&s.size.to_le_bytes());
    }

    // ---- section 名称表 ----
    let mut shstrtab = StrTab::new();
    let name_text = shstrtab.add(".text");
    let name_rodata = shstrtab.add(".rodata");
    let name_data = shstrtab.add(".data");
    let name_bss = shstrtab.add(".bss");
    let name_symtab = shstrtab.add(".symtab");
    let name_strtab = shstrtab.add(".strtab");
    let name_shstrtab = shstrtab.add(".shstrtab");
    let name_rela_text = shstrtab.add(".rela.text");
    let name_note_gnu_stack = shstrtab.add(".note.GNU-stack");

    // ---- 计算各 section 在文件中的偏移 ----
    let ehsize: u64 = 64;
    let shentsize: u64 = 64;
    let mut off = ehsize;

    let text_off = off;
    off += obj.text.len() as u64;
    off = align_up(off, 16);

    let rodata_off = off;
    off += obj.rodata.len() as u64;
    off = align_up(off, 16);

    let data_off = off;
    off += obj.data.len() as u64;
    off = align_up(off, 8);

    // .bss 不占文件空间
    let bss_off = off;

    let symtab_off = off;
    off += symtab.len() as u64;
    off = align_up(off, 8);

    let strtab_off = off;
    off += strtab.buf.len() as u64;
    off = align_up(off, 8);

    let shstrtab_off = off;
    off += shstrtab.buf.len() as u64;
    off = align_up(off, 8);

    let rela_off = off;
    off += rela.len() as u64;
    off = align_up(off, 8);

    let shoff = off;

    // ---- 写文件 ----
    let mut out: Vec<u8> = Vec::new();

    // ELF 头
    out.extend_from_slice(&[0x7f, b'E', b'L', b'F']);
    out.push(ELFCLASS64);
    out.push(ELFDATA2LSB);
    out.push(EV_CURRENT);
    out.extend_from_slice(&[0u8; 9]); // EI_OSABI..EI_PAD
    out.extend_from_slice(&ET_REL.to_le_bytes());
    out.extend_from_slice(&EM_X86_64.to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes()); // e_version
    out.extend_from_slice(&0u64.to_le_bytes()); // e_entry
    out.extend_from_slice(&0u64.to_le_bytes()); // e_phoff
    out.extend_from_slice(&shoff.to_le_bytes()); // e_shoff
    out.extend_from_slice(&0u32.to_le_bytes()); // e_flags
    out.extend_from_slice(&(ehsize as u16).to_le_bytes()); // e_ehsize
    out.extend_from_slice(&0u16.to_le_bytes()); // e_phentsize
    out.extend_from_slice(&0u16.to_le_bytes()); // e_phnum
    out.extend_from_slice(&(shentsize as u16).to_le_bytes()); // e_shentsize
    out.extend_from_slice(&(sec::COUNT as u16).to_le_bytes()); // e_shnum
    out.extend_from_slice(&(sec::SHSTRTAB as u16).to_le_bytes()); // e_shstrndx

    // section 内容
    pad_to(&mut out, text_off);
    out.extend_from_slice(&obj.text);
    pad_to(&mut out, rodata_off);
    out.extend_from_slice(&obj.rodata);
    pad_to(&mut out, data_off);
    out.extend_from_slice(&obj.data);
    pad_to(&mut out, symtab_off);
    out.extend_from_slice(&symtab);
    pad_to(&mut out, strtab_off);
    out.extend_from_slice(&strtab.buf);
    pad_to(&mut out, shstrtab_off);
    out.extend_from_slice(&shstrtab.buf);
    pad_to(&mut out, rela_off);
    out.extend_from_slice(&rela);

    // section 头表
    pad_to(&mut out, shoff);

    // 0: NULL
    write_shdr(&mut out, &ShdrParams::default());
    // 1: .text
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_text,
            typ: SHT_PROGBITS,
            flags: SHF_ALLOC | SHF_EXECINSTR,
            offset: text_off,
            size: obj.text.len() as u64,
            addralign: 16,
            ..Default::default()
        },
    );
    // 2: .rodata
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_rodata,
            typ: SHT_PROGBITS,
            flags: SHF_ALLOC,
            offset: rodata_off,
            size: obj.rodata.len() as u64,
            addralign: 16,
            ..Default::default()
        },
    );
    // 3: .data
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_data,
            typ: SHT_PROGBITS,
            flags: SHF_ALLOC | SHF_WRITE,
            offset: data_off,
            size: obj.data.len() as u64,
            addralign: 8,
            ..Default::default()
        },
    );
    // 4: .bss
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_bss,
            typ: SHT_NOBITS,
            flags: SHF_ALLOC | SHF_WRITE,
            offset: bss_off,
            size: obj.bss_size,
            addralign: 16,
            ..Default::default()
        },
    );
    // 5: .symtab
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_symtab,
            typ: SHT_SYMTAB,
            flags: 0,
            addr: 0,
            offset: symtab_off,
            size: symtab.len() as u64,
            link: sec::STRTAB as u32,
            info: first_global as u32,
            addralign: 8,
            entsize: 24,
        },
    );
    // 6: .strtab
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_strtab,
            typ: SHT_STRTAB,
            offset: strtab_off,
            size: strtab.buf.len() as u64,
            addralign: 1,
            ..Default::default()
        },
    );
    // 7: .shstrtab
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_shstrtab,
            typ: SHT_STRTAB,
            offset: shstrtab_off,
            size: shstrtab.buf.len() as u64,
            addralign: 1,
            ..Default::default()
        },
    );
    // 8: .rela.text
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_rela_text,
            typ: SHT_RELA,
            flags: 0,
            addr: 0,
            offset: rela_off,
            size: rela.len() as u64,
            link: sec::SYMTAB as u32,
            info: sec::TEXT as u32,
            addralign: 8,
            entsize: 24,
        },
    );
    // 9: .note.GNU-stack（空，标记栈不可执行）
    write_shdr(
        &mut out,
        &ShdrParams {
            name: name_note_gnu_stack,
            typ: SHT_PROGBITS,
            offset: rela_off,
            size: 0,
            addralign: 1,
            ..Default::default()
        },
    );

    Ok(out)
}

#[derive(Default)]
struct ShdrParams {
    name: u32,
    typ: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entsize: u64,
}

fn write_shdr(out: &mut Vec<u8>, p: &ShdrParams) {
    out.extend_from_slice(&p.name.to_le_bytes());
    out.extend_from_slice(&p.typ.to_le_bytes());
    out.extend_from_slice(&p.flags.to_le_bytes());
    out.extend_from_slice(&p.addr.to_le_bytes());
    out.extend_from_slice(&p.offset.to_le_bytes());
    out.extend_from_slice(&p.size.to_le_bytes());
    out.extend_from_slice(&p.link.to_le_bytes());
    out.extend_from_slice(&p.info.to_le_bytes());
    out.extend_from_slice(&p.addralign.to_le_bytes());
    out.extend_from_slice(&p.entsize.to_le_bytes());
}

fn align_up(n: u64, align: u64) -> u64 {
    if align == 0 {
        return n;
    }
    n.div_ceil(align) * align
}

fn pad_to(out: &mut Vec<u8>, offset: u64) {
    while (out.len() as u64) < offset {
        out.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{MachineObject, ObjReloc, ObjSymbol, RelKind, RelTarget};

    #[test]
    fn test_minimal_elf_header() {
        let obj = MachineObject {
            text: vec![0x90, 0xC3],
            ..Default::default()
        };
        let elf = write_relocatable_elf(&obj).unwrap();
        assert_eq!(&elf[0..4], &[0x7f, b'E', b'L', b'F']);
        assert_eq!(elf[16], 1); // ET_REL
        assert_eq!(elf[18], 62); // EM_X86_64
    }

    #[test]
    fn test_symbol_and_reloc_present() {
        let obj = MachineObject {
            text: vec![0xE8, 0, 0, 0, 0, 0xC3],
            rodata: b"hi\0".to_vec(),
            symbols: vec![ObjSymbol {
                name: "puts".into(),
                section: None,
                value: 0,
                size: 0,
                is_func: false,
                is_global: true,
            }],
            relocations: vec![ObjReloc {
                offset: 1,
                target: RelTarget::Symbol("puts".into()),
                kind: RelKind::Plt32,
                addend: -4,
            }],
            ..Default::default()
        };
        let elf = write_relocatable_elf(&obj).unwrap();
        // 至少能写出且包含字符串表里的 puts
        assert!(elf.windows(4).any(|w| w == b"puts"));
    }
}
