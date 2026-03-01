# X 语言 Benchmarks Game 示例

本目录包含 [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) 的 10 个测试的 X 语言版本，用于**完整流水线**（含 LLVM codegen）的验证与性能对比。

## 十个测试

| 文件 | 对应 Benchmark | 说明 |
|------|----------------|------|
| [nbody.x](nbody.x) | n-body | N 体模拟（递归步进，最小版） |
| [fannkuch_redux.x](fannkuch_redux.x) | fannkuch-redux | 索引/排列翻转计数 |
| [spectral_norm.x](spectral_norm.x) | spectral-norm | 矩阵谱范（1×1 最小版） |
| [mandelbrot.x](mandelbrot.x) | mandelbrot | Mandelbrot 单点迭代 |
| [fasta.x](fasta.x) | fasta | FASTA 重复输出 |
| [knucleotide.x](knucleotide.x) | k-nucleotide | k-nucleotide（占位，无 hash） |
| [revcomp.x](revcomp.x) | reverse-complement | 反向互补（递归长度） |
| [binary_trees.x](binary_trees.x) | binary-trees | 二叉树的节点计数 |
| [pidigits.x](pidigits.x) | pidigits | Pi 位数（占位，无大整数） |
| [regex_redux.x](regex_redux.x) | regex-redux | 正则归约（占位，无 regex） |

部分测试因当前语言缺少数组/哈希/大整数/正则而采用最小实现或占位；随语言特性补齐后可替换为完整实现。

## 环境要求

- **LLVM 21**：完整流水线（编译到 .o / 可执行文件）需要本机安装 LLVM 21，并设置环境变量：

```powershell
# Windows PowerShell
$env:LLVM_SYS_211_PREFIX = "C:\Program Files\LLVM"
```

```bash
# Linux / macOS
export LLVM_SYS_211_PREFIX=/usr
```

## 完整流水线（含 LLVM）

使用 **codegen** 特性走完整流水线：**解析 → 类型检查 → HIR → Perceus → AST → LLVM 代码生成**。  
codegen 将 X 程序的 AST 直接 lowering 为 LLVM IR，生成的目标文件（.o/.obj）包含真实的 `main` 与用户函数，并声明 `printf` 供 `print(整数)` 使用；链接时需与 C 运行时链接（如 `clang nbody.o -o nbody` 或 `gcc nbody.o -o nbody`）。

### 1. 带 codegen 编译工具链

```bash
cargo build --features codegen
```

### 2. 编译单个 benchmark 到目标文件

```bash
cargo run --features codegen -- compile examples/nbody.x -o nbody.o --no-link
```

### 3. 输出 LLVM IR

```bash
cargo run --features codegen -- compile examples/nbody.x --emit llvm-ir
```

### 4. 批量编译 10 个 benchmark（脚本）

在项目根目录执行：

**Windows (PowerShell):**

```powershell
$env:LLVM_SYS_211_PREFIX = "C:\Program Files\LLVM"  # 按实际路径修改
cargo build --features codegen
cd examples; .\build_benchmarks.ps1
```

**Linux / macOS (bash):**

```bash
export LLVM_SYS_211_PREFIX=/usr
cargo build --features codegen
bash examples/build_benchmarks.sh
```

生成的目标文件在 `build/` 下。链接并运行示例：

```bash
clang build/nbody.o -o nbody && ./nbody
# 或 gcc build/nbody.o -o nbody && ./nbody
```

## 仅解释执行（无需 LLVM）

不启用 codegen 时可用解释器运行（不生成机器码）：

```bash
cargo run --no-default-features -- run examples/nbody.x
```

## 参考

- [Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)
- 项目根目录 [CLAUDE.md](../CLAUDE.md)：构建与流水线说明
