//! LLVM 后端 - 直接生成 LLVM IR
//!
//! 通过 LLVM 进行深度优化，生成高质量的原生代码
//! 不依赖 inkwell 库，直接生成 LLVM IR 文本

use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use x_codegen::{CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::{
    BinaryOp, Block, Declaration, Expression, Function, GlobalVar, Literal, Program,
    Statement, Type, UnaryOp,
};
use x_parser::ast::Program as AstProgram;

/// LLVM 后端配置
#[derive(Debug, Clone)]
pub struct LlvmBackendConfig {
    /// 输出目录
    pub output_dir: Option<PathBuf>,
    /// 是否启用优化
    pub optimize: bool,
    /// 是否生成调试信息
    pub debug_info: bool,
    /// 目标三元组（如 "x86_64-pc-linux-gnu"）
    pub target_triple: Option<String>,
    /// 模块名称
    pub module_name: String,
}

impl Default for LlvmBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: false,
            target_triple: None,
            module_name: "main".to_string(),
        }
    }
}

/// LLVM 后端错误类型
#[derive(Debug, thiserror::Error)]
pub enum LlvmError {
    #[error("LLVM 代码生成错误: {0}")]
    CodegenError(String),
    #[error("类型错误: {0}")]
    TypeError(String),
    #[error("未实现的特性: {0}")]
    Unimplemented(String),
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("格式化错误: {0}")]
    FmtError(#[from] std::fmt::Error),
}

/// LLVM 后端
pub struct LlvmBackend {
    config: LlvmBackendConfig,
    /// LLVM IR 输出缓冲区
    output: String,
    /// 字符串字面量计数器
    string_counter: usize,
    /// 全局字符串常量映射（内容 -> 名称）
    string_constants: HashMap<String, String>,
    /// 临时寄存器计数器
    temp_counter: usize,
    /// 标签计数器
    label_counter: usize,
    /// 当前函数的局部变量到寄存器的映射
    local_vars: HashMap<String, String>,
    /// 当前函数的局部变量类型映射
    local_var_types: HashMap<String, Type>,
    /// 外部函数声明（用于去重）
    extern_decls: HashMap<String, String>,
}

impl LlvmBackend {
    /// 创建新的 LLVM 后端实例
    pub fn new(config: LlvmBackendConfig) -> Self {
        Self {
            config,
            output: String::new(),
            string_counter: 0,
            string_constants: HashMap::new(),
            temp_counter: 0,
            label_counter: 0,
            local_vars: HashMap::new(),
            local_var_types: HashMap::new(),
            extern_decls: HashMap::new(),
        }
    }

    /// 生成新的临时寄存器名称
    fn new_temp(&mut self) -> String {
        let temp = format!("%t{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    /// 生成新的标签名称
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}.{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// 获取或创建字符串常量的全局名称
    fn get_or_create_string_constant(&mut self, s: &str) -> String {
        if let Some(name) = self.string_constants.get(s) {
            return name.clone();
        }

        let name = format!("@.str.{}", self.string_counter);
        self.string_counter += 1;
        self.string_constants.insert(s.to_string(), name.clone());
        name
    }

    /// 将 LIR 类型转换为 LLVM IR 类型字符串
    fn llvm_type(&self, ty: &Type) -> Result<String, LlvmError> {
        match ty {
            Type::Void => Ok("void".to_string()),
            Type::Bool => Ok("i1".to_string()),
            Type::Char | Type::Schar => Ok("i8".to_string()),
            Type::Uchar => Ok("i8".to_string()),
            Type::Short => Ok("i16".to_string()),
            Type::Ushort => Ok("i16".to_string()),
            Type::Int => Ok("i32".to_string()),
            Type::Uint => Ok("i32".to_string()),
            Type::Long => Ok("i64".to_string()),
            Type::Ulong => Ok("i64".to_string()),
            Type::LongLong => Ok("i64".to_string()),
            Type::UlongLong => Ok("i64".to_string()),
            Type::Float => Ok("float".to_string()),
            Type::Double => Ok("double".to_string()),
            Type::LongDouble => Ok("x86_fp80".to_string()), // x86 扩展精度浮点
            Type::Size | Type::Uintptr => Ok("i64".to_string()),
            Type::Ptrdiff | Type::Intptr => Ok("i64".to_string()),
            Type::Pointer(_) => Ok("i8*".to_string()), // 通用指针类型
            Type::Array(inner, size) => {
                let inner_ty = self.llvm_type(inner)?;
                let size = size.unwrap_or(0);
                Ok(format!("[{} x {}]", size, inner_ty))
            }
            Type::FunctionPointer(ret, params) => {
                let ret_ty = self.llvm_type(ret)?;
                let param_tys: Result<Vec<String>, _> =
                    params.iter().map(|p| self.llvm_type(p)).collect();
                let param_tys = param_tys?;
                Ok(format!("{} ({})", ret_ty, param_tys.join(", ")))
            }
            Type::Named(name) => {
                // 结构体类型，使用不透明指针或命名类型
                Ok(format!("%struct.{}*", name))
            }
            Type::Qualified(_, inner) => self.llvm_type(inner),
        }
    }

    /// 获取类型的位宽（用于整数类型）
    fn type_bits(&self, ty: &Type) -> Result<u32, LlvmError> {
        match ty {
            Type::Bool => Ok(1),
            Type::Char | Type::Schar | Type::Uchar => Ok(8),
            Type::Short | Type::Ushort => Ok(16),
            Type::Int | Type::Uint => Ok(32),
            Type::Long | Type::Ulong | Type::LongLong | Type::UlongLong | Type::Size
            | Type::Uintptr | Type::Ptrdiff | Type::Intptr => Ok(64),
            _ => Err(LlvmError::TypeError(format!(
                "不是整数类型: {:?}",
                ty
            ))),
        }
    }

    /// 写入一行到输出
    fn emit(&mut self, line: &str) {
        writeln!(self.output, "{}", line).unwrap();
    }

    /// 写入一行带缩进
    fn emit_indent(&mut self, indent: usize, line: &str) {
        writeln!(self.output, "{}{}", "  ".repeat(indent), line).unwrap();
    }

    /// 生成模块头部
    fn emit_module_header(&mut self) {
        self.emit(&format!("; ModuleID = '{}'", self.config.module_name));
        if let Some(ref triple) = self.config.target_triple {
            self.emit(&format!("target triple = \"{}\"", triple));
        } else {
            // 默认目标三元组
            #[cfg(target_os = "linux")]
            self.emit("target triple = \"x86_64-pc-linux-gnu\"");
            #[cfg(target_os = "windows")]
            self.emit("target triple = \"x86_64-pc-windows-msvc\"");
            #[cfg(target_os = "macos")]
            self.emit("target triple = \"x86_64-apple-macosx10.15.0\"");
        }
        self.emit("");
    }

    /// 生成字符串常量声明
    fn emit_string_constants(&mut self) {
        if self.string_constants.is_empty() {
            return;
        }

        self.emit("; String constants");
        // Clone to avoid borrow issues
        let constants: Vec<(String, String)> = self
            .string_constants
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (content, name) in constants {
            // 转义字符串中的特殊字符
            let escaped = self.escape_llvm_string(&content);
            let len = content.len() + 1; // 包含 null 终止符
            self.emit(&format!(
                "{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"",
                name, len, escaped
            ));
        }
        self.emit("");
    }

    /// 转义 LLVM 字符串中的特殊字符
    fn escape_llvm_string(&self, s: &str) -> String {
        let mut escaped = String::new();
        for c in s.chars() {
            match c {
                '\\' => escaped.push_str("\\\\"),
                '"' => escaped.push_str("\\22"),
                '\n' => escaped.push_str("\\0A"),
                '\r' => escaped.push_str("\\0D"),
                '\t' => escaped.push_str("\\09"),
                '\0' => escaped.push_str("\\00"),
                c if c.is_ascii() && !c.is_control() => escaped.push(c),
                c => {
                    // 非 ASCII 字符，转义为十六进制
                    let mut buf = [0u8; 4];
                    let bytes = c.encode_utf8(&mut buf);
                    for b in bytes.bytes() {
                        escaped.push_str(&format!("\\{:02X}", b));
                    }
                }
            }
        }
        escaped
    }

    /// 生成外部函数声明
    fn emit_extern_function(&mut self, name: &str, ret_type: &Type, param_types: &[Type]) {
        // 去重
        if self.extern_decls.contains_key(name) {
            return;
        }

        let ret_ty = self.llvm_type(ret_type).unwrap_or_else(|_| "i32".to_string());
        let params: Vec<String> = param_types
            .iter()
            .map(|t| self.llvm_type(t).unwrap_or_else(|_| "i8*".to_string()))
            .collect();

        let decl = if params.is_empty() {
            format!("declare {} @{}()", ret_ty, name)
        } else {
            format!("declare {} @{}({})", ret_ty, name, params.join(", "))
        };

        self.extern_decls.insert(name.to_string(), decl.clone());
        self.emit(&decl);
    }

    /// 生成标准库外部函数声明
    fn emit_stdlib_decls(&mut self) {
        self.emit("; Standard library declarations");

        // printf
        self.emit("declare i32 @printf(i8*, ...)");

        // puts
        self.emit("declare i32 @puts(i8*)");

        // malloc / free
        self.emit("declare i8* @malloc(i64)");
        self.emit("declare void @free(i8*)");

        // memcpy / memset
        self.emit("declare i8* @memcpy(i8*, i8*, i64)");
        self.emit("declare i8* @memset(i8*, i32, i64)");

        // 字符串操作
        self.emit("declare i64 @strlen(i8*)");
        self.emit("declare i8* @strcpy(i8*, i8*)");

        self.emit("");
    }

    /// 生成 LIR 函数
    fn emit_function(&mut self, func: &Function) -> Result<(), LlvmError> {
        // 重置计数器
        self.temp_counter = 0;
        self.local_vars.clear();
        self.local_var_types.clear();

        let ret_ty = self.llvm_type(&func.return_type)?;

        // 参数列表
        let params: Vec<String> = func
            .parameters
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let ty = self.llvm_type(&p.type_).unwrap_or_else(|_| "i8*".to_string());
                format!("{} %{}.param", ty, p.name)
            })
            .collect();

        // 函数签名
        let linkage = if func.is_static { "internal " } else { "" };
        let func_decl = if params.is_empty() {
            format!(
                "define {}{} @{}() {{",
                linkage, ret_ty, func.name
            )
        } else {
            format!(
                "define {}{} @{}({}) {{",
                linkage, ret_ty, func.name, params.join(", ")
            )
        };

        self.emit(&func_decl);

        // 入口基本块
        self.emit_indent(1, "entry:");

        // 为参数分配栈空间并存储
        for param in &func.parameters {
            let param_ty = self.llvm_type(&param.type_)?;
            let ptr = self.new_temp();
            self.emit_indent(2, &format!("{} = alloca {}", ptr, param_ty));
            let param_reg = format!("%{}.param", param.name);
            self.emit_indent(2, &format!("store {} {}, {}* {}", param_ty, param_reg, param_ty, ptr));
            self.local_vars.insert(param.name.clone(), ptr);
            self.local_var_types.insert(param.name.clone(), param.type_.clone());
        }

        // 生成函数体
        self.emit_block(&func.body, 2)?;

        // 确保函数有返回指令
        self.emit_indent(2, "; End of function");

        self.emit("}");
        self.emit("");

        Ok(())
    }

    /// 生成语句块
    fn emit_block(&mut self, block: &Block, indent: usize) -> Result<(), LlvmError> {
        for stmt in &block.statements {
            self.emit_statement(stmt, indent)?;
        }
        Ok(())
    }

    /// 生成语句
    fn emit_statement(&mut self, stmt: &Statement, indent: usize) -> Result<(), LlvmError> {
        match stmt {
            Statement::Return(Some(expr)) => {
                let (value, _ty) = self.emit_expression(expr)?;
                let ret_ty = self.llvm_type(&self.infer_expr_type(expr))?;
                self.emit_indent(indent, &format!("ret {} {}", ret_ty, value));
            }
            Statement::Return(None) => {
                self.emit_indent(indent, "ret void");
            }
            Statement::Expression(expr) => {
                self.emit_expression(expr)?;
            }
            Statement::Variable(var) => {
                let ty = self.llvm_type(&var.type_)?;
                let ptr = self.new_temp();
                self.emit_indent(indent, &format!("{} = alloca {}", ptr, ty));
                self.local_vars.insert(var.name.clone(), ptr.clone());
                self.local_var_types.insert(var.name.clone(), var.type_.clone());

                if let Some(init) = &var.initializer {
                    let (value, _) = self.emit_expression(init)?;
                    self.emit_indent(
                        indent,
                        &format!("store {} {}, {}* {}", ty, value, ty, ptr),
                    );
                }
            }
            Statement::If(if_stmt) => {
                // 生成条件表达式
                let (cond, _) = self.emit_expression(&if_stmt.condition)?;

                // 创建标签
                let then_label = self.new_label("then");
                let else_label = if if_stmt.else_branch.is_some() {
                    self.new_label("else")
                } else {
                    self.new_label("endif")
                };
                let end_label = self.new_label("endif");

                // 条件跳转
                self.emit_indent(
                    indent,
                    &format!("br i1 {}, label %{}, label %{}", cond, then_label, else_label),
                );

                // Then 分支
                self.emit_indent(indent - 1, &format!("{}:", then_label));
                self.emit_statement(&if_stmt.then_branch, indent)?;
                self.emit_indent(indent, &format!("br label %{}", end_label));

                // Else 分支
                if let Some(else_branch) = &if_stmt.else_branch {
                    self.emit_indent(indent - 1, &format!("{}:", else_label));
                    self.emit_statement(else_branch, indent)?;
                    self.emit_indent(indent, &format!("br label %{}", end_label));
                }

                // End 标签
                self.emit_indent(indent - 1, &format!("{}:", end_label));
            }
            Statement::While(while_stmt) => {
                let cond_label = self.new_label("while.cond");
                let body_label = self.new_label("while.body");
                let end_label = self.new_label("while.end");

                // 跳转到条件检查
                self.emit_indent(indent, &format!("br label %{}", cond_label));

                // 条件检查
                self.emit_indent(indent - 1, &format!("{}:", cond_label));
                let (cond, _) = self.emit_expression(&while_stmt.condition)?;
                self.emit_indent(
                    indent,
                    &format!("br i1 {}, label %{}, label %{}", cond, body_label, end_label),
                );

                // 循环体
                self.emit_indent(indent - 1, &format!("{}:", body_label));
                self.emit_statement(&while_stmt.body, indent)?;
                self.emit_indent(indent, &format!("br label %{}", cond_label));

                // 结束
                self.emit_indent(indent - 1, &format!("{}:", end_label));
            }
            Statement::For(for_stmt) => {
                let cond_label = self.new_label("for.cond");
                let body_label = self.new_label("for.body");
                let incr_label = self.new_label("for.incr");
                let end_label = self.new_label("for.end");

                // 初始化
                if let Some(init) = &for_stmt.initializer {
                    self.emit_statement(init, indent)?;
                }
                self.emit_indent(indent, &format!("br label %{}", cond_label));

                // 条件检查
                self.emit_indent(indent - 1, &format!("{}:", cond_label));
                if let Some(cond) = &for_stmt.condition {
                    let (cond_val, _) = self.emit_expression(cond)?;
                    self.emit_indent(
                        indent,
                        &format!(
                            "br i1 {}, label %{}, label %{}",
                            cond_val, body_label, end_label
                        ),
                    );
                } else {
                    self.emit_indent(indent, &format!("br label %{}", body_label));
                }

                // 循环体
                self.emit_indent(indent - 1, &format!("{}:", body_label));
                self.emit_statement(&for_stmt.body, indent)?;
                self.emit_indent(indent, &format!("br label %{}", incr_label));

                // 增量
                self.emit_indent(indent - 1, &format!("{}:", incr_label));
                if let Some(incr) = &for_stmt.increment {
                    self.emit_expression(incr)?;
                }
                self.emit_indent(indent, &format!("br label %{}", cond_label));

                // 结束
                self.emit_indent(indent - 1, &format!("{}:", end_label));
            }
            Statement::DoWhile(do_while) => {
                let body_label = self.new_label("do.body");
                let cond_label = self.new_label("do.cond");
                let end_label = self.new_label("do.end");

                // 跳转到循环体
                self.emit_indent(indent, &format!("br label %{}", body_label));

                // 循环体
                self.emit_indent(indent - 1, &format!("{}:", body_label));
                self.emit_statement(&do_while.body, indent)?;
                self.emit_indent(indent, &format!("br label %{}", cond_label));

                // 条件检查
                self.emit_indent(indent - 1, &format!("{}:", cond_label));
                let (cond, _) = self.emit_expression(&do_while.condition)?;
                self.emit_indent(
                    indent,
                    &format!("br i1 {}, label %{}, label %{}", cond, body_label, end_label),
                );

                // 结束
                self.emit_indent(indent - 1, &format!("{}:", end_label));
            }
            Statement::Break => {
                // 需要上下文来知道跳转到哪个标签
                // 简化实现：添加注释
                self.emit_indent(indent, "; break - not fully implemented without loop context");
            }
            Statement::Continue => {
                self.emit_indent(
                    indent,
                    "; continue - not fully implemented without loop context",
                );
            }
            Statement::Empty => {}
            Statement::Compound(block) => {
                self.emit_block(block, indent)?;
            }
            Statement::Label(name) => {
                self.emit_indent(indent - 1, &format!("{}:", name));
            }
            Statement::Goto(name) => {
                self.emit_indent(indent, &format!("br label %{}", name));
            }
            Statement::Declaration(_) => {
                // 嵌套声明在函数体内处理
            }
            Statement::Switch(_) | Statement::Match(_) | Statement::Try(_) => {
                // 这些复杂语句需要更完整的实现
                self.emit_indent(
                    indent,
                    &format!("; TODO: complex statement: {:?}", stmt),
                );
            }
        }
        Ok(())
    }

    /// 生成表达式，返回结果寄存器和类型
    fn emit_expression(&mut self, expr: &Expression) -> Result<(String, Type), LlvmError> {
        match expr {
            Expression::Literal(lit) => self.emit_literal(lit),
            Expression::Variable(name) => {
                // Clone the name and pointer to avoid borrow issues
                let name = name.clone();
                if let Some(ptr) = self.local_vars.get(&name).cloned() {
                    // Get the type from our type tracking
                    let ty = self
                        .local_var_types
                        .get(&name)
                        .cloned()
                        .unwrap_or(Type::Int);
                    let llvm_ty = self.llvm_type(&ty)?;
                    let result = self.new_temp();
                    self.emit_indent(
                        2,
                        &format!("{} = load {}, {}* {}", result, llvm_ty, llvm_ty, ptr),
                    );
                    Ok((result, ty))
                } else {
                    // 可能是全局变量
                    let ty = Type::Int;
                    let result = self.new_temp();
                    let llvm_ty = self.llvm_type(&ty)?;
                    self.emit_indent(
                        2,
                        &format!("{} = load {}, {}* @{}", result, llvm_ty, llvm_ty, name),
                    );
                    Ok((result, ty))
                }
            }
            Expression::Binary(op, left, right) => {
                self.emit_binary_op(*op, left, right)
            }
            Expression::Unary(op, expr) => self.emit_unary_op(*op, expr),
            Expression::Call(func, args) => self.emit_call(func, args),
            Expression::Assign(target, value) => {
                let (value_reg, value_ty) = self.emit_expression(value)?;
                let llvm_ty = self.llvm_type(&value_ty)?;

                if let Expression::Variable(name) = target.as_ref() {
                    if let Some(ptr) = self.local_vars.get(name) {
                        self.emit_indent(
                            2,
                            &format!("store {} {}, {}* {}", llvm_ty, value_reg, llvm_ty, ptr),
                        );
                    }
                }
                Ok((value_reg, value_ty))
            }
            Expression::Cast(ty, expr) => {
                let (value, expr_ty) = self.emit_expression(expr)?;
                let target_ty = self.llvm_type(ty)?;
                let source_ty = self.llvm_type(&expr_ty)?;

                let result = self.new_temp();

                // 判断是否为浮点或整数类型
                let is_from_float = matches!(
                    expr_ty,
                    Type::Float | Type::Double | Type::LongDouble
                );
                let is_to_float = matches!(
                    ty,
                    Type::Float | Type::Double | Type::LongDouble
                );
                let is_from_integer = matches!(
                    expr_ty,
                    Type::Bool
                        | Type::Char
                        | Type::Schar
                        | Type::Uchar
                        | Type::Short
                        | Type::Ushort
                        | Type::Int
                        | Type::Uint
                        | Type::Long
                        | Type::Ulong
                        | Type::LongLong
                        | Type::UlongLong
                        | Type::Size
                        | Type::Ptrdiff
                        | Type::Intptr
                        | Type::Uintptr
                );
                let is_to_integer = matches!(
                    ty,
                    Type::Bool
                        | Type::Char
                        | Type::Schar
                        | Type::Uchar
                        | Type::Short
                        | Type::Ushort
                        | Type::Int
                        | Type::Uint
                        | Type::Long
                        | Type::Ulong
                        | Type::LongLong
                        | Type::UlongLong
                        | Type::Size
                        | Type::Ptrdiff
                        | Type::Intptr
                        | Type::Uintptr
                );

                // 确定转换类型
                if is_from_float && is_to_integer {
                    // 浮点转整数
                    self.emit_indent(
                        2,
                        &format!("{} = fptosi {} {} to {}", result, source_ty, value, target_ty),
                    );
                } else if is_from_integer && is_to_float {
                    // 整数转浮点
                    self.emit_indent(
                        2,
                        &format!("{} = sitofp {} {} to {}", result, source_ty, value, target_ty),
                    );
                } else if is_from_integer && is_to_integer {
                    // 整数扩展/截断
                    let from_bits = self.type_bits(&expr_ty)?;
                    let to_bits = self.type_bits(ty)?;
                    if to_bits > from_bits {
                        self.emit_indent(
                            2,
                            &format!("{} = sext {} {} to {}", result, source_ty, value, target_ty),
                        );
                    } else {
                        self.emit_indent(
                            2,
                            &format!("{} = trunc {} {} to {}", result, source_ty, value, target_ty),
                        );
                    }
                } else {
                    // 位转换
                    self.emit_indent(
                        2,
                        &format!("{} = bitcast {} {} to {}", result, source_ty, value, target_ty),
                    );
                }

                Ok((result, ty.clone()))
            }
            Expression::AddressOf(expr) => {
                if let Expression::Variable(name) = expr.as_ref() {
                    if let Some(ptr) = self.local_vars.get(name) {
                        return Ok((ptr.clone(), Type::Pointer(Box::new(Type::Int))));
                    }
                }
                Err(LlvmError::Unimplemented("AddressOf for non-variables".to_string()))
            }
            Expression::Dereference(expr) => {
                let (ptr, _ptr_ty) = self.emit_expression(expr)?;
                let ty = Type::Int;
                let llvm_ty = self.llvm_type(&ty)?;
                let result = self.new_temp();
                self.emit_indent(
                    2,
                    &format!("{} = load {}, {}* {}", result, llvm_ty, llvm_ty, ptr),
                );
                Ok((result, ty))
            }
            Expression::Member(obj, name) => {
                let (obj_ptr, _obj_ty) = self.emit_expression(obj)?;
                // 简化：假设字段偏移为 0
                let field_ptr = self.new_temp();
                let ty = Type::Int;
                let llvm_ty = self.llvm_type(&ty)?;
                self.emit_indent(
                    2,
                    &format!(
                        "{} = getelementptr inbounds %struct.{}* {}, i32 0, i32 0",
                        field_ptr, name, obj_ptr
                    ),
                );
                let result = self.new_temp();
                self.emit_indent(
                    2,
                    &format!("{} = load {}, {}* {}", result, llvm_ty, llvm_ty, field_ptr),
                );
                Ok((result, ty))
            }
            Expression::Index(arr, idx) => {
                let (arr_ptr, _arr_ty) = self.emit_expression(arr)?;
                let (idx_val, _idx_ty) = self.emit_expression(idx)?;
                let ty = Type::Int;
                let llvm_ty = self.llvm_type(&ty)?;
                let elem_ptr = self.new_temp();
                self.emit_indent(
                    2,
                    &format!(
                        "{} = getelementptr inbounds {}* {}, i32 {}",
                        elem_ptr, llvm_ty, arr_ptr, idx_val
                    ),
                );
                let result = self.new_temp();
                self.emit_indent(
                    2,
                    &format!("{} = load {}, {}* {}", result, llvm_ty, llvm_ty, elem_ptr),
                );
                Ok((result, ty))
            }
            Expression::Ternary(cond, then_expr, else_expr) => {
                let (cond_val, _) = self.emit_expression(cond)?;

                let result = self.new_temp();
                let result_ty = self.infer_expr_type(then_expr);
                let llvm_ty = self.llvm_type(&result_ty)?;

                let then_label = self.new_label("ternary.then");
                let else_label = self.new_label("ternary.else");
                let end_label = self.new_label("ternary.end");

                self.emit_indent(
                    2,
                    &format!("br i1 {}, label %{}, label %{}", cond_val, then_label, else_label),
                );

                // Then 分支
                self.emit_indent(1, &format!("{}:", then_label));
                let (then_val, _) = self.emit_expression(then_expr)?;
                self.emit_indent(2, &format!("br label %{}", end_label));

                // Else 分支
                self.emit_indent(1, &format!("{}:", else_label));
                let (else_val, _) = self.emit_expression(else_expr)?;
                self.emit_indent(2, &format!("br label %{}", end_label));

                // End - PHI 指令
                self.emit_indent(1, &format!("{}:", end_label));
                self.emit_indent(
                    2,
                    &format!(
                        "{} = phi {} [ {}, %{} ], [ {}, %{} ]",
                        result, llvm_ty, then_val, then_label, else_val, else_label
                    ),
                );

                Ok((result, result_ty))
            }
            Expression::SizeOf(ty) => {
                let llvm_ty = self.llvm_type(ty)?;
                let result = self.new_temp();
                let size = self.type_size(ty);
                self.emit_indent(2, &format!("{} = add i64 {}, 0", result, size));
                Ok((result, Type::Ulong))
            }
            Expression::AlignOf(ty) => {
                let align = self.type_align(ty);
                let result = self.new_temp();
                self.emit_indent(2, &format!("{} = add i64 {}, 0", result, align));
                Ok((result, Type::Ulong))
            }
            _ => {
                // 未实现的表达式类型
                let result = self.new_temp();
                self.emit_indent(
                    2,
                    &format!("; TODO: expression type: {:?}", expr),
                );
                Ok((result, Type::Int))
            }
        }
    }

    /// 生成字面量
    fn emit_literal(&mut self, lit: &Literal) -> Result<(String, Type), LlvmError> {
        match lit {
            Literal::Integer(n) => Ok((n.to_string(), Type::Int)),
            Literal::UnsignedInteger(n) => Ok((n.to_string(), Type::Uint)),
            Literal::Long(n) => Ok((n.to_string(), Type::Long)),
            Literal::UnsignedLong(n) => Ok((n.to_string(), Type::Ulong)),
            Literal::LongLong(n) => Ok((n.to_string(), Type::LongLong)),
            Literal::UnsignedLongLong(n) => Ok((n.to_string(), Type::UlongLong)),
            Literal::Float(n) => {
                let result = self.new_temp();
                self.emit_indent(2, &format!("{} = fptrunc double {} to float", result, n));
                Ok((result, Type::Float))
            }
            Literal::Double(n) => {
                let result = self.new_temp();
                // 使用十六进制表示浮点数以避免精度问题
                let bits = n.to_bits();
                self.emit_indent(
                    2,
                    &format!("{} = fadd double 0x{:016X}, 0x0000000000000000", result, bits),
                );
                Ok((result, Type::Double))
            }
            Literal::Char(c) => Ok(((*c as u32).to_string(), Type::Char)),
            Literal::String(s) => {
                let name = self.get_or_create_string_constant(s);
                let len = s.len() + 1;
                let result = self.new_temp();
                // 获取字符串指针
                self.emit_indent(
                    2,
                    &format!(
                        "{} = getelementptr [{} x i8], [{} x i8]* {}, i32 0, i32 0",
                        result, len, len, name
                    ),
                );
                Ok((result, Type::Pointer(Box::new(Type::Char))))
            }
            Literal::Bool(b) => Ok((if *b { "1" } else { "0" }.to_string(), Type::Bool)),
            Literal::NullPointer => Ok(("null".to_string(), Type::Pointer(Box::new(Type::Void)))),
        }
    }

    /// 生成二元运算
    fn emit_binary_op(
        &mut self,
        op: BinaryOp,
        left: &Expression,
        right: &Expression,
    ) -> Result<(String, Type), LlvmError> {
        let (left_val, left_ty) = self.emit_expression(left)?;
        let (right_val, _right_ty) = self.emit_expression(right)?;

        let result = self.new_temp();
        let llvm_ty = self.llvm_type(&left_ty)?;

        let is_float = matches!(
            left_ty,
            Type::Float | Type::Double | Type::LongDouble
        );

        let instruction = match op {
            BinaryOp::Add => {
                if is_float {
                    format!("{} = fadd {} {}, {}", result, llvm_ty, left_val, right_val)
                } else {
                    format!("{} = add {} {}, {}", result, llvm_ty, left_val, right_val)
                }
            }
            BinaryOp::Subtract => {
                if is_float {
                    format!("{} = fsub {} {}, {}", result, llvm_ty, left_val, right_val)
                } else {
                    format!("{} = sub {} {}, {}", result, llvm_ty, left_val, right_val)
                }
            }
            BinaryOp::Multiply => {
                if is_float {
                    format!("{} = fmul {} {}, {}", result, llvm_ty, left_val, right_val)
                } else {
                    format!("{} = mul {} {}, {}", result, llvm_ty, left_val, right_val)
                }
            }
            BinaryOp::Divide => {
                if is_float {
                    format!("{} = fdiv {} {}, {}", result, llvm_ty, left_val, right_val)
                } else {
                    // 使用有符号除法
                    format!("{} = sdiv {} {}, {}", result, llvm_ty, left_val, right_val)
                }
            }
            BinaryOp::Modulo => {
                if is_float {
                    format!("{} = frem {} {}, {}", result, llvm_ty, left_val, right_val)
                } else {
                    format!("{} = srem {} {}, {}", result, llvm_ty, left_val, right_val)
                }
            }
            BinaryOp::LeftShift => {
                format!("{} = shl {} {}, {}", result, llvm_ty, left_val, right_val)
            }
            BinaryOp::RightShift => {
                format!("{} = ashr {} {}, {}", result, llvm_ty, left_val, right_val)
            }
            BinaryOp::LessThan => {
                if is_float {
                    format!(
                        "{} = fcmp olt {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                } else {
                    format!(
                        "{} = icmp slt {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                }
            }
            BinaryOp::LessThanEqual => {
                if is_float {
                    format!(
                        "{} = fcmp ole {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                } else {
                    format!(
                        "{} = icmp sle {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                }
            }
            BinaryOp::GreaterThan => {
                if is_float {
                    format!(
                        "{} = fcmp ogt {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                } else {
                    format!(
                        "{} = icmp sgt {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                }
            }
            BinaryOp::GreaterThanEqual => {
                if is_float {
                    format!(
                        "{} = fcmp oge {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                } else {
                    format!(
                        "{} = icmp sge {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                }
            }
            BinaryOp::Equal => {
                if is_float {
                    format!(
                        "{} = fcmp oeq {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                } else {
                    format!(
                        "{} = icmp eq {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                }
            }
            BinaryOp::NotEqual => {
                if is_float {
                    format!(
                        "{} = fcmp one {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                } else {
                    format!(
                        "{} = icmp ne {} {}, {}",
                        result, llvm_ty, left_val, right_val
                    )
                }
            }
            BinaryOp::BitAnd => {
                format!("{} = and {} {}, {}", result, llvm_ty, left_val, right_val)
            }
            BinaryOp::BitOr => {
                format!("{} = or {} {}, {}", result, llvm_ty, left_val, right_val)
            }
            BinaryOp::BitXor => {
                format!("{} = xor {} {}, {}", result, llvm_ty, left_val, right_val)
            }
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => {
                // 短路求值需要基本块，这里简化处理
                let op_str = if matches!(op, BinaryOp::LogicalAnd) {
                    "and"
                } else {
                    "or"
                };
                format!("{} = {} i1 {}, {}", result, op_str, left_val, right_val)
            }
        };

        self.emit_indent(2, &instruction);

        let result_ty = if matches!(
            op,
            BinaryOp::LessThan
                | BinaryOp::LessThanEqual
                | BinaryOp::GreaterThan
                | BinaryOp::GreaterThanEqual
                | BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::LogicalAnd
                | BinaryOp::LogicalOr
        ) {
            Type::Bool
        } else {
            left_ty
        };

        Ok((result, result_ty))
    }

    /// 生成一元运算
    fn emit_unary_op(&mut self, op: UnaryOp, expr: &Expression) -> Result<(String, Type), LlvmError> {
        let (val, ty) = self.emit_expression(expr)?;
        let result = self.new_temp();
        let llvm_ty = self.llvm_type(&ty)?;

        let is_float = matches!(ty, Type::Float | Type::Double | Type::LongDouble);

        let instruction = match op {
            UnaryOp::Minus => {
                if is_float {
                    format!("{} = fneg {} {}", result, llvm_ty, val)
                } else {
                    format!("{} = sub {} 0, {}", result, llvm_ty, val)
                }
            }
            UnaryOp::Not => {
                format!("{} = xor i1 {}, 1", result, val)
            }
            UnaryOp::BitNot => {
                format!("{} = xor {} {}, -1", result, llvm_ty, val)
            }
            UnaryOp::PreIncrement | UnaryOp::PostIncrement => {
                // 需要变量地址来更新
                return Err(LlvmError::Unimplemented("Increment operators".to_string()));
            }
            UnaryOp::PreDecrement | UnaryOp::PostDecrement => {
                return Err(LlvmError::Unimplemented("Decrement operators".to_string()));
            }
            UnaryOp::Plus => {
                return Ok((val, ty));
            }
        };

        self.emit_indent(2, &instruction);
        Ok((result, ty))
    }

    /// 生成函数调用
    fn emit_call(
        &mut self,
        func: &Expression,
        args: &[Expression],
    ) -> Result<(String, Type), LlvmError> {
        let func_name = match func {
            Expression::Variable(name) => name.clone(),
            _ => {
                return Err(LlvmError::Unimplemented(
                    "Indirect function calls".to_string(),
                ))
            }
        };

        // 评估参数
        let mut arg_values = Vec::new();
        let mut arg_types = Vec::new();
        for arg in args {
            let (val, ty) = self.emit_expression(arg)?;
            arg_values.push(val);
            arg_types.push(self.llvm_type(&ty)?);
        }

        // 确定返回类型
        let ret_type = self.infer_call_return_type(&func_name);
        let llvm_ret = self.llvm_type(&ret_type)?;

        let result = self.new_temp();

        // 构建参数列表
        let args_str: Vec<String> = arg_types
            .iter()
            .zip(arg_values.iter())
            .map(|(ty, val)| format!("{} {}", ty, val))
            .collect();

        if args_str.is_empty() {
            self.emit_indent(
                2,
                &format!("{} = call {} @{}()", result, llvm_ret, func_name),
            );
        } else {
            self.emit_indent(
                2,
                &format!("{} = call {} @{}({})", result, llvm_ret, func_name, args_str.join(", ")),
            );
        }

        Ok((result, ret_type))
    }

    /// 推断表达式类型（简化实现）
    fn infer_expr_type(&self, expr: &Expression) -> Type {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Integer(_) => Type::Int,
                Literal::UnsignedInteger(_) => Type::Uint,
                Literal::Long(_) => Type::Long,
                Literal::UnsignedLong(_) => Type::Ulong,
                Literal::LongLong(_) => Type::LongLong,
                Literal::UnsignedLongLong(_) => Type::UlongLong,
                Literal::Float(_) => Type::Float,
                Literal::Double(_) => Type::Double,
                Literal::Char(_) => Type::Char,
                Literal::String(_) => Type::Pointer(Box::new(Type::Char)),
                Literal::Bool(_) => Type::Bool,
                Literal::NullPointer => Type::Pointer(Box::new(Type::Void)),
            },
            Expression::Binary(op, _, _) => {
                if matches!(
                    *op,
                    BinaryOp::LessThan
                        | BinaryOp::LessThanEqual
                        | BinaryOp::GreaterThan
                        | BinaryOp::GreaterThanEqual
                        | BinaryOp::Equal
                        | BinaryOp::NotEqual
                        | BinaryOp::LogicalAnd
                        | BinaryOp::LogicalOr
                ) {
                    Type::Bool
                } else {
                    Type::Int
                }
            }
            Expression::Cast(ty, _) => ty.clone(),
            Expression::SizeOf(_) | Expression::AlignOf(_) => Type::Ulong,
            Expression::Ternary(_, then_expr, _) => self.infer_expr_type(then_expr),
            _ => Type::Int,
        }
    }

    /// 推断函数返回类型（简化实现）
    fn infer_call_return_type(&self, func_name: &str) -> Type {
        match func_name {
            "printf" | "puts" => Type::Int,
            "malloc" => Type::Pointer(Box::new(Type::Void)),
            "free" => Type::Void,
            "strlen" => Type::Ulong,
            _ => Type::Int,
        }
    }

    /// 获取类型大小（字节）
    fn type_size(&self, ty: &Type) -> u64 {
        match ty {
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            Type::Long | Type::Ulong | Type::LongLong | Type::UlongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::LongDouble => 16,
            Type::Size | Type::Uintptr | Type::Ptrdiff | Type::Intptr => 8,
            Type::Pointer(_) => 8,
            Type::Array(inner, size) => {
                let inner_size = self.type_size(inner);
                inner_size * size.unwrap_or(1)
            }
            _ => 8,
        }
    }

    /// 获取类型对齐（字节）
    fn type_align(&self, ty: &Type) -> u64 {
        match ty {
            Type::Bool => 1,
            Type::Char | Type::Schar | Type::Uchar => 1,
            Type::Short | Type::Ushort => 2,
            Type::Int | Type::Uint => 4,
            Type::Long | Type::Ulong | Type::LongLong | Type::UlongLong => 8,
            Type::Float => 4,
            Type::Double => 8,
            Type::LongDouble => 16,
            Type::Pointer(_) => 8,
            _ => 8,
        }
    }

    /// 生成全局变量
    fn emit_global(&mut self, global: &GlobalVar) {
        let ty = self.llvm_type(&global.type_).unwrap_or_else(|_| "i32".to_string());
        let name = format!("@{}", global.name);

        let mut decl = format!("{} = ", name);
        if global.is_static {
            decl.push_str("internal ");
        }
        decl.push_str("global ");

        if let Some(init) = &global.initializer {
            if let Expression::Literal(lit) = init {
                match lit {
                    Literal::Integer(n) => decl.push_str(&format!("i32 {}", n)),
                    Literal::Long(n) => decl.push_str(&format!("i64 {}", n)),
                    Literal::Double(n) => {
                        let bits = n.to_bits();
                        decl.push_str(&format!("double 0x{:016X}", bits));
                    }
                    Literal::Bool(b) => decl.push_str(&format!("i1 {}", if *b { 1 } else { 0 })),
                    Literal::String(s) => {
                        let str_name = self.get_or_create_string_constant(s);
                        let len = s.len() + 1;
                        decl.push_str(&format!("[{} x i8]* {}", len, str_name));
                    }
                    _ => decl.push_str(&format!("{} zeroinitializer", ty)),
                }
            } else {
                decl.push_str(&format!("{} zeroinitializer", ty));
            }
        } else {
            decl.push_str(&format!("{} zeroinitializer", ty));
        }

        self.emit(&decl);
    }

    /// 从 LIR 程序生成 LLVM IR
    pub fn generate_from_lir_program(&mut self, lir: &Program) -> Result<CodegenOutput, LlvmError> {
        // 重置状态
        self.output.clear();
        self.string_counter = 0;
        self.string_constants.clear();
        self.extern_decls.clear();

        // 生成模块头部
        self.emit_module_header();

        // 第一遍：收集字符串常量和外部函数声明
        for decl in &lir.declarations {
            match decl {
                Declaration::ExternFunction(extern_func) => {
                    self.emit_extern_function(
                        &extern_func.name,
                        &extern_func.return_type,
                        &extern_func.parameters,
                    );
                }
                Declaration::Global(global) => {
                    if let Some(Expression::Literal(Literal::String(s))) = &global.initializer {
                        self.get_or_create_string_constant(s);
                    }
                }
                _ => {}
            }
        }

        // 第二遍：生成代码
        self.emit_stdlib_decls();

        // 生成全局变量
        let has_globals = lir
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Global(_)));
        if has_globals {
            self.emit("; Global variables");
            for decl in &lir.declarations {
                if let Declaration::Global(global) = decl {
                    self.emit_global(global);
                }
            }
            self.emit("");
        }

        // 生成字符串常量
        self.emit_string_constants();

        // 生成函数
        for decl in &lir.declarations {
            if let Declaration::Function(func) = decl {
                self.emit_function(func)?;
            }
        }

        // 生成 main 函数包装器（如果没有）
        let has_main = lir
            .declarations
            .iter()
            .any(|d| matches!(d, Declaration::Function(f) if f.name == "main"));

        if !has_main {
            self.emit_main_wrapper();
        }

        // 创建输出文件
        let output_file = OutputFile {
            path: PathBuf::from(format!("{}.ll", self.config.module_name)),
            content: self.output.as_bytes().to_vec(),
            file_type: FileType::LlvmIr,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 生成默认的 main 函数包装器
    fn emit_main_wrapper(&mut self) {
        self.emit("; Default main function");
        self.emit("define i32 @main() {");
        self.emit_indent(1, "entry:");
        self.emit_indent(2, "ret i32 0");
        self.emit("}");
    }
}

impl CodeGenerator for LlvmBackend {
    type Config = LlvmBackendConfig;
    type Error = LlvmError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_ast(&mut self, program: &AstProgram) -> Result<CodegenOutput, Self::Error> {
        // 从 AST 直接生成需要完整的实现
        // 这里返回错误提示用户使用 LIR 接口
        Err(LlvmError::Unimplemented(
            "LLVM 后端不支持直接从 AST 生成。请使用 LIR 接口。".to_string(),
        ))
    }

    fn generate_from_hir(&mut self, _hir: &x_hir::Hir) -> Result<CodegenOutput, Self::Error> {
        Err(LlvmError::Unimplemented(
            "LLVM 后端不支持直接从 HIR 生成。请使用 LIR 接口。".to_string(),
        ))
    }

    fn generate_from_lir(&mut self, lir: &Program) -> Result<CodegenOutput, Self::Error> {
        self.generate_from_lir_program(lir)
    }
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use x_lir::Parameter;

    #[test]
    fn test_llvm_type_mapping() {
        let backend = LlvmBackend::new(LlvmBackendConfig::default());

        assert_eq!(backend.llvm_type(&Type::Void).unwrap(), "void");
        assert_eq!(backend.llvm_type(&Type::Bool).unwrap(), "i1");
        assert_eq!(backend.llvm_type(&Type::Char).unwrap(), "i8");
        assert_eq!(backend.llvm_type(&Type::Int).unwrap(), "i32");
        assert_eq!(backend.llvm_type(&Type::Long).unwrap(), "i64");
        assert_eq!(backend.llvm_type(&Type::Float).unwrap(), "float");
        assert_eq!(backend.llvm_type(&Type::Double).unwrap(), "double");
        assert_eq!(backend.llvm_type(&Type::Pointer(Box::new(Type::Int))).unwrap(), "i8*");
    }

    #[test]
    fn test_simple_function() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![Declaration::Function(Function {
                name: "add".to_string(),
                return_type: Type::Int,
                parameters: vec![
                    Parameter {
                        name: "a".to_string(),
                        type_: Type::Int,
                    },
                    Parameter {
                        name: "b".to_string(),
                        type_: Type::Int,
                    },
                ],
                body: Block {
                    statements: vec![Statement::Return(Some(
                        Expression::Binary(
                            BinaryOp::Add,
                            Box::new(Expression::Variable("a".to_string())),
                            Box::new(Expression::Variable("b".to_string())),
                        ),
                    ))],
                },
                is_static: false,
                is_inline: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].file_type, FileType::LlvmIr);

        let content = String::from_utf8(output.files[0].content.clone()).unwrap();
        assert!(content.contains("define i32 @add"));
        assert!(content.contains("add i32"));
    }

    #[test]
    fn test_main_function() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![Declaration::Function(Function {
                name: "main".to_string(),
                return_type: Type::Int,
                parameters: vec![],
                body: Block {
                    statements: vec![Statement::Return(Some(Expression::int(0)))],
                },
                is_static: false,
                is_inline: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("define i32 @main"));
        assert!(content.contains("ret i32 0"));
    }

    #[test]
    fn test_global_variable() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![
                Declaration::Global(GlobalVar {
                    name: "counter".to_string(),
                    type_: Type::Int,
                    initializer: Some(Expression::int(42)),
                    is_static: false,
                }),
                Declaration::Function(Function {
                    name: "main".to_string(),
                    return_type: Type::Int,
                    parameters: vec![],
                    body: Block {
                        statements: vec![Statement::Return(Some(Expression::int(0)))],
                    },
                    is_static: false,
                    is_inline: false,
                }),
            ],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("@counter"));
        assert!(content.contains("global i32 42"));
    }

    #[test]
    fn test_string_constant() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![Declaration::Global(GlobalVar {
                name: "message".to_string(),
                type_: Type::Pointer(Box::new(Type::Char)),
                initializer: Some(Expression::string("Hello, World!")),
                is_static: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("@.str.0"));
        assert!(content.contains("Hello, World!"));
    }

    #[test]
    fn test_printf_call() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let message = Expression::string("Hello");
        let call = Expression::Call(
            Box::new(Expression::Variable("printf".to_string())),
            vec![message],
        );

        let lir = Program {
            declarations: vec![Declaration::Function(Function {
                name: "main".to_string(),
                return_type: Type::Int,
                parameters: vec![],
                body: Block {
                    statements: vec![
                        Statement::Expression(call),
                        Statement::Return(Some(Expression::int(0))),
                    ],
                },
                is_static: false,
                is_inline: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("call i32 @printf"));
    }

    #[test]
    fn test_if_statement() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![Declaration::Function(Function {
                name: "test_if".to_string(),
                return_type: Type::Int,
                parameters: vec![],
                body: Block {
                    statements: vec![
                        Statement::Variable(x_lir::Variable::new("x", Type::Int).init(Expression::int(10))),
                        Statement::If(x_lir::IfStatement {
                            condition: Expression::Binary(
                                BinaryOp::GreaterThan,
                                Box::new(Expression::Variable("x".to_string())),
                                Box::new(Expression::int(5)),
                            ),
                            then_branch: Box::new(Statement::Return(Some(Expression::int(1)))),
                            else_branch: Some(Box::new(Statement::Return(Some(Expression::int(0))))),
                        }),
                    ],
                },
                is_static: false,
                is_inline: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("icmp sgt"));
        assert!(content.contains("br i1"));
    }

    #[test]
    fn test_while_loop() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![Declaration::Function(Function {
                name: "count_down".to_string(),
                return_type: Type::Int,
                parameters: vec![Parameter {
                    name: "n".to_string(),
                    type_: Type::Int,
                }],
                body: Block {
                    statements: vec![
                        Statement::While(x_lir::WhileStatement {
                            condition: Expression::Binary(
                                BinaryOp::GreaterThan,
                                Box::new(Expression::Variable("n".to_string())),
                                Box::new(Expression::int(0)),
                            ),
                            body: Box::new(Statement::Expression(Expression::Assign(
                                Box::new(Expression::Variable("n".to_string())),
                                Box::new(Expression::Binary(
                                    BinaryOp::Subtract,
                                    Box::new(Expression::Variable("n".to_string())),
                                    Box::new(Expression::int(1)),
                                )),
                            ))),
                        }),
                        Statement::Return(Some(Expression::Variable("n".to_string()))),
                    ],
                },
                is_static: false,
                is_inline: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("while.cond"));
        assert!(content.contains("while.body"));
    }

    #[test]
    fn test_ternary_expression() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let ternary = Expression::Ternary(
            Box::new(Expression::Binary(
                BinaryOp::GreaterThan,
                Box::new(Expression::int(5)),
                Box::new(Expression::int(3)),
            )),
            Box::new(Expression::int(1)),
            Box::new(Expression::int(0)),
        );

        let lir = Program {
            declarations: vec![Declaration::Function(Function {
                name: "test_ternary".to_string(),
                return_type: Type::Int,
                parameters: vec![],
                body: Block {
                    statements: vec![Statement::Return(Some(ternary))],
                },
                is_static: false,
                is_inline: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("phi i32"));
    }

    #[test]
    fn test_float_operations() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![Declaration::Function(Function {
                name: "float_add".to_string(),
                return_type: Type::Double,
                parameters: vec![
                    Parameter {
                        name: "a".to_string(),
                        type_: Type::Double,
                    },
                    Parameter {
                        name: "b".to_string(),
                        type_: Type::Double,
                    },
                ],
                body: Block {
                    statements: vec![Statement::Return(Some(Expression::Binary(
                        BinaryOp::Add,
                        Box::new(Expression::Variable("a".to_string())),
                        Box::new(Expression::Variable("b".to_string())),
                    )))],
                },
                is_static: false,
                is_inline: false,
            })],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("fadd double"));
    }

    #[test]
    fn test_extern_function_decl() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig::default());

        let lir = Program {
            declarations: vec![
                Declaration::ExternFunction(x_lir::ExternFunction {
                    name: "external_func".to_string(),
                    return_type: Type::Int,
                    parameters: vec![Type::Int, Type::Pointer(Box::new(Type::Char))],
                }),
                Declaration::Function(Function {
                    name: "main".to_string(),
                    return_type: Type::Int,
                    parameters: vec![],
                    body: Block {
                        statements: vec![Statement::Return(Some(Expression::int(0)))],
                    },
                    is_static: false,
                    is_inline: false,
                }),
            ],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("declare i32 @external_func"));
    }

    #[test]
    fn test_module_header() {
        let mut backend = LlvmBackend::new(LlvmBackendConfig {
            module_name: "test_module".to_string(),
            target_triple: Some("x86_64-pc-linux-gnu".to_string()),
            ..Default::default()
        });

        let lir = Program {
            declarations: vec![],
        };

        let result = backend.generate_from_lir(&lir);
        assert!(result.is_ok());

        let content =
            String::from_utf8(result.unwrap().files[0].content.clone()).unwrap();
        assert!(content.contains("; ModuleID = 'test_module'"));
        assert!(content.contains("target triple = \"x86_64-pc-linux-gnu\""));
    }
}
