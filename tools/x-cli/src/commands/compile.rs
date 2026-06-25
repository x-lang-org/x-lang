use crate::pipeline;
use crate::utils;
use x_codegen::CodeGenerator;
use x_codegen::Target;
use x_codegen_csharp::{CSharpBackend, CSharpConfig};
use x_codegen_erlang::{ErlangBackend, ErlangBackendConfig};
use x_codegen_java::{JavaBackend, JavaConfig};
use x_codegen_llvm::{LlvmBackend, LlvmBackendConfig};
use x_codegen_native::{NativeBackend, NativeBackendConfig};
use x_codegen_python::{PythonBackend, PythonBackendConfig};
use x_codegen_rust::{RustBackend, RustBackendConfig};
use x_codegen_swift::{SwiftBackend, SwiftBackendConfig};
use x_codegen_typescript::{TypeScriptBackend, TypeScriptBackendConfig};
use x_codegen_zig::{ZigBackend, ZigBackendConfig, ZigTarget};

/// 捆绑的 C 运行时源码（与 native 目标文件一并编译链接）。
const X_RUNTIME_SRC: &str = include_str!("../../../../library/runtime/xrt.c");
const X_RUNTIME_HDR: &str = include_str!("../../../../library/runtime/xrt.h");

/// 把 `--target` 字符串解析为 Native 后端的 (架构, 操作系统)。
/// 返回 None 表示这不是一个 Native 三元组（应交给其它后端处理）。
fn parse_native_triple(
    t: &str,
) -> Option<(x_codegen_native::TargetArch, x_codegen_native::TargetOS)> {
    use x_codegen_native::{TargetArch, TargetOS};

    if t == "native" {
        let arch = match std::env::consts::ARCH {
            "x86_64" => TargetArch::X86_64,
            "aarch64" => TargetArch::AArch64,
            "riscv64" => TargetArch::RiscV64,
            _ => TargetArch::X86_64,
        };
        let os = match std::env::consts::OS {
            "linux" => TargetOS::Linux,
            "macos" => TargetOS::MacOS,
            "windows" => TargetOS::Windows,
            _ => TargetOS::Linux,
        };
        return Some((arch, os));
    }

    let lower = t.to_lowercase();
    let arch = if lower.starts_with("x86_64") || lower.starts_with("amd64") {
        TargetArch::X86_64
    } else if lower.starts_with("aarch64") || lower.starts_with("arm64") {
        TargetArch::AArch64
    } else if lower.starts_with("riscv64") {
        TargetArch::RiscV64
    } else {
        return None;
    };

    let os = if lower.contains("linux") {
        TargetOS::Linux
    } else if lower.contains("darwin") || lower.contains("macos") || lower.contains("apple") {
        TargetOS::MacOS
    } else if lower.contains("windows") || lower.contains("win32") || lower.contains("mingw") {
        TargetOS::Windows
    } else {
        // 仅给出架构（如 "aarch64"）时默认 Linux
        TargetOS::Linux
    };

    Some((arch, os))
}

/// 把 (arch, os) 映射为 `zig cc -target <triple>` 使用的 Zig 三元组
fn zig_cc_triple(arch: x_codegen_native::TargetArch, os: x_codegen_native::TargetOS) -> String {
    use x_codegen_native::{TargetArch, TargetOS};
    let a = match arch {
        TargetArch::X86_64 => "x86_64",
        TargetArch::AArch64 => "aarch64",
        TargetArch::RiscV64 => "riscv64",
        TargetArch::Wasm32 => "wasm32",
    };
    match os {
        // 交叉 Linux 用 musl 以便静态链接（qemu-user 可直接运行，无需目标 sysroot）
        TargetOS::Linux => format!("{}-linux-musl", a),
        TargetOS::MacOS => format!("{}-macos", a),
        TargetOS::Windows => format!("{}-windows-gnu", a),
    }
}

/// 获取当前主机平台的目标三元组
fn get_host_target_triple() -> &'static str {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match (arch, os) {
        ("x86_64", "macos") => "x86_64-apple-darwin",
        ("x86_64", "linux") => "x86_64-pc-linux-gnu",
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "macos") => "arm64-apple-darwin",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        _ => "x86_64-pc-linux-gnu",
    }
}

#[allow(unused_variables)]
pub fn exec(
    file: &str,
    output: Option<&str>,
    emit: Option<&str>,
    no_link: bool,
    release: bool,
    target: Option<&str>,
) -> Result<(), String> {
    let content =
        std::fs::read_to_string(file).map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    if let Some(stage) = emit {
        return emit_stage(file, &content, stage);
    }

    let out_path = output.unwrap_or_else(|| file.strip_suffix(".x").unwrap_or(file));

    // Parse target
    let parsed_target = match target {
        None | Some("native") => Target::Native,
        Some("wasm" | "wasm32-wasi" | "wasm32-freestanding") => Target::Zig,
        Some("ts" | "typescript") => Target::TypeScript,
        Some("zig") => Target::Zig,
        Some("erlang" | "erl") => Target::Erlang,
        Some("llvm" | "ll") => Target::Llvm,
        Some("rust" | "rs") => Target::Rust,
        Some("python" | "py") => Target::Python,
        Some("java") => Target::Java,
        Some("csharp" | "cs" | "dotnet" | "net") => Target::CSharp,
        Some("swift") => Target::Swift,
        Some(t) => {
            if let Some(t) = Target::from_str(t) {
                t
            } else if parse_native_triple(t).is_some() {
                // 形如 x86_64-linux / aarch64-linux / riscv64-linux / *-macos / *-windows
                Target::Native
            } else {
                return Err(format!(
                    "未知目标平台: {}（支持: native, <arch>-<os> 三元组, wasm, wasm32-wasi, wasm32-freestanding, zig, ts/typescript, erlang/erl, python/py, rust/rs, java, csharp/cs/dotnet, swift）",
                    t
                ));
            }
        }
    };

    // Run the full compiler pipeline: source → imports/prelude → AST → HIR → MIR → LIR
    let project_dir = std::path::Path::new(file)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let pipeline_output = pipeline::run_pipeline_with_project_dir(&content, project_dir)?;

    // All backends use the full pipeline via LIR
    match parsed_target {
        // ── Native 后端 - LIR 直出机器码（可重定位 ELF），仅用系统链接器 ──────
        Target::Native => {
            use x_codegen_native::{OutputFormat, TargetOS};

            // 解析目标三元组 →(arch, os)；默认主机
            let triple_str = target.unwrap_or("native");
            let (arch, os) = parse_native_triple(triple_str)
                .ok_or_else(|| format!("无法解析 Native 目标三元组: {}", triple_str))?;

            // 是否为交叉编译：非主机三元组（或显式给出 triple）。
            let is_host = matches!(triple_str, "native")
                || (arch_matches_host(arch) && os == TargetOS::Linux && cfg!(target_os = "linux"));

            let mut backend = NativeBackend::new(NativeBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
                arch,
                format: OutputFormat::ObjectFile,
                os,
            });

            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("Native 代码生成失败: {}", e))?;

            let obj_bytes = &codegen_output.files[0].content;

            // 目标文件扩展名按 OS
            let obj_ext = if os == TargetOS::Windows { "obj" } else { "o" };
            let temp_dir = std::env::temp_dir();
            let obj_path = temp_dir.join(format!("x_native_output.{}", obj_ext));
            std::fs::write(&obj_path, obj_bytes).map_err(|e| format!("无法写入目标文件: {}", e))?;

            // 输出路径：Windows 目标补 .exe
            let mut output_path = std::path::PathBuf::from(out_path);
            if os == TargetOS::Windows && output_path.extension().is_none() {
                output_path.set_extension("exe");
            }

            // 写出捆绑的 C 运行时（xrt.c + xrt.h），与目标文件一并编译链接。
            let runtime_path = temp_dir.join("xrt.c");
            let header_path = temp_dir.join("xrt.h");
            std::fs::write(&header_path, X_RUNTIME_HDR)
                .map_err(|e| format!("无法写入运行时头文件: {}", e))?;
            std::fs::write(&runtime_path, X_RUNTIME_SRC)
                .map_err(|e| format!("无法写入运行时源文件: {}", e))?;

            if is_host {
                // 主机：直接用系统链接器（cc）
                link_object_linux(&obj_path, &output_path, &runtime_path)?;
            } else {
                // 交叉：用 zig cc -target <triple>（自带各平台 libc/交叉链接）
                link_object_zig(&obj_path, &output_path, arch, os, &runtime_path)?;
            }

            let _ = std::fs::remove_file(&runtime_path);
            let _ = std::fs::remove_file(&header_path);

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&output_path) {
                    let mut perms = meta.permissions();
                    perms.set_mode(perms.mode() | 0o755);
                    let _ = std::fs::set_permissions(&output_path, perms);
                }
            }

            let _ = std::fs::remove_file(&obj_path);

            eprintln!("编译成功: {}", output_path.display());
        }
        // ── Zig-based targets (Native + Wasm) ────────────────────────────────
        Target::Zig => {
            let zig_target = match target {
                None | Some("native") => ZigTarget::Native,
                Some("wasm" | "wasm32-wasi") => ZigTarget::Wasm32Wasi,
                Some("wasm32-freestanding") => ZigTarget::Wasm32Freestanding,
                _ => ZigTarget::Native,
            };
            let mut backend = ZigBackend::new(ZigBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
                target: zig_target,
            });
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("Zig代码生成失败: {}", e))?;

            let zig_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            let output_path = std::path::PathBuf::from(out_path);

            if let Some(t_str) = target {
                if t_str != "native" {
                    if let Some(tgt) = match t_str {
                        "wasm" | "wasm32-wasi" => Some(ZigTarget::Wasm32Wasi),
                        "wasm32-freestanding" => Some(ZigTarget::Wasm32Freestanding),
                        _ => None,
                    } {
                        utils::status("Target", tgt.as_zig_target());
                    }
                }
            }

            backend
                .compile_zig_code(&zig_code, &output_path)
                .map_err(|e| format!("Zig编译失败: {}", e))?;

            eprintln!("编译成功: {}", output_path.display());
        }

        // ── TypeScript target ─────────────────────────────────────────────────
        // TypeScript is the primary JS-family target.
        // With --no-link: emit a .ts file only.
        // Without --no-link: also invoke `tsc` to compile .ts → .js
        Target::TypeScript => {
            let mut backend = TypeScriptBackend::new(TypeScriptBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
            });
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("TypeScript代码生成失败: {}", e))?;

            let ts_code = String::from_utf8_lossy(&codegen_output.files[0].content);

            // Always write the .ts source
            let ts_out_path = format!("{}.ts", out_path);
            std::fs::write(&ts_out_path, ts_code.as_bytes())
                .map_err(|e| format!("无法写入TypeScript文件 {}: {}", ts_out_path, e))?;

            if no_link {
                eprintln!("已生成TypeScript代码: {}", ts_out_path);
                return Ok(());
            }

            // Compile .ts → .js via tsc
            let out_dir = std::path::Path::new(out_path)
                .parent()
                .unwrap_or(std::path::Path::new("."));

            let status = std::process::Command::new("tsc")
                .arg("--outDir")
                .arg(out_dir)
                .arg("--target")
                .arg("ES2020")
                .arg("--module")
                .arg("commonjs")
                .arg("--strict")
                .arg("--esModuleInterop")
                .arg(&ts_out_path)
                .status();

            match status {
                Ok(s) if s.success() => {
                    let js_path = std::path::Path::new(&ts_out_path).with_extension("js");
                    eprintln!("编译成功: {}", js_path.display());
                }
                Ok(_) => {
                    eprintln!("已生成TypeScript代码: {}", ts_out_path);
                    return Err("TypeScript编译失败 (tsc 返回非零退出码)".to_string());
                }
                Err(_) => {
                    eprintln!("已生成TypeScript代码: {}", ts_out_path);
                    eprintln!(
                        "提示: 安装TypeScript后可编译为JavaScript: npm install -g typescript"
                    );
                    eprintln!("      然后运行: tsc {}", ts_out_path);
                    return Err("tsc 未找到，请安装 TypeScript".to_string());
                }
            }
        }

        // ── LLVM 后端 ─────────────────────────────────────────────────────────
        Target::Llvm => {
            let target_triple = get_host_target_triple();
            let mut backend = LlvmBackend::new(LlvmBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
                target_triple: Some(target_triple.to_string()),
                module_name: "main".to_string(),
            });

            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("LLVM代码生成失败: {}", e))?;

            let llvm_ir = String::from_utf8_lossy(&codegen_output.files[0].content);
            let output_path = std::path::PathBuf::from(out_path);

            // 写入 .ll 文件
            let ll_path = output_path.with_extension("ll");
            std::fs::write(&ll_path, llvm_ir.as_bytes())
                .map_err(|e| format!("无法写入LLVM IR文件: {}", e))?;

            if no_link {
                eprintln!("已生成LLVM IR: {}", ll_path.display());
                return Ok(());
            }

            // 使用 clang 编译 .ll → .o 并链接
            let obj_path = output_path.with_extension("o");

            let clang_status = std::process::Command::new("clang")
                .arg("-c")
                .arg("-o")
                .arg(&obj_path)
                .arg(&ll_path)
                .status()
                .map_err(|e| format!("运行clang失败: {}", e))?;

            if !clang_status.success() {
                return Err(format!("clang 编译失败"));
            }

            let clangxx_path = which::which("clang++")
                .or_else(|_| which::which("clang"))
                .map_err(|_| "未找到 clang（请安装 Clang）".to_string())?;

            let link_status = std::process::Command::new(&clangxx_path)
                .arg("-o")
                .arg(&output_path)
                .arg(&obj_path)
                .status()
                .map_err(|e| format!("链接失败: {}", e))?;

            if !link_status.success() {
                return Err(format!("链接失败"));
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&output_path)
                    .map_err(|e| e.to_string())?
                    .permissions();
                perms.set_mode(perms.mode() | 0o755);
                std::fs::set_permissions(&output_path, perms).map_err(|e| e.to_string())?;
            }

            let _ = std::fs::remove_file(&obj_path);
            eprintln!("编译成功: {}", output_path.display());
        }

        // ── Rust target ─────────────────────────────────────────────────────────
        // Generates Rust source, then compiles to executable via cargo
        Target::Rust => {
            let mut backend = RustBackend::new(RustBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
            });
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("Rust代码生成失败: {}", e))?;

            let rust_code = String::from_utf8_lossy(&codegen_output.files[0].content);

            // Write the .rs source
            let rs_out_path = format!("{}.rs", out_path);
            std::fs::write(&rs_out_path, rust_code.as_bytes())
                .map_err(|e| format!("无法写入Rust文件: {}", e))?;

            if no_link {
                eprintln!("已生成Rust代码: {}", rs_out_path);
                return Ok(());
            }

            // Compile .rs → exe via cargo
            let output_path = std::path::PathBuf::from(out_path);
            let exe_path = if cfg!(windows) {
                output_path.with_extension("exe")
            } else {
                output_path
            };

            match RustBackend::compile_rust(&rust_code, &exe_path) {
                Ok(final_path) => {
                    eprintln!("编译成功: {}", final_path.display());
                }
                Err(e) => {
                    eprintln!("已生成Rust代码: {}", rs_out_path);
                    eprintln!("提示: 安装 Rust 后可编译为可执行文件");
                    eprintln!("      然后运行: cargo build --release {}", rs_out_path);
                    return Err(format!("Rust编译失败: {}", e));
                }
            }
        }

        // ── Erlang target ───────────────────────────────────────────────────────
        // Generates .erl source file. With --no-link: emit .erl only.
        // Without --no-link: also run erl to execute the script
        Target::Erlang => {
            let mut backend = ErlangBackend::new(ErlangBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
                module_name: Some("main".to_string()),
            });
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("Erlang代码生成失败: {}", e))?;

            let erl_code = String::from_utf8_lossy(&codegen_output.files[0].content);

            // Always write the .erl source
            let erl_out_path = format!("{}.erl", out_path);
            std::fs::write(&erl_out_path, erl_code.as_bytes())
                .map_err(|e| format!("无法写入Erlang文件 {}: {}", erl_out_path, e))?;

            if no_link {
                eprintln!("已生成Erlang代码: {}", erl_out_path);
                return Ok(());
            }

            // Run Erlang to execute the script
            let erl_path = which::which("erl")
                .or_else(|_| which::which("escript"))
                .map_err(|_| "未找到 Erlang 解释器（请安装 Erlang）".to_string())?;

            // Get the directory and module name
            let out_dir = std::path::Path::new(out_path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let module_name = "main";

            // Compile and run using erl
            let status = std::process::Command::new(&erl_path)
                .arg("-n")
                .arg("-eval")
                .arg(format!("c('{}'), main:main(), halt(0).", erl_out_path))
                .current_dir(out_dir)
                .status()
                .map_err(|e| format!("执行Erlang失败: {}", e))?;

            if status.success() {
                eprintln!("执行成功");
            } else {
                return Err(format!("Erlang 脚本执行失败，退出码: {:?}", status.code()));
            }
        }

        // ── Python target ─────────────────────────────────────────────────────────
        Target::Python => {
            let mut backend = PythonBackend::new(PythonBackendConfig {
                output_dir: None,
                optimize: release,
                debug_info: !release,
            });
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("Python代码生成失败: {}", e))?;

            let py_code = String::from_utf8_lossy(&codegen_output.files[0].content);

            // Write the .py source
            let py_out_path = format!("{}.py", out_path);
            std::fs::write(&py_out_path, py_code.as_bytes())
                .map_err(|e| format!("无法写入Python文件: {}", e))?;

            if no_link {
                eprintln!("已生成Python代码: {}", py_out_path);
                return Ok(());
            }

            // Run Python to execute the script
            let python_path = which::which("python3")
                .or_else(|_| which::which("python"))
                .map_err(|_| "未找到 Python 解释器（请安装 Python）".to_string())?;

            let status = std::process::Command::new(&python_path)
                .arg(&py_out_path)
                .status()
                .map_err(|e| format!("执行Python失败: {}", e))?;

            if status.success() {
                eprintln!("执行成功");
            } else {
                return Err(format!("Python 脚本执行失败，退出码: {:?}", status.code()));
            }
        }

        // ── Java target ────────────────────────────────────────────────────────────
        Target::Java => {
            let mut backend = JavaBackend::new(JavaConfig::default());
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("Java代码生成失败: {}", e))?;

            let java_code = String::from_utf8_lossy(&codegen_output.files[0].content);

            // Write the .java source (always use Main.java to match class name)
            let out_dir = std::path::Path::new(out_path)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let java_out_path = out_dir.join("Main.java");

            // Create directory if needed
            std::fs::create_dir_all(out_dir).map_err(|e| format!("无法创建目录: {}", e))?;
            std::fs::write(&java_out_path, java_code.as_bytes())
                .map_err(|e| format!("无法写入Java文件: {}", e))?;

            if no_link {
                eprintln!("已生成Java代码: {}", java_out_path.display());
                return Ok(());
            }

            // Find javac and java
            let javac_path =
                which::which("javac").map_err(|_| "未找到 javac（请安装 JDK）".to_string())?;

            // Compile .java → .class
            let compile_status = std::process::Command::new(&javac_path)
                .arg(&java_out_path)
                .status()
                .map_err(|e| format!("Java编译失败: {}", e))?;

            if !compile_status.success() {
                return Err(format!("javac 编译失败"));
            }

            // Run the Java program
            let java_path =
                which::which("java").map_err(|_| "未找到 java（请安装 JRE/JDK）".to_string())?;

            let run_status = std::process::Command::new(&java_path)
                .arg("Main")
                .current_dir(out_dir)
                .status()
                .map_err(|e| format!("执行Java失败: {}", e))?;

            if run_status.success() {
                eprintln!("执行成功");
            } else {
                return Err(format!(
                    "Java 程序执行失败，退出码: {:?}",
                    run_status.code()
                ));
            }
        }

        // ── C#/.NET target ─────────────────────────────────────────────────────────
        Target::CSharp => {
            let mut backend = CSharpBackend::new(CSharpConfig::default());
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("C#代码生成失败: {}", e))?;

            let csharp_code = String::from_utf8_lossy(&codegen_output.files[0].content);

            // Write the .cs source
            let cs_out_path = format!("{}.cs", out_path);
            std::fs::write(&cs_out_path, csharp_code.as_bytes())
                .map_err(|e| format!("无法写入C#文件: {}", e))?;

            if no_link {
                eprintln!("已生成C#代码: {}", cs_out_path);
                return Ok(());
            }

            // Try dotnet CLI first, fallback to mono
            let dotnet_path = which::which("dotnet").ok();
            let mcs_path = which::which("mcs").ok();
            let mono_path = which::which("mono").ok();

            if let Some(dotnet) = dotnet_path {
                // Use dotnet to run the C# code
                // Create a temporary project
                let temp_dir = std::env::temp_dir().join("xlang_csharp_build");
                let _ = std::fs::remove_dir_all(&temp_dir);
                std::fs::create_dir_all(&temp_dir)
                    .map_err(|e| format!("创建临时目录失败: {}", e))?;

                let cs_file = temp_dir.join("Program.cs");
                std::fs::write(&cs_file, csharp_code.as_bytes())
                    .map_err(|e| format!("写入临时文件失败: {}", e))?;

                let csproj_content = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>disable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
</Project>"#;
                let csproj_file = temp_dir.join("csharp.csproj");
                std::fs::write(&csproj_file, csproj_content)
                    .map_err(|e| format!("写入项目文件失败: {}", e))?;

                // Build and run
                let build_status = std::process::Command::new(&dotnet)
                    .arg("run")
                    .current_dir(&temp_dir)
                    .status()
                    .map_err(|e| format!("dotnet run 失败: {}", e))?;

                if build_status.success() {
                    eprintln!("执行成功");
                    return Ok(());
                } else {
                    return Err(format!(
                        "dotnet run 执行失败，退出码: {:?}",
                        build_status.code()
                    ));
                }
            } else if let (Some(mcs), Some(mono)) = (mcs_path, mono_path) {
                // Use Mono to compile and run
                let exe_out_path = if cfg!(windows) {
                    std::path::PathBuf::from(out_path).with_extension("exe")
                } else {
                    std::path::PathBuf::from(out_path)
                };

                let compile_status = std::process::Command::new(&mcs)
                    .arg("-out:")
                    .arg(&exe_out_path)
                    .arg(&cs_out_path)
                    .status()
                    .map_err(|e| format!("Mono编译失败: {}", e))?;

                if !compile_status.success() {
                    return Err(format!("mcs 编译失败"));
                }

                let run_status = std::process::Command::new(&mono)
                    .arg(&exe_out_path)
                    .status()
                    .map_err(|e| format!("执行Mono失败: {}", e))?;

                if run_status.success() {
                    eprintln!("执行成功");
                } else {
                    return Err(format!(
                        "Mono 程序执行失败，退出码: {:?}",
                        run_status.code()
                    ));
                }
            } else {
                return Err("未找到 dotnet 或 mono（请安装 .NET SDK 或 Mono）".to_string());
            }
        }

        // ── Swift target ───────────────────────────────────────────────────────────
        Target::Swift => {
            let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
            let codegen_output = backend
                .generate_from_lir(&pipeline_output.lir)
                .map_err(|e| format!("Swift代码生成失败: {}", e))?;

            let swift_code = String::from_utf8_lossy(&codegen_output.files[0].content);

            // Write the .swift source
            let swift_out_path = format!("{}.swift", out_path);
            std::fs::write(&swift_out_path, swift_code.as_bytes())
                .map_err(|e| format!("无法写入Swift文件: {}", e))?;

            if no_link {
                eprintln!("已生成Swift代码: {}", swift_out_path);
                return Ok(());
            }

            // Find swiftc
            let swiftc_path = which::which("swiftc")
                .or_else(|_| which::which("swift"))
                .map_err(|_| "未找到 swiftc（请安装 Swift）".to_string())?;

            // Compile .swift → executable
            let exe_out_path = if cfg!(windows) {
                std::path::PathBuf::from(out_path).with_extension("exe")
            } else {
                std::path::PathBuf::from(out_path)
            };

            let compile_status = std::process::Command::new(&swiftc_path)
                .arg("-o")
                .arg(&exe_out_path)
                .arg(&swift_out_path)
                .status()
                .map_err(|e| format!("Swift编译失败: {}", e))?;

            if !compile_status.success() {
                return Err(format!("swiftc 编译失败"));
            }

            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&exe_out_path)
                    .map_err(|e| e.to_string())?
                    .permissions();
                perms.set_mode(perms.mode() | 0o755);
                std::fs::set_permissions(&exe_out_path, perms).map_err(|e| e.to_string())?;
            }

            // Run the executable
            let run_status = std::process::Command::new(&exe_out_path)
                .status()
                .map_err(|e| format!("执行Swift失败: {}", e))?;

            if run_status.success() {
                eprintln!("执行成功");
            } else {
                return Err(format!(
                    "Swift 程序执行失败，退出码: {:?}",
                    run_status.code()
                ));
            }
        }
    }

    Ok(())
}

fn emit_stage(file: &str, content: &str, stage: &str) -> Result<(), String> {
    match stage.to_lowercase().as_str() {
        "tokens" => {
            let mut lexer = x_lexer::Lexer::new(content);
            loop {
                match lexer.next_token() {
                    Ok((token, span)) => {
                        println!("{:?}  @ {}..{}", token, span.start, span.end);
                        if matches!(token, x_lexer::token::Token::Eof) {
                            break;
                        }
                    }
                    Err(e) => {
                        return Err(format!("词法错误: {:?}", e));
                    }
                }
            }
            Ok(())
        }
        "ast" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let program = pipeline::prepare_program(content, project_dir)
                .map_err(|e| format!("{}", e))?;
            println!("{:#?}", program);
            Ok(())
        }
        // ── Backend source-emit options ──────────────────────────────────────
        "zig" => {
            // 使用完整流水线 LIR → Zig 代码生成
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = ZigBackend::new(
                ZigBackendConfig::default(),
            );
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("Zig代码生成失败: {}", e))?;
            let zig_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", zig_code);
            Ok(())
        }
        // TypeScript / JavaScript: both emit from LIR for accuracy
        "ts" | "typescript" | "js" | "javascript" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend =
                TypeScriptBackend::new(TypeScriptBackendConfig::default());
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("TypeScript代码生成失败: {}", e))?;
            let ts_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", ts_code);
            Ok(())
        }
        "java" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = JavaBackend::new(JavaConfig::default());
            let codegen_output = backend.generate_from_lir(&output.lir)
                .map_err(|e| format!("Java代码生成失败: {}", e))?;
            let java_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", java_code);
            Ok(())
        }
        "dotnet" | "csharp" => {
            // 使用 LIR 进行代码生成（符合编译流水线要求）
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = CSharpBackend::new(
                CSharpConfig::default(),
            );
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("C#代码生成失败: {}", e))?;
            let csharp_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", csharp_code);
            Ok(())
        }
        "rust" => {
            // 使用 LIR 进行代码生成（符合编译流水线要求）
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = RustBackend::new(
                RustBackendConfig::default(),
            );
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("Rust代码生成失败: {}", e))?;
            let rust_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", rust_code);
            Ok(())
        }
        "swift" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = SwiftBackend::new(SwiftBackendConfig::default());
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("Swift代码生成失败: {}", e))?;
            let swift_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", swift_code);
            Ok(())
        }
        "c" => {
            // C 后端使用 Rust 后端生成 C 风格代码
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = RustBackend::new(
                RustBackendConfig::default(),
            );
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("C代码生成失败: {}", e))?;
            let c_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", c_code);
            Ok(())
        }
        "erlang" | "erl" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = ErlangBackend::new(ErlangBackendConfig::default());
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("Erlang代码生成失败: {}", e))?;
            let erl_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", erl_code);
            Ok(())
        }
        "python" | "py" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let mut backend = PythonBackend::new(PythonBackendConfig::default());
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("Python代码生成失败: {}", e))?;
            let py_code = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", py_code);
            Ok(())
        }
        "llvm" | "ll" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            let target_triple = get_host_target_triple();
            let mut backend = LlvmBackend::new(
                LlvmBackendConfig {
                    output_dir: None,
                    optimize: false,
                    debug_info: true,
                    target_triple: Some(target_triple.to_string()),
                    module_name: "main".to_string(),
                },
            );
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("LLVM代码生成失败: {}", e))?;
            let llvm_ir = String::from_utf8_lossy(&codegen_output.files[0].content);
            println!("{}", llvm_ir);
            Ok(())
        }
        // ── IR dump options ───────────────────────────────────────────────────
        "hir" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            println!("{:#?}", output.hir);
            Ok(())
        }
        "mir" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            println!("{:#?}", output.mir);
            Ok(())
        }
        "lir" => {
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            println!("{:#?}", output.lir);
            Ok(())
        }
        "native" | "obj" => {
            // 输出 Native 后端直出的可重定位 ELF 目标文件字节（原始二进制 → stdout）
            if !cfg!(target_arch = "x86_64") || !cfg!(target_os = "linux") {
                return Err(format!(
                    "Native 后端目前仅支持 x86_64 Linux 主机（当前: {} {}）",
                    std::env::consts::ARCH,
                    std::env::consts::OS
                ));
            }
            let project_dir = std::path::Path::new(file)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let output = pipeline::run_pipeline_with_project_dir(content, project_dir)?;
            use x_codegen_native::{NativeBackend, NativeBackendConfig, OutputFormat, TargetArch, TargetOS};
            let mut backend = NativeBackend::new(NativeBackendConfig {
                output_dir: None,
                optimize: false,
                debug_info: true,
                arch: TargetArch::X86_64,
                format: OutputFormat::ObjectFile,
                os: TargetOS::Linux,
            });
            let codegen_output = backend
                .generate_from_lir(&output.lir)
                .map_err(|e| format!("Native 代码生成失败: {}", e))?;
            use std::io::Write;
            std::io::stdout()
                .write_all(&codegen_output.files[0].content)
                .map_err(|e| format!("写出目标文件字节失败: {}", e))?;
            Ok(())
        }
        _ => Err(format!(
            "未知 --emit 阶段: {}\n支持的选项: tokens, ast, hir, mir, lir, zig, ts, js, native, obj, typescript, javascript, c, rust, swift, dotnet, csharp, erlang",
            stage
        )),
    }
}

// ── 链接：把直出的可重定位 ELF 目标文件交系统链接器生成可执行文件 ─────────────

/// 给定 Native 架构是否与当前主机架构一致
fn arch_matches_host(arch: x_codegen_native::TargetArch) -> bool {
    use x_codegen_native::TargetArch;
    matches!(
        (arch, std::env::consts::ARCH),
        (TargetArch::X86_64, "x86_64")
            | (TargetArch::AArch64, "aarch64")
            | (TargetArch::RiscV64, "riscv64")
    )
}

/// 交叉链接：用 `zig cc -target <triple>` 把目标文件链接为可执行文件。
/// Zig 自带各平台 libc 与交叉链接器，无需安装目标平台工具链。
fn link_object_zig(
    obj_path: &std::path::Path,
    output_path: &std::path::Path,
    arch: x_codegen_native::TargetArch,
    os: x_codegen_native::TargetOS,
    runtime_src: &std::path::Path,
) -> Result<(), String> {
    let zig = which::which("zig")
        .map_err(|_| "交叉链接需要 zig（未找到，请安装 Zig 0.13+）".to_string())?;
    let triple = zig_cc_triple(arch, os);

    let mut cmd = std::process::Command::new(&zig);
    cmd.arg("cc").arg("-target").arg(&triple);
    // Linux 交叉用静态链接，便于 qemu-user 直接执行
    if os == x_codegen_native::TargetOS::Linux {
        cmd.arg("-static");
    }
    let out = cmd
        .arg("-o")
        .arg(output_path)
        .arg(obj_path)
        .arg(runtime_src)
        .output()
        .map_err(|e| format!("运行 zig cc 失败: {}", e))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "zig cc 交叉链接失败 (target {}):\n{}",
            triple,
            stderr.trim()
        ));
    }
    Ok(())
}

/// Linux x86_64：用 cc/clang/gcc 链接 `.o` 为可执行文件（无需外部汇编器）
fn link_object_linux(
    obj_path: &std::path::Path,
    output_path: &std::path::Path,
    runtime_src: &std::path::Path,
) -> Result<(), String> {
    let linker = which::which("cc")
        .or_else(|_| which::which("clang"))
        .or_else(|_| which::which("gcc"))
        .map_err(|_| "未找到 cc/clang/gcc 链接器".to_string())?;

    let status = std::process::Command::new(&linker)
        .arg("-o")
        .arg(output_path)
        .arg(obj_path)
        .arg(runtime_src)
        .status()
        .map_err(|e| format!("链接失败: {}", e))?;

    if !status.success() {
        return Err("Linux 链接失败".to_string());
    }

    Ok(())
}
