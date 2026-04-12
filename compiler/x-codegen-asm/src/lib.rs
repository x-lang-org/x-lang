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

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use x_codegen::{escape_assembly_string, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::{
    self as lir, BinaryOp, Declaration, Expression, Function, Initializer, Literal, MatchStatement,
    Pattern, Statement, SwitchStatement, Type, UnaryOp,
};
use x_parser::ast::Program as AstProgram;

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
#[allow(dead_code)]
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
    /// 当前函数名（用于生成唯一标签）
    current_function: String,
    /// 循环 break 标签栈 - 支持嵌套循环 break
    loop_break_stack: Vec<String>,
    /// 循环 continue 标签栈 - 支持嵌套循环 continue
    loop_continue_stack: Vec<String>,
    /// 结构体字段偏移表: (结构体名称) -> (字段名称 -> 字节偏移)
    struct_field_offsets: HashMap<String, HashMap<String, usize>>,
    /// 当前函数参数与局部变量的静态类型（遗留 `emit_expr` 路径上解析 Member / PointerMember）
    local_and_param_types: HashMap<String, Type>,
    /// 导入的外部函数集合：(dll_name, function_name)
    imported_functions: Vec<(String, String)>,
}

/// 全局变量信息
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct GlobalInfo {
    /// 类型大小（字节）
    size: usize,
    /// 是否已初始化
    initialized: bool,
    /// 对齐要求
    align: usize,
    /// 初始化表达式
    initializer: Option<lir::Expression>,
}

/// 函数信息
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct FunctionInfo {
    /// 参数数量
    param_count: usize,
    /// 局部变量数量
    local_count: usize,
    /// 栈帧大小
    stack_frame_size: usize,
}

#[allow(dead_code)]
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
            current_function: String::new(),
            loop_break_stack: Vec::new(),
            loop_continue_stack: Vec::new(),
            struct_field_offsets: HashMap::new(),
            local_and_param_types: HashMap::new(),
            imported_functions: Vec::new(),
        }
    }

    fn init_local_and_param_types_for_function(&mut self, func: &Function) {
        self.local_and_param_types.clear();
        for p in &func.parameters {
            self.local_and_param_types
                .insert(p.name.clone(), p.type_.clone());
        }
        for stmt in &func.body.statements {
            Self::collect_native_var_types_stmt(stmt, &mut self.local_and_param_types);
        }
    }

    fn collect_native_var_types_stmt(stmt: &Statement, types: &mut HashMap<String, Type>) {
        match stmt {
            Statement::Variable(var) => {
                types.insert(var.name.clone(), var.type_.clone());
            }
            Statement::Compound(block) => {
                for s in &block.statements {
                    Self::collect_native_var_types_stmt(s, types);
                }
            }
            Statement::If(if_stmt) => {
                Self::collect_native_var_types_stmt(&*if_stmt.then_branch, types);
                if let Some(else_branch) = &if_stmt.else_branch {
                    Self::collect_native_var_types_stmt(&**else_branch, types);
                }
            }
            Statement::For(for_stmt) => {
                if let Some(init) = &for_stmt.initializer {
                    Self::collect_native_var_types_stmt(&**init, types);
                }
                Self::collect_native_var_types_stmt(&*for_stmt.body, types);
            }
            Statement::While(while_stmt) => {
                Self::collect_native_var_types_stmt(&*while_stmt.body, types);
            }
            Statement::DoWhile(do_while) => {
                Self::collect_native_var_types_stmt(&*do_while.body, types);
            }
            Statement::Switch(sw) => {
                for c in &sw.cases {
                    Self::collect_native_var_types_stmt(&c.body, types);
                }
                if let Some(def) = &sw.default {
                    Self::collect_native_var_types_stmt(&**def, types);
                }
            }
            Statement::Match(m) => {
                for case in &m.cases {
                    for s in &case.body.statements {
                        Self::collect_native_var_types_stmt(s, types);
                    }
                }
            }
            Statement::Try(t) => {
                for s in &t.body.statements {
                    Self::collect_native_var_types_stmt(s, types);
                }
                for c in &t.catch_clauses {
                    for s in &c.body.statements {
                        Self::collect_native_var_types_stmt(s, types);
                    }
                }
                if let Some(fin) = &t.finally_block {
                    for s in &fin.statements {
                        Self::collect_native_var_types_stmt(s, types);
                    }
                }
            }
            Statement::Declaration(Declaration::Function(f)) => {
                for s in &f.body.statements {
                    Self::collect_native_var_types_stmt(s, types);
                }
            }
            _ => {}
        }
    }

    fn peel_qualified_ty(ty: &Type) -> &Type {
        match ty {
            Type::Qualified(_, inner) => Self::peel_qualified_ty(inner),
            t => t,
        }
    }

    fn struct_name_from_pointer_type(ty: &Type) -> Option<String> {
        let ty = Self::peel_qualified_ty(ty);
        match ty {
            Type::Pointer(inner) => {
                let inner = Self::peel_qualified_ty(inner);
                if let Type::Named(s) = inner {
                    Some(s.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn struct_name_from_aggregate_type(ty: &Type) -> Option<String> {
        let ty = Self::peel_qualified_ty(ty);
        match ty {
            Type::Named(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn infer_pointee_struct_for_expr(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Variable(n) => self
                .local_and_param_types
                .get(n)
                .and_then(Self::struct_name_from_pointer_type),
            Expression::Cast(ty, e) => Self::struct_name_from_pointer_type(ty)
                .or_else(|| self.infer_pointee_struct_for_expr(e)),
            Expression::Parenthesized(e) => self.infer_pointee_struct_for_expr(e),
            _ => None,
        }
    }

    fn infer_aggregate_struct_for_expr(&self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Variable(n) => self
                .local_and_param_types
                .get(n)
                .and_then(Self::struct_name_from_aggregate_type),
            Expression::Dereference(inner) => self.infer_pointee_struct_for_expr(inner),
            Expression::Cast(ty, e) => Self::struct_name_from_aggregate_type(ty)
                .or_else(|| self.infer_aggregate_struct_for_expr(e)),
            Expression::Parenthesized(e) => self.infer_aggregate_struct_for_expr(e),
            _ => None,
        }
    }

    fn unique_field_offset_among_structs(&self, field: &str) -> Option<usize> {
        let mut found: Option<usize> = None;
        for (_sn, fields) in &self.struct_field_offsets {
            if let Some(&off) = fields.get(field) {
                if found.is_some() {
                    return None;
                }
                found = Some(off);
            }
        }
        found
    }

    /// 遗留 x86/AArch64/RISC-V 路径：按基表达式类型解析字段偏移；无法解析时仅当全程序中该字段名唯一时回退。
    fn resolve_field_offset_legacy(
        &self,
        base: &Expression,
        field: &str,
        pointer_member: bool,
    ) -> usize {
        let struct_name = if pointer_member {
            self.infer_pointee_struct_for_expr(base)
        } else {
            self.infer_aggregate_struct_for_expr(base)
        };
        if let Some(s) = struct_name {
            if let Some(fields) = self.struct_field_offsets.get(&s) {
                if let Some(&o) = fields.get(field) {
                    return o;
                }
            }
        }
        self.unique_field_offset_among_structs(field).unwrap_or(0)
    }

    /// 生成唯一标签名
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("L{}_{}", prefix, self.label_counter);
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
        // 仅在汇编输出模式添加注释头
        if matches!(self.config.format, OutputFormat::Assembly) {
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
            self.emit_raw("")?;
        }
        Ok(())
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
                // NASM 语法
                self.emit_raw("section .rodata")?;
                for (literal, label) in &string_literals {
                    self.emit_raw(&format!("{}:", label))?;
                    // NASM: db "string", 0
                    self.emit_raw(&format!(
                        "    db \"{}\", 0",
                        escape_assembly_string(literal)
                    ))?;
                }
                self.emit_raw("")?;

                // 全局变量（与 AArch64 分支一致：有初值占位为 0，无初值则仅预留）
                if !globals.is_empty() {
                    self.emit_raw("section .data")?;
                    for (name, info) in &globals {
                        self.emit_raw(&format!("{}:", name))?;
                        if info.initialized {
                            self.emit_raw(&format!("    times {} db 0", info.size))?;
                        } else {
                            self.emit_raw(&format!("    resb {}", info.size))?;
                        }
                    }
                    self.emit_raw("")?;
                }
            }
            TargetArch::AArch64 => {
                // 字符串字面量
                self.emit_raw(".section .rodata")?;
                for (literal, label) in &string_literals {
                    self.emit_raw(&format!("{}:", label))?;
                    self.emit_raw(&format!(
                        "    .asciz \"{}\"",
                        escape_assembly_string(literal)
                    ))?;
                }
                // 全局变量（与 x86_64 的 times/resb、assembly/aarch64 中 GAS 逻辑一致）
                if !globals.is_empty() {
                    self.emit_raw(".section .data")?;
                    for (name, info) in &globals {
                        self.emit_raw(&format!("{}:", name))?;
                        if let Some(init) = &info.initializer {
                            // 评估初始化表达式
                            if let lir::Expression::Literal(lit) = init {
                                let value = match lit {
                                    lir::Literal::Integer(n) => *n as u64,
                                    lir::Literal::Float(f) => {
                                        // 将浮点数转换为 u64 位表示
                                        f.to_bits()
                                    }
                                    lir::Literal::String(_s) => {
                                        // 字符串：使用字符串的指针（简化处理）
                                        0
                                    }
                                    lir::Literal::Char(c) => *c as u64,
                                    lir::Literal::Bool(b) => {
                                        if *b {
                                            1
                                        } else {
                                            0
                                        }
                                    }
                                    _ => 0,
                                };
                                // 根据类型大小输出适当的指令
                                match info.size {
                                    1 => self.emit_raw(&format!("    .byte {}", value))?,
                                    2 => self.emit_raw(&format!("    .short {}", value))?,
                                    4 => self.emit_raw(&format!("    .word {}", value))?,
                                    8 => self.emit_raw(&format!("    .quad {}", value))?,
                                    _ => self.emit_raw(&format!("    .space {}", info.size))?,
                                }
                            } else {
                                // 复杂初始化表达式，使用零初始化
                                self.emit_raw(&format!("    .zero {}", info.size))?;
                            }
                        } else if info.initialized {
                            self.emit_raw(&format!("    .space {}", info.size))?;
                        } else {
                            self.emit_raw(&format!("    .zero {}", info.size))?;
                        }
                    }
                }
            }
            TargetArch::RiscV64 => {
                self.emit_raw(".section .rodata")?;
                for (literal, label) in &string_literals {
                    self.emit_raw(&format!("{}:", label))?;
                    self.emit_raw(&format!(
                        "    .asciz \"{}\"",
                        escape_assembly_string(literal)
                    ))?;
                }
                // 全局变量（与 AArch64 分支、assembly/riscv.rs 中 GAS 逻辑一致）
                if !globals.is_empty() {
                    self.emit_raw("")?;
                    self.emit_raw(".section .data")?;
                    for (name, info) in &globals {
                        self.emit_raw(&format!("{}:", name))?;
                        if info.initialized {
                            self.emit_raw(&format!("    .space {}", info.size))?;
                        } else {
                            self.emit_raw(&format!("    .zero {}", info.size))?;
                        }
                    }
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
        self.struct_field_offsets.clear();
        self.imported_functions.clear();

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
                            initializer: global.initializer.clone(),
                        },
                    );
                }
                Declaration::Function(func) => {
                    // 收集函数中的字符串字面量
                    self.collect_string_literals(&func.body)?;
                }
                Declaration::Struct(struct_decl) => {
                    // 收集结构体字段并计算偏移量
                    let mut offsets = HashMap::new();
                    let mut current_offset = 0;
                    let mut max_align = 1;

                    for field in &struct_decl.fields {
                        let align = self.type_align(&field.type_);
                        let size = self.type_size(&field.type_);

                        // 对齐偏移
                        if current_offset % align != 0 {
                            current_offset += align - (current_offset % align);
                        }

                        offsets.insert(field.name.clone(), current_offset);
                        current_offset += size;

                        if align > max_align {
                            max_align = align;
                        }
                    }

                    let _ = current_offset.next_multiple_of(max_align.max(1));

                    self.struct_field_offsets
                        .insert(struct_decl.name.clone(), offsets);
                }
                Declaration::Class(class_decl) => {
                    // 收集类字段并计算偏移量（包含父类）
                    let mut offsets = HashMap::new();
                    let mut current_offset = 0;
                    let mut max_align = 1;

                    // 如果有虚表，第一个字段是vptr
                    if class_decl.has_vtable {
                        // 虚表指针占 8 字节（x86_64）
                        current_offset = 8;
                        max_align = 8;
                    }

                    for field in &class_decl.fields {
                        let align = self.type_align(&field.type_);
                        let size = self.type_size(&field.type_);

                        // 对齐偏移
                        if current_offset % align != 0 {
                            current_offset += align - (current_offset % align);
                        }

                        offsets.insert(field.name.clone(), current_offset);
                        current_offset += size;

                        if align > max_align {
                            max_align = align;
                        }
                    }

                    let _ = current_offset.next_multiple_of(max_align.max(1));

                    self.struct_field_offsets
                        .insert(class_decl.name.clone(), offsets);
                }
                Declaration::ExternFunction(ext_func) => {
                    // 收集导入的外部函数
                    // 默认放在 kernel32.dll (Windows) 或 libc (Linux)
                    #[cfg(target_os = "windows")]
                    let dll_name = "kernel32.dll".to_string();
                    #[cfg(not(target_os = "windows"))]
                    let dll_name = "libc.so.6".to_string();
                    self.imported_functions
                        .push((dll_name, ext_func.name.clone()));
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

    fn collect_declaration_strings(&mut self, decl: &Declaration) -> NativeResult<()> {
        match decl {
            Declaration::Function(func) => self.collect_string_literals(&func.body),
            Declaration::Global(g) => {
                if let Some(init) = &g.initializer {
                    self.collect_expr_strings(init)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn collect_pattern_strings(&mut self, pat: &Pattern) -> NativeResult<()> {
        match pat {
            Pattern::Literal(Literal::String(s)) => {
                if !self.string_literals.contains_key(s) {
                    let label = format!("LC{}", self.string_literals.len());
                    self.string_literals.insert(s.clone(), label);
                }
                Ok(())
            }
            Pattern::Literal(_) => Ok(()),
            Pattern::Constructor(_, ps) => {
                for p in ps {
                    self.collect_pattern_strings(p)?;
                }
                Ok(())
            }
            Pattern::Tuple(ps) => {
                for p in ps {
                    self.collect_pattern_strings(p)?;
                }
                Ok(())
            }
            Pattern::Record(_, fields) => {
                for (_, p) in fields {
                    self.collect_pattern_strings(p)?;
                }
                Ok(())
            }
            Pattern::Or(a, b) => {
                self.collect_pattern_strings(a)?;
                self.collect_pattern_strings(b)
            }
            Pattern::Wildcard | Pattern::Variable(_) => Ok(()),
        }
    }

    /// 从语句收集字符串字面量
    fn collect_stmt_strings(&mut self, stmt: &Statement) -> NativeResult<()> {
        match stmt {
            Statement::Expression(expr) => self.collect_expr_strings(expr),
            Statement::Declaration(decl) => self.collect_declaration_strings(decl),
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
                if let Some(init) = &for_stmt.initializer {
                    self.collect_stmt_strings(init)?;
                }
                if let Some(cond) = &for_stmt.condition {
                    self.collect_expr_strings(cond)?;
                }
                if let Some(inc) = &for_stmt.increment {
                    self.collect_expr_strings(inc)?;
                }
                self.collect_stmt_strings(&for_stmt.body)
            }
            Statement::DoWhile(do_while) => {
                self.collect_stmt_strings(&do_while.body)?;
                self.collect_expr_strings(&do_while.condition)
            }
            Statement::Switch(sw) => {
                self.collect_expr_strings(&sw.expression)?;
                for c in &sw.cases {
                    self.collect_expr_strings(&c.value)?;
                    self.collect_stmt_strings(&c.body)?;
                }
                if let Some(def) = &sw.default {
                    self.collect_stmt_strings(def)?;
                }
                Ok(())
            }
            Statement::Match(m) => {
                self.collect_expr_strings(&m.scrutinee)?;
                for case in &m.cases {
                    self.collect_pattern_strings(&case.pattern)?;
                    if let Some(g) = &case.guard {
                        self.collect_expr_strings(g)?;
                    }
                    self.collect_string_literals(&case.body)?;
                }
                Ok(())
            }
            Statement::Try(t) => {
                self.collect_string_literals(&t.body)?;
                for c in &t.catch_clauses {
                    self.collect_string_literals(&c.body)?;
                }
                if let Some(fin) = &t.finally_block {
                    self.collect_string_literals(fin)?;
                }
                Ok(())
            }
            Statement::Return(Some(expr)) => self.collect_expr_strings(expr),
            Statement::Compound(block) => self.collect_string_literals(block),
            _ => Ok(()),
        }
    }

    fn collect_initializer_strings(&mut self, init: &Initializer) -> NativeResult<()> {
        match init {
            Initializer::Expression(e) => self.collect_expr_strings(e),
            Initializer::List(items) => {
                for i in items {
                    self.collect_initializer_strings(i)?;
                }
                Ok(())
            }
            Initializer::Named(_, inner) => self.collect_initializer_strings(inner),
            Initializer::Indexed(idx, inner) => {
                self.collect_expr_strings(idx)?;
                self.collect_initializer_strings(inner)
            }
        }
    }

    /// 从表达式收集字符串字面量
    fn collect_expr_strings(&mut self, expr: &Expression) -> NativeResult<()> {
        match expr {
            Expression::Literal(Literal::String(s)) => {
                if !self.string_literals.contains_key(s) {
                    let label = format!("LC{}", self.string_literals.len());
                    self.string_literals.insert(s.clone(), label);
                }
                Ok(())
            }
            Expression::Literal(_) | Expression::Variable(_) => Ok(()),
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
            Expression::Comma(exprs) => {
                for e in exprs {
                    self.collect_expr_strings(e)?;
                }
                Ok(())
            }
            Expression::Parenthesized(e) => self.collect_expr_strings(e),
            Expression::InitializerList(items) => {
                for item in items {
                    self.collect_initializer_strings(item)?;
                }
                Ok(())
            }
            Expression::CompoundLiteral(_, items) => {
                for item in items {
                    self.collect_initializer_strings(item)?;
                }
                Ok(())
            }
            Expression::SizeOf(_) | Expression::AlignOf(_) => Ok(()),
            Expression::SizeOfExpr(e) => self.collect_expr_strings(e),
        }
    }

    /// 生成代码段
    fn emit_text_section(&mut self, lir: &lir::Program) -> NativeResult<()> {
        match self.config.arch {
            TargetArch::X86_64 => {
                // NASM 语法 - 声明外部函数
                self.emit_raw("; External functions")?;
                self.emit_raw("extern printf")?;
                self.emit_raw("extern malloc")?;
                self.emit_raw("extern free")?;
                self.emit_raw("extern println")?;
                self.emit_raw("extern print")?;
                self.emit_raw("extern exit")?;
                self.emit_raw("")?;
                // NASM 语法
                self.emit_raw("section .text code")?;
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
        // 设置当前函数名
        self.current_function = func.name.clone();
        self.init_local_and_param_types_for_function(func);
        // 清除循环标签追踪 - 新函数开始
        self.loop_break_stack.clear();
        self.loop_continue_stack.clear();

        // 函数标签 - NASM 语法
        // 只有 main 函数才声明为全局（入口点）
        if func.name == "main" {
            self.emit_raw(&format!("global {}", func.name))?;
        }
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
                self.local_offsets
                    .insert(param.name.clone(), ((i + 1) * 8) as i32);
                self.emit_line(&format!(
                    "mov qword ptr [rbp-{}], {}",
                    (i + 1) * 8,
                    arg_regs[i]
                ))?;
            } else {
                // 参数在栈中
                let offset = ((i - arg_regs.len() + 2) * 8) as i32;
                self.local_offsets.insert(param.name.clone(), offset);
            }
        }

        // 生成函数体
        self.emit_block(&func.body)?;

        // 更新栈空间分配（修复 bug: 之前是 sub rsp, 0）
        // 确保 16 字节对齐（Windows x64 要求）
        let stack_needed = (self.stack_size + 15) & !15;
        if stack_needed > 0 {
            // 替换 "sub rsp, 0" 为实际需要的大小
            let old_line = format!("sub rsp, 0  ; {}", stack_alloc_label);
            let new_line = format!("sub rsp, {}", stack_needed);
            self.asm_output = self.asm_output.replace(&old_line, &new_line);
        }

        // 函数尾声（如果有返回语句，可能已经跳转到这里）
        let epilogue_label = format!("L{}_epilogue", func.name);
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

        // 清除循环标签追踪 - 函数结束
        self.loop_break_stack.clear();
        self.loop_continue_stack.clear();

        Ok(())
    }

    /// 生成 AArch64 函数
    fn emit_aarch64_function(&mut self, func: &lir::Function) -> NativeResult<()> {
        self.init_local_and_param_types_for_function(func);
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
                self.local_offsets
                    .insert(param.name.clone(), ((i + 1) * 8) as i32);
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
        self.init_local_and_param_types_for_function(func);
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
                self.local_offsets
                    .insert(param.name.clone(), ((i + 1) * 8) as i32);
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
        let local_count = func
            .body
            .statements
            .iter()
            .filter(|s| matches!(s, Statement::Variable(_)))
            .count();

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
                    self.emit_line(&format!("mov qword ptr [rbp-{}], rax", offset))?;
                } else {
                    self.emit_line(&format!("mov qword ptr [rbp-{}], 0", offset))?;
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

                // 将标签推入栈，支持嵌套循环
                self.loop_break_stack.push(end_label.clone());
                self.loop_continue_stack.push(start_label.clone());

                self.emit_raw(&format!("{}:", start_label))?;
                self.emit_expr(&while_stmt.condition)?;
                self.emit_line("test rax, rax")?;
                self.emit_line(&format!("jz {}", end_label))?;

                self.emit_statement(&while_stmt.body)?;
                self.emit_line(&format!("jmp {}", start_label))?;

                self.emit_raw(&format!("{}:", end_label))?;

                // 弹出标签
                self.loop_break_stack.pop();
                self.loop_continue_stack.pop();
            }
            Statement::For(for_stmt) => {
                let start_label = self.new_label("for_start");
                let end_label = self.new_label("for_end");

                // 将标签推入栈，支持嵌套循环
                self.loop_break_stack.push(end_label.clone());
                self.loop_continue_stack.push(start_label.clone());

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

                // 弹出标签
                self.loop_break_stack.pop();
                self.loop_continue_stack.pop();
            }
            Statement::Break => {
                // Break 跳转到当前最内层循环的结束标签
                if let Some(end_label) = self.loop_break_stack.last() {
                    self.emit_line(&format!("jmp {}", end_label))?;
                } else {
                    self.emit_line("; break - error: not inside a loop")?;
                }
            }
            Statement::Continue => {
                // Continue 跳转到当前最内层循环的开始标签
                if let Some(start_label) = self.loop_continue_stack.last() {
                    self.emit_line(&format!("jmp {}", start_label))?;
                } else {
                    self.emit_line("; continue - error: not inside a loop")?;
                }
            }
            Statement::Compound(block) => {
                self.emit_block(block)?;
            }
            Statement::Empty => {}
            Statement::Label(label) => {
                // 添加函数名前缀确保唯一性
                let unique_label = format!("{}_{}", self.current_function, label);
                self.emit_raw(&format!("{}:", unique_label))?;
            }
            Statement::Goto(label) => {
                // 使用相同的函数名前缀
                let unique_label = format!("{}_{}", self.current_function, label);
                self.emit_line(&format!("jmp {}", unique_label))?;
            }
            Statement::Switch(switch) => {
                self.emit_switch(switch)?;
            }
            Statement::Match(match_stmt) => {
                self.emit_match(match_stmt)?;
            }
            Statement::Try(_try) => {
                // 异常处理需要平台特定支持，暂不实现
                self.emit_line("; TODO: try/catch/finally not implemented for native backend")?;
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

    /// 生成 switch 语句
    fn emit_switch(&mut self, switch: &SwitchStatement) -> NativeResult<()> {
        // 计算 switch 表达式的值，结果放在 rax
        self.emit_expr(&switch.expression)?;

        // 为每个 case 和 end 创建标签
        let mut case_labels = Vec::new();
        let end_label = self.new_label("switch_end");

        // 生成每个 case 的比较和条件跳转
        for case in &switch.cases {
            let case_label = self.new_label("case");
            case_labels.push(case_label.clone());

            // 将 case 值加载到 rcx
            self.emit_line("push rax")?; // 保存 original value
            self.emit_expr(&case.value)?;
            self.emit_line("mov rcx, rax")?;
            self.emit_line("pop rax")?; // restore original value

            // 比较，如果相等则跳转到这个 case
            self.emit_line("cmp rax, rcx")?;
            self.emit_line(&format!("je {}", case_label))?;
        }

        // 默认分支
        if let Some(_default) = &switch.default {
            self.emit_line(&format!("jmp {}_default", end_label))?;
        } else {
            self.emit_line(&format!("jmp {}", end_label))?;
        }

        // 生成每个 case 的代码
        for (i, case) in switch.cases.iter().enumerate() {
            let case_label = &case_labels[i];
            self.emit_raw(&format!("{}:", case_label))?;
            self.emit_statement(&case.body)?;
            // case 执行完跳转到 end
            self.emit_line(&format!("jmp {}", end_label))?;
        }

        // 生成默认分支
        if let Some(default) = &switch.default {
            self.emit_raw(&format!("{}_default:", end_label))?;
            self.emit_statement(default)?;
        }

        // switch 结束标签
        self.emit_raw(&format!("{}:", end_label))?;

        Ok(())
    }

    /// 生成 match 语句（模式匹配）
    fn emit_match(&mut self, match_stmt: &MatchStatement) -> NativeResult<()> {
        // 计算 scrutinee 表达式的值
        self.emit_expr(&match_stmt.scrutinee)?;

        let end_label = self.new_label("match_end");
        let mut case_labels = Vec::new();

        // 对于每个 case，生成比较和跳转
        // 这里只处理字面量匹配的简单情况
        // 复杂的构造器模式匹配需要更复杂的实现（比较标签、解构等）
        for (i, case) in match_stmt.cases.iter().enumerate() {
            let case_label = self.new_label(&format!("match_case_{}", i));
            case_labels.push(case_label.clone());

            match &case.pattern {
                Pattern::Literal(lit) => {
                    // 保存 scrutinee，比较字面量
                    self.emit_line("push rax")?;
                    self.emit_literal(lit)?;
                    self.emit_line("mov rcx, rax")?;
                    self.emit_line("pop rax")?;
                    self.emit_line("cmp rax, rcx")?;
                    if let Some(guard) = &case.guard {
                        // 如果有守卫，需要满足守卫条件才跳转
                        self.emit_line("je {case_label}_guard_check")?;
                        self.emit_raw(&format!("{case_label}_guard_check:"))?;
                        self.emit_expr(guard)?;
                        self.emit_line("test rax, rax")?;
                        self.emit_line(&format!("jnz {case_label}"))?;
                    } else {
                        self.emit_line(&format!("je {}", case_label))?;
                    }
                }
                Pattern::Wildcard => {
                    // 通配符总是匹配，直接跳转
                    self.emit_line(&format!("jmp {}", case_label))?;
                }
                Pattern::Variable(_name) => {
                    // 变量模式总是匹配，将值绑定到变量
                    // 简单的实现：直接跳转，不需要比较
                    self.emit_line(&format!("jmp {}", case_label))?;
                }
                _ => {
                    // 复杂模式暂不支持
                    self.emit_line(&format!(
                        "; TODO: unsupported pattern match case {:?}",
                        case.pattern
                    ))?;
                }
            }
        }

        // 如果没有匹配，直接跳到结束
        self.emit_line(&format!("jmp {}", end_label))?;

        // 生成每个 case 的代码
        for (i, case) in match_stmt.cases.iter().enumerate() {
            let case_label = &case_labels[i];
            self.emit_raw(&format!("{}:", case_label))?;
            self.emit_block(&case.body)?;
            self.emit_line(&format!("jmp {}", end_label))?;
        }

        // match 结束
        self.emit_raw(&format!("{}:", end_label))?;

        Ok(())
    }

    /// 遗留路径：嵌套 `Initializer` 展平后的栈槽数量（每项 8 字节，用于 `sub rsp`）。
    fn count_flat_initializer_slots(init: &Initializer) -> usize {
        match init {
            Initializer::Expression(_) => 1,
            Initializer::List(list) => list.iter().map(Self::count_flat_initializer_slots).sum(),
            Initializer::Named(_, inner) => Self::count_flat_initializer_slots(inner),
            Initializer::Indexed(_, inner) => Self::count_flat_initializer_slots(inner),
        }
    }

    fn count_flat_initializer_list_slots(items: &[Initializer]) -> usize {
        items.iter().map(Self::count_flat_initializer_slots).sum()
    }

    /// 遗留路径：深度优先发射初始化器。`Indexed` 会先求值下标表达式（副作用），内层值写入下一可用槽（不做真正的稀疏布局）。
    fn emit_flat_initializer_on_stack(
        &mut self,
        init: &Initializer,
        slot: &mut usize,
    ) -> NativeResult<()> {
        match init {
            Initializer::Expression(e) => {
                self.emit_expr(e)?;
                self.emit_line(&format!("mov [rsp+{}], rax", *slot * 8))?;
                *slot += 1;
            }
            Initializer::List(list) => {
                for i in list {
                    self.emit_flat_initializer_on_stack(i, slot)?;
                }
            }
            Initializer::Named(_, inner) => {
                self.emit_flat_initializer_on_stack(inner, slot)?;
            }
            Initializer::Indexed(idx, inner) => {
                self.emit_expr(idx)?;
                self.emit_flat_initializer_on_stack(inner, slot)?;
            }
        }
        Ok(())
    }

    fn emit_initializer_list_stack_legacy(&mut self, items: &[Initializer]) -> NativeResult<()> {
        let num_slots = Self::count_flat_initializer_list_slots(items);
        let size = num_slots * 8;
        self.emit_line(&format!("sub rsp, {}", size))?;
        let mut slot = 0usize;
        for item in items {
            self.emit_flat_initializer_on_stack(item, &mut slot)?;
        }
        debug_assert_eq!(slot, num_slots);
        self.emit_line("mov rax, rsp")?;
        Ok(())
    }

    /// 生成表达式
    fn emit_expr(&mut self, expr: &Expression) -> NativeResult<()> {
        match expr {
            Expression::Literal(lit) => self.emit_literal(lit)?,
            Expression::Variable(name) => {
                if let Some(offset) = self.local_offsets.get(name) {
                    self.emit_line(&format!("mov rax, qword ptr [rbp-{}]", offset))?;
                } else if self.globals.contains_key(name) {
                    self.emit_line(&format!("mov rax, qword ptr [{}]", name))?;
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
                            self.emit_line(&format!("mov qword ptr [rbp-{}], rax", offset))?;
                        } else if self.globals.contains_key(name) {
                            self.emit_line(&format!("mov qword ptr [{}], rax", name))?;
                        }
                    }
                    Expression::Dereference(ptr) => {
                        self.emit_expr(ptr)?;
                        self.emit_line("mov qword ptr [rax], rax")?;
                    }
                    Expression::Member(obj, field) => {
                        // obj.field = value
                        self.emit_expr(obj)?;
                        let offset = self.resolve_field_offset_legacy(obj, field, false);
                        self.emit_line(&format!("add rax, {}", offset))?;
                        self.emit_line("mov [rax], rax")?;
                    }
                    Expression::PointerMember(obj, field) => {
                        // obj->field = value
                        self.emit_expr(obj)?;
                        self.emit_line("mov rax, [rax]")?;
                        let offset = self.resolve_field_offset_legacy(obj, field, true);
                        self.emit_line(&format!("add rax, {}", offset))?;
                        self.emit_line("mov [rax], rax")?;
                    }
                    Expression::Index(arr, idx) => {
                        // arr[i] = value
                        self.emit_expr(arr)?;
                        self.emit_line("mov rcx, rax")?;
                        self.emit_expr(idx)?;
                        // 假设每个元素是 8 字节指针
                        self.emit_line("shl rax, 3")?;
                        self.emit_line("add rcx, rax")?;
                        self.emit_line("mov [rcx], rax")?;
                    }
                    _ => {
                        return Err(NativeError::InvalidOperand(
                            "Invalid assignment target".to_string(),
                        ));
                    }
                }
            }
            Expression::AddressOf(e) => match e.as_ref() {
                Expression::Variable(name) => {
                    if let Some(offset) = self.local_offsets.get(name) {
                        self.emit_line(&format!("lea rax, [rbp-{}]", offset))?;
                    } else if self.globals.contains_key(name) {
                        self.emit_line(&format!("lea rax, [{}]", name))?;
                    }
                }
                _ => self.emit_expr(e)?,
            },
            Expression::Dereference(e) => {
                self.emit_expr(e)?;
                self.emit_line("mov rax, qword ptr [rax]")?;
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
                let offset = self.resolve_field_offset_legacy(obj, name, false);
                self.emit_line(&format!("add rax, {}", offset))?;
                self.emit_line("mov rax, [rax]")?;
            }
            Expression::PointerMember(obj, name) => {
                self.emit_expr(obj)?;
                self.emit_line("mov rax, [rax]")?;
                let offset = self.resolve_field_offset_legacy(obj, name, true);
                if offset > 0 {
                    self.emit_line(&format!("add rax, {}", offset))?;
                }
                self.emit_line("mov rax, [rax]")?;
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
                            self.emit_line(&format!("mov rax, qword ptr [rbp-{}]", offset))?;
                            // 执行操作
                            match op {
                                BinaryOp::Add => self.emit_line("add rax, rcx")?,
                                BinaryOp::Subtract => self.emit_line("sub rax, rcx")?,
                                BinaryOp::Multiply => self.emit_line("imul rax, rcx")?,
                                _ => {}
                            }
                            // 存回
                            self.emit_line(&format!("mov qword ptr [rbp-{}], rax", offset))?;
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
                self.emit_initializer_list_stack_legacy(items)?;
            }
            Expression::CompoundLiteral(_, items) => {
                self.emit_initializer_list_stack_legacy(items)?;
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
            // References - just leave the value as is since assembly doesn't need special handling
            UnaryOp::Reference => {}
            UnaryOp::MutableReference => {}
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
                self.emit_line("mov rcx, rcx")?;
                self.emit_line("shr rax, cl")?;
            }
            BinaryOp::RightShiftArithmetic => {
                self.emit_line("mov rcx, rcx")?;
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
        for (_i, arg) in args.iter().enumerate() {
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
                    BinaryOp::LeftShift => self.emit_raw("i64.shl")?,
                    BinaryOp::RightShift => self.emit_raw("i64.shr_u")?,
                    BinaryOp::RightShiftArithmetic => self.emit_raw("i64.shr_s")?,
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

    #[cfg(test)]
    fn legacy_asm_snapshot_for_test(&self) -> &str {
        &self.asm_output
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

    fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, Self::Error> {
        // 使用新的 AssemblyGenerator 处理所有架构
        self.generate_from_lir_with_assembly_module(lir)
    }
}

impl NativeBackend {
    /// 使用新的 assembly 模块生成代码（x86_64）
    fn generate_from_lir_with_assembly_module(
        &mut self,
        lir: &x_lir::Program,
    ) -> Result<CodegenOutput, NativeError> {
        // 使用新的 AssemblyGenerator 生成汇编
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

                // macOS/Linux：返回汇编文本，由 x-cli 用 clang 汇编并链接（与 ObjectFile 路径区分）
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

    /// 遗留 `generate_lir` → `emit_expr` 路径：同模块多 struct 的同名成员须按基指针静态类型解析。
    #[test]
    fn test_legacy_x86_resolve_pointer_member_field_by_struct_type() {
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
            .push(Statement::Return(Some(Expression::PointerMember(
                Box::new(Expression::var("pb")),
                "x".into(),
            ))));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };
        let mut backend = NativeBackend::new(config);
        backend.generate_lir(&program).unwrap();
        let content = backend.legacy_asm_snapshot_for_test();
        assert!(
            content.contains("add rax, 4"),
            "应按 *B 解析 `B::x` 偏移 4，而非误用 `A::x` 的 0: {content}"
        );
    }

    /// 遗留路径须在 `Comma` / 嵌套 `Initializer` 中收集字符串，否则 .rodata 缺失会导致链接后读错地址。
    #[test]
    fn test_legacy_x86_collect_strings_comma_and_nested_initializer() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Expression(Expression::Comma(vec![
                Expression::Literal(Literal::String("hello".into())),
                Expression::int(0),
            ])));
        func.body
            .statements
            .push(Statement::Expression(Expression::InitializerList(vec![
                lir::Initializer::List(vec![lir::Initializer::Expression(Expression::Literal(
                    Literal::String("world".into()),
                ))]),
            ])));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };
        let mut backend = NativeBackend::new(config);
        backend.generate_lir(&program).unwrap();
        let content = backend.legacy_asm_snapshot_for_test();
        assert!(
            content.contains(r#"db "hello""#) && content.contains(r#"db "world""#),
            "comma / nested initializer 中的字符串应进入 .rodata: {content}"
        );
    }

    /// 嵌套 `Initializer::List` 须按展平后的标量个数分配栈，而非仅按顶层项数。
    #[test]
    fn test_legacy_x86_initializer_list_nested_stack_size() {
        let mut program = lir::Program::new();
        let mut func = lir::Function::new("main", Type::Int);
        func.body
            .statements
            .push(Statement::Return(Some(Expression::InitializerList(vec![
                lir::Initializer::List(vec![
                    lir::Initializer::Expression(Expression::int(1)),
                    lir::Initializer::Expression(Expression::int(2)),
                ]),
            ]))));
        program.add(Declaration::Function(func));

        let config = NativeBackendConfig {
            arch: TargetArch::X86_64,
            format: OutputFormat::Assembly,
            ..Default::default()
        };
        let mut backend = NativeBackend::new(config);
        backend.generate_lir(&program).unwrap();
        let content = backend.legacy_asm_snapshot_for_test();
        assert!(
            content.lines().any(|l| l.trim() == "sub rsp, 16"),
            "两层标量应分配 16 字节栈，而非仅顶层 1 项的 8 字节: {content}"
        );
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
    fn test_escape_assembly_string() {
        assert_eq!(escape_assembly_string("hello"), "hello");
        assert_eq!(escape_assembly_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_assembly_string("tab\there"), "tab\\there");
        assert_eq!(escape_assembly_string("quote\"test"), "quote\\\"test");
        assert_eq!(escape_assembly_string("back\\slash"), "back\\\\slash");
    }
}
