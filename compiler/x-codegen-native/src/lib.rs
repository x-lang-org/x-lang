//! Native 后端 - LIR 直译机器码
//!
//! 无需外部编译器，直接生成可执行机器码。
//! 支持多种目标架构：x86_64, AArch64, RISC-V, Wasm32
//!
//! # 架构概述
//!
//! ```text
//! LIR → NativeCodegen → Assembly/OptimizedIR → BinaryEmitter → Executable/Object
//! ```
//!
//! # 支持的架构
//!
//! - **x86_64**: System V AMD64 ABI (Linux/macOS), Microsoft x64 (Windows)
//! - **AArch64**: ARM64 架构
//! - **RISC-V**: RV64 架构
//! - **Wasm32**: WebAssembly
//!
//! # 示例
//!
//! ```ignore
//! use x_codegen_native::{NativeBackend, NativeBackendConfig, TargetArch};
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

use std::collections::HashMap;
use std::fmt::{self, Display, Write};
use std::path::PathBuf;
use x_codegen::{CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::{self as lir, BinaryOp, Declaration, Expression, Literal, Statement, Type, UnaryOp};
use x_parser::ast::Program as AstProgram;

// ============================================================================
// 公共接口
// ============================================================================

pub mod arch;
pub mod emitter;
pub mod encoding;

pub use arch::{Instruction, MemoryOperand, Register, TargetArch};
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
pub struct NativeBackend {
    config: NativeBackendConfig,
    /// 汇编输出缓冲区
    asm_output: String,
    /// 机器码输出缓冲区
    code_output: Vec<u8>,
    /// 当前缩进级别
    indent: usize,
    /// 标签计数器
    label_counter: usize,
    /// 字符串字面量表
    string_literals: HashMap<String, String>,
    /// 全局变量表
    globals: HashMap<String, GlobalInfo>,
    /// 函数表
    functions: HashMap<String, FunctionInfo>,
    /// 当前函数的栈帧大小
    stack_size: usize,
    /// 局部变量栈偏移
    local_offsets: HashMap<String, i32>,
}

/// 全局变量信息
#[derive(Debug, Clone)]
struct GlobalInfo {
    /// 类型大小（字节）
    size: usize,
    /// 是否已初始化
    initialized: bool,
    /// 对齐要求
    align: usize,
}

/// 函数信息
#[derive(Debug, Clone)]
struct FunctionInfo {
    /// 参数数量
    param_count: usize,
    /// 局部变量数量
    local_count: usize,
    /// 栈帧大小
    stack_frame_size: usize,
}

impl NativeBackend {
    /// 创建新的 Native 后端
    pub fn new(config: NativeBackendConfig) -> Self {
        Self {
            config,
            asm_output: String::new(),
            code_output: Vec::new(),
            indent: 0,
            label_counter: 0,
            string_literals: HashMap::new(),
            globals: HashMap::new(),
            functions: HashMap::new(),
            stack_size: 0,
            local_offsets: HashMap::new(),
        }
    }

    /// 生成唯一标签名
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!(".L{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// 输出一行汇编
    fn emit_line(&mut self, line: &str) -> NativeResult<()> {
        writeln!(self.asm_output, "{}{}", "    ".repeat(self.indent), line)?;
        Ok(())
    }

    /// 输出原始指令（无缩进）
    fn emit_raw(&mut self, text: &str) -> NativeResult<()> {
        writeln!(self.asm_output, "{}", text)?;
        Ok(())
    }

    /// 生成汇编头部
    fn emit_header(&mut self) -> NativeResult<()> {
        match self.config.arch {
            TargetArch::X86_64 => {
                if self.config.os == TargetOS::Windows {
                    self.emit_raw("; Generated by X Compiler Native Backend")?;
                    self.emit_raw("; Target: x86_64-pc-windows-msvc")?;
                } else {
                    self.emit_raw("# Generated by X Compiler Native Backend")?;
                    self.emit_raw("# Target: x86_64-unknown-linux-gnu")?;
                }
            }
            TargetArch::AArch64 => {
                self.emit_raw("// Generated by X Compiler Native Backend")?;
                self.emit_raw("// Target: aarch64")?;
            }
            TargetArch::RiscV64 => {
                self.emit_raw("# Generated by X Compiler Native Backend")?;
                self.emit_raw("# Target: riscv64")?;
            }
            TargetArch::Wasm32 => {
                self.emit_raw(";; Generated by X Compiler Native Backend")?;
                self.emit_raw(";; Target: wasm32")?;
            }
        }
        self.emit_raw("")
    }

    /// 生成数据段
    fn emit_data_section(&mut self) -> NativeResult<()> {
        if self.string_literals.is_empty() && self.globals.is_empty() {
            return Ok(());
        }

        // 克隆数据以避免借用冲突（emit_raw 需要 &mut self）
        let string_literals: Vec<(String, String)> = self
            .string_literals
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let globals: Vec<(String, GlobalInfo)> = self
            .globals
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        match self.config.arch {
            TargetArch::X86_64 => {
                self.emit_raw(".section .rodata")?;
                for (literal, label) in &string_literals {
                    self.emit_raw(&format!("{}:", label))?;
                    self.emit_raw(&format!("    .asciz \"{}\"", escape_string(literal)))?;
                }
                self.emit_raw("")?;

                // 全局变量
                if !globals.is_empty() {
                    self.emit_raw(".section .data")?;
                    for (name, info) in &globals {
                        self.emit_raw(&format!("{}:", name))?;
                        self.emit_raw(&format!("    .zero {}", info.size))?;
                    }
                    self.emit_raw("")?;
                }
            }
            TargetArch::AArch64 => {
                self.emit_raw(".section .rodata")?;
                for (literal, label) in &string_literals {
                    self.emit_raw(&format!("{}:", label))?;
                    self.emit_raw(&format!("    .asciz \"{}\"", escape_string(literal)))?;
                }
            }
            TargetArch::RiscV64 => {
                self.emit_raw(".section .rodata")?;
                for (literal, label) in &string_literals {
                    self.emit_raw(&format!("{}:", label))?;
                    self.emit_raw(&format!("    .asciz \"{}\"", escape_string(literal)))?;
                }
            }
            TargetArch::Wasm32 => {
                // Wasm 数据段会在模块中处理
            }
        }
        Ok(())
    }

    /// 从 LIR 生成代码
    fn generate_lir(&mut self, lir: &lir::Program) -> NativeResult<()> {
        self.asm_output.clear();
        self.code_output.clear();
        self.label_counter = 0;
        self.string_literals.clear();
        self.globals.clear();
        self.functions.clear();

        // 第一遍：收集所有字符串字面量和全局变量
        self.collect_literals_and_globals(lir)?;

        // 生成头部
        self.emit_header()?;

        // 生成数据段
        self.emit_data_section()?;

        // 生成代码段
        self.emit_text_section(lir)?;

        Ok(())
    }

    /// 收集字符串字面量和全局变量
    fn collect_literals_and_globals(&mut self, lir: &lir::Program) -> NativeResult<()> {
        for decl in &lir.declarations {
            match decl {
                Declaration::Global(global) => {
                    let size = self.type_size(&global.type_);
                    self.globals.insert(
                        global.name.clone(),
                        GlobalInfo {
                            size,
                            initialized: global.initializer.is_some(),
                            align: self.type_align(&global.type_),
                        },
                    );
                }
                Declaration::Function(func) => {
                    // 收集函数中的字符串字面量
                    self.collect_string_literals(&func.body)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 从语句块收集字符串字面量
    fn collect_string_literals(&mut self, block: &lir::Block) -> NativeResult<()> {
        for stmt in &block.statements {
            self.collect_stmt_strings(stmt)?;
        }
        Ok(())
    }

    /// 从语句收集字符串字面量
    fn collect_stmt_strings(&mut self, stmt: &Statement) -> NativeResult<()> {
        match stmt {
            Statement::Expression(expr) => self.collect_expr_strings(expr),
            Statement::Variable(var) => {
                if let Some(init) = &var.initializer {
                    self.collect_expr_strings(init)
                } else {
                    Ok(())
                }
            }
            Statement::If(if_stmt) => {
                self.collect_expr_strings(&if_stmt.condition)?;
                self.collect_stmt_strings(&if_stmt.then_branch)?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.collect_stmt_strings(else_branch)?;
                }
                Ok(())
            }
            Statement::While(while_stmt) => {
                self.collect_expr_strings(&while_stmt.condition)?;
                self.collect_stmt_strings(&while_stmt.body)
            }
            Statement::For(for_stmt) => {
                if let Some(cond) = &for_stmt.condition {
                    self.collect_expr_strings(cond)?;
                }
                if let Some(inc) = &for_stmt.increment {
                    self.collect_expr_strings(inc)?;
                }
                self.collect_stmt_strings(&for_stmt.body)
            }
            Statement::Return(Some(expr)) => self.collect_expr_strings(expr),
            Statement::Compound(block) => self.collect_string_literals(block),
            _ => Ok(()),
        }
    }

    /// 从表达式收集字符串字面量
    fn collect_expr_strings(&mut self, expr: &Expression) -> NativeResult<()> {
        match expr {
            Expression::Literal(Literal::String(s)) => {
                if !self.string_literals.contains_key(s) {
                    let label = format!(".LC{}", self.string_literals.len());
                    self.string_literals.insert(s.clone(), label);
                }
                Ok(())
            }
            Expression::Unary(_, e) => self.collect_expr_strings(e),
            Expression::Binary(_, left, right) => {
                self.collect_expr_strings(left)?;
                self.collect_expr_strings(right)
            }
            Expression::Ternary(cond, then_e, else_e) => {
                self.collect_expr_strings(cond)?;
                self.collect_expr_strings(then_e)?;
                self.collect_expr_strings(else_e)
            }
            Expression::Call(func, args) => {
                self.collect_expr_strings(func)?;
                for arg in args {
                    self.collect_expr_strings(arg)?;
                }
                Ok(())
            }
            Expression::Index(arr, idx) => {
                self.collect_expr_strings(arr)?;
                self.collect_expr_strings(idx)
            }
            Expression::Member(obj, _) => self.collect_expr_strings(obj),
            Expression::PointerMember(obj, _) => self.collect_expr_strings(obj),
            Expression::AddressOf(e) => self.collect_expr_strings(e),
            Expression::Dereference(e) => self.collect_expr_strings(e),
            Expression::Cast(_, e) => self.collect_expr_strings(e),
            Expression::Assign(target, val) => {
                self.collect_expr_strings(target)?;
                self.collect_expr_strings(val)
            }
            Expression::AssignOp(_, target, val) => {
                self.collect_expr_strings(target)?;
                self.collect_expr_strings(val)
            }
            Expression::InitializerList(items) => {
                for item in items {
                    if let lir::Initializer::Expression(e) = item {
                        self.collect_expr_strings(e)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// 生成代码段
    fn emit_text_section(&mut self, lir: &lir::Program) -> NativeResult<()> {
        match self.config.arch {
            TargetArch::X86_64 => {
                self.emit_raw(".section .text")?;
            }
            TargetArch::AArch64 => {
                self.emit_raw(".section .text")?;
            }
            TargetArch::RiscV64 => {
                self.emit_raw(".section .text")?;
            }
            TargetArch::Wasm32 => {
                self.emit_raw("(module")?;
                self.indent += 1;
            }
        }

        // 生成函数
        for decl in &lir.declarations {
            if let Declaration::Function(func) = decl {
                self.emit_function(func)?;
            }
        }

        if self.config.arch == TargetArch::Wasm32 {
            self.indent -= 1;
            self.emit_raw(")")?;
        }

        Ok(())
    }

    /// 生成函数
    fn emit_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        self.stack_size = 0;
        self.local_offsets.clear();

        match self.config.arch {
            TargetArch::X86_64 => self.emit_x86_64_function(func),
            TargetArch::AArch64 => self.emit_aarch64_function(func),
            TargetArch::RiscV64 => self.emit_riscv_function(func),
            TargetArch::Wasm32 => self.emit_wasm_function(func),
        }
    }

    /// 生成 x86_64 函数
    fn emit_x86_64_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        // 函数标签
        self.emit_raw(&format!(".globl {}", func.name))?;
        self.emit_raw(&format!("{}:", func.name))?;

        // 函数序言
        self.emit_line("push rbp")?;
        self.emit_line("mov rbp, rsp")?;

        // 分配栈空间（先假设 0，后面会更新）
        let stack_alloc_label = self.new_label("stack_alloc");
        self.emit_line(&format!("sub rsp, 0  ; {}", stack_alloc_label))?;

        // 保存被调用者保存的寄存器
        let callee_saved = vec!["rbx", "r12", "r13", "r14", "r15"];
        for reg in &callee_saved {
            self.emit_line(&format!("push {}", reg))?;
        }

        // 处理参数
        let arg_regs = if self.config.os.uses_system_v_abi() {
            vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"]
        } else {
            // Windows x64 calling convention
            vec!["rcx", "rdx", "r8", "r9"]
        };

        for (i, param) in func.parameters.iter().enumerate() {
            if i < arg_regs.len() {
                // 参数在寄存器中
                self.local_offsets.insert(param.name.clone(), ((i + 1) * 8) as i32);
                self.emit_line(&format!("mov [rbp-{}], {}", (i + 1) * 8, arg_regs[i]))?;
            } else {
                // 参数在栈中
                let offset = ((i - arg_regs.len() + 2) * 8) as i32;
                self.local_offsets.insert(param.name.clone(), offset);
            }
        }

        // 生成函数体
        self.emit_block(&func.body)?;

        // 函数尾声（如果有返回语句，可能已经跳转到这里）
        let epilogue_label = format!(".L{}_epilogue", func.name);
        self.emit_raw(&format!("{}:", epilogue_label))?;

        // 恢复被调用者保存的寄存器
        for reg in callee_saved.iter().rev() {
            self.emit_line(&format!("pop {}", reg))?;
        }

        // 恢复栈指针
        self.emit_line("mov rsp, rbp")?;
        self.emit_line("pop rbp")?;
        self.emit_line("ret")?;
        self.emit_raw("")?;

        Ok(())
    }

    /// 生成 AArch64 函数
    fn emit_aarch64_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        self.emit_raw(&format!(".globl _{}", func.name))?;
        self.emit_raw(&format!("_{}:", func.name))?;

        // 函数序言
        self.emit_line("stp x29, x30, [sp, #-16]!")?;
        self.emit_line("mov x29, sp")?;

        // 分配栈空间
        self.emit_line("sub sp, sp, #32")?;

        // 参数寄存器: x0-x7
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 8 {
                self.local_offsets.insert(param.name.clone(), ((i + 1) * 8) as i32);
                self.emit_line(&format!("str x{}, [x29, #-{}]", i, (i + 1) * 8))?;
            }
        }

        // 生成函数体
        self.emit_block(&func.body)?;

        // 函数尾声
        self.emit_line("add sp, sp, #32")?;
        self.emit_line("ldp x29, x30, [sp], #16")?;
        self.emit_line("ret")?;
        self.emit_raw("")?;

        Ok(())
    }

    /// 生成 RISC-V 函数
    fn emit_riscv_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        self.emit_raw(&format!(".globl {}", func.name))?;
        self.emit_raw(&format!("{}:", func.name))?;

        // 函数序言
        self.emit_line("addi sp, sp, -32")?;
        self.emit_line("sd ra, 24(sp)")?;
        self.emit_line("sd s0, 16(sp)")?;
        self.emit_line("addi s0, sp, 32")?;

        // 参数寄存器: a0-a7
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 8 {
                self.local_offsets.insert(param.name.clone(), ((i + 1) * 8) as i32);
                self.emit_line(&format!("sd a{}, -{}(s0)", i, (i + 1) * 8))?;
            }
        }

        // 生成函数体
        self.emit_block(&func.body)?;

        // 函数尾声
        self.emit_line("ld ra, 24(sp)")?;
        self.emit_line("ld s0, 16(sp)")?;
        self.emit_line("addi sp, sp, 32")?;
        self.emit_line("ret")?;
        self.emit_raw("")?;

        Ok(())
    }

    /// 生成 Wasm 函数
    fn emit_wasm_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        // 计算局部变量数量
        let local_count = func.body.statements.iter().filter(|s| matches!(s, Statement::Variable(_))).count();

        self.emit_raw(&format!("(func ${}", func.name))?;
        self.indent += 1;

        // 参数
        for param in &func.parameters {
            self.emit_raw(&format!("(param ${} i64)", param.name))?;
        }

        // 返回类型
        self.emit_raw("(result i64)")?;

        // 局部变量
        if local_count > 0 {
            self.emit_raw(&format!("(local {} i64)", local_count))?;
        }

        // 生成函数体
        self.emit_wasm_block(&func.body)?;

        self.indent -= 1;
        self.emit_raw(")")?;

        // 导出函数
        self.emit_raw(&format!("(export \"{}\" (func ${}))", func.name, func.name))?;

        Ok(())
    }

    /// 生成语句块
    fn emit_block(&mut self, block: &lir::Block) -> NativeResult<()> {
        for stmt in &block.statements {
            self.emit_statement(stmt)?;
        }
        Ok(())
    }

    /// 生成语句
    fn emit_statement(&mut self, stmt: &Statement) -> NativeResult<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.emit_expr(expr)?;
            }
            Statement::Variable(var) => {
                self.stack_size += 8;
                let offset = self.stack_size as i32;
                self.local_offsets.insert(var.name.clone(), offset);

                if let Some(init) = &var.initializer {
                    self.emit_expr(init)?;
                    self.emit_line(&format!("mov [rbp-{}], rax", offset))?;
                } else {
                    self.emit_line(&format!("mov qword [rbp-{}], 0", offset))?;
                }
            }
            Statement::Return(Some(expr)) => {
                self.emit_expr(expr)?;
                self.emit_line("mov rsp, rbp")?;
                self.emit_line("pop rbp")?;
                self.emit_line("ret")?;
            }
            Statement::Return(None) => {
                self.emit_line("xor rax, rax")?;
                self.emit_line("mov rsp, rbp")?;
                self.emit_line("pop rbp")?;
                self.emit_line("ret")?;
            }
            Statement::If(if_stmt) => {
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");

                self.emit_expr(&if_stmt.condition)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jz {}", else_label))?;

                self.emit_statement(&if_stmt.then_branch)?;
                self.emit_line(&format!("jmp {}", end_label))?;

                self.emit_raw(&format!("{}:", else_label))?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.emit_statement(else_branch)?;
                }
                self.emit_raw(&format!("{}:", end_label))?;
            }
            Statement::While(while_stmt) => {
                let start_label = self.new_label("while_start");
                let end_label = self.new_label("while_end");

                self.emit_raw(&format!("{}:", start_label))?;
                self.emit_expr(&while_stmt.condition)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jz {}", end_label))?;

                self.emit_statement(&while_stmt.body)?;
                self.emit_line(&format!("jmp {}", start_label))?;

                self.emit_raw(&format!("{}:", end_label))?;
            }
            Statement::For(for_stmt) => {
                let start_label = self.new_label("for_start");
                let end_label = self.new_label("for_end");

                // 初始化
                if let Some(init) = &for_stmt.initializer {
                    self.emit_statement(init)?;
                }

                self.emit_raw(&format!("{}:", start_label))?;

                // 条件
                if let Some(cond) = &for_stmt.condition {
                    self.emit_expr(cond)?;
                    self.emit_line("test rax, rax")?;
                    self.emit_line(&format!("jz {}", end_label))?;
                }

                // 循环体
                self.emit_statement(&for_stmt.body)?;

                // 增量
                if let Some(inc) = &for_stmt.increment {
                    self.emit_expr(inc)?;
                }

                self.emit_line(&format!("jmp {}", start_label))?;
                self.emit_raw(&format!("{}:", end_label))?;
            }
            Statement::Break => {
                // TODO: 需要跟踪循环结束标签
                self.emit_line("; break - placeholder")?;
            }
            Statement::Continue => {
                // TODO: 需要跟踪循环开始标签
                self.emit_line("; continue - placeholder")?;
            }
            Statement::Compound(block) => {
                self.emit_block(block)?;
            }
            Statement::Empty => {}
            Statement::Label(label) => {
                self.emit_raw(&format!("{}:", label))?;
            }
            Statement::Goto(label) => {
                self.emit_line(&format!("jmp {}", label))?;
            }
            Statement::Switch(_) | Statement::Match(_) | Statement::Try(_) => {
                // 复杂控制流，需要更详细的实现
                self.emit_line("; TODO: switch/match/try not fully implemented")?;
            }
            Statement::DoWhile(do_while) => {
                let start_label = self.new_label("do_while_start");
                let end_label = self.new_label("do_while_end");

                self.emit_raw(&format!("{}:", start_label))?;
                self.emit_statement(&do_while.body)?;
                self.emit_expr(&do_while.condition)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jnz {}", start_label))?;
                self.emit_raw(&format!("{}:", end_label))?;
            }
            Statement::Declaration(_) => {}
        }
        Ok(())
    }

    /// 生成表达式
    fn emit_expr(&mut self, expr: &Expression) -> NativeResult<()> {
        match expr {
            Expression::Literal(lit) => self.emit_literal(lit)?,
            Expression::Variable(name) => {
                if let Some(offset) = self.local_offsets.get(name) {
                    self.emit_line(&format!("mov rax, [rbp-{}]", offset))?;
                } else if self.globals.contains_key(name) {
                    self.emit_line(&format!("mov rax, [{}]", name))?;
                } else {
                    return Err(NativeError::CodegenError(format!(
                        "Undefined variable: {}",
                        name
                    )));
                }
            }
            Expression::Unary(op, e) => self.emit_unary(*op, e)?,
            Expression::Binary(op, left, right) => self.emit_binary(*op, left, right)?,
            Expression::Call(func, args) => self.emit_call(func, args)?,
            Expression::Assign(target, value) => {
                self.emit_expr(value)?;
                match target.as_ref() {
                    Expression::Variable(name) => {
                        if let Some(offset) = self.local_offsets.get(name) {
                            self.emit_line(&format!("mov [rbp-{}], rax", offset))?;
                        } else if self.globals.contains_key(name) {
                            self.emit_line(&format!("mov [{}], rax", name))?;
                        }
                    }
                    Expression::Dereference(ptr) => {
                        self.emit_expr(ptr)?;
                        self.emit_line("mov [rax], rdx")?;
                        // 重新加载结果到 rax
                        self.emit_expr(value)?;
                    }
                    _ => {
                        return Err(NativeError::InvalidOperand(
                            "Invalid assignment target".to_string(),
                        ));
                    }
                }
            }
            Expression::AddressOf(e) => {
                match e.as_ref() {
                    Expression::Variable(name) => {
                        if let Some(offset) = self.local_offsets.get(name) {
                            self.emit_line(&format!("lea rax, [rbp-{}]", offset))?;
                        }
                    }
                    _ => self.emit_expr(e)?,
                }
            }
            Expression::Dereference(e) => {
                self.emit_expr(e)?;
                self.emit_line("mov rax, [rax]")?;
            }
            Expression::Cast(_, e) => {
                self.emit_expr(e)?;
                // 类型转换可能需要额外的指令
            }
            Expression::Index(arr, idx) => {
                self.emit_expr(arr)?;
                self.emit_line("mov rcx, rax")?;
                self.emit_expr(idx)?;
                self.emit_line("shl rax, 3")?;
                self.emit_line("add rcx, rax")?;
                self.emit_line("mov rax, [rcx]")?;
            }
            Expression::Member(obj, name) => {
                self.emit_expr(obj)?;
                // 字段偏移计算
                let offset = 0; // TODO: 从结构体信息获取
                self.emit_line(&format!("add rax, {}", offset))?;
                self.emit_line("mov rax, [rax]")?;
                let _ = name; // 抑制未使用警告
            }
            Expression::PointerMember(obj, name) => {
                self.emit_expr(obj)?;
                self.emit_line("mov rax, [rax]")?;
                let offset = 0; // TODO: 从结构体信息获取
                if offset > 0 {
                    self.emit_line(&format!("add rax, {}", offset))?;
                }
                self.emit_line("mov rax, [rax]")?;
                let _ = name;
            }
            Expression::Ternary(cond, then_e, else_e) => {
                let else_label = self.new_label("ternary_else");
                let end_label = self.new_label("ternary_end");

                self.emit_expr(cond)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jz {}", else_label))?;

                self.emit_expr(then_e)?;
                self.emit_line(&format!("jmp {}", end_label))?;

                self.emit_raw(&format!("{}:", else_label))?;
                self.emit_expr(else_e)?;

                self.emit_raw(&format!("{}:", end_label))?;
            }
            Expression::SizeOf(ty) => {
                let size = self.type_size(ty);
                self.emit_line(&format!("mov rax, {}", size))?;
            }
            Expression::AlignOf(ty) => {
                let align = self.type_align(ty);
                self.emit_line(&format!("mov rax, {}", align))?;
            }
            Expression::AssignOp(op, target, value) => {
                // 先计算值
                self.emit_expr(value)?;
                self.emit_line("mov rcx, rax")?;
                // 加载目标
                match target.as_ref() {
                    Expression::Variable(name) => {
                        if let Some(offset) = self.local_offsets.get(name).copied() {
                            self.emit_line(&format!("mov rax, [rbp-{}]", offset))?;
                            // 执行操作
                            match op {
                                BinaryOp::Add => self.emit_line("add rax, rcx")?,
                                BinaryOp::Subtract => self.emit_line("sub rax, rcx")?,
                                BinaryOp::Multiply => self.emit_line("imul rax, rcx")?,
                                _ => {}
                            }
                            // 存回
                            self.emit_line(&format!("mov [rbp-{}], rax", offset))?;
                        }
                    }
                    _ => {}
                }
            }
            Expression::Comma(exprs) => {
                for expr in exprs {
                    self.emit_expr(expr)?;
                }
            }
            Expression::Parenthesized(e) => {
                self.emit_expr(e)?;
            }
            Expression::InitializerList(items) => {
                // 在栈上分配空间
                let size = items.len() * 8;
                self.emit_line(&format!("sub rsp, {}", size))?;
                for (i, item) in items.iter().enumerate() {
                    if let lir::Initializer::Expression(e) = item {
                        self.emit_expr(e)?;
                        self.emit_line(&format!("mov [rsp+{}], rax", i * 8))?;
                    }
                }
                self.emit_line("mov rax, rsp")?;
            }
            Expression::CompoundLiteral(_, items) => {
                // 类似 InitializerList
                let size = items.len() * 8;
                self.emit_line(&format!("sub rsp, {}", size))?;
                for (i, item) in items.iter().enumerate() {
                    if let lir::Initializer::Expression(e) = item {
                        self.emit_expr(e)?;
                        self.emit_line(&format!("mov [rsp+{}], rax", i * 8))?;
                    }
                }
                self.emit_line("mov rax, rsp")?;
            }
            Expression::SizeOfExpr(e) => {
                // 计算表达式类型的大小
                let _ = e;
                self.emit_line("mov rax, 8")?;
            }
        }
        Ok(())
    }

    /// 生成字面量
    fn emit_literal(&mut self, lit: &Literal) -> NativeResult<()> {
        match lit {
            Literal::Integer(n) => {
                self.emit_line(&format!("mov rax, {}", n))?;
            }
            Literal::UnsignedInteger(n) => {
                self.emit_line(&format!("mov rax, {}", n))?;
            }
            Literal::Long(n) => {
                self.emit_line(&format!("mov rax, {}", n))?;
            }
            Literal::UnsignedLong(n) => {
                self.emit_line(&format!("mov rax, {}", n))?;
            }
            Literal::LongLong(n) => {
                self.emit_line(&format!("mov rax, {}", n))?;
            }
            Literal::UnsignedLongLong(n) => {
                self.emit_line(&format!("mov rax, {}", n))?;
            }
            Literal::Float(n) => {
                // 浮点数需要特殊处理
                self.emit_line(&format!("mov eax, __float32__({:?})", n))?;
            }
            Literal::Double(n) => {
                self.emit_line(&format!("mov rax, __float64__({:?})", n))?;
            }
            Literal::Char(c) => {
                self.emit_line(&format!("mov rax, {}", *c as i32))?;
            }
            Literal::String(s) => {
                if let Some(label) = self.string_literals.get(s) {
                    self.emit_line(&format!("lea rax, [{}]", label))?;
                }
            }
            Literal::Bool(b) => {
                self.emit_line(&format!("mov rax, {}", if *b { 1 } else { 0 }))?;
            }
            Literal::NullPointer => {
                self.emit_line("xor rax, rax")?;
            }
        }
        Ok(())
    }

    /// 生成一元运算
    fn emit_unary(&mut self, op: UnaryOp, expr: &Expression) -> NativeResult<()> {
        self.emit_expr(expr)?;
        match op {
            UnaryOp::Minus => {
                self.emit_line("neg rax")?;
            }
            UnaryOp::Not => {
                self.emit_line("test rax, rax")?;
                self.emit_line("setz al")?;
                self.emit_line("movzx rax, al")?;
            }
            UnaryOp::BitNot => {
                self.emit_line("not rax")?;
            }
            UnaryOp::PreIncrement => {
                self.emit_line("inc rax")?;
                // 如果是变量，需要存回
            }
            UnaryOp::PreDecrement => {
                self.emit_line("dec rax")?;
            }
            UnaryOp::PostIncrement => {
                self.emit_line("mov rcx, rax")?;
                self.emit_line("inc rax")?;
                self.emit_line("mov rax, rcx")?;
            }
            UnaryOp::PostDecrement => {
                self.emit_line("mov rcx, rax")?;
                self.emit_line("dec rax")?;
                self.emit_line("mov rax, rcx")?;
            }
            UnaryOp::Plus => {}
        }
        Ok(())
    }

    /// 生成二元运算
    fn emit_binary(
        &mut self,
        op: BinaryOp,
        left: &Expression,
        right: &Expression,
    ) -> NativeResult<()> {
        // 特殊处理短路运算
        match op {
            BinaryOp::LogicalAnd => {
                let false_label = self.new_label("and_false");
                let end_label = self.new_label("and_end");

                self.emit_expr(left)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jz {}", false_label))?;

                self.emit_expr(right)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jz {}", false_label))?;

                self.emit_line("mov rax, 1")?;
                self.emit_line(&format!("jmp {}", end_label))?;

                self.emit_raw(&format!("{}:", false_label))?;
                self.emit_line("xor rax, rax")?;

                self.emit_raw(&format!("{}:", end_label))?;
                return Ok(());
            }
            BinaryOp::LogicalOr => {
                let true_label = self.new_label("or_true");
                let end_label = self.new_label("or_end");

                self.emit_expr(left)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jnz {}", true_label))?;

                self.emit_expr(right)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jnz {}", true_label))?;

                self.emit_line("xor rax, rax")?;
                self.emit_line(&format!("jmp {}", end_label))?;

                self.emit_raw(&format!("{}:", true_label))?;
                self.emit_line("mov rax, 1")?;

                self.emit_raw(&format!("{}:", end_label))?;
                return Ok(());
            }
            _ => {}
        }

        // 普通二元运算
        self.emit_expr(left)?;
        self.emit_line("push rax")?;
        self.emit_expr(right)?;
        self.emit_line("mov rcx, rax")?;
        self.emit_line("pop rax")?;

        match op {
            BinaryOp::Add => {
                self.emit_line("add rax, rcx")?;
            }
            BinaryOp::Subtract => {
                self.emit_line("sub rax, rcx")?;
            }
            BinaryOp::Multiply => {
                self.emit_line("imul rax, rcx")?;
            }
            BinaryOp::Divide => {
                self.emit_line("cqo")?;
                self.emit_line("idiv rcx")?;
            }
            BinaryOp::Modulo => {
                self.emit_line("cqo")?;
                self.emit_line("idiv rcx")?;
                self.emit_line("mov rax, rdx")?;
            }
            BinaryOp::LeftShift => {
                self.emit_line("mov rcx, rcx")?;
                self.emit_line("shl rax, cl")?;
            }
            BinaryOp::RightShift => {
                self.emit_line("sar rax, cl")?;
            }
            BinaryOp::LessThan => {
                self.emit_line("cmp rax, rcx")?;
                self.emit_line("setl al")?;
                self.emit_line("movzx rax, al")?;
            }
            BinaryOp::LessThanEqual => {
                self.emit_line("cmp rax, rcx")?;
                self.emit_line("setle al")?;
                self.emit_line("movzx rax, al")?;
            }
            BinaryOp::GreaterThan => {
                self.emit_line("cmp rax, rcx")?;
                self.emit_line("setg al")?;
                self.emit_line("movzx rax, al")?;
            }
            BinaryOp::GreaterThanEqual => {
                self.emit_line("cmp rax, rcx")?;
                self.emit_line("setge al")?;
                self.emit_line("movzx rax, al")?;
            }
            BinaryOp::Equal => {
                self.emit_line("cmp rax, rcx")?;
                self.emit_line("sete al")?;
                self.emit_line("movzx rax, al")?;
            }
            BinaryOp::NotEqual => {
                self.emit_line("cmp rax, rcx")?;
                self.emit_line("setne al")?;
                self.emit_line("movzx rax, al")?;
            }
            BinaryOp::BitAnd => {
                self.emit_line("and rax, rcx")?;
            }
            BinaryOp::BitOr => {
                self.emit_line("or rax, rcx")?;
            }
            BinaryOp::BitXor => {
                self.emit_line("xor rax, rcx")?;
            }
            _ => {}
        }
        Ok(())
    }

    /// 生成函数调用
    fn emit_call(&mut self, func: &Expression, args: &[Expression]) -> NativeResult<()> {
        // 保存参数
        let arg_regs = if self.config.os.uses_system_v_abi() {
            vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"]
        } else {
            vec!["rcx", "rdx", "r8", "r9"]
        };

        // Windows x64 需要为参数分配影子空间
        if self.config.os.uses_microsoft_abi() {
            self.emit_line("sub rsp, 32")?;
        }

        // 计算参数值并保存
        for (i, arg) in args.iter().enumerate() {
            self.emit_expr(arg)?;
            self.emit_line("push rax")?;
        }

        // 加载参数到寄存器
        for (i, arg) in args.iter().enumerate().rev() {
            self.emit_line("pop rax")?;
            if i < arg_regs.len() {
                self.emit_line(&format!("mov {}, rax", arg_regs[i]))?;
            } else {
                // 栈传递
                self.emit_line(&format!("push rax  ; arg {} on stack", i))?;
            }
            let _ = arg;
        }

        // 调用函数
        match func {
            Expression::Variable(name) => {
                self.emit_line(&format!("call {}", name))?;
            }
            _ => {
                // 通过函数指针调用
                self.emit_expr(func)?;
                self.emit_line("call rax")?;
            }
        }

        // 清理栈参数
        let stack_args = args.len().saturating_sub(arg_regs.len());
        if stack_args > 0 {
            self.emit_line(&format!("add rsp, {}", stack_args * 8))?;
        }

        // Windows x64 清理影子空间
        if self.config.os.uses_microsoft_abi() {
            self.emit_line("add rsp, 32")?;
        }

        Ok(())
    }

    /// 生成 Wasm 语句块
    fn emit_wasm_block(&mut self, block: &lir::Block) -> NativeResult<()> {
        self.emit_raw("(block")?;
        self.indent += 1;

        for stmt in &block.statements {
            self.emit_wasm_statement(stmt)?;
        }

        self.indent -= 1;
        self.emit_raw(")")?;
        Ok(())
    }

    /// 生成 Wasm 语句
    fn emit_wasm_statement(&mut self, stmt: &Statement) -> NativeResult<()> {
        match stmt {
            Statement::Expression(expr) => self.emit_wasm_expr(expr)?,
            Statement::Return(Some(expr)) => {
                self.emit_wasm_expr(expr)?;
            }
            Statement::Return(None) => {
                self.emit_raw("i64.const 0")?;
            }
            Statement::Variable(var) => {
                if let Some(init) = &var.initializer {
                    self.emit_wasm_expr(init)?;
                } else {
                    self.emit_raw("i64.const 0")?;
                }
            }
            Statement::If(if_stmt) => {
                self.emit_wasm_expr(&if_stmt.condition)?;
                self.emit_raw("(if")?;
                self.indent += 1;
                self.emit_raw("(then")?;
                self.indent += 1;
                self.emit_wasm_statement(&if_stmt.then_branch)?;
                self.indent -= 1;
                self.emit_raw(")")?;
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.emit_raw("(else")?;
                    self.indent += 1;
                    self.emit_wasm_statement(else_branch)?;
                    self.indent -= 1;
                    self.emit_raw(")")?;
                }
                self.indent -= 1;
                self.emit_raw(")")?;
            }
            Statement::Compound(block) => {
                for s in &block.statements {
                    self.emit_wasm_statement(s)?;
                }
            }
            _ => {
                // 其他语句类型
                self.emit_raw(&format!(";; TODO: {:?}", stmt))?;
            }
        }
        Ok(())
    }

    /// 生成 Wasm 表达式
    fn emit_wasm_expr(&mut self, expr: &Expression) -> NativeResult<()> {
        match expr {
            Expression::Literal(Literal::Integer(n)) => {
                self.emit_raw(&format!("i64.const {}", n))?;
            }
            Expression::Literal(Literal::Bool(b)) => {
                self.emit_raw(&format!("i64.const {}", if *b { 1 } else { 0 }))?;
            }
            Expression::Variable(name) => {
                self.emit_raw(&format!("local.get ${}", name))?;
            }
            Expression::Binary(op, left, right) => {
                self.emit_wasm_expr(left)?;
                self.emit_wasm_expr(right)?;
                match op {
                    BinaryOp::Add => self.emit_raw("i64.add")?,
                    BinaryOp::Subtract => self.emit_raw("i64.sub")?,
                    BinaryOp::Multiply => self.emit_raw("i64.mul")?,
                    BinaryOp::Divide => self.emit_raw("i64.div_s")?,
                    BinaryOp::LessThan => self.emit_raw("i64.lt_s")?,
                    BinaryOp::Equal => self.emit_raw("i64.eq")?,
                    _ => self.emit_raw(&format!(";; TODO: binary op {:?}", op))?,
                }
            }
            Expression::Call(func, args) => {
                for arg in args {
                    self.emit_wasm_expr(arg)?;
                }
                match func.as_ref() {
                    Expression::Variable(name) => {
                        self.emit_raw(&format!("call ${}", name))?;
                    }
                    _ => {}
                }
            }
            _ => {
                self.emit_raw(&format!(";; TODO: expr {:?}", expr))?;
            }
        }
        Ok(())
    }

    /// 获取类型大小（字节）
    fn type_size(&self, ty: &Type) -> usize {
        match ty {
            Type::Void => 0,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint | Type::Float => 4,
            Type::Long | Type::Ulong | Type::Double | Type::Pointer(_) => 8,
            Type::LongLong | Type::UlongLong | Type::LongDouble => 16,
            Type::Size | Type::Ptrdiff | Type::Intptr | Type::Uintptr => 8,
            Type::Array(inner, size) => {
                let inner_size = self.type_size(inner);
                size.map_or(inner_size, |s| inner_size * s as usize)
            }
            Type::Named(_) => 8, // 默认指针大小
            Type::Qualified(_, inner) => self.type_size(inner),
            Type::FunctionPointer(_, _) => 8,
        }
    }

    /// 获取类型对齐要求
    fn type_align(&self, ty: &Type) -> usize {
        match ty {
            Type::Void => 1,
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint | Type::Float => 4,
            Type::Long | Type::Ulong | Type::Double | Type::Pointer(_) => 8,
            Type::LongLong | Type::UlongLong | Type::LongDouble => 16,
            Type::Size | Type::Ptrdiff | Type::Intptr | Type::Uintptr => 8,
            Type::Array(inner, _) => self.type_align(inner),
            Type::Named(_) => 8,
            Type::Qualified(_, inner) => self.type_align(inner),
            Type::FunctionPointer(_, _) => 8,
        }
    }
}

impl CodeGenerator for NativeBackend {
    type Config = NativeBackendConfig;
    type Error = NativeError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_ast(&mut self, _program: &AstProgram) -> Result<CodegenOutput, Self::Error> {
        Err(NativeError::Unimplemented(
            "Native 后端尚未实现从 AST 生成，请从 LIR 生成".to_string(),
        ))
    }

    fn generate_from_hir(&mut self, _hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error> {
        Err(NativeError::Unimplemented(
            "Native 后端尚未实现从 HIR 生成，请从 LIR 生成".to_string(),
        ))
    }

    fn generate_from_lir(
        &mut self,
        lir: &x_lir::Program,
    ) -> Result<CodegenOutput, Self::Error> {
        self.generate_lir(lir)?;

        let file_type = match self.config.format {
            OutputFormat::Executable => FileType::Executable,
            OutputFormat::ObjectFile => FileType::ObjectFile,
            OutputFormat::Assembly => FileType::Assembly,
            OutputFormat::RawBinary => FileType::Executable,
        };

        let extension = match self.config.format {
            OutputFormat::Executable => "exe",
            OutputFormat::ObjectFile => "o",
            OutputFormat::Assembly => match self.config.arch {
                TargetArch::X86_64 => "s",
                TargetArch::AArch64 => "s",
                TargetArch::RiscV64 => "s",
                TargetArch::Wasm32 => "wat",
            },
            OutputFormat::RawBinary => "bin",
        };

        let output_file = OutputFile {
            path: PathBuf::from(format!("output.{}", extension)),
            content: self.asm_output.as_bytes().to_vec(),
            file_type,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 转义字符串用于汇编输出
fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            c if c.is_ascii() => result.push(c),
            c => result.push_str(&format!("\\x{:02x}", c as u32)),
        }
    }
    result
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
        func.body.statements.push(Statement::Return(Some(Expression::int(42))));
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
        assert!(output.files[0].path.extension().unwrap() == "s");
    }

    #[test]
    fn test_x86_64_return_value() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test", Type::Int);
        func.body.statements.push(Statement::Return(Some(Expression::int(123))));
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
        callee.body.statements.push(Statement::Return(Some(Expression::int(100))));
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
        func.body.statements.push(Statement::Return(Some(Expression::int(42))));
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
        func.body.statements.push(Statement::Return(Some(Expression::int(42))));
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
        func.body.statements.push(Statement::Return(Some(Expression::int(42))));
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
        assert!(content.contains("i64.const 42"));
    }

    #[test]
    fn test_windows_calling_convention() {
        let mut lir = lir::Program::new();
        let mut func = lir::Function::new("test", Type::Int);
        func.body.statements.push(Statement::Return(Some(Expression::int(1))));
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
        func.body.statements.push(Statement::return_(Some(Expression::int(0))));
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
        func.body.statements.push(Statement::Return(Some(Expression::string("hello"))));
        lir.add(lir::Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };

        let mut backend = NativeBackend::new(config);
        let result = backend.generate_from_lir(&lir).unwrap();

        let content = String::from_utf8(result.files[0].content.clone()).unwrap();
        assert!(content.contains(".section .rodata"));
        assert!(content.contains(".asciz"));
    }

    #[test]
    fn test_type_size() {
        let backend = NativeBackend::new(NativeBackendConfig::default());

        assert_eq!(backend.type_size(&Type::Void), 0);
        assert_eq!(backend.type_size(&Type::Bool), 1);
        assert_eq!(backend.type_size(&Type::Char), 1);
        assert_eq!(backend.type_size(&Type::Int), 4);
        assert_eq!(backend.type_size(&Type::Long), 8);
        assert_eq!(backend.type_size(&Type::Pointer(Box::new(Type::Int))), 8);
        assert_eq!(
            backend.type_size(&Type::Array(Box::new(Type::Int), Some(10))),
            40
        );
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_string("tab\there"), "tab\\there");
        assert_eq!(escape_string("quote\"test"), "quote\\\"test");
        assert_eq!(escape_string("back\\slash"), "back\\\\slash");
    }
}
