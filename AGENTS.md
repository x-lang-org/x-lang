# PROJECT KNOWLEDGE BASE — X Language

**Generated:** 2026-05-06 | **Commit:** 4c04741 | **Branch:** main

## OVERVIEW

X is a Rust-implemented language toolchain: parser/typechecker/IR pipeline, Perceus memory analysis, multi-backend codegen, CLI/LSP/tools, spec tests, and `.x` library sources.

**Authority order**:
1. `DESIGN_GOALS.md` — constitutional document
2. `SPEC.md` / `spec/` — formal language behavior
3. Workspace / crate AGENTS files — local routing and hazards

## STRUCTURE

```text
x-lang/
├── compiler/   # compiler workspace: frontend, IR, codegen, interpreter
├── tools/      # CLI, LSP, syntax asset generators
├── tests/      # TOML/spec/integration test material + spec runner crate
├── library/    # X standard-library and library sources (.x), not Cargo crates
├── spec/       # formal language specification chapters
├── docs/       # tutorials, research, generated book, references
└── examples/   # user-maintained examples; do not edit
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add/change language syntax | `SPEC.md` → `compiler/x-parser/AGENTS.md` | Spec first, then lexer/parser/typechecker |
| Type error or inference bug | `compiler/x-typechecker/AGENTS.md` | Central type logic is concentrated in one large crate |
| Perceus / dup-drop / reuse bug | `compiler/x-mir/AGENTS.md` | Memory-safety-critical |
| New backend or backend contract change | `compiler/x-codegen/AGENTS.md` | Shared trait first, special backends after |
| Native/asm backend issue | `compiler/x-codegen-asm/AGENTS.md` | Default native compile path |
| LLVM IR emission issue | `compiler/x-codegen-llvm/AGENTS.md` | Emits `.ll` text, external tooling later |
| CLI flag / pipeline / target selection | `tools/x-cli/AGENTS.md` | `main.rs` + `pipeline.rs` + command modules |
| LSP/editor integration | `tools/x-lsp/AGENTS.md` | stdio server, handlers, state |
| Syntax assets/editor grammars | `tools/x-syntax-gen/AGENTS.md` | Regenerate after lexer token changes |
| VS Code extension / editor packaging | `tools/x-lang-vscode/README.md` | Node/VS Code extension, outside Cargo workspaces |
| Test placement / running tests | `tests/AGENTS.md` | Spec tests vs integration vs crate tests |
| Spec wording / chapter ownership | `spec/AGENTS.md` | `DESIGN_GOALS.md` still wins on conflict |
| Stdlib / `.x` libraries | `library/AGENTS.md` | Loaded by compiler import/prelude logic |

## CONVENTIONS

- Two Cargo workspaces, not one root workspace: `compiler/` and `tools/` build separately.
- Both workspaces use `resolver = "2"` and centralize shared crates in `[workspace.dependencies]`.
- `tools/` depends on compiler crates via `../compiler/*` path dependencies.
- `println!` in library Rust code is discouraged; use `log::debug!` / structured diagnostics.
- `src/lib.rs` is the normal public entry for crates; `tools/*/src/main.rs` are binary entry points.
- Prefer parent AGENTS for global rules; child AGENTS should own only local execution detail.

## ANTI-PATTERNS (PROJECT-WIDE)

- Do **not** modify `examples/`; fix compiler/runtime behavior instead.
- Do **not** weaken Perceus invariants; dup/drop/reuse mistakes are compiler bugs.
- Do **not** bypass type checking or describe type safety as optional.
- Do **not** add ad-hoc dependencies outside workspace coordination.
- Do **not** duplicate root policy into every child AGENTS file.

## COMMANDS

```bash
# Build workspaces
cd compiler && cargo build
cd tools && cargo build

# Core tests
cd compiler && cargo test
cd tools && cargo test
python tests/run_tests.py

# Common CLI checks
cd tools/x-cli && cargo run -- run ../../examples/hello.x
cd tools/x-cli && cargo run -- check ../../examples/hello.x
cd tools/x-cli && cargo run -- compile ../../examples/hello.x -o hello

# Format
cargo fmt
```

## NOTES

- `tests/spec_runner` is a standalone crate under `tests/`, not a workspace root.
- `library/` is product code, but its contents are `.x` sources rather than Rust crates.
- `docs/book/`, `docs/research/`, `leetcode/`, `.jj/`, and similar reference/state areas are usually not AGENTS targets.
- Existing specialized AGENTS already own: `compiler/`, `compiler/x-parser/`, `compiler/x-mir/`, `compiler/x-codegen/`, `tools/x-cli/`.
