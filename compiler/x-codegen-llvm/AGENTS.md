# AGENTS.md — compiler/x-codegen-llvm/

**Scope**: LLVM IR text emission from compiler IR.

## OVERVIEW

`x-codegen-llvm` emits textual `.ll` output without `inkwell`; it writes LLVM IR directly and leaves later object/binary generation to external tools.

## KEY TYPES

| Type | Role |
|------|------|
| `LlvmBackend` | backend implementation |
| `LlvmBackendConfig` | target triple, module naming, backend config |
| `impl x_codegen::CodeGenerator` | shared contract implementation |

## LOCAL RULES

- Shared backend contracts live in `compiler/x-codegen/AGENTS.md`; do not redefine them here.
- Keep this crate focused on LLVM IR emission, not downstream `llc`/`clang` orchestration.
- LIR/AST path handling should stay deterministic and textually inspectable.

## COMMANDS

```bash
cd compiler && cargo test -p x-codegen-llvm
cd tools/x-cli && cargo run -- compile ../../examples/hello.x --emit llvm
```

## HAZARDS

- Large single-file backend; review neighboring cases before editing instruction emission.
- If all backends break after a trait change, start at `x-codegen`, not here.
