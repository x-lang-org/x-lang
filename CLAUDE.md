# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

X language is a modern programming language with natural-language-style keywords (`needs`, `given`, `wait`, `when`/`is`, `can`, `atomic`), mathematical function notation, explicit effect/error types (R·E·A), and Perceus-style memory management (compile-time dup/drop, reuse analysis). It supports functional, declarative, OOP, and procedural paradigms.

**Current phase**: Phase 1 largely done: Lexer, Parser, AST, and tree-walk Interpreter. Type checker, HIR, Perceus, and multiple codegen backends (Zig, LLVM, JavaScript, JVM, .NET) exist as crates with varying degrees of completeness. The Zig backend is the most mature and supports core language features. The canonical language specification is [spec/](spec/) (see [spec/README.md](spec/README.md)); [README.md](README.md) is the project introduction.

## Build System

This project uses **Cargo** (Rust). No Buck2.


### Zig Compiler Dependency

The Zig backend requires Zig 0.13.0 or higher to be installed and available in PATH. Zig is used for both native and Wasm code generation, and includes its own LLVM backend so no separate LLVM installation is required for the Zig backend.

Download Zig from: https://ziglang.org/download/

Verify installation:
```bash
zig version
```

### Common Commands

```bash
# Build CLI
cd tools/x-cli && cargo build
cd tools/x-cli && cargo build --release

# Run a .x file (parse + interpret)
cd tools/x-cli && cargo run -- run <file.x>

# Check syntax and types
cd tools/x-cli && cargo run -- check <file.x>

# Compile: full pipeline; --emit for debugging
cd tools/x-cli && cargo run -- compile <file.x> [-o output] [--emit tokens|ast|hir|pir|zig] [--no-link]
# With Zig backend (most mature): generates Zig code and compiles to executable or Wasm
cd tools/x-cli && cargo run -- compile hello.x -o hello

# Run all compiler unit tests
cd compiler && cargo test

# Run a single test
cd compiler && cargo test -p <crate> <test_name>
# Example: Run parser tests
cd compiler && cargo test -p x-parser parse_function

# Run spec tests
cargo run -p x-spec
# or: ./test.sh (runs both unit and spec tests)

# Run examples
cd tools/x-cli && cargo run -- run ../../examples/hello.x
cd tools/x-cli && cargo run -- run ../../examples/fib.x

# Build and run benchmarks (Zig backend recommended)
cd examples && ./build_benchmarks.sh --backend zig && cd ..
```

### Examples Directory

The `examples/` directory contains:
- **Benchmark programs**: 10 programs from the Computer Language Benchmarks Game (binary_trees, fannkuch_redux, etc.)
- **build_benchmarks.sh/build_benchmarks.ps1**: Scripts to build and run all benchmarks with different backends
- **expected/**: Expected outputs for benchmark programs
- **README.md**: Details about the benchmarks and how to run them

## Architecture

The compiler pipeline (current and target) is:

```mermaid
flowchart LR
    Source[Source] --> Lexer[Lexer]
    Lexer --> Parser[Parser]
    Parser --> AST[AST]
    AST --> TypeChecker[TypeChecker]
    TypeChecker --> HIR[HIR]
    HIR --> Perceus[Perceus]
    Perceus --> Codegen[Codegen]
    Perceus --> Interpreter[Interpreter]
```

| Stage       | Pass         | IR / Output     | Crate Location           |
|------------|--------------|-----------------|-----------------|
| 1          | Lexer        | tokens          | `compiler/x-lexer`         |
| 2          | Parser       | AST             | `compiler/x-parser`        |
| 3          | TypeChecker  | (typed AST/HIR) | `compiler/x-typechecker`   |
| 4          | HIR          | HIR (untyped)   | `compiler/x-hir`           |
| 5          | Perceus      | dup/drop/reuse  | `compiler/x-perceus`       |
| 6          | Codegen      | Multiple backends | `compiler/x-codegen`       |
| (alternate)| Interpreter  | run from AST    | `compiler/x-interpreter`   |
| CLI        | Command line interface | binary | `tools/x-cli` |

### Codegen Backends

| Backend | Status | Description |
|---------|--------|-------------|
| Zig | ✅ Mature | Compiles to Zig, then uses Zig compiler to produce native or Wasm binaries. Most features implemented. |
| JavaScript | 🚧 Early | Compiles to JavaScript for browser/Node.js. |
| JVM | 🚧 Early | Compiles to JVM bytecode. |
| .NET | 🚧 Early | Compiles to .NET CIL. |

**Current reality**: The full pipeline is wired in the CLI:
- **run**: Source → Parse → TypeCheck → Interpreter
- **check**: Source → Parse → TypeCheck
- **compile**: Source → Parse → TypeCheck → HIR → Perceus → Codegen → executable/object file. Use `--emit tokens|ast|hir|pir|zig` to dump intermediate stages.

## Crate Responsibilities

| Crate           | Location | Purpose |
|-----------------|----------|---------|
| x-cli           | `tools/x-cli` | CLI binary (run, compile, check, format, package, repl). Orchestrates pipeline. |
| x-lexer         | `compiler/x-lexer` | Tokenization. Produces token stream from source. |
| x-parser        | `compiler/x-parser` | Parsing. Builds AST (Program, declarations, expressions, types). |
| x-hir           | `compiler/x-hir` | High-level IR (post-parse, pre-typing). Currently a stub. |
| x-typechecker   | `compiler/x-typechecker` | Type checking and semantic analysis. Error types defined; logic mostly stub. |
| x-perceus       | `compiler/x-perceus` | Perceus-style analysis (dup/drop, reuse). Present; integration TBD. |
| x-codegen       | `compiler/x-codegen` | Common codegen infrastructure + Zig backend. XIR (X Intermediate Representation) definition. |
| x-codegen-js    | `compiler/x-codegen-js` | JavaScript backend. |
| x-codegen-jvm   | `compiler/x-codegen-jvm` | JVM backend. |
| x-codegen-dotnet | `compiler/x-codegen-dotnet` | .NET backend. |
| x-interpreter   | `compiler/x-interpreter` | Tree-walk interpreter over AST. Used by `run`. |
| x-stdlib        | `library/x-stdlib` | Standard library definitions. |
| x-spec          | `spec/x-spec` | Specification test runner. TOML cases with optional README section refs. |

## Testing

- **Unit tests**: In each crate under `#[cfg(test)]`. Run with `cd compiler && cargo test`.
- **Spec tests**: In `spec/x-spec`. TOML cases with `source`, `exit_code`, `compile_fail`, `error_contains`, and optional `spec = ["section"]` for traceability to the spec in [spec/](spec/). Run with `cargo run -p x-spec` or a top-level `test.sh` if added.
- **Benchmark tests**: In `examples/`. Run with `build_benchmarks.sh` to test codegen backends against expected outputs.

When adding a language feature, add or update spec tests that reference the relevant README section.

## Path to Industrial-Grade

当前实现是「可用的原型」；要接近工业级编译器，需补齐以下能力（按优先级排序）：

1. **诊断与位置**
   - ✅ **已做**：词法/解析错误带源码位置（`Span`、`file:line:col`、snippet）。见 `x-lexer/span.rs`、`ParseError::SyntaxError { message, span }`、CLI 的 `format_parse_error`。
   - 待做：类型检查错误、运行时错误也带 span；多错误收集与恢复（parser 可尝试继续解析并报告多条错误）。

2. **类型检查**
   - 现状：`x-typechecker::type_check` 为桩，直接返回 `Ok(())`。
   - 待做：按 README 类型系统实现约束检查、函数签名、未定义变量/类型等；错误类型带 span。

3. **语言 Feature Parity**
   - Zig backend supports core features: functions, variables, integers, booleans, if/else, while loops, print
   - Missing: arrays, records/structs, Option/Result, pattern matching, classes/interfaces, effect system, Perceus RC

4. **Performance**
   - 待做：大文件/大 AST 下的内存与耗时；必要时增量解析、LSP 友好接口。

5. **Toolchain**
   - 待做：LSP (hover, jump, diagnostics), formatter implementation, package management and multi-file compilation。

## Modifying the Language / Implementation Steps

When adding or changing language features, follow this order:

1. **Update the specification** in [spec/](spec/) (see [spec/README.md](spec/README.md)) and/or [docs/](docs/) as needed (lexical, types, expressions, functions, etc.).
2. **Update x-lexer** if new tokens or comment syntax are needed.
3. **Update x-parser** for new syntax (grammar and AST nodes).
4. **Update x-hir** if the change introduces new IR constructs.
5. **Update x-typechecker** for type rules and semantic checks.
6. **Update x-codegen or x-interpreter** for code generation or execution behavior. Prioritize the Zig backend for new features.
7. **Add or update spec tests** in `spec/x-spec` with `spec = ["section"]` pointing to README.

## Code Style and Logging

- Use standard Rust style and `cargo fmt`.
- Prefer `log` (or `tracing` if adopted) for compiler diagnostics. Use `log::debug!` for pass-internal details; avoid `println!` in library code so that `RUST_LOG=debug` controls verbosity.
- When adding new passes, consider one high-level log line per pass (e.g. "lexing complete", "typecheck complete") with key metrics.

## Version Control

This project uses **Git**. Issue tracking can stay on GitHub (or existing workflow). No Jujutsu (jj) or bd (beads) requirement.


## Quick Reference

- **Spec**: [spec/](spec/) - 完整的语言规格说明书（[spec/README.md](spec/README.md) 为目录）
- **Run**: `cd tools/x-cli && cargo run -- run <file.x>` - 运行 .x 文件（解析 + 解释执行）
- **Check**: `cd tools/x-cli && cargo run -- check <file.x>` - 检查语法和类型
- **Emit tokens/AST**: `cd tools/x-cli && cargo run -- compile <file.x> --emit tokens` 或 `--emit ast` - 输出中间表示
- **Tests**:
  - 所有单元测试：`cd compiler && cargo test`
  - 规格测试：`cargo run -p x-spec` 或 `./test.sh`
  - 单个测试：`cd compiler && cargo test -p <crate> <test_name>` 例如 `cargo test -p x-parser parse_function`
- **Examples**: 查看 `examples/` 目录下的示例程序，如 `hello.x`、`fib.x` 等
- **Errors**: 解析/语法错误会输出 `file:line:col` 与源码片段。
