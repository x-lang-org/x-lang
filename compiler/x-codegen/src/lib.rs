// 代码生成公共接口和抽象层
// 这个 crate 定义了所有后端共享的接口

use std::path::PathBuf;
use x_parser::ast::Program;

pub mod error;
pub mod xir;
pub mod lower;
#[cfg(feature = "llvm")]
pub mod llvm_lower;
pub mod c_backend;
pub mod zig_backend;
pub mod target;

pub use error::{CodeGenError, CodeGenResult};
pub use c_backend::{CBackend, CBackendConfig, CCompiler, CStandard, CResult};
pub use zig_backend::{ZigBackend, ZigBackendConfig, ZigBackendError, ZigResult};
pub use xir::*;
pub use target::{Target, FileType};

/// 代码生成配置
#[derive(Debug, PartialEq, Clone)]
pub struct CodeGenConfig {
    /// 目标平台
    pub target: Target,
    /// 输出目录
    pub output_dir: Option<PathBuf>,
    /// 是否启用优化
    pub optimize: bool,
    /// 是否生成调试信息
    pub debug_info: bool,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        Self {
            target: Target::Zig,
            output_dir: None,
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
    fn generate_from_ast(&mut self, program: &Program) -> Result<CodegenOutput, Self::Error>;

    /// 从 HIR 生成代码（高级接口）
    fn generate_from_hir(&mut self, hir: &()) -> Result<CodegenOutput, Self::Error>;

    /// 从 PerceusIR 生成代码（最终目标）
    fn generate_from_pir(&mut self, pir: &()) -> Result<CodegenOutput, Self::Error>;
}

/// 获取指定目标的代码生成器
pub fn get_code_generator(target: Target, config: CodeGenConfig) -> CodeGenResult<Box<dyn DynamicCodeGenerator>> {
    match target {
        Target::Native | Target::Wasm | Target::LlvmIr => {
            #[cfg(feature = "llvm")]
            {
                return Ok(Box::new(LlvmCodeGenerator::new(LlvmConfig {
                    target: match target {
                        Target::Native => LlvmTarget::Native,
                        Target::Wasm => LlvmTarget::Wasm,
                        Target::LlvmIr => LlvmTarget::LlvmIr,
                        _ => unreachable!(),
                    },
                    output_dir: config.output_dir,
                    optimize: config.optimize,
                    debug_info: config.debug_info,
                })));
            }
            #[cfg(not(feature = "llvm"))]
            return Err(CodeGenError::UnsupportedFeature(
                "LLVM backend not enabled. Build with --features llvm.".to_string(),
            ));
        }
        Target::Jvm => {
            #[cfg(feature = "jvm")]
            return Ok(Box::new(JvmCodeGenerator::new(JvmConfig {
                output_dir: config.output_dir,
                optimize: config.optimize,
                debug_info: config.debug_info,
            })));
            #[cfg(not(feature = "jvm"))]
            return Err(CodeGenError::UnsupportedFeature(
                "JVM backend not enabled. Build with --features jvm.".to_string(),
            ));
        }
        Target::DotNet => {
            #[cfg(feature = "dotnet")]
            return Ok(Box::new(DotNetCodeGenerator::new(DotNetConfig {
                output_dir: config.output_dir,
                optimize: config.optimize,
                debug_info: config.debug_info,
            })));
            #[cfg(not(feature = "dotnet"))]
            return Err(CodeGenError::UnsupportedFeature(
                ".NET backend not enabled. Build with --features dotnet.".to_string(),
            ));
        }
        Target::JavaScript | Target::TypeScript => {
            #[cfg(feature = "js")]
            return Ok(Box::new(JavaScriptCodeGenerator::new(JavaScriptConfig {
                output_dir: config.output_dir,
                optimize: config.optimize,
                debug_info: config.debug_info,
                target_language: match target {
                    Target::JavaScript => TargetLanguage::JavaScript,
                    Target::TypeScript => TargetLanguage::TypeScript,
                    _ => unreachable!(),
                },
            })));
            #[cfg(not(feature = "js"))]
            return Err(CodeGenError::UnsupportedFeature(
                "JavaScript backend not enabled. Build with --features js.".to_string(),
            ));
        }
        Target::Pyc | Target::Python => {
            #[cfg(feature = "python")]
            return Ok(Box::new(PythonCodeGenerator::new(PythonConfig {
                output_dir: config.output_dir,
                optimize: config.optimize,
                debug_info: config.debug_info,
                output_format: match target {
                    Target::Pyc => PythonOutputFormat::Bytecode,
                    Target::Python => PythonOutputFormat::Source,
                    _ => unreachable!(),
                },
            })));
            #[cfg(not(feature = "python"))]
            return Err(CodeGenError::UnsupportedFeature(
                "Python backend not enabled. Build with --features python.".to_string(),
            ));
        }
        Target::Zig => {
            return Ok(Box::new(ZigCodeGenerator::new(ZigConfig {
                output_dir: config.output_dir,
                optimize: config.optimize,
                debug_info: config.debug_info,
            })));
        }
    }
}

/// 动态代码生成器 trait（用于类型擦除）
pub trait DynamicCodeGenerator {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput>;
}

// 下面的是临时占位符，实际应该在各个 x-codegen-* crate 中实现

#[cfg(feature = "llvm")]
pub struct LlvmCodeGenerator;

#[cfg(feature = "llvm")]
#[derive(Debug, Clone)]
pub struct LlvmConfig {
    pub target: LlvmTarget,
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

#[cfg(feature = "llvm")]
#[derive(Debug, PartialEq, Clone)]
pub enum LlvmTarget {
    Native,
    Wasm,
    LlvmIr,
}

#[cfg(feature = "jvm")]
pub struct JvmCodeGenerator;

#[cfg(feature = "jvm")]
#[derive(Debug, Clone)]
pub struct JvmConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

#[cfg(feature = "dotnet")]
pub struct DotNetCodeGenerator;

#[cfg(feature = "dotnet")]
#[derive(Debug, Clone)]
pub struct DotNetConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

#[cfg(feature = "js")]
pub struct JavaScriptCodeGenerator;

#[cfg(feature = "js")]
#[derive(Debug, Clone)]
pub struct JavaScriptConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub target_language: TargetLanguage,
}

#[cfg(feature = "js")]
#[derive(Debug, PartialEq, Clone)]
pub enum TargetLanguage {
    JavaScript,
    TypeScript,
}

pub struct ZigCodeGenerator {
    backend: ZigBackend,
}

#[derive(Debug, Clone)]
pub struct ZigConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl CodeGenerator for ZigCodeGenerator {
    type Config = ZigConfig;
    type Error = ZigBackendError;

    fn new(config: Self::Config) -> Self {
        let backend_config = ZigBackendConfig {
            output_dir: config.output_dir,
            optimize: config.optimize,
            debug_info: config.debug_info,
        };
        Self {
            backend: ZigBackend::new(backend_config),
        }
    }

    fn generate_from_ast(&mut self, program: &Program) -> Result<CodegenOutput, Self::Error> {
        let zig_code = self.backend.generate_from_ast(program)?;
        let files = vec![OutputFile {
            path: PathBuf::from("output.zig"),
            content: zig_code.into_bytes(),
            file_type: FileType::ZigSource,
        }];
        Ok(CodegenOutput {
            files,
            dependencies: vec![],
        })
    }

    fn generate_from_hir(&mut self, _hir: &()) -> Result<CodegenOutput, Self::Error> {
        Err(ZigBackendError::UnsupportedFeature("HIR generation not supported yet".to_string()))
    }

    fn generate_from_pir(&mut self, _pir: &()) -> Result<CodegenOutput, Self::Error> {
        Err(ZigBackendError::UnsupportedFeature("PIR generation not supported yet".to_string()))
    }
}

impl DynamicCodeGenerator for ZigCodeGenerator {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput> {
        CodeGenerator::generate_from_ast(self, program).map_err(|e| CodeGenError::GenerationError(e.to_string()))
    }
}
