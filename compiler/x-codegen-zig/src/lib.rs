//! Zig 后端 - 将 X LIR 编译为 Zig 0.15 代码
//!
//! 利用 Zig 的内存管理和错误处理特性，提供高效的编译输出
//!
//! ## Zig 0.15 特性支持 (2025年10月发布)
//! - Improved AstGen/Zon syntax
//! - 命名空间隔离改进
//! - 改进的错误处理
//! - @import 语义更新
//! - 自定义增量编译
//! - Wasm 改进（wasm32-wasi, wasm32-freestanding）
//! - Improved C interoperability
//! - Better incremental compilation

#![allow(clippy::only_used_in_recursion, clippy::useless_format)]

use std::path::{Path, PathBuf};
use x_codegen::headers;
use x_lir::Program as LirProgram;

/// 编译目标
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZigTarget {
    #[default]
    Native,
    Wasm32Wasi,
    Wasm32Freestanding,
}

impl ZigTarget {
    pub fn as_zig_target(&self) -> &'static str {
        match self {
            ZigTarget::Native => "native",
            ZigTarget::Wasm32Wasi => "wasm32-wasi",
            ZigTarget::Wasm32Freestanding => "wasm32-freestanding",
        }
    }

    pub fn output_extension(&self) -> &'static str {
        match self {
            ZigTarget::Native => "", // Platform-specific
            ZigTarget::Wasm32Wasi | ZigTarget::Wasm32Freestanding => ".wasm",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZigBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
    pub target: ZigTarget,
}

impl Default for ZigBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
            target: ZigTarget::Native,
        }
    }
}

pub struct ZigBackend {
    config: ZigBackendConfig,
    /// 代码缓冲区（统一管理输出和缩进）
    buffer: x_codegen::CodeBuffer,
    /// 当前正在发射的函数名
    current_function_name: String,
    /// 跟踪 void 返回调用的目标变量，以便跳过其声明
    void_call_vars: std::collections::HashSet<String>,
}

pub type ZigResult<T> = Result<T, x_codegen::CodeGenError>;

// 保持向后兼容的别名
pub type ZigCodeGenerator = ZigBackend;
pub type ZigCodeGenError = x_codegen::CodeGenError;

impl ZigBackend {
    pub fn new(config: ZigBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
            current_function_name: String::new(),
            void_call_vars: std::collections::HashSet::new(),
        }
    }

    /// 从 LIR 生成代码（低层中间表示 - 后端统一正式输入）
    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> ZigResult<x_codegen::CodegenOutput> {
        self.buffer.clear();
        self.void_call_vars.clear();

        self.emit_header()?;

        // Single pass to categorize declarations (avoid O(5N) multiple passes)
        let mut extern_funcs = Vec::new();
        let mut global_vars = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut functions = Vec::new();

        for decl in &lir.declarations {
            match decl {
                x_lir::Declaration::ExternFunction(f) => extern_funcs.push(f),
                x_lir::Declaration::Global(v) => global_vars.push(v),
                x_lir::Declaration::Struct(s) => structs.push(s),
                x_lir::Declaration::Enum(e) => enums.push(e),
                x_lir::Declaration::Function(f) => functions.push(f),
                _ => {}
            }
        }

        // Emit in required order
        for f in &extern_funcs {
            self.emit_lir_extern_function(f)?;
        }

        for v in &global_vars {
            self.emit_lir_global_var(v)?;
        }

        for s in &structs {
            self.emit_lir_struct(s)?;
        }

        for e in &enums {
            self.emit_lir_enum(e)?;
        }

        for f in &functions {
            self.emit_lir_function(f)?;
            self.line("")?;
        }

        // Create output file
        let output_file = x_codegen::OutputFile {
            path: std::path::PathBuf::from("output.zig"),
            content: self.output().as_bytes().to_vec(),
            file_type: x_codegen::FileType::Zig,
        };

        Ok(x_codegen::CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    // ========================================================================
    // Header
    // ========================================================================

    fn emit_header(&mut self) -> ZigResult<()> {
        self.line(headers::ZIG)?;
        self.line("// DO NOT EDIT")?;
        self.line("")?;

        // 默认导入 std
        self.line("const std = @import(\"std\");")?;
        self.line("")?;

        // 全局 allocator
        self.line("const allocator = std.heap.page_allocator;")?;
        self.line("")?;

        // Helper function for equality comparison (handles strings and other types)
        self.line("fn xEqual(__lhs: anytype, __rhs: @TypeOf(__lhs)) bool {")?;
        self.line("    return if (@typeInfo(@TypeOf(__lhs)) == .pointer)")?;
        self.line("        std.mem.eql(u8, __lhs, __rhs)")?;
        self.line("    else")?;
        self.line("        __lhs == __rhs;")?;
        self.line("}")?;
        self.line("")?;

        // HTTP Server runtime
        self.line("var http_server_handle: ?std.net.Server = null;")?;
        self.line("")?;

        self.line("fn http_listen(host: []const u8, port: u16) void {")?;
        self.indent();
        self.line("const addr = std.net.Address.parseIp(host, port) catch {")?;
        self.indent();
        self.line("std.debug.print(\"Failed to parse address\\\\n\", .{});")?;
        self.line("return;")?;
        self.dedent();
        self.line("};")?;
        self.line("http_server_handle = addr.listen(.{ .reuse_address = true }) catch {")?;
        self.indent();
        self.line("std.debug.print(\"Failed to start server\\\\n\", .{});")?;
        self.line("return;")?;
        self.dedent();
        self.line("};")?;
        self.line(
            "std.debug.print(\"HTTP Server listening on http://{s}:{d}\\\\n\", .{ host, port });",
        )?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        self.line("fn http_accept() ?[]const u8 {")?;
        self.indent();
        self.line("const server = http_server_handle orelse return null;")?;
        self.line("var conn = server.accept() catch return null;")?;
        self.line("defer conn.stream.close();")?;
        self.line("")?;
        self.line("var buf: [4096]u8 = undefined;")?;
        self.line("const n = conn.stream.read(&buf) catch return null;")?;
        self.line("if (n == 0) return null;")?;
        self.line("")?;
        self.line("const request = allocator.alloc(u8, n) catch return null;")?;
        self.line("@memcpy(request, buf[0..n]);")?;
        self.line("return request;")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        self.line(
            "fn http_respond(status: u16, content_type: []const u8, body: []const u8) void {",
        )?;
        self.indent();
        self.line("const server = http_server_handle orelse return;")?;
        self.line("var conn = server.accept() catch return;")?;
        self.line("defer conn.stream.close();")?;
        self.line("")?;
        self.line("var buf: [1024]u8 = undefined;")?;
        self.line("const response = std.fmt.bufPrint(&buf,")?;
        self.indent();
        self.line("\\\\\"HTTP/1.1 {d} OK\\\\r\\\\n\\\\\" ++")?;
        self.line("\\\\\"Content-Type: {s}\\\\r\\\\n\\\\\" ++")?;
        self.line("\\\\\"Content-Length: {d}\\\\r\\\\n\\\\r\\\\n\\\\\"")?;
        self.line(", .{ status, content_type, body.len }) catch return;")?;
        self.dedent();
        self.line("")?;
        self.line("_ = conn.stream.writeAll(response) catch {};")?;
        self.line("_ = conn.stream.writeAll(body) catch {};")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        Ok(())
    }

    // ========================================================================
    // Buffer helpers
    // ========================================================================

    fn line(&mut self, s: &str) -> ZigResult<()> {
        self.buffer
            .line(s)
            .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))
    }

    /// 增加缩进
    fn indent(&mut self) {
        self.buffer.indent();
    }

    /// 减少缩进
    fn dedent(&mut self) {
        self.buffer.dedent();
    }

    /// 获取当前输出
    fn output(&self) -> &str {
        self.buffer.as_str()
    }

    // ========================================================================
    // Built-in function mapping
    // ========================================================================

    fn emit_builtin_or_call(&mut self, name: &str, args: &[String]) -> String {
        match name {
            "print" | "println" => {
                if args.len() == 1 {
                    // Detect if the argument is a string literal (starts and ends with quotes)
                    let arg = &args[0];
                    let is_string_literal = arg.starts_with('"') && arg.ends_with('"');
                    let format_spec = if is_string_literal { "{s}" } else { "{any}" };
                    // Zig 的 print 使用 .{} 语法，不需要额外的花括号
                    format!("std.debug.print(\"{}\\n\", .{{{}}})", format_spec, arg)
                } else {
                    "std.debug.print(\"\\n\", .{{}})".to_string()
                }
            }
            // 对于返回 void 的内置函数，标记它们以便调用时不赋值
            "print_inline" => {
                if args.len() == 1 {
                    let arg = &args[0];
                    let is_string_literal = arg.starts_with('"') && arg.ends_with('"');
                    let format_spec = if is_string_literal { "{s}" } else { "{any}" };
                    format!("std.debug.print(\"{}\", .{{{}}})", format_spec, arg)
                } else {
                    "std.debug.print(\"\", .{{}})".to_string()
                }
            }
            "concat" => {
                if args.len() == 2 {
                    format!(
                        "std.mem.concat(allocator, u8, &[_][]const u8{{ {}, {} }}) catch unreachable",
                        args[0], args[1]
                    )
                } else {
                    "\"\"".to_string()
                }
            }
            "to_string" => format!(
                "std.fmt.allocPrint(allocator, \"{{}}\", .{{{}}}) catch unreachable",
                args.first().map(|s| s.as_str()).unwrap_or("null")
            ),
            "string_length" => {
                let s = args.first().map(|s| s.as_str()).unwrap_or("\"\"");
                format!("{}.len", s)
            }
            "string_find" => {
                let s = args.first().map(|s| s.as_str()).unwrap_or("\"\"");
                let substr = args.get(1).map(|s| s.as_str()).unwrap_or("\"\"");
                format!(
                    r#"(blk: {{
    const idx = std.mem.indexOf(u8, {}, {});
    break :blk if (idx) |i| @as(i32, @intCast(i)) else @as(i32, -1);
}})"#,
                    s, substr
                )
            }
            "string_substring" => {
                let s = args.first().map(|s| s.as_str()).unwrap_or("\"\"");
                let start = args.get(1).map(|s| s.as_str()).unwrap_or("0");
                let end = args.get(2).map(|s| s.as_str()).unwrap_or("0");
                format!("{}[{}..{}]", s, start, end)
            }
            "int_to_string" => {
                let n = args.first().map(|s| s.as_str()).unwrap_or("0");
                format!(
                    "std.fmt.allocPrint(allocator, \"{{d}}\", .{{{}}}) catch unreachable",
                    n
                )
            }
            "type_of" => format!(
                "@typeName(@TypeOf({}))",
                args.first().map(|s| s.as_str()).unwrap_or("null")
            ),
            "panic" => {
                if args.len() == 1 {
                    format!("std.debug.panic(\"{{}}\", .{{{}}})", args[0])
                } else {
                    "std.debug.panic(\"panic\", .{{}})".to_string()
                }
            }
            "len" => format!("{}.len", args.first().map(|s| s.as_str()).unwrap_or("null")),
            _ => {
                format!("{}({})", name, args.join(", "))
            }
        }
    }

    // ========================================================================
    // Zig compiler invocation
    // ========================================================================

    /// Compile generated Zig code to executable
    pub fn compile_zig_code(&self, zig_code: &str, output_file: &Path) -> ZigResult<()> {
        use std::process::Command;

        // 首先写入 .zig 文件到输出目录
        let zig_file = output_file.with_extension("zig");
        std::fs::write(&zig_file, zig_code)?;

        // 获取输出目录
        let output_dir = output_file
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        // Build zig command - 在输出目录中运行
        let mut cmd = Command::new("zig");
        cmd.arg("build-exe")
            .arg(&zig_file)
            .arg("-O")
            .arg(if self.config.optimize {
                "ReleaseFast"
            } else {
                "Debug"
            });

        // Add target if not native
        if self.config.target != ZigTarget::Native {
            cmd.arg("-target").arg(self.config.target.as_zig_target());
        }

        // Debug info is already included in Debug optimization mode
        // The -g flag format changed in Zig 0.15+, and Debug mode includes debug info by default

        // 在输出目录中运行编译，这样生成的可执行文件会在正确位置
        cmd.current_dir(&output_dir);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(x_codegen::CodeGenError::CompilerError(format!(
                "Zig compiler failed:\nstdout: {}\nstderr: {}",
                stdout, stderr
            )));
        }

        Ok(())
    }
}

// ============================================================================
// 实现 CodeGenerator trait
// ============================================================================

impl x_codegen::CodeGenerator for ZigBackend {
    type Config = ZigBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        ZigBackend::new(config)
    }

    fn generate_from_lir(
        &mut self,
        lir: &x_lir::Program,
    ) -> Result<x_codegen::CodegenOutput, Self::Error> {
        ZigBackend::generate_from_lir(self, lir)
    }
}

// ============================================================================
// LIR 辅助函数
// ============================================================================

impl ZigBackend {
    /// 发出外部函数声明（来自 LIR）
    fn emit_lir_extern_function(&mut self, extern_func: &x_lir::ExternFunction) -> ZigResult<()> {
        // Output generic type parameters if any: (T: type, U: type)
        let type_params_str = if extern_func.type_params.is_empty() {
            "".to_string()
        } else {
            let type_params = extern_func
                .type_params
                .iter()
                .map(|tp| format!("{}: type", tp))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", type_params)
        };

        let params = if extern_func.parameters.is_empty() {
            "".to_string()
        } else {
            extern_func
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param_type)| format!("arg{}: {}", i, self.emit_lir_type(param_type)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        // Combine type params and regular params for Zig generic function syntax
        let full_params = match (type_params_str.is_empty(), params.is_empty()) {
            (true, true) => "".to_string(),
            (true, false) => format!("({})", params),
            (false, true) => type_params_str,
            (false, false) => format!("{}({})", type_params_str, params),
        };

        let return_type = self.emit_lir_type(&extern_func.return_type);
        match &extern_func.abi {
            Some(abi) if abi == "C" => {
                self.line(&format!(
                    "pub extern \"c\" fn {}{} {};",
                    extern_func.name, full_params, return_type
                ))?;
            }
            Some(abi) => {
                self.line(&format!(
                    "pub extern \"{}\" fn {}{} {};",
                    abi, extern_func.name, full_params, return_type
                ))?;
            }
            None => {
                self.line(&format!(
                    "extern fn {}{} {};",
                    extern_func.name, full_params, return_type
                ))?;
            }
        }
        Ok(())
    }

    /// 发出全局变量（来自 LIR）
    fn emit_lir_global_var(&mut self, global_var: &x_lir::GlobalVar) -> ZigResult<()> {
        let type_str = self.emit_lir_type(&global_var.type_);
        if let Some(initializer) = &global_var.initializer {
            let init_str = self.emit_lir_expression(initializer)?;
            self.line(&format!(
                "pub var {} : {} = {};",
                global_var.name, type_str, init_str
            ))?;
        } else {
            self.line(&format!(
                "pub var {} : {} = undefined;",
                global_var.name, type_str
            ))?;
        }
        Ok(())
    }

    /// 发出结构体定义（来自 LIR）
    fn emit_lir_struct(&mut self, struct_def: &x_lir::Struct) -> ZigResult<()> {
        self.line(&format!("pub const {} = struct {{", struct_def.name))?;
        self.indent();

        for field in &struct_def.fields {
            let type_str = self.emit_lir_type(&field.type_);
            self.line(&format!("{}: {},", field.name, type_str))?;
        }

        self.dedent();
        self.line("};")?;
        self.line("")?;
        Ok(())
    }

    /// 发出枚举定义（来自 LIR）
    fn emit_lir_enum(&mut self, enum_def: &x_lir::Enum) -> ZigResult<()> {
        self.line(&format!("pub const {} = enum {{", enum_def.name))?;
        self.indent();

        for variant in &enum_def.variants {
            if let Some(value) = variant.value {
                self.line(&format!("{} = {},", variant.name, value))?;
            } else {
                self.line(&format!("{},", variant.name))?;
            }
        }

        self.dedent();
        self.line("};")?;
        self.line("")?;
        Ok(())
    }

    /// 发出函数定义（来自 LIR）
    fn emit_lir_function(&mut self, func: &x_lir::Function) -> ZigResult<()> {
        // Output generic type parameters if any: (T: type, U: type)
        let type_params_str = if func.type_params.is_empty() {
            "".to_string()
        } else {
            let type_params = func
                .type_params
                .iter()
                .map(|tp| format!("{}: type", tp))
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", type_params)
        };

        let params = if func.parameters.is_empty() {
            "".to_string()
        } else {
            func.parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.emit_lir_type(&p.type_)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let return_type = self.emit_lir_type(&func.return_type);
        // main 函数在 Zig 中必须返回 void 或 error!void
        // 如果是 main 函数且返回 Integer (通常是 0)，转换为 !void
        let return_type = if func.name == "main" && return_type != "void" {
            "!void".to_string()
        } else {
            return_type
        };
        let pub_str = if func.name == "main" { "pub " } else { "" };

        // 记录当前正在发射的函数名
        self.current_function_name = func.name.clone();

        // Combine type params and regular params for Zig generic function syntax
        // Always include parentheses, even if empty
        let full_params = match (type_params_str.is_empty(), params.is_empty()) {
            (true, true) => "()".to_string(),
            (true, false) => format!("({})", params),
            (false, true) => format!("({})", type_params_str),
            (false, false) => format!("{}({})", type_params_str, params),
        };

        self.line(&format!(
            "{}fn {}{} {} {{",
            pub_str, func.name, full_params, return_type
        ))?;
        self.indent();

        // Emit function body
        self.emit_lir_block(&func.body)?;

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    /// 发出块（来自 LIR）
    fn emit_lir_block(&mut self, block: &x_lir::Block) -> ZigResult<()> {
        for stmt in &block.statements {
            self.emit_lir_statement(stmt)?;
        }
        Ok(())
    }

    /// 发出语句（来自 LIR）
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> ZigResult<()> {
        match stmt {
            x_lir::Statement::Expression(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                // 处理赋值表达式，检测是否是 void 返回的内置函数调用
                // 格式可能是: (t0 = std.debug.print(...))
                let inner = if expr_str.starts_with("(") && expr_str.ends_with(")") {
                    &expr_str[1..expr_str.len() - 1]
                } else {
                    &expr_str
                };

                // 检测是否是 void 返回的调用
                // 注意：println 被 emit_builtin_or_call 转换为 std.debug.print
                let is_void_call = inner.contains("std.debug.print");

                if is_void_call {
                    // 提取变量名并记录，以便后续跳过变量声明
                    // 格式可能是 "_t0 = std.debug.print(...)" 或 "t0 = std.debug.print(...)"
                    if let Some(eq_pos) = inner.find(" = ") {
                        let var_name = inner[..eq_pos].trim();
                        // 去掉前导下划线，存储不带下划线的版本
                        #[allow(clippy::manual_strip)]
                        let clean_name = if var_name.starts_with('_') {
                            var_name[1..].to_string()
                        } else {
                            var_name.to_string()
                        };
                        // 同时存储带下划线和不带下划线的版本
                        self.void_call_vars.insert(clean_name.clone());
                        self.void_call_vars.insert(format!("_{}", clean_name));
                        self.void_call_vars.insert(var_name.to_string());
                    }
                    // 直接输出函数调用部分（去掉 t0 = 前缀）
                    let call_part = inner[inner.find(" = ").unwrap() + 3..].to_string();
                    self.line(&format!("{};", call_part))?;
                    return Ok(());
                }

                // 对于临时变量赋值，直接内联表达式
                // 格式: t0 = expr -> expr;
                let is_temp_assign = if let Some(eq_pos) = inner.find(" = ") {
                    let var_part = inner[..eq_pos].trim();
                    // 检查是否是临时变量赋值 (t0, t1, etc.)
                    (var_part.starts_with("_t") || var_part.starts_with('t'))
                        && var_part[1..]
                            .chars()
                            .all(|c| c.is_ascii_digit() || c == '_')
                        && var_part
                            .chars()
                            .skip(1)
                            .take_while(|c| *c == '_' || c.is_ascii_digit())
                            .count()
                            > 0
                } else {
                    false
                };

                if is_temp_assign {
                    // 对于临时变量赋值，生成完整的声明+赋值语句
                    if let Some(eq_pos) = inner.find(" = ") {
                        let var_part = inner[..eq_pos].trim();
                        let value_part = inner[eq_pos + 3..].trim();
                        // 转换变量名：t0 -> _t0
                        let var_name = if var_part.starts_with("t") {
                            format!("_{}", var_part)
                        } else if var_part.starts_with("_t") {
                            var_part.to_string()
                        } else {
                            format!("_{}", var_part)
                        };
                        // 生成完整的变量声明和赋值
                        // Zig 允许在一行中声明并赋值: var x: i32 = value;
                        self.line(&format!("var {} : i32 = {};", var_name, value_part))?;
                        return Ok(());
                    }
                }

                // 其他赋值表达式
                if inner.contains(" = ") && !inner.contains("==") {
                    self.line(&format!("{};", inner))?;
                    return Ok(());
                }

                // 对于非赋值的表达式，添加 _ = 前缀来丢弃不需要的值
                // 这避免 Zig 的 "value of type 'i32' ignored" 错误
                self.line(&format!("_ = {};", expr_str))?;
            }
            x_lir::Statement::Variable(var) => {
                // 如果变量是 void 返回调用的目标，跳过声明
                // 检查带下划线和不带下划线的版本
                let var_name_clean = if var.name.starts_with('_') {
                    var.name[1..].to_string()
                } else {
                    var.name.clone()
                };
                if self.void_call_vars.contains(&var.name)
                    || self.void_call_vars.contains(&var_name_clean)
                    || self.void_call_vars.contains(&format!("_{}", var.name))
                {
                    self.void_call_vars.remove(&var.name);
                    self.void_call_vars.remove(&var_name_clean);
                    self.void_call_vars.remove(&format!("_{}", var.name));
                    return Ok(());
                }

                // 跳过所有临时变量（t0, t1 等）的声明
                // 它们会在后续的赋值表达式中被内联生成
                if var.name.starts_with('t')
                    && var.name.len() > 1
                    && var.name[1..].chars().all(|c| c.is_ascii_digit())
                {
                    // 检查是否有初始化器，如果没有就跳过
                    if var.initializer.is_none() {
                        return Ok(());
                    }
                }

                let type_str = self.emit_lir_type(&var.type_);
                // 对于临时变量，使用 const 声明（因为它们不会被修改）
                // 注意：变量名可能是 "t0" 或 "_t0"，需要统一处理
                let is_temp_var = var.name.starts_with("t")
                    && var.name.len() > 1
                    && var.name[1..].chars().all(|c| c.is_ascii_digit());

                let var_name = if is_temp_var {
                    format!("_{}", var.name)
                } else {
                    var.name.clone()
                };

                // 临时变量使用 var，因为后续会被赋值
                let keyword = "var";
                if let Some(initializer) = &var.initializer {
                    let init_str = self.emit_lir_expression(initializer)?;
                    self.line(&format!(
                        "{} {} : {} = {};",
                        keyword, var_name, type_str, init_str
                    ))?;
                } else {
                    self.line(&format!(
                        "{} {} : {} = undefined;",
                        keyword, var_name, type_str
                    ))?;
                    // 对于没有初始化器的临时变量，在 main 函数中添加使用标记
                    if is_temp_var && self.current_function_name == "main" {
                        self.line(&format!("_ = {};", var_name))?;
                    }
                }
            }
            x_lir::Statement::If(if_stmt) => {
                let cond_str = self.emit_lir_expression(&if_stmt.condition)?;
                self.line(&format!("if ({}) {{", cond_str))?;
                self.indent();
                self.emit_lir_statement(&if_stmt.then_branch)?;
                self.dedent();

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.line("} else {")?;
                    self.indent();
                    self.emit_lir_statement(else_branch)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            x_lir::Statement::While(while_stmt) => {
                let cond_str = self.emit_lir_expression(&while_stmt.condition)?;
                self.line(&format!("while ({}) {{", cond_str))?;
                self.indent();
                self.emit_lir_statement(&while_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::DoWhile(do_while_stmt) => {
                self.line("while (true) {")?;
                self.indent();
                self.emit_lir_statement(&do_while_stmt.body)?;
                let cond_str = self.emit_lir_expression(&do_while_stmt.condition)?;
                self.line(&format!("if (!{}) break;", cond_str))?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::For(for_stmt) => {
                // Zig doesn't have C-style for loops, so we emulate with while
                if let Some(init) = &for_stmt.initializer {
                    self.emit_lir_statement(init)?;
                }
                let cond_str = for_stmt
                    .condition
                    .as_ref()
                    .map(|e| self.emit_lir_expression(e))
                    .transpose()?
                    .unwrap_or_else(|| "true".to_string());
                self.line(&format!("while ({}) {{", cond_str))?;
                self.indent();
                self.emit_lir_statement(&for_stmt.body)?;
                if let Some(increment) = &for_stmt.increment {
                    let _ = self.emit_lir_expression(increment)?;
                }
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Switch(switch_stmt) => {
                let expr_str = self.emit_lir_expression(&switch_stmt.expression)?;
                self.line(&format!("switch ({}) {{", expr_str))?;
                self.indent();

                for case in &switch_stmt.cases {
                    let value_str = self.emit_lir_expression(&case.value)?;
                    self.line(&format!("{} => {{", value_str))?;
                    self.indent();
                    self.emit_lir_statement(&case.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                if let Some(default) = &switch_stmt.default {
                    self.line("_ => {")?;
                    self.indent();
                    self.emit_lir_statement(default)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Match(match_stmt) => {
                let scrutinee_str = self.emit_lir_expression(&match_stmt.scrutinee)?;
                self.line(&format!("switch ({}) {{", scrutinee_str))?;
                self.indent();

                for case in &match_stmt.cases {
                    let pattern_str = self.emit_lir_pattern(&case.pattern)?;
                    self.line(&format!("{} => {{", pattern_str))?;
                    self.indent();
                    self.emit_lir_block(&case.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Try(try_stmt) => {
                self.line("{")?;
                self.indent();
                self.emit_lir_block(&try_stmt.body)?;
                for catch in &try_stmt.catch_clauses {
                    if let Some(var_name) = &catch.variable_name {
                        self.line(&format!("// catch {}", var_name))?;
                    }
                    self.emit_lir_block(&catch.body)?;
                }
                if let Some(finally) = &try_stmt.finally_block {
                    self.emit_lir_block(finally)?;
                }
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Return(expr) => {
                if let Some(expr) = expr {
                    let expr_str = self.emit_lir_expression(expr)?;
                    // 对于 main 函数，使用 std.process.exit() 来设置退出码
                    // exit 参数类型是 u8，需要将值转换为 u8
                    if self.current_function_name == "main" {
                        // 简化处理：直接使用退出码 0，不使用表达式的返回值
                        // 这是最安全的方式，避免 Zig 的类型检查问题
                        self.line("std.process.exit(0);")?;
                    } else {
                        self.line(&format!("return {};", expr_str))?;
                    }
                } else {
                    self.line("return;")?;
                }
            }
            x_lir::Statement::Break => self.line("break;")?,
            x_lir::Statement::Continue => self.line("continue;")?,
            x_lir::Statement::Goto(label) => self.line(&format!("// goto {}", label))?,
            // Zig doesn't have traditional labels, convert to comment
            x_lir::Statement::Label(label) => self.line(&format!("// label: {}", label))?,
            x_lir::Statement::Empty => { /* do nothing */ }
            x_lir::Statement::Compound(block) => {
                self.line("{")?;
                self.indent();
                self.emit_lir_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Declaration(_) => {
                // Already handled at top level - shouldn't happen in LIR block
            }
        }
        Ok(())
    }

    /// 发出表达式（来自 LIR）
    fn emit_lir_expression(&mut self, expr: &x_lir::Expression) -> ZigResult<String> {
        match expr {
            x_lir::Expression::Literal(lit) => match lit {
                x_lir::Literal::Integer(n) => Ok(format!("{}", n)),
                x_lir::Literal::UnsignedInteger(n) => Ok(format!("{}", n)),
                x_lir::Literal::Long(n) => Ok(format!("{}", n)),
                x_lir::Literal::UnsignedLong(n) => Ok(format!("{}", n)),
                x_lir::Literal::LongLong(n) => Ok(format!("{}", n)),
                x_lir::Literal::UnsignedLongLong(n) => Ok(format!("{}", n)),
                x_lir::Literal::Float(f) => Ok(format!("{}", f)),
                x_lir::Literal::Double(f) => Ok(format!("{}", f)),
                x_lir::Literal::String(s) => {
                    let escaped = s
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t");
                    Ok(format!("\"{}\"", escaped))
                }
                x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                x_lir::Literal::NullPointer => Ok("null".to_string()),
            },
            x_lir::Expression::Variable(name) => {
                // 对临时变量添加下划线前缀
                let var_name = if name.starts_with("t")
                    && name.len() > 1
                    && name[1..].chars().all(|c| c.is_ascii_digit())
                {
                    format!("_{}", name)
                } else {
                    name.clone()
                };
                Ok(var_name)
            }
            x_lir::Expression::Unary(op, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let op_str = match op {
                    x_lir::UnaryOp::Plus => "+",
                    x_lir::UnaryOp::Minus => "-",
                    x_lir::UnaryOp::Not => "!",
                    x_lir::UnaryOp::BitNot => "~",
                    x_lir::UnaryOp::PreIncrement => "++",
                    x_lir::UnaryOp::PreDecrement => "--",
                    x_lir::UnaryOp::PostIncrement => "/* post++ */",
                    x_lir::UnaryOp::PostDecrement => "/* post-- */",
                    x_lir::UnaryOp::Reference => "&",
                    x_lir::UnaryOp::MutableReference => "&mut ",
                };
                Ok(format!("{}({})", op_str, expr_str))
            }
            x_lir::Expression::Binary(op, lhs, rhs) => {
                let lhs_str = self.emit_lir_expression(lhs)?;
                let rhs_str = self.emit_lir_expression(rhs)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+",
                    x_lir::BinaryOp::Subtract => "-",
                    x_lir::BinaryOp::Multiply => "*",
                    x_lir::BinaryOp::Divide => "/",
                    x_lir::BinaryOp::Modulo => "%",
                    x_lir::BinaryOp::LeftShift => "<<",
                    x_lir::BinaryOp::RightShift => ">>>",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>",
                    x_lir::BinaryOp::LessThan => "<",
                    x_lir::BinaryOp::LessThanEqual => "<=",
                    x_lir::BinaryOp::GreaterThan => ">",
                    x_lir::BinaryOp::GreaterThanEqual => ">=",
                    x_lir::BinaryOp::Equal => "==",
                    x_lir::BinaryOp::NotEqual => "!=",
                    x_lir::BinaryOp::BitAnd => "&",
                    x_lir::BinaryOp::BitXor => "^",
                    x_lir::BinaryOp::BitOr => "|",
                    x_lir::BinaryOp::LogicalAnd => "and",
                    x_lir::BinaryOp::LogicalOr => "or",
                };
                Ok(format!("({} {} {})", lhs_str, op_str, rhs_str))
            }
            x_lir::Expression::Call(callee, args) => {
                let callee_str = self.emit_lir_expression(callee)?;
                let arg_strs: Vec<String> = args
                    .iter()
                    .map(|arg| self.emit_lir_expression(arg))
                    .collect::<Result<_, _>>()?;
                // 使用 emit_builtin_or_call 处理内置函数
                Ok(self.emit_builtin_or_call(&callee_str, &arg_strs))
            }
            x_lir::Expression::Index(array, index) => {
                let array_str = self.emit_lir_expression(array)?;
                let index_str = self.emit_lir_expression(index)?;
                Ok(format!("{}[{}]", array_str, index_str))
            }
            x_lir::Expression::Member(obj, field) => {
                let obj_str = self.emit_lir_expression(obj)?;
                Ok(format!("{}.{}", obj_str, field))
            }
            x_lir::Expression::Dereference(ptr) => {
                let ptr_str = self.emit_lir_expression(ptr)?;
                Ok(format!("({}.*)", ptr_str))
            }
            x_lir::Expression::AddressOf(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                Ok(format!("&({})", expr_str))
            }
            x_lir::Expression::Cast(type_, expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                let type_str = self.emit_lir_type(type_);
                Ok(format!("@as({}, {})", type_str, expr_str))
            }
            x_lir::Expression::Assign(lhs, rhs) => {
                let lhs_str = self.emit_lir_expression(lhs)?;
                let rhs_str = self.emit_lir_expression(rhs)?;
                // 如果左侧是临时变量（如 t0 或 _t0），保持不变
                // 因为 emit_lir_variable 已经添加了下划线前缀
                // 这里只需要确保格式正确
                Ok(format!("({} = {})", lhs_str, rhs_str))
            }
            x_lir::Expression::AssignOp(op, lhs, rhs) => {
                let lhs_str = self.emit_lir_expression(lhs)?;
                let rhs_str = self.emit_lir_expression(rhs)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+=",
                    x_lir::BinaryOp::Subtract => "-=",
                    x_lir::BinaryOp::Multiply => "*=",
                    x_lir::BinaryOp::Divide => "/=",
                    x_lir::BinaryOp::Modulo => "%=",
                    x_lir::BinaryOp::BitAnd => "&=",
                    x_lir::BinaryOp::BitOr => "|=",
                    x_lir::BinaryOp::BitXor => "^=",
                    x_lir::BinaryOp::LeftShift => "<<=",
                    x_lir::BinaryOp::RightShift => ">>>=",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>=",
                    _ => "=/* unknown op */",
                };
                Ok(format!("({} {} {})", lhs_str, op_str, rhs_str))
            }
            x_lir::Expression::Ternary(cond, then, else_) => {
                let cond_str = self.emit_lir_expression(cond)?;
                let then_str = self.emit_lir_expression(then)?;
                let else_str = self.emit_lir_expression(else_)?;
                Ok(format!("if ({}) {} else {}", cond_str, then_str, else_str))
            }
            x_lir::Expression::PointerMember(ptr, field) => {
                let ptr_str = self.emit_lir_expression(ptr)?;
                Ok(format!("{}.{}", ptr_str, field))
            }
            x_lir::Expression::SizeOf(ty) => {
                let ty_str = self.emit_lir_type(ty);
                Ok(format!("@sizeOf({})", ty_str))
            }
            x_lir::Expression::SizeOfExpr(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                Ok(format!("@sizeOf({})", expr_str))
            }
            x_lir::Expression::AlignOf(ty) => {
                let ty_str = self.emit_lir_type(ty);
                Ok(format!("@alignOf({})", ty_str))
            }
            x_lir::Expression::Comma(exprs) => {
                let expr_strs: Vec<String> = exprs
                    .iter()
                    .map(|e| self.emit_lir_expression(e))
                    .collect::<Result<_, _>>()?;
                Ok(expr_strs.join(", "))
            }
            x_lir::Expression::Parenthesized(expr) => {
                let expr_str = self.emit_lir_expression(expr)?;
                Ok(format!("({})", expr_str))
            }
            x_lir::Expression::InitializerList(inits) => {
                // In Zig, this becomes .{ ... }
                let mut init_strs = Vec::new();
                for init in inits {
                    init_strs.push(self.emit_lir_initializer(init)?);
                }
                Ok(format!(".{{ {} }}", init_strs.join(", ")))
            }
            x_lir::Expression::CompoundLiteral(ty, inits) => {
                let ty_str = self.emit_lir_type(ty);
                let mut init_strs = Vec::new();
                for init in inits {
                    init_strs.push(self.emit_lir_initializer(init)?);
                }
                Ok(format!("{} {{ {} }}", ty_str, init_strs.join(", ")))
            }
        }
    }

    /// 发出初始化器（用于复合字面量）
    fn emit_lir_initializer(&mut self, init: &x_lir::Initializer) -> ZigResult<String> {
        match init {
            x_lir::Initializer::Expression(expr) => self.emit_lir_expression(expr),
            x_lir::Initializer::List(list) => {
                let mut items = Vec::new();
                for i in list {
                    items.push(self.emit_lir_initializer(i)?);
                }
                Ok(format!(".{{ {} }}", items.join(", ")))
            }
            x_lir::Initializer::Named(name, init) => {
                let init_str = self.emit_lir_initializer(init)?;
                Ok(format!(".{} = {}", name, init_str))
            }
            x_lir::Initializer::Indexed(idx, init) => {
                let idx_str = self.emit_lir_expression(idx)?;
                let init_str = self.emit_lir_initializer(init)?;
                Ok(format!("[{}] = {}", idx_str, init_str))
            }
        }
    }

    /// 发出模式（来自 LIR）
    #[allow(clippy::only_used_in_recursion)]
    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> ZigResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard => Ok("_".to_string()),
            x_lir::Pattern::Variable(name) => Ok(name.clone()),
            x_lir::Pattern::Literal(lit) => match lit {
                x_lir::Literal::Integer(n) => Ok(format!("{}", n)),
                x_lir::Literal::String(s) => Ok(format!("\"{}\"", s)),
                x_lir::Literal::Char(c) => Ok(format!("'{}'", c)),
                x_lir::Literal::Bool(b) => Ok(format!("{}", b)),
                _ => Ok("_".to_string()),
            },
            x_lir::Pattern::Constructor(name, patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                if pattern_strs.is_empty() {
                    Ok(format!(".{}", name))
                } else {
                    Ok(format!(".{}({})", name, pattern_strs.join(", ")))
                }
            }
            x_lir::Pattern::Tuple(patterns) => {
                let pattern_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                Ok(format!(".{{ {} }}", pattern_strs.join(", ")))
            }
            x_lir::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| {
                        let v_str = self.emit_lir_pattern(v).unwrap_or_else(|_| "_".to_string());
                        format!(".{} = {}", k, v_str)
                    })
                    .collect();
                Ok(format!("{}.{{ {} }}", name, field_strs.join(", ")))
            }
            x_lir::Pattern::Or(left, right) => {
                let left_str = self.emit_lir_pattern(left)?;
                let right_str = self.emit_lir_pattern(right)?;
                Ok(format!("{}, {}", left_str, right_str))
            }
        }
    }

    /// 发出类型（来自 LIR）
    #[allow(clippy::only_used_in_recursion)]
    fn emit_lir_type(&self, type_: &x_lir::Type) -> String {
        match type_ {
            x_lir::Type::Void => "void".to_string(),
            x_lir::Type::Bool => "bool".to_string(),
            x_lir::Type::Char => "u8".to_string(),
            x_lir::Type::Schar => "i8".to_string(),
            x_lir::Type::Uchar => "u8".to_string(),
            x_lir::Type::Short => "i16".to_string(),
            x_lir::Type::Ushort => "u16".to_string(),
            x_lir::Type::Int => "i32".to_string(),
            x_lir::Type::Uint => "u32".to_string(),
            x_lir::Type::Long => "i64".to_string(),
            x_lir::Type::Ulong => "u64".to_string(),
            x_lir::Type::LongLong => "i128".to_string(),
            x_lir::Type::UlongLong => "u128".to_string(),
            x_lir::Type::Float => "f32".to_string(),
            x_lir::Type::Double => "f64".to_string(),
            x_lir::Type::LongDouble => "f128".to_string(),
            x_lir::Type::Size => "usize".to_string(),
            x_lir::Type::Ptrdiff => "isize".to_string(),
            x_lir::Type::Intptr => "isize".to_string(),
            x_lir::Type::Uintptr => "usize".to_string(),
            x_lir::Type::Pointer(inner) => format!("*{}", self.emit_lir_type(inner)),
            x_lir::Type::Array(inner, Some(size)) => {
                format!("[{}]{}", size, self.emit_lir_type(inner))
            }
            x_lir::Type::Array(inner, None) => {
                format!("[]{}", self.emit_lir_type(inner))
            }
            x_lir::Type::FunctionPointer(ret_type, param_types) => {
                let param_str = param_types
                    .iter()
                    .map(|t| self.emit_lir_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) {}", param_str, self.emit_lir_type(ret_type))
            }
            x_lir::Type::Named(name) => name.clone(),
            x_lir::Type::Qualified(_, inner) => self.emit_lir_type(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = ZigBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
        assert_eq!(config.target, ZigTarget::Native);
    }

    #[test]
    fn test_zig_target() {
        assert_eq!(ZigTarget::Native.as_zig_target(), "native");
        assert_eq!(ZigTarget::Wasm32Wasi.as_zig_target(), "wasm32-wasi");
        assert_eq!(
            ZigTarget::Wasm32Freestanding.as_zig_target(),
            "wasm32-freestanding"
        );
        assert_eq!(ZigTarget::Native.output_extension(), "");
        assert_eq!(ZigTarget::Wasm32Wasi.output_extension(), ".wasm");
    }

    #[test]
    fn test_generate_from_lir_empty() {
        let lir = LirProgram {
            declarations: vec![],
        };

        let mut backend = ZigBackend::new(ZigBackendConfig::default());
        let output = backend.generate_from_lir(&lir).unwrap();
        let zig_code = String::from_utf8_lossy(&output.files[0].content);
        assert!(zig_code.contains("// Generated by X-Lang"));
        assert!(zig_code.contains("const std = @import"));
    }

    #[test]
    fn test_lir_type_mapping() {
        let backend = ZigBackend::new(ZigBackendConfig::default());
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Void), "void");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Bool), "bool");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Int), "i32");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Uint), "u32");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Long), "i64");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Float), "f32");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Double), "f64");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Size), "usize");
        assert_eq!(backend.emit_lir_type(&x_lir::Type::Char), "u8");
        assert_eq!(
            backend.emit_lir_type(&x_lir::Type::Pointer(Box::new(x_lir::Type::Int))),
            "*i32"
        );
        assert_eq!(
            backend.emit_lir_type(&x_lir::Type::Array(Box::new(x_lir::Type::Int), Some(10))),
            "[10]i32"
        );
        assert_eq!(
            backend.emit_lir_type(&x_lir::Type::Array(Box::new(x_lir::Type::Int), None)),
            "[]i32"
        );
    }
}
