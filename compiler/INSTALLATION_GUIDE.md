# X 编译器最新设计更新安装指南

## 概述

本指南介绍 X 编译器的最新**两阶段后端架构**设计及其安装和配置方法。

### 核心设计更新

X 编译器现已采用统一的**两阶段后端架构**：

**第一阶段（当前）**：十大后端
- LLVM、TypeScript、Java、Python、Erlang、Zig、C#、Swift、Rust、Native
- 所有后端都以 LIR（低级中间表示）为输入
- 翻译为目标语言或字节码，利用已有工具链完成编译

**第二阶段（规划中）**：优化与直接生成
- Java 第二阶段：JVM 字节码直接生成
- C# 第二阶段：.NET IL 直接生成
- Erlang 优化版本：改进 BEAM 字节码生成
- Native 优化版本：改进机器码生成

---

## 前置条件

### 系统要求

- **操作系统**：Linux、macOS 或 Windows（WSL2 推荐）
- **Rust**：1.70.0 或更高版本
- **Cargo**：最新版本

### 安装 Rust

如果还未安装 Rust，运行：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

验证安装：

```bash
rustc --version
cargo --version
```

---

## 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/x-lang/x-lang.git
cd x-lang
```

### 2. 编译编译器

```bash
cd compiler
cargo build --release
```

首次编译需要 5-10 分钟。

### 3. 验证安装

```bash
# 显示编译器帮助信息
cd ../tools/x-cli
cargo run -- --help

# 运行示例程序
cargo run -- run ../../examples/hello.x
```

---

## 项目结构

### 核心目录

```
x-lang/
├── compiler/                    # 编译器核心
│   ├── x-lexer/                # 词法分析
│   ├── x-parser/               # 语法分析
│   ├── x-typechecker/          # 类型检查
│   ├── x-hir/                  # 高级中间表示（HIR）
│   ├── x-mir/                  # 中层中间表示（MIR）+ Perceus
│   ├── x-lir/                  # 低级中间表示（LIR）
│   │
│   ├── x-codegen/              # 【第一阶段】十大后端框架
│   │   ├── src/
│   │   │   ├── typescript_backend.rs   # TypeScript 后端
│   │   │   ├── java_backend.rs         # Java 后端
│   │   │   ├── python_backend.rs       # Python 后端
│   │   │   ├── zig_backend.rs          # Zig 后端 ✅
│   │   │   ├── csharp_backend.rs       # C# 后端
│   │   │   ├── swift_backend.rs        # Swift 后端
│   │   │   ├── rust_backend.rs         # Rust 后端
│   │   │   ├── erlang_backend.rs       # Erlang 后端
│   │   │   └── ...
│   │   └── docs/
│   │       ├── README.md               # 后端总览
│   │       ├── ARCHITECTURE.md         # 第一阶段架构
│   │       ├── backends/
│   │       │   ├── TEMPLATE.md         # 后端实现模板
│   │       │   ├── typescript.md
│   │       │   ├── java.md
│   │       │   └── ...
│   │       └── guides/
│   │           ├── adding_backend.md   # 添加新后端指南
│   │           └── testing.md
│   │
│   ├── x-codegen-llvm/         # 【第一阶段】LLVM 后端（独立）
│   ├── x-codegen-native/       # 【规划中】Native 后端
│   │
│   ├── x-interpreter/          # 解释执行引擎
│   ├── RESTRUCTURE.md           # 重构指南
│   ├── INSTALLATION_GUIDE.md    # 本文件
│   └── Cargo.toml              # 工作空间配置
│
├── tools/
│   └── x-cli/                  # 命令行工具
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── spec/                        # 语言规格
├── docs/                        # 文档和教程
├── examples/                    # 示例代码
├── COMPILER_ARCHITECTURE.md     # 编译器整体架构
├── DESIGN_GOALS.md              # 设计目标
├── README.md                    # 项目 README
└── PLAN.md                      # 项目计划
```

### 新增文档说明

| 文件 | 说明 |
|------|------|
| `RESTRUCTURE.md` | 编译器重构指南（两阶段后端架构详解） |
| `INSTALLATION_GUIDE.md` | 本文件（安装和配置指南） |
| `x-codegen/docs/README.md` | 后端框架总览 |
| `x-codegen/docs/ARCHITECTURE.md` | 第一阶段架构详细说明 |
| `x-codegen/docs/backends/TEMPLATE.md` | 后端实现模板 |
| `x-codegen/docs/guides/adding_backend.md` | 添加新后端的详细指南 |

---

## 十大后端安装和配置

### 按优先级列表

#### 🟢 优先级 1（必需）

##### 1. Zig 后端（✅ 已成熟）

**状态**：完全可用、经过测试、文档完善

**安装 Zig**：

```bash
# macOS
brew install zig

# Linux (Ubuntu)
wget https://ziglang.org/download/latest/zig-linux-x86_64.tar.xz
tar xf zig-linux-x86_64.tar.xz
export PATH=$PATH:$(pwd)/zig-linux-x86_64

# Windows
# 从 https://ziglang.org/download/ 下载并添加到 PATH

# 验证
zig version
```

**使用**：

```bash
# 编译为原生可执行文件（推荐）
cd tools/x-cli
cargo run -- compile ../../examples/hello.x -o hello

# 编译为 WebAssembly
cargo run -- compile ../../examples/hello.x --target wasm -o hello.wasm

# 只生成 Zig 源码
cargo run -- compile ../../examples/hello.x --emit-source -o hello.zig
```

##### 2. TypeScript 后端（🚧 早期）

**状态**：基本功能可用

**安装 TypeScript**：

```bash
# 使用 npm
npm install -g typescript

# 或使用 yarn
yarn global add typescript

# 验证
tsc --version
```

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --target typescript -o hello.js
```

##### 3. Java 后端（🚧 早期）

**状态**：基本功能可用

**安装 Java**：

```bash
# macOS
brew install openjdk

# Linux (Ubuntu)
sudo apt-get install default-jdk

# Windows
# 从 https://www.oracle.com/java/technologies/javase-downloads.html 下载

# 验证
javac -version
java -version
```

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --target java -o hello.jar
```

##### 4. Python 后端（🚧 早期）

**状态**：基本功能可用

**安装 Python**：

```bash
# macOS
brew install python@3.11

# Linux (Ubuntu)
sudo apt-get install python3.11

# Windows
# 从 https://www.python.org/downloads/ 下载

# 验证
python3 --version
```

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --target python -o hello.py
```

##### 5. C# 后端（🚧 早期）

**状态**：基本功能可用

**安装 .NET SDK**：

```bash
# macOS
brew install dotnet

# Linux (Ubuntu)
wget https://dot.net/v1/dotnet-install.sh
chmod +x dotnet-install.sh
./dotnet-install.sh

# Windows
# 从 https://dotnet.microsoft.com/download 下载

# 验证
dotnet --version
```

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --target csharp -o hello.dll
```

##### 6. Rust 后端（🚧 早期）

**状态**：基本功能可用（Rust 已安装）

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --target rust -o hello.rs
```

#### 🟡 优先级 2（可选）

##### 7. LLVM 后端（🚧 早期）

**状态**：支持工业级优化

**安装 LLVM**：

```bash
# macOS
brew install llvm

# Linux (Ubuntu)
sudo apt-get install llvm-14

# 验证
llc --version
```

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --backend llvm -o hello
```

##### 8. Swift 后端（📋 规划中）

**状态**：开发中

**安装 Swift**：

```bash
# macOS（Swift 已包含在 Xcode 中）
xcode-select --install

# Linux
# 从 https://swift.org/download/ 下载

# 验证
swift --version
```

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --target swift -o hello
```

##### 9. Erlang 后端（📋 规划中）

**状态**：开发中

**安装 Erlang/OTP**：

```bash
# macOS
brew install erlang

# Linux (Ubuntu)
sudo apt-get install erlang

# Windows
# 从 https://www.erlang.org/downloads 下载

# 验证
erl -version
```

**使用**：

```bash
cargo run -- compile ../../examples/hello.x --target erlang -o hello.beam
```

#### 🔴 优先级 3（规划中）

##### 10. Native 后端（📋 规划中）

**状态**：设计中，尚未实现

**计划**：直接生成 x86_64 或 ARM64 机器码，无需外部工具链

---

## 完整安装脚本

### 一键安装所有必要工具（推荐）

#### macOS

```bash
#!/bin/bash

# 安装 Homebrew（如果未安装）
if ! command -v brew &> /dev/null; then
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# 安装所有工具
brew install rust zig typescript openjdk python@3.11 dotnet llvm erlang swift

# 验证安装
echo "=== Verification ==="
rustc --version
zig version
tsc --version
javac -version
python3 --version
dotnet --version
llc --version
erl -version
swift --version
```

#### Linux (Ubuntu)

```bash
#!/bin/bash

# 更新包管理器
sudo apt-get update

# 安装必要工具
sudo apt-get install -y \
    build-essential \
    curl \
    git \
    npm \
    default-jdk \
    python3.11 \
    llvm-14

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装 Zig
wget https://ziglang.org/download/latest/zig-linux-x86_64.tar.xz
tar xf zig-linux-x86_64.tar.xz
export PATH=$PATH:$(pwd)/zig-linux-x86_64

# 安装 .NET
wget https://dot.net/v1/dotnet-install.sh
chmod +x dotnet-install.sh
./dotnet-install.sh
export PATH=$PATH:$HOME/.dotnet

# 安装 TypeScript
npm install -g typescript

# 安装 Erlang
sudo apt-get install -y erlang

# 验证安装
echo "=== Verification ==="
rustc --version
zig version
tsc --version
javac -version
python3 --version
dotnet --version
llc --version
erl -version
```

---

## 编译器编译和测试

### 完整编译

```bash
cd x-lang/compiler
cargo build --release
```

编译产物位于：`target/release/`

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定后端的测试
cargo test zig_backend
cargo test typescript_backend
cargo test java_backend

# 运行集成测试
cargo test --test integration
```

### 性能构建

```bash
# 优化编译时间
cargo build -j 8

# 查看编译时间
time cargo build --release
```

---

## CLI 工具配置

### 编译和运行 CLI

```bash
cd tools/x-cli
cargo build --release
```

CLI 产物位于：`target/release/x`（或 `x.exe` on Windows）

### 添加到 PATH（可选）

```bash
# 获取完整路径
BINARY_PATH=$(pwd)/target/release/x

# 添加到 PATH（临时）
export PATH=$PATH:$BINARY_PATH

# 或者创建符号链接（永久）
sudo ln -s $BINARY_PATH /usr/local/bin/x

# 验证
x --version
```

### CLI 命令参考

```bash
# 运行 X 程序
x run program.x

# 检查语法和类型
x check program.x

# 编译为原生可执行文件（使用 Zig 后端）
x compile program.x -o program

# 编译为特定格式
x compile program.x --target typescript -o program.js
x compile program.x --target java -o program.jar
x compile program.x --target python -o program.py
x compile program.x --target rust -o program.rs
x compile program.x --target csharp -o program.dll

# 只生成源码（不编译）
x compile program.x --emit-source -o program.zig

# 使用 LLVM 后端
x compile program.x --backend llvm -o program

# 显示帮助
x --help
x compile --help
```

---

## 文档导航

### 核心架构文档

1. **[COMPILER_ARCHITECTURE.md](../COMPILER_ARCHITECTURE.md)**
   - 编译器整体架构
   - 前端、中端、后端详细说明
   - 编译流水线和 crate 组织

2. **[RESTRUCTURE.md](./RESTRUCTURE.md)**
   - 两阶段后端架构详细说明
   - 十大后端规范和分类
   - 目录结构规划和实现路线

### 后端开发文档

3. **[x-codegen/docs/README.md](./x-codegen/docs/README.md)**
   - 后端框架总览
   - 十大后端一览表
   - 快速开始指南

4. **[x-codegen/docs/ARCHITECTURE.md](./x-codegen/docs/ARCHITECTURE.md)**
   - 第一阶段架构详解
   - 后端分类和特点
   - 编译流程详细描述

5. **[x-codegen/docs/backends/TEMPLATE.md](./x-codegen/docs/backends/TEMPLATE.md)**
   - 后端实现模板
   - 源码翻译型后端实现指南
   - 字节码翻译型后端实现指南
   - 测试和调试技巧

6. **[x-codegen/docs/guides/adding_backend.md](./x-codegen/docs/guides/adding_backend.md)**
   - 添加新后端的完整指南
   - 分步实现说明
   - 最佳实践和常见问题

### 设计和规格文档

7. **[../DESIGN_GOALS.md](../DESIGN_GOALS.md)**
   - X 语言设计目标和原则

8. **[../SPEC.md](../SPEC.md)**
   - X 语言规格说明书

---

## 常见问题

### Q1: 哪个后端最成熟？

**A**: **Zig 后端**（✅ 成熟）是当前唯一完全可用的后端。推荐作为首选。

### Q2: 我应该安装所有后端工具链吗？

**A**: 不需要。根据需求选择性安装：
- **最小化**：只安装 Zig（推荐）
- **完整体验**：安装 P1 优先级的后端
- **全部**：安装所有后端（用于开发测试）

### Q3: 如何贡献新后端？

**A**: 按以下步骤：
1. 阅读 [添加新后端指南](./x-codegen/docs/guides/adding_backend.md)
2. 参考 [后端实现模板](./x-codegen/docs/backends/TEMPLATE.md)
3. 查看 [Zig 后端实现](./x-codegen/src/zig_backend.rs) 作为参考
4. 编写代码、测试和文档
5. 提交 PR

### Q4: 编译失败怎么办？

**A**: 尝试以下步骤：
1. 确保 Rust 是最新版本：`rustup update`
2. 清理构建缓存：`cargo clean`
3. 重新编译：`cargo build --release`
4. 检查依赖版本：`cargo tree`
5. 查看错误信息并搜索相关 Issue

### Q5: 如何报告 Bug？

**A**: 
1. 检查是否已有相关 Issue
2. 提供完整的错误信息和重现步骤
3. 包含操作系统和工具链版本信息
4. 如可能，提供最小化的重现案例

### Q6: 如何优化编译速度？

**A**:
- 使用 `--release` 模式进行发布构建
- 使用多线程编译：`cargo build -j 8`
- 利用增量编译：只修改必要代码
- 考虑使用 mold 链接器（Linux）

### Q7: 第二阶段何时开始？

**A**: 在所有十大后端都达到"基础可用"状态后，预计 Q3 2024 开始第二阶段优化和直接生成工作。

---

## 环境变量配置

### 可选的环境变量

```bash
# 设置日志级别（调试用）
export RUST_LOG=debug

# 设置并行编译线程数
export CARGO_BUILD_JOBS=8

# 启用编译时间分析
export CARGO_VERBOSE=1

# 禁用编译缓存（用于调试）
export CARGO_TARGET_DIR=/tmp/x-build
```

### 推荐的 ~/.bashrc 或 ~/.zshrc 配置

```bash
# X 编译器配置
export RUST_LOG=info
export PATH=$PATH:$HOME/.cargo/bin

# 如果自定义了二进制路径
# export PATH=$PATH:/path/to/x-lang/tools/x-cli/target/release

# 别名（可选）
alias x='cargo run --release --manifest-path ~/x-lang/tools/x-cli/Cargo.toml --'
alias x-check='cargo check -p x-codegen'
alias x-test='cargo test -p x-codegen'
```

---

## 相关资源

### 官方链接

- **X 语言网站**：[x-lang.org](https://x-lang.org)
- **GitHub 仓库**：[github.com/x-lang/x-lang](https://github.com/x-lang/x-lang)
- **Issue 跟踪**：[GitHub Issues](https://github.com/x-lang/x-lang/issues)
- **讨论区**：[GitHub Discussions](https://github.com/x-lang/x-lang/discussions)

### 工具链文档

- **Rust 官方**：https://www.rust-lang.org/
- **Zig 官方**：https://ziglang.org/
- **TypeScript 官方**：https://www.typescriptlang.org/
- **Java 官方**：https://docs.oracle.com/en/java/
- **Python 官方**：https://docs.python.org/
- **C#/.NET 官方**：https://dotnet.microsoft.com/
- **Swift 官方**：https://swift.org/
- **Erlang 官方**：https://www.erlang.org/
- **LLVM 官方**：https://llvm.org/

---

## 故障排查

### 编译错误

#### `error: failed to resolve: use of undeclared crate`

**解决方案**：
```bash
# 确保所有依赖都在 Cargo.toml 中
cargo update
cargo build --release
```

#### `error: linking with 'cc' failed`

**解决方案**：
```bash
# 安装 C 工具链
# macOS
xcode-select --install

# Linux (Ubuntu)
sudo apt-get install build-essential

# Windows
# 使用 Visual Studio Build Tools 或 MSVC
```

### 运行时错误

#### `toolchain not found`

**解决方案**：
```bash
# 检查工具是否安装
zig version
tsc --version
javac -version

# 将工具添加到 PATH
export PATH=$PATH:/path/to/tool/bin
```

#### 生成的代码语法错误

**解决方案**：
```bash
# 使用 --emit-source 查看生成的代码
x compile program.x --emit-source -o output

# 手工检查输出文件
cat output

# 在目标工具链中验证
tsc output.ts
```

### 性能问题

#### 编译过慢

**解决方案**：
```bash
# 使用发布模式
cargo build --release

# 使用更多线程
cargo build -j 16

# 检查什么花费时间
cargo build -j 1 --verbose

# 使用 cargo-build-time 分析
cargo install cargo-build-time
cargo build-time --release
```

---

## 下一步

### 初级用户

1. ✅ 完成本指南的安装步骤
2. ✅ 运行示例程序：`x run examples/hello.x`
3. ✅ 编译示例为原生代码：`x compile examples/hello.x -o hello`
4. ✅ 阅读 [README.md](../README.md) 了解语言基础

### 中级用户

1. ✅ 阅读 [COMPILER_ARCHITECTURE.md](../COMPILER_ARCHITECTURE.md) 理解编译器设计
2. ✅ 探索 [x-codegen/docs/](./x-codegen/docs/) 了解后端框架
3. ✅ 尝试不同的后端：`x compile --target typescript`
4. ✅ 编写自己的 X 程序

### 高级用户

1. ✅ 阅读 [添加新后端指南](./x-codegen/docs/guides/adding_backend.md)
2. ✅ 研究现有后端实现（特别是 Zig 后端）
3. ✅ 为项目贡献代码：实现缺失的后端或优化现有实现
4. ✅ 参与第二阶段优化工作

---

## 贡献指南

感谢对 X 语言的兴趣！如果你想贡献代码：

1. **Fork 仓库** - 在 GitHub 上 fork 项目
2. **创建分支** - 为你的功能创建新分支
3. **编写代码** - 遵循项目风格指南
4. **编写测试** - 为新功能添加测试
5. **提交 PR** - 用清晰的描述提交 Pull Request

更多详情见 [CLAUDE.md](../CLAUDE.md)。

---

## 许可证

X 语言采用多重许可，你可任选其一：

- MIT License
- Apache License 2.0
- BSD 3-Clause License

详见 [LICENSE 文件](../LICENSES.md)。

---

## 支持和反馈

- **问题**：提交 [GitHub Issue](https://github.com/x-lang/x-lang/issues)
- **讨论**：参加 [GitHub Discussions](https://github.com/x-lang/x-lang/discussions)
- **贡献**：提交 [Pull Request](https://github.com/x-lang/x-lang/pulls)
- **反馈**：在 Issue 中分享想法和建议

---

**最后更新**：2024 年 1 月  
**维护者**：X 语言核心团队

---

## 版本历史

### v0.4.0（2024 年 1 月 - 当前）

- ✅ 发布两阶段后端架构设计
- ✅ 整理十大后端文档
- ✅ 发布 RESTRUCTURE.md 和 INSTALLATION_GUIDE.md
- ✅ 发布后端实现模板和开发指南
- ✅ Zig 后端达到成熟状态

### v0.3.0（2023 年）

- 基本的代码生成功能
- 早期的后端支持（TypeScript、Java、Zig 等）

### 更早版本

详见 [GitHub Release](https://github.com/x-lang/x-lang/releases)