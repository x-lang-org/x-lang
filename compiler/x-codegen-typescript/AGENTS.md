# AGENTS.md — compiler/x-codegen-typescript/

**Updated:** 2026-04-22 | **Backend:** TypeScript

---

## OVERVIEW

TypeScript backend - generates TypeScript source for browser/Node.js/Deno.

---

## KEY TYPES

| Type | Purpose |
|------|---------|
| `TypeScriptBackend` | Main backend struct |
| `TypeScriptBackendConfig` | Backend configuration |

---

## PIPELINE

Input: x_parser::ast or x_lir::Program
Output: TypeScript source

---

## IMPLEMENTATION

- impl CodeGenerator for TypeScriptBackend
- Supports target runtimes: browser, node, deno

---

## DEPENDENCIES

- **Input:** x_parser::ast or x_lir::Program
- **Output:** .ts files

---

## TESTS

```bash
cd compiler && cargo test -p x-codegen-typescript
```

---

## NEXT STEPS

- See x-codegen/ for base trait
