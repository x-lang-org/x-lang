//! PE/COFF 目标文件写出器（x86_64 + arm64）
//!
//! 生成 Windows 链接器可用的 COFF 目标文件，含 `.text`/`.rdata`/`.data`/`.bss`
//! section、符号表、字符串表，以及 `.text` 内引用的重定位记录。宿主多为
//! Linux，执行验证在 Windows（或 wine for x86_64）完成。

use crate::arch::TargetArch;
use crate::machine::{MachineObject, RelKind, RelTarget, SecKind};
use crate::{NativeError, NativeResult};

const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_MACHINE_ARM64: u16 = 0xAA64;

const IMAGE_SCN_CNT_CODE: u32 = 0x0000_0020;
const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x0000_0040;
const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x0000_0080;
const IMAGE_SCN_ALIGN_16BYTES: u32 = 0x0050_0000;
const IMAGE_SCN_MEM_EXECUTE: u32 = 0x2000_0000;
const IMAGE_SCN_MEM_READ: u32 = 0x4000_0000;
const IMAGE_SCN_MEM_WRITE: u32 = 0x8000_0000;

// x86_64 重定位
const IMAGE_REL_AMD64_ADDR64: u16 = 0x0001;
const IMAGE_REL_AMD64_REL32: u16 = 0x0004;
// arm64 重定位
const IMAGE_REL_ARM64_ADDR64: u16 = 0x000E;
const IMAGE_REL_ARM64_BRANCH26: u16 = 0x0003;
const IMAGE_REL_ARM64_PAGEBASE_REL21: u16 = 0x0004;
const IMAGE_REL_ARM64_PAGEOFFSET_12A: u16 = 0x0006;

const IMAGE_SYM_CLASS_EXTERNAL: u8 = 2;
const IMAGE_SYM_CLASS_STATIC: u8 = 3;

struct CoffReloc {
    vaddr: u32,
    sym_idx: u32,
    typ: u16,
}

pub fn write_coff(obj: &MachineObject, arch: TargetArch) -> NativeResult<Vec<u8>> {
    let machine = match arch {
        TargetArch::X86_64 => IMAGE_FILE_MACHINE_AMD64,
        TargetArch::AArch64 => IMAGE_FILE_MACHINE_ARM64,
        other => {
            return Err(NativeError::Unimplemented(format!(
                "COFF 不支持架构 {:?}",
                other
            )))
        }
    };

    // section 编号：1=.text 2=.rdata 3=.data 4=.bss
    let nsections = 4u16;

    // ---- 符号表 ----
    // COFF 符号：每个 18 字节；长名（>8）放字符串表。
    struct CoffSym {
        name: String,
        value: u32,
        section_number: i16, // 1-based, 0=undef
        class: u8,
    }
    let mut syms: Vec<CoffSym> = Vec::new();
    let mut sym_index: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    // section 符号（用于 section 相对重定位的锚点）
    for (i, _name) in [".text", ".rdata", ".data", ".bss"].iter().enumerate() {
        sym_index.insert(format!("$sec{}", i + 1), syms.len() as u32);
        syms.push(CoffSym {
            name: _name.to_string(),
            value: 0,
            section_number: (i + 1) as i16,
            class: IMAGE_SYM_CLASS_STATIC,
        });
    }

    for s in &obj.symbols {
        let section_number = match s.section {
            Some(SecKind::Text) => 1,
            Some(SecKind::Rodata) => 2,
            Some(SecKind::Data) => 3,
            Some(SecKind::Bss) => 4,
            None => 0,
        };
        sym_index.insert(s.name.clone(), syms.len() as u32);
        syms.push(CoffSym {
            name: s.name.clone(),
            value: s.value as u32,
            section_number,
            class: IMAGE_SYM_CLASS_EXTERNAL,
        });
    }

    // ---- 重定位（仅 .text） ----
    let mut text_relocs: Vec<CoffReloc> = Vec::new();
    for r in &obj.relocations {
        let sym_idx = match &r.target {
            RelTarget::Symbol(name) => *sym_index.get(name).ok_or_else(|| {
                NativeError::CodegenError(format!("COFF 重定位引用未知符号: {}", name))
            })?,
            RelTarget::Section(sk) => {
                let n = match sk {
                    SecKind::Text => 1,
                    SecKind::Rodata => 2,
                    SecKind::Data => 3,
                    SecKind::Bss => 4,
                };
                *sym_index.get(&format!("$sec{}", n)).unwrap()
            }
        };
        let typ = match (arch, r.kind) {
            (TargetArch::X86_64, RelKind::Plt32) | (TargetArch::X86_64, RelKind::Pc32) => {
                IMAGE_REL_AMD64_REL32
            }
            (TargetArch::X86_64, RelKind::Abs64) => IMAGE_REL_AMD64_ADDR64,
            (TargetArch::AArch64, RelKind::Aarch64Call26) => IMAGE_REL_ARM64_BRANCH26,
            (TargetArch::AArch64, RelKind::Aarch64AdrPrelPgHi21) => IMAGE_REL_ARM64_PAGEBASE_REL21,
            (TargetArch::AArch64, RelKind::Aarch64AddAbsLo12Nc) => IMAGE_REL_ARM64_PAGEOFFSET_12A,
            (TargetArch::AArch64, RelKind::Abs64) => IMAGE_REL_ARM64_ADDR64,
            other => {
                return Err(NativeError::Unimplemented(format!(
                    "COFF 不支持重定位 {:?}",
                    other
                )))
            }
        };
        text_relocs.push(CoffReloc {
            vaddr: r.offset as u32,
            sym_idx,
            typ,
        });
    }

    // ---- 文件布局 ----
    let header_size = 20u32;
    let secthdr_size = 40u32 * nsections as u32;
    let mut off = header_size + secthdr_size;

    let text_ptr = if obj.text.is_empty() { 0 } else { off };
    off += obj.text.len() as u32;
    let rdata_ptr = if obj.rodata.is_empty() { 0 } else { off };
    off += obj.rodata.len() as u32;
    let data_ptr = if obj.data.is_empty() { 0 } else { off };
    off += obj.data.len() as u32;
    // .bss 无文件数据
    let reloc_ptr = if text_relocs.is_empty() { 0 } else { off };
    off += (text_relocs.len() * 10) as u32;
    let symtab_ptr = off;
    off += syms.len() as u32 * 18;
    let _strtab_ptr = off;

    // 字符串表
    let mut strtab: Vec<u8> = vec![0, 0, 0, 0]; // 前 4 字节为长度（含自身）
    let mut name_field: Vec<[u8; 8]> = Vec::new();
    for s in &syms {
        let mut nf = [0u8; 8];
        if s.name.len() <= 8 {
            nf[..s.name.len()].copy_from_slice(s.name.as_bytes());
        } else {
            let strx = strtab.len() as u32;
            strtab.extend_from_slice(s.name.as_bytes());
            strtab.push(0);
            // name 字段：前 4 字节 0，后 4 字节为字符串表偏移
            nf[0..4].copy_from_slice(&0u32.to_le_bytes());
            nf[4..8].copy_from_slice(&strx.to_le_bytes());
        }
        name_field.push(nf);
    }
    let strtab_len = strtab.len() as u32;
    strtab[0..4].copy_from_slice(&strtab_len.to_le_bytes());

    let mut out: Vec<u8> = Vec::new();

    // COFF 头
    out.extend_from_slice(&machine.to_le_bytes());
    out.extend_from_slice(&nsections.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
    out.extend_from_slice(&symtab_ptr.to_le_bytes());
    out.extend_from_slice(&(syms.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // SizeOfOptionalHeader
    out.extend_from_slice(&0u16.to_le_bytes()); // Characteristics

    let write_secthdr = |out: &mut Vec<u8>,
                         name: &str,
                         size: u32,
                         ptr: u32,
                         reloff: u32,
                         nreloc: u16,
                         chars: u32| {
        let mut nb = [0u8; 8];
        nb[..name.len()].copy_from_slice(name.as_bytes());
        out.extend_from_slice(&nb);
        out.extend_from_slice(&0u32.to_le_bytes()); // VirtualSize
        out.extend_from_slice(&0u32.to_le_bytes()); // VirtualAddress
        out.extend_from_slice(&size.to_le_bytes()); // SizeOfRawData
        out.extend_from_slice(&ptr.to_le_bytes()); // PointerToRawData
        out.extend_from_slice(&reloff.to_le_bytes()); // PointerToRelocations
        out.extend_from_slice(&0u32.to_le_bytes()); // PointerToLineNumbers
        out.extend_from_slice(&nreloc.to_le_bytes()); // NumberOfRelocations
        out.extend_from_slice(&0u16.to_le_bytes()); // NumberOfLineNumbers
        out.extend_from_slice(&chars.to_le_bytes()); // Characteristics
    };

    write_secthdr(
        &mut out,
        ".text",
        obj.text.len() as u32,
        text_ptr,
        reloc_ptr,
        text_relocs.len() as u16,
        IMAGE_SCN_CNT_CODE | IMAGE_SCN_ALIGN_16BYTES | IMAGE_SCN_MEM_EXECUTE | IMAGE_SCN_MEM_READ,
    );
    write_secthdr(
        &mut out,
        ".rdata",
        obj.rodata.len() as u32,
        rdata_ptr,
        0,
        0,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_ALIGN_16BYTES | IMAGE_SCN_MEM_READ,
    );
    write_secthdr(
        &mut out,
        ".data",
        obj.data.len() as u32,
        data_ptr,
        0,
        0,
        IMAGE_SCN_CNT_INITIALIZED_DATA
            | IMAGE_SCN_ALIGN_16BYTES
            | IMAGE_SCN_MEM_READ
            | IMAGE_SCN_MEM_WRITE,
    );
    write_secthdr(
        &mut out,
        ".bss",
        obj.bss_size as u32,
        0,
        0,
        0,
        IMAGE_SCN_CNT_UNINITIALIZED_DATA
            | IMAGE_SCN_ALIGN_16BYTES
            | IMAGE_SCN_MEM_READ
            | IMAGE_SCN_MEM_WRITE,
    );

    // section 数据
    out.extend_from_slice(&obj.text);
    out.extend_from_slice(&obj.rodata);
    out.extend_from_slice(&obj.data);

    // 重定位
    for r in &text_relocs {
        out.extend_from_slice(&r.vaddr.to_le_bytes());
        out.extend_from_slice(&r.sym_idx.to_le_bytes());
        out.extend_from_slice(&r.typ.to_le_bytes());
    }

    // 符号表
    for (i, s) in syms.iter().enumerate() {
        out.extend_from_slice(&name_field[i]);
        out.extend_from_slice(&s.value.to_le_bytes());
        out.extend_from_slice(&s.section_number.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // Type
        out.push(s.class);
        out.push(0); // NumberOfAuxSymbols
    }

    // 字符串表
    out.extend_from_slice(&strtab);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::MachineObject;

    #[test]
    fn test_coff_header_x86_64() {
        let obj = MachineObject {
            text: vec![0xc3],
            ..Default::default()
        };
        let c = write_coff(&obj, TargetArch::X86_64).unwrap();
        assert_eq!(
            u16::from_le_bytes(c[0..2].try_into().unwrap()),
            IMAGE_FILE_MACHINE_AMD64
        );
        assert_eq!(u16::from_le_bytes(c[2..4].try_into().unwrap()), 4);
    }

    #[test]
    fn test_coff_header_arm64() {
        let obj = MachineObject {
            text: vec![0xc0, 0x03, 0x5f, 0xd6],
            ..Default::default()
        };
        let c = write_coff(&obj, TargetArch::AArch64).unwrap();
        assert_eq!(
            u16::from_le_bytes(c[0..2].try_into().unwrap()),
            IMAGE_FILE_MACHINE_ARM64
        );
    }
}
