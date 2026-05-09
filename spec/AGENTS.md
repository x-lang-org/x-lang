# AGENTS.md — spec/

**Scope**: formal language-spec documents and chapter organization.

## OVERVIEW

`spec/` is the formal specification tree for language behavior. It is authoritative for syntax/semantics documentation, but `DESIGN_GOALS.md` still overrides it on design conflicts.

## STRUCTURE

| Area | Purpose |
|------|---------|
| `README.md` | chapter map + authority note |
| `docs/00-philosophy.md` ... `docs/11-metaprogramming.md` | chapter-by-chapter formal spec |
| `docs/04-error-handling.md`, `docs/11-advanced-features.md` | extra canonical chapters beyond the simple numbered spine |

## RULES

- Spec work should stay precise and behavior-oriented.
- If a design tension appears, resolve against `DESIGN_GOALS.md` first.
- Compiler/test changes for language semantics should point back to the relevant spec chapter.

## WHERE TO LOOK

| Task | Location |
|------|----------|
| chapter ownership / map | `README.md` |
| lexical rules | `docs/01-lexical.md` |
| type rules | `docs/02-types.md` |
| memory/Perceus semantics | `docs/10-memory.md` |

## HAZARDS

- Do not let tutorial/docs wording become the canonical source when `spec/` disagrees.
- Do not duplicate the full spec into AGENTS files; route to chapters.
