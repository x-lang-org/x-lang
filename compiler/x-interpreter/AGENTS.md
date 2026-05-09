# AGENTS.md — compiler/x-interpreter/

**Scope**: AST-based execution path used by `x run` and interpreter-oriented runtime behavior.

## OVERVIEW

`x-interpreter` executes `x_parser::ast::Program` directly without MIR/LIR/codegen.

## KEY TYPES

| Type | Role |
|------|------|
| `Interpreter` | runtime entry; `Interpreter::new()` / `run(&mut self, program)` |
| `Value` | runtime value model |
| `InterpreterError` | runtime / undefined symbol / execution errors |

## INTEGRATION

- `tools/x-cli/src/commands/run.rs` constructs the interpreter after type checking.
- Behavior should track language semantics, but backend-only platform specifics do not belong here.

## COMMANDS

```bash
cd compiler && cargo test -p x-interpreter
cd tools/x-cli && cargo run -- run ../../examples/hello.x
```

## HAZARDS

- Do not use interpreter quirks to justify deviations from spec semantics.
- If the bug is in parsing/type rules, fix the upstream crate first.
