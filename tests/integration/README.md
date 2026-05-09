# X Language Integration Tests

This directory documents the `x test integration` flow and the `.x`-source integration cases that belong to it.

It is only one layer of the repository's test story. Broader validation in this repo also uses `python tests/run_tests.py` for the TOML/spec harness and workspace `cargo test` runs for Rust crates.

## Directory Structure

```
tests/integration/
├── runner.rs           # Test runner framework
├── basic/              # Basic language feature tests
│   ├── arithmetic.x
│   ├── comparison.x
│   ├── logic.x
│   ├── variables.x
│   └── control_flow.x
├── types/              # Type system tests
│   ├── type_inference.x
│   ├── generics.x
│   ├── enums.x
│   ├── records.x
│   └── option_result.x
├── functions/          # Function tests
│   ├── basic_functions.x
│   ├── recursion.x
│   ├── higher_order.x
│   └── closures.x
├── patterns/           # Pattern matching tests
│   ├── basic_patterns.x
│   ├── enum_patterns.x
│   └── destructuring.x
└── stdlib/             # Standard library tests
    ├── io_test.x
    ├── string_test.x
    ├── math_test.x
    └── collections_test.x
```

## Running Tests

```bash
# Run the integration-test flow documented in this directory
x test integration

# Run with verbose output
x test integration --verbose

# Run tests from a specific category
x test integration basic
x test integration types
x test integration functions
x test integration patterns
x test integration stdlib
```

## Test File Format

Each test file uses comment annotations to specify expected behavior:

```x
// @test description of test
// @exit_code: 0          # Expected exit code (default: 0)
// @stdout: expected      # Expected stdout output
// @stderr: expected      # Expected stderr output
// @compile_fail          # Test should fail to compile
// @error_contains: msg   # Error message should contain this
// @skip                  # Skip this test
// @skip: reason          # Skip with reason

let x = 42
println(x)
```

## Test Categories

### basic/
Tests for fundamental language features:
- Arithmetic operations (+, -, *, /, %)
- Comparison operators (<, >, <=, >=, ==, !=)
- Logical operations (&&, ||, !)
- Variable declarations and scoping
- Control flow (if/else, while, for)

### types/
Tests for the type system:
- Type inference
- Generic functions and types
- Enum definitions and usage
- Record/class types
- Option and Result types

### functions/
Tests for function features:
- Basic function definitions
- Recursive functions
- Higher-order functions
- Closures and capturing

### patterns/
Tests for pattern matching:
- Basic pattern matching with `when`
- Enum variant patterns
- Destructuring patterns

### stdlib/
Tests for standard library functions:
- IO operations
- String manipulation
- Math functions
- Collection operations

## Adding New Tests

1. Create a new `.x` file in the appropriate category directory
2. Add test annotations at the top of the file
3. Write test code that validates the feature
4. Run `x test integration` to verify

## Test Runner Architecture

The integration test runner used by this flow:
1. Discovers all `.x` files in the integration test directories
2. Parses test annotations from comments
3. Executes each test using the `x run` command
4. Verifies output matches expectations
5. Generates a detailed test report
