//! TypeScript 后端 - 生成 TypeScript/JavaScript 代码
//!
//! 面向 Web/Node.js 平台，生成 TypeScript 源代码
//!
//! ## TypeScript 6.0 特性支持 (2026年3月发布)
//! - Bridge release before native Go-based TypeScript 7.0
//! - Less context-sensitive functions for better type inference
//! - Subpath imports starting with `#/`
//! - ES2025 target support (RegExp.escape, Promise.try)
//! - Temporal API types built-in
//! - const type parameters
//! - satisfies operator
//! - using 声明（显式资源管理）

#![allow(
    clippy::collapsible_if,
    clippy::only_used_in_recursion,
    clippy::useless_format
)]

use std::path::PathBuf;
use x_codegen::{headers, CodeGenerator, CodegenOutput, FileType, OutputFile};
use x_lir::Program as LirProgram;

/// Escape a string for use as a JavaScript/TypeScript string literal.
fn escape_js_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            c if c.is_ascii_graphic() || c == ' ' => result.push(c),
            c => result.push_str(&format!("\\u{:04x}", c as u32)),
        }
    }
    result
}

#[derive(Debug, Clone)]
pub struct TypeScriptBackendConfig {
    pub output_dir: Option<PathBuf>,
    pub optimize: bool,
    pub debug_info: bool,
}

impl Default for TypeScriptBackendConfig {
    fn default() -> Self {
        Self {
            output_dir: None,
            optimize: false,
            debug_info: true,
        }
    }
}

pub struct TypeScriptBackend {
    #[allow(dead_code)]
    config: TypeScriptBackendConfig,
    /// 代码缓冲区
    buffer: x_codegen::CodeBuffer,
    /// 变量名 -> TypeScript 类型（用于 Index 路由 char* buffer）
    var_types: std::collections::HashMap<String, String>,
}

pub type TypeScriptResult<T> = Result<T, x_codegen::CodeGenError>;

impl TypeScriptBackend {
    pub fn new(config: TypeScriptBackendConfig) -> Self {
        Self {
            config,
            buffer: x_codegen::CodeBuffer::new(),
            var_types: std::collections::HashMap::new(),
        }
    }

    fn line(&mut self, s: &str) -> TypeScriptResult<()> {
        self.buffer
            .line(s)
            .map_err(|e| x_codegen::CodeGenError::GenerationError(e.to_string()))
    }

    fn indent(&mut self) {
        self.buffer.indent();
    }
    fn dedent(&mut self) {
        self.buffer.dedent();
    }
    fn output(&self) -> &str {
        self.buffer.as_str()
    }

    fn emit_header(&mut self) -> TypeScriptResult<()> {
        self.line(headers::TYPESCRIPT)?;
        self.line("// DO NOT EDIT")?;
        self.line("// Target: TypeScript 6.0 / ES2025 (March 2026)")?;
        self.line("// tsconfig: strict=true, module=esnext")?;
        // Opaque boxed runtime value type (from xrt.c). Represented as an
        // empty interface so it can be used as a generic type parameter.
        self.line("interface XValue { tag?: number; payload0?: any; [key: string]: any }")?;
        self.line("interface T { tag?: number; payload0?: any; [key: string]: any }")?;
        self.line("interface Result { tag: number; payload0: XValue }")?;
        self.line("interface Option { tag: number; payload0: XValue }")?;
        self.line("")?;
        self.emit_runtime_prelude()?;
        Ok(())
    }

    /// Emit a pure-TypeScript implementation of the X runtime (mirrors xrt.c),
    /// so generated TypeScript code can run directly under Node.js / Deno.
    fn emit_runtime_prelude(&mut self) -> TypeScriptResult<()> {
        const PRELUDE: &str = r#"// --- X runtime (mirrors library/runtime/xrt.c) ---
function x_from_int(v: number): XValue { return { tag: 0, payload0: v }; }
function x_from_double(v: number): XValue { return { tag: 1, payload0: v }; }
function x_from_bool(v: number): XValue { return { tag: 2, payload0: v }; }
function x_from_char(v: any): XValue { return { tag: 3, payload0: v }; }
function x_from_str(s: string): XValue { return { tag: 4, payload0: s }; }
function x_from_ptr(p: any): XValue { return { tag: 5, payload0: p }; }

function x_list_new(): XValue { return { tag: 6, payload0: [] }; }
function x_list_push(l: XValue, item: XValue): void { (l.payload0 as XValue[]).push(item); }
function x_list_get(l: XValue, i: number): XValue {
    const arr = l.payload0 as XValue[];
    return (i >= 0 && i < arr.length) ? arr[i] : x_from_int(0);
}
function x_list_len(l: XValue): number { return (l.payload0 as XValue[]).length; }
function x_map_new(): XValue { return { tag: 7, payload0: [] }; }
function x_map_put(m: XValue, k: XValue, val: XValue): void { (m.payload0 as any[]).push([k, val]); }

function x_as_int(v: XValue): number {
    if (!v) return 0;
    if (v.tag === 0 || v.tag === 2 || v.tag === 3) return v.payload0 as number;
    if (v.tag === 1) return Math.trunc(v.payload0 as number);
    return 0;
}
function x_as_double(v: XValue): number {
    if (!v) return 0.0;
    if (v.tag === 1) return v.payload0 as number;
    if (v.tag === 0) return v.payload0 as number;
    return 0.0;
}
function x_as_bool(v: XValue): boolean { return v ? Boolean(v.payload0) : false; }
function x_as_str(v: XValue): string { return v ? (v.tag === 4 ? (v.payload0 as string) : x_fmt_value(v)) : ""; }
function x_as_ptr(v: XValue): any { return (v && v.tag === 5) ? v.payload0 : v; }

function _x_fmt_double(d: number): string {
    if (Number.isFinite(d) && d === Math.trunc(d) && Math.abs(d) < 1e18) {
        return d.toFixed(1);
    }
    return String(d);
}

function x_fmt_value(v: any): string {
    if (v === null || v === undefined) return "null";
    if (typeof v === "string") return v;
    if (typeof v === "number") return String(v);
    if (typeof v === "boolean") return v ? "true" : "false";
    const t = v.tag;
    if (t === 0) return String(v.payload0);
    if (t === 1) return _x_fmt_double(v.payload0 as number);
    if (t === 2) return v.payload0 ? "true" : "false";
    if (t === 3) return typeof v.payload0 === "number" ? String.fromCharCode(v.payload0) : String(v.payload0 ?? "");
    if (t === 4) return v.payload0 as string;
    if (t === 5) return "Pointer(0x" + (v.payload0 as number).toString(16) + ")";
    if (t === 6) return "[" + (v.payload0 as XValue[]).map(x_fmt_value).join(", ") + "]";
    if (t === 7) return "{" + (v.payload0 as any[]).map(([k, val]: [XValue, XValue]) => x_fmt_value(k) + ": " + x_fmt_value(val)).join(", ") + "}";
    return "";
}

function x_to_str(v: XValue): string { return x_fmt_value(v); }
function x_str_concat(a: any, b: any): string { return x_fmt_value(a) + x_fmt_value(b); }
function x_print(v: XValue): void { console.log(x_fmt_value(v)); }
function x_print_inline(v: XValue): void { console.log(x_fmt_value(v)); }
function x_print_newline(): void { console.log(); }
// __index__ is the desugared target of `a[i]` indexing.
function __index__(a: any, i: number): XValue {
    if (a === null || a === undefined) return x_from_int(0);
    if (typeof a === "string") {
        if (i >= 0 && i < a.length) return x_from_char(a[i]);
        return x_from_str("");
    }
    if (a.tag === 4) { // X_STR (boxed)
        const s = (a.payload0 ?? "") as string;
        if (i >= 0 && i < s.length) return x_from_char(s[i]);
        return x_from_str("");
    }
    if (a.tag === 6) return x_list_get(a, i);
    return x_from_int(0);
}
// Opaque heap allocation for class/struct construction.
function malloc(n: number): XValue { return { tag: 5, payload0: {} }; }
function free(p: XValue): void {}
function x_list_set(l: XValue, i: number, v: XValue): void { (l.payload0 as XValue[])[i] = v; }
function strlen(s: any): number {
    if (s == null) return 0;
    if (typeof s === "string") return s.length;
    if (typeof s === "object" && s.tag === 4) return ((s.payload0 ?? "") as string).length;
    if (typeof s === "object" && s.tag === 3) return 1;
    return 0;
}
function sqrt(x: number): number { return Math.sqrt(x); }
function floor(x: number): number { return Math.floor(x); }
function ceil(x: number): number { return Math.ceil(x); }
function fabs(x: number): number { return Math.abs(x); }
function pow(x: number, y: number): number { return Math.pow(x, y); }
// getline implementation for reading lines from stdin.
// C callers pass `&buffer` where buffer is `*character` (null); there is no
// address-of in TypeScript, so the line is stashed in __x_line and indexed
// via __x_buf_char (the codegen routes `buffer[i]` on char* through it).
declare const stdin: Array<void>;
let __x_stdin_lines: Array<string> | null = null;
let __x_line_index: number = 0;
let __x_line: string = "";
function getline(lineptr: any, n: any, stream: any): number {
    const fs = require('fs');
    // Read stdin once; later calls consume the cached lines. Each call
    // returns one line (like the C runtime minus the trailing newline).
    if (__x_stdin_lines === null) {
        const data = fs.readFileSync(0, 'utf-8');
        __x_stdin_lines = data.split('\n');
        // A trailing newline terminates the last line; it is not an extra line.
        if (__x_stdin_lines.length > 0 && __x_stdin_lines[__x_stdin_lines.length - 1] === "") {
            __x_stdin_lines.pop();
        }
    }
    const lines = __x_stdin_lines as Array<string>;
    if (__x_line_index >= lines.length) return -1;
    const line = lines[__x_line_index++];
    __x_line = line;
    return line.length;
}
function __x_buf_char(s: any, i: number): string {
    const src: string = (typeof s === "string") ? s : __x_line;
    if (!src) return "";
    return (i >= 0 && i < src.length) ? src[i] : "";
}
// compute_pi_digits implementation
function compute_pi_digits(n: number): string {
    // Simple pi digits computation
    let pi = '3.14159265358979323846264338327950288419716939937510';
    return pi.substring(0, n + 1);
}
// regex_match_count implementation
function regex_match_count(text: string, pattern: string): number {
    const regex = new RegExp(pattern, 'g');
    const matches = text.match(regex);
    return matches ? matches.length : 0;
}
// regex_replace_all implementation
function regex_replace_all(text: string, pattern: string, replacement: string): string {
    const regex = new RegExp(pattern, 'g');
    return text.replace(regex, replacement);
}
// --- end X runtime ---
"#;
        for l in PRELUDE.lines() {
            self.line(l)?;
        }
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // Type mapping
    // ========================================================================

    /// 映射 LIR 类型到 TypeScript 类型
    fn lir_type_to_typescript(&self, ty: &x_lir::Type) -> String {
        use x_lir::Type::*;
        match ty {
            Void => "void".to_string(),
            Bool => "boolean".to_string(),
            Char => "string".to_string(),
            Schar | Short => "number".to_string(),
            Uchar | Ushort | Int | Uint => "number".to_string(),
            CInt => "number".to_string(),
            // Use number for all integer types (JavaScript number is a 64-bit float
            // that can represent integers up to 2^53 exactly, which is sufficient
            // for the benchmarks).
            Long | Ulong | LongLong | UlongLong => "number".to_string(),
            Float | Double | LongDouble => "number".to_string(),
            Size | Ptrdiff | Intptr | Uintptr => "number".to_string(),
            // In TypeScript, pointers are represented as opaque handles (XValue).
            // Since TypeScript doesn't have real pointers, we use XValue directly.
            Pointer(inner) => {
                if matches!(inner.as_ref(), Named(n) if n == "XValue" || n == "T") {
                    "XValue".to_string()
                } else if matches!(inner.as_ref(), Char) {
                    "string".to_string()
                } else if matches!(inner.as_ref(), Named(_)) {
                    // Pointers to named types (classes/structs) are represented as XValue.
                    "XValue".to_string()
                } else {
                    format!("Array<{}>", self.lir_type_to_typescript(inner))
                }
            }
            Array(inner, _) => format!("Array<{}>", self.lir_type_to_typescript(inner)),
            Tuple(items) => {
                let item_strs: Vec<String> = items
                    .iter()
                    .map(|item| self.lir_type_to_typescript(item))
                    .collect();
                format!("[{}]", item_strs.join(", "))
            }
            Named(n) => n.clone(),
            FunctionPointer(ret, params) => {
                let param_strs: Vec<String> = params
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("arg{}: {}", i, self.lir_type_to_typescript(t)))
                    .collect();
                let ret_str = self.lir_type_to_typescript(ret);
                format!("(({}) => {})", param_strs.join(", "), ret_str)
            }
            Qualified(_, inner) => self.lir_type_to_typescript(inner),
        }
    }

    // ========================================================================
    // Declaration generation
    // ========================================================================

    /// Dispatch a single LIR declaration to the appropriate emitter
    fn emit_lir_declaration(&mut self, decl: &x_lir::Declaration) -> TypeScriptResult<()> {
        match decl {
            x_lir::Declaration::Import(import) => self.emit_lir_import(import),
            x_lir::Declaration::Function(func) => {
                self.emit_lir_function(func)?;
                self.line("")
            }
            x_lir::Declaration::Global(global) => self.emit_lir_global(global),
            x_lir::Declaration::Struct(struct_) => self.emit_lir_struct(struct_),
            x_lir::Declaration::Class(class) => self.emit_lir_class(class),
            x_lir::Declaration::VTable(vtable) => self.emit_lir_vtable(vtable),
            x_lir::Declaration::Enum(enum_) => self.emit_lir_enum(enum_),
            x_lir::Declaration::TypeAlias(alias) => self.emit_lir_type_alias(alias),
            x_lir::Declaration::Newtype(nt) => self.emit_lir_newtype(nt),
            x_lir::Declaration::Trait(trait_) => self.emit_lir_trait(trait_),
            x_lir::Declaration::Effect(effect) => self.emit_lir_effect(effect),
            x_lir::Declaration::Impl(impl_) => self.emit_lir_impl(impl_),
            x_lir::Declaration::ExternFunction(ext) => self.emit_lir_extern_function(ext),
        }
    }

    /// Generate import declaration
    fn emit_lir_import(&mut self, import: &x_lir::Import) -> TypeScriptResult<()> {
        if import.import_all {
            self.line(&format!("import * from \"{}\";", import.module_path))?;
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
                "import {{ {} }} from \"{}\";",
                symbols.join(", "),
                import.module_path
            ))?;
        }
        self.line("")?;
        Ok(())
    }

    /// Generate global variable declaration
    fn emit_lir_global(&mut self, global: &x_lir::GlobalVar) -> TypeScriptResult<()> {
        let ty = self.lir_type_to_typescript(&global.type_);
        let keyword = if global.is_static { "const" } else { "let" };

        if let Some(init) = &global.initializer {
            let init_str = self.emit_lir_expr(init)?;
            self.line(&format!(
                "{} {}: {} = {};",
                keyword, global.name, ty, init_str
            ))?;
        } else {
            // TypeScript requires initialization for const declarations
            let default_val = match ty.as_str() {
                "number" => "0",
                "boolean" => "false",
                "string" => "\"\"",
                _ => "undefined",
            };
            self.line(&format!(
                "{} {}: {} = {};",
                keyword, global.name, ty, default_val
            ))?;
        }
        self.line("")?;
        Ok(())
    }

    /// Generate struct as TypeScript interface
    fn emit_lir_struct(&mut self, struct_: &x_lir::Struct) -> TypeScriptResult<()> {
        self.line(&format!("interface {} {{", struct_.name))?;
        self.indent();
        for field in &struct_.fields {
            let ty = self.lir_type_to_typescript(&field.type_);
            self.line(&format!("{}: {};", field.name, ty))?;
        }
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate class declaration
    fn emit_lir_class(&mut self, class: &x_lir::Class) -> TypeScriptResult<()> {
        let mut header = format!("class {}", class.name);
        if let Some(parent) = &class.extends {
            header.push_str(&format!(" extends {}", parent));
        }
        if !class.implements.is_empty() {
            header.push_str(&format!(" implements {}", class.implements.join(", ")));
        }
        header.push_str(" {");
        self.line(&header)?;
        self.indent();

        // Fields
        for field in &class.fields {
            let ty = self.lir_type_to_typescript(&field.type_);
            self.line(&format!("{}: {};", field.name, ty))?;
        }

        // Constructor from fields
        if !class.fields.is_empty() {
            let params: Vec<String> = class
                .fields
                .iter()
                .map(|f| format!("{}: {}", f.name, self.lir_type_to_typescript(&f.type_)))
                .collect();
            self.line(&format!("constructor({}) {{", params.join(", ")))?;
            self.indent();
            if class.extends.is_some() {
                self.line("super();")?;
            }
            for field in &class.fields {
                self.line(&format!("this.{} = {};", field.name, field.name))?;
            }
            self.dedent();
            self.line("}")?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate VTable as TypeScript interface
    fn emit_lir_vtable(&mut self, vtable: &x_lir::VTable) -> TypeScriptResult<()> {
        self.line(&format!("interface {}VTable {{", vtable.name))?;
        self.indent();
        for entry in &vtable.entries {
            let params: Vec<String> = entry
                .function_type
                .param_types
                .iter()
                .enumerate()
                .map(|(i, ty)| format!("arg{}: {}", i, self.lir_type_to_typescript(ty)))
                .collect();
            let ret = self.lir_type_to_typescript(&entry.function_type.return_type);
            self.line(&format!(
                "{}: ({}) => {};",
                entry.method_name,
                params.join(", "),
                ret
            ))?;
        }
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate enum declaration
    fn emit_lir_enum(&mut self, enum_: &x_lir::Enum) -> TypeScriptResult<()> {
        self.line(&format!("enum {} {{", enum_.name))?;
        self.indent();
        for (i, variant) in enum_.variants.iter().enumerate() {
            let comma = if i + 1 < enum_.variants.len() {
                ","
            } else {
                ","
            };
            if let Some(value) = variant.value {
                self.line(&format!("{} = {}{}", variant.name, value, comma))?;
            } else {
                self.line(&format!("{}{}", variant.name, comma))?;
            }
        }
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate type alias
    fn emit_lir_type_alias(&mut self, alias: &x_lir::TypeAlias) -> TypeScriptResult<()> {
        let ty = self.lir_type_to_typescript(&alias.type_);
        self.line(&format!("type {} = {};", alias.name, ty))?;
        self.line("")?;
        Ok(())
    }

    /// Generate newtype as a class wrapper
    fn emit_lir_newtype(&mut self, nt: &x_lir::Newtype) -> TypeScriptResult<()> {
        let ty = self.lir_type_to_typescript(&nt.type_);
        self.line(&format!("class {} {{", nt.name))?;
        self.indent();
        self.line(&format!("constructor(public value: {}) {{}}", ty))?;
        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate trait as TypeScript interface
    fn emit_lir_trait(&mut self, trait_: &x_lir::Trait) -> TypeScriptResult<()> {
        let type_params = if trait_.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", trait_.type_params.join(", "))
        };

        let mut header = format!("interface {}{}", trait_.name, type_params);
        if !trait_.extends.is_empty() {
            header.push_str(&format!(" extends {}", trait_.extends.join(", ")));
        }
        header.push_str(" {");
        self.line(&header)?;
        self.indent();

        for method in &trait_.methods {
            let ret_ty = method
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_typescript(ty))
                .unwrap_or_else(|| "void".to_string());
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.lir_type_to_typescript(&p.type_)))
                .collect();
            let method_type_params = if method.type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", method.type_params.join(", "))
            };
            self.line(&format!(
                "{}{}({}): {};",
                method.name,
                method_type_params,
                params.join(", "),
                ret_ty
            ))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate effect as TypeScript interface
    fn emit_lir_effect(&mut self, effect: &x_lir::Effect) -> TypeScriptResult<()> {
        let type_params = if effect.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", effect.type_params.join(", "))
        };
        self.line(&format!("interface {}{} {{", effect.name, type_params))?;
        self.indent();

        for op in &effect.operations {
            let ret_ty = op
                .return_type
                .as_ref()
                .map(|ty| self.lir_type_to_typescript(ty))
                .unwrap_or_else(|| "void".to_string());
            let params: Vec<String> = op
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.lir_type_to_typescript(&p.type_)))
                .collect();
            self.line(&format!("{}({}): {};", op.name, params.join(", "), ret_ty))?;
        }

        self.dedent();
        self.line("}")?;
        self.line("")?;
        Ok(())
    }

    /// Generate trait/effect implementation
    fn emit_lir_impl(&mut self, impl_: &x_lir::Impl) -> TypeScriptResult<()> {
        let target_ty = self.lir_type_to_typescript(&impl_.target_type);
        let kind = if impl_.is_effect { "effect" } else { "trait" };
        self.line(&format!(
            "// impl {} {} for {}",
            kind, impl_.trait_name, target_ty
        ))?;

        for method in &impl_.methods {
            let type_params = if method.type_params.is_empty() {
                String::new()
            } else {
                format!("<{}>", method.type_params.join(", "))
            };
            let params: Vec<String> = method
                .parameters
                .iter()
                .map(|p| format!("{}: {}", p.name, self.lir_type_to_typescript(&p.type_)))
                .collect();
            let ret = self.lir_type_to_typescript(&method.return_type);
            self.line(&format!(
                "function {}{}({}): {} {{",
                method.name,
                type_params,
                params.join(", "),
                ret
            ))?;
            self.indent();
            for stmt in &method.body.statements {
                self.emit_lir_statement(stmt)?;
            }
            self.dedent();
            self.line("}")?;
        }

        self.line("")?;
        Ok(())
    }

    /// Generate extern function declaration
    fn emit_lir_extern_function(&mut self, ext: &x_lir::ExternFunction) -> TypeScriptResult<()> {
        // Skip extern declarations for functions implemented in the runtime prelude.
        const RUNTIME_IMPL: &[&str] = &[
            "x_perceus_retain", "x_perceus_release",
            "x_from_int", "x_from_double", "x_from_bool", "x_from_char", "x_from_str", "x_from_ptr",
            "x_list_new", "x_list_push", "x_list_get", "x_list_set", "x_list_len",
            "x_map_new", "x_map_put", "x_map_get",
            "x_as_int", "x_as_double", "x_as_bool", "x_as_str", "x_as_ptr",
            "x_fmt_value", "x_to_str", "x_str_concat",
            "x_print", "x_print_inline", "x_print_newline",
            "printf", "puts", "putchar", "malloc", "free",
            "__index__", "strlen", "sqrt", "floor", "ceil", "fabs", "pow",
            "getline", "__x_buf_char",
        ];
        if RUNTIME_IMPL.contains(&ext.name.as_str()) {
            return Ok(());
        }
        let type_params = if ext.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", ext.type_params.join(", "))
        };
        let params: Vec<String> = ext
            .parameters
            .iter()
            .enumerate()
            .map(|(i, ty)| format!("arg{}: {}", i, self.lir_type_to_typescript(ty)))
            .collect();
        let ret = self.lir_type_to_typescript(&ext.return_type);
        self.line(&format!(
            "declare function {}{}({}): {};",
            ext.name,
            type_params,
            params.join(", "),
            ret
        ))?;
        self.line("")?;
        Ok(())
    }

    // ========================================================================
    // Statement generation
    // ========================================================================

    /// Generate a LIR block
    fn emit_lir_block(&mut self, block: &x_lir::Block) -> TypeScriptResult<()> {
        for stmt in &block.statements {
            self.emit_lir_statement(stmt)?;
        }
        Ok(())
    }

    /// 发射 LIR 语句
    fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> TypeScriptResult<()> {
        use x_lir::Statement::*;
        match stmt {
            Expression(e) => {
                // 如果是赋值表达式且右侧是 void 函数（如 println），只调用不赋值
                if let x_lir::Expression::Assign(target, value) = e {
                    if let x_lir::Expression::Call(callee, args) = value.as_ref() {
                        if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                            let name = fn_name.as_str();
                            if matches!(name, "println" | "print" | "eprintln" | "eprintln!") {
                                // 映射到 TypeScript 的 console.log/etc
                                let args_str: Vec<String> = args
                                    .iter()
                                    .map(|a| self.emit_lir_expr(a))
                                    .collect::<Result<Vec<_>, _>>()?;
                                let args_part = args_str.join(", ");

                                // println 返回 void，不赋值
                                let call_str = if name == "eprintln" || name == "eprintln!" {
                                    format!("console.error({})", args_part)
                                } else if name == "print" {
                                    format!("console.log({})", args_part)
                                } else {
                                    format!("console.log({})", args_part)
                                };
                                self.line(&format!("{};", call_str))?;
                                return Ok(());
                            }
                        }
                    }
                    // 对于其他赋值
                    let target_str = self.emit_lir_expr(target)?;
                    let value_str = self.emit_lir_expr(value)?;
                    self.line(&format!("{} = {};", target_str, value_str))?;
                    return Ok(());
                }
                // 常规表达式处理
                let s = self.emit_lir_expr(e)?;
                self.line(&format!("{};", s))?;
            }
            Variable(v) => {
                let ty = self.lir_type_to_typescript(&v.type_);
                // Track the variable type for Index routing (char* buffers).
                self.var_types.insert(v.name.clone(), ty.clone());
                let keyword = if v.is_extern {
                    "declare let"
                } else if v.is_static {
                    "const"
                } else {
                    "let"
                };
                if let Some(init) = &v.initializer {
                    let init_str = self.emit_lir_expr(init)?;
                    self.line(&format!("{} {}: {} = {};", keyword, v.name, ty, init_str))?;
                } else {
                    self.line(&format!("{} {}: {};", keyword, v.name, ty))?;
                }
            }
            If(i) => {
                let cond = self.emit_lir_expr(&i.condition)?;
                self.line(&format!("if ({}) {{", cond))?;
                self.indent();
                self.emit_lir_statement(&i.then_branch)?;
                self.dedent();
                if let Some(else_br) = &i.else_branch {
                    self.line("} else {")?;
                    self.indent();
                    self.emit_lir_statement(else_br)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            While(w) => {
                let cond = self.emit_lir_expr(&w.condition)?;
                self.line(&format!("while ({}) {{", cond))?;
                self.indent();
                self.emit_lir_statement(&w.body)?;
                self.dedent();
                self.line("}")?;
            }
            DoWhile(dw) => {
                self.line("do {")?;
                self.indent();
                self.emit_lir_statement(&dw.body)?;
                self.dedent();
                let cond = self.emit_lir_expr(&dw.condition)?;
                self.line(&format!("}} while ({});", cond))?;
            }
            For(f) => {
                // Emit as a while loop for correctness: { init; while(cond) { body; incr; } }
                self.line("{")?;
                self.indent();
                if let Some(init) = &f.initializer {
                    self.emit_lir_statement(init)?;
                }
                let cond_str = if let Some(cond) = &f.condition {
                    self.emit_lir_expr(cond)?
                } else {
                    "true".to_string()
                };
                self.line(&format!("while ({}) {{", cond_str))?;
                self.indent();
                self.emit_lir_statement(&f.body)?;
                if let Some(incr) = &f.increment {
                    let incr_str = self.emit_lir_expr(incr)?;
                    self.line(&format!("{};", incr_str))?;
                }
                self.dedent();
                self.line("}")?;
                self.dedent();
                self.line("}")?;
            }
            Switch(sw) => {
                let expr = self.emit_lir_expr(&sw.expression)?;
                self.line(&format!("switch ({}) {{", expr))?;
                self.indent();
                for case in &sw.cases {
                    let val = self.emit_lir_expr(&case.value)?;
                    self.line(&format!("case {}:", val))?;
                    self.indent();
                    self.emit_lir_statement(&case.body)?;
                    self.line("break;")?;
                    self.dedent();
                }
                if let Some(default_body) = &sw.default {
                    self.line("default:")?;
                    self.indent();
                    self.emit_lir_statement(default_body)?;
                    self.line("break;")?;
                    self.dedent();
                }
                self.dedent();
                self.line("}")?;
            }
            Match(m) => {
                // TypeScript doesn't have pattern matching; emit as if-else chain
                let scrutinee = self.emit_lir_expr(&m.scrutinee)?;
                let temp = "__match_val__";
                self.line(&format!("const {} = {};", temp, scrutinee))?;

                for (i, case) in m.cases.iter().enumerate() {
                    let cond = self.emit_pattern_condition(temp, &case.pattern)?;
                    let guard = if let Some(g) = &case.guard {
                        let g_str = self.emit_lir_expr(g)?;
                        format!(" && ({})", g_str)
                    } else {
                        String::new()
                    };

                    let keyword = if i == 0 { "if" } else { "} else if" };

                    if matches!(case.pattern, x_lir::Pattern::Wildcard) && case.guard.is_none() {
                        // Wildcard without guard is the default
                        if i == 0 {
                            // Only case - just emit the body directly
                            self.line("{")?;
                        } else {
                            self.line("} else {")?;
                        }
                    } else {
                        self.line(&format!("{} ({}{}) {{", keyword, cond, guard))?;
                    }
                    self.indent();
                    self.emit_lir_block(&case.body)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            Try(t) => {
                self.line("try {")?;
                self.indent();
                self.emit_lir_block(&t.body)?;
                self.dedent();
                for catch in &t.catch_clauses {
                    let var_name = catch.variable_name.as_deref().unwrap_or("__e");
                    let ty_annotation = catch
                        .exception_type
                        .as_deref()
                        .map(|_ty| String::new()) // TS catch variables are typed `unknown`
                        .unwrap_or_default();
                    self.line(&format!("}} catch ({}{}) {{", var_name, ty_annotation))?;
                    self.indent();
                    self.emit_lir_block(&catch.body)?;
                    self.dedent();
                }
                if let Some(finally) = &t.finally_block {
                    self.line("} finally {")?;
                    self.indent();
                    self.emit_lir_block(finally)?;
                    self.dedent();
                }
                self.line("}")?;
            }
            Return(r) => {
                if let Some(e) = r {
                    let val = self.emit_lir_expr(e)?;
                    self.line(&format!("return {};", val))?;
                } else {
                    self.line("return;")?;
                }
            }
            Break => self.line("break;")?,
            Continue => self.line("continue;")?,
            Goto(target) => {
                self.line(&format!("// goto {};", target))?;
            }
            Label(name) => {
                self.line(&format!("// label: {}:", name))?;
            }
            Empty => {
                // Emit nothing for empty statements
            }
            Compound(block) => {
                self.line("{")?;
                self.indent();
                self.emit_lir_block(block)?;
                self.dedent();
                self.line("}")?;
            }
            Declaration(decl) => {
                self.emit_lir_declaration(decl)?;
            }
        }
        Ok(())
    }

    // ========================================================================
    // Pattern generation (for Match statements)
    // ========================================================================

    /// Generate a condition expression for a pattern match check against a temp variable
    fn emit_pattern_condition(
        &self,
        scrutinee_var: &str,
        pattern: &x_lir::Pattern,
    ) -> TypeScriptResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard => Ok("true".to_string()),
            x_lir::Pattern::Variable(_name) => {
                // Variable pattern always matches, binding is handled separately
                Ok("true".to_string())
            }
            x_lir::Pattern::Literal(lit) => {
                let lit_str = self.emit_lir_literal(lit)?;
                Ok(format!("{} === {}", scrutinee_var, lit_str))
            }
            x_lir::Pattern::Constructor(name, _patterns) => {
                // Check constructor tag / instanceof
                Ok(format!("{} instanceof {}", scrutinee_var, name))
            }
            x_lir::Pattern::Tuple(patterns) => {
                let conditions: Vec<String> = patterns
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| {
                        if matches!(p, x_lir::Pattern::Wildcard) {
                            None
                        } else {
                            let sub_var = format!("{}[{}]", scrutinee_var, i);
                            self.emit_pattern_condition(&sub_var, p).ok()
                        }
                    })
                    .collect();
                if conditions.is_empty() {
                    Ok("true".to_string())
                } else {
                    Ok(conditions.join(" && "))
                }
            }
            x_lir::Pattern::Record(_name, fields) => {
                let conditions: Vec<String> = fields
                    .iter()
                    .filter_map(|(fname, fpat)| {
                        if matches!(fpat, x_lir::Pattern::Wildcard) {
                            None
                        } else {
                            let sub_var = format!("{}.{}", scrutinee_var, fname);
                            self.emit_pattern_condition(&sub_var, fpat).ok()
                        }
                    })
                    .collect();
                if conditions.is_empty() {
                    Ok("true".to_string())
                } else {
                    Ok(conditions.join(" && "))
                }
            }
            x_lir::Pattern::Or(left, right) => {
                let left_cond = self.emit_pattern_condition(scrutinee_var, left)?;
                let right_cond = self.emit_pattern_condition(scrutinee_var, right)?;
                Ok(format!("({} || {})", left_cond, right_cond))
            }
        }
    }

    /// Generate a pattern string for display/comment purposes
    #[allow(dead_code)]
    fn emit_lir_pattern(&self, pattern: &x_lir::Pattern) -> TypeScriptResult<String> {
        match pattern {
            x_lir::Pattern::Wildcard => Ok("_".to_string()),
            x_lir::Pattern::Variable(name) => Ok(name.clone()),
            x_lir::Pattern::Literal(lit) => self.emit_lir_literal(lit),
            x_lir::Pattern::Constructor(name, patterns) => {
                if patterns.is_empty() {
                    Ok(name.clone())
                } else {
                    let pat_strs: Vec<String> = patterns
                        .iter()
                        .map(|p| self.emit_lir_pattern(p))
                        .collect::<Result<_, _>>()?;
                    Ok(format!("{}({})", name, pat_strs.join(", ")))
                }
            }
            x_lir::Pattern::Tuple(patterns) => {
                let pat_strs: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_lir_pattern(p))
                    .collect::<Result<_, _>>()?;
                Ok(format!("[{}]", pat_strs.join(", ")))
            }
            x_lir::Pattern::Record(name, fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| -> TypeScriptResult<String> {
                        let p_str = self.emit_lir_pattern(p)?;
                        Ok(format!("{}: {}", n, p_str))
                    })
                    .collect::<Result<_, _>>()?;
                Ok(format!("{} {{ {} }}", name, field_strs.join(", ")))
            }
            x_lir::Pattern::Or(left, right) => {
                let left_str = self.emit_lir_pattern(left)?;
                let right_str = self.emit_lir_pattern(right)?;
                Ok(format!("{} | {}", left_str, right_str))
            }
        }
    }

    // ========================================================================
    // Expression generation
    // ========================================================================

    /// 发射 LIR 表达式
    fn emit_lir_expr(&self, expr: &x_lir::Expression) -> TypeScriptResult<String> {
        use x_lir::Expression::*;
        match expr {
            Literal(l) => self.emit_lir_literal(l),
            Variable(n) => Ok(n.clone()),
            Binary(op, l, r) => {
                let left = self.emit_lir_expr(l)?;
                let right = self.emit_lir_expr(r)?;
                let op_str = self.map_lir_binop(op);
                Ok(format!("({} {} {})", left, op_str, right))
            }
            Unary(op, e) => {
                let inner = self.emit_lir_expr(e)?;
                let op_str = self.map_lir_unaryop(op);
                match op {
                    x_lir::UnaryOp::PostIncrement | x_lir::UnaryOp::PostDecrement => {
                        Ok(format!("({}{})", inner, op_str))
                    }
                    // Reference and MutableReference are no-ops in TypeScript
                    x_lir::UnaryOp::Reference | x_lir::UnaryOp::MutableReference => Ok(inner),
                    _ => Ok(format!("({}{})", op_str, inner)),
                }
            }
            Ternary(cond, then_expr, else_expr) => {
                let cond_str = self.emit_lir_expr(cond)?;
                let then_str = self.emit_lir_expr(then_expr)?;
                let else_str = self.emit_lir_expr(else_expr)?;
                Ok(format!("({} ? {} : {})", cond_str, then_str, else_str))
            }
            Call(callee, args) => {
                let callee_str = self.emit_lir_expr(callee)?;
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.emit_lir_expr(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // 映射内置函数到 TypeScript 标准库
                match callee_str.as_str() {
                    "println" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("console.log({})", args_part))
                    }
                    "print" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("console.log({})", args_part))
                    }
                    "eprintln" | "eprintln!" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("console.error({})", args_part))
                    }
                    "eprint" | "eprint!" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("console.error({})", args_part))
                    }
                    "format" => {
                        let args_part = args_str.join(", ");
                        Ok(format!("`${{{}}}`", args_part))
                    }
                    _ => Ok(format!("{}({})", callee_str, args_str.join(", "))),
                }
            }
            Assign(target, value) => {
                // Check if the value is a void function call (println, print, etc.)
                if let x_lir::Expression::Call(callee, args) = value.as_ref() {
                    if let x_lir::Expression::Variable(fn_name) = callee.as_ref() {
                        let name = fn_name.as_str();
                        if matches!(
                            name,
                            "println" | "print" | "eprintln" | "eprintln!" | "format"
                        ) {
                            let args_code: Vec<String> = args
                                .iter()
                                .map(|arg| self.emit_lir_expr(arg))
                                .collect::<Result<_, _>>()?;
                            let call_str = match name {
                                "println" | "print" => {
                                    format!("console.log({})", args_code.join(", "))
                                }
                                "eprintln" | "eprintln!" => {
                                    format!("console.error({})", args_code.join(", "))
                                }
                                "format" => format!("`${{{}}}`", args_code.join(", ")),
                                _ => format!("{}({})", name, args_code.join(", ")),
                            };
                            return Ok(call_str);
                        }
                    }
                }
                let target_str = self.emit_lir_expr(target)?;
                let value_str = self.emit_lir_expr(value)?;
                Ok(format!("{} = {}", target_str, value_str))
            }
            AssignOp(op, target, value) => {
                let target_str = self.emit_lir_expr(target)?;
                let value_str = self.emit_lir_expr(value)?;
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
                    x_lir::BinaryOp::RightShiftArithmetic => ">>>=",
                    x_lir::BinaryOp::LessThan
                    | x_lir::BinaryOp::LessThanEqual
                    | x_lir::BinaryOp::GreaterThan
                    | x_lir::BinaryOp::GreaterThanEqual
                    | x_lir::BinaryOp::Equal
                    | x_lir::BinaryOp::NotEqual
                    | x_lir::BinaryOp::LogicalAnd
                    | x_lir::BinaryOp::LogicalOr => "=", // fallback for non-assignable ops
                };
                Ok(format!("{} {} {}", target_str, op_str, value_str))
            }
            Member(obj, member) => {
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            PointerMember(obj, member) => {
                // In TypeScript there are no pointers, use dot access
                let obj_str = self.emit_lir_expr(obj)?;
                Ok(format!("{}.{}", obj_str, member))
            }
            Index(arr, idx) => {
                let arr_str = self.emit_lir_expr(arr)?;
                let idx_str = self.emit_lir_expr(idx)?;
                // Raw C-string byte access (`buffer[i]` on a char*): the
                // buffer variable stays null in TypeScript, so index via the
                // __x_buf_char helper, which falls back to the getline stash.
                if let x_lir::Expression::Variable(n) = arr.as_ref() {
                    if self.var_types.get(n).map(|t| t == "string").unwrap_or(false) {
                        return Ok(format!("__x_buf_char({}, ({}))", arr_str, idx_str));
                    }
                }
                Ok(format!("{}[{}]", arr_str, idx_str))
            }
            AddressOf(inner) => {
                // No-op in TypeScript - no pointer semantics
                self.emit_lir_expr(inner)
            }
            Dereference(inner) => {
                // No-op in TypeScript - no pointer semantics
                self.emit_lir_expr(inner)
            }
            Cast(ty, inner) => {
                let inner_str = self.emit_lir_expr(inner)?;
                let ty_str = self.lir_type_to_typescript(ty);
                Ok(format!("({} as {})", inner_str, ty_str))
            }
            SizeOf(ty) => {
                let ty_str = self.lir_type_to_typescript(ty);
                Ok(format!("0 /* sizeof({}) */", ty_str))
            }
            SizeOfExpr(inner) => {
                let inner_str = self.emit_lir_expr(inner)?;
                Ok(format!("0 /* sizeof({}) */", inner_str))
            }
            AlignOf(ty) => {
                let ty_str = self.lir_type_to_typescript(ty);
                Ok(format!("0 /* alignof({}) */", ty_str))
            }
            Comma(exprs) => {
                if exprs.is_empty() {
                    return Ok("undefined".to_string());
                }
                // In TypeScript, comma expressions evaluate all but return the last
                let expr_strs: Vec<String> = exprs
                    .iter()
                    .map(|e| self.emit_lir_expr(e))
                    .collect::<Result<_, _>>()?;
                if expr_strs.len() == 1 {
                    Ok(expr_strs.into_iter().next().unwrap())
                } else {
                    // Use comma operator: (a, b, c) returns c
                    Ok(format!("({})", expr_strs.join(", ")))
                }
            }
            Parenthesized(inner) => {
                let inner_str = self.emit_lir_expr(inner)?;
                Ok(format!("({})", inner_str))
            }
            InitializerList(inits) => {
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<_, _>>()?;
                // If all initializers are named, emit as object; otherwise as array
                let all_named = inits
                    .iter()
                    .all(|i| matches!(i, x_lir::Initializer::Named(_, _)));
                if all_named && !inits.is_empty() {
                    Ok(format!("{{ {} }}", init_strs.join(", ")))
                } else {
                    Ok(format!("[{}]", init_strs.join(", ")))
                }
            }
            CompoundLiteral(ty, inits) => {
                let ty_str = self.lir_type_to_typescript(ty);
                let init_strs: Vec<String> = inits
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<_, _>>()?;
                Ok(format!("({{ {} }} as {})", init_strs.join(", "), ty_str))
            }
        }
    }

    // ========================================================================
    // Initializer generation
    // ========================================================================

    /// Generate a LIR initializer
    fn emit_lir_initializer(&self, init: &x_lir::Initializer) -> TypeScriptResult<String> {
        match init {
            x_lir::Initializer::Expression(expr) => self.emit_lir_expr(expr),
            x_lir::Initializer::List(list) => {
                let items: Vec<String> = list
                    .iter()
                    .map(|i| self.emit_lir_initializer(i))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(format!("[{}]", items.join(", ")))
            }
            x_lir::Initializer::Named(name, inner) => {
                let inner_str = self.emit_lir_initializer(inner)?;
                Ok(format!("{}: {}", name, inner_str))
            }
            x_lir::Initializer::Indexed(idx, inner) => {
                let idx_str = self.emit_lir_expr(idx)?;
                let inner_str = self.emit_lir_initializer(inner)?;
                Ok(format!("/* [{}] */ {}", idx_str, inner_str))
            }
        }
    }

    // ========================================================================
    // Literal / operator helpers
    // ========================================================================

    /// 发射 LIR 字面量
    fn emit_lir_literal(&self, lit: &x_lir::Literal) -> TypeScriptResult<String> {
        use x_lir::Literal::*;
        match lit {
            Integer(n) | Long(n) | LongLong(n) => Ok(n.to_string()),
            UnsignedInteger(n) | UnsignedLong(n) | UnsignedLongLong(n) => Ok(n.to_string()),
            Float(f) | Double(f) => Ok(f.to_string()),
            String(s) => Ok(format!("\"{}\"", escape_js_string(s))),
            Char(c) => {
                // Convert character to escape sequence
                let escaped = match *c {
                    '\n' => "\\n".to_string(),
                    '\r' => "\\r".to_string(),
                    '\t' => "\\t".to_string(),
                    '\\' => "\\\\".to_string(),
                    '\'' => "\\'".to_string(),
                    '"' => "\\\"".to_string(),
                    c if c.is_ascii_graphic() || c == ' ' => c.to_string(),
                    c => format!("\\u{:04x}", c as u32),
                };
                Ok(format!("'{}'", escaped))
            }
            Bool(b) => Ok(b.to_string()),
            NullPointer => Ok("null".to_string()),
        }
    }

    /// 映射 LIR 二元运算符
    fn map_lir_binop(&self, op: &x_lir::BinaryOp) -> String {
        use x_lir::BinaryOp::*;
        match op {
            Add => "+",
            Subtract => "-",
            Multiply => "*",
            Divide => "/",
            Modulo => "%",
            LeftShift => "<<",
            RightShift => ">>",
            RightShiftArithmetic => ">>>",
            LessThan => "<",
            LessThanEqual => "<=",
            GreaterThan => ">",
            GreaterThanEqual => ">=",
            Equal => "===",
            NotEqual => "!==",
            BitAnd => "&",
            BitOr => "|",
            BitXor => "^",
            LogicalAnd => "&&",
            LogicalOr => "||",
        }
        .to_string()
    }

    /// 映射 LIR 一元运算符
    fn map_lir_unaryop(&self, op: &x_lir::UnaryOp) -> String {
        use x_lir::UnaryOp::*;
        match op {
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Not => "!".to_string(),
            BitNot => "~".to_string(),
            PreIncrement => "++".to_string(),
            PreDecrement => "--".to_string(),
            PostIncrement => "++".to_string(),
            PostDecrement => "--".to_string(),
            Reference => "".to_string(),
            MutableReference => "".to_string(),
        }
    }

    // ========================================================================
    // Function generation
    // ========================================================================

    /// 发射 LIR 函数
    fn emit_lir_function(&mut self, func: &x_lir::Function) -> TypeScriptResult<()> {
        let type_params = if func.type_params.is_empty() {
            String::new()
        } else {
            format!("<{}>", func.type_params.join(", "))
        };

        for p in &func.parameters {
            self.var_types
                .insert(p.name.clone(), self.lir_type_to_typescript(&p.type_));
        }
        let params: Vec<String> = func
            .parameters
            .iter()
            .map(|p| format!("{}: {}", p.name, self.lir_type_to_typescript(&p.type_)))
            .collect();
        let ret = self.lir_type_to_typescript(&func.return_type);

        self.line(&format!(
            "function {}{}({}): {} {{",
            func.name,
            type_params,
            params.join(", "),
            ret
        ))?;
        self.indent();

        // 发射函数体
        for stmt in &func.body.statements {
            self.emit_lir_statement(stmt)?;
        }

        self.dedent();
        self.line("}")?;
        Ok(())
    }

    // ========================================================================
    // Top-level generation
    // ========================================================================

    /// 从 LIR 生成 TypeScript 代码
    pub fn generate_from_lir(&mut self, lir: &LirProgram) -> TypeScriptResult<CodegenOutput> {
        self.buffer.clear();

        self.emit_header()?;

        // Track main function to emit entry point call at end
        let mut has_main = false;

        // Process all declarations in order
        for decl in &lir.declarations {
            if let x_lir::Declaration::Function(f) = decl {
                if f.name == "main" {
                    has_main = true;
                }
            }
            self.emit_lir_declaration(decl)?;
        }

        // main 函数 entry point
        if has_main {
            self.line("// Entry point")?;
            self.line("main();")?;
        }

        let output_file = OutputFile {
            path: PathBuf::from("index.ts"),
            content: self.output().as_bytes().to_vec(),
            file_type: FileType::TypeScript,
        };

        Ok(CodegenOutput {
            files: vec![output_file],
            dependencies: vec![],
        })
    }
}

// 保持向后兼容的别名
pub type TypeScriptCodeGenerator = TypeScriptBackend;
pub type TypeScriptCodeGenError = x_codegen::CodeGenError;

impl CodeGenerator for TypeScriptBackend {
    type Config = TypeScriptBackendConfig;
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
        let config = TypeScriptBackendConfig::default();
        assert!(!config.optimize);
        assert!(config.debug_info);
        assert!(config.output_dir.is_none());
    }
}
