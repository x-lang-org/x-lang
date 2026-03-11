// .NET CIL生成器 - .NET后端
// 这个 crate 实现了 x-codegen 的 CodeGenerator trait

use std::path::PathBuf;
use x_codegen::{CodeGenResult, CodeGenerator, CodegenOutput};
use x_parser::ast::Program;

/// .NET代码生成器配置
#[derive(Debug, Clone)]
pub struct DotNetConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl Default for DotNetConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
        }
    }
}

/// .NET代码生成器
pub struct DotNetCodeGenerator {
    config: DotNetConfig,
}

impl CodeGenerator for DotNetCodeGenerator {
    type Config = DotNetConfig;
    type Error = DotNetCodeGenError;

    fn new(config: Self::Config) -> Self {
        Self { config }
    }

    fn generate_from_ast(&mut self, _program: &Program) -> Result<CodegenOutput, Self::Error> {
        Err(DotNetCodeGenError::Unimplemented(
            ".NET backend not yet implemented".to_string(),
        ))
    }

    fn generate_from_hir(&mut self, _hir: &()) -> Result<CodegenOutput, Self::Error> {
        Err(DotNetCodeGenError::Unimplemented(
            ".NET backend not yet implemented".to_string(),
        ))
    }

    fn generate_from_pir(&mut self, _pir: &()) -> Result<CodegenOutput, Self::Error> {
        Err(DotNetCodeGenError::Unimplemented(
            ".NET backend not yet implemented".to_string(),
        ))
    }
}

/// .NET代码生成错误
#[derive(thiserror::Error, Debug)]
pub enum DotNetCodeGenError {
    #[error("代码生成错误: {0}")]
    GenerationError(String),

    #[error("未实现: {0}")]
    Unimplemented(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<x_codegen::CodeGenError> for DotNetCodeGenError {
    fn from(err: x_codegen::CodeGenError) -> Self {
        match err {
            x_codegen::CodeGenError::GenerationError(msg) => {
                DotNetCodeGenError::GenerationError(msg)
            }
            x_codegen::CodeGenError::IoError(e) => DotNetCodeGenError::IoError(e),
            _ => DotNetCodeGenError::GenerationError(format!("{:?}", err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x_codegen::CodeGenerator;

    #[test]
    fn test_config_default() {
        let config = DotNetConfig::default();
        assert!(config.output_dir.is_none());
        assert!(!config.optimize);
        assert!(config.debug_info);
    }

    #[test]
    fn test_config_custom() {
        let config = DotNetConfig {
            output_dir: Some(PathBuf::from("/tmp/output")),
            optimize: true,
            debug_info: false,
        };
        assert_eq!(config.output_dir, Some(PathBuf::from("/tmp/output")));
        assert!(config.optimize);
        assert!(!config.debug_info);
    }

    #[test]
    fn test_generator_new() {
        let config = DotNetConfig::default();
        let _generator = DotNetCodeGenerator::new(config);
    }

    #[test]
    fn test_generate_from_ast_unimplemented() {
        use x_parser::ast::Program;
        let config = DotNetConfig::default();
        let mut generator = DotNetCodeGenerator::new(config);
        let program = Program {
            declarations: vec![],
            statements: vec![],
        };
        let result = generator.generate_from_ast(&program);
        assert!(matches!(result, Err(DotNetCodeGenError::Unimplemented(_))));
    }

    #[test]
    fn test_error_display() {
        let err = DotNetCodeGenError::GenerationError("test error".to_string());
        assert_eq!(err.to_string(), "代码生成错误: test error");

        let err = DotNetCodeGenError::Unimplemented("not implemented".to_string());
        assert_eq!(err.to_string(), "未实现: not implemented");
    }
}
