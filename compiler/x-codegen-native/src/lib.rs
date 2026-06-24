//! Native 后端 - LIR 直出机器码
//!
//! 仿照 Zig 自举后端：把 LIR 直接降级为机器码字节，边发射边登记标签修补
//! （函数内跳转/调用）与重定位（外部符号、字符串/数据引用），写出可重定位
//! ELF 目标文件，再由系统链接器（cc）链接为可执行文件。不依赖外部汇编器。
//!
//! # 架构概述
//!
//! ```text
//! LIR → MachineCodeGen → 机器码字节 + 重定位 + 符号 → 可重定位 ELF (.o) → cc 链接
//! ```
//!
//! # 支持的目标
//!
//! - **x86_64 Linux**: System V AMD64 ABI（当前唯一支持的直出目标）
//!
//! # 示例
//!
//! ```ignore
//! use x_codegen_native::{NativeBackend, NativeBackendConfig, TargetArch, TargetOS, OutputFormat};
//!
//! let config = NativeBackendConfig {
//!     arch: TargetArch::X86_64,
//!     os: TargetOS::Linux,
//!     format: OutputFormat::ObjectFile,
//!     ..Default::default()
//! };
//!
//! let mut backend = NativeBackend::new(config);
//! let output = backend.generate_from_lir(&lir)?;
//! ```

#![allow(
    clippy::byte_char_slices,
    clippy::collapsible_if,
    clippy::explicit_auto_deref,
    clippy::for_kv_map,
    clippy::if_same_then_else,
    clippy::io_other_error,
    clippy::manual_div_ceil,
    clippy::manual_range_contains,
    clippy::needless_borrow,
    clippy::only_used_in_recursion,
    clippy::redundant_closure,
    clippy::single_match,
    clippy::unnecessary_cast,
    clippy::unused_enumerate_index,
    clippy::useless_format
)]

use std::path::PathBuf;
use x_codegen::{CodeGenerator, CodegenOutput, FileType, OutputFile};

// ============================================================================
// 公共接口
// ============================================================================

pub mod arch;
pub mod emitter;
pub mod encoding;
pub mod machine;

pub use arch::{Instruction, MemoryOperand, Register, TargetArch};
pub use emitter::write_relocatable_elf;
pub use encoding::MachineCodeEncoder;
pub use machine::{MachineCodeGen, MachineObject};

// ============================================================================
// 配置与错误类型
// ============================================================================

/// Native 后端配置
#[derive(Debug, Clone)]
pub struct NativeBackendConfig {
    /// 输出目录
    pub output_dir: Option<PathBuf>,
    /// 是否启用优化
    pub optimize: bool,
    /// 是否生成调试信息
    pub debug_info: bool,
    /// 目标架构
    pub arch: TargetArch,
    /// 输出格式
    pub format: OutputFormat,
    /// 操作系统（影响调用约定）
    pub os: TargetOS,
}

impl Default for NativeBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            arch: TargetArch::default(),
            format: OutputFormat::default(),
            os: TargetOS::default(),
        }
    }
}

/// 目标操作系统
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetOS {
    #[default]
    Linux,
    MacOS,
    Windows,
}

impl TargetOS {
    /// 获取目标三段式标识
    pub fn target_triple(&self, arch: TargetArch) -> String {
        match (arch, self) {
            (TargetArch::X86_64, TargetOS::Linux) => "x86_64-unknown-linux-gnu".to_string(),
            (TargetArch::X86_64, TargetOS::MacOS) => "x86_64-apple-darwin".to_string(),
            (TargetArch::X86_64, TargetOS::Windows) => "x86_64-pc-windows-msvc".to_string(),
            (TargetArch::AArch64, TargetOS::Linux) => "aarch64-unknown-linux-gnu".to_string(),
            (TargetArch::AArch64, TargetOS::MacOS) => "aarch64-apple-darwin".to_string(),
            (TargetArch::AArch64, TargetOS::Windows) => "aarch64-pc-windows-msvc".to_string(),
            (TargetArch::RiscV64, TargetOS::Linux) => "riscv64-unknown-linux-gnu".to_string(),
            (TargetArch::RiscV64, _) => "riscv64-unknown-elf".to_string(),
            (TargetArch::Wasm32, _) => "wasm32-unknown-unknown".to_string(),
        }
    }

    /// 是否使用 System V ABI
    pub fn uses_system_v_abi(&self) -> bool {
        matches!(self, TargetOS::Linux | TargetOS::MacOS)
    }

    /// 是否使用 Microsoft x64 调用约定
    pub fn uses_microsoft_abi(&self) -> bool {
        matches!(self, TargetOS::Windows)
    }
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// 可执行文件
    #[default]
    Executable,
    /// 目标文件（.o/.obj）
    ObjectFile,
    /// 汇编代码（.s/.asm）
    Assembly,
    /// 机器码（原始字节）
    RawBinary,
}

/// Native 后端错误类型
#[derive(Debug, thiserror::Error)]
pub enum NativeError {
    #[error("机器码生成错误: {0}")]
    CodegenError(String),

    #[error("不支持的架构: {0}")]
    UnsupportedArch(String),

    #[error("未实现的功能: {0}")]
    Unimplemented(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("格式化错误: {0}")]
    FmtError(#[from] std::fmt::Error),

    #[error("编码错误: {0}")]
    EncodingError(String),

    #[error("无效的操作数: {0}")]
    InvalidOperand(String),

    #[error("寄存器分配失败: {0}")]
    RegisterAllocationFailed(String),

    #[error("不支持的类型: {0}")]
    UnsupportedType(String),
}

pub type NativeResult<T> = Result<T, NativeError>;

// ============================================================================
// Native 后端实现
// ============================================================================

/// Native 后端
///
/// 直接从 LIR 生成机器码，无需外部编译器。
/// 代码生成委托给 `assembly` 模块中的架构特定 `AssemblyGenerator`。
pub struct NativeBackend {
    config: NativeBackendConfig,
}

impl NativeBackend {
    /// 创建新的 Native 后端
    pub fn new(config: NativeBackendConfig) -> Self {
        Self { config }
    }

    /// 直出机器码：生成可重定位 ELF 目标文件字节
    fn generate_from_lir_impl(
        &mut self,
        lir: &x_lir::Program,
    ) -> Result<CodegenOutput, NativeError> {
        // 当前仅支持 x86_64 Linux 直出机器码
        if self.config.arch != TargetArch::X86_64 {
            return Err(NativeError::Unimplemented(format!(
                "Native 直出机器码目前仅支持 x86_64，收到: {}",
                self.config.arch
            )));
        }
        if self.config.os != TargetOS::Linux {
            return Err(NativeError::Unimplemented(format!(
                "Native 直出机器码目前仅支持 Linux，收到: {:?}",
                self.config.os
            )));
        }

        let mut gen = MachineCodeGen::new(self.config.os);
        let object = gen.generate(lir)?;
        let elf = emitter::write_relocatable_elf(&object)?;

        let output_file = OutputFile {
            path: PathBuf::from("output.o"),
            content: elf,
            file_type: FileType::ObjectFile,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

impl CodeGenerator for NativeBackend {
    type Config = NativeBackendConfig;
    type Error = NativeError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, Self::Error> {
        self.generate_from_lir_impl(lir)
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use x_lir::{self as lir, Expression, Statement, Type};

    fn compile_to_object(lir: &lir::Program) -> Vec<u8> {
        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            os: TargetOS::Linux,
            format: OutputFormat::ObjectFile,
            ..Default::default()
        };
        let mut backend = NativeBackend::new(config);
        let out = backend.generate_from_lir(lir).unwrap();
        assert_eq!(out.files.len(), 1);
        assert_eq!(out.files[0].file_type, FileType::ObjectFile);
        out.files[0].content.clone()
    }

    #[test]
    fn test_config_default() {
        let config = NativeBackendConfig::default();
        assert_eq!(config.arch, TargetArch::X86_64);
        assert_eq!(config.os, TargetOS::Linux);
    }

    #[test]
    fn test_emits_relocatable_elf_object() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(42))));
        program.add(lir::Declaration::Function(func));

        let obj = compile_to_object(&program);
        // ELF 魔数
        assert_eq!(&obj[0..4], &[0x7f, b'E', b'L', b'F']);
        // ET_REL = 1 (e_type at offset 16)
        assert_eq!(obj[16], 1);
        // EM_X86_64 = 62 (e_machine at offset 18)
        assert_eq!(obj[18], 62);
    }

    #[test]
    fn test_non_x86_64_is_unimplemented() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(0))));
        program.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::AArch64,
            os: TargetOS::Linux,
            format: OutputFormat::ObjectFile,
            ..Default::default()
        };
        let mut backend = NativeBackend::new(config);
        assert!(backend.generate_from_lir(&program).is_err());
    }
}
