//! Native 后端 - 汇编生成与机器码发射
//!
//! 生成汇编代码，然后通过外部汇编器或直接编码转换为机器码。
//! 支持多种目标架构：x86_64, AArch64, RISC-V, Wasm32
//!
//! # 架构概述
//!
//! ```text
//! LIR → AssemblyGenerator → Assembly Text → Assembler → Object/Binary
//!                                    ↓
//!                              (optional) Direct Encoding
//! ```
//!
//! # 支持的架构
//!
//! - **x86_64**: System V AMD64 ABI (Linux/macOS), Microsoft x64 (Windows)
//! - **AArch64**: ARM64 架构（Apple Silicon, AWS Graviton）
//! - **RISC-V**: RV64 架构
//! - **Wasm32**: WebAssembly MVP + reference-types
//!
//! # 目标版本 (2026)
//!
//! - x86_64: AVX-512, AMX 支持
//! - AArch64: ARMv9.5-A + SVE/SVE2/SVE3
//! - RISC-V: RVA23U64 Profile
//! - Wasm32: WebAssembly 2.0 + WasmGC
//!
//! # 示例
//!
//! ```ignore
//! use x_codegen_asm::{NativeBackend, NativeBackendConfig, TargetArch};
//!
//! let config = NativeBackendConfig {
//!     arch: TargetArch::X86_64,
//!     format: OutputFormat::Assembly,
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
use x_codegen::{escape_assembly_string, CodeGenerator, CodegenOutput, FileType, OutputFile};

// ============================================================================
// 公共接口
// ============================================================================

pub mod arch;
pub mod assembler;
pub mod assembly;
pub mod emitter;
pub mod encoding;

pub use arch::{Instruction, MemoryOperand, Register, TargetArch};
pub use assembler::{
    create_assembler, Assembler, AssemblerConfig, DirectEncoder, ExternalAssembler,
};
pub use assembly::{create_generator, AssemblyGenerator, X86_64AssemblyGenerator};
pub use emitter::{BinaryEmitter, BinaryFormat};
pub use encoding::MachineCodeEncoder;

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

    /// 使用 assembly 模块生成代码
    fn generate_from_lir_impl(
        &mut self,
        lir: &x_lir::Program,
    ) -> Result<CodegenOutput, NativeError> {
        // 使用 AssemblyGenerator 生成汇编
        let mut generator = create_generator(self.config.arch, self.config.os);
        let asm_output = generator.generate(lir)?;

        // 根据输出格式处理
        match self.config.format {
            OutputFormat::Assembly => {
                // 直接返回汇编文本
                let extension = generator.extension();
                let output_file = OutputFile {
                    path: PathBuf::from(format!("output.{}", extension)),
                    content: asm_output.as_bytes().to_vec(),
                    file_type: FileType::Assembly,
                };

                Ok(CodegenOutput {
                    files: vec![output_file],
                    dependencies: vec![],
                })
            }
            OutputFormat::ObjectFile => {
                // 使用汇编器生成目标文件
                use crate::assembler::{create_assembler, AssemblerConfig};
                let config = AssemblerConfig::for_os(self.config.os);
                let assembler = create_assembler(self.config.arch, self.config.os, config);

                // 创建临时输出路径
                let output_path = std::env::temp_dir().join("output.o");
                assembler.assemble(&asm_output, &output_path)?;

                // 读取目标文件
                let object_data = std::fs::read(&output_path)?;
                let _ = std::fs::remove_file(&output_path); // 清理临时文件

                let output_file = OutputFile {
                    path: PathBuf::from("output.o"),
                    content: object_data,
                    file_type: FileType::ObjectFile,
                };

                Ok(CodegenOutput {
                    files: vec![output_file],
                    dependencies: vec![],
                })
            }
            OutputFormat::Executable => {
                // Windows: 使用内置 MASM + Microsoft 链接器生成 PE 可执行文件。
                #[cfg(windows)]
                {
                    use crate::assembler::{
                        create_assembler, AssemblerConfig, LinkerConfig, MicrosoftLinker,
                    };
                    use std::env;
                    use std::path::PathBuf;

                    let asm_config = AssemblerConfig::for_os(self.config.os);
                    let assembler = create_assembler(self.config.arch, self.config.os, asm_config);

                    let temp_obj = env::temp_dir().join("x_native_output.obj");
                    assembler.assemble(&asm_output, &temp_obj)?;

                    let output_path = PathBuf::from("output.exe");

                    if MicrosoftLinker::is_available() {
                        let linker_config = LinkerConfig::default();
                        let linker = MicrosoftLinker::new(linker_config);
                        linker.link(&[&temp_obj], &output_path)?;
                    }

                    let _ = std::fs::remove_file(&temp_obj);

                    let exe_data = std::fs::read(&output_path).unwrap_or_default();

                    let output_file = OutputFile {
                        path: output_path,
                        content: exe_data,
                        file_type: FileType::Executable,
                    };

                    Ok(CodegenOutput {
                        files: vec![output_file],
                        dependencies: vec![],
                    })
                }

                // macOS/Linux：返回汇编文本，由 x-cli 用 clang 汇编并链接
                #[cfg(not(windows))]
                {
                    let extension = generator.extension();
                    let output_file = OutputFile {
                        path: PathBuf::from(format!("output.{}", extension)),
                        content: asm_output.as_bytes().to_vec(),
                        file_type: FileType::Assembly,
                    };

                    Ok(CodegenOutput {
                        files: vec![output_file],
                        dependencies: vec![],
                    })
                }
            }
            OutputFormat::RawBinary => {
                // 原始二进制直接返回汇编
                let extension = generator.extension();
                let output_file = OutputFile {
                    path: PathBuf::from(format!("output.{}", extension)),
                    content: asm_output.as_bytes().to_vec(),
                    file_type: FileType::Assembly,
                };

                Ok(CodegenOutput {
                    files: vec![output_file],
                    dependencies: vec![],
                })
            }
        }
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
    use x_codegen::escape_assembly_string;
    use x_lir::{self as lir, Declaration, Expression, Statement, Type};

    #[test]
    fn test_config_default() {
        let config = NativeBackendConfig::default();
        assert_eq!(config.arch, TargetArch::X86_64);
        assert_eq!(config.format, OutputFormat::Executable);
        assert_eq!(config.os, TargetOS::Linux);
    }

    #[test]
    fn test_target_arch_default() {
        let arch = TargetArch::default();
        assert_eq!(arch, TargetArch::X86_64);
    }

    #[test]
    fn test_target_os_triple() {
        let os = TargetOS::Linux;
        assert_eq!(
            os.target_triple(TargetArch::X86_64),
            "x86_64-unknown-linux-gnu"
        );

        let os = TargetOS::MacOS;
        assert_eq!(
            os.target_triple(TargetArch::AArch64),
            "aarch64-apple-darwin"
        );

        let os = TargetOS::Windows;
        assert_eq!(
            os.target_triple(TargetArch::X86_64),
            "x86_64-pc-windows-msvc"
        );
    }

    #[test]
    fn test_target_os_abi() {
        assert!(TargetOS::Linux.uses_system_v_abi());
        assert!(TargetOS::MacOS.uses_system_v_abi());
        assert!(TargetOS::Windows.uses_microsoft_abi());
        assert!(!TargetOS::Linux.uses_microsoft_abi());
    }

    #[test]
    fn test_simple_function() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(42))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.files.len(), 1);
        // x86_64 uses .asm extension (NASM syntax)
        assert!(output.files[0].path.extension().unwrap() == "asm");
    }

    #[test]
    fn test_x86_64_return_value() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(123))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("mov rax, 123"));
    }

    #[test]
    fn test_x86_64_binary_add() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("add_test", Type::Int);
        func.body.statements.push(Statement::Return(Some(
            Expression::int(10).add(Expression::int(20)),
        )));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("add rax, rcx"));
    }

    #[test]
    fn test_x86_64_function_call() {
        let mut lir = lir::Program::new();

        // 被调用函数
        let mut callee = lir::Function::new("helper", Type::Int);
        callee
            .body
            .statements
            .push(Statement::Return(Some(Expression::int(100))));
        lir.add(lir::Declaration::Function(callee));

        // 主函数
        let mut main_func = lir::Function::new("main", Type::Int);
        main_func.body.statements.push(Statement::Return(Some(
            Expression::var("helper").call(vec![]),
        )));
        lir.add(lir::Declaration::Function(main_func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("call helper"));
    }

    #[test]
    fn test_aarch64_function() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(42))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::AArch64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("stp x29, x30"));
        assert!(content.contains("ret"));
    }

    #[test]
    fn test_riscv_function() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(42))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::RiscV64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("addi sp, sp"));
        assert!(content.contains("ret"));
    }

    #[test]
    fn test_wasm_function() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(42))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("(module"));
        // LIR Int 映射为 Wasm i32
        assert!(content.contains("i32.const 42"));
        assert!(
            content.contains("(local $ret_val i32)"),
            "Wasm lowering must declare scratch locals used by return/temps: {content}"
        );
        assert!(
            content.contains("local.get $ret_val"),
            "WAT 具名局部须带 $ 前缀: {content}"
        );
    }

    #[test]
    fn test_wasm_if_br_if_follows_eqz() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body.statements.push(Statement::if_(
            Expression::int(1),
            Statement::return_(Some(Expression::int(10))),
            Some(Statement::return_(Some(Expression::int(20)))),
        ));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();
        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        let pos_eqz = content
            .find("i32.eqz")
            .expect("if lowering should emit i32.eqz");
        let pos_br_if = content
            .find("br_if")
            .expect("if lowering should emit br_if");
        assert!(
            pos_eqz < pos_br_if,
            "br_if jumps on non-zero; if(cond) uses eqz so false goes to else: {content}"
        );
        assert!(
            content.contains("(block $L_if_merge_"),
            "if 应生成 WAT 嵌套 block，而非汇编式 `L_:` 标号: {content}"
        );
    }

    #[test]
    fn test_wasm_generate_resets_label_counter_each_module() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body.statements.push(Statement::if_(
            Expression::int(1),
            Statement::return_(Some(Expression::int(0))),
            None,
        ));
        lir.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let a = backend.generate_from_lir(&lir).unwrap();
        let b = backend.generate_from_lir(&lir).unwrap();
        let sa = String::from_utf8(a.files[0].content.clone()).unwrap();
        let sb = String::from_utf8(b.files[0].content.clone()).unwrap();
        assert!(
            sa.contains("$L_if_merge_0") && sb.contains("$L_if_merge_0"),
            "每次 generate 应重置 label_counter，使标号从 0 重新计数"
        );
    }

    #[test]
    fn test_wasm_second_module_clears_field_offsets() {
        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };
        let mut backend = NativeBackend::new(config);

        let mut p1 = lir::Program::new();
        p1.add(Declaration::Struct(lir::Struct {
            name: "A".into(),
            fields: vec![lir::Field {
                name: "x".into(),
                type_: Type::Int,
            }],
        }));
        let mut f1 = lir::Function::new("main", Type::Int);
        f1.body
            .statements
            .push(Statement::return_(Some(Expression::int(0))));
        p1.add(Declaration::Function(f1));
        backend.generate_from_lir(&p1).unwrap();

        let mut p2 = lir::Program::new();
        p2.add(Declaration::Struct(lir::Struct {
            name: "B".into(),
            fields: vec![
                lir::Field {
                    name: "pad".into(),
                    type_: Type::Int,
                },
                lir::Field {
                    name: "x".into(),
                    type_: Type::Int,
                },
            ],
        }));
        let mut f2 = lir::Function::new("main", Type::Int)
            .param("p", Type::Pointer(Box::new(Type::Named("B".into()))));
        f2.body
            .statements
            .push(Statement::return_(Some(Expression::PointerMember(
                Box::new(Expression::var("p")),
                "x".into(),
            ))));
        p2.add(Declaration::Function(f2));

        let out = backend.generate_from_lir(&p2).unwrap();
        let content = String::from_utf8(out.files[0].content.clone()).unwrap();
        assert!(
            content.contains("i32.const 4"),
            "第二次 generate 须清空 field_offsets，否则同名字段 x 会沿用上一模块偏移 0: {content}"
        );
    }

    /// 同一模块内 A.x 与 B.x 并存时，须按参数类型解析为 `B::x`（偏移 4），不能误用 `A::x`（0）。
    #[test]
    fn test_wasm_same_module_two_structs_same_field_name() {
        let mut program = lir::Program::new();
        program.add(Declaration::Struct(lir::Struct {
            name: "A".into(),
            fields: vec![lir::Field {
                name: "x".into(),
                type_: Type::Int,
            }],
        }));
        program.add(Declaration::Struct(lir::Struct {
            name: "B".into(),
            fields: vec![
                lir::Field {
                    name: "pad".into(),
                    type_: Type::Int,
                },
                lir::Field {
                    name: "x".into(),
                    type_: Type::Int,
                },
            ],
        }));
        let mut func = lir::Function::new("main", Type::Int)
            .param("pb", Type::Pointer(Box::new(Type::Named("B".into()))));
        func.body
            .statements
            .push(Statement::return_(Some(Expression::PointerMember(
                Box::new(Expression::var("pb")),
                "x".into(),
            ))));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };
        let mut backend = NativeBackend::new(config);
        let out = backend.generate_from_lir(&program).unwrap();
        let content = String::from_utf8(out.files[0].content.clone()).unwrap();
        assert!(
            content.contains("i32.const 4"),
            "应按 `B::x` 生成字段偏移 4: {content}"
        );
        assert!(
            !content.contains(";; TODO: field offset not found"),
            "不应因同名字段回退失败: {content}"
        );
    }

    #[test]
    fn test_wasm_sequential_string_data_offsets() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Expression(Expression::string("x")));
        func.body
            .statements
            .push(Statement::Expression(Expression::string("yy")));
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(0))));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&program).unwrap();
        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(
            content.contains("(data (i32.const 0)"),
            "first string at 0: {content}"
        );
        assert!(
            content.contains("(data (i32.const 4)"),
            "after 1-byte string, next offset aligned to 4: {content}"
        );
    }

    #[test]
    fn test_wasm_global_data_after_strings() {
        let mut program = lir::Program::new();
        program.add(Declaration::Global(lir::GlobalVar {
            name: "g".into(),
            type_: Type::Int,
            initializer: None,
            is_static: true,
        }));
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Expression(Expression::string("x")));
        func.body
            .statements
            .push(Statement::Return(Some(Expression::var("g"))));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&program).unwrap();
        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(
            content.contains(";; Global `g` @4"),
            "one 1-byte string padded to 4, then global int at 4: {content}"
        );
        assert!(
            content.contains("i32.const 4") && content.contains("i32.load"),
            "reading global `g` must load from its linear memory offset: {content}"
        );
    }

    #[test]
    fn test_wasm_data_string_wat_byte_escapes() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::string("a\"b"))));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&program).unwrap();
        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(
            content.contains("(data (i32.const 0) \"\\61\\22\\62\")"),
            "WAT data must use \\\\hh for each byte (a, quote, b): {content}"
        );
        assert!(
            content.contains("i32.const 0"),
            "string pointer must be a numeric linear memory offset: {content}"
        );
        assert!(
            !content.contains("@addr"),
            "invalid placeholder WAT must not appear: {content}"
        );
    }

    #[test]
    fn test_wasm_second_string_literal_uses_next_data_offset() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Expression(Expression::string("x")));
        func.body
            .statements
            .push(Statement::Return(Some(Expression::string("yy"))));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::Wasm32,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&program).unwrap();
        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(
            content.contains("i32.const 4"),
            "second string starts after 1-byte + align 4: {content}"
        );
        assert!(
            !content.contains("@addr"),
            "invalid placeholder WAT must not appear: {content}"
        );
    }

    #[test]
    fn test_windows_calling_convention() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::int(1))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            os: TargetOS::Windows,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("; Target: x86_64-pc-windows-msvc"));
    }

    #[test]
    fn test_if_statement() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test_if", Type::Int);

        let if_stmt = Statement::if_(
            Expression::int(1),
            Statement::return_(Some(Expression::int(10))),
            Some(Statement::return_(Some(Expression::int(20)))),
        );
        func.body.statements.push(if_stmt);
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("test rax, rax"));
        assert!(content.contains("jz"));
        assert!(content.contains("jmp"));
    }

    #[test]
    fn test_while_statement() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test_while", Type::Int);

        let while_stmt = Statement::while_(
            Expression::int(0),
            Statement::return_(Some(Expression::int(1))),
        );
        func.body.statements.push(while_stmt);
        func.body
            .statements
            .push(Statement::return_(Some(Expression::int(0))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains("jmp"));
    }

    #[test]
    fn test_string_literal() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test_str", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::string("hello"))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        // NASM syntax uses "section .rodata" (without leading dot)
        assert!(content.contains("section .rodata"));
        assert!(content.contains("db"));
    }

    #[test]
    fn test_escape_assembly_string() {
        assert_eq!(escape_assembly_string("hello"), "hello");
        assert_eq!(escape_assembly_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_assembly_string("tab\there"), "tab\\there");
        assert_eq!(escape_assembly_string("quote\"test"), "quote\\\"test");
        assert_eq!(escape_assembly_string("back\\slash"), "back\\\\slash");
    }
}
