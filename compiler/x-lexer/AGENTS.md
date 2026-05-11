# AGENTS.md - compiler/x-lexer/
**Updated:** 2026-04-22 | **Stage:** Frontend (Lexer)
---
## OVERVIEW
The x-lexer crate performs hand-written tokenization of source text. It converts a &str into a stream of Tokens that the parser consumes, while handling UTF-8 BOM sequences, string interpolation syntax like ${...}, and multi-line or raw string literals.
 
## KEY TYPES
| Type | Description |
|---|---|
| `Lexer<'a>` | Tokenizer driver that consumes input and produces tokens. It uses a simple state machine. |
| `LexerState` | The lexical states: Normal, String, Interpolation. |
| `Token` | The token variants used by the parser. Categories include keywords, literals, punctuation, Ident, and Eof. |
| `Span` | Byte-offset range for a token; used for diagnostics and error messages. |
| `TokenIterator` | Iterator over tokens produced by the lexer for consumption by the parser. |

## PIPELINE
Input: raw source text as a string slice (&str)
Output: a Token stream delivered to the x-parser

## COMMON TASKS
| Task | Approach |
|---|---|
| Tokenize source into a TokenIterator | Step through input, emitting Tokens with accurate Spans; track LexerState for strings and interpolations. |
| Handle UTF-8 BOM | Detect and skip BOM at start before tokenization. |
| Parse string literals and interpolation | Enter String state, emit String tokens, switch to Interpolation state on `${...}` blocks, re-enter Normal state after closing. |
| Support multi-line and raw strings | Recognize syntax for raw blocks and multi-line literals and emit appropriate tokens. |
| Attach Span to tokens | Record start and end byte offsets for precise diagnostics. |
| Lexical errors | Emit Token with Span context and a helpful diagnostic message. |
| Sync with x-parser and SPEC.md | Ensure token kinds, names, and edge cases match parser expectations and spec definitions. |
| Testing | Run cargo test -p x-lexer from crate root or as defined in repository. Ensure tests cover BOM, strings, interpolation, and errors. |

## DEPENDENCIES
- Input: raw source text as &str
- Output: Token stream to x-parser
- Downstream: compiler/x-parser consumes the Token stream to build the AST

## CONVENTIONS
1) Hand-written scanner (not logos).  
2) New or renamed Token variants must stay in sync with SPEC.md and x-parser expectations.  
3) Error messages must include Span information for diagnostics.  
4) Handle UTF-8 BOM and string interpolation details consistently across crates.  
5) Keep lexer logic self-contained; avoid pulling in external scanning engines.

## COMMANDS
- cd compiler && cargo test -p x-lexer

## NEXT STEPS
- Update SPEC.md and crate CLAUDE.md for lexer features as needed
- Review and align tests in compiler/tests or crate-specific tests
- Consider updating documentation in compiler/CLAUDE.md or workspace docs

<!-- OMO_INTERNAL_INITIATOR -->
