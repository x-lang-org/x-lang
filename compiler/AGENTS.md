# AGENTS.md — compiler/

**Scope**: compiler workspace only. Owns cross-crate pipeline routing, stage ordering, and safety invariants. Child AGENTS files own crate-local detail.

## OVERVIEW

`compiler/` is the main Cargo workspace for the X compiler:
- frontend: `x-lexer`, `x-parser`, `x-typechecker`
- mid-end: `x-hir`, `x-mir`, `x-lir`
- codegen: `x-codegen` + `x-codegen-*`
- runtime/test support: `x-interpreter`, `x-test-integration`

## WORKSPACE MEMBERS

| Cluster | Crates | Notes |
|---------|--------|-------|
| Frontend | `x-lexer`, `x-parser`, `x-typechecker` | source → AST → type environment |
| IR | `x-hir`, `x-mir`, `x-lir` | semantic lowering + Perceus + low-level IR |
| Codegen core | `x-codegen` | shared trait/contracts for backends |
| Backends | `x-codegen-zig`, `-typescript`, `-python`, `-rust`, `-java`, `-csharp`, `-llvm`, `-swift`, `-erlang`, `-asm` | target-specific emission |
| Runtime/tests | `x-interpreter`, `x-test-integration` | `run` path + integration harness |

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Parser / grammar bug | `x-parser/AGENTS.md` |
| Type inference / diagnostics | `x-typechecker/AGENTS.md` |
| Perceus / ownership / reuse | `x-mir/AGENTS.md` |
| Shared backend contract | `x-codegen/AGENTS.md` |
| Native compile/link path | `x-codegen-asm/AGENTS.md` |
| LLVM IR emission | `x-codegen-llvm/AGENTS.md` |
| AST interpreter behavior | `x-interpreter/AGENTS.md` |

## CROSS-CRATE RULES

- Feature work follows pipeline order: spec → lexer → parser → typechecker → HIR → MIR → LIR → backend/interpreter → tests.
- `x-mir` owns memory-safety-critical dup/drop/reuse logic.
- `x-codegen` owns shared backend contracts; backend crates should not redefine those rules.
- `x-test-integration` and `tests/` validate behavior; do not move correctness policy into ad-hoc scripts.
- Use `[workspace.dependencies]` for shared deps and keep inter-crate APIs stable.

## ANTI-PATTERNS

- Do not skip stages when a feature changes semantics.
- Do not patch examples instead of fixing the compiler.
- Do not print debug noise from library crates with `println!`.
- Do not copy root-level Perceus/type-safety policy into every child file; link to the owner.

## COMMANDS

```bash
cd compiler && cargo build
cd compiler && cargo test
cd compiler && cargo test -p x-parser
cd compiler && cargo test -p x-mir
cd compiler && cargo test -p x-typechecker
cd compiler && cargo test -p x-codegen-asm
cd compiler && cargo test -p x-codegen-llvm
```

## NOTES

- Workspace resolver is v2.
- `x-typechecker` and `x-interpreter` now have child AGENTS; use them for local guidance.
- Backend family is broad, but only `x-codegen-asm` and `x-codegen-llvm` currently justify dedicated child AGENTS at this depth.
