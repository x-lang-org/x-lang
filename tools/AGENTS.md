# AGENTS.md — tools/

**Scope**: tools workspace only. Owns shared rules for CLI, LSP, and syntax-generation crates.

## OVERVIEW

`tools/` is a separate Cargo workspace for developer-facing binaries:
- `x-cli` — primary user toolchain entrypoint
- `x-lsp` — stdio language server
- `x-syntax-gen` — editor syntax asset generator

Related but outside this Cargo workspace:
- `x-lang-vscode/` — VS Code extension/package glue around syntax + LSP assets

## WORKSPACE RULES

- Build/test this workspace separately from `compiler/`.
- Compiler crates are imported through `../compiler/*` path dependencies in `tools/Cargo.toml`.
- API changes in compiler crates should be verified against `cd tools && cargo build`.
- Shared guidance lives here; leaf crates own command/module detail.

## WHERE TO LOOK

| Task | Location |
|------|----------|
| CLI workflow / compile/run/check | `x-cli/AGENTS.md` |
| Language-server internals | `x-lsp/AGENTS.md` |
| Syntax/highlight generation | `x-syntax-gen/AGENTS.md` |
| VS Code extension packaging | `x-lang-vscode/README.md` |

## COMMANDS

```bash
cd tools && cargo build
cd tools && cargo test
cd tools && cargo test -p x-cli
cd tools && cargo test -p x-lsp
cd tools && cargo test -p x-syntax-gen
```

## HAZARDS

- Do not duplicate compiler pipeline ownership here; route that to `compiler/` or `x-cli`.
- Do not assume tools are standalone; most functionality depends on compiler crate APIs.
