# X 语言文档

## 引言

### 什么是 X 语言

X 是一门**现代的、通用的编程语言**，适用于从底层系统编程到上层应用开发的全栈场景。它的设计融合了多种编程范式的优点，旨在提供一种既安全又高效的开发体验。

X 语言的核心特性包括：

- **可读性第一**：所有关键字使用英文全称（`function`、`mutable`、`match`、`implement`），代码读起来像清晰的英文散文
- **类型安全**：Hindley-Milner 类型推断、代数数据类型、穷尽模式匹配——编译通过即无类型错误
- **无 null、无异常**：用 `Option<T>` 代替 null，用 `Result<T, E>` 代替异常，`?` 运算符传播错误，编译器强制处理所有路径
- **内存安全**：Perceus 编译时引用计数——无 GC 停顿、无手动管理、无生命周期标注，重用分析让函数式代码零分配原地更新
- **多范式**：函数式（纯函数、管道 `|>`、模式匹配）、面向对象（类、继承、trait）、过程式（`let mutable`、循环）、声明式（`where`/`sort by`）

X 不是领域特定语言（DSL），而是能覆盖从底层系统到上层应用的全栈语言，适用于：

- 系统编程（OS、嵌入式、驱动）
- 应用开发（桌面、服务端、CLI 工具）
- 高性能计算（科学计算、数据处理）
- Web 后端与基础设施

### X 语言的设计哲学

X 语言的设计由 17 条核心原则驱动，其中**可读性第一**具有最高优先级——当其他原则与可读性冲突时，可读性胜出。

#### 核心设计原则

1. **通用性**：一门语言，从系统到应用
2. **类型安全**：编译通过 ≈ 无类型错误
3. **内存安全**：Perceus：无 GC、无手动管理、无泄漏
4. **多范式**：FP + OOP + 过程式 + 声明式，按需选择
5. **博采众长**：站在 Python/Rust/Go/Kotlin/Haskell/... 的肩膀上
6. **HM 推断**：少写类型，多靠推断
7. **默认不可变**：`let` 不可变，`let mutable` 可变；值/引用小写/大写；`const` 编译期常量
8. **多种并发**：goroutine + Actor + async/await，按需选择
9. **完整工具链**：`x` CLI 对标 Cargo
10. **效果系统**：副作用在类型中可见（`with`）
11. **C FFI**：与 C 零开销互操作
12. **多后端**：C / LLVM / JS / JVM / .NET，一次编写，多处运行
13. **无异常**：`Option` + `Result` + `?`，错误即值
14. **关键字全称**：英文全称，含义自明，不缩写
15. **可读性第一**：代码应像散文一样可读，最高优先级
16. **不使用奇怪的符号**：只使用常见的、含义直观的符号
17. **AI 友好与预编译补全**：语言与工具链应对机器可读、语义明确

#### 设计准则

- **类型安全**：如果编译通过，程序就不应出现类型错误。
- **内存安全**：安全的内存管理不应以牺牲性能或开发体验为代价。
- **多范式**：为问题选择最合适的范式，而非强迫开发者适应单一范式。
- **无异常**：可能失败的操作必须在返回类型中体现，由编译器保证处理。
- **可读性**：宁可多打几个字符，也不要牺牲可读性。写代码花一次时间，读代码花一百次。
- **符号使用**：符号应「看一眼就懂」；需要解释时就用英文单词代替。

### 快速开始

X 语言提供了简洁的命令行工具，让你可以快速运行、检查和编译 X 语言程序。

#### 运行 .x 文件

```bash
cd tools/x-cli && cargo run -- run <file.x>
```

例如，运行经典的 "Hello, World!" 程序：

```bash
cd tools/x-cli && cargo run -- run ../../examples/hello.x
```

#### 检查语法与类型

```bash
cd tools/x-cli && cargo run -- check <file.x>
```

#### 编译为可执行文件（C 后端）

```bash
cd tools/x-cli && cargo run -- compile <file.x> -o <output>
```

#### 语言特性示例

X 语言支持两种编程风格：

1. **脚本风格**（推荐用于简单程序）：
   ```x
   println("Hello, World!")
   ```

2. **传统风格**（推荐用于复杂程序）：
   ```x
   function main() {
       println("Hello, World!")
   }

   main()
   ```

### 安装与设置

X 语言的开发环境设置非常简单，只需按照以下步骤操作：

#### 1. 克隆仓库

```bash
git clone <repository-url>
cd x-lang
```

#### 2. 构建工具链

X 语言的工具链使用 Rust 开发，因此需要先安装 Rust：

1. 访问 [rust-lang.org](https://www.rust-lang.org/) 下载并安装 Rust
2. 验证安装：
   ```bash
   rustc --version
   cargo --version
   ```

然后构建 X 语言工具链：

```bash
cd tools/x-cli
cargo build
```

#### 3. 运行测试

为了确保安装正确，可以运行测试：

```bash
cd ../../ # 回到项目根目录
cargo test
```

#### 4. 环境变量（可选）

为了方便使用，可以将 `tools/x-cli/target/debug` 添加到系统 PATH 环境变量中，这样就可以直接使用 `x` 命令：

```bash
# Windows PowerShell
$env:PATH += ";C:\path\to\x-lang\tools\x-cli\target\debug"

# Linux/macOS
export PATH="$PATH:/path/to/x-lang/tools/x-cli/target/debug"
```

#### 5. 依赖项

- **C 后端**：需要安装 GCC/Clang/MSVC 等 C 编译器
- **LLVM 后端**：需要安装 LLVM 15+（可选）

详细的依赖项说明和安装指南请参考 [CLAUDE.md](CLAUDE.md) 文件。

### 下一步

现在你已经了解了 X 语言的基本概念和设置方法，可以开始探索以下内容：

- **语言规格**：查看 [spec/README.md](spec/README.md) 了解 X 语言的正式语法与语义定义
- **示例程序**：浏览 [examples/](examples/) 目录中的示例，学习 X 语言的各种特性
- **标准库**：查看 [library/stdlib/README.md](library/stdlib/README.md) 了解 X 语言的标准库功能
- **编译器架构**：探索 [compiler/](compiler/) 目录，了解 X 语言编译器的实现细节

X 语言旨在提供一种既安全又高效的编程体验，希望你能在使用过程中感受到它的设计之美。
