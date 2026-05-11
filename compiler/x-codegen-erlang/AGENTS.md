AGENTS.md

OVERVIEW: Erlang backend - generates Erlang/OTP module source. Variables must follow Erlang syntax (uppercase/underscore start). Handles loops via tail-recursive helper functions.

KEY TYPES:
- ErlangBackend + ErlangBackendConfig (module_name)
- ErlangBackend::new: default module_name is "x_module"
- impl CodeGenerator for ErlangBackend: generate_from_ast and generate_from_lir both exist
- Internal state: loop_counter (unique loop helper function names), exports list

DEPENDENCIES:
- Input: x_parser::ast or x_lir::Program

TESTS:
- cargo test -p x-codegen-erlang

ARCHITECTURE NOTES:
- ErlangBackend holds module_name string; exports list; loop_counter
- Exports map to -export([...]). Names derived from processed symbols
- Tail-recursive helpers named loop_0, loop_1, etc.

CONFIGURATION:
- ErlangBackendConfig { module_name: String }
- Default: module_name = "x_module"

BEHAVIOR:
- generate_from_ast(ast) -> emits Erlang module with -module(module_name). -export([...])
- generate_from_lir(lir) -> same, using lowered instructions
- Each loop constructs a tail-recursive helper with a unique name loop_<counter>
- Variables must start uppercase with underscores (Erlang atoms and variables)

STATE:
- loop_counter: u64
- exports: Vec<String>

TESTING GUIDANCE:
- Run tests in compiler crate: cd compiler && cargo test -p x-codegen-erlang
- Ensure codegen.erlang follows Erlang syntax (uppercase vars, atoms lower_snake)

USAGE EXAMPLE:
- let backend = ErlangBackend::new("my_mod");
- let ast = x_parser::Ast; // pseudo
- backend.generate_from_ast(&ast);

LIMITATIONS:
- Requires input AST or LIR; does not run the Erlang compiler itself
- Exports must be declared before -export

OPEN QUESTIONS:
- None explicit; design is minimal per scope

<!-- OMO_INTERNAL_INITIATOR -->
