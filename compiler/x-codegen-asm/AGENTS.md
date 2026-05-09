# AGENTS.md — compiler/x-codegen-asm/

**Scope**: native / wasm assembly backend and assembly/link integration boundaries.

## OVERVIEW

`x-codegen-asm` is the default native compile path: LIR → assembly text / encoded output, then external assembler/linker handoff when needed.

## KEY MODULES

| Module | Role |
|--------|------|
| `arch` | target constants / ABI helpers |
| `assembly/*` | architecture-specific generation (`x86_64`, `aarch64`, `wasm`, etc.) |
| `assembler` | assembler tool integration |
| `encoding` | machine-code encoding helpers |
| `emitter` | output plumbing |

## KEY TYPES

- `NativeBackend`
- `NativeBackendConfig`
- `TargetArch`, `TargetOS`, `OutputFormat`

## LOCAL RULES

- Keep shared backend contracts in `x-codegen`; keep platform assembly specifics here.
- Multi-arch shared logic changes must be checked across all relevant arch paths.
- CLI platform link behavior lives partly in `tools/x-cli/src/commands/compile.rs`; coordinate, do not duplicate.

## COMMANDS

```bash
cd compiler && cargo test -p x-codegen-asm
cd tools/x-cli && cargo run -- compile ../../examples/hello.x --target native -o hello
```

## HAZARDS

- Easy place for arch divergence: constant pools, field offsets, aggregate init, calling convention details.
- A native compile failure may be codegen, assembler discovery, or CLI linking; inspect both this crate and `x-cli`.
