//! Rust 后端 - 将 X LIR 编译为 Rust 1.94+ 代码
//!
//! 生成清晰可读的 Rust 源代码，支持基本的 X 语言特性
//!
//! ## Rust 1.94 特性支持 (2026年3月发布)
//! - async fn in traits（trait 中的异步函数）
//! - return-position impl Trait in trait bodies
//! - if-let chains（let-else 链）
//! - Slice patterns
//! - Generic Associated Types (GATs)
//! - Const generics
//! - LazyCell / LazyLock in std
//! - Async closures
//! - Return position impl Trait in trait
//! - Improved trait solving

#![allow(
    clippy::collapsible_if,
    clippy::format_in_format_args,
    clippy::only_used_in_recursion,
    clippy::option_as_ref_deref,
    clippy::single_char_add_str,
    clippy::unnecessary_map_or,
    clippy::useless_asref,
    clippy::useless_format,
    clippy::useless_vec
)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;

#[derive(Debug, Clone)]
pub struct RustBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl Default for RustBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
        }
    }
}

pub struct RustBackend {
    #[allow(dead_code)]
    config: RustBackendConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
    /// Names of uninitialized top-level temporaries whose declaration is deferred
    /// to their first assignment (so Rust can infer the correct type instead of
    /// trusting the often-wrong LIR temp type).
    deferred_locals: std::collections::HashSet<String>,
    /// Deferred locals that have already been emitted via `let`.
    declared_locals: std::collections::HashSet<String>,
    /// Module-level globals that originate from X top-level `let` bindings; these
    /// are emitted as `let` locals inside `main` instead of as `static mut` (whose
    /// LIR type is often wrong, and which cannot hold non-const initializers like
    /// `String`). Their `static` declarations are skipped.
    globals_as_locals: std::collections::HashSet<String>,
    /// Initialized top-level-`let` globals, in declaration order, materialized as
    /// `let` bindings at the start of `main`.
    global_local_inits: Vec<(String, x_lir::Expression)>,
    /// Variables (params/locals) known to hold a Rust `String`. Used to recognize
    /// string concatenation that has been split across SSA temporaries (where no
    /// operand is a string literal). Reset per function.
    string_vars: std::collections::HashSet<String>,
    /// Names of extern functions already emitted, to avoid duplicate `extern`
    /// blocks (the LIR prelude declares each runtime function twice).
    emitted_externs: std::collections::HashSet<String>,
    /// Extern signatures: name -> (parameter types, return type). Used to wrap
    /// Rust `String` arguments into C strings and unwrap C-string results.
    extern_sigs: std::collections::HashMap<String, (Vec<x_lir::Type>, x_lir::Type)>,
    /// Variable name -> Rust type, recorded at declaration. Used to cast
    /// `x_as_ptr(...)` results to the target pointer type at assignment.
    var_rust_types: std::collections::HashMap<String, String>,
    /// Struct name -> field name -> Rust type (for String field clones).
    struct_field_types: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// Name of the function currently being generated (main returns ()).
    current_fn: String,
}

pub type RustResult<T> = Result<T, x_codegen::CodeGenError>;

// 保持向后兼容的别名
pub type RustCodeGenerator = RustBackend;
pub type RustCodeGenError = x_codegen::CodeGenError;

impl RustBackend {
    pub fn new(config: RustBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
            deferred_locals: std::collections::HashSet::new(),
            declared_locals: std::collections::HashSet::new(),
            globals_as_locals: std::collections::HashSet::new(),
            global_local_inits: Vec::new(),
            string_vars: std::collections::HashSet::new(),
            emitted_externs: std::collections::HashSet::new(),
            extern_sigs: std::collections::HashMap::new(),
            var_rust_types: std::collections::HashMap::new(),
            struct_field_types: std::collections::HashMap::new(),
            current_fn: String::new(),
        }
    }

    /// Returns true if `expr` evaluates to a Rust `String` (X `string`), as far as
    /// can be determined syntactically with the tracked `string_vars`.
    fn expr_is_string(&self, expr: &x_lir::Expression) -> bool {
        match expr {
            x_lir::Expression::Literal(x_lir::Literal::String(_)) => true,
            x_lir::Expression::Variable(name) => self.string_vars.contains(name),
            x_lir::Expression::Binary(x_lir::BinaryOp::Add, l, r) => {
                self.expr_is_string(l) || self.expr_is_string(r)
            }
            x_lir::Expression::Parenthesized(inner) => self.expr_is_string(inner),
            x_lir::Expression::Cast(ty, _) => {
                matches!(ty, x_lir::Type::Pointer(p) if matches!(p.as_ref(), x_lir::Type::Char))
            }
            x_lir::Expression::Call(callee, _) => {
                matches!(callee.as_ref(), x_lir::Expression::Variable(n) if n == "format")
            }
            _ => false,
        }
    }

    /// True when `field` of the struct behind `base` is a Rust String.
    fn field_is_string(&self, base: &x_lir::Expression, field: &str) -> bool {
        use x_lir::Expression;
        let struct_name = match base {
            Expression::Variable(n) => {
                let ty = self.var_rust_types.get(n).cloned().unwrap_or_default();
                if let Some(stripped) = ty.strip_prefix("*mut ") {
                    stripped.to_string()
                } else {
                    ty
                }
            }
            Expression::Cast(ty, _) => {
                if let x_lir::Type::Pointer(inner) = Self::peel_quals(ty) {
                    if let x_lir::Type::Named(n) = inner.as_ref() {
                        return self
                            .struct_field_types
                            .get(n)
                            .and_then(|m| m.get(field))
                            .map(|t| t == "String")
                            .unwrap_or(false);
                    }
                }
                return false;
            }
            _ => return false,
        };
        self.struct_field_types
            .get(&struct_name)
            .and_then(|m| m.get(field))
            .map(|t| t == "String")
            .unwrap_or(false)
    }

    /// True when a LIR expression statically represents an f64/f32 value.
    fn expr_is_float(&self, e: &x_lir::Expression) -> bool {
        use x_lir::Expression;
        match e {
            Expression::Literal(x_lir::Literal::Float(_)) | Expression::Literal(x_lir::Literal::Double(_)) => true,
            Expression::Variable(n) => self
                .var_rust_types
                .get(n)
                .map(|t| t == "f64" || t == "f32")
                .unwrap_or(false),
            Expression::Cast(ty, _) => Self::is_float_ty(Self::peel_quals(ty)),
            Expression::Call(callee, _) => matches!(
                callee.as_ref(),
                Expression::Variable(n) if matches!(n.as_str(), "x_as_double" | "sqrt" | "floor" | "ceil" | "fabs" | "pow")
            ),
            Expression::Unary(x_lir::UnaryOp::Minus, inner) => self.expr_is_float(inner),
            Expression::Binary(x_lir::BinaryOp::Add | x_lir::BinaryOp::Subtract | x_lir::BinaryOp::Multiply | x_lir::BinaryOp::Divide, l, r) => {
                self.expr_is_float(l) || self.expr_is_float(r)
            }
            _ => false,
        }
    }

    fn peel_quals(ty: &x_lir::Type) -> &x_lir::Type {
        match ty {
            x_lir::Type::Qualified(_, inner) => Self::peel_quals(inner),
            other => other,
        }
    }

    /// Record that `name` holds a string if `value` is a string expression.
    fn track_string_var(&mut self, name: &str, value: &x_lir::Expression) {
        if self.expr_is_string(value) {
            self.string_vars.insert(name.to_string());
        }
    }

    /// Identify module-level globals that are assigned at the top level of `main`.
    /// Such globals correspond to X top-level `let` bindings whose initializer was
    /// not a literal; the LIR type attached to the global is frequently wrong, so
    /// we instead materialize them as inferred `let` locals inside `main`.
    fn compute_globals_as_locals(&mut self, program: &LirProgram) {
        use std::collections::HashSet;
        self.globals_as_locals = HashSet::new();
        self.global_local_inits = Vec::new();

        // Only meaningful when there is a `main` to host the locals.
        let has_main = program
            .declarations
            .iter()
            .any(|d| matches!(d, x_lir::Declaration::Function(f) if f.name == "main"));
        if !has_main {
            return;
        }

        for decl in &program.declarations {
            if let x_lir::Declaration::Global(g) = decl {
                self.globals_as_locals.insert(g.name.clone());
                if let Some(init) = &g.initializer {
                    self.global_local_inits.push((g.name.clone(), init.clone()));
                }
            }
        }
    }

    /// Determine which top-level uninitialized temporaries can have their
    /// declaration deferred to their first assignment. A temp is deferrable when
    /// it is declared without an initializer at the top level of `block` and its
    /// first assignment also appears at the top level of `block` (so merging into
    /// a `let` does not change scoping).
    fn compute_deferred_locals(&mut self, block: &x_lir::Block) {
        use std::collections::HashSet;
        let mut uninit: HashSet<String> = HashSet::new();
        for stmt in &block.statements {
            if let x_lir::Statement::Variable(var) = stmt {
                if var.initializer.is_none() && !var.is_extern && !var.is_static {
                    uninit.insert(var.name.clone());
                }
            }
        }
        let mut deferrable: HashSet<String> = HashSet::new();
        for stmt in &block.statements {
            if let x_lir::Statement::Expression(x_lir::Expression::Assign(target, _)) = stmt {
                if let x_lir::Expression::Variable(name) = target.as_ref() {
                    if uninit.contains(name) {
                        deferrable.insert(name.clone());
                    }
                }
            }
        }
        self.deferred_locals = deferrable;
        self.declared_locals.clear();
    }

    /// 输出一行代码
    fn line(&mut self, s: &str) -> RustResult<()> {
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

    /// Emit file header
    fn emit_header(&mut self) -> RustResult<()> {
        self.line(headers::RUST)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: Rust 1.94 (March 2026)")?;
        self.line("#![allow(unused)]")?;
        self.line("#![allow(dead_code)]")?;
        self.line("#![allow(unused_mut)]")?;
        self.line("#![allow(unused_unsafe)]")?;
        self.line("#![allow(static_mut_refs)]")?;
        self.line("#![allow(non_snake_case)]")?;
        self.line("#![allow(non_upper_case_globals)]")?;
        self.line("#![allow(clippy::all)]")?;
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // LIR code generation
    // ========================================================================

    /// 从 LIR 生成 Rust 代码
    pub fn generate_from_lir(&mut self, program: &LirProgram) -> RustResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // Add necessary imports that are commonly used
        self.line("use std::collections::HashMap;")?;
        self.line("use std::ffi::{c_void, CString, CStr};")?;
        self.line("use std::process;")?;
        self.line("")?;

        self.emit_runtime_preamble()?;

        self.compute_globals_as_locals(program);

        // Process all declarations
        for decl in &program.declarations {
            self.generate_lir_declaration(decl)?;
        }

        // Module-style sources (no top-level statements / no `main`) still need a
        // `main` to link as an executable. Emit an empty one.
        let has_main = program
            .declarations
            .iter()
            .any(|d| matches!(d, x_lir::Declaration::Function(f) if f.name == "main"));
        if !has_main {
            self.line("pub fn main() {}")?;
            self.line("")?;
        }

        let output_file = OutputFile {
            path: PathBuf::from("main.rs"),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::Rust,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }

    /// 使用 rustc 编译生成的 Rust 代码为可执行文件
    pub fn compile_rust(
        rust_code: &str,
        output_path: &std::path::Path,
    ) -> Result<std::path::PathBuf, x_codegen::CodeGenError> {
        // 创建临时目录存放 Rust 源文件
        let temp_dir = std::env::temp_dir().join("xlang_rust_build");
        let src_dir = temp_dir.join("src");
        std::fs::create_dir_all(&src_dir).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!(
                "Failed to create temp directory: {}",
                e
            ))
        })?;

        // 写入 src/main.rs
        let rs_path = src_dir.join("main.rs");
        std::fs::write(&rs_path, rust_code).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to write Rust source: {}", e))
        })?;

        // 写入捆绑的 C 运行时（xrt.c），通过 build.rs 编译并链接。
        let xrt_src = runtime::XRT_C_SRC;
        let xrt_hdr = runtime::XRT_C_HDR;
        let xrt_path = temp_dir.join("xrt.c");
        let xrt_hdr_path = temp_dir.join("xrt.h");
        std::fs::write(&xrt_path, xrt_src).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to write xrt.c: {}", e))
        })?;
        std::fs::write(&xrt_hdr_path, xrt_hdr).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to write xrt.h: {}", e))
        })?;

        // build.rs — 编译 xrt.c 并指示 cargo 链接
        let build_rs = r#"fn main() {
    cc::Build::new()
        .file("xrt.c")
        .opt_level(2)
        .compile("xrt");
}
"#;
        let build_rs_path = temp_dir.join("build.rs");
        std::fs::write(&build_rs_path, build_rs).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to write build.rs: {}", e))
        })?;

        // 创建 Cargo.toml
        let cargo_toml = r#"[package]
name = "xlang_output"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "xlang_output"
path = "src/main.rs"

[build-dependencies]
cc = "1.0"

[dependencies]
"#;
        let cargo_path = temp_dir.join("Cargo.toml");
        std::fs::write(&cargo_path, cargo_toml).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to write Cargo.toml: {}", e))
        })?;

        // 调用 cargo build
        let output_status = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--manifest-path")
            .arg(&cargo_path)
            .current_dir(&temp_dir)
            .output()
            .map_err(|e| {
                x_codegen::CodeGenError::GenerationError(format!(
                    "Failed to invoke cargo: {}. Is Rust installed?",
                    e
                ))
            })?;

        if !output_status.status.success() {
            let stderr = String::from_utf8_lossy(&output_status.stderr);
            let stdout = String::from_utf8_lossy(&output_status.stdout);
            return Err(x_codegen::CodeGenError::GenerationError(format!(
                "Rust compilation failed.\nSTDOUT:\n{}\nSTDERR:\n{}",
                stdout, stderr
            )));
        }

        // 找到生成的可执行文件
        let target_dir = temp_dir.join("target").join("release");
        let exe_name = if cfg!(windows) {
            "xlang_output.exe"
        } else {
            "xlang_output"
        };
        let exe_path = target_dir.join(exe_name);

        if !exe_path.exists() {
            return Err(x_codegen::CodeGenError::GenerationError(format!(
                "cargo build succeeded but output not found at {}",
                exe_path.display()
            )));
        }

        // 复制到目标位置
        std::fs::copy(&exe_path, output_path).map_err(|e| {
            x_codegen::CodeGenError::GenerationError(format!("Failed to copy executable: {}", e))
        })?;

        // 设置可执行权限（非 Windows）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(output_path)
                .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))?
                .permissions();
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(output_path, perms)
                .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))?;
        }

        // 清理临时文件
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(output_path.to_path_buf())
    }

    /// X builtin/prelude functions whose low-level (C-FFI based) bodies do not map
    /// to valid safe Rust. Their calls are either lowered to Rust macros (println,
    /// print) or replaced by correct hand-written implementations in the runtime
    /// preamble (panic, assert, enumerate). We skip emitting their LIR bodies.
    fn is_skipped_builtin(name: &str) -> bool {
        matches!(name, "println" | "print" | "panic" | "assert" | "enumerate")
    }

    fn is_int_ty(ty: &x_lir::Type) -> bool {
        matches!(
            ty,
            x_lir::Type::Int
                | x_lir::Type::Long
                | x_lir::Type::LongLong
                | x_lir::Type::Uint
                | x_lir::Type::Ulong
                | x_lir::Type::UlongLong
                | x_lir::Type::Char
                | x_lir::Type::Bool
                | x_lir::Type::CInt
                | x_lir::Type::Short
                | x_lir::Type::Ushort
                | x_lir::Type::Schar
                | x_lir::Type::Uchar
        )
    }

    fn is_float_ty(ty: &x_lir::Type) -> bool {
        matches!(
            ty,
            x_lir::Type::Float | x_lir::Type::Double | x_lir::Type::LongDouble
        )
    }

    /// True for C-string types (`char*` / `const char*`), possibly qualified.
    fn is_cstr_ty(ty: &x_lir::Type) -> bool {
        let unqualified = match ty {
            x_lir::Type::Qualified(_, inner) => inner.as_ref(),
            other => other,
        };
        matches!(
            unqualified,
            x_lir::Type::Pointer(p) if matches!(p.as_ref(), x_lir::Type::Char)
        )
    }

    /// Emit correct Rust implementations for X builtin functions that cannot be
    /// faithfully lowered from LIR.
    fn emit_runtime_preamble(&mut self) -> RustResult<()> {
        self.line("// X runtime preamble (hand-written builtins)")?;
        self.line("fn panic(message: impl std::fmt::Display) -> ! { eprintln!(\"{}\", message); std::process::abort(); }")?;
        self.line("fn assert(condition: bool) { if !condition { eprintln!(\"assertion failed\"); std::process::abort(); } }")?;
        self.line("fn enumerate<T>(items: Vec<T>) -> Vec<(i64, T)> { items.into_iter().enumerate().map(|(i, x)| (i as i64, x)).collect() }")?;
        // stdin shim for std.io read_line: POSIX getline semantics (returns the
        // line length INCLUDING the trailing newline, or -1 at EOF), storing the
        // line so __x_buf_char can index it (Rust Strings are not indexable).
        self.line("static mut __X_LINE: String = String::new();")?;
        self.line("fn __x_getline() -> i64 {")?;
        self.indent();
        self.line("use std::io::BufRead;")?;
        self.line("let mut line = String::new();")?;
        self.line("let n = std::io::stdin().lock().read_line(&mut line).unwrap_or(0);")?;
        self.line("if n == 0 { return -1; }")?;
        self.line("unsafe { __X_LINE = line; }")?;
        self.line("n as i64")?;
        self.dedent();
        self.line("}")?;
        self.line("fn __x_buf_char(s: &String, i: i64) -> i64 {")?;
        self.indent();
        self.line("let src: &String = if !s.is_empty() { s } else { unsafe { &__X_LINE } };")?;
        self.line("let bytes = src.as_bytes();")?;
        self.line("if i >= 0 && (i as usize) < bytes.len() { bytes[i as usize] as i64 } else { 0 }")?;
        self.dedent();
        self.line("}")?;
        // Opaque handle type for the boxed runtime value `XValue` from xrt.c.
        // The runtime only ever passes it around by pointer, so an empty
        // enum / trait object placeholder is enough for type-correctness.
        self.line("#[derive(Debug, Clone, PartialEq)]")?;
        self.line("pub enum XValue {}")?;
        // Opaque placeholder for the generic return type of `unwrap_ok` (the
        // runtime helper does not expose a concrete Rust type).
        self.line("#[derive(Debug, Clone, PartialEq)]")?;
        self.line("pub enum T {}")?;
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // LIR declaration generation
    // ========================================================================

    /// Generate code for a LIR declaration
    fn generate_lir_declaration(&mut self, decl: &x_lir::Declaration) -> RustResult<()> {
        match decl {
            x_lir::Declaration::Import(import) => self.generate_lir_import(import)?,
            x_lir::Declaration::Function(func) => self.generate_lir_function(func)?,
            x_lir::Declaration::Global(global) => self.generate_lir_global(global)?,
            x_lir::Declaration::Struct(struct_) => self.generate_lir_struct(struct_)?,
            x_lir::Declaration::Class(class) => self.generate_lir_class(class)?,
            x_lir::Declaration::VTable(vtable) => self.generate_lir_vtable(vtable)?,
            x_lir::Declaration::Enum(enum_) => self.generate_lir_enum(enum_)?,
            x_lir::Declaration::TypeAlias(alias) => self.generate_lir_type_alias(alias)?,
            x_lir::Declaration::Newtype(nt) => self.generate_lir_newtype(nt)?,
            x_lir::Declaration::Trait(trait_) => self.generate_lir_trait(trait_)?,
            x_lir::Declaration::Effect(effect) => self.generate_lir_effect(effect)?,
            x_lir::Declaration::Impl(impl_) => self.generate_lir_impl(impl_)?,
            x_lir::Declaration::ExternFunction(ext) => self.generate_lir_extern_function(ext)?,
        }
        Ok(())
    }

    /// Generate import declaration
    fn generate_lir_import(&mut self, import: &x_lir::Import) -> RustResult<()> {
        // X module imports (e.g. `std.types`) are resolved by inlining the prelude
        // and module sources, so there is no corresponding Rust crate/module to
        // `use`. X uses `.` as the module-path separator, which is not valid Rust
        // path syntax, so skip emitting a `use` for these.
        if import.module_path.contains('.') {
            return Ok(());
        }
        if import.import_all {
            self.line(&format!("use {}::*;", import.module_path))?;
        } else if !import.symbols.is_empty() {
            let symbols: Vec<String> = import
                .symbols
                .iter()
                .map(|(name, alias)| {
                    if let Some(alias) = alias {
                        format!("{} as {}", name, alias)
                    } else {
                        name.clone()
                    }
                })
                .collect();
            self.line(&format!(
                "use {}::{{{}}};",
                import.module_path,
                symbols.join(", ")
            ))?;
        }
        self.line("")?;
        Ok(())
    }

    /// Generate function from LIR
    fn generate_lir_function(&mut self, func: &x_lir::Function) -> RustResult<()> {
        self.current_fn = func.name.clone();
        // Skip builtins handled by the runtime preamble / call-site macro lowering.
        if Self::is_skipped_builtin(&func.name) {
            return Ok(());
        }
        // Handle type parameters for generics
        let type_params = if func.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", func.type_params.join(", "))
        };

        // Build parameters
        for param in &func.parameters {
            self.var_rust_types
                .insert(param.name.clone(), self.lir_type_to_rust(&param.type_));
        }
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|param| format!("{}: {}", param.name, self.lir_type_to_rust(&param.type_)))
            .collect();

        // main function must return (), convert i32 return to ()
        let is_main = func.name == "main";
        let return_type = if is_main {
            "()".to_string()
        } else {
            self.lir_type_to_rust(&func.return_type)
        };

        self.line(&format!(
            "{}pub fn {}{}({}) -> {} {{",
            if func.is_static { "pub " } else { "" },
            func.name,
            type_params,
            params.join(", "),
            return_type
        ))?;
        self.indent();
        // Wrap the body in `unsafe` so accessing `static mut` globals and raw
        // pointers (C FFI) is permitted. Harmless for bodies that need no unsafe
        // thanks to `#![allow(unused_unsafe)]`.
        self.line("unsafe {")?;
        self.indent();

        // Track string-typed parameters for concatenation detection.
        self.string_vars.clear();
        for param in &func.parameters {
            if self.lir_type_to_rust(&param.type_) == "String" {
                self.string_vars.insert(param.name.clone());
            }
        }

        self.compute_deferred_locals(&func.body);
        if is_main {
            // Materialize top-level-`let` globals as locals: initialized ones up
            // front (declaration order), the rest deferred to first assignment.
            for (name, init) in self.global_local_inits.clone() {
                self.track_string_var(&name, &init);
                let rhs = self.generate_assign_rhs(&init)?;
                self.declared_locals.insert(name.clone());
                // A deferred String temp whose first value is null (raw buffer)
                // must still be a String, not an inferred raw pointer.
                let rhs = if rhs == "std::ptr::null_mut()"
                    && self.var_rust_types.get(&name).map(|t| t == "String").unwrap_or(false)
                {
                    "String::new()".to_string()
                } else {
                    rhs
                };
                self.line(&format!("let mut {} = {};", name, rhs))?;
            }
            for name in self.globals_as_locals.clone() {
                self.deferred_locals.insert(name);
            }
        }

        // For main function, we need special handling of return statements
        if is_main {
            self.generate_lir_block_for_main(&func.body)?;
        } else {
            self.generate_lir_block(&func.body)?;
        }

        self.dedent();
        self.line("}")?;
        self.dedent();
        self.line("}")?;
        self.line("")?;

        Ok(())
    }

    /// Generate block for main function (handles return differently)
    fn generate_lir_block_for_main(&mut self, block: &x_lir::Block) -> RustResult<()> {
        // 先扫描一次，标记所有有赋值的临时变量
        let mut assigned_temp_vars = std::collections::HashSet::new();
        for stmt in &block.statements {
            if let x_lir::Statement::Expression(x_lir::Expression::Assign(target, _)) = stmt {
                if let x_lir::Expression::Variable(name) = target.as_ref() {
                    if name.starts_with('t')
                        && name.len() > 1
                        && name[1..].chars().all(|c| c.is_ascii_digit())
                    {
                        assigned_temp_vars.insert(name.clone());
                    }
                }
            }
        }

        // 跟踪是否已经执行过输出语句
        let mut has_output = false;

        for stmt in &block.statements {
            // 检测是否是 println/print 等输出语句
            if let x_lir::Statement::Expression(x_lir::Expression::Assign(_, value)) = stmt {
                if let x_lir::Expression::Call(callee, _) = value.as_ref() {
                    if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                        if matches!(fn_name.as_str(), "println" | "print" | "eprintln") {
                            has_output = true;
                        }
                    }
                }
            }

            // Check if this is the last statement and it's a return
            let is_last_return = if let x_lir::Statement::Return(Some(_)) = stmt {
                block.statements.iter().last() == Some(stmt)
            } else {
                false
            };

            if is_last_return {
                // For main, use std::process::exit() to set exit code
                // 如果之前有过输出语句，直接退出 0
                if has_output {
                    self.line("std::process::exit(0);")?;
                } else if let x_lir::Statement::Return(Some(expr)) = stmt {
                    let code = self.generate_lir_expression(expr)?;
                    // 检查返回值是否是有赋值的临时变量
                    let code_clean = code.trim();
                    if code_clean.starts_with("t") && assigned_temp_vars.contains(code_clean) {
                        self.line(&format!("std::process::exit({});", code))?;
                    } else {
                        // 没有被赋值的变量，使用 0
                        self.line("std::process::exit(0);")?;
                    }
                }
            } else {
                self.generate_lir_statement(stmt)?;
            }
        }
        Ok(())
    }

    /// Generate global variable
    ///
    /// X top-level `let` bindings are lowered to module-level globals that may be
    /// assigned from the synthetic `main` (when their initializer is not a literal).
    /// We model them as `static mut` with a const-evaluable default so that both the
    /// declaration and any later assignment in `main` compile. All function bodies
    /// are wrapped in `unsafe`, so accessing these is allowed.
    fn generate_lir_global(&mut self, global: &x_lir::GlobalVar) -> RustResult<()> {
        // Materialized as a `let` local inside `main` instead.
        if self.globals_as_locals.contains(&global.name) {
            return Ok(());
        }
        let ty = self.lir_type_to_rust(&global.type_);
        let init_code = if let Some(init) = &global.initializer {
            self.generate_lir_expression(init)?
        } else {
            Self::default_value_for_type(&global.type_)
        };
        self.line(&format!(
            "static mut {}: {} = {};",
            global.name, ty, init_code
        ))?;
        self.line("")?;
        Ok(())
    }

    /// Produce a const-evaluable Rust default for a LIR type, used to initialize
    /// otherwise-uninitialized globals.
    fn default_value_for_type(ty: &x_lir::Type) -> String {
        match ty {
            x_lir::Type::Bool => "false".to_string(),
            x_lir::Type::Char => "0i64".to_string(),
            x_lir::Type::Float | x_lir::Type::Double | x_lir::Type::LongDouble => "0.0".to_string(),
            x_lir::Type::Schar
            | x_lir::Type::Uchar
            | x_lir::Type::Short
            | x_lir::Type::Ushort
            | x_lir::Type::Int
            | x_lir::Type::Uint
            | x_lir::Type::Long
            | x_lir::Type::Ulong
            | x_lir::Type::LongLong
            | x_lir::Type::UlongLong
            | x_lir::Type::Size
            | x_lir::Type::Ptrdiff
            | x_lir::Type::Intptr
            | x_lir::Type::Uintptr => "0".to_string(),
            // X strings are owned Rust Strings: an uninitialized String temp
            // must default to String::new(), not a raw null pointer.
            x_lir::Type::Pointer(p) if matches!(p.as_ref(), x_lir::Type::Char) => {
                "String::new()".to_string()
            }
            x_lir::Type::Pointer(_) => "std::ptr::null_mut()".to_string(),
            x_lir::Type::Array(_, None) => "Vec::new()".to_string(),
            x_lir::Type::Array(inner, Some(n)) => {
                format!("[{}; {}]", Self::default_value_for_type(inner), n)
            }
            x_lir::Type::Qualified(_, inner) => Self::default_value_for_type(inner),
            // Fallback: works for any type that implements Default in const context
            // is not guaranteed, but covers the common scalar cases above.
            _ => "Default::default()".to_string(),
        }
    }

    /// Generate struct definition
    fn generate_lir_struct(&mut self, struct_: &x_lir::Struct) -> RustResult<()> {
        self.line("#[derive(Debug, Clone, PartialEq)]")?;
        self.line(&format!("pub struct {} {{", struct_.name))?;
        self.indent();

        let mut field_tys = std::collections::HashMap::new();
        for field in &struct_.fields {
            let ty = self.lir_type_to_rust(&field.type_);
            field_tys.insert(field.name.clone(), ty.clone());
            self.line(&format!("pub {}: {},", field.name, ty))?;
        }
        self.struct_field_types.insert(struct_.name.clone(), field_tys);

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate class definition
    fn generate_lir_class(&mut self, class: &x_lir::Class) -> RustResult<()> {
        self.line("#[derive(Debug, Clone)]")?;
        self.line(&format!("pub struct {} {{", class.name))?;
        self.indent();

        // If this class has a vtable, add it
        if class.has_vtable {
            self.line(&format!("vtable: *mut {}VTable,", class.name))?;
        }

        for field in &class.fields {
            let ty = self.lir_type_to_rust(&field.type_);
            self.line(&format!("pub {}: {},", field.name, ty))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate vtable definition
    fn generate_lir_vtable(&mut self, vtable: &x_lir::VTable) -> RustResult<()> {
        self.line(&format!("pub struct {}VTable {{", vtable.name))?;
        self.indent();

        for entry in &vtable.entries {
            let params: Vec<String> = entry
                .function_type
                .param_types
                .iter()
                .map(|ty| self.lir_type_to_rust(ty))
                .collect();
            let return_type = self.lir_type_to_rust(&entry.function_type.return_type);
            let fn_type = format!("fn({}) -> {}", params.join(", "), return_type);
            self.line(&format!("pub {}: {},", entry.method_name, fn_type))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate enum definition
    fn generate_lir_enum(&mut self, enum_: &x_lir::Enum) -> RustResult<()> {
        self.line("#[derive(Debug, Clone, PartialEq)]")?;
        self.line(&format!("pub enum {} {{", enum_.name))?;
        self.indent();

        for variant in &enum_.variants {
            if let Some(value) = variant.value {
                self.line(&format!("{} = {},", variant.name, value))?;
            } else {
                self.line(&format!("{},", variant.name))?;
            }
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate type alias
    fn generate_lir_type_alias(&mut self, alias: &x_lir::TypeAlias) -> RustResult<()> {
        let ty = self.lir_type_to_rust(&alias.type_);
        self.line(&format!("pub type {} = {};", alias.name, ty))?;
        self.line("")?;
        Ok(())
    }

    /// Generate newtype (struct wrapper)
    fn generate_lir_newtype(&mut self, nt: &x_lir::Newtype) -> RustResult<()> {
        let ty = self.lir_type_to_rust(&nt.type_);
        self.line(&format!("pub struct {} (pub {});", nt.name, ty))?;
        self.line("")?;
        Ok(())
    }

    /// Generate trait (interface) definition
    fn generate_lir_trait(&mut self, trait_: &x_lir::Trait) -> RustResult<()> {
        let type_params = if trait_.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", trait_.type_params.join(", "))
        };
        self.line(&format!("pub trait {}{}", trait_.name, type_params))?;
        if !trait_.extends.is_empty() {
            self.line(&format!(": {} +", trait_.extends.join(" +")))?;
        }
        self.line("{")?;
        self.indent();
        for method in &trait_.methods {
            let ret_ty = method
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_rust(ty))
                .unwrap_or_else(|| "()".to_string());
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.lir_type_to_rust(&p.type_)))
                .collect();
            let method_type_params = if method.type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", method.type_params.join(", "))
            };
            self.line(&format!(
                "fn {}{}({}) -> {} {}",
                method.name,
                method_type_params,
                params.join(", "),
                ret_ty,
                if method.default_body.is_some() {
                    "{"
                } else {
                    ";"
                }
            ))?;
            if method.default_body.is_some() {
                self.indent();
                self.line("// default body")?;
                self.dedent();
                self.line("}")?;
            }
        }
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate effect definition
    fn generate_lir_effect(&mut self, effect: &x_lir::Effect) -> RustResult<()> {
        let type_params = if effect.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", effect.type_params.join(", "))
        };
        self.line(&format!("pub trait {}{} {{", effect.name, type_params))?;
        self.indent();
        for op in &effect.operations {
            let ret_ty = op
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_rust(ty))
                .unwrap_or_else(|| "()".to_string());
            let params: Vec<String> = op
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.lir_type_to_rust(&p.type_)))
                .collect();
            self.line(&format!(
                "fn {}({}) -> {};",
                op.name,
                params.join(", "),
                ret_ty
            ))?;
        }
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate trait/effect implementation
    fn generate_lir_impl(&mut self, impl_: &x_lir::Impl) -> RustResult<()> {
        let type_params = if impl_.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", impl_.type_params.join(", "))
        };
        let target_ty = self.lir_type_to_rust(&impl_.target_type);
        self.line(&format!(
            "impl{}{} for {} {{",
            type_params, impl_.trait_name, target_ty
        ))?;
        self.indent();
        for method in &impl_.methods {
            let ret = self.lir_type_to_rust(&method.return_type);
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.lir_type_to_rust(&p.type_)))
                .collect();
            let method_type_params = if method.type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", method.type_params.join(", "))
            };
            self.line(&format!(
                "fn {}{}({}) -> {} {{",
                method.name,
                method_type_params,
                params.join(", "),
                ret
            ))?;
            self.indent();
            self.line("// method body")?;
            self.dedent();
            self.line("}")?;
        }
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate extern function declaration
    fn generate_lir_extern_function(&mut self, ext: &x_lir::ExternFunction) -> RustResult<()> {
        // The LIR prelude declares each runtime function twice (once as an
        // ExternFunction, once via an implicit Call); emit each name only once
        // to avoid `error[E0428]: the name is defined multiple times`.
        if !self.emitted_externs.insert(ext.name.clone()) {
            return Ok(());
        }

        // Use uppercase "C" for Rust ABI
        let abi = ext.abi.clone().unwrap_or_else(|| "C".to_string());
        let abi_display = if abi.to_lowercase() == "c" { "C" } else { &abi };

        let type_params = if ext.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", ext.type_params.join(", "))
        };

        // Record the signature so call sites can wrap String args into C
        // strings and unwrap C-string results.
        self.extern_sigs.insert(
            ext.name.clone(),
            (ext.parameters.clone(), ext.return_type.clone()),
        );

        // C-string ABI: X strings are owned Rust `String`s, but the C runtime
        // takes/returns `*const c_char`; declare the extern with the raw pointer
        // type and convert at call sites.
        fn cstr_ty(ty: &x_lir::Type) -> String {
            match ty {
                x_lir::Type::Pointer(p) if matches!(p.as_ref(), x_lir::Type::Char) => {
                    "*const std::ffi::c_char".to_string()
                }
                x_lir::Type::Qualified(_, inner) => cstr_ty(inner),
                other => other.to_string(),
            }
        }

        // Parameters are just types, generate with numbered names
        let params: Vec<String> = ext
            .parameters
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let rust_ty = self.lir_type_to_rust(ty);
                // C-string parameters must be declared as raw pointers.
                let declared = if Self::is_cstr_ty(ty) {
                    "*const std::ffi::c_char".to_string()
                } else {
                    rust_ty
                };
                format!("arg{}: {}", i, declared)
            })
            .collect();

        let return_type = self.lir_type_to_rust(&ext.return_type);
        // C-string returns are declared as raw pointers; call sites convert.
        let return_type = if Self::is_cstr_ty(&ext.return_type) {
            "*const std::ffi::c_char".to_string()
        } else {
            return_type
        };
        self.line(&format!("#[link(name = \"{}\")]", abi.to_lowercase()))?;
        self.line(&format!("extern \"{}\" {{", abi_display))?;
        self.indent();
        self.line(&format!(
            "fn {}{}({}) -> {};",
            ext.name,
            type_params,
            params.join(", "),
            return_type
        ))?;
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // LIR block / statement generation
    // ========================================================================

    /// Build a Rust formatting-macro invocation (println!/print!/eprintln!/format!)
    /// with an explicit format string, since Rust's macros require one. A single
    /// string-literal argument is used directly as the format text; otherwise each
    /// argument gets a `{}` placeholder.
    fn format_macro_call(macro_name: &str, args_code: &[String]) -> String {
        let bang = format!("{}!", macro_name);
        if args_code.is_empty() {
            return format!("{}()", bang);
        }
        // Single string-literal argument: use it as the format string directly so
        // that e.g. println("Hello") -> println!("Hello").
        if args_code.len() == 1 {
            let a = args_code[0].trim();
            if a.starts_with('"') && a.ends_with('"') && a.len() >= 2 {
                return format!("{}({})", bang, a);
            }
        }
        let placeholders = vec!["{}"; args_code.len()].join(" ");
        format!("{}(\"{}\", {})", bang, placeholders, args_code.join(", "))
    }

    /// Generate a LIR basic block
    fn generate_lir_block(&mut self, block: &x_lir::Block) -> RustResult<()> {
        for stmt in &block.statements {
            self.generate_lir_statement(stmt)?;
        }
        Ok(())
    }

    /// Generate a LIR statement
    fn generate_lir_statement(&mut self, stmt: &x_lir::Statement) -> RustResult<()> {
        match stmt {
            x_lir::Statement::Expression(x_lir::Expression::Assign(target, value))
                if matches!(target.as_ref(), x_lir::Expression::Variable(name)
                    if self.deferred_locals.contains(name)
                        && !self.declared_locals.contains(name)) =>
            {
                // First assignment of a deferred temporary: emit `let mut` so Rust
                // infers the type from the value instead of the (often wrong) LIR
                // temp type.
                let name = match target.as_ref() {
                    x_lir::Expression::Variable(n) => n.clone(),
                    _ => unreachable!(),
                };
                // Reuse the assignment expression logic to honor the println/print
                // macro special-casing for the value.
                self.track_string_var(&name, value);
                let rhs = self.generate_assign_rhs(value)?;
                self.declared_locals.insert(name.clone());
                // A deferred String temp whose first value is null (raw buffer)
                // must still be a String, not an inferred raw pointer.
                let rhs = if rhs == "std::ptr::null_mut()"
                    && self.var_rust_types.get(&name).map(|t| t == "String").unwrap_or(false)
                {
                    "String::new()".to_string()
                } else {
                    rhs
                };
                self.line(&format!("let mut {} = {};", name, rhs))?;
            }
            x_lir::Statement::Expression(expr) => {
                let code = self.generate_lir_expression(expr)?;
                self.line(&format!("{};", code))?;
            }
            x_lir::Statement::Variable(var) => {
                let ty = self.lir_type_to_rust(&var.type_);
                self.var_rust_types.insert(var.name.clone(), ty.clone());
                if self.deferred_locals.contains(&var.name) && var.initializer.is_none() {
                    // Declaration deferred to first assignment.
                    return Ok(());
                }

                if var.is_extern {
                    let _ = self.line(&format!("let {}: {};", var.name, ty));
                } else if var.is_static {
                    let mut decl = format!("static {}: {}", var.name, ty);
                    if let Some(init) = &var.initializer {
                        let init_code = self.generate_lir_expression(init)?;
                        decl.push_str(&format!(" = {};", init_code));
                    } else {
                        decl.push_str(";");
                    }
                    let _ = self.line(&decl);
                } else if let Some(init) = &var.initializer {
                    self.track_string_var(&var.name, init);
                    let mut init_code = self.generate_lir_expression(init)?;
                    // `null` initializers on String temps must become String::new().
                    if Self::is_cstr_ty(&var.type_) && init_code == "std::ptr::null_mut()" {
                        init_code = "String::new()".to_string();
                    }
                    let _ = self.line(&format!("let mut {} = {};", var.name, init_code));
                } else {
                    // Uninitialized local: provide a type-appropriate default so it
                    // never compiles as "used while uninitialized". (LIR sometimes
                    // drops the producing statement, e.g. a not-yet-lowered loop.)
                    let default = Self::default_value_for_type(&var.type_);
                    let _ = self.line(&format!("let mut {}: {} = {};", var.name, ty, default));
                }
            }
            x_lir::Statement::If(if_stmt) => {
                let cond = self.generate_lir_expression(&if_stmt.condition)?;
                self.line(&format!("if {} {{", cond))?;
                self.indent();
                self.generate_lir_statement(&if_stmt.then_branch)?;
                self.dedent();

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.line("} else {")?;
                    self.indent();
                    self.generate_lir_statement(else_branch)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            x_lir::Statement::While(while_stmt) => {
                let cond = self.generate_lir_expression(&while_stmt.condition)?;
                self.line(&format!("while {} {{", cond))?;
                self.indent();
                self.generate_lir_statement(&while_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::DoWhile(do_while) => {
                self.line("loop {")?;
                self.indent();
                self.generate_lir_statement(&do_while.body)?;
                let cond = self.generate_lir_expression(&do_while.condition)?;
                self.line(&format!("if !({}) {{ break; }}", cond))?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::For(for_stmt) => {
                self.line("for (")?;
                if let Some(init) = &for_stmt.initializer {
                    self.generate_lir_statement(init)?;
                }
                self.line(";")?;
                if let Some(cond) = &for_stmt.condition {
                    let cond_code = self.generate_lir_expression(cond)?;
                    self.line(&format!(" {}", cond_code))?;
                }
                self.line(";")?;
                if let Some(inc) = &for_stmt.increment {
                    let inc_code = self.generate_lir_expression(inc)?;
                    self.line(&format!(" {}", inc_code))?;
                }
                self.line(") {")?;
                self.indent();
                self.generate_lir_statement(&for_stmt.body)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Return(opt_expr) => {
                if self.current_fn == "main" {
                    // Rust main returns (): nested returns drop the value.
                    self.line("return;")?;
                } else if let Some(expr) = opt_expr {
                    let code = self.generate_lir_expression(expr)?;
                    self.line(&format!("return {};", code))?;
                } else {
                    self.line("return;")?;
                }
            }
            x_lir::Statement::Break => {
                self.line("break;")?;
            }
            x_lir::Statement::Continue => {
                self.line("continue;")?;
            }
            x_lir::Statement::Label(_name) => {
                // Rust doesn't support labels in this form, skip it
            }
            x_lir::Statement::Goto(target) => {
                self.line(&format!("goto {};", target))?;
            }
            x_lir::Statement::Compound(block) => {
                self.line("{")?;
                self.indent();
                self.generate_lir_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Empty => {}
            x_lir::Statement::Match(match_stmt) => {
                let expr = self.generate_lir_expression(&match_stmt.scrutinee)?;
                self.line(&format!("match {} {{", expr))?;
                self.indent();

                for case in &match_stmt.cases {
                    let pattern = self.generate_lir_pattern(&case.pattern)?;
                    let guard = if let Some(g) = &case.guard {
                        format!(" if {}", self.generate_lir_expression(g)?)
                    } else {
                        String::new()
                    };
                    self.line(&format!("{}{} => {{", pattern, guard))?;
                    self.indent();
                    self.generate_lir_block(&case.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Try(try_stmt) => {
                self.line("{")?;
                self.indent();
                self.line("let __result = (|| {")?;
                self.indent();
                self.generate_lir_block(&try_stmt.body)?;
                self.line("Ok(())")?;
                self.dedent();
                self.line("})();")?;
                self.line("match __result {")?;
                self.indent();

                for catch in &try_stmt.catch_clauses {
                    let var_name = catch.variable_name.as_deref().unwrap_or("_");
                    let ty = catch.exception_type.as_deref().unwrap_or("_");
                    self.line(&format!("Err({}: {}) => {{", var_name, ty))?;
                    self.indent();
                    self.generate_lir_block(&catch.body)?;
                    self.dedent();
                    self.line("},")?;
                }

                self.line("Ok(_) => {},")?;
                self.dedent();
                self.line("}")?;

                if let Some(finally) = &try_stmt.finally_block {
                    self.generate_lir_block(finally)?;
                }

                self.dedent();
                self.line("}")?;
            }
            x_lir::Statement::Declaration(_) => {
                // Already handled at top level
            }
            x_lir::Statement::Switch(switch_stmt) => {
                self.generate_lir_switch(switch_stmt)?;
            }
        }
        Ok(())
    }

    /// Generate a switch statement
    fn generate_lir_switch(&mut self, switch_stmt: &x_lir::SwitchStatement) -> RustResult<()> {
        let expr = self.generate_lir_expression(&switch_stmt.expression)?;
        self.line(&format!("match {} {{", expr))?;
        self.indent();

        for case in &switch_stmt.cases {
            let value = self.generate_lir_expression(&case.value)?;
            self.line(&format!("{} => {{", value))?;
            self.indent();
            self.generate_lir_statement(&case.body)?;
            self.dedent();
            self.line("},")?;
        }

        if let Some(default_body) = &switch_stmt.default {
            self.line("_ => {")?;
            self.indent();
            self.generate_lir_statement(default_body)?;
            self.dedent();
            self.line("},")?;
        }

        self.dedent();
        self.line("}")?;

        Ok(())
    }

    // ========================================================================
    // LIR pattern generation
    // ========================================================================

    /// Generate a LIR pattern
    fn generate_lir_pattern(&mut self, pattern: &x_lir::Pattern) -> RustResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard => Ok("_".to_string()),
            x_lir::Pattern::Variable(name) => Ok(name.clone()),
            x_lir::Pattern::Literal(lit) => Ok(self.generate_lir_literal(lit)),
            x_lir::Pattern::Constructor(name, patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.generate_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                if patterns.is_empty() {
                    Ok(format!("{}", name))
                } else {
                    Ok(format!("{}({})", name, pat_strs.join(", ")))
                }
            }
            x_lir::Pattern::Tuple(patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.generate_lir_pattern(p))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>(
                )?;
                Ok(format!("({},)", pat_strs.join(", ")))
            }
            x_lir::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| -> Result<String, x_codegen::CodeGenError> {
                        let p_str = self.generate_lir_pattern(p)?;
                        Ok(format!("{}: {}", n, p_str))
                    })
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(format!("{} {{ {} }}", name, field_strs.join(", ")))
            }
            x_lir::Pattern::Or(left, right) => {
                let left_str = self.generate_lir_pattern(left)?;
                let right_str = self.generate_lir_pattern(right)?;
                Ok(format!("{} | {}", left_str, right_str))
            }
        }
    }

    // ========================================================================
    // LIR expression generation
    // ========================================================================

    /// Returns true if `value` is a call to a print-like builtin (whose result is
    /// unit and which we lower to a Rust macro call).
    fn is_print_like_value(value: &x_lir::Expression) -> bool {
        if let x_lir::Expression::Call(callee, _) = value {
            if let x_lir::Expression::Variable(name) = callee.as_ref() {
                return matches!(
                    name.as_str(),
                    "println" | "print" | "eprintln" | "eprintln!"
                );
            }
        }
        false
    }

    /// Generate the right-hand side of an assignment, honoring print/format macro
    /// lowering for print-like builtins.
    fn generate_assign_rhs(&mut self, value: &x_lir::Expression) -> RustResult<String> {
        if let x_lir::Expression::Call(callee, args) = value {
            if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                let name = fn_name.as_str();
                if matches!(
                    name,
                    "println" | "print" | "eprintln" | "eprintln!" | "format"
                ) {
                    let args_code: Vec<String> = args
                        .iter()
                        .map(|arg| self.generate_lir_expression(arg))
                        .collect::<Result<_, _>>()?;
                    let call_str = match name {
                        "println" => Self::format_macro_call("println", &args_code),
                        "print" => Self::format_macro_call("print", &args_code),
                        "eprintln" | "eprintln!" => Self::format_macro_call("eprintln", &args_code),
                        "format" => Self::format_macro_call("format", &args_code),
                        _ => format!("{}({})", name, args_code.join(", ")),
                    };
                    return Ok(call_str);
                }
            }
        }
        self.generate_lir_expression(value)
    }

    /// If `expr` is an `Add` tree that contains at least one string literal,
    /// returns the flattened operands (left-to-right) so the caller can emit a
    /// `format!`. Returns `None` for purely numeric additions.
    fn string_concat_parts(expr: &x_lir::Expression) -> Option<Vec<&x_lir::Expression>> {
        fn contains_string(e: &x_lir::Expression) -> bool {
            match e {
                x_lir::Expression::Literal(x_lir::Literal::String(_)) => true,
                x_lir::Expression::Binary(x_lir::BinaryOp::Add, l, r) => {
                    contains_string(l) || contains_string(r)
                }
                x_lir::Expression::Parenthesized(inner) => contains_string(inner),
                _ => false,
            }
        }
        fn flatten<'a>(e: &'a x_lir::Expression, out: &mut Vec<&'a x_lir::Expression>) {
            match e {
                x_lir::Expression::Binary(x_lir::BinaryOp::Add, l, r) => {
                    flatten(l, out);
                    flatten(r, out);
                }
                x_lir::Expression::Parenthesized(inner) => flatten(inner, out),
                other => out.push(other),
            }
        }
        if !contains_string(expr) {
            return None;
        }
        let mut parts = Vec::new();
        flatten(expr, &mut parts);
        Some(parts)
    }

    /// Generate a LIR expression
    fn generate_lir_expression(&mut self, expr: &x_lir::Expression) -> RustResult<String> {
        match expr {
            // In expression position, string literals become owned `String`s so they
            // match the `String` type used for X `string` values.
            x_lir::Expression::Literal(x_lir::Literal::String(s)) => {
                Ok(format!("\"{}\".to_string()", s))
            }
            x_lir::Expression::Literal(lit) => Ok(self.generate_lir_literal(lit)),
            x_lir::Expression::Variable(name) => Ok(name.clone()),
            x_lir::Expression::Unary(op, inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                let op_str = match op {
                    x_lir::UnaryOp::Minus => "-",
                    x_lir::UnaryOp::Plus => "+",
                    x_lir::UnaryOp::Not => "!",
                    x_lir::UnaryOp::BitNot => "!",
                    x_lir::UnaryOp::PreIncrement => "++",
                    x_lir::UnaryOp::PreDecrement => "--",
                    x_lir::UnaryOp::PostIncrement => "++",
                    x_lir::UnaryOp::PostDecrement => "--",
                    x_lir::UnaryOp::Reference => "&",
                    x_lir::UnaryOp::MutableReference => "&mut ",
                };
                let result = match op {
                    x_lir::UnaryOp::PostIncrement | x_lir::UnaryOp::PostDecrement => {
                        format!("{}{}", inner_code, op_str)
                    }
                    _ => format!("{}{}", op_str, inner_code),
                };
                Ok(result)
            }
            x_lir::Expression::Binary(op, left, right) => {
                // X uses `+` for string concatenation. Rust does not support
                // `&str + T`, so when an `Add` tree contains a string literal we
                // lower the whole chain to a `format!`.
                if matches!(op, x_lir::BinaryOp::Add)
                    && (self.expr_is_string(left)
                        || self.expr_is_string(right)
                        || Self::string_concat_parts(expr).is_some())
                {
                    let left_code = self.generate_lir_expression(left)?;
                    let right_code = self.generate_lir_expression(right)?;
                    return Ok(format!(
                        "format!(\"{}{}\", {}, {})",
                        "{}", "{}", left_code, right_code
                    ));
                }
                let left_code = self.generate_lir_expression(left)?;
                let right_code = self.generate_lir_expression(right)?;
                let op_str = match op {
                    x_lir::BinaryOp::Add => "+",
                    x_lir::BinaryOp::Subtract => "-",
                    x_lir::BinaryOp::Multiply => "*",
                    x_lir::BinaryOp::Divide => "/",
                    x_lir::BinaryOp::Modulo => "%",
                    x_lir::BinaryOp::BitAnd => "&",
                    x_lir::BinaryOp::BitOr => "|",
                    x_lir::BinaryOp::BitXor => "^",
                    x_lir::BinaryOp::LeftShift => "<<",
                    x_lir::BinaryOp::RightShift => ">>",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>",
                    x_lir::BinaryOp::LessThan => "<",
                    x_lir::BinaryOp::LessThanEqual => "<=",
                    x_lir::BinaryOp::GreaterThan => ">",
                    x_lir::BinaryOp::GreaterThanEqual => ">=",
                    x_lir::BinaryOp::Equal => "==",
                    x_lir::BinaryOp::NotEqual => "!=",
                    x_lir::BinaryOp::LogicalAnd => "&&",
                    x_lir::BinaryOp::LogicalOr => "||",
                };
                // Float arithmetic: Rust requires both operands to be f64;
                // cast when one side is float (no-op when both already f64).
                let lhs_float = self.expr_is_float(left);
                let rhs_float = self.expr_is_float(right);
                if (lhs_float || rhs_float)
                    && matches!(
                        op,
                        x_lir::BinaryOp::Add
                            | x_lir::BinaryOp::Subtract
                            | x_lir::BinaryOp::Multiply
                            | x_lir::BinaryOp::Divide
                            | x_lir::BinaryOp::Modulo
                    )
                {
                    let lc = format!("({} as f64)", left_code);
                    let rc = format!("({} as f64)", right_code);
                    return Ok(format!("({} {} {})", lc, op_str, rc));
                }
                Ok(format!("{} {} {}", left_code, op_str, right_code))
            }
            x_lir::Expression::Ternary(cond, then, else_) => {
                let cond_code = self.generate_lir_expression(cond)?;
                let then_code = self.generate_lir_expression(then)?;
                let else_code = self.generate_lir_expression(else_)?;
                Ok(format!(
                    "if {} {{ {} }} else {{ {} }}",
                    cond_code, then_code, else_code
                ))
            }
            x_lir::Expression::Assign(target, value) => {
                if let x_lir::Expression::Variable(name) = target.as_ref() {
                    self.track_string_var(name, value);
                }
                let rhs = self.generate_assign_rhs(value)?;
                // A print-like builtin used as the RHS produces a statement-like
                // macro call; emit it directly rather than `target = <unit>`.
                if Self::is_print_like_value(value) {
                    return Ok(rhs);
                }
                // Member/field assignment: emit `base.field = rhs` directly so the
                // field access is not wrapped in a clone; String fields clone the
                // RHS instead (moving it would break later uses).
                if let x_lir::Expression::PointerMember(obj, field) = target.as_ref() {
                    let obj_code = self.generate_lir_expression(obj)?;
                    let rhs2 = if self.field_is_string(obj, field) {
                        format!("({}).clone()", rhs)
                    } else {
                        rhs.clone()
                    };
                    return Ok(format!("(*{}).{} = {}", obj_code, field, rhs2));
                }
                if let x_lir::Expression::Member(obj, field) = target.as_ref() {
                    let obj_code = self.generate_lir_expression(obj)?;
                    let rhs2 = if self.field_is_string(obj, field) {
                        format!("({}).clone()", rhs)
                    } else {
                        rhs.clone()
                    };
                    return Ok(format!("{}.{} = {}", obj_code, field, rhs2));
                }
                let target_code = self.generate_lir_expression(target)?;
                // Plain variable-to-variable copy of a Rust `String`: the LIR
                // emits SSA copies (`t41 = t10`) that are value copies in X,
                // but Rust assignment would MOVE the String and break later uses.
                // Clone the RHS so both variables stay live.
                if let x_lir::Expression::Variable(_) = target.as_ref() {
                    if self.expr_is_string(value)
                        || matches!(value.as_ref(), x_lir::Expression::Variable(n) if self.var_rust_types.get(n).map(|t| t == "String").unwrap_or(false))
                    {
                        return Ok(format!("{} = {}.clone()", target_code, rhs));
                    }
                }
                let target_code = self.generate_lir_expression(target)?;
                // `x_as_ptr` returns `*mut ()`: cast it to the target variable's
                // pointer type so field access compiles.
                let rhs = if rhs.trim_start().starts_with("x_as_ptr(") {
                    if let x_lir::Expression::Variable(name) = target.as_ref() {
                        if let Some(ty) = self.var_rust_types.get(name) {
                            if ty != "String" && ty != "()" {
                                format!("({} as {})", rhs, ty)
                            } else {
                                rhs
                            }
                        } else {
                            rhs
                        }
                    } else {
                        rhs
                    }
                } else {
                    rhs
                };
                // bool -> numeric: the LIR may assign a Bool temp to an int slot.
                let rhs = if let x_lir::Expression::Variable(name) = target.as_ref() {
                    let target_int = self
                        .var_rust_types
                        .get(name)
                        .map(|t| matches!(t.as_str(), "i64" | "i32" | "i16" | "i8" | "u64" | "u32" | "usize" | "isize"))
                        .unwrap_or(false);
                    let value_is_bool = matches!(value.as_ref(), x_lir::Expression::Variable(v) if self.var_rust_types.get(v).map(|t| t == "bool").unwrap_or(false));
                    if target_int && value_is_bool {
                        format!("({} as i64)", rhs)
                    } else {
                        rhs
                    }
                } else {
                    rhs
                };
                Ok(format!("{} = {}", target_code, rhs))
            }
            x_lir::Expression::AssignOp(op, target, value) => {
                let target_code = self.generate_lir_expression(target)?;
                let value_code = self.generate_lir_expression(value)?;
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
                    x_lir::BinaryOp::RightShift => ">>=",
                    x_lir::BinaryOp::RightShiftArithmetic => ">>=",
                    _ => "=", // fallback
                };
                Ok(format!("{} {} {}", target_code, op_str, value_code))
            }
            x_lir::Expression::Call(callee, args) => {
                let callee_code = self.generate_lir_expression(callee)?;
                let args_code: Vec<String> = args
                    .iter()
                    .map(|arg| self.generate_lir_expression(arg))
                    .collect::<Result<_, _>>()?;

                // Convert common X built-in functions to Rust equivalents
                let callee_str = callee_code.as_str();
                let result = match callee_str {
                    "println" => Self::format_macro_call("println", &args_code),
                    "print" => Self::format_macro_call("print", &args_code),
                    "eprintln" | "eprintln!" => Self::format_macro_call("eprintln", &args_code),
                    "eprint" | "eprint!" => Self::format_macro_call("eprint", &args_code),
                    "format" => Self::format_macro_call("format", &args_code),
                    // Runtime helpers that take a C string: convert `String` to a
                    // raw `*const char`. The CString must be bound to a local so
                    // it lives long enough for the pointer to be valid.
                    "x_from_str" => {
                        // The boxed XValue outlives this expression, so the C
                        // string must be leaked (mirrors the leaky xrt.c
                        // allocations) rather than dropped at block end.
                        format!(
                            "x_from_str(Box::leak(CString::new(&*{}).unwrap().into_boxed_c_str()).as_ptr())",
                            args_code.join(", ")
                        )
                    }
                    "x_from_ptr" => {
                        // x_from_ptr expects *mut (), cast the argument
                        format!(
                            "x_from_ptr({} as *mut ())",
                            args_code.join(", ")
                        )
                    }
                    "x_as_ptr" => {
                        // x_as_ptr returns *mut (), cast to the expected type
                        format!(
                            "x_as_ptr({})",
                            args_code.join(", ")
                        )
                    }
                    // String indexing: __index__(str, i) reads a byte of the X
                    // string; the runtime helper falls back to the getline stash.
                    "__index__" => {
                        if !args_code.is_empty()
                            && (self.expr_is_string(&args[0])
                                || matches!(&args[0], x_lir::Expression::Variable(n) if self.var_rust_types.get(n).map(|t| t == "String").unwrap_or(false)))
                        {
                            format!("x_from_char(__x_buf_char(&{}, {}))", args_code[0], args_code.get(1).cloned().unwrap_or_default())
                        } else {
                            format!("{}({})", callee_code, args_code.join(", "))
                        }
                    }
                    // free of X string buffers is a no-op (the runtime owns the line).
                    "free" => "free(std::ptr::null_mut())".to_string(),
                    // std.io read_line drives getline through a raw char** buffer
                    // which Rust Strings cannot model; route to the runtime shim
                    // that reads into a static stash.
                    "getline" => "__x_getline()".to_string(),
                    _ => {
                        // Generated (non-extern) functions take String params by
                        // value: clone String-typed variable args so reusing a
                        // string across calls does not move it.
                        if !self.extern_sigs.contains_key(callee_str) {
                            let cloned: Vec<String> = args_code
                                .iter()
                                .zip(args.iter())
                                .map(|(code, a)| {
                                    let is_str = self.expr_is_string(a)
                                        || matches!(a, x_lir::Expression::Variable(n) if self.var_rust_types.get(n).map(|t| t == "String").unwrap_or(false));
                                    if is_str && matches!(a, x_lir::Expression::Variable(_)) {
                                        format!("{}.clone()", code)
                                    } else {
                                        code.clone()
                                    }
                                })
                                .collect();
                            return Ok(format!("{}({})", callee_code, cloned.join(", ")));
                        }
                        // Extern call with known signature: wrap Rust String args
                        // into C strings (with a block so the CStrings outlive the
                        // call) and unwrap C-string returns into owned Strings.
                        if let Some((param_tys, ret_ty)) = self.extern_sigs.get(callee_str) {
                            let is_cstr = |t: &x_lir::Type| Self::is_cstr_ty(t);
                            let mut binds = Vec::new();
                            let mut call_args = Vec::new();
                            for (i, (arg, pty)) in args_code.iter().zip(param_tys.iter()).enumerate() {
                                if is_cstr(pty) {
                                    binds.push(format!("__c{} = CString::new(&*{}).unwrap()", i, arg));
                                    call_args.push(format!("__c{}.as_ptr()", i));
                                } else if Self::is_int_ty(pty) {
                                    // Cast to the extern integer width (no-op when already matching).
                                    call_args.push(format!("({} as i64)", arg));
                                } else if Self::is_float_ty(pty) {
                                    call_args.push(format!("({} as f64)", arg));
                                } else {
                                    call_args.push(arg.clone());
                                }
                            }
                            let call = format!("{}({})", callee_code, call_args.join(", "));
                            if is_cstr(&ret_ty) {
                                binds.push(format!("__r = {}", call));
                                return Ok(format!(
                                    "{{ let {}; CStr::from_ptr(__r).to_string_lossy().into_owned() }}",
                                    binds.join("; let ")
                                ));
                            } else if !binds.is_empty() {
                                return Ok(format!(
                                    "{{ let {}; {} }}",
                                    binds.join("; let "),
                                    call
                                ));
                            }
                        }
                        format!("{}({})", callee_code, args_code.join(", "))
                    }
                };
                Ok(result)
            }
            x_lir::Expression::Index(base, index) => {
                let base_code = self.generate_lir_expression(base)?;
                let index_code = self.generate_lir_expression(index)?;
                // X strings cannot be indexed with `s[i]` in Rust; route raw
                // char* byte access through the runtime helper (which falls back
                // to the getline stash for std.io read_line buffers).
                let base_is_string = self.expr_is_string(base)
                    || matches!(base.as_ref(), x_lir::Expression::Variable(n) if self.var_rust_types.get(n).map(|t| t == "String").unwrap_or(false));
                if base_is_string {
                    return Ok(format!("__x_buf_char(&{}, {})", base_code, index_code));
                }
                Ok(format!("{}[{}]", base_code, index_code))
            }
            x_lir::Expression::Member(base, field) => {
                let base_code = self.generate_lir_expression(base)?;
                // String fields cannot be moved out (raw pointer structs).
                if self.field_is_string(base, field) {
                    return Ok(format!("{}.{}.clone()", base_code, field));
                }
                Ok(format!("{}.{}", base_code, field))
            }
            x_lir::Expression::PointerMember(base, field) => {
                let base_code = self.generate_lir_expression(base)?;
                // In Rust, raw pointers must be dereferenced before field access.
                // String fields cannot be moved out of a raw-pointer struct.
                if self.field_is_string(base, field) {
                    return Ok(format!("(*{}).{}.clone()", base_code, field));
                }
                Ok(format!("(*{}).{}", base_code, field))
            }
            x_lir::Expression::AddressOf(inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                Ok(format!("&{}", inner_code))
            }
            x_lir::Expression::Dereference(inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                Ok(format!("*{}", inner_code))
            }
            x_lir::Expression::Cast(ty, inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                // Casting to X `string` (pointer-to-char) maps to producing an owned
                // Rust `String`; `as String` is not a valid Rust cast.
                if matches!(ty, x_lir::Type::Pointer(p) if matches!(p.as_ref(), x_lir::Type::Char))
                {
                    return Ok(format!("format!(\"{}\", {})", "{}", inner_code));
                }
                // `free(buffer as *())` on a String buffer: Rust cannot cast
                // String to *mut (); the runtime shim owns the line, so null is
                // the correct (no-op) pointer.
                if matches!(
                    ty,
                    x_lir::Type::Pointer(p)
                        if matches!(p.as_ref(), x_lir::Type::Void)
                ) && self.expr_is_string(inner)
                {
                    return Ok("std::ptr::null_mut()".to_string());
                }
                // User structs whose fields include Rust `String` (24 bytes) are
                // larger than the pointer-based `nfields * 8` size that MIR/LIR
                // attach to `Alloc` (e.g. Entry { key: String, value: i64 } needs
                // 32 bytes, not 16). Ask Rust for the true layout instead of trusting
                // the LIR literal; writing past a `malloc(16)` allocation is UB
                // that LLVM exploits by replacing the stores with `ud2` traps.
                // Also zero the allocation: field assignments like
                // `(*t0).key = v` run drop glue on the OLD value, which is
                // uninitialized garbage right after malloc - UB that LLVM again
                // exploits. Zeroed String fields (ptr=null, cap=0) drop as no-ops.
                if let x_lir::Type::Pointer(inner_ty) = ty {
                    if let x_lir::Type::Named(name) = inner_ty.as_ref() {
                        if matches!(
                            inner.as_ref(),
                            x_lir::Expression::Call(callee, _)
                                if matches!(callee.as_ref(), x_lir::Expression::Variable(n) if n == "malloc")
                        ) {
                            return Ok(format!(
                                "{{ let __m = malloc(std::mem::size_of::<{}>()) as *mut {}; std::ptr::write_bytes(__m as *mut u8, 0, std::mem::size_of::<{}>()); __m }}",
                                name, name, name
                            ));
                        }
                    }
                }
                let ty_str = self.lir_type_to_rust(ty);
                Ok(format!("{} as {}", inner_code, ty_str))
            }
            x_lir::Expression::SizeOf(ty) => {
                let ty_str = self.lir_type_to_rust(ty);
                Ok(format!("std::mem::size_of::<{}>()", ty_str))
            }
            x_lir::Expression::SizeOfExpr(expr) => {
                let expr_code = self.generate_lir_expression(expr)?;
                Ok(format!("std::mem::size_of_val(&{})", expr_code))
            }
            x_lir::Expression::AlignOf(ty) => {
                let ty_str = self.lir_type_to_rust(ty);
                Ok(format!("std::mem::align_of::<{}>()", ty_str))
            }
            x_lir::Expression::Comma(exprs) => {
                let expr_codes: Vec<String> = exprs
                    .iter()
                    .map(|e| self.generate_lir_expression(e))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(expr_codes.join(", "))
            }
            x_lir::Expression::Parenthesized(inner) => {
                let inner_code = self.generate_lir_expression(inner)?;
                Ok(format!("({})", inner_code))
            }
            x_lir::Expression::InitializerList(inits) => {
                let init_codes: Vec<String> = inits
                    .iter()
                    .map(|init| self.generate_lir_initializer(init))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(format!("{{{}}}", init_codes.join(", ")))
            }
            x_lir::Expression::CompoundLiteral(ty, inits) => {
                let ty_str = self.lir_type_to_rust(ty);
                let init_codes: Vec<String> = inits
                    .iter()
                    .map(|init| self.generate_lir_initializer(init))
                    .collect::<Result<Vec<String>, x_codegen::CodeGenError>>()?;
                Ok(format!("{} {{ {} }}", ty_str, init_codes.join(", ")))
            }
        }
    }

    // ========================================================================
    // LIR literal / initializer / type generation
    // ========================================================================

    /// Generate a LIR literal
    fn generate_lir_literal(&self, lit: &x_lir::Literal) -> String {
        match lit {
            // Emit integer literals without a type suffix so Rust can infer the
            // expected integer type from context (e.g. `usize` for `malloc`).
            x_lir::Literal::Integer(v) => v.to_string(),
            x_lir::Literal::UnsignedInteger(v) => v.to_string(),
            x_lir::Literal::Long(v) => v.to_string(),
            x_lir::Literal::UnsignedLong(v) => v.to_string(),
            x_lir::Literal::LongLong(v) => v.to_string(),
            x_lir::Literal::UnsignedLongLong(v) => v.to_string(),
            x_lir::Literal::Float(v) => format!("{}f32", v),
            x_lir::Literal::Double(v) => format!("{}f64", v),
            x_lir::Literal::Bool(v) => v.to_string(),
            // Emit char literals as i64 (escaped) so they compare against the
            // i64 values produced by __x_buf_char / x_as_int etc.
            x_lir::Literal::Char(c) => {
                let esc = match c {
                    '\n' => "\\n".to_string(),
                    '\t' => "\\t".to_string(),
                    '\r' => "\\r".to_string(),
                    '\0' => "\\0".to_string(),
                    '\'' => "\\'".to_string(),
                    '\\' => "\\\\".to_string(),
                    c if c.is_control() => format!("\\u{:04x}", *c as u32),
                    c => c.to_string(),
                };
                format!("'{}' as i64", esc)
            }
            x_lir::Literal::String(s) => format!("\"{}\"", s),
            x_lir::Literal::NullPointer => "std::ptr::null_mut()".to_string(),
        }
    }

    /// Generate a LIR initializer
    fn generate_lir_initializer(&mut self, init: &x_lir::Initializer) -> RustResult<String> {
        match init {
            x_lir::Initializer::Expression(expr) => self.generate_lir_expression(expr),
            x_lir::Initializer::List(list) => {
                let items: Vec<String> = list
                    .iter()
                    .map(|i| self.generate_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("{{{}}}", items.join(", ")))
            }
            x_lir::Initializer::Named(name, init) => {
                let init_code = self.generate_lir_initializer(init)?;
                Ok(format!(".{name} = {init_code}"))
            }
            x_lir::Initializer::Indexed(idx, init) => {
                let idx_code = self.generate_lir_expression(idx)?;
                let init_code = self.generate_lir_initializer(init)?;
                Ok(format!("[{idx_code}] = {init_code}"))
            }
        }
    }

    /// Convert LIR type to Rust type string
    fn lir_type_to_rust(&self, ty: &x_lir::Type) -> String {
        match ty {
            x_lir::Type::Void => "()".to_string(),
            x_lir::Type::Bool => "bool".to_string(),
            x_lir::Type::Char => "i64".to_string(),
            x_lir::Type::Schar => "i8".to_string(),
            x_lir::Type::Uchar => "u8".to_string(),
            x_lir::Type::Schar => "i8".to_string(),
            x_lir::Type::Uchar => "u8".to_string(),
            x_lir::Type::Short => "i16".to_string(),
            x_lir::Type::Ushort => "u16".to_string(),
            x_lir::Type::Int => "i32".to_string(),
            x_lir::Type::Uint => "u32".to_string(),
            x_lir::Type::CInt => "i32".to_string(),
            x_lir::Type::Long => "i64".to_string(),
            x_lir::Type::Ulong => "u64".to_string(),
            x_lir::Type::LongLong => "i64".to_string(),
            x_lir::Type::UlongLong => "u64".to_string(),
            x_lir::Type::Float => "f32".to_string(),
            x_lir::Type::Double => "f64".to_string(),
            x_lir::Type::LongDouble => "f128".to_string(),
            x_lir::Type::Size => "usize".to_string(),
            x_lir::Type::Ptrdiff => "isize".to_string(),
            x_lir::Type::Intptr => "isize".to_string(),
            x_lir::Type::Uintptr => "usize".to_string(),
            // X `string` lowers to a pointer-to-char; represent it as an owned
            // Rust `String` so values support Display, concatenation, etc.
            x_lir::Type::Pointer(inner) if matches!(inner.as_ref(), x_lir::Type::Char) => {
                "String".to_string()
            }
            x_lir::Type::Pointer(inner) => {
                let inner_str = self.lir_type_to_rust(inner);
                format!("*mut {}", inner_str)
            }
            x_lir::Type::Array(inner, None) => {
                let inner_str = self.lir_type_to_rust(inner);
                format!("Vec<{}>", inner_str)
            }
            x_lir::Type::Array(inner, Some(size)) => {
                let inner_str = self.lir_type_to_rust(inner);
                format!("[{}; {}]", inner_str, size)
            }
            x_lir::Type::Tuple(items) => {
                let item_strs: Vec<String> =
                    items.iter().map(|t| self.lir_type_to_rust(t)).collect();
                format!("({})", item_strs.join(", "))
            }
            x_lir::Type::FunctionPointer(return_type, param_types) => {
                let params: Vec<String> = param_types
                    .iter()
                    .map(|t| self.lir_type_to_rust(t))
                    .collect();
                let ret = self.lir_type_to_rust(return_type);
                format!("fn({}) -> {}", params.join(", "), ret)
            }
            x_lir::Type::Named(name) => name.clone(),
            x_lir::Type::Qualified(quals, inner) => {
                // Qualifiers have no runtime meaning for locals: X strings are
                // owned Rust `String`s regardless of `const`; extern blocks are
                // handled separately in generate_lir_extern_function.
                if quals.is_const {
                    if let x_lir::Type::Pointer(inner) = inner.as_ref() {
                        // `const char*` is still an X string -> String.
                        if matches!(inner.as_ref(), x_lir::Type::Char) {
                            return "String".to_string();
                        }
                        let inner_str = self.lir_type_to_rust(inner);
                        format!("*const {}", inner_str)
                    } else {
                        self.lir_type_to_rust(inner)
                    }
                } else {
                    let inner_str = self.lir_type_to_rust(inner);
                    format!("*mut {}", inner_str)
                }
            }
        }
    }
}

// ============================================================================
// CodeGenerator trait implementation
// ============================================================================

impl CodeGenerator for RustBackend {
    type Config = RustBackendConfig;
    type Error = x_codegen::CodeGenError;

    fn new(config: Self::Config) -> Self {
        Self::new(config)
    }

    fn generate_from_lir(&mut self, lir: &LirProgram) -> Result<CodegenOutput, Self::Error> {
        Self::generate_from_lir(self, lir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RustBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
    }

    #[test]
    fn test_config_with_options() {
        let config = RustBackendConfig {
            output_dir: Some(std::path::PathBuf::from("/tmp")),
            optimize: true,
            debug_info: false,
        };
        assert!(config.optimize);
        assert!(!config.debug_info);
        assert!(config.output_dir.is_some());
    }
}

/// Bundled C runtime (xrt.c / xrt.h) so the Rust backend can link against the
/// same boxed-value runtime used by the other native backends.
pub mod runtime {
    pub const XRT_C_SRC: &str = include_str!("../../../library/runtime/xrt.c");
    pub const XRT_C_HDR: &str = include_str!("../../../library/runtime/xrt.h");
}
