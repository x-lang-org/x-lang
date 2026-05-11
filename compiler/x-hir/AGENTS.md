<!-- OMO_INTERNAL_INITIATOR -->

# AGENTS.md — X Language: x-hir

Overview
- The High-level IR (HIR) stage for the compiler. This pass provides a typed environment, desugaring, and a bridge to Perceus metadata. It takes an AST and produces a mid-end representation that is friendly to subsequent MIR lowering and Perceus analysis.

Key types
- Hir: core mid-end container
  - module_name: String
  - declarations: Vec<HirDeclaration>
  - statements: Vec<HirStatement>
  - type_env: HirTypeEnv
  - perceus_info: HirPerceusInfo
- HirDeclaration, HirStatement, HirExpression: mid-end equivalents of the AST nodes, shaped for analysis and optimization rather than direct source mapping
- HirTypeEnv: environment mapping identifiers to types for semantic checks
- HirPerceusInfo: Perceus metadata hooks and reuse/dup/drop hints used by subsequent passes

Entry points (functions)
- ast_to_hir: convert an AST (from x-parser) to a Hir
- analyze_semantics: perform semantic analysis, type checks, and Perceus binding
- optimize_hir: run optimization passes over Hir
- constant_fold_hir: perform constant folding on Hir trees
- dead_code_eliminate_hir: remove unreachable/dead code within Hir

Dependences
- Input: AST produced by the x-parser (front-end)
- Output: HIR consumed by the x-mir pass (CFG construction and MIR preparation)
- Some backends still implement generate_from_hir using the HIR as their input

Tests
- Run in repository root:
- cd compiler && cargo test -p x-hir

Output path
- Outputs are surfaced through the compiler's standard pass interfaces and written to the MIR pipeline; see x-mir for CFG lowering details.

Notes
- This AGENTS.md documents the intended responsibilities and interfaces for the x-hir pass. Changes here should align with DESIGN_GOALS.md and the overall compiler pipeline documented in AGENTS.md for other crates.
