#!/bin/bash
# 测试所有后端

cd /Users/xiongdi/x-lang/tools/x-cli

echo "========== 测试各后端 =========="

# 简单的测试代码
TEST_FILE="test_simple.x"
echo 'function add(a: integer, b: integer) -> integer = a + b

function main() {
    let result = add(10, 20)
    println(result)
}' > "$TEST_FILE"

BACKENDS=("zig" "c" "rust" "python" "ts" "java" "csharp" "swift" "erlang" "llvm")

for backend in "${BACKENDS[@]}"; do
    echo ""
    echo "--- 测试 $backend 后端 ---"

    OUTPUT_FILE="/tmp/test_output_${backend}"

    if ./target/debug/x compile "$TEST_FILE" -o "$OUTPUT_FILE" --target "$backend" 2>&1; then
        echo "✓ $backend: 编译成功"
    else
        echo "✗ $backend: 编译失败"
    fi
done

rm -f "$TEST_FILE"
echo ""
echo "========== 测试完成 =========="