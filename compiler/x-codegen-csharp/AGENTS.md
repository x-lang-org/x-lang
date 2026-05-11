# AGENTS.md
Overview
- This agent implements the CSharpBackend for the compiler x-codegen-csharp.
- It emits C# source targeting the .NET platform.
- It consumes the IR in x_parser::ast or x_lir::Program form.

Key types
- CSharpBackend: central codegen driver
- CSharpBackendConfig: customizable gen options
- impl CodeGenerator for CSharpBackend: trait implementation

Responsibilities
- Transform the typed IR into valid C# code
- Emit namespaces, using statements, class definitions, and members
- Apply naming conventions, manage visibility, and generate boilerplate
- Emit comments and preserve source references when available

Dependencies
- Input: x_parser::ast or x_lir::Program
- Other backends for cross-backend patterns

Interfaces
- Implement CodeGenerator for CSharpBackend
- Methods: generate_code(input, config) -> String
- The GeneratedCode carries file path, content, and optional metadata
- Support per-file generation hooks and partial codegen

Workflow
- Called by the common codegen pipeline after parsing to x_parser::ast or x_lir::Program
- Pass config from compiler options; handle warnings and errors gracefully

Testing
- cd compiler && cargo test -p x-codegen-csharp
- Tests cover: type mapping, namespace generation, class skeletons, and file emission

Configuration
- CSharpBackendConfig fields: target_namespace, root_class_name, include_namespace, generate_comments, formatting_style
- Default values provided by the crate; override via cargo features or CLI
- Hooks for post-generation formatting with dotnet format

Examples
- Map a simple type int to System.Int32; generate a POCO with properties
- Emit a namespace wrapper and a single public class

Quality and style
- Align with DESIGN_GOALS.md for language design and safety
- Use log::debug! for internal tracing; avoid println!

Risks
- Mismatch of IR types to CSharp types
- Namespace collisions in large models
- Incomplete MIR data causing partial code emission

Versioning and compatibility
- Ties to crate version x-codegen-csharp
- Ensure compatibility with downstream backends

Next steps
- Expand tests for complex type graphs
- Add support for generics and collection mappings

See also
- input types: x_parser::ast, x_lir::Program

<!-- OMO_INTERNAL_INITIATOR -->
