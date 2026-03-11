# interpreter_test - X 语言解释器测试工具

本文件为 Claude Code (claude.ai/code) 提供在本子项目工作时的指导规范。

## 子项目概览

**interpreter_test** 是一个简单的测试工具，用于演示和手动测试 `x-interpreter` 树遍历解释器的功能。它读取固定的源代码字符串，调用语法分析器解析，然后调用解释器执行，并打印执行结果。

### 功能定位

- 简单演示 x-interpreter 的基本用法
- 手动测试解释器执行
- 开发调试辅助工具

### 目录结构

```
interpreter_test/
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
| `x-interpreter` | 树遍历解释器 | `../compiler/x-interpreter` |
| `x-parser` | 语法分析器 | `../compiler/x-parser` |
| `x-lexer` | 词法分析器（间接依赖） | `../compiler/x-lexer` |

### 下游依赖

无 - 这是一个独立的测试二进制程序。

## 当前状态

### ✅ 已实现

1. 基本框架：创建 XParser 和 Interpreter 实例
2. 简单执行：对 `"1 + 1"` 进行解析和执行
3. 结果显示：打印成功或失败信息

### 🔴 待改进

1. 硬编码的源代码，无法从命令行参数或文件读取
2. 不捕获或显示 `print` 语句的输出
3. 不显示解释器的返回值或最终状态
4. 无错误处理的详细展示
5. 无单元测试

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
3. 捕获并显示解释器的输出
4. 显示最终变量环境或返回值
5. 使用 `pipeline::format_parse_error` 显示带位置的解析错误
6. 支持传递命令行参数给被解释的程序

## 测试

当前无单元测试。通过 `cargo run` 手动验证。

## 关联文档

- 项目根目录 [CLAUDE.md](../CLAUDE.md) - 总览和架构
- [DESIGN_GOALS.md](../DESIGN_GOALS.md) - 设计目标（最高权威）
- [TODO.md](./TODO.md) - 本子项目待办事项
- [x-interpreter](../compiler/x-interpreter/) - 被测试的解释器
