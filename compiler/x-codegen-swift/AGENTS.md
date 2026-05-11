# AGENTS.md

Overview: Swift backend - generates Swift source for Apple ecosystem.

KEY TYPES:
- SwiftBackend + SwiftBackendConfig
- impl CodeGenerator for SwiftBackend

DEPENDENCIES:
- Input: x_parser::ast or x_lir::Program

TESTS:
- cd compiler && cargo test -p x-codegen-swift

CONTEXT:
- This agent translates the language IR into Swift source files.
- Keeps Swift naming conventions and module boundaries.

OUTPUT:
- AGENTS.md describes the Swift backend component used by the codegen pipeline.

DESIGN NOTES:
- SwiftBackend implements CodeGenerator trait.
- SwiftBackendConfig provides target OS, Swift version, and module path.
- Input AST/IR to Swift emission is deterministic.

RUNTIME CONSIDERATIONS:
- Ensure imports map to Foundation, Swift standard libraries, and Apple frameworks.
- Prefer explicit access control (public, internal, fileprivate).
- Use robust error handling with Result and custom SwiftError types.

TESTING GUIDANCE:
- Unit tests on code generation patterns; fixture AST inputs.
- cargo test -p x-codegen-swift ensures compilation correctness.

MAPPING STRATEGY:
- Map AST types to Swift types; preserve nullability as Optionals.
- Emit Swift modules with public API surface.
- Generate header headers and license headers in each file.

LICENSE:
- Include Apache-2.0 / MIT as appropriate.

OPERATIONAL LOG:
- This file is created by an automated task (OMO).

<!-- OMO_INTERNAL_INITIATOR -->

CHANGELOG (optional):
- N/A for initial AGENTS.md.

SEE ALSO:
- AGENTS.md for other codegen backends.

HINTS:
- This AGENTS.md is intended for maintainers diving into the Swift backend.

STATUS: Active

Next steps: Prepare sample AST input and generated Swift output.
