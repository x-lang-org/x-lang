# X 语言编译器重构指南

## 概述

本文档描述 X 语言编译器的重构计划，以支持新的**两阶段后端架构**：

- **第一阶段（当前）**：十大后端，LIR 翻译为目标语言或字节码
- **第二阶段（未来）**：优化与直接生成，绕过中间语言提升性能

---

## I. 目标与原则

### 架构目标

1. **统一 LIR 输入**：所有后端共享同一个低级中间表示
2. **充分利用现有工具链**：通过源码翻译，复用 tsc、javac、rustc、swiftc 等成熟编译器
3. **分阶段成熟**：先通过翻译获得可用性，后续优化为直接生成
4. **编译速度优先**：特别是开发循环中的增量编译
5. **多目标支持**：原生代码、字节码、Web 等全覆盖

### 设计原则

- **兼容性**：第二阶段不破坏第一阶段的 API
- **测试覆盖**：每个后端都有独立的单元和集成测试
- **文档优先**：每个后端有详细的实现文档
- **渐进式**：从最简单的后端开始，逐步完善复杂后端

---

## II. 十大后端概览

### 第一阶段 - LIR 翻译架构

```
LIR (通用中间表示)
  │
  ├── 【源码翻译型】翻译为高级语言源码
  │   ├── TypeScript (→ .ts → tsc → JavaScript)
  │   ├── Java (→ .java → javac → .class)
  │   ├── Python (→ .py → CPython)
  │   ├── Zig (→ .zig → Zig 编译器 → Native/Wasm)
  │   ├── C# (→ .cs → dotnet → .dll/.exe)
  │   ├── Swift (→ .swift → swiftc → Native)
  │   └── Rust (→ .rs → rustc → Native)
  │
  ├── 【中间IR翻译型】翻译为标准中间表示
  │   └── LLVM (→ LLVM IR → LLVM opt → Native/Wasm)
  │
  ├── 【字节码翻译型】翻译为 VM 字节码
  │   └── Erlang (→ .beam → Erlang VM)
  │
  └── 【原生机器码型】直接生成机器码
      └── Native (→ x86_64/ARM64 → 可执行文件)
```

### 十大后端详表

| # | 后端 | 类型 | 成熟度 | 位置 | 第二阶段计划 |
|----|------|------|--------|------|-------------|
| 1 | **LLVM** | IR 翻译 | 🚧 早期 | `x-codegen-llvm/` | 考虑直接生成 |
| 2 | **TypeScript** | 源码翻译 | 🚧 早期 | `x-codegen/ts_backend.rs` | 优化输出 |
| 3 | **Java** | 源码翻译 | 🚧 早期 | `x-codegen/java_backend.rs` | ⭐ JVM 字节码直接生成 |
| 4 | **Python** | 源码翻译 | 🚧 早期 | `x-codegen/python_backend.rs` | 优化字节码 |
| 5 | **Erlang** | 字节码翻译 | 📋 规划 | `x-codegen/erlang_backend.rs` | ⭐ BEAM 字节码优化 |
| 6 | **Zig** | 源码翻译 | ✅ 成熟 | `x-codegen/zig_backend.rs` | 性能优化 |
| 7 | **C#** | 源码翻译 | 🚧 早期 | `x-codegen/csharp_backend.rs` | ⭐ .NET IL 直接生成 |
| 8 | **Swift** | 源码翻译 | 📋 规划 | `x-codegen/swift_backend.rs` | 性能优化 |
| 9 | **Rust** | 源码翻译 | 🚧 早期 | `x-codegen/rust_backend.rs` | 优化输出 |
| 10 | **Native** | 机器码型 | 📋 规划 | `x-codegen-native/` | ⭐ 完整机器码生成 |

**⭐ 标记**：第二阶段有重要的直接生成计划

---

## III. 目录结构规划

### 当前结构（编译器）

```
compiler/
├── Cargo.toml                          # 工作空间配置
├── Cargo.lock
├── RESTRUCTURE.md                      # 本文件
│
├── x-lexer/                            # 词法分析
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── tests/
│
├── x-parser/                           # 语法分析
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── tests/
│
├── x-typechecker/                      # 类型检查
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── tests/
│
├── x-hir/                              # 高级中间表示
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── tests/
│
├── x-mir/                              # 中层中间表示 + Perceus
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── tests/
│
├── x-lir/                              # 低级中间表示（XIR）
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── tests/
│
├── x-codegen/                          # 后端 - 第一阶段
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                      # 后端工厂和通用接口
│   │   ├── error.rs
│   │   ├── target.rs                   # 目标平台定义
│   │   │
│   │   ├── lir_to_ast.rs               # 通用 LIR → AST 翻译（可选）
│   │   │
│   │   ├── typescript_backend.rs       # TypeScript 后端
│   │   ├── java_backend.rs             # Java 后端
│   │   ├── python_backend.rs           # Python 后端
│   │   ├── zig_backend.rs              # Zig 后端 ✅
│   │   ├── csharp_backend.rs           # C# 后端
│   │   ├── swift_backend.rs            # Swift 后端
│   │   ├── rust_backend.rs             # Rust 后端
│   │   ├── erlang_backend.rs           # Erlang/BEAM 后端
│   │   │
│   │   ├── compile_with_*.rs           # 编译辅助（zig、java 等）
│   │   └── tests/
│   └── docs/
│       ├── README.md                   # 后端总览
│       ├── backends/                   # 各后端实现文档
│       │   ├── typescript.md
│       │   ├── java.md
│       │   ├── python.md
│       │   ├── zig.md
│       │   ├── csharp.md
│       │   ├── swift.md
│       │   ├── rust.md
│       │   └── erlang.md
│       └── architecture.md             # 第一阶段架构说明
│
├── x-codegen-llvm/                     # LLVM 后端（特殊）
│   ├── Cargo.toml
│   ├── src/lib.rs
│   ├── tests/
│   └── docs/
│       └── README.md
│
├── x-codegen-native/                   # Native 后端（规划中）
│   ├── Cargo.toml
│   ├── src/lib.rs
│   ├── tests/
│   └── docs/
│       ├── README.md
│       └── architecture.md
│
├── x-interpreter/                      # 解释执行引擎
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── tests/
│
└── tests/                              # 集成测试
    ├── integration/
    │   ├── backends/                   # 后端集成测试
    │   │   ├── typescript.rs
    │   │   ├── java.rs
    │   │   ├── python.rs
    │   │   ├── zig.rs
    │   │   ├── csharp.rs
    │   │   ├── swift.rs
    │   │   ├── rust.rs
    │   │   ├── erlang.rs
    │   │   ├── llvm.rs
    │   │   └── native.rs
    │   ├── e2e/                        # 端到端测试
    │   │   └── compile.rs
    │   └── lib.rs
    └── fixtures/                       # 测试用例
        ├── hello.x
        ├── fibonacci.x
        ├── array.x
        └── ...
```

### 优化后的建议结构

```
x-codegen/                              # 统一后端框架
├── Cargo.toml
├── src/
│   ├── lib.rs                          # 导出公共接口
│   │
│   ├── common/                         # 通用模块
│   │   ├── mod.rs
│   │   ├── error.rs                    # 统一错误处理
│   │   ├── target.rs                   # 目标平台定义
│   │   ├── context.rs                  # 代码生成上下文
│   │   └── utils.rs                    # 通用工具函数
│   │
│   ├── lir/                            # LIR 相关
│   │   ├── mod.rs
│   │   ├── visitor.rs                  # LIR 访问者模式
│   │   └── transformers.rs             # 通用转换
│   │
│   ├── backends/                       # 十大后端
│   │   ├── mod.rs
│   │   ├── base.rs                     # Backend trait 定义
│   │   │
│   │   ├── typescript/                 # TypeScript 后端
│   │   │   ├── mod.rs
│   │   │   ├── codegen.rs
│   │   │   ├── tests.rs
│   │   │   └── examples.rs
│   │   │
│   │   ├── java/                       # Java 后端
│   │   │   ├── mod.rs
│   │   │   ├── codegen.rs
│   │   │   ├── stage2_bytecode.rs     # 第二阶段规划
│   │   │   ├── tests.rs
│   │   │   └── examples.rs
│   │   │
│   │   ├── python/
│   │   │   ├── mod.rs
│   │   │   ├── codegen.rs
│   │   │   ├── tests.rs
│   │   │   └── examples.rs
│   │   │
│   │   ├── zig/                        # Zig 后端 ✅
│   │   │   ├── mod.rs
│   │   │   ├── codegen.rs
│   │   │   ├── tests.rs
│   │   │   └── examples.rs
│   │   │
│   │   ├── csharp/                     # C# 后端
│   │   │   ├── mod.rs
│   │   │   ├── codegen.rs
│   │   │   ├── stage2_il.rs            # 第二阶段规划
│   │   │   ├── tests.rs
│   │   │   └── examples.rs
│   │   │
│   │   ├── swift/
│   │   │   ├── mod.rs
│   │   │   ├── codegen.rs
│   │   │   ├── tests.rs
│   │   │   └── examples.rs
│   │   │
│   │   ├── rust/
│   │   │   ├── mod.rs
│   │   │   ├── codegen.rs
│   │   │   ├── tests.rs
│   │   │   └── examples.rs
│   │   │
│   │   └── erlang/                     # Erlang/BEAM 后端
│   │       ├── mod.rs
│   │       ├── codegen.rs
│   │       ├── stage2_beam.rs          # 第二阶段规划
│   │       ├── tests.rs
│   │       └── examples.rs
│   │
│   ├── toolchain/                      # 工具链集成
│   │   ├── mod.rs
│   │   ├── typescript.rs               # tsc 集成
│   │   ├── java.rs                     # javac 集成
│   │   ├── python.rs                   # CPython 集成
│   │   ├── zig.rs                      # Zig 编译器集成
│   │   ├── csharp.rs                   # dotnet 集成
│   │   ├── swift.rs                    # swiftc 集成
│   │   ├── rust.rs                     # rustc 集成
│   │   └── erlang.rs                   # Erlang 编译器集成
│   │
│   ├── factory.rs                      # 后端工厂模式
│   └── tests/
│       ├── integration/
│       │   └── all_backends.rs         # 全后端集成测试
│       └── unit/
│           └── backends.rs             # 单后端单元测试
│
├── docs/
│   ├── README.md                       # 总览文档
│   ├── ARCHITECTURE.md                 # 第一阶段架构
│   ├── STAGE2_PLAN.md                  # 第二阶段规划
│   │
│   ├── backends/                       # 后端实现指南
│   │   ├── TEMPLATE.md                 # 后端实现模板
│   │   ├── typescript.md
│   │   ├── java.md
│   │   ├── python.md
│   │   ├── zig.md
│   │   ├── csharp.md
│   │   ├── swift.md
│   │   ├── rust.md
│   │   └── erlang.md
│   │
│   └── guides/                         # 开发指南
│       ├── adding_backend.md           # 如何添加新后端
│       ├── testing.md                  # 测试策略
│       └── profiling.md                # 性能分析
│
└── examples/                           # 后端示例代码
    ├── hello.x                         # 所有后端共用的示例
    ├── fibonacci.x
    ├── array_ops.x
    └── type_system.x
```

---

## IV. 第一阶段实现清单

### A. 后端工厂和通用接口

- [ ] 定义 `Backend` trait（通用接口）
  - `fn codegen(&self, module: &LirModule) -> Result<BackendOutput>`
  - `fn compile(&self, output: BackendOutput) -> Result<Executable>`
- [ ] 定义 `BackendContext`（代码生成上下文）
- [ ] 定义 `Target`（目标平台）
- [ ] 实现后端工厂 `BackendFactory`

### B. 十大后端实现状态

#### ✅ 已完成

- [x] **Zig 后端**（成熟）
  - 代码生成完整
  - 编译和测试通过
  - 文档完善

#### 🚧 进行中/早期

- [ ] **LLVM 后端**（独立 crate）
  - [ ] LLVM IR 生成
  - [ ] 优化 Pass 集成
  - [ ] 多目标支持
  - [ ] 测试覆盖

- [ ] **TypeScript 后端**
  - [ ] LIR → TypeScript AST 转换
  - [ ] 类型系统映射
  - [ ] 代码格式化
  - [ ] tsc 集成测试

- [ ] **Java 后端**
  - [ ] LIR → Java AST 转换
  - [ ] 类型系统映射
  - [ ] javac 集成
  - [ ] .class 文件验证

- [ ] **Python 后端**
  - [ ] LIR → Python AST 转换
  - [ ] 动态类型适配
  - [ ] CPython 执行验证
  - [ ] NumPy 互操作

- [ ] **C# 后端**
  - [ ] LIR → C# AST 转换
  - [ ] .NET 类型映射
  - [ ] dotnet 集成
  - [ ] Unity 兼容性

- [ ] **Swift 后端**
  - [ ] LIR → Swift AST 转换
  - [ ] 类型系统映射
  - [ ] swiftc 集成
  - [ ] iOS/macOS 支持

- [ ] **Rust 后端**
  - [ ] LIR → Rust AST 转换
  - [ ] 所有权系统适配
  - [ ] rustc 集成
  - [ ] crate 发布支持

#### 📋 规划中

- [ ] **Erlang 后端**
  - [ ] BEAM 字节码生成或 Erlang 源码翻译
  - [ ] Actor 模型映射
  - [ ] 错误处理体系
  - [ ] 分布式特性支持

- [ ] **Native 后端**
  - [ ] x86_64 指令生成
  - [ ] ARM64 指令生成
  - [ ] 寄存器分配
  - [ ] 可执行文件生成

### C. 文档清单

- [ ] `x-codegen/docs/README.md` - 后端总览
- [ ] `x-codegen/docs/ARCHITECTURE.md` - 第一阶段架构详解
- [ ] `x-codegen/docs/backends/TEMPLATE.md` - 后端实现模板
- [ ] `x-codegen/docs/backends/*.md` - 各后端实现指南（10 个）
- [ ] `x-codegen/docs/guides/adding_backend.md` - 添加新后端指南
- [ ] `x-codegen/docs/guides/testing.md` - 测试策略
- [ ] `x-codegen-llvm/docs/README.md` - LLVM 后端文档
- [ ] `x-codegen-native/docs/README.md` - Native 后端文档

### D. 测试清单

- [ ] 全后端集成测试框架
- [ ] 每个后端单元测试
- [ ] 端到端编译测试
- [ ] 目标程序行为验证测试
- [ ] 性能基准测试

---

## V. 第二阶段规划

### A. 直接字节码生成

```
【第二阶段优化】

Java 后端：
  LIR → JVM 字节码 (.class) [直接生成]
  优势：跳过 Java 源码层，编译更快

C# 后端：
  LIR → .NET IL/CIL (.dll/.exe) [直接生成]
  优势：跳过 C# 源码层，编译更快

Erlang 后端：
  LIR → BEAM 字节码优化版本
  优势：改进字节码质量，运行更快

Native 后端：
  LIR → 优化的机器码生成
  优势：更好的寄存器分配、指令调度
```

### B. 实现顺序

1. **高优先级**（收益大）
   - Java 字节码直接生成
   - C# IL 直接生成
   - Native 代码优化

2. **中优先级**（收益中等）
   - Erlang 字节码优化
   - LLVM 直接生成（可选）

3. **低优先级**（收益相对小）
   - 其他后端优化

---

## VI. 开发流程

### 1. 添加新后端

```
步骤：
1. 在 x-codegen/src/backends/ 创建新目录
2. 实现 Backend trait
3. 实现 LIR → 目标格式 转换
4. 集成目标工具链
5. 编写单元和集成测试
6. 编写文档和示例
7. 在工厂中注册后端
```

参考：`x-codegen/docs/guides/adding_backend.md`

### 2. 测试策略

```
三层测试：
1. 单元测试：后端内部功能测试
2. 集成测试：LIR 输入 → 目标代码输出
3. 端到端测试：编译 → 执行 → 验证结果
```

### 3. 版本管理

- **v0.1**：Zig 后端稳定（当前）
- **v0.2**：TypeScript + Python 后端可用
- **v0.3**：Java + C# 后端可用
- **v0.4**：所有十大后端可用
- **v1.0**：第二阶段优化完成

---

## VII. 模板与示例

### 后端实现模板

```rust
// x-codegen/src/backends/YOUR_BACKEND/mod.rs

use crate::common::{Backend, BackendContext, BackendOutput};
use x_lir::module::Module;

pub struct YourBackend {
    context: BackendContext,
}

impl Backend for YourBackend {
    fn codegen(&self, module: &Module) -> Result<BackendOutput> {
        // 1. 遍历 LIR 模块
        // 2. 转换为目标格式（AST 或字节码）
        // 3. 返回 BackendOutput
        todo!()
    }

    fn compile(&self, output: BackendOutput) -> Result<Executable> {
        // 调用目标工具链编译
        // 返回可执行文件路径
        todo!()
    }
}
```

---

## VIII. 时间线

| 阶段 | 目标 | 时间 |
|------|------|------|
| **第一阶段** | 十大后端可用 | Q1-Q2 2024 |
| **第二阶段** | 直接生成优化 | Q3-Q4 2024 |
| **v1.0** | 正式发布 | Q1 2025 |

---

## IX. 参考资源

- [COMPILER_ARCHITECTURE.md](../COMPILER_ARCHITECTURE.md) - 编译器整体架构
- [x-lir/docs/](./x-lir/) - LIR 格式说明
- 各后端的官方文档：
  - [Zig 文档](https://ziglang.org/documentation/)
  - [LLVM 文档](https://llvm.org/docs/)
  - [TypeScript 文档](https://www.typescriptlang.org/docs/)
  - [Java 文档](https://docs.oracle.com/en/java/)
  - [Python 文档](https://docs.python.org/)
  - [C# 文档](https://docs.microsoft.com/en-us/dotnet/csharp/)
  - [Swift 文档](https://swift.org/documentation/)
  - [Rust 文档](https://www.rust-lang.org/documentation.html)
  - [Erlang/OTP 文档](https://www.erlang.org/doc/)

---

## X. 常见问题

### Q1: 为什么选择这十个后端？

**A**: 这十个后端覆盖了主流开发领域：
- 原生高性能（LLVM、Zig、Rust、Native）
- Web/JavaScript（TypeScript）
- 企业级（Java、C#）
- 数据科学（Python）
- 系统编程（Rust、Zig、C）
- 函数式/并发（Erlang）
- Apple 生态（Swift）

### Q2: 为什么 Zig 已经成熟，其他还在早期？

**A**: Zig 后端通过源码翻译的方式最早实现，且 Zig 编译器本身成熟稳定。其他后端正在逐步完善。

### Q3: 第二阶段何时开始？

**A**: 在所有十大后端都达到"可用"状态后开始，预计 Q3 2024。

### Q4: 如何贡献新的后端？

**A**: 参考 `x-codegen/docs/guides/adding_backend.md` 和后端实现模板。

---

**最后更新**：2024 年 1 月  
**维护者**：X 语言核心团队