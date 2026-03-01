// 代码生成库：AST → LLVM IR / 目标文件

mod lower;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::targets::{FileType, RelocMode, CodeModel, Target as LlvmTarget, TargetMachine};
use x_parser::ast::Program;

pub use lower::generate_code;

#[derive(Debug, PartialEq, Clone)]
pub struct CodeGenConfig {
    pub target: CodegenTarget,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CodegenTarget {
    Native,
    Wasm,
    LlvmIr,
}

impl Default for CodeGenConfig {
    fn default() -> Self {
        Self { target: CodegenTarget::Native }
    }
}

/// 代码生成错误
#[derive(thiserror::Error, Debug)]
pub enum CodeGenError {
    #[error("代码生成错误: {0}")]
    GenerationError(String),
}

pub fn write_object_or_ir(
    module: &Module,
    config: &CodeGenConfig,
) -> Result<Vec<u8>, CodeGenError> {
    match config.target {
        CodegenTarget::LlvmIr => {
            let ir = module.to_string();
            Ok(ir.as_bytes().to_vec())
        }
        CodegenTarget::Native => {
            LlvmTarget::initialize_native(&inkwell::targets::InitializationConfig::default())
                .map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
            let target_triple = TargetMachine::get_default_triple();
            let cpu = TargetMachine::get_host_cpu_name().to_str().unwrap().to_string();
            let features = TargetMachine::get_host_cpu_features().to_str().unwrap().to_string();

            let target = LlvmTarget::from_triple(&target_triple)
                .map_err(|e| CodeGenError::GenerationError(e.to_string()))?;
            let target_machine = target
                .create_target_machine(
                    &target_triple,
                    &cpu,
                    &features,
                    inkwell::OptimizationLevel::Default,
                    RelocMode::Default,
                    CodeModel::Default,
                )
                .expect("create_target_machine");
            let object_code = target_machine
                .write_to_memory_buffer(module, FileType::Object)
                .map_err(|e: inkwell::support::LLVMString| CodeGenError::GenerationError(e.to_string()))?;
            Ok(object_code.as_slice().to_vec())
        }
        CodegenTarget::Wasm => {
            Err(CodeGenError::GenerationError("Wasm目标未实现".to_string()))
        }
    }
}
