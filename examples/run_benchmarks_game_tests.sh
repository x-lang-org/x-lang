#!/usr/bin/env bash
# Benchmarks Game 测试方式：运行各示例并 diff 输出与 expected/*.txt
# 用法：在项目根目录执行 bash examples/run_benchmarks_game_tests.sh
# 可选：CARGO_TARGET_DIR=target_examples_test 避免 target/debug/x 占用

set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLES_DIR="$ROOT/examples"
EXPECTED_DIR="$ROOT/examples/expected"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
X_BIN="$ROOT/$TARGET_DIR/debug/x"

if [[ ! -x "$X_BIN" ]]; then
  echo "构建 x (no-default-features)..."
  (cd "$ROOT" && cargo build --no-default-features)
fi
[[ -x "$X_BIN" ]] || { echo "找不到 $X_BIN"; exit 1; }

TESTS=(nbody fannkuch_redux spectral_norm mandelbrot fasta revcomp binary_trees knucleotide pidigits regex_redux)
FAILED=0
for name in "${TESTS[@]}"; do
  prog="$EXAMPLES_DIR/$name.x"
  exp="$EXPECTED_DIR/$name.txt"
  [[ -f "$prog" ]] || { echo "跳过（不存在）: $prog"; continue; }
  [[ -f "$exp" ]] || { echo "跳过（无预期）: $exp"; continue; }

  actual="$("$X_BIN" run -q "$prog" 2>/dev/null)" || true
  exit=$?
  expected="$(cat "$exp")"
  if [[ $exit -ne 0 ]]; then
    echo "FAIL $name (exit $exit)"
    ((FAILED++))
  elif [[ "$actual" != "$expected" ]]; then
    echo "FAIL $name (output diff)"
    echo "expected: $expected"
    echo "actual:   $actual"
    ((FAILED++))
  else
    echo "ok  $name"
  fi
done
if [[ $FAILED -gt 0 ]]; then
  echo ""
  echo "$FAILED 个失败"
  exit 1
fi
echo ""
echo "全部通过（Benchmarks Game 方式：run + diff）"
