# AGENTS.md — tests/

**Scope**: repository-level test material and test-routing. Owns where new behavior should be tested.

## OVERVIEW

`tests/` mixes two things:
- large TOML-based language/spec test corpus under category directories
- standalone `spec_runner` crate under `tests/spec_runner`

## WHERE TO PUT TESTS

| Need | Location |
|------|----------|
| Language feature/spec conformance | `tests/` category TOML files |
| Spec-chapter-linked checks | `tests/spec/` |
| Standalone runner code | `tests/spec_runner/` |
| Compiler crate-local unit behavior | corresponding crate under `compiler/` |
| CLI integration/smoke tests | `tools/x-cli/tests` |

## TEST TAXONOMY

- top-level category directories mirror language areas: lexical, types, expressions, statements, functions, effects, modules, patterns, memory, metaprogramming, etc.
- `tests/spec/` uses TOML fields like `source`, `compile_fail`, `stdout`, `spec`, `tags`, `target`.
- `tests/spec_runner` is executable harness code, not just fixtures.

## COMMANDS

```bash
python tests/run_tests.py
python tests/run_tests.py --category types
python tests/run_tests.py --verbose
cd tests/spec_runner && cargo run
```

## RULES

- New tests should reference the relevant spec section when possible.
- Prefer minimal, isolated cases over broad kitchen-sink fixtures.
- Do not confuse `examples/` with test fixtures; examples are user-maintained.

## HAZARDS

- `tests/integration/README.md` documents an `x test integration` flow, while current repo practice also relies on Python/TOML runners and crate tests; keep routing explicit when editing docs or harnesses.
