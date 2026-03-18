# X 语言

X 是一门**现代通用编程语言**，强调可读性第一、类型安全与内存安全，支持函数式、面向对象、过程式和声明式多种范式。采用 Perceus 编译时引用计数（无 GC、无手动管理），无 null、无异常，通过 `Option`/`Result` 与效果系统显式表达副作用与错误。

本仓库为 X 语言的**编译器、工具链与规格**实现与文档。

---

## 文档与规格

| 文档 | 说明 |
|------|------|
| [DESIGN_GOALS.md](DESIGN_GOALS.md) | **设计目标与原则**（宪法性文件，所有设计决策的最终依据） |
| [spec/](spec/) | **语言规格说明书**（语法、类型、语义的正式定义，见 [spec/README.md](spec/README.md)） |
| [docs/](docs/) | 教程、附录与专题文档（关键字、类型、表达式、编译器架构等） |
| [CLAUDE.md](CLAUDE.md) | 贡献与开发指南（构建、测试、目录结构） |

---

## 快速开始

- **运行 .x 文件**（解析 + 解释执行）  
  `cd tools/x-cli && cargo run -- run <file.x>`
- **检查语法与类型**  
  `cd tools/x-cli && cargo run -- check <file.x>`
- **编译为可执行文件**（Zig 后端）
  `cd tools/x-cli && cargo run -- compile <file.x> -o <output>`
- **运行示例**  
  `cd tools/x-cli && cargo run -- run ../../examples/hello.x`

更多命令与 LLVM 依赖说明见 [CLAUDE.md](CLAUDE.md)。

---

## 项目结构

| 目录 | 说明 |
|------|------|
| `compiler/` | 编译器核心（词法、语法、类型检查、HIR、Perceus、多后端代码生成、解释器） |
| `tools/x-cli` | 命令行工具（`run`、`check`、`compile` 等） |
| `spec/` | 语言规格（设计哲学、词法、类型、表达式、函数、效果、内存等） |
| `docs/` | 教程与专题文档 |
| `examples/` | 示例与基准程序 |

---

## 许可

本项目采用多重许可，你可任选其一使用、修改与分发：

- MIT License  
- Apache License 2.0  
- BSD 3-Clause License  

详见 [LICENSES.md](LICENSES.md)。
