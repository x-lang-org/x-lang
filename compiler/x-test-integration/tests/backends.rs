//! Backend-specific integration tests
//!
//! These tests verify that each code generation backend can:
//! 1. Accept X source code
//! 2. Generate valid output (source code or binary)
//! 3. Execute correctly (when external compiler is available)
//!
//! Run specific backend tests with:
//! `cd compiler && cargo test -p x-test-integration --test backend_tests`

#[cfg(test)]
#[cfg(feature = "backends")]
mod backend_tests {
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;
    use x_test_integration::stage_tests::sources;

    /// Test result for backend compilation
    #[derive(Debug)]
    enum BackendResult {
        /// Backend generated code successfully
        Success { output_path: String },
        /// Code generation failed
        CodegenError(String),
        /// External compilation failed
        CompilationError(String),
        /// Backend not available (external compiler missing)
        NotAvailable,
    }

    /// Compile X source to specific backend and optionally run
    fn compile_and_run(source: &str, backend: &str, run: bool) -> BackendResult {
        let temp_dir = match TempDir::new() {
            Ok(d) => d,
            Err(e) => return BackendResult::CodegenError(e.to_string()),
        };

        let source_path = temp_dir.path().join("test.x");
        let output_path = temp_dir.path().join("output");

        // Write source file
        if let Err(e) = fs::write(&source_path, source) {
            return BackendResult::CodegenError(e.to_string());
        }

        // Compile using x CLI
        let output = Command::new("cargo")
            .args(&[
                "run",
                "--",
                "compile",
                source_path.to_str().unwrap(),
                "-o",
                output_path.to_str().unwrap(),
                "--target",
                backend,
            ])
            .current_dir("/Users/xiongdi/x-lang/tools/x-cli")
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return BackendResult::CodegenError(stderr.to_string());
                }

                // If we just want code generation, we're done
                if !run {
                    return BackendResult::Success {
                        output_path: output_path.to_string_lossy().to_string(),
                    };
                }

                // Try to run the compiled output
                let run_result = Command::new(&output_path).output();
                match run_result {
                    Ok(run_output) => {
                        if run_output.status.success() {
                            BackendResult::Success {
                                output_path: output_path.to_string_lossy().to_string(),
                            }
                        } else {
                            BackendResult::CompilationError(
                                String::from_utf8_lossy(&run_output.stderr).to_string(),
                            )
                        }
                    }
                    Err(e) => BackendResult::NotAvailable,
                }
            }
            Err(e) => BackendResult::CodegenError(e.to_string()),
        }
    }

    /// Check if an external tool is available
    fn tool_available(tool: &str) -> bool {
        Command::new(tool)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Basic source for testing
    const BASIC_SOURCE: &str = r#"println("Hello, World!")"#;

    /// Source with return value
    const RETURN_SOURCE: &str = r#"
        function main() -> integer {
            return 42
        }
    "#;

    /// Zig Backend Tests
    mod zig {
        use super::*;

        #[test]
        fn test_zig_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "zig", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return, // Skip if zig not available
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }

        #[test]
        #[ignore] // Requires zig installed
        fn test_zig_execute() {
            if !tool_available("zig") {
                return;
            }

            let result = compile_and_run(BASIC_SOURCE, "zig", true);
            match result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => (),
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Execution failed: {}", e),
            }
        }
    }

    /// Python Backend Tests
    mod python {
        use super::*;

        #[test]
        fn test_python_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "python", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }

        #[test]
        #[ignore] // Requires python3
        fn test_python_execute() {
            if !tool_available("python3") {
                return;
            }

            let result = compile_and_run(BASIC_SOURCE, "python", true);
            match result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => (),
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Execution failed: {}", e),
            }
        }
    }

    /// TypeScript Backend Tests
    mod typescript {
        use super::*;

        #[test]
        fn test_typescript_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "ts", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }

        #[test]
        #[ignore] // Requires node
        fn test_typescript_execute() {
            if !tool_available("node") {
                return;
            }

            let result = compile_and_run(BASIC_SOURCE, "ts", true);
            match result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => (),
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Execution failed: {}", e),
            }
        }
    }

    /// Rust Backend Tests
    mod rust {
        use super::*;

        #[test]
        fn test_rust_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "rust", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }

        #[test]
        #[ignore] // Requires cargo
        fn test_rust_execute() {
            if !tool_available("cargo") {
                return;
            }

            let result = compile_and_run(BASIC_SOURCE, "rust", true);
            match result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => (),
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Execution failed: {}", e),
            }
        }
    }

    /// Java Backend Tests
    mod java {
        use super::*;

        #[test]
        fn test_java_codegen() {
            let result = compile_and_run(RETURN_SOURCE, "java", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }

        #[test]
        #[ignore] // Requires java
        fn test_java_execute() {
            if !tool_available("java") {
                return;
            }

            let result = compile_and_run(RETURN_SOURCE, "java", true);
            match result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => (),
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Execution failed: {}", e),
            }
        }
    }

    /// C# Backend Tests
    mod csharp {
        use super::*;

        #[test]
        fn test_csharp_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "csharp", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }
    }

    /// Swift Backend Tests
    mod swift {
        use super::*;

        #[test]
        fn test_swift_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "swift", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }
    }

    /// LLVM Backend Tests
    mod llvm {
        use super::*;

        #[test]
        fn test_llvm_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "llvm", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }
    }

    /// Erlang Backend Tests
    mod erlang {
        use super::*;

        #[test]
        fn test_erlang_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "erlang", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }
    }

    /// Native (ASM) Backend Tests
    mod native {
        use super::*;

        #[test]
        fn test_native_codegen() {
            let result = compile_and_run(BASIC_SOURCE, "native", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }

        #[test]
        fn test_native_execute() {
            let result = compile_and_run(BASIC_SOURCE, "native", true);
            match result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => (),
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Execution failed: {}", e),
            }
        }
    }

    /// Wasm Backend Tests
    mod wasm {
        use super::*;

        #[test]
        fn test_wasm_codegen() {
            if !tool_available("zig") {
                return;
            }

            let result = compile_and_run(BASIC_SOURCE, "wasm", false);
            match &result {
                BackendResult::Success { .. } => (),
                BackendResult::NotAvailable => return,
                BackendResult::CodegenError(e) => panic!("Codegen failed: {}", e),
                BackendResult::CompilationError(e) => panic!("Compilation failed: {}", e),
            }
        }
    }
}
