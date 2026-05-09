# AGENTS.md — tools/x-cli/

**Scope**: user-facing CLI behavior and pipeline orchestration only. Tools-workspace policy belongs in `tools/AGENTS.md`.

## OVERVIEW

`x-cli` is the main binary crate:
- entry point: `src/main.rs`
- pipeline orchestration: `src/pipeline.rs`
- command handlers: `src/commands/*`

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Add global flag / subcommand routing | `src/main.rs` |
| Change pipeline sequencing or shared compile logic | `src/pipeline.rs` |
| Change `run` behavior | `src/commands/run.rs` |
| Change `check` behavior | `src/commands/check.rs` |
| Change `compile` targets/linking | `src/commands/compile.rs` |
| Change `test` command behavior | `src/commands/test_cmd.rs` |

## LOCAL RULES

- CLI should route cleanly into compiler crates; do not re-implement stage logic here.
- `run` and `check` use big-stack wrappers around type checking; preserve that behavior when touching deep-AST paths.
- Backend defaults and native linking details can route into `x-codegen-asm` or target-specific backends.
- Keep user-facing behavior deterministic and error messages actionable.

## COMMANDS

```bash
cd tools/x-cli && cargo build
cd tools && cargo test -p x-cli
cd tools/x-cli && cargo run -- run ../../examples/hello.x
cd tools/x-cli && cargo run -- check ../../examples/hello.x
cd tools/x-cli && cargo run -- compile ../../examples/hello.x -o hello
```

## HAZARDS

- Windows/native link behavior frequently crosses into `src/commands/compile.rs` and `x-codegen-asm`.
- Do not put workspace-wide compiler dependency notes here; keep those in `tools/AGENTS.md`.
