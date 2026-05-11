# AGENTS.md

Overview
- Java backend that generates Java 25 LTS style source code.
- Consumes compiler IRs and emits idiomatic Java classes.
- Integrated into the codegen pipeline as an optional backend.

KEY TYPES
- JavaBackend: orchestrator for codegen tasks.
- JavaConfig: configuration for codegen; fields include class_name (default "Main"), package_name, and options.
- generate_from_lir: entry point that translates from x_lir::Program to Java source.
- matches ast: supports x_parser::ast input as alternative.
- JavaResult<T>: a type alias for Result<T, x_codegen::CodeGenError>.

INTERFACES
- pub fn new(config: JavaConfig) -> Self
- pub fn generate_from_ast(ast: &x_parser::Ast) -> JavaResult<String>
- pub fn generate_from_lir(program: &x_lir::Program) -> JavaResult<String>

DEPENDENCIES
- Input: x_parser::ast or x_lir::Program
- Output: a single Java file or a set of files under output_dir, depending on package layout.
- Requires x_codegen crate for CodeGenError type.

TESTS
- cargo test -p x-codegen-java
- cargo test -p x-codegen-java --features generate_from_lir

PIPELINE NOTES
- The backend should emit Java 17+ code compatible with Java 25 LTS style.
- Use standard Java naming conventions: CamelCase types, lowerCamelCase methods.
- When generating, prefer explicit visibility, final classes, and Javadoc where helpful.
- The backend should gracefully report CodeGenError on failures.

ERROR HANDLING
- Map internal errors to x_codegen::CodeGenError variants.
- Include context in the error messages for easier debugging.

TEST DATA
- Provide small AST and small LIR samples under tests/resources.
- Validate that output compiles in a separate step.

MAINTENANCE
- Align with DESIGN_GOALS.md for language constraints.
- Do not modify examples/ in this repository.

CROSS-REFERENCES
- See compiler/AGENTS.md for pipeline coordination.
- See compiler/x-parser/CLAUDE.md for AST structure.

<!-- OMO_INTERNAL_INITIATOR -->
