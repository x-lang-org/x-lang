# X 编译流水线快速参考指南

> 本文档为编译流水线各阶段的快速参考。详见 [CLAUDE.md](../../CLAUDE.md) 的"编译流水线各阶段独立运行与输出"部分。
>
> **目录**：[快速命令](#快速命令速查表) | [调试流程](#调试流程) | [各阶段详解](#各阶段详细说明) | [测试验证](#测试与验证各阶段)

---

## 📋 本文档的改进工作汇总

本指南完整规范了 X 编译流水线的各个阶段，确保每个阶段都可以独立运行并输出阶段性成果。

### ✅ 已完成

1. **编译流水线全面文档化**
   - 在 [CLAUDE.md](../../CLAUDE.md) 中添加"编译流水线各阶段独立运行与输出"章节（约 2000+ 行）
   - 详细说明 9 个编译阶段的职责、输入、输出、命令、实现位置
   - 包含实际示例、常见问题排查、实现建议与最佳实践

2. **快速参考指南**（本文档）
   - 命令速查表（各阶段一览）
   - 调试流程（从错误排查到问题解决）
   - 各阶段详细说明（输入输出、命令、示例）
   - 完整工作流示例（3 个场景）
   - 常见问题解答

3. **CLI 工具 `--emit` 选项支持**
   - `--emit tokens`：词法分析阶段输出
   - `--emit ast`：语法分析阶段输出
   - `check`：类型检查阶段输出
   - `--emit hir`、`--emit mir`、`--emit lir`：各 IR 阶段输出
   - `--emit zig|rust|c|dotnet`：代码生成阶段输出
   - `compile`：完整编译流程
   - `run`：解释执行

4. **阶段性成果输出规范**
   - 每个阶段都能独立停止并输出成果
   - 所有输出都附带源代码位置信息（Span）
   - 错误消息含位置与上下文代码片段

### 🚧 规划中 / 待完成

1. **类型检查器实现**（高优先级）
   - 当前是桩实现，所有代码都能通过
   - 需要完整的 Hindley-Milner 类型推断
   - 需要约束求解与泛型支持
   - 需要 Option/Result 穷尽处理检查

2. **中间表示（HIR/MIR/LIR）转换**（高优先级）
   - 框架已建立，转换逻辑待完成
   - 需要符号表生成与维护
   - 需要控制流与数据流分析
   - 需要内存管理信息（Perceus）传递

3. **Perceus 内存管理代码生成**（中优先级）
   - 在代码生成时插入 dup/drop 操作
   - 实现复用分析优化
   - 生成对应的目标代码

4. **性能监控与调试工具**（中优先级）
   - `--verbose` 标志输出耗时信息
   - `RUST_LOG=debug` 启用详细日志
   - 增量编译缓存支持

5. **其他后端完善**（低优先级）
   - Rust 后端：从早期实现扩展
   - C 后端：从早期实现扩展
   - .NET 后端：从早期实现扩展

### 📚 相关文档更新

- `CLAUDE.md`：新增"编译流水线各阶段独立运行与输出"完整章节
- `docs/compiler-pipeline/README.md`：本文档（快速参考与测试指南）

---

## 一句话总结

X 编译器采用**分阶段处理**，每个阶段可独立运行并输出阶段成果。使用 `--emit <stage>` 在任意阶段停止。

```
源代码 → 词法 → 语法 → 类型检查 → HIR → MIR → LIR → 代码生成 → 编译链接 → 可执行文件
         ↓        ↓                                    ↓
      tokens   ast   (check)                  zig/rust/c
```

---

## 快速命令速查表

### 各阶段独立运行

| 阶段 | 命令 | 输出 | 用途 |
|------|------|------|------|
| **词法分析** | `compile <file.x> --emit tokens` | 令牌流 | 检查词法分析的正确性 |
| **语法分析** | `compile <file.x> --emit ast` | 抽象语法树 | 验证语法规则与优先级 |
| **类型检查** | `check <file.x>` | ✓通过 或 ✗错误 | 验证类型安全 |
| **高层 IR** | `compile <file.x> --emit hir` | HIR 树形结构 | 检查符号表与语义转换 |
| **中层 IR** | `compile <file.x> --emit mir` | MIR 树形结构 | 验证控制流与数据流 |
| **低层 IR** | `compile <file.x> --emit lir` | LIR 树形结构 | 检查内存布局与调用约定 |
| **代码生成** | `compile <file.x> --emit zig` | Zig 源代码 | 查看生成的目标代码 |
| **编译链接** | `compile <file.x> -o <output>` | 可执行文件 | 生成最终二进制文件 |
| **解释执行** | `run <file.x>` | 程序输出 | 快速测试（无编译） |

### 常用组合

```bash
cd tools/x-cli

# ⚡ 快速运行（无编译）
cargo run -- run ../../examples/hello.x

# ✓ 检查语法与类型
cargo run -- check ../../examples/hello.x

# 🔨 完整编译为可执行文件
cargo run -- compile ../../examples/hello.x -o hello

# 🎯 调试：查看各阶段输出
cargo run -- compile test.x --emit tokens  # 词法
cargo run -- compile test.x --emit ast     # 语法
cargo run -- check test.x                  # 类型
cargo run -- compile test.x --emit hir     # HIR
cargo run -- compile test.x --emit mir     # MIR
cargo run -- compile test.x --emit lir     # LIR
cargo run -- compile test.x --emit zig     # 代码生成

# 🌍 跨平台编译
cargo run -- compile test.x --target wasm -o test.wasm

# 📦 发布优化
cargo run -- compile test.x --release -o test_optimized

# 🔍 生成 Zig 代码供检查（不链接）
cargo run -- compile test.x --no-link
cat test.zig
```

---

## 调试流程

### 问题：编译失败

**步骤**：
```bash
# 1. 检查词法分析
cargo run -- compile test.x --emit tokens
# → 有无非法字符？令牌序列是否合理？

# 2. 检查语法分析
cargo run -- compile test.x --emit ast
# → AST 结构是否正确？是否解析到期望的深度？

# 3. 检查类型
cargo run -- check test.x
# → 有无类型错误？变量/函数是否定义？

# 4. 查看中间表示
cargo run -- compile test.x --emit hir
cargo run -- compile test.x --emit mir
cargo run -- compile test.x --emit lir
# → IR 转换是否正确？

# 5. 查看生成的代码
cargo run -- compile test.x --emit zig
# → Zig 代码语法是否正确？逻辑是否对应源代码？

# 6. 尝试完整编译
cargo run -- compile test.x -o test
# → 若 Zig 编译器报错，说明生成的代码有问题（见步骤 5）
```

### 问题：运行时错误

**步骤**：
```bash
# 1. 用解释器运行（观察程序行为）
cargo run -- run test.x

# 2. 查看生成的 Zig 代码
cargo run -- compile test.x --emit zig

# 3. 手工运行生成的代码
zig build-exe test.zig
./test

# 4. 添加日志调试
# 在源代码中使用 println()，重新运行
cargo run -- run test.x
```

---

## 各阶段详细说明

### 1️⃣ 词法分析（Lexing）

**输入**：源代码文本  
**输出**：令牌序列 + 位置信息（Span）

**命令**：
```bash
cargo run -- compile test.x --emit tokens
```

**示例输出**：
```
Token(Function) @ 0..8
Token(Identifier("main")) @ 9..13
Token(LeftParen) @ 13..14
Token(RightParen) @ 14..15
Token(LeftBrace) @ 16..17
...
Token(Eof) @ 100..100
```

**调试建议**：
- 验证关键字识别：`let`、`function`、`if` 等
- 检查字符串转义：`"\n"` 应正确识别为单个令牌
- 确认注释被忽略：`//` 和 `/* ... */` 不出现在令牌流中

---

### 2️⃣ 语法分析（Parsing）

**输入**：令牌序列  
**输出**：抽象语法树（AST）

**命令**：
```bash
cargo run -- compile test.x --emit ast
```

**示例输出**（简化）：
```
Program {
  declarations: [
    Function {
      name: "main",
      params: [],
      return_type: None,
      body: Block {
        statements: [
          Expression(Call {
            callee: Identifier("println"),
            args: [String("Hello")]
          })
        ]
      }
    }
  ]
}
```

**调试建议**：
- 验证运算符优先级：`a + b * c` 应解析为 `a + (b * c)`
- 检查表达式嵌套深度
- 验证语句块与控制流结构

---

### 3️⃣ 类型检查（Type Checking）

**输入**：AST  
**输出**：类型检查结果（成功或详细错误）

**命令**：
```bash
cargo run -- check test.x
```

**成功输出**：
```
Finished 检查通过（语法 + 类型）
```

**失败输出示例**：
```
error.x:5:10: 类型不匹配
  expected: integer
  found: string
  
  5 | let x: integer = "hello"
    |                  ^^^^^^^
```

**调试建议**：
- 添加显式类型注解，看是否能通过类型检查
- 逐行简化代码，隔离问题

**现状**：🚧 类型检查器目前是桩实现，所有代码都能通过。完整实现正在进行中。

---

### 4️⃣ - 6️⃣ 中间表示（HIR / MIR / LIR）

**HIR**（高层）：语言级别的语义转换（宏展开、模式匹配等）  
**MIR**（中层）：函数级别的控制流与数据流  
**LIR**（低层）：接近机器的表示

**命令**：
```bash
cargo run -- compile test.x --emit hir
cargo run -- compile test.x --emit mir
cargo run -- compile test.x --emit lir
```

**调试建议**：
- 查看中间表示是否简化了源代码
- 验证符号表与类型信息是否正确传递
- 检查内存管理代码（Perceus dup/drop）是否插入

**现状**：🚧 框架已建立，转换逻辑待完成

---

### 7️⃣ 代码生成（Code Generation）

**输入**：AST 或 IR  
**输出**：目标语言源代码

**Zig 后端**（推荐）：
```bash
cargo run -- compile test.x --emit zig
```

**其他后端**（实验性）：
```bash
cargo run -- compile test.x --emit rust
cargo run -- compile test.x --emit c
cargo run -- compile test.x --emit dotnet
```

**调试建议**：
- 手工检查生成的代码语法是否正确
- 验证控制流是否对应源代码
- 查看生成的代码是否可被目标编译器接受

**现状**：
- ✅ Zig：成熟
- 🚧 Rust、C、.NET：早期实现

---

### 8️⃣ 编译链接（Compilation & Linking）

**输入**：目标语言源代码  
**输出**：可执行文件或目标文件

**命令**：
```bash
# 默认编译（Zig 后端）
cargo run -- compile test.x -o test

# 指定目标平台
cargo run -- compile test.x --target native -o test
cargo run -- compile test.x --target wasm -o test.wasm

# 仅生成中间代码，不链接
cargo run -- compile test.x --no-link
# 输出: test.zig（供手动检查或后续处理）

# 发布优化
cargo run -- compile test.x --release -o test_opt
```

**调试建议**：
- 若 Zig 编译器报错，用 `--emit zig` 查看生成的代码
- 使用 `--no-link` 停止在代码生成阶段，便于检查
- 验证可执行文件能否运行

---

### 9️⃣ 解释执行（Interpretation）

**输入**：源代码  
**输出**：程序的标准输出与返回值

**命令**：
```bash
cargo run -- run test.x
```

**示例**：
```bash
$ cargo run -- run ../../examples/hello.x
Hello, world!
    Finished 运行成功
```

**特点**：
- ⚡ 无编译开销，立即反馈
- 🐛 支持交互式调试（规划中）
- 👍 适合快速迭代与原型

---

## 完整工作流示例

### 场景 A：快速开发

```bash
cd tools/x-cli

# 1. 编写代码
echo 'function greet(name) = "Hello, " + name' > greet.x

# 2. 快速测试（解释执行）
cargo run -- run greet.x

# 3. 若有错误，逐阶段检查
cargo run -- compile greet.x --emit tokens
cargo run -- compile greet.x --emit ast
cargo run -- check greet.x

# 4. 修复后，编译为可执行文件
cargo run -- compile greet.x -o greet

# 5. 运行可执行文件
./greet
```

### 场景 B：调试编译问题

```bash
cd tools/x-cli

# 编译报错，按顺序检查各阶段
cargo run -- compile complex.x --emit tokens
cargo run -- compile complex.x --emit ast
cargo run -- check complex.x
cargo run -- compile complex.x --emit hir
cargo run -- compile complex.x --emit mir
cargo run -- compile complex.x --emit lir
cargo run -- compile complex.x --emit zig

# 查看生成的 Zig 代码，手工分析问题
cat complex.zig
```

### 场景 C：跨平台编译

```bash
cd tools/x-cli

# 编译为原生可执行文件
cargo run -- compile app.x -o app

# 编译为 WebAssembly
cargo run -- compile app.x --target wasm -o app.wasm

# 编译为发布版本（优化）
cargo run -- compile app.x --release -o app_release
```

---

## 常见问题

### Q：AST 输出太大，难以阅读

**A**：使用 grep 过滤或保存到文件：
```bash
cargo run -- compile test.x --emit ast | grep -i function
cargo run -- compile test.x --emit ast > ast.txt
# 用编辑器打开 ast.txt
```

### Q：想查看生成的 Zig 代码但不编译

**A**：使用 `--emit zig` 或 `--no-link`：
```bash
# 输出到标准输出
cargo run -- compile test.x --emit zig

# 或生成文件
cargo run -- compile test.x --no-link
cat test.zig
```

### Q：Zig 编译器报错，不知道是哪里的问题

**A**：查看生成的 Zig 代码：
```bash
cargo run -- compile test.x --emit zig > test_gen.zig
# 手工检查 test_gen.zig 的语法和逻辑
```

### Q：想快速测试，不想编译

**A**：使用 `run` 命令（解释执行）：
```bash
cargo run -- run test.x
```

### Q：编译成功，但运行结果不对

**A**：
```bash
# 1. 用解释器运行，观察行为是否相同
cargo run -- run test.x

# 2. 查看生成的 Zig 代码
cargo run -- compile test.x --emit zig

# 3. 在源代码中加 println() 调试
# 重新运行解释器或编译
```

### Q：想对比两个后端的生成代码

**A**：分别生成：
```bash
cargo run -- compile test.x --emit zig > zig_gen.zig
cargo run -- compile test.x --emit rust > rust_gen.rs
diff zig_gen.zig rust_gen.rs
```

---

## 性能建议

### 快速反馈

```bash
# 用解释器快速测试（毫秒级）
cargo run -- run test.x

# 用 --check 验证语法和类型（秒级）
cargo run -- check test.x
```

### 深度调试

```bash
# 逐阶段输出，找出问题所在
cargo run -- compile test.x --emit tokens
cargo run -- compile test.x --emit ast
# ... 继续往下
```

### 发布优化

```bash
# 使用 --release 生成优化的可执行文件
cargo run -- compile test.x --release -o test_opt
```

---

## 实现状态

| 阶段 | 状态 | 说明 |
|------|------|------|
| 词法分析 | ✅ 完成 | 所有令牌与关键字已支持 |
| 语法分析 | ✅ 完成 | 核心语法规则已实现 |
| 类型检查 | 🚧 进行中 | 当前是桩实现，需要完整的类型推断逻辑 |
| HIR 转换 | 🚧 进行中 | 框架已建立，转换逻辑待完成 |
| MIR 转换 | 🚧 进行中 | 框架已建立，转换逻辑待完成 |
| LIR 转换 | 🚧 进行中 | 框架已建立，转换逻辑待完成 |
| Zig 后端 | ✅ 成熟 | 支持函数、变量、控制流、内置函数 |
| Rust 后端 | 🚧 早期 | 部分特性支持 |
| C 后端 | 🚧 早期 | 部分特性支持 |
| .NET 后端 | 🚧 早期 | 部分特性支持 |
| 解释器 | ✅ 成熟 | 支持核心语言特性 |

---

## 下一步

- [ ] 完整实现类型检查器
- [ ] 完成 HIR、MIR、LIR 转换逻辑
- [ ] 实现 Perceus 内存管理代码生成
- [ ] 优化编译速度（增量编译、并行化）
- [ ] 添加 LSP 支持
- [ ] 完善其他后端（Rust、C、.NET）

---

## 相关文档

- **主要指南**：[CLAUDE.md](../../CLAUDE.md) - "编译流水线各阶段独立运行与输出"
- **设计目标**：[DESIGN_GOALS.md](../../DESIGN_GOALS.md)
- **语言规范**：[spec/README.md](../../spec/README.md)
- **编译器架构**：[docs/](../) 其他文档
