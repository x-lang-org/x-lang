# AGENTS.md — compiler/x-codegen-native/

**Scope**: native direct-machine-code backend and `.o`/link integration boundaries.

## OVERVIEW

`x-codegen-native` is the default native compile path: LIR → machine-code bytes + relocations → relocatable ELF (`ET_REL` `.o`), then handoff to the system linker (`cc`). **No external assembler.** Currently x86_64 Linux only (System V AMD64 ABI).

## KEY MODULES

| Module | Role |
|--------|------|
| `arch` | target constants / ABI helpers, register & instruction definitions |
| `encoding` | `X86_64Encoder`: encode one x86-64 instruction to bytes |
| `machine` | `machine/x86_64.rs` `MachineCodeGen`: LIR → `.text` bytes, label fixups, `.rodata` strings, `.bss` globals, symbol/relocation collection (`machine/mod.rs` shared model) |
| `emitter` | `write_relocatable_elf`: write `MachineObject` as `ET_REL` ELF64 |

## KEY TYPES

- `NativeBackend`, `NativeBackendConfig`
- `MachineCodeGen`, `MachineObject`
- `TargetArch`, `TargetOS`, `OutputFormat`

## LOCAL RULES

- Keep shared backend contracts in `x-codegen`; keep x86_64 machine-code specifics here.
- x86_64 Linux only; other arch/OS return `NativeError::Unimplemented`.
- Labels are function-local: cleared per function, jump fixups resolved at function end (avoids `bb0`-style cross-function collisions). Call fixups resolve globally (internal → rel32, external → `R_X86_64_PLT32`).
- CLI linking lives in `tools/x-cli/src/commands/compile.rs` (`link_object_linux`, `cc` only); coordinate, do not duplicate.

## COMMANDS

```bash
cd compiler && cargo test -p x-codegen-native
cd tools/x-cli && cargo run -- compile ../../examples/hello.x --target native -o hello
```

## HAZARDS

- x86-64 instruction encoding (REX/ModR/M), relocation addends (PC32 = offset − 4, PLT32 = −4), and 16-byte stack alignment before calls are the easy places to get bytes wrong.
- A native compile failure may be codegen, ELF writing, or CLI linking; inspect both this crate and `x-cli`.
- Control flow: LIR currently arrives largely flattened from `x-mir`/`x-lir`; structured `if`/`while`/`for` and `Label`/`Goto` are handled here, but missing branch lowering upstream limits end-to-end behavior.
