# AGENTS.md — library/

**Scope**: X-language library sources under `library/`. These are product-code `.x` modules, not Rust Cargo crates.

## OVERVIEW

`library/` contains language-side libraries loaded by the compiler/import pipeline:
- `stdlib/` — core standard-library modules and prelude-facing code
- `xweb/` — example/minimal web framework built around Zig stdlib integration

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Prelude / core type modules | `stdlib/` |
| Web/server library sources | `xweb/` |
| How compiler finds/loads stdlib | `tools/x-cli/src/pipeline.rs` and compiler import logic |

## RULES

- Treat these as `.x` source trees, not Rust crates.
- Library semantics should remain aligned with `SPEC.md` and compiler behavior.
- Changes here often require compiler/import/runtime verification, not just file edits.

## NOTES

- `stdlib/README.md` lists modules like `prelude.x`, `types.x`, `collections.x`, `io.x`, `fs.x`, `net.x`.
- `xweb/` uses Zig stdlib concepts directly and is especially coupled to Zig-backend behavior.

## HAZARDS

- Do not assume Cargo commands validate these directories directly.
- `xweb` behavior can expose Zig backend gaps rather than library-only bugs.
