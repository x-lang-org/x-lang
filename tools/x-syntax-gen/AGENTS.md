# AGENTS.md — tools/x-syntax-gen/

**Scope**: syntax/highlight asset generation from lexer token definitions.

## OVERVIEW

`x-syntax-gen` turns compiler token metadata into editor-specific syntax assets.

## KEY FLOW

1. read/build token mapping from `x-lexer`
2. dispatch generator by subcommand in `src/main.rs`
3. write editor-specific outputs via `src/generators/*`

## WHERE TO LOOK

| Task | Location |
|------|----------|
| command registration | `src/main.rs` |
| token/category mapping | `src/token_mapping.rs` |
| editor-specific output | `src/generators/*` |

## LOCAL RULES

- Lexer token/category changes should trigger regeneration here.
- Keep compiler token meaning canonical; do not fork syntax semantics in templates.
- Output format details belong in generator modules, not the shared mapping layer.

## COMMANDS

```bash
cd tools && cargo test -p x-syntax-gen
cd tools/x-syntax-gen && cargo run -- all --output ./output
```

## HAZARDS

- Easy place for drift between editor grammars and actual compiler tokens.
- Generated outputs are secondary; token model correctness is primary.
