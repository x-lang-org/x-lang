#!/usr/bin/env bash
set -e
# Run all unit tests (requires LLVM for x-codegen; otherwise use:
#   cargo test -p x-lexer -p x-parser -p x-typechecker -p x-hir -p x-perceus -p x-interpreter)
cargo test
echo "Running spec tests..."
cargo run -p x-spec
echo "All tests passed."
