# CLAUDE.md - simple_x_cli

## 功能定位

simple_x_cli 是 X 语言的轻量级命令行工具，提供基本的代码运行功能。它是 X 语言工具链的简化版本，专注于快速执行 X 语言代码的核心功能。

### 与其他组件的关系

```
┌─────────────────────────────────────────────────────────────────────┐
│ simple_x_cli                                                       │
├─────────────────────────────────────────────────────────────────────┤
│ 依赖组件                                                           │
├─────────────────────────────────────────────────────────────────────┤
│ • x-lexer       : 词法分析器，将源代码转换为 token 流              │
│ • x-parser      : 语法分析器，将 token 流构建为 AST（抽象语法树）  │
│ • x-interpreter : 树遍历解释器，直接执行 AST                        │
└─────────────────────────────────────────────────────────────────────┘
```

## 核心功能

1. **文件读取**：读取 .x 源代码文件
2. **词法分析**：使用 x-lexer 将源代码转换为 token 流
3. **语法分析**：使用 x-parser 将 token 流解析为 AST
4. **解释执行**：使用 x-interpreter 直接执行 AST
5. **错误处理**：提供基本的错误报告功能

## 构建与运行

### 构建

```bash
cd /C/Users/Administrator/Documents/x-lang/simple_x_cli
cargo build
```

### 运行

```bash
cargo run -- run <file.x>
# 或者直接运行可执行文件
./target/debug/simple_x_cli run <file.x>
```

### 示例

```bash
cd /C/Users/Administrator/Documents/x-lang/simple_x_cli
cargo run -- run ../test_basic.x
```

## 项目结构

```
simple_x_cli/
├── Cargo.toml          # 项目配置和依赖声明
└── src/
    └── main.rs         # 主入口文件
```

## 源代码说明

### main.rs

main.rs 是 simple_x_cli 的唯一源代码文件，包含以下核心功能：

1. **命令行参数解析**：验证输入参数格式
2. **文件读取**：读取 X 语言源代码文件
3. **解析流程**：调用 x-parser 解析源代码为 AST
4. **解释执行**：调用 x-interpreter 执行 AST
5. **错误处理**：处理文件读取、解析和执行过程中的错误

## 当前状态与限制

### 已实现功能

- ✅ 基本的命令行参数解析
- ✅ 文件读取功能
- ✅ 与 x-lexer、x-parser 和 x-interpreter 的集成
- ✅ 基本的错误报告机制
- ✅ 成功/失败状态显示

### 限制

- 只支持 `run` 命令，没有其他 CLI 功能
- 错误报告较为简单，缺少详细的位置信息
- 不支持类型检查、代码生成等高级功能
- 没有格式化输出或调试支持

## 开发流程

如需修改 simple_x_cli，可直接编辑 `src/main.rs` 文件，然后重新构建。

### 常用命令

```bash
# 构建并运行
cargo run -- run ../test_basic.x

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 检查代码
cargo clippy
```
