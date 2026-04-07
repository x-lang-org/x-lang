#!/bin/bash

# Simple 100 test cases for ASM backend - X language syntax

cd /Users/xiongdi/x-lang/tools/x-cli

PASS=0
FAIL=0

run_test() {
    local name=$1
    local code=$2
    local expected_output=$3
    local expected_exit=$4

    echo "$code" > "/tmp/test_${name}.x"

    if cargo run -- compile "/tmp/test_${name}.x" -o "/tmp/test_${name}" 2>&1 | grep -q "编译成功"; then
        actual_output=$("/tmp/test_${name}" 2>&1)
        actual_exit=$?

        if [ "$actual_exit" -eq "$expected_exit" ] && [ "$actual_output" = "$expected_output" ]; then
            echo "✓ $name"
            ((PASS++))
        else
            echo "✗ $name (exit: $actual_exit, expected: $expected_exit, output: '$actual_output', expected: '$expected_output')"
            ((FAIL++))
        fi
    else
        echo "✗ $name (compile failed)"
        ((FAIL++))
    fi
}

echo "Running 100 tests..."
echo "===================="

# 1-20: Simple println with constants (use println directly)
run_test "p_1" "println(1)" "1\n" 0
run_test "p_2" "println(2)" "2\n" 0
run_test "p_3" "println(3)" "3\n" 0
run_test "p_4" "println(4)" "4\n" 0
run_test "p_5" "println(5)" "5\n" 0
run_test "p_10" "println(10)" "10\n" 0
run_test "p_20" "println(20)" "20\n" 0
run_test "p_42" "println(42)" "42\n" 0
run_test "p_100" "println(100)" "100\n" 0
run_test "p_0" "println(0)" "0\n" 0
run_test "p_7" "println(7)" "7\n" 0
run_test "p_8" "println(8)" "8\n" 0
run_test "p_9" "println(9)" "9\n" 0
run_test "p_11" "println(11)" "11\n" 0
run_test "p_12" "println(12)" "12\n" 0
run_test "p_15" "println(15)" "15\n" 0
run_test "p_50" "println(50)" "50\n" 0
run_test "p_99" "println(99)" "99\n" 0
run_test "p_255" "println(255)" "255\n" 0

# 21-40: String literals
run_test "s_1" 'println("a")' "a\n" 0
run_test "s_2" 'println("test")' "test\n" 0
run_test "s_3" 'println("hello")' "hello\n" 0
run_test "s_4" 'println("x")' "x\n" 0
run_test "s_5" 'println("ABC")' "ABC\n" 0
run_test "s_6" 'println("OK")' "OK\n" 0
run_test "s_7" 'println("hi")' "hi\n" 0

# 41-60: Variable operations (X language syntax)
run_test "v_1" "let mutable a = 5
println(a)" "5\n" 0
run_test "v_2" "let mutable a = 10
println(a)" "10\n" 0
run_test "v_3" "let mutable a = 1
let mutable b = 2
println(a)" "1\n" 0
run_test "v_4" "let mutable a = 100
println(a)" "100\n" 0

# 61-80: Return value tests (using return statement)
run_test "r_42" "return 42" "" 42
run_test "r_0" "return 0" "" 0
run_test "r_1" "return 1" "" 1
run_test "r_100" "return 100" "" 100
run_test "r_99" "return 99" "" 99
run_test "r_255" "return 255" "" 255
run_test "r_128" "return 128" "" 128
run_test "r_64" "return 64" "" 64
run_test "r_32" "return 32" "" 32
run_test "r_16" "return 16" "" 16

# 81-100: More expressions
run_test "e_1" "let mutable a = 5
a += 3
println(a)" "8\n" 0
run_test "e_2" "let mutable a = 10
a -= 3
println(a)" "7\n" 0
run_test "e_3" "let mutable a = 6
a *= 7
println(a)" "42\n" 0
run_test "e_4" "let mutable a = 20
a /= 4
println(a)" "5\n" 0
run_test "e_5" "let mutable a = 9
a += 1
println(a)" "10\n" 0
run_test "e_6" "let mutable a = 0
a += 42
println(a)" "42\n" 0
run_test "e_7" "let mutable a = 50
a -= 10
println(a)" "40\n" 0
run_test "e_8" "let mutable a = 3
a *= 3
println(a)" "9\n" 0
run_test "e_9" "let mutable a = 100
a /= 10
println(a)" "10\n" 0
run_test "e_10" "let mutable a = 5
a += 5
println(a)" "10\n" 0

echo "===================="
echo "Passed: $PASS"
echo "Failed: $FAIL"
echo "Total: $((PASS + FAIL))"

if [ $FAIL -gt 0 ]; then
    exit 1
fi