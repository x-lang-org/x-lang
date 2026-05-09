# AGENTS.md — compiler/x-mir/

**Scope**: MIR lowering plus Perceus ownership analysis. This file owns local memory-safety invariants.

## OVERVIEW

`x-mir` is where high-level IR becomes control/data-flow-aware MIR and receives dup/drop/reuse annotations.

## OWNED RESPONSIBILITIES

1. lower HIR → MIR
2. insert `dup` where shared use requires refcount growth
3. insert `drop` where lifetime ends
4. mark safe `reuse` sites when uniqueness holds
5. preserve deterministic, balanced ownership behavior

## CRITICAL INVARIANTS

- Every reference-counted value must get correct dup/drop handling.
- `reuse` is valid only when uniqueness/refcount conditions hold.
- Incorrect marks are compiler bugs, not user mistakes.
- MIR output must remain deterministic for the same HIR input.

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Dataflow / ownership analysis | local analysis modules such as `src/dataflow.rs` |
| MIR lowering entrypoints | `src/lib.rs` |
| Perceus correctness tests | crate tests + integration/spec coverage |

## COMMANDS

```bash
cd compiler && cargo test -p x-mir
cd compiler && cargo test -p x-mir --lib perceus
```

## HAZARDS

- Never trade correctness for fewer annotations.
- Backend issues caused by missing ownership metadata often start here, not in codegen.
- Do not duplicate generic compiler guidance here; route non-MIR work back to parent/peer AGENTS.
