#!/usr/bin/env bash
# 使用完整流水线（LLVM codegen）编译 examples/ 下 10 个 Benchmarks Game 测试
# 需先安装 LLVM 21 并设置 LLVM_SYS_211_PREFIX

set -e
cd "$(dirname "$0")/.."
export LLVM_SYS_211_PREFIX="${LLVM_SYS_211_PREFIX:-/usr}"

echo "Building x with codegen..."
cargo build --features codegen

mkdir -p build
BENCHMARKS=(nbody fannkuch_redux spectral_norm mandelbrot fasta knucleotide revcomp binary_trees pidigits regex_redux)

for name in "${BENCHMARKS[@]}"; do
  echo "Compiling examples/${name}.x -> build/${name}.o"
  cargo run --features codegen -- compile "examples/${name}.x" -o "build/${name}.o" --no-link
done

echo "Done. Object files in build/"
