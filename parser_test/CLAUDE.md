# parser_test - X 语言语法分析器测试工具

本文件为 Claude Code (claude.ai/code) 提供在本子项目工作时的指导规范。

## 子项目概览

**parser_test** 是一个简单的测试工具，用于演示和手动测试 `x-parser` 语法分析器的功能。它读取固定的源代码字符串，调用语法分析器进行解析，并将生成的 AST 打印到控制台。

### 功能定位

- 简单演示 x-parser 的基本用法
- 手动测试语法分析器输出
- 查看 AST 结构用于调试
- 开发调试辅助工具

### 目录结构

```
parser_test/
├── Cargo.toml          # 包配置
├── CLAUDE.md           # 本文件
├── TODO.md             # 子项目待办事项
└── src/
    └── main.rs         # 主程序入口
```

## 依赖关系

### 上游依赖（使用的 crate）

| Crate | 用途 | 路径 |
|-------|------|------|
| `x-parser` | 语法分析器 | `../compiler/x-parser` |
| `x-lexer` | 词法分析器（间接依赖） | `../compiler/x-lexer` |

### 下游依赖

无 - 这是一个独立的测试二进制程序。

## 当前状态

### ✅ 已实现

1. 基本框架：创建 XParser 实例
2. 简单解析：对 `"1 + 1"` 进行解析
3. 输出显示：使用 Debug 格式打印 Program AST

### 🔴 待改进

1. 硬编码的源代码，无法从命令行参数或文件读取
2. 输出使用 Debug 格式，可读性较差
3. 无错误处理的详细展示（不显示源码位置）
4. 无单元测试

## 常用命令

```bash
# 构建并运行
cargo run

# 只构建
cargo build
```

## 修改指南

### 扩展功能建议

1. 支持从命令行参数接收源文件路径
2. 支持从标准输入读取代码
3. 更友好的 AST 输出格式（美化、缩进、彩色）
4. 使用 `pipeline::format_parse_error` 显示带位置的错误
5. 可选输出 JSON 格式便于工具处理
6. 支持 `--emit ast` 类似 x-cli 的功能

## 测试

当前无单元测试。通过 `cargo run` 手动验证。

## 关联文档

- 项目根目录 [CLAUDE.md](../CLAUDE.md) - 总览和架构
- [DESIGN_GOALS.md](../DESIGN_GOALS.md) - 设计目标（最高权威）
- [TODO.md](./TODO.md) - 本子项目待办事项
- [x-parser](../compiler/x-parser/) - 被测试的语法分析器
