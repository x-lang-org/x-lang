# x-cli - X 语言命令行工具

本文件为 Claude Code (claude.ai/code) 提供在本子项目工作时的指导规范。

## 子项目概览

**x-cli** 是 X 语言的官方命令行工具链入口，负责编排整个编译流水线、提供用户交互命令、管理项目和包。它是用户与 X 语言编译器交互的主要界面。

### 功能定位

- **编译流水线编排**：协调词法分析 → 语法分析 → 类型检查 → HIR → Perceus → 代码生成的完整流程
- **用户命令接口**：提供 `run`、`compile`、`check`、`build`、`test` 等开发命令
- **项目管理**：项目初始化、依赖管理、打包发布等功能
- **多后端支持**：支持 Zig、JavaScript、JVM、.NET 等多个代码生成后端

### 目录结构

```
tools/x-cli/
├── Cargo.toml          # 包配置
├── CLAUDE.md           # 本文件
├── TODO.md             # 子项目待办事项
└── src/
    ├── main.rs         # CLI 入口和命令定义
    ├── pipeline.rs     # 编译流水线编排
    ├── project.rs      # 项目结构管理
    ├── manifest.rs     # x.toml 清单解析
    ├── config.rs       # 配置管理
    ├── lockfile.rs     # 锁文件管理
    ├── registry.rs     # 包注册表接口
    ├── resolver.rs     # 依赖解析
    ├── utils.rs        # 工具函数
    └── commands/       # 各命令实现
        ├── mod.rs
        ├── run.rs      # ✅ 已实现 - 运行程序
        ├── compile.rs  # ✅ 已实现 - 编译（含 --emit）
        ├── check.rs    # ✅ 已实现 - 语法/类型检查
        ├── build.rs    # ✅ 部分实现 - 项目构建
        ├── test_cmd.rs # 🚧 桩实现 - 测试
        ├── bench.rs    # 🚧 桩实现 - 基准测试
        ├── clean.rs    # 🚧 桩实现 - 清理
        ├── doc.rs      # 🚧 桩实现 - 文档生成
        ├── fmt.rs      # 🚧 桩实现 - 格式化
        ├── lint.rs     # 🚧 桩实现 - 代码检查
        ├── repl.rs     # 🚧 桩实现 - REPL
        ├── init.rs     # 🚧 桩实现 - 初始化项目
        ├── new.rs      # 🚧 桩实现 - 新建项目
        ├── add.rs      # 🚧 桩实现 - 添加依赖
        ├── remove.rs   # 🚧 桩实现 - 移除依赖
        ├── update.rs   # 🚧 桩实现 - 更新依赖
        ├── vendor.rs   # 🚧 桩实现 - 依赖本地化
        ├── package.rs  # 🚧 桩实现 - 打包
        ├── publish.rs  # 🚧 桩实现 - 发布
        ├── ...         # 更多命令
```

## 依赖关系

### 上游依赖（使用的 crate）

| Crate | 用途 | 路径 |
|-------|------|------|
| `x-lexer` | 词法分析器 | `../../compiler/x-lexer` |
| `x-parser` | 语法分析器 | `../../compiler/x-parser` |
| `x-typechecker` | 类型检查器 | `../../compiler/x-typechecker` |
| `x-hir` | 高层中间表示 | `../../compiler/x-hir` |
| `x-perceus` | Perceus 内存管理分析 | `../../compiler/x-perceus` |
| `x-codegen` | 代码生成（Zig 后端） | `../../compiler/x-codegen` |
| `x-codegen-js` | JavaScript 后端 | `../../compiler/x-codegen-js` |
| `x-interpreter` | 树遍历解释器 | `../../compiler/x-interpreter` |

### 第三方依赖

| Crate | 用途 |
|-------|------|
| `clap` | CLI 参数解析 |
| `colored` | 终端彩色输出 |
| `serde` + `serde_json` + `toml` | 序列化/反序列化 |
| `thiserror` | 错误类型定义 |
| `log` + `env_logger` | 日志 |
| `dirs` | 用户目录定位 |
| `walkdir` | 目录遍历 |
| `flate2` + `tar` | 打包压缩 |

### 下游依赖

无 - x-cli 是二进制 crate，不被其他 crate 依赖。

## 当前状态

### ✅ 已实现

1. **CLI 框架**：完整的 clap 命令定义，包含 30+ 命令的骨架
2. **核心编译命令**：
   - `x run <file.x>` - 解析 → 类型检查 → 解释执行
   - `x check <file.x>` - 语法和类型检查
   - `x compile --emit tokens|ast|hir|pir|dotnet` - 输出中间表示
3. **项目结构**：`Project` 结构体支持查找 x.toml、定位源文件
4. **错误格式化**：`format_parse_error` 显示带行号列号和源码片段的错误

### 🚧 部分实现

1. **`x build`** - 骨架存在，条件编译的代码生成逻辑未启用
2. **`x compile`** - Zig 后端完整编译需 AST→XIR 转换（目前仅支持 `--emit`）

### ❌ 桩实现（返回 "未实现"）

所有其他命令（`test`、`bench`、`clean`、`doc`、`fmt`、`lint`、`repl`、包管理命令等）均为桩实现。

## 常用命令

```bash
# 构建
cargo build
cargo build --release

# 运行示例
cargo run -- run ../../examples/hello.x
cargo run -- check ../../examples/fib.x
cargo run -- compile ../../examples/hello.x --emit ast

# 运行测试
cargo test
```

## 修改指南

### 添加新命令

1. 在 `src/commands/` 中添加新文件 `<cmd_name>.rs`
2. 在 `src/commands/mod.rs` 中 `pub mod <cmd_name>`
3. 在 `src/main.rs` 的 `Commands` 枚举中添加变体
4. 在 `dispatch` 函数中匹配并调用 `commands::<cmd_name>::exec()`

### 修改流水线

`pipeline.rs` 中的 `run_pipeline` 函数定义了完整编译流程。修改时保持阶段顺序：词法 → 语法 → 类型检查 → HIR → Perceus。

## 测试

当前通过顶层 `cargo test` 间接测试。单元测试待添加。

## Testing & Verification

### 最小验证（只验证本 crate）

```bash
cd tools
cargo test -p x-cli
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

```bash
cd tools
cargo llvm-cov -p x-cli --tests --lcov --output-path target/coverage/x-cli.lcov
```

### 集成验证（与 compiler 工作区联动）

```bash
cd compiler
cargo test
```

### 外部依赖（Zig）

若需要走 Zig 后端完整编译链路（生成可执行文件/Wasm），需要安装 Zig 并加入 PATH（见仓库根 `CLAUDE.md` 的 Zig 说明）。

## 关联文档

- 项目根目录 [CLAUDE.md](../../CLAUDE.md) - 总览和架构
- [DESIGN_GOALS.md](../../DESIGN_GOALS.md) - 设计目标（最高权威）
- [TODO.md](./TODO.md) - 本子项目待办事项
