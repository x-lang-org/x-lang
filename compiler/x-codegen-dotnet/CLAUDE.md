# x-codegen-dotnet

## 项目概述

x-codegen-dotnet 是 X 语言编译器的 .NET 后端，负责将 X 语言程序编译为 .NET CIL（公共中间语言）代码。它实现了 x-codegen 中定义的 CodeGenerator trait，提供了完整的代码生成功能。

## 功能定位

- **语言后端**：为 X 语言提供 .NET 平台支持
- **代码生成**：将 X 语言程序（通过 AST、HIR 或 PIR）转换为 CIL 代码
- **编译目标**：支持生成 .NET 程序集（DLL）或可执行文件（EXE）
- **集成点**：与 X 语言编译器流水线无缝集成

## 架构与依赖

```
x-codegen-dotnet/
├── Cargo.toml       # 项目元数据和依赖
└── src/
    └── lib.rs       # 核心实现
```

### 核心依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| x-parser | 工作区 | 提供 AST 结构 |
| x-codegen | 工作区 | 代码生成基础设施 |
| thiserror | 工作区 | 错误类型定义 |
| log | 工作区 | 日志功能 |

### 主要组件

1. **DotNetConfig**：.NET 代码生成器配置
2. **DotNetCodeGenerator**：实现 CodeGenerator trait 的 .NET 代码生成器
3. **DotNetCodeGenError**：代码生成过程中的错误类型

## 实现状态

**当前状态**：早期阶段（桩实现）

- 已实现代码生成器框架和基本错误处理
- 所有生成方法均返回未实现错误
- 需要实现完整的 CIL 代码生成逻辑

## 使用方法

### 作为库使用

```rust
use x_codegen_dotnet::{DotNetCodeGenerator, DotNetConfig};
use x_parser::Parser;

// 解析 X 语言代码
let parser = Parser::new();
let program = parser.parse("fn main() { print(\"Hello\") }").unwrap();

// 创建 .NET 代码生成器
let config = DotNetConfig {
    output_dir: Some("output".into()),
    optimize: false,
    debug_info: true,
};

let mut generator = DotNetCodeGenerator::new(config);

// 生成代码（当前返回未实现错误）
let result = generator.generate_from_ast(&program);
```

### 通过编译器 CLI 使用

```bash
cd tools/x-cli && cargo run -- compile --target dotnet <file.x> -o output
```

## 代码风格与规范

- 使用标准 Rust 风格，执行 `cargo fmt` 格式化
- 错误处理遵循 `thiserror` 库规范
- 使用 `log` 库进行日志记录
- 文档字符串使用中文，符合项目整体风格

## 相关资源

- **主项目文档**：[../..//CLAUDE.md](../../CLAUDE.md)
- **代码生成基础设施**：[../x-codegen/CLAUDE.md](../x-codegen/CLAUDE.md)
- **编译器架构**：[../../ARCHITECTURE.md](../../ARCHITECTURE.md)

## Testing & Verification

### 最小验证（只验证本 crate）

```bash
cd compiler
cargo test -p x-codegen-dotnet
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

```bash
cd compiler
cargo llvm-cov -p x-codegen-dotnet --tests --lcov --output-path target/coverage/x-codegen-dotnet.lcov
```

### 集成验证（通过 CLI 路径）

```bash
cd tools/x-cli
cargo test -p x-cli
```
