# 编译流水线文档索引

> 本索引为 X 编译器编译流水线的完整文档导航。所有文档都确保编译的每个阶段都可独立运行并输出阶段性成果。

## 📚 文档导航

### 1. 快速开始（3 分钟）

**推荐文档**：[README.md](./README.md) - "一句话总结" 与 "快速命令速查表"

快速学习编译流程的各个阶段与对应的命令：

```bash
cd tools/x-cli

# 词法分析
cargo run -- compile test.x --emit tokens

# 语法分析
cargo run -- compile test.x --emit ast

# 类型检查
cargo run -- check test.x

# 代码生成
cargo run -- compile test.x --emit zig

# 完整编译
cargo run -- compile test.x -o test
```

### 2. 详细参考（20 分钟）

**推荐文档**：[README.md](./README.md) - "各阶段详细说明" 与 "常见问题"

逐个了解 9 个编译阶段：

| 阶段 | 命令 | 输出 | 文档位置 |
|------|------|------|---------|
| 1️⃣ 词法分析 | `--emit tokens` | 令牌流 | [README.md#词法分析](./README.md#1️⃣-词法分析lexing) |
| 2️⃣ 语法分析 | `--emit ast` | AST | [README.md#语法分析](./README.md#2️⃣-语法分析parsing) |
| 3️⃣ 类型检查 | `check` | 检查结果 | [README.md#类型检查](./README.md#3️⃣-类型检查type-checking) |
| 4️⃣-6️⃣ 中间表示 | `--emit hir/mir/lir` | IR 树形结构 | [README.md#中间表示](./README.md#4️⃣---6️⃣-中间表示hirlir) |
| 7️⃣ 代码生成 | `--emit zig/rust` | 目标代码 | [README.md#代码生成](./README.md#7️⃣-代码生成code-generation) |
| 8️⃣ 编译链接 | `compile -o <out>` | 可执行文件 | [README.md#编译链接](./README.md#8️⃣-编译链接compilation--linking) |
| 9️⃣ 解释执行 | `run` | 程序输出 | [README.md#解释执行](./README.md#9️⃣-解释执行interpretation) |

### 3. 调试指南（10 分钟）

**推荐文档**：[README.md](./README.md) - "调试流程" 与 "完整工作流示例"

学习如何排查编译问题：

**当编译失败时**：
1. 用 `--emit tokens` 检查词法分析
2. 用 `--emit ast` 检查语法分析
3. 用 `check` 检查类型错误
4. 用 `--emit hir/mir/lir` 检查中间表示转换
5. 用 `--emit zig` 检查代码生成
6. 查看生成的 Zig 代码，手工分析

**完整工作流示例**：
- 快速开发：编写 → 测试 → 编译 → 运行
- 调试问题：逐阶段检查 → 定位问题 → 修复
- 跨平台编译：编译多个目标平台

### 4. 深度学习（30 分钟）

**推荐文档**：[../../CLAUDE.md](../../CLAUDE.md) - "编译流水线各阶段独立运行与输出"

详细了解每个阶段的：
- 职责与目标
- 输入与输出
- 实现位置（Crate）
- 完整的命令示例
- 现状与待做项
- 调试建议与最佳实践

本章约 2000+ 行，包含：
- 流水线总体设计
- 9 个阶段的详细说明
- 完整工作流示例
- 常见问题排查
- 实现检查清单
- 扩展与完善计划

### 5. 测试与验证（45 分钟）

**推荐文档**：[README.md](./README.md) - "测试各个编译阶段"

学习如何验证各个阶段的正确性：

**单元测试**：
```bash
cd compiler
cargo test                    # 运行所有单元测试
cargo test -p x-lexer        # 运行词法分析测试
cargo test -p x-parser       # 运行语法分析测试
cargo test -p x-codegen      # 运行代码生成测试
```

**规格测试**：
```bash
cargo run -p x-spec          # 运行规格测试
```

**示例程序**：
```bash
cd tools/x-cli
cargo run -- run ../../examples/hello.x
cargo run -- compile ../../examples/hello.x -o hello
./hello
```

**检查清单**：
- [ ] 词法分析：令牌与位置信息
- [ ] 语法分析：AST 结构与优先级
- [ ] 类型检查：类型推断与约束
- [ ] 代码生成：生成代码的正确性
- [ ] 解释执行：程序行为一致性

### 6. 改进工作总结（15 分钟）

**推荐文档**：[IMPROVEMENTS.md](./IMPROVEMENTS.md)

了解本次改进工作的：
- 核心成果
- 文档体系
- 实现现状
- 下一步规划

包含：
- 📋 改进概述
- 🎯 核心成果（6 个方面）
- 📊 改进统计
- 🔧 实现现状
- 🎓 最佳实践
- 📈 后续改进路线图

---

## 🎯 按使用场景快速导航

### 🚀 快速开发

1. 写代码
2. 快速测试：`cargo run -- run test.x`（无编译）
3. 检查错误：`cargo run -- check test.x`
4. 编译：`cargo run -- compile test.x -o test`
5. 运行：`./test`

**参考文档**：[README.md#完整工作流示例](./README.md#完整工作流示例) - "场景 A：快速开发"

### 🔍 调试编译问题

1. 查看词法分析：`cargo run -- compile test.x --emit tokens`
2. 查看语法分析：`cargo run -- compile test.x --emit ast`
3. 查看类型检查：`cargo run -- check test.x`
4. 查看代码生成：`cargo run -- compile test.x --emit zig`
5. 手工分析生成的代码

**参考文档**：[README.md#调试流程](./README.md#调试流程) 或 [../../CLAUDE.md](../../CLAUDE.md)

### 🌍 跨平台编译

```bash
# 原生平台
cargo run -- compile app.x -o app

# WebAssembly
cargo run -- compile app.x --target wasm -o app.wasm

# 优化版本
cargo run -- compile app.x --release -o app_opt
```

**参考文档**：[README.md#完整工作流示例](./README.md#完整工作流示例) - "场景 C：跨平台编译"

### ✅ 验证新特性

1. 添加到规范：`spec/README.md`
2. 更新词法分析器：`compiler/x-lexer`
3. 更新语法分析器：`compiler/x-parser`
4. 更新代码生成器：`compiler/x-codegen`
5. 添加规格测试：`spec/x-spec`
6. 运行测试验证

**参考文档**：[README.md#常见测试场景](./README.md#常见测试场景) - "场景 1：验证新语言特性"

### 📚 学习编译原理

按顺序学习：

1. **词法分析**
   - 读：[README.md#词法分析](./README.md#1️⃣-词法分析lexing)
   - 做：`cargo run -- compile hello.x --emit tokens`
   - 深入：[CLAUDE.md](../../CLAUDE.md) - "1️⃣ 词法分析"

2. **语法分析**
   - 读：[README.md#语法分析](./README.md#2️⃣-语法分析parsing)
   - 做：`cargo run -- compile hello.x --emit ast`
   - 深入：[CLAUDE.md](../../CLAUDE.md) - "2️⃣ 语法分析"

3. **类型检查**
   - 读：[README.md#类型检查](./README.md#3️⃣-类型检查type-checking)
   - 做：`cargo run -- check hello.x`
   - 深入：[CLAUDE.md](../../CLAUDE.md) - "3️⃣ 类型检查"

4. 继续学习其他阶段...

---

## 📖 文档对照表

| 文档 | 位置 | 内容 | 适合人群 | 阅读时间 |
|------|------|------|---------|---------|
| **快速参考指南** | [README.md](./README.md) | 命令速查、调试流程、常见问题、测试指南 | 所有人 | 20 分钟 |
| **改进工作总结** | [IMPROVEMENTS.md](./IMPROVEMENTS.md) | 改进目标、成果、现状、规划 | 想了解改进的人 | 15 分钟 |
| **详细编译指南** | [../../CLAUDE.md](../../CLAUDE.md) | 9 阶段详解、实现细节、最佳实践 | 编译器开发者 | 1 小时 |
| **语言规范** | [../../spec/README.md](../../spec/README.md) | 语法规则、类型系统、语义 | 语言学习者、开发者 | 2+ 小时 |
| **设计目标** | [../../DESIGN_GOALS.md](../../DESIGN_GOALS.md) | 设计原则、语言理念 | 所有人 | 30 分钟 |

---

## 🔗 相关资源

### 源代码位置

```
compiler/
├── x-lexer/          # 词法分析实现
├── x-parser/         # 语法分析实现
├── x-typechecker/    # 类型检查实现
├── x-hir/            # HIR 转换实现
├── x-mir/            # MIR 转换实现
├── x-lir/            # LIR 转换实现
├── x-codegen/        # 代码生成实现（Zig、Rust、C、.NET）
└── x-interpreter/    # 解释器实现

tools/
└── x-cli/            # CLI 工具实现

examples/             # 示例与基准程序

spec/
└── x-spec/           # 规格测试
```

### 常用命令速查

```bash
# 快速运行（无编译）
cargo run -- run <file.x>

# 检查语法与类型
cargo run -- check <file.x>

# 查看各阶段输出
cargo run -- compile <file.x> --emit tokens
cargo run -- compile <file.x> --emit ast
cargo run -- compile <file.x> --emit hir
cargo run -- compile <file.x> --emit zig

# 完整编译
cargo run -- compile <file.x> -o <output>

# 运行单元测试
cd compiler && cargo test

# 运行规格测试
cargo run -p x-spec
```

### 获取帮助

**在线资源**：
- 🏗️ [项目 README](../../README.md) - 项目概览
- 📋 [CLAUDE.md](../../CLAUDE.md) - 完整开发指南
- 📚 [spec/README.md](../../spec/README.md) - 语言规范
- 🎨 [DESIGN_GOALS.md](../../DESIGN_GOALS.md) - 设计目标

**本地快速查询**：
```bash
# 查看此索引
cat docs/compiler-pipeline/INDEX.md

# 查看快速参考
cat docs/compiler-pipeline/README.md

# 查看改进总结
cat docs/compiler-pipeline/IMPROVEMENTS.md

# 查看详细编译指南
grep -A 100 "编译流水线各阶段独立运行与输出" CLAUDE.md
```

---

## ✨ 核心特性一览

### ✅ 已实现
- 词法分析：所有令牌类型支持
- 语法分析：核心语法规则完整
- 代码生成：Zig 后端成熟，其他后端早期
- 解释执行：支持核心语言特性

### 🚧 进行中
- 类型检查：框架完成，检查逻辑待实现
- HIR/MIR/LIR 转换：框架完成，转换逻辑待实现
- Perceus 内存管理：设计完成，代码生成待实现

### 📅 规划中
- LSP 支持（代码提示、跳转、诊断）
- 增量编译缓存
- 性能监控工具
- 其他后端完善

---

## 🎓 学习路径建议

### 初级用户（想快速使用编译器）

1. 阅读：[README.md](./README.md) - "一句话总结"
2. 尝试：快速命令速查表中的 3 个命令
3. 参考：[README.md#常见问题](./README.md#常见问题) 当遇到问题时

**预计时间**：10 分钟

### 中级用户（想理解编译过程）

1. 阅读：[README.md](./README.md) - 完整内容
2. 尝试：[README.md#完整工作流示例](./README.md#完整工作流示例) 中的 3 个场景
3. 学习：[README.md#各阶段详细说明](./README.md#各阶段详细说明) 中关心的阶段

**预计时间**：1 小时

### 高级用户（想开发编译器或贡献）

1. 阅读：[CLAUDE.md](../../CLAUDE.md) 的"编译流水线各阶段独立运行与输出"
2. 阅读：[IMPROVEMENTS.md](./IMPROVEMENTS.md) 了解实现现状与规划
3. 查看：源代码 `compiler/x-*` 了解具体实现
4. 参考：[README.md#测试与验证各个编译阶段](./README.md#测试各个编译阶段) 的检查清单
5. 贡献：按照 [../../CLAUDE.md#修改语言--实现步骤](../../CLAUDE.md#修改语言--实现步骤) 添加新特性

**预计时间**：2+ 小时（持续）

---

## 📞 获得帮助

### 问题排查流程

遇到问题时，按以下顺序查阅文档：

1. **问题在哪个阶段？**
   - 查看：[README.md#调试流程](./README.md#调试流程)

2. **该阶段的命令是什么？**
   - 查看：[README.md#快速命令速查表](./README.md#快速命令速查表)

3. **该阶段详细说明**
   - 查看：[README.md#各阶段详细说明](./README.md#各阶段详细说明)

4. **常见问题与解决方案**
   - 查看：[README.md#常见问题](./README.md#常见问题)

5. **深入实现细节**
   - 查看：[CLAUDE.md](../../CLAUDE.md)
   - 查看：源代码注释

### 如果找不到答案

- 提交 Issue（如使用 GitHub）
- 查看 [../../CLAUDE.md](../../CLAUDE.md) 中的"代码风格和日志"
- 启用详细日志：`RUST_LOG=debug cargo run -- compile test.x --emit ast`

---

## 🎉 总结

本文档集合提供了 X 编译器编译流水线的完整指导：

- ✅ **完整的命令参考**：9 个阶段的所有命令与选项
- ✅ **详细的工作流示例**：快速开发、调试问题、跨平台编译
- ✅ **有效的问题排查**：逐阶段调试、常见问题解答
- ✅ **规范的实现指导**：检查清单、最佳实践、测试方法
- ✅ **清晰的发展规划**：当前状态、优先级、时间估计

希望这些资源能帮助你高效地使用与开发 X 编译器！

---

**导航**：
- 快速开始 → [README.md](./README.md)
- 深度学习 → [CLAUDE.md](../../CLAUDE.md)
- 了解改进 → [IMPROVEMENTS.md](./IMPROVEMENTS.md)
- 语言规范 → [spec/README.md](../../spec/README.md)
- 设计目标 → [DESIGN_GOALS.md](../../DESIGN_GOALS.md)

*本索引最后更新：2024*