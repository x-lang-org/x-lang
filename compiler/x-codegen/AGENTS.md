# AGENTS.md — compiler/x-codegen/

**Scope**: shared codegen contracts only. Backend-specific emission rules belong in concrete `x-codegen-*` crates.

## OVERVIEW

This crate defines the common interface between LIR and target backends.
- canonical owner for backend contracts
- shared utilities/state used across backend implementations
- routing point before touching `x-codegen-zig`, `x-codegen-asm`, `x-codegen-llvm`, or other targets

## OWNED CONCEPTS

| Concept | Why it lives here |
|--------|--------------------|
| `CodeGenerator` trait / shared interface | all backends depend on it |
| common output/error structures | contract must stay uniform |
| target-agnostic naming/format helpers | avoids backend drift |

## WHERE TO LOOK

| Task | Action |
|------|--------|
| New backend | define shared expectations here first, then create/update `x-codegen-{lang}` |
| Backend method signature change | update shared trait here before touching leaf backends |
| Output mismatch only in one target | go to that backend crate, not here |

## RULES

- Keep this crate language-agnostic.
- Do not hide target-specific behavior in shared code unless every backend needs it.
- Trait/API changes imply coordinated updates across backend crates and CLI routing.
- LIR is the stable formal input for backends; avoid reintroducing AST-stage assumptions here.

## COMMANDS

```bash
cd compiler && cargo test -p x-codegen
cd compiler && cargo test -p x-codegen-asm
cd compiler && cargo test -p x-codegen-llvm
cd compiler && cargo test -p x-codegen-zig
```

## CHILD ROUTING

- `x-codegen-asm/AGENTS.md` — native / wasm assembly path, multi-arch hazards
- `x-codegen-llvm/AGENTS.md` — LLVM IR text emission
- other `x-codegen-*` crates — follow this contract and local `CLAUDE.md`
