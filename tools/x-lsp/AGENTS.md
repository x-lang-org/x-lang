# AGENTS.md — tools/x-lsp/

**Scope**: LSP server internals, editor protocol handling, and compiler-analysis integration for IDE features.

## OVERVIEW

`x-lsp` is a stdio language server built on `lsp-server` / `lsp-types` and compiler crate reuse.

## KEY FILES

| File | Role |
|------|------|
| `src/main.rs` | process entry, logging init, server bootstrap |
| `src/server.rs` | message loop and stdio connection |
| `src/handlers/` | request/notification dispatch |
| `src/analysis/` | compiler-backed analysis |
| `src/state/` | open-doc/version/diagnostic state |
| `src/utils.rs` | helpers |

## LOCAL RULES

- Keep editor-protocol concerns separate from compiler semantics.
- If a feature depends on parser/typechecker behavior, fix the compiler crate first and adapt LSP second.
- Preserve buildability even when some handlers are incomplete or gated with `#[allow(dead_code)]`.

## COMMANDS

```bash
cd tools && cargo test -p x-lsp
cd tools && cargo build -p x-lsp
```

## HAZARDS

- API drift in compiler crates often breaks `analysis.rs` first.
- Avoid inventing editor behavior that diverges from compiler diagnostics.
