# AGENTS.md
Overview: Rust backend - generates Rust source code for Rust interop.

## KEY TYPES
- RustBackend: central code generator for Rust interop glue
- RustBackendConfig: configuration holder for targets, crates, and interop options
- Implement CodeGenerator for RustBackend: the trait contract

## INTERFACES
- CodeGenerator trait: defines emit, finalize, and helpers
- RustBackend implements CodeGenerator with plan: translate AST/MIR into Rust code
- Public API surface: new(config), generate(ast_or_program) -> String

## DEPENDENCIES
- Input: x_parser::ast or x_lir::Program
- Access to common generator utilities from compiler/x-codegen
- Optional: templates, codegen helpers, and path resolution

## INPUT VARIANTS
- x_parser::ast: consumed for initial translation to intermediate Rust glue
- x_lir::Program: consumed for final lowering into Rust source

## OUTPUT
- Rust source code strings ready for compilation
- Interop glue for FFI boundaries, extern blocks, and safe wrappers
- Optional module-level attributes and visibility controls

## TESTS
- Command: cd compiler && cargo test -p x-codegen-rust
- Coverage focus: basic glue generation, error paths, and small sample programs

## NOTES
- Keep changes scoped to the Rust backend; avoid cross-crate churn
- Align with existing CodeGenerator conventions in compiler/x-codegen
- Document design decisions here for quick onboarding

## DESIGN CONSTRAINTS
- Do not assume ownership transfer beyond safe Rust FFI rules
- Keep lifetime annotations explicit where needed
- Use zero-cost abstractions for wrappers
- Prefer safe wrappers over raw externs

## DATA STRUCTURES
- RustBackend: { config, target_crate, out_dir }
- RustBackendConfig: { crate_name, bindgen, abi, features }

## OPERATIONS
- translate_ast -> glue blocks
- resolve_types -> type mapping
- emit_code -> final source

## USAGE
- Instantiate with RustBackendConfig
- Call generate(ast_or_program), then output to file via standard Write

<!-- OMO_INTERNAL_INITIATOR -->
