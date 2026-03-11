# x-interpreter

## 项目概述

x-interpreter 是 X 语言的树遍历解释器，负责直接执行 X 语言程序的抽象语法树（AST）。它是 X 语言编译器的重要组成部分，提供了快速执行 X 语言代码的途径。

## 功能定位

- **快速执行**：直接解释执行 X 语言代码，无需编译
- **开发工具**：支持快速原型开发和调试
- **语言特性验证**：用于验证语言设计和实现
- **学习工具**：帮助理解 X 语言的执行模型

## 架构与依赖

```
x-interpreter/
├── Cargo.toml       # 项目元数据和依赖
└── src/
    └── lib.rs       # 核心实现
```

### 核心依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| x-parser | 工作区 | 提供 AST 结构 |
| x-perceus | 工作区 | Perceus 内存管理 |
| im | 15.1 | 不可变数据结构 |
| tokio | 1.35 | 异步运行时 |
| num-bigint | 0.4 | 大整数支持 |
| num-traits | 0.2 | 数值类型支持 |
| rand | 0.8 | 随机数生成 |
| thiserror | 1.0 | 错误类型定义 |
| log | 0.4 | 日志功能 |

### 主要组件

1. **Interpreter**：解释器核心实例
2. **Value**：解释器支持的数据类型
3. **eval_expression**：表达式求值函数
4. **eval_statement**：语句执行函数
5. **eval_function**：函数调用处理

## 实现状态

**当前状态**：功能完善的树遍历解释器

- 支持大部分 X 语言特性
- 实现了完整的表达式求值和语句执行
- 支持大整数、数组、映射等复杂类型
- 包含完整的错误处理机制

## 使用方法

### 作为库使用

```rust
use x_interpreter::Interpreter;
use x_parser::Parser;

// 解析 X 语言代码
let parser = Parser::new();
let program = parser.parse("fn main() { print(\"Hello\") }").unwrap();

// 创建解释器实例
let mut interpreter = Interpreter::new();

// 执行程序
let result = interpreter.eval_program(program);

match result {
    Ok(value) => println!("程序执行结果: {:?}", value),
    Err(error) => eprintln!("执行错误: {:?}", error),
}
```

### 通过编译器 CLI 使用

```bash
cd tools/x-cli && cargo run -- run <file.x>
```

## 支持的特性

### 基础类型

- 整数（i8, i16, i32, i64, isize）
- 浮点数（f32, f64）
- 布尔值
- 字符和字符串
- 大整数（BigInt）

### 数据结构

- 数组
- 映射（Map）
- 可选类型（Option）
- 结果类型（Result）

### 控制流

- if/else 语句
- while/for 循环
- 函数调用
- 递归函数

### 函数特性

- 函数定义和调用
- 闭包支持
- 可变参数函数

## 代码风格与规范

- 使用标准 Rust 风格，执行 `cargo fmt` 格式化
- 解释器状态保持在结构体内部
- 值类型使用 enum 表示，支持深度克隆
- 错误处理使用 thiserror 库
- 文档字符串使用中文，符合项目整体风格

## 相关资源

- **主项目文档**：[../..//CLAUDE.md](../../CLAUDE.md)
- **语法分析器**：[../x-parser/CLAUDE.md](../x-parser/CLAUDE.md)
- **编译器架构**：[../../ARCHITECTURE.md](../../ARCHITECTURE.md)

## Testing & Verification

### 最小验证（只验证本 crate）

```bash
cd compiler
cargo test -p x-interpreter
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

```bash
cd compiler
cargo llvm-cov -p x-interpreter --tests --lcov --output-path target/coverage/x-interpreter.lcov
```

### 集成验证（通过 CLI 路径）

```bash
cd tools/x-cli
cargo test -p x-cli
```
