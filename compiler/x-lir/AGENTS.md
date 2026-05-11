OVERVIEW: Low-level IR / XIR - C-like structure, THE UNIFIED INPUT for almost all codegen backends.

KEY TYPES:
- `Program { declarations: Vec<Declaration> }`
- `Declaration`: Function, GlobalVar, Struct, Class, Enum, Import, ExternFunction
- `Function`: name, parameters, return_type, body: Block
- `Statement` / `Expression` / `Type`

KEY FUNCTIONS:
- lower_mir_to_lir (src/lower.rs): MIR → LIR
- peephole_optimize_program / peephole_optimize_function (src/peephole.rs)

DEPENDENCIES:
- Input: MIR from x-mir
- Output: LIR → all codegen backends (x-codegen-zig, x-codegen-python, etc.)

DEBUG: cd tools/x-cli && cargo run -- compile path/to/file.x --emit lir

TESTS: cd compiler && cargo test -p x-lir

OUTPUT PATH: C:\Users\ixion\Documents\x-lang\compiler\x-lir\AGENTS.md
<!-- OMO_INTERNAL_INITIATOR -->
