# AGENTS.md — compiler/x-codegen-zig/

**Updated:** 2026-04-22 | **Backend:** Zig | **Status:** MOST MATURE

---

## OVERVIEW

Zig backend - generates Zig 0.15 style source code from AST/HIR/LIR, then system `zig` compiles to native or Wasm.

---

## KEY TYPES

| Type | Purpose |
|------|---------|
| `ZigBackend` | Main backend struct |
| `ZigBackendConfig` | output_dir, optimize, debug_info, target |
| `ZigTarget` | Native, Wasm32Wasi, Wasm32Freestanding |

---

## PIPELINE

Input: x_parser::ast (primary) or x_lir::Program
Output: Zig source → system zig build-exe

---

## COMMON TASKS

| Task | Approach |
|------|----------|
| Generate native binary | Default target, `zig build-exe` |
| Generate Wasm | Use `Wasm32Wasi` or `Wasm32Freestanding` target |
| Debug output | Enable debug_info in config |

---

## DEPENDENCIES

- **Input:** x_parser::ast or x_lir::Program
- **Uses:** x_codegen::CodeBuffer
- **Requires:** Zig 0.13+ installed

---

## CONVENTIONS

1. generate_from_ast is main entry
2. ZigFmt for formatting emitted code
3. Error handling via ZigGenError

---

## TESTS

```bash
cd compiler && cargo test -p x-codegen-zig
```

---

## NEXT STEPS

- See x-codegen/ for base trait
- For Wasm: use ZigTarget enum
