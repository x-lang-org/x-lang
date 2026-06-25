//! Mach-O `MH_OBJECT` 目标文件写出器（x86_64 + arm64）
//!
//! 生成 64 位可重定位 Mach-O 目标文件，含单个 `LC_SEGMENT_64`（其中
//! `__text`/`__cstring`/`__data`/`__bss` section）与 `LC_SYMTAB`，并为
//! `.text` 内的引用发射 `relocation_info` 记录。由于宿主多为 Linux，
//! 该产物在本机仅做结构校验，执行在 macOS CI 完成。

use crate::arch::TargetArch;
use crate::machine::{MachineObject, RelKind, RelTarget, SecKind};
use crate::{NativeError, NativeResult};

// CPU 类型
const CPU_ARCH_ABI64: u32 = 0x0100_0000;
const CPU_TYPE_X86_64: u32 = 7 | CPU_ARCH_ABI64;
const CPU_TYPE_ARM64: u32 = 12 | CPU_ARCH_ABI64;
const CPU_SUBTYPE_X86_64_ALL: u32 = 3;
const CPU_SUBTYPE_ARM64_ALL: u32 = 0;

const MH_MAGIC_64: u32 = 0xfeed_facf;
const MH_OBJECT: u32 = 1;
const MH_SUBSECTIONS_VIA_SYMBOLS: u32 = 0x2000;

const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x2;

// section 属性
const S_REGULAR: u32 = 0x0;
const S_ZEROFILL: u32 = 0x1;
const S_CSTRING_LITERALS: u32 = 0x2;
const S_ATTR_PURE_INSTRUCTIONS: u32 = 0x8000_0000;
const S_ATTR_SOME_INSTRUCTIONS: u32 = 0x0000_0400;

// 符号类型
const N_EXT: u8 = 0x01;
const N_SECT: u8 = 0x0e;
const N_UNDF: u8 = 0x00;

// x86_64 重定位类型
const X86_64_RELOC_BRANCH: u32 = 2;
const X86_64_RELOC_SIGNED: u32 = 1;
const X86_64_RELOC_UNSIGNED: u32 = 0;
// arm64 重定位类型
const ARM64_RELOC_BRANCH26: u32 = 2;
const ARM64_RELOC_PAGE21: u32 = 3;
const ARM64_RELOC_PAGEOFF12: u32 = 4;
const ARM64_RELOC_UNSIGNED: u32 = 0;
const ARM64_RELOC_ADDEND: u32 = 10;

struct Section {
    sectname: [u8; 16],
    addr: u64,
    size: u64,
    offset: u32,
    align: u32,
    reloff: u32,
    nreloc: u32,
    flags: u32,
}

fn sect_name(s: &str) -> [u8; 16] {
    let mut a = [0u8; 16];
    let b = s.as_bytes();
    a[..b.len()].copy_from_slice(b);
    a
}

/// 一条 Mach-O 重定位（scattered=0）
struct MachReloc {
    r_address: i32,
    r_symbolnum: u32, // extern: 符号下标；非 extern: section 号或 ADDEND 值
    r_pcrel: bool,
    r_length: u8, // 0=1,1=2,2=4,3=8
    r_extern: bool,
    r_type: u32,
}

impl MachReloc {
    fn encode(&self) -> [u8; 8] {
        let mut out = [0u8; 8];
        out[..4].copy_from_slice(&self.r_address.to_le_bytes());
        let packed: u32 = (self.r_symbolnum & 0x00ff_ffff)
            | ((self.r_pcrel as u32) << 24)
            | (((self.r_length & 0x3) as u32) << 25)
            | ((self.r_extern as u32) << 27)
            | ((self.r_type & 0xf) << 28);
        out[4..].copy_from_slice(&packed.to_le_bytes());
        out
    }
}

pub fn write_macho(obj: &MachineObject, arch: TargetArch) -> NativeResult<Vec<u8>> {
    let (cputype, cpusubtype) = match arch {
        TargetArch::X86_64 => (CPU_TYPE_X86_64, CPU_SUBTYPE_X86_64_ALL),
        TargetArch::AArch64 => (CPU_TYPE_ARM64, CPU_SUBTYPE_ARM64_ALL),
        other => {
            return Err(NativeError::Unimplemented(format!(
                "Mach-O 不支持架构 {:?}",
                other
            )))
        }
    };

    // ---- 符号表：先定义（N_SECT），后外部未定义（N_UNDF） ----
    // Mach-O 重定位的 r_symbolnum 指向符号表下标。
    let mut symnames: Vec<(String, u8, u8, u64)> = Vec::new(); // name, n_type, n_sect, value
    let mut sym_index: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    // 计算段内 section 地址（addr 连续累计）
    let text_addr: u64 = 0;
    let text_size = obj.text.len() as u64;
    let cstring_addr = align_up(text_addr + text_size, 1);
    let cstring_size = obj.rodata.len() as u64;
    let data_addr = align_up(cstring_addr + cstring_size, 8);
    let data_size = obj.data.len() as u64;
    let bss_addr = align_up(data_addr + data_size, 8);
    let bss_size = obj.bss_size;

    // 只发射非空的 section（__text 始终保留）。空的 __cstring/__data/__bss 会让
    // ld64/lld 在链接极小目标文件时崩溃，因此动态决定包含哪些 section 并据此
    // 分配 section 号（n_sect 从 1 起按出现顺序编号）。
    let mut order: Vec<SecKind> = vec![SecKind::Text];
    if cstring_size > 0 {
        order.push(SecKind::Rodata);
    }
    if data_size > 0 {
        order.push(SecKind::Data);
    }
    if bss_size > 0 {
        order.push(SecKind::Bss);
    }
    let nsect_of =
        |sk: SecKind| -> u8 { (order.iter().position(|x| *x == sk).unwrap_or(0) + 1) as u8 };
    let addr_of = |sk: SecKind| -> u64 {
        match sk {
            SecKind::Text => text_addr,
            SecKind::Rodata => cstring_addr,
            SecKind::Data => data_addr,
            SecKind::Bss => bss_addr,
        }
    };

    for s in &obj.symbols {
        if let Some(sk) = s.section {
            let n_sect = nsect_of(sk);
            let base = addr_of(sk);
            let idx = symnames.len() as u32;
            sym_index.insert(s.name.clone(), idx);
            symnames.push((
                format!("_{}", s.name),
                N_SECT | if s.is_global { N_EXT } else { 0 },
                n_sect,
                base + s.value,
            ));
        }
    }
    for s in &obj.symbols {
        if s.section.is_none() {
            let idx = symnames.len() as u32;
            sym_index.insert(s.name.clone(), idx);
            symnames.push((format!("_{}", s.name), N_UNDF | N_EXT, 0, 0));
        }
    }

    // 为每个（已包含的）section 合成一个本地锚符号（供 section 相对重定位引用，带 addend）。
    let mut section_anchor: std::collections::HashMap<u8, u32> = std::collections::HashMap::new();
    {
        for &sk in &order {
            let nsect = nsect_of(sk);
            let name = match sk {
                SecKind::Text => "ltmp_text",
                SecKind::Rodata => "ltmp_cstring",
                SecKind::Data => "ltmp_data",
                SecKind::Bss => "ltmp_bss",
            };
            let idx = symnames.len() as u32;
            section_anchor.insert(nsect, idx);
            symnames.push((name.to_string(), N_SECT, nsect, addr_of(sk)));
        }
    }

    // 可变 text 副本：x86_64 的 addend 需写入指令立即数。
    let mut text = obj.text.clone();

    // ---- 把 ObjReloc 转换为 Mach-O 重定位（仅 .text） ----
    let mut text_relocs: Vec<MachReloc> = Vec::new();
    for r in &obj.relocations {
        let (symnum, is_section, addend) = match &r.target {
            RelTarget::Symbol(name) => (
                *sym_index.get(name).ok_or_else(|| {
                    NativeError::CodegenError(format!("Mach-O 重定位引用未知符号: {}", name))
                })?,
                false,
                r.addend,
            ),
            RelTarget::Section(sk) => {
                let nsect = nsect_of(*sk);
                (*section_anchor.get(&nsect).unwrap(), true, r.addend)
            }
        };
        let (r_type, r_pcrel, r_length) = match (arch, r.kind) {
            (TargetArch::X86_64, RelKind::Plt32) | (TargetArch::X86_64, RelKind::Pc32) => (
                if matches!(r.kind, RelKind::Plt32) {
                    X86_64_RELOC_BRANCH
                } else {
                    X86_64_RELOC_SIGNED
                },
                true,
                2u8,
            ),
            (TargetArch::X86_64, RelKind::Abs64) => (X86_64_RELOC_UNSIGNED, false, 3u8),
            (TargetArch::AArch64, RelKind::Aarch64Call26) => (ARM64_RELOC_BRANCH26, true, 2u8),
            (TargetArch::AArch64, RelKind::Aarch64AdrPrelPgHi21) => (ARM64_RELOC_PAGE21, true, 2u8),
            (TargetArch::AArch64, RelKind::Aarch64AddAbsLo12Nc) => {
                (ARM64_RELOC_PAGEOFF12, false, 2u8)
            }
            (TargetArch::AArch64, RelKind::Abs64) => (ARM64_RELOC_UNSIGNED, false, 3u8),
            other => {
                return Err(NativeError::Unimplemented(format!(
                    "Mach-O 不支持重定位 {:?}",
                    other
                )))
            }
        };

        // 处理 addend：
        // - x86_64：写入指令字段（SIGNED/BRANCH 用 32 位，UNSIGNED 用 64 位）。
        // - arm64：addend != 0 时，前置一条 ARM64_RELOC_ADDEND。
        if addend != 0 {
            match arch {
                TargetArch::X86_64 => {
                    let at = r.offset as usize;
                    if r_length == 3 {
                        let v = addend; // 绝对，pcrel 的 SIGNED addend 仍写入字段
                        text[at..at + 8].copy_from_slice(&v.to_le_bytes());
                    } else {
                        let v = addend as i32;
                        text[at..at + 4].copy_from_slice(&v.to_le_bytes());
                    }
                }
                TargetArch::AArch64 => {
                    text_relocs.push(MachReloc {
                        r_address: r.offset as i32,
                        r_symbolnum: (addend as u32) & 0x00ff_ffff,
                        r_pcrel: false,
                        r_length: 2,
                        r_extern: false,
                        r_type: ARM64_RELOC_ADDEND,
                    });
                }
                _ => {}
            }
        }

        let _ = is_section;
        text_relocs.push(MachReloc {
            r_address: r.offset as i32,
            r_symbolnum: symnum,
            r_pcrel,
            r_length,
            r_extern: true, // 均以符号下标引用（含 section 锚符号）
            r_type,
        });
    }

    // ---- 文件布局 ----
    // header(32) + LC_SEGMENT_64(72) + nsects*section(80) + LC_SYMTAB(24)
    let nsects = order.len() as u32;
    let ncmds = 2u32;
    let seg_cmdsize = 72u32 + nsects * 80;
    let symtab_cmdsize = 24u32;
    let sizeofcmds = seg_cmdsize + symtab_cmdsize;
    let header_end = 32 + sizeofcmds as u64;

    let mut off = header_end;
    let text_off = off as u32;
    off += text_size;
    off = align_up(off, 1);
    let cstring_off = off as u32;
    off += cstring_size;
    off = align_up(off, 8);
    let data_off = off as u32;
    off += data_size;
    off = align_up(off, 8);
    // .bss zerofill: 无文件内容
    let reloc_off = off as u32;
    off += (text_relocs.len() * 8) as u64;
    off = align_up(off, 8);
    let symoff = off as u32;
    off += (symnames.len() * 16) as u64;
    let stroff = off as u32;
    // 字符串表
    let mut strtab: Vec<u8> = vec![0];
    let mut name_offs: Vec<u32> = Vec::new();
    for (name, _, _, _) in &symnames {
        name_offs.push(strtab.len() as u32);
        strtab.extend_from_slice(name.as_bytes());
        strtab.push(0);
    }
    let strsize = strtab.len() as u32;

    let sections: Vec<Section> = order
        .iter()
        .map(|sk| match sk {
            SecKind::Text => Section {
                sectname: sect_name("__text"),
                addr: text_addr,
                size: text_size,
                offset: text_off,
                align: 4,
                reloff: if text_relocs.is_empty() { 0 } else { reloc_off },
                nreloc: text_relocs.len() as u32,
                flags: S_REGULAR | S_ATTR_PURE_INSTRUCTIONS | S_ATTR_SOME_INSTRUCTIONS,
            },
            SecKind::Rodata => Section {
                sectname: sect_name("__cstring"),
                addr: cstring_addr,
                size: cstring_size,
                offset: cstring_off,
                align: 0,
                reloff: 0,
                nreloc: 0,
                flags: S_CSTRING_LITERALS,
            },
            SecKind::Data => Section {
                sectname: sect_name("__data"),
                addr: data_addr,
                size: data_size,
                offset: data_off,
                align: 3,
                reloff: 0,
                nreloc: 0,
                flags: S_REGULAR,
            },
            SecKind::Bss => Section {
                sectname: sect_name("__bss"),
                addr: bss_addr,
                size: bss_size,
                offset: 0,
                align: 3,
                reloff: 0,
                nreloc: 0,
                flags: S_ZEROFILL,
            },
        })
        .collect();

    let mut out: Vec<u8> = Vec::new();

    // mach_header_64
    out.extend_from_slice(&MH_MAGIC_64.to_le_bytes());
    out.extend_from_slice(&cputype.to_le_bytes());
    out.extend_from_slice(&cpusubtype.to_le_bytes());
    out.extend_from_slice(&MH_OBJECT.to_le_bytes());
    out.extend_from_slice(&ncmds.to_le_bytes());
    out.extend_from_slice(&sizeofcmds.to_le_bytes());
    out.extend_from_slice(&MH_SUBSECTIONS_VIA_SYMBOLS.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // reserved

    // LC_SEGMENT_64
    out.extend_from_slice(&LC_SEGMENT_64.to_le_bytes());
    out.extend_from_slice(&seg_cmdsize.to_le_bytes());
    out.extend_from_slice(&[0u8; 16]); // segname "" (空段名)
    out.extend_from_slice(&0u64.to_le_bytes()); // vmaddr
    let vmsize = align_up(bss_addr + bss_size, 8);
    out.extend_from_slice(&vmsize.to_le_bytes()); // vmsize
    out.extend_from_slice(&(text_off as u64).to_le_bytes()); // fileoff
    let filesize = (data_off as u64 + data_size) - text_off as u64;
    out.extend_from_slice(&filesize.to_le_bytes()); // filesize
    out.extend_from_slice(&7u32.to_le_bytes()); // maxprot rwx
    out.extend_from_slice(&7u32.to_le_bytes()); // initprot rwx
    out.extend_from_slice(&(sections.len() as u32).to_le_bytes()); // nsects
    out.extend_from_slice(&0u32.to_le_bytes()); // flags

    for s in &sections {
        out.extend_from_slice(&s.sectname);
        // segname: 定义在 __TEXT/__DATA 中；这里用 __TEXT 给前两个，__DATA 给后两个
        let seg = if s.sectname.starts_with(b"__text") || s.sectname.starts_with(b"__cstring") {
            sect_name("__TEXT")
        } else {
            sect_name("__DATA")
        };
        out.extend_from_slice(&seg);
        out.extend_from_slice(&s.addr.to_le_bytes());
        out.extend_from_slice(&s.size.to_le_bytes());
        out.extend_from_slice(&s.offset.to_le_bytes());
        out.extend_from_slice(&s.align.to_le_bytes());
        out.extend_from_slice(&s.reloff.to_le_bytes());
        out.extend_from_slice(&s.nreloc.to_le_bytes());
        out.extend_from_slice(&s.flags.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes()); // reserved1
        out.extend_from_slice(&0u32.to_le_bytes()); // reserved2
        out.extend_from_slice(&0u32.to_le_bytes()); // reserved3
    }

    // LC_SYMTAB
    out.extend_from_slice(&LC_SYMTAB.to_le_bytes());
    out.extend_from_slice(&symtab_cmdsize.to_le_bytes());
    out.extend_from_slice(&symoff.to_le_bytes());
    out.extend_from_slice(&(symnames.len() as u32).to_le_bytes());
    out.extend_from_slice(&stroff.to_le_bytes());
    out.extend_from_slice(&strsize.to_le_bytes());

    // section 数据
    pad_to(&mut out, text_off as u64);
    out.extend_from_slice(&text);
    pad_to(&mut out, cstring_off as u64);
    out.extend_from_slice(&obj.rodata);
    pad_to(&mut out, data_off as u64);
    out.extend_from_slice(&obj.data);

    // 重定位
    pad_to(&mut out, reloc_off as u64);
    for r in &text_relocs {
        out.extend_from_slice(&r.encode());
    }

    // nlist_64 符号表
    pad_to(&mut out, symoff as u64);
    for (i, (_, n_type, n_sect, value)) in symnames.iter().enumerate() {
        out.extend_from_slice(&name_offs[i].to_le_bytes()); // n_strx
        out.push(*n_type); // n_type
        out.push(*n_sect); // n_sect
        out.extend_from_slice(&0u16.to_le_bytes()); // n_desc
        out.extend_from_slice(&value.to_le_bytes()); // n_value
    }

    // 字符串表
    pad_to(&mut out, stroff as u64);
    out.extend_from_slice(&strtab);

    Ok(out)
}

fn align_up(n: u64, a: u64) -> u64 {
    if a == 0 {
        return n;
    }
    n.div_ceil(a) * a
}

fn pad_to(out: &mut Vec<u8>, offset: u64) {
    while (out.len() as u64) < offset {
        out.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::MachineObject;

    #[test]
    fn test_macho_header_x86_64() {
        let obj = MachineObject {
            text: vec![0xc3],
            ..Default::default()
        };
        let m = write_macho(&obj, TargetArch::X86_64).unwrap();
        assert_eq!(&m[0..4], &MH_MAGIC_64.to_le_bytes());
        assert_eq!(
            u32::from_le_bytes(m[4..8].try_into().unwrap()),
            CPU_TYPE_X86_64
        );
    }

    #[test]
    fn test_macho_header_arm64() {
        let obj = MachineObject {
            text: vec![0xc0, 0x03, 0x5f, 0xd6],
            ..Default::default()
        };
        let m = write_macho(&obj, TargetArch::AArch64).unwrap();
        assert_eq!(
            u32::from_le_bytes(m[4..8].try_into().unwrap()),
            CPU_TYPE_ARM64
        );
    }
}
