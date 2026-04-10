# AGENTS.md

## Critical Constraints

- **DESIGN_GOALS.md is the constitutional document** - all design decisions must align with it
- **examples/ directory is user-maintained** - never modify `.x` or `.zig` files there; if examples fail, fix the compiler instead
- Build system is **Cargo (Rust)**, not Buck2

## Environment Requirements

- **Rust toolchain** (latest stable)
- **Zig 0.13.0+** in PATH - required for native/Wasm compilation via Zig backend

## Essential Commands

```bash
# Run X programs (interpreted)
cd tools/x-cli && cargo run -- run <file.x>

# Type check
cd tools/x-cli && cargo run -- check <file.x>

# Compile to executable (Zig backend - most mature)
cd tools/x-cli && cargo run -- compile <file.x> -o <output>

# Run compiler tests
cd compiler && cargo test

# Run single package tests
cd compiler && cargo test -p x-parser
cd compiler && cargo test -p x-typechecker

# Format code
cargo fmt
```

## Workspace Structure

**Compiler workspace** (`compiler/`):
- `x-lexer` → `x-parser` → `x-typechecker` → `x-hir` → `x-mir` → `x-lir` → `x-codegen-*`
- **MIR includes Perceus** (memory analysis: dup/drop/reuse)
- **Zig backend is most mature** (`x-codegen-zig`)

**CLI** (`tools/x-cli`):
- Orchestrates full pipeline
- Commands: `run`, `check`, `compile`, `test`

**Standard library** (`library/stdlib/`):
- Core types: `types.x` (Option, Result, List, Map)
- IO: `io.x`, `fs.x`, `string.x`
- Prelude auto-imported: `prelude.x`

## IR Pipeline

```
Source → Lexer → Parser → AST → TypeChecker → HIR → MIR (+Perceus) → LIR → Backend
```

- **--emit** flag outputs intermediate results: `tokens`, `ast`, `hir`, `mir`, `lir`, `zig`, `c`, `rust`, `ts`, `js`, `dotnet`

## Testing

- **Unit tests**: `cd compiler && cargo test`
- **Spec tests**: `tests/spec/*.toml` (TOML format, 73 test cases)
- **Integration tests**: `tests/integration/` (X source files)

## Key Files

- Pipeline entrypoint: `tools/x-cli/src/pipeline.rs`
- Type checker: `compiler/x-typechecker/src/lib.rs`
- Parser: `compiler/x-parser/src/parser.rs`
- Perceus analysis: `compiler/x-mir/src/perceus.rs`
- Language spec: `spec/`

## Backend Status

| Backend | Status |
|---------|--------|
| Zig | ✅ Mature (use this) |
| C, Rust, JS/TS, Java, C#, Python, LLVM | 🚧 Early |
| Swift | 📋 Planned |

## Style Notes

- Keywords are full English words: `function`, `mutable`, `integer`, `boolean` (not `fn`, `mut`, `int`, `bool`)
- Use `log` crate for diagnostics, not `println!`
- Run `cargo fmt` before committing
