# AGENTS.md — compiler/x-parser/

**Scope**: grammar, AST construction, parser-local debugging. Do not restate full compiler pipeline here.

## OVERVIEW

`x-parser` transforms lexer tokens into the AST rooted at `Program`.
- parser implementation: `src/parser.rs`
- AST types: `src/ast.rs`
- public entry points: `src/lib.rs`

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Add/change parse logic | `src/parser.rs` |
| Add/change AST node | `src/ast.rs` |
| Parse entrypoints/errors | `src/lib.rs` |

## LOCAL RULES

- Spec changes come first; parser work should mirror `SPEC.md`, not invent syntax locally.
- Keep invalid input as structured parse errors, not panics.
- Grammar changes usually pair with AST updates and parser tests.
- If the issue is tokenization, route up to `x-lexer`; if semantics/type rules, route to `x-typechecker`.

## COMMANDS

```bash
cd compiler && cargo test -p x-parser
cd compiler && cargo test -p x-parser -- --nocapture
```

## HAZARDS

- Large parser edits can destabilize precedence or recovery behavior.
- Parser fixes that only patch examples are wrong; fix grammar/AST behavior instead.
