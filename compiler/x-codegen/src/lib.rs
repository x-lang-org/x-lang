// 代码生成公共接口和抽象层
// 这个 crate 定义了所有后端共享的接口

use std::path::PathBuf;
use x_parser::ast::Program;
pub use x_hir;
pub use x_perceus;

pub mod error;
pub mod lower;
pub mod xir;

pub mod csharp_backend;
pub mod java_backend;
pub mod python_backend;
pub mod rust_backend;
pub mod target;
pub mod typescript_backend;
pub mod zig_backend;

pub use error::{CodeGenError, CodeGenResult};
pub use target::{FileType, Target};
pub use xir::*;

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
            target: Target::Native,
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
    fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error>;

    /// 从 PerceusIR 生成代码（最终目标）
    fn generate_from_pir(&mut self, pir: &x_perceus::PerceusIR) -> Result<CodegenOutput, Self::Error>;
}

/// 获取指定目标的代码生成器
pub fn get_code_generator(
    target: Target,
    config: CodeGenConfig,
) -> CodeGenResult<Box<dyn DynamicCodeGenerator>> {
    match target {
        Target::Native | Target::Wasm => {
            let zig_target = match target {
                Target::Wasm => zig_backend::ZigTarget::Wasm32Wasi,
                _ => zig_backend::ZigTarget::Native,
            };
            return Ok(Box::new(zig_backend::ZigBackend::new(
                zig_backend::ZigBackendConfig {
                    output_dir: config.output_dir,
                    optimize: config.optimize,
                    debug_info: config.debug_info,
                    target: zig_target,
                },
            )));
        }
        Target::Jvm => {
            return Ok(Box::new(java_backend::JavaBackend::new(
                java_backend::JavaBackendConfig {
                    output_dir: config.output_dir,
                    optimize: config.optimize,
                    debug_info: config.debug_info,
                },
            )));
        }
        Target::DotNet => {
            return Ok(Box::new(csharp_backend::CSharpBackend::new(
                csharp_backend::CSharpBackendConfig {
                    output_dir: config.output_dir,
                    optimize: config.optimize,
                    debug_info: config.debug_info,
                },
            )));
        }
        Target::TypeScript => {
            return Ok(Box::new(typescript_backend::TypeScriptBackend::new(
                typescript_backend::TypeScriptBackendConfig {
                    output_dir: config.output_dir,
                    optimize: config.optimize,
                    debug_info: config.debug_info,
                },
            )));
        }
        Target::Python => {
            return Ok(Box::new(python_backend::PythonBackend::new(
                python_backend::PythonBackendConfig {
                    output_dir: config.output_dir,
                    optimize: config.optimize,
                    debug_info: config.debug_info,
                },
            )));
        }
        Target::Rust => {
            return Ok(Box::new(rust_backend::RustBackend::new(
                rust_backend::RustBackendConfig {
                    output_dir: config.output_dir,
                    optimize: config.optimize,
                    debug_info: config.debug_info,
                },
            )));
        }
    }
}

/// 动态代码生成器 trait（用于类型擦除）
pub trait DynamicCodeGenerator {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput>;
}

impl DynamicCodeGenerator for zig_backend::ZigBackend {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput> {
        self.generate_from_ast(program)
            .map_err(|e| CodeGenError::GenerationError(format!("Zig backend error: {:?}", e)))
    }
}

impl DynamicCodeGenerator for python_backend::PythonBackend {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput> {
        self.generate_from_ast(program)
            .map_err(|e| CodeGenError::GenerationError(format!("Python backend error: {:?}", e)))
    }
}

impl DynamicCodeGenerator for java_backend::JavaBackend {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput> {
        self.generate_from_ast(program)
            .map_err(|e| CodeGenError::GenerationError(format!("Java backend error: {:?}", e)))
    }
}

impl DynamicCodeGenerator for csharp_backend::CSharpBackend {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput> {
        self.generate_from_ast(program)
            .map_err(|e| CodeGenError::GenerationError(format!("C# backend error: {:?}", e)))
    }
}

impl DynamicCodeGenerator for typescript_backend::TypeScriptBackend {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput> {
        self.generate_from_ast(program).map_err(|e| {
            CodeGenError::GenerationError(format!("TypeScript backend error: {:?}", e))
        })
    }
}

impl DynamicCodeGenerator for rust_backend::RustBackend {
    fn generate_from_ast(&mut self, program: &Program) -> CodeGenResult<CodegenOutput> {
        self.generate_from_ast(program)
            .map_err(|e| CodeGenError::GenerationError(format!("Rust backend error: {:?}", e)))
    }
}

// 下面的是临时占位符，实际应该在各个 x-codegen-* crate 中实现

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
}
