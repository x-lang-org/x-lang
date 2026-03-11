// JVM字节码生成器 - JVM后端
// 这个 crate 实现了 x-codegen 的 CodeGenerator trait

use std::path::PathBuf;
use x_codegen::{CodeGenResult, CodeGenerator, CodegenOutput, Target};
use x_parser::ast::Program;

/// JVM代码生成器配置
#[derive(Debug, Clone)]
pub struct JvmConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl Default for JvmConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
        }
    }
}

/// JVM代码生成器
pub struct JvmCodeGenerator {
    config: JvmConfig,
}

impl CodeGenerator for JvmCodeGenerator {
    type Config = JvmConfig;
    type Error = JvmCodeGenError;

    fn new(config: Self::Config) -> Self {
        Self { config }
    }

    fn generate_from_ast(&mut self, _program: &Program) -> Result<CodegenOutput, Self::Error> {
        Err(JvmCodeGenError::Unimplemented(
            "JVM backend not yet implemented".to_string(),
        ))
    }

    fn generate_from_hir(&mut self, _hir: &()) -> Result<CodegenOutput, Self::Error> {
        Err(JvmCodeGenError::Unimplemented(
            "JVM backend not yet implemented".to_string(),
        ))
    }

    fn generate_from_pir(&mut self, _pir: &()) -> Result<CodegenOutput, Self::Error> {
        Err(JvmCodeGenError::Unimplemented(
            "JVM backend not yet implemented".to_string(),
        ))
    }
}

/// JVM代码生成错误
#[derive(thiserror::Error, Debug)]
pub enum JvmCodeGenError {
    #[error("代码生成错误: {0}")]
    GenerationError(String),

    #[error("未实现: {0}")]
    Unimplemented(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<x_codegen::CodeGenError> for JvmCodeGenError {
    fn from(err: x_codegen::CodeGenError) -> Self {
        match err {
            x_codegen::CodeGenError::GenerationError(msg) => JvmCodeGenError::GenerationError(msg),
            x_codegen::CodeGenError::IoError(e) => JvmCodeGenError::IoError(e),
            _ => JvmCodeGenError::GenerationError(format!("{:?}", err)),
        }
    }
}
