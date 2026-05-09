# X 语言

**X 是一门现代通用编程语言，强调可读性第一、类型安全与内存安全，融合多编程范式，采用 Perceus 编译期内存管理。**

[![Build Status](https://github.com/xx-lang/x-lang/workflows/CI/badge.svg)](https://github.com/xx-lang/x-lang/actions)

---

## 📖 语言简介

X 语言设计的核心信条是**可读性第一**。当可读性与其他目标冲突时，可读性永远胜出。语言广泛汲取了 Python、Rust、Go、Kotlin、Haskell、Koka 等语言的优秀实践，致力于打造一门兼顾安全、性能与开发体验的现代语言。

> **当前状态说明**：X 正在持续完善中。仓库已经具备可运行的编译器/解释器/CLI 基础能力，并在本地开发闭环上持续增强，但远程包生态、部分编译阶段与多后端成熟度仍不均衡，不应将整个项目理解为“已经全面生产就绪”。

核心设计理念：
- **可读胜过简洁**：关键字使用完整英文单词（`function` 而非 `fn`，`integer` 而非 `int`）
- **安全默认**：无 null、无异常、编译期确保内存安全
- **多范式融合**：函数式、面向对象、过程式、声明式按需选择
- **无 GC 无手动管理**：Perceus 编译期引用计数与重用分析，延迟确定，性能优异

[查看完整设计目标 → DESIGN_GOALS.md](DESIGN_GOALS.md)

---

## ✨ 主要特性

| 特性 | 说明 |
|------|------|
| **自然语言风格关键字** | `needs`、`given`、`await`、`when`/`is`、`can`、`atomic`，代码像散文一样可读 |
| **数学函数表示法** | `f(x) = x + 1`，简洁直观，定义函数如同写数学 |
| **Hindley-Milner 类型推断** | 绝大多数类型注解可省略，编译器自动推断 |
| **显式效果系统 (R·E·A)** | 函数的副作用在类型签名中显式声明，副作用可见可控 |
| **Perceus 内存管理** | 编译期插入 `dup`/`drop`，重用分析优化为原地更新，无 GC 无手动管理 |
| **代数数据类型 + 模式匹配** | ADT 表达数据结构，`match` 穷尽匹配确保全覆盖 |
| **多范式支持** | 函数式（纯函数、不可变、管道）、面向对象（类、继承、接口）、过程式（可变、循环）、声明式（查询语法） |
| **零成本抽象** | 高级抽象不牺牲运行时性能 |
| **没有 null** | 通过 `Option` 类型显式表示可空，编译器强制处理 |
| **没有异常** | 通过 `Result` 类型显式表示错误，`?` 运算符轻松传播错误 |
| **多种并发模型** | Go 风格协程、Actor 模型、Async/Await 结构化并发 |
| **C FFI** | 直接调用 C 库函数，访问现有生态 |
| **多后端代码生成** | 一次编写，多处运行，后端可插拔 |

---

## 🚀 快速开始

### 环境要求

- **Rust 工具链**（用于编译 X 编译器）
- **Zig 0.13.0+**（用于 Zig 后端生成本机可执行文件，推荐使用）

### 下载 Zig

```bash
# 下载地址：https://ziglang.org/download/
# 验证安装
zig version
# 应该输出 0.13.0 或更高版本
```

### 克隆并构建

```bash
git clone https://github.com/xx-lang/x-lang.git
cd x-lang

# 构建 CLI 工具
cd tools/x-cli
cargo build
```

### 运行你的第一个 X 程序

```x
// hello.x
function main() -> Unit {
    println("Hello, X!")
}
```

```bash
# 解释执行
cd tools/x-cli
cargo run -- run ../../examples/hello.x
# 输出: Hello, X!
```

### 编译为可执行文件

```bash
# 使用 Zig 后端编译
cd tools/x-cli
cargo run -- compile ../../examples/hello.x -o hello

# 运行生成的可执行文件
./hello
# 输出: Hello, X!
```

### 常用命令

| 命令 | 说明 |
|------|------|
| `cargo run -- run <file.x>` | 解析 → 类型检查 → 解释执行 |
| `cargo run -- check <file.x>` | 只解析和类型检查，输出错误 |
| `cargo run -- compile <file.x> -o output` | 完整编译，生成可执行文件 |
| `cargo run -- compile <file.x> --emit ast` | 输出 AST（用于调试） |
| `cargo run -- compile <file.x> --emit zig` | 输出生成的 Zig 代码 |

支持输出的中间表示：`tokens`、`ast`、`hir`、`mir`、`lir`、`zig`、`c`、`rust`、`ts`、`js`、`dotnet`

---

## 📊 代码生成后端状态

| 后端 | 状态 | 说明 |
|------|------|------|
| **Zig** | ✅ 当前最成熟 | 编译到 Zig 源码，使用 Zig 编译器生成原生/Wasm；在现有后端中验证最充分 |
| **C** | 🚧 早期 | 编译到 C 源码，最大可移植性 |
| **Rust** | 🚧 早期 | 编译到 Rust 源码，Rust 生态互操作 |
| **JavaScript/TypeScript** | 🚧 早期 | 编译到 JS/TS，运行在浏览器/Node.js |
| **JVM** | 🚧 早期 | 编译到 JVM 字节码 |
| **.NET** | 🚧 早期 | 编译到 .NET CIL |
| **Python** | 🚧 早期 | 编译到 Python 源码 |
| **LLVM** | 🚧 早期 | 生成 LLVM IR，用于深度优化 |
| **Swift** | 📋 计划中 | 编译到 Swift 源码，Apple 生态 |
| **Native** | 📋 计划中 | 直接机器码生成，快速编译 |

**Zig 后端是目前验证最充分的后端**，也是当前最适合日常试用的路径；但整个工具链和包生态仍在持续完善中。

---

## 🏗️ 编译器架构

X 采用经典的三段式编译器架构：**前端 → 中端 → 后端**

```
源代码
  ↓ 词法分析 (x-lexer)
词法单元流
  ↓ 语法分析 (x-parser)
AST（抽象语法树）
  ↓ 类型检查 (x-typechecker)
带类型注解的 AST
  ↓ HIR 降阶 (x-hir)
HIR（高层中间表示）
  ↓ MIR 降阶 (x-mir)
MIR（中层中间表示，控制流图）
  ↓ Perceus 内存分析 (x-mir)
dup/drop/reuse 分析完成
  ↓ LIR 降阶 (x-lir)
LIR（低层中间表示 = XIR）
  ↓ 代码生成 (x-codegen*)
目标代码 / 可执行文件
```

### 项目结构

| 目录 | 说明 |
|------|------|
| `compiler/` | 编译器核心 |
| `compiler/x-lexer` | 词法分析 |
| `compiler/x-parser` | 语法分析，构建 AST |
| `compiler/x-typechecker` | 类型检查与语义分析 |
| `compiler/x-hir` | 高层中间表示 (HIR) |
| `compiler/x-mir` | 中层中间表示 (MIR) + Perceus 内存分析 |
| `compiler/x-lir` | 低层中间表示 (LIR/XIR) |
| `compiler/x-codegen` | 通用代码生成基础设施 + 多后端 |
| `compiler/x-codegen-js` | JavaScript 后端 |
| `compiler/x-codegen-jvm` | JVM 字节码后端 |
| `compiler/x-codegen-dotnet` | .NET CIL 后端 |
| `compiler/x-interpreter` | 基于 AST 的解释器 |
| `tools/x-cli` | 命令行工具（`run`、`check`、`compile`） |
| `spec/` | 语言规格说明书 |
| `docs/` | 教程与专题文档 |
| `examples/` | 示例程序 |
| `library/stdlib` | X 语言核心标准库 |

---

## 📚 文档

| 文档 | 说明 |
|------|------|
| [DESIGN_GOALS.md](DESIGN_GOALS.md) | **设计目标与原则**（宪法性文件，所有设计决策的最终依据） |
| [SPEC.md](SPEC.md) | **语言规格说明书**（完整语法与语义定义） |
| [spec/README.md](spec/README.md) | 规格目录 |
| [docs/](docs/) | 教程、附录与专题文档 |
| [CLAUDE.md](CLAUDE.md) | 贡献与开发指南（构建、测试、目录结构） |
| [LICENSES.md](LICENSES.md) | 许可证文本 |

---

## 🧪 测试

```bash
# 运行所有编译器单元测试
cd compiler
cargo test

# 运行单个 crate 的测试
cargo test -p x-parser

# 运行单个测试
cargo test -p x-parser parse_function

# 运行所有测试（包含规范测试）
./test.sh
```

更多测试说明见 [CLAUDE.md](CLAUDE.md)。

---

## 💻 示例代码

### Hello World

```x
// examples/hello.x
function main() -> Unit {
    println("Hello, X!")
}
```

### 斐波那契数列

```x
// examples/fib.x
function fib(n: integer) -> integer {
    when n is
        0 -> 0
        1 -> 1
        else -> fib(n - 1) + fib(n - 2)
}

function main() -> Unit {
    println("fib(10) = ${fib(10)}")
    // 输出: fib(10) = 55
}
```

### 函数式风格

```x
// 管道运算符
result = [1, 2, 3, 4, 5]
    |> filter(x => x % 2 == 0)
    |> map(x => x * 2)
    |> sum
```

### 面向对象风格

```x
class Point {
    x: float
    y: float

    constructor(x: float, y: float) {
        this.x = x
        this.y = y
    }

    method distance() -> float {
        return sqrt(x*x + y*y)
    }
}

let p = new Point(3.0, 4.0)
println(p.distance()) // 5.0
```

更多示例参见 [`examples/`](examples/) 目录。

---

## 🔒 许可证

本项目是多许可开源软件。你可以在以下任一许可证下使用、修改和分发：

- [MIT License](https://opensource.org/licenses/MIT)
- [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [BSD 3-Clause License](https://opensource.org/licenses/BSD-3-Clause)

详见 [LICENSES.md](LICENSES.md)。

---

## 🤝 贡献

欢迎贡献！请阅读 [CLAUDE.md](CLAUDE.md) 了解开发流程和代码风格要求。

---

*X 语言——可读、安全、多范式的现代通用编程语言*
