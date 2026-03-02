#!/usr/bin/env bash
# Benchmarks Game 测试方式：LLVM 编译成二进制后运行，再 diff 输出与 expected/*.txt
# 依赖：LLVM 21（LLVM_SYS_211_PREFIX）、clang 或 gcc（用于链接）
# 用法：在项目根目录执行 bash examples/run_benchmarks_game_tests_compile.sh

set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLES_DIR="$ROOT/examples"
EXPECTED_DIR="$ROOT/examples/expected"
OUT_DIR="$ROOT/examples/out"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
X_BIN="$ROOT/$TARGET_DIR/debug/x"

if [[ ! -x "$X_BIN" ]]; then
  echo "构建 x (--features codegen --no-default-features)..."
  (cd "$ROOT" && cargo build -p x-cli --features codegen --no-default-features)
fi
[[ -x "$X_BIN" ]] || { echo "请先构建带 codegen 的 x 并确保本机有 LLVM 21 与 clang/gcc"; exit 1; }

mkdir -p "$OUT_DIR"

TESTS=(nbody fannkuch_redux spectral_norm mandelbrot fasta revcomp binary_trees knucleotide pidigits regex_redux)
FAILED=0
for name in "${TESTS[@]}"; do
  prog="$EXAMPLES_DIR/$name.x"
  exp="$EXPECTED_DIR/$name.txt"
  exe="$OUT_DIR/$name"
  [[ -f "$prog" ]] || { echo "跳过（不存在）: $prog"; continue; }
  [[ -f "$exp" ]] || { echo "跳过（无预期）: $exp"; continue; }

  if ! "$X_BIN" compile "$prog" -o "$exe" &>/dev/null; then
    echo "FAIL $name (compile failed)"
    ((FAILED++))
    continue
  fi
  if [[ ! -x "$exe" ]]; then
    echo "FAIL $name (exe not produced, link may have failed)"
    ((FAILED++))
    continue
  fi

  actual="$("$exe" 2>/dev/null)" || true
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
echo "全部通过（LLVM 编译为二进制后 run + diff）"
