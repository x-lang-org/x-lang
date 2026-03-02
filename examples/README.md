# X 语言 Benchmarks Game 示例

本目录包含 [Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) 的 10 个测试的 X 语言版本，**全部已完整实现**。

## 十个测试与实现状态

| 文件 | Benchmark | 说明 | 实现状态 |
|------|-----------|------|----------|
| [nbody.x](nbody.x) | n-body | 5 体 Jovian 行星轨道模拟（symplectic integrator）| **完整** |
| [fannkuch_redux.x](fannkuch_redux.x) | fannkuch-redux | n=7 时全排列翻煎饼计数 | **完整** |
| [spectral_norm.x](spectral_norm.x) | spectral-norm | N=100 矩阵谱范数 A(i,j)=1/((i+j)(i+j+1)/2+i+1) | **完整** |
| [mandelbrot.x](mandelbrot.x) | mandelbrot | 200×200 Mandelbrot 集合 PBM 输出 | **完整** |
| [fasta.x](fasta.x) | fasta | FASTA 格式序列生成（重复 + 随机） | **完整** |
| [revcomp.x](revcomp.x) | reverse-complement | DNA 反向互补（A↔T, C↔G 等） | **完整** |
| [binary_trees.x](binary_trees.x) | binary-trees | 分配/遍历/释放完美二叉树 | **完整** |
| [knucleotide.x](knucleotide.x) | k-nucleotide | k-mer 频率统计（使用 HashMap） | **完整** |
| [pidigits.x](pidigits.x) | pidigits | Pi 位数计算（大整数 spigot 算法） | **完整** |
| [regex_redux.x](regex_redux.x) | regex-redux | DNA 正则模式匹配与 IUB 代码替换 | **完整** |

所有 10 个测试均为**真实算法实现**，非简化版或占位符。

## 语言特性使用

这些 benchmark 使用了 X 语言的以下特性：

| 特性 | 使用示例 |
|------|----------|
| **浮点运算** | nbody, spectral_norm, mandelbrot, fasta |
| **数组（引用语义）** | nbody, fannkuch_redux, spectral_norm, binary_trees |
| **while 循环** | 全部 10 个 |
| **变量赋值** | 全部 10 个 |
| **递归函数** | binary_trees, revcomp |
| **字符串操作** | fasta, revcomp, knucleotide, regex_redux |
| **HashMap** | knucleotide |
| **大整数** | pidigits（内置 `compute_pi_digits`） |
| **正则匹配** | regex_redux（内置 `regex_match_count`） |
| **一元运算符 (-, !)** | nbody, mandelbrot, fannkuch_redux |
| **逻辑运算符 (&&, \|\|)** | mandelbrot, fannkuch_redux |
| **复合赋值 (+=, -=)** | 部分 benchmark |

## 内置函数

解释器提供以下内置函数，支撑 benchmark 运行：

**数组**: `new_array(size, init)`, `len(arr)`, `push(arr, val)`, `pop(arr)`, `swap(arr, i, j)`, `reverse_range(arr, start, end)`, `copy_array(arr)`, `sort_by_value_desc(arr)`

**字符串**: `concat(a, b, ...)`, `char_at(s, i)`, `substring(s, start, end)`, `str_upper(s)`, `str_lower(s)`, `str_contains(s, pat)`, `str_replace(s, from, to)`, `str_split(s, delim)`, `str_trim(s)`, `str_starts_with(s, prefix)`, `str_find(s, pat)`

**数学**: `sqrt(x)`, `abs(x)`, `floor(x)`, `ceil(x)`, `round(x)`, `sin(x)`, `cos(x)`, `pow(base, exp)`

**转换**: `to_string(v)`, `to_int(v)`, `to_float(v)`, `format_float(f, precision)`

**映射**: `new_map()`, `map_set(m, key, val)`, `map_get(m, key)`, `map_contains(m, key)`, `map_keys(m)`

**正则**: `regex_match_count(text, pattern)`, `regex_replace_all(text, pattern, replacement)`

**其他**: `compute_pi_digits(n)`, `type_of(v)`, `print(...)`, `println(...)`, `print_inline(...)`

## 运行方式

### 解释执行（无需 LLVM）

```bash
cargo run -- run examples/nbody.x
cargo run -- run examples/binary_trees.x
# ... 其余同理
```

### 自动化测试（Benchmarks Game 方式：run + diff）

在项目根目录执行：

```powershell
# Windows PowerShell
cd c:\Users\ixion\Documents\x-lang
$X = "target\debug\x.exe"
$tests = @("nbody","fannkuch_redux","spectral_norm","mandelbrot","fasta","revcomp","binary_trees","knucleotide","pidigits","regex_redux")
foreach ($t in $tests) {
  $out = & $X run -q "examples\$t.x" 2>&1 | Out-String
  $exp = Get-Content "examples\expected\$t.txt" -Raw
  if ($out.Trim() -eq $exp.Trim()) { Write-Host "ok  $t" }
  else { Write-Host "FAIL $t" }
}
```

```bash
# Linux / macOS
for t in nbody fannkuch_redux spectral_norm mandelbrot fasta revcomp binary_trees knucleotide pidigits regex_redux; do
  actual=$(cargo run -- run -q examples/$t.x 2>&1)
  expected=$(cat examples/expected/$t.txt)
  if [ "$actual" = "$expected" ]; then echo "ok  $t"; else echo "FAIL $t"; fi
done
```

## 十个算法与参考

| 文件 | 官方要点 | 当前实现说明 |
|------|----------|-------------|
| **nbody.x** | Symplectic integrator，5 个 Jovian 天体，能量守恒 | 5 体浮点模拟，N=1000 步 dt=0.01，输出初始和最终能量 |
| **fannkuch_redux.x** | 对 n! 排列做翻煎饼计数 | n=7，全 5040 种排列，输出 checksum=228 和 Pfannkuchen(7)=16 |
| **spectral_norm.x** | 矩阵 A(i,j)=1/((i+j)(i+j+1)/2+i+1) 的谱范数 | N=100，10 轮幂迭代，输出 1.274219991 |
| **mandelbrot.x** | 200×200 逃逸迭代，输出 PBM | 200×200 网格，50 次迭代上限，输出字节值 |
| **fasta.x** | 生成 FASTA 序列（重复 + 加权随机） | ALU 重复序列 + IUB/Homo sapiens 随机序列，每行 60 字符 |
| **revcomp.x** | DNA 反向互补 | 完整互补映射（A↔T, C↔G, M↔K, R↔Y 等），60 字符分行输出 |
| **binary_trees.x** | 分配/遍历/释放完美二叉树 | 数组表示二叉树，stretch depth 11，long-lived depth 10 |
| **knucleotide.x** | k-mer 频率统计 | HashMap 统计 1-mer/2-mer 频率表 + 指定 k-mer 计数 |
| **pidigits.x** | 大整数 spigot 算法计算 Pi 位数 | N=27 位，每行 10 位，使用内置大整数 |
| **regex_redux.x** | 正则多轮匹配与 IUB 代码替换 | 9 种 DNA 模式计数 + 11 种 IUB 代码展开 |

## 参考

- [Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)
- 项目根目录 [CLAUDE.md](../CLAUDE.md)：构建与流水线说明
