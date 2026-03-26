//! C# 后端 - 生成 C# 源代码
//!
//! 面向 .NET 平台，生成 C# 源代码

use std::path::PathBuf;
use x_codegen::{CodeGenerator, CodegenOutput, OutputFile, FileType};
use x_lir::Program as LirProgram;
use x_parser::ast::Program as AstProgram;

/// C# 后端配置
#[derive(Debug, Clone)]
pub struct CSharpConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub namespace: Option<String>,
}

impl Default for CSharpConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            namespace: None,
        }
    }
}

/// C# 后端
pub struct CSharpBackend {
    config: CSharpConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum CSharpError {
    #[error("C# 代码生成错误: {0}")]
    GenerationError(String),
    #[error("未实现: {0}")]
    Unimplemented(String),
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}

impl CSharpBackend {
    pub fn new(config: CSharpConfig) -> Self {
        Self { config }
    }
}

impl CodeGenerator for CSharpBackend {
    type Config = CSharpConfig;
    type Error = CSharpError;

    fn new(config: Self::Config) -> Self {
        Self { config }
    }

    fn generate_from_ast(&mut self, _program: &AstProgram) -> Result<CodegenOutput, Self::Error> {
        // TODO: 实现 AST -> C# 源码生成
        Err(CSharpError::Unimplemented("C# 后端尚未实现".to_string()))
    }

    fn generate_from_hir(&mut self, _hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error> {
        Err(CSharpError::Unimplemented("C# 后端尚未实现".to_string()))
    }

    fn generate_from_lir(&mut self, _lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        // TODO: 实现 LIR -> C# 源码生成
        Err(CSharpError::Unimplemented("C# 后端尚未实现".to_string()))
    }
}

// 保持向后兼容的别名
pub type DotNetCodeGenerator = CSharpBackend;
pub type DotNetConfig = CSharpConfig;
pub type DotNetCodeGenError = CSharpError;
pub type DotNetResult<T> = Result<T, CSharpError>;
