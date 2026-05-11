# AGENTS.md

Overview: Integration test harness for the compiler. This crate provides test infrastructure for running integration tests across the compilation pipeline.

DEPENDENCIES:
- Uses: all compiler crates (x-lexer, x-parser, x-typechecker, x-hir, x-mir, x-lir, x-codegen)
- Coordinates with: tests/spec_runner/ at project root

TESTS:
- Run: cd compiler && cargo test -p x-test-integration

OUTPUT PATH:
- C:\Users\ixion\Documents\x-lang\compiler\x-test-integration\AGENTS.md

Note: Less detailed since this is a testing utility crate.
<!-- OMO_INTERNAL_INITIATOR -->
