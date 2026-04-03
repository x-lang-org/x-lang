//! 代码生成核心接口
//!
//! 这个 crate 定义了所有后端共享的接口和数据结构。
//! 具体的后端实现在独立的 x-codegen-* crate 中。

use std::path::PathBuf;
pub use x_hir;
pub use x_lir;
pub use x_mir;
use x_parser::ast::Program as AstProgram;

pub mod error;
pub mod target;
pub mod utils;
pub mod xir;

pub use error::{CodeGenError, CodeGenResult};
pub use target::{FileType, OutputFormat, Target};
pub use utils::{CodeBuffer, OperatorConfig, escape_assembly_string, escape_string, generate_header_with_version, headers, GENERATOR_NAME};
pub use xir::*;

/// 代码生成配置
#[derive(Debug, PartialEq, Clone)]
pub struct CodeGenConfig {
    /// 目标平台
    pub target: Target,
    /// 输出目录
    pub output_dir: Option<PathBuf>,
    /// 输出格式
    pub output_format: OutputFormat,
    /// 是否启用优化
    pub optimize: bool,
    /// 是否生成调试信息
    pub debug_info: bool,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        Self {
            target: Target::Native,
            output_dir: None,
            output_format: OutputFormat::default(),
            optimize: false,
            debug_info: true,
        }
    }
}

/// 代码生成输出
#[derive(Debug)]
pub struct CodegenOutput {
    /// 生成的文件列表
    pub files: Vec<OutputFile>,
    /// 依赖项列表
    pub dependencies: Vec<String>,
}

/// 单个输出文件
#[derive(Debug)]
pub struct OutputFile {
    /// 文件路径
    pub path: PathBuf,
    /// 文件内容
    pub content: Vec<u8>,
    /// 文件类型
    pub file_type: FileType,
}

/// 代码生成器 trait - 所有后端都要实现这个 trait
pub trait CodeGenerator {
    type Config;
    type Error;

    /// 创建新的代码生成器
    fn new(config: Self::Config) -> Self;

    /// 从 AST 生成代码（初级接口，用于向后兼容）
    fn generate_from_ast(&mut self, program: &AstProgram) -> Result<CodegenOutput, Self::Error>;

    /// 从 HIR 生成代码（高级接口）
    fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error>;

    /// 从 LIR 生成代码（后端统一正式输入）
    fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, Self::Error>;
}

/// 动态代码生成器 trait（用于类型擦除）
pub trait DynamicCodeGenerator: 'static {
    fn generate_from_ast(&mut self, program: &AstProgram) -> CodeGenResult<CodegenOutput>;

    /// Generate code from LIR (optional, default returns error)
    fn generate_from_lir(&mut self, _lir: &x_lir::Program) -> CodeGenResult<CodegenOutput> {
        Err(CodeGenError::UnsupportedFeature(
            "LIR generation not implemented for this backend".to_string(),
        ))
    }
}

// ============================================================================
// 十大后端概览
// ============================================================================
//
// 后端实现位于独立的 crate 中：
//
// 1. x-codegen-zig        - Zig 后端（Native/Wasm），通过 Zig 编译器
// 2. x-codegen-typescript - TypeScript 后端，浏览器/Node.js/Deno
// 3. x-codegen-python     - Python 后端，生成 Python 源码
// 4. x-codegen-rust       - Rust 后端，生成 Rust 源码
// 5. x-codegen-java       - Java 后端，生成 Java 源码
// 6. x-codegen-csharp     - C# 后端，.NET 平台
// 7. x-codegen-llvm       - LLVM 后端，直接生成 LLVM IR
// 8. x-codegen-swift      - Swift 后端，Apple 生态
// 9. x-codegen-erlang     - Erlang 后端，并发/分布式系统
// 10. x-codegen-native    - Native 后端，LIR 直译机器码
//
// 使用方式：
// ```rust
// use x_codegen::CodeGenerator;
// use x_codegen_zig::{ZigBackend, ZigBackendConfig};
//
// let config = ZigBackendConfig::default();
// let mut backend = ZigBackend::new(config);
// let output = backend.generate_from_lir(&lir)?;
// ```
