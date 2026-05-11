# AGENTS.md — compiler/x-codegen-python/

**Updated:** 2026-04-22 | **Backend:** Python

---

## OVERVIEW

Python backend - generates Python source code. Consumes both x_parser::ast and x_lir::Program.

---

## KEY TYPES

| Type | Purpose |
|------|---------|
| `PythonBackend` | Main backend struct |
| `PythonBackendConfig` | Backend configuration |
| `PythonResult<T>` | Result<T, x_codegen::CodeGenError> |

---

## PIPELINE

Input: x_parser::ast or x_lir::Program
Output: Python source text

---

## IMPLEMENTATION

- impl CodeGenerator for PythonBackend
- generate_from_lir method exists

---

## DEPENDENCIES

- **Input:** x_parser::ast or x_lir::Program
- **Error type:** x_codegen::CodeGenError

---

## TESTS

```bash
cd compiler && cargo test -p x-codegen-python
```

---

## NEXT STEPS

- See x-codegen/ for base trait
