# AGENTS.md — compiler/ Workspace

**Updated:** 2026-04-22 | **Workspace:** compiler/ (Rust)

---

**Hub of X language compilation pipeline.** Shared workspace for all frontend, IR stages, and 10 backends.

---

## OVERVIEW

Rust-based X compiler pipeline with **Perceus analysis** and **multi-backend codegen**.
- Frontend: lexer → parser → typechecker
- Mid-end: HIR → MIR (Perceus dup/drop/reuse) → LIR
- Back-end: 10+ language targets (Zig, Python, Rust, TypeScript, Java, C#, LLVM, Swift, Erlang, ASM)

---

## WORKSPACE MEMBERS

| Stage | Crate | Role |
|-------|-------|------|
| Lexer | `x-lexer` | Tokenization |
| Parser | `x-parser` | AST generation (LALRPOP) |
| Typechecker | `x-typechecker` | Type inference & checking |
| HIR | `x-hir` | High-level IR (semantic tree) |
| **MIR** | `x-mir` | **Mid-level IR + Perceus analysis** ⭐ |
| LIR | `x-lir` | Low-level IR (platform-neutral) |
| Codegen Core | `x-codegen` | Multi-backend abstraction trait |
| **Backends** | `x-codegen-{zig,python,rust,java,csharp,llvm,swift,erlang,asm,typescript}` | Target code generators |
| Runtime | `x-interpreter` | AST-based fallback interpreter |
| Testing | `x-test-integration` | Integration test harness |

---

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Fix parser crash | `x-parser/` (LALRPOP grammar) |
| Add new backend | `x-codegen-{lang}/` (copy existing, implement `emit()`) |
| Debug Perceus dup/drop | `x-mir/` (reference counting analysis) |
| Type checking bug | `x-typechecker/` |
| Lexer issue | `x-lexer/` |
| IR lowering problem | `x-hir/`, `x-mir/`, `x-lir/` (in sequence) |
| Integration tests | `x-test-integration/` or `tests/spec_runner/` |

---

## KEY CONVENTIONS

- **Workspace resolver**: v2 (modern Cargo resolution)
- **Shared dependencies**: Centralized in `[workspace.dependencies]`
- **Module layout**: Each crate exposes `src/lib.rs` as public API
- **Logging**: Use `log::debug!()` for internal tracing; **never** `println!()` in library code
- **API boundaries**: Stable inter-crate interfaces; avoid coupling beyond dependencies
- **Error handling**: Use `Result<T, E>` with ergonomic error types (via `thiserror`)
- **Testing**: Unit tests in crates, integration specs in `tests/`
- **Codegen abstraction**: All backends implement the `Emit` trait from `x-codegen`

---

## ANTI-PATTERNS

- ❌ **Don't use `println!()` in library code** → use `log::debug!()` instead
- ❌ **Don't skip Perceus analysis in MIR** → every value must have correct dup/drop marks
- ❌ **Don't modify user examples** → fix compiler if examples fail
- ❌ **Don't bypass workspace deps** → add to `[workspace.dependencies]` for consistency
- ❌ **Don't expose unstable internals** → maintain stable public APIs

---

## COMMANDS

```bash
# Build all compiler crates
cd compiler && cargo build

# Run all tests
cd compiler && cargo test

# Test specific stage
cd compiler && cargo test -p x-parser
cd compiler && cargo test -p x-mir
cd compiler && cargo test -p x-codegen-zig

# Format
cargo fmt
```

---

## PERCEUS (Non-Negotiable Safety)

**Perceus reference counting is the centerpiece of X's memory safety.**

- **dup** = copy a reference (increment ref count)
- **drop** = release a reference (decrement ref count)
- **reuse** = optimize when ref count = 1 (in-place mutation)

**Critical**: Any bug in dup/drop insertion = memory safety violation = **compiler bug**, not user error. All Perceus invariants must be enforced in MIR stage.

---

## NEXT STEPS

1. For **feature implementation**, see root `AGENTS.md` → Feature Implementation Order
2. For **Perceus bugs**, check `x-mir/AGENTS.md`
3. For **backend work**, check `x-codegen/AGENTS.md`
4. For **CLI integration**, see `tools/x-cli/AGENTS.md`
