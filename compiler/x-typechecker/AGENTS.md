# AGENTS.md — compiler/x-typechecker/

**Scope**: type inference, semantic checks, error shaping, and type-environment APIs.

## OVERVIEW

`x-typechecker` validates `x_parser::ast::Program` and returns either diagnostics/failure or a retained `TypeEnv`.

## KEY FILES

| File | Role |
|------|------|
| `src/lib.rs` | main implementation; `type_check`, `type_check_with_env`, `TypeEnv`, `TypeCheckResult` |
| `src/errors.rs` | `TypeError`, severity, category |
| `src/exhaustiveness.rs` | match exhaustiveness-related logic |
| `src/format.rs` | formatting helpers |

## PUBLIC SEARCH TARGETS

- `type_check(program: &Program) -> Result<(), TypeError>`
- `type_check_with_env(program) -> Result<TypeEnv, TypeError>`
- `TypeCheckResult`

## LOCAL RULES

- Keep type rules aligned with `DESIGN_GOALS.md` and `SPEC.md`; do not invent semantics locally.
- CLI big-stack wrappers live in `tools/x-cli/src/pipeline.rs`; preserve compatibility with them.
- This crate owns semantic/type errors, not parser tokenization or backend lowering.

## COMMANDS

```bash
cd compiler && cargo test -p x-typechecker
```

## HAZARDS

- `src/lib.rs` is very large; prefer surgical edits and verify adjacent logic before changing broad behavior.
- Exhaustiveness, inference, and diagnostics can interact; avoid fixing one path by silently weakening another.
