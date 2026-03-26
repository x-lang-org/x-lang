# X 语言编译器流水线整改 TODO

> 本文档根据 [COMPILER_PIPELINE_AUDIT.md](./COMPILER_PIPELINE_AUDIT.md) 的审计结果制定。
> 目标：确保编译器严格遵循完整的编译流水线，从源代码到可运行产物的完整链路。

**当前合规性评分：65/100**

---

## 📋 执行摘要

### 核心问题

1. ❌ **代码生成后端绕过了完整流水线**
   - `ZigBackend` 直接使用 AST 生成代码
   - 跳过了 HIR、MIR、LIR 中的优化与内存管理分析
   - 影响所有后端（Zig、JavaScript、JVM、.NET）

2. ❌ **后端设计与流水线脱节**
   - 缺少 LIR → 代码生成的入口
   - 无法利用 MIR 中的 Perceus 内存管理分析结果

3. ❌ **`--emit` 调试选项不完整**
   - 缺少 `--emit hir`、`--emit mir`、`--emit lir`
   - 无法逐阶段观察编译过程

### 整改优先级

| 阶段 | 项目 | 优先级 | 工作量 | 预期完成 |
|------|------|--------|--------|---------|
| Phase 1 | 创建统一代码生成接口 | ⭐⭐⭐ 高 | 2-3 天 | 第 1 周 |
| Phase 1 | 修复 Zig 后端实现 | ⭐⭐⭐ 高 | 3-5 天 | 第 2 周 |
| Phase 1 | 修复编译命令流水线 | ⭐⭐⭐ 高 | 1 天 | 第 2 周 |
| Phase 2 | 适配其他后端 | ⭐⭐ 中 | 2-3 天/个 | 第 3-4 周 |
| Phase 3 | 完整的 `--emit` 输出 | ⭐ 低 | 1 天 | 第 4 周 |
| Phase 3 | 流水线文档与测试 | ⭐ 低 | 2-3 天 | 第 4-5 周 |

**总预计工作量：2-3 周**

---

## 🎯 Phase 1: 核心修复（高优先级）

### Task 1.1: 创建统一代码生成接口

**文件位置**: `compiler/x-codegen/src/unified_codegen.rs` (新建)

**目标**: 定义所有后端必须实现的统一接口

**待做项**:

- [ ] 定义 `CodeGenerator` trait
  - [ ] `fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String>`
  - [ ] 支持 target 配置（native、wasm、wasm32-freestanding）
  - [ ] 支持优化级别配置（debug、release）

- [ ] 定义 `CodegenOutput` 结构体
  - [ ] `files: Vec<OutputFile>` - 生成的源文件
  - [ ] `diagnostics: Vec<Diagnostic>` - 编译诊断信息
  - [ ] 实现 Debug、Display traits

- [ ] 定义 `OutputFile` 结构体
  - [ ] `name: String` - 文件名
  - [ ] `content: Vec<u8>` - 文件内容
  - [ ] `language: Language` - 目标语言

- [ ] 在 `x-codegen/src/lib.rs` 中导出
  ```rust
  pub use unified_codegen::{CodeGenerator, CodegenOutput, OutputFile};
  ```

- [ ] 编写单元测试
  - [ ] 测试 trait 定义的有效性
  - [ ] 测试 OutputFile 的序列化

**验收标准**:
- ✅ trait 定义清晰，无歧义
- ✅ 所有字段都有文档注释
- ✅ 单元测试覆盖率 > 80%

---

### Task 1.2: 修复 Zig 后端实现

**文件位置**: `compiler/x-codegen/src/zig_backend.rs`

**目标**: 实现 `generate_from_lir()` 方法，从 LIR 而非 AST 生成代码

**待做项**:

- [ ] 分析当前 `generate_from_ast()` 的实现
  - [ ] 理解 AST → Zig 的映射规则
  - [ ] 识别哪些信息来自 AST，哪些来自 MIR/LIR

- [ ] 实现 `CodeGenerator` trait
  ```rust
  impl CodeGenerator for ZigBackend {
      fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String> {
          // 从 LIR 生成 Zig 代码
          todo!()
      }
  }
  ```

- [ ] 映射 LIR → Zig 语言特性
  - [ ] LIR 函数 → Zig 函数
  - [ ] LIR 变量 → Zig 变量声明
  - [ ] LIR 控制流 → Zig if/while
  - [ ] LIR 内存操作（dup/drop） → Zig 对应实现

- [ ] 利用 Perceus 优化信息
  - [ ] LIR 中已包含 dup/drop 操作
  - [ ] 生成高效的内存管理代码
  - [ ] 避免不必要的内存复制

- [ ] 处理多平台目标
  - [ ] native 编译
  - [ ] wasm 编译（需要 Zig 支持）
  - [ ] wasm32-freestanding 编译

- [ ] 编写单元测试
  - [ ] 测试简单函数的 LIR → Zig 转换
  - [ ] 测试变量与内存操作
  - [ ] 测试控制流结构
  - [ ] 测试与旧 `generate_from_ast()` 的行为对比

**验收标准**:
- ✅ 实现完整的 `generate_from_lir()` 方法
- ✅ 所有单元测试通过
- ✅ 生成的 Zig 代码能被 Zig 编译器接受
- ✅ 编译结果与原 AST 方式的行为一致

**参考资源**:
- `compiler/x-lir/src/lir.rs` - LIR 数据结构定义
- `compiler/x-codegen/src/zig_backend.rs` - 当前实现

---

### Task 1.3: 修复编译命令流水线

**文件位置**: `tools/x-cli/src/commands/compile.rs`

**目标**: 确保 `compile` 命令使用完整的编译流水线（LIR 作为后端输入）

**待做项**:

- [ ] 修改 `exec()` 函数
  ```rust
  pub fn exec(...) -> Result<(), String> {
      let content = std::fs::read_to_string(file)?;
      
      // ✅ 使用完整流水线
      let pipeline_output = pipeline::run_pipeline(&content)?;
      
      // ✅ 从 LIR 生成代码
      let mut backend = ZigBackend::new(config);
      let codegen_output = backend.generate_from_lir(&pipeline_output.lir)?;
      
      // ✅ 编译与链接
      // ...
  }
  ```

- [ ] 移除直接调用 `generate_from_ast()`
  - [ ] 搜索所有 `generate_from_ast` 调用
  - [ ] 替换为 `generate_from_lir` 调用
  - [ ] 确保仅在 `--emit ast` 调试时使用 AST

- [ ] 验证 `pipeline::run_pipeline()` 的输出
  - [ ] 确保 `PipelineOutput` 包含 LIR
  - [ ] 检查 LIR 的完整性与正确性

- [ ] 处理编译选项传递
  - [ ] `--target` → BackendConfig
  - [ ] `--release` → OptimizationLevel
  - [ ] `--emit` → 确定是否调试输出

- [ ] 编写集成测试
  - [ ] 测试简单程序的完整编译流程
  - [ ] 验证生成的可执行文件能正确运行
  - [ ] 对比新旧实现的输出（应该一致）

**验收标准**:
- ✅ 编译命令使用完整流水线
- ✅ 所有后端都从 LIR 输入
- ✅ 集成测试通过
- ✅ 编译结果与审计前行为一致

**参考资源**:
- `tools/x-cli/src/pipeline.rs` - 流水线实现
- `tools/x-cli/src/commands/compile.rs` - 当前编译命令

---

## 🚀 Phase 2: 其他后端适配（中优先级）

### Task 2.1: 适配 JavaScript 后端

**文件位置**: `compiler/x-codegen/src/js_backend.rs`

**目标**: 实现 `generate_from_lir()` 方法，从 LIR 生成 JavaScript 代码

**待做项**:

- [ ] 实现 `CodeGenerator` trait
  ```rust
  impl CodeGenerator for JavaScriptBackend {
      fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String> {
          // 从 LIR 生成 JavaScript 代码
          todo!()
      }
  }
  ```

- [ ] 映射 LIR → JavaScript
  - [ ] LIR 函数 → JavaScript 函数
  - [ ] LIR 变量 → JavaScript 变量声明
  - [ ] LIR 控制流 → JavaScript if/while
  - [ ] LIR 内存操作 → JavaScript 对象/引用计数管理

- [ ] 支持多种 JavaScript 环境
  - [ ] Node.js
  - [ ] 浏览器（ES6+）
  - [ ] 服务端（Node.js）

- [ ] 编写单元测试
  - [ ] 测试基本功能的转换
  - [ ] 测试生成的代码能在 Node.js 中运行
  - [ ] 测试输出与 Zig 后端的行为一致

**验收标准**:
- ✅ 实现完整的 `generate_from_lir()` 方法
- ✅ 单元测试通过
- ✅ 生成的 JavaScript 代码能被 Node.js 执行
- ✅ 行为与 Zig 后端一致

---

### Task 2.2: 适配 JVM 后端

**文件位置**: `compiler/x-codegen-jvm/src/lib.rs`

**目标**: 实现 `generate_from_lir()` 方法，从 LIR 生成 JVM 字节码

**待做项**:

- [ ] 实现 `CodeGenerator` trait
  ```rust
  impl CodeGenerator for JvmBackend {
      fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String> {
          // 从 LIR 生成 JVM 字节码
          todo!()
      }
  }
  ```

- [ ] 映射 LIR → JVM 字节码
  - [ ] LIR 函数 → JVM 方法
  - [ ] LIR 变量 → JVM 本地变量
  - [ ] LIR 控制流 → JVM 条件跳转指令
  - [ ] LIR 内存操作 → JVM GC（或自定义引用计数）

- [ ] 生成可执行的 JAR 文件
  - [ ] 创建类文件（.class）
  - [ ] 打包为 JAR
  - [ ] 包含 Main 类

- [ ] 编写单元测试
  - [ ] 测试基本功能的转换
  - [ ] 测试生成的 JAR 能在 JVM 中运行
  - [ ] 测试输出与 Zig 后端的行为一致

**验收标准**:
- ✅ 实现完整的 `generate_from_lir()` 方法
- ✅ 单元测试通过
- ✅ 生成的 JAR 文件能被 JVM 执行
- ✅ 行为与 Zig 后端一致

---

### Task 2.3: 适配 .NET 后端

**文件位置**: `compiler/x-codegen-dotnet/src/lib.rs`

**目标**: 实现 `generate_from_lir()` 方法，从 LIR 生成 C# 代码或 .NET IL

**待做项**:

- [ ] 实现 `CodeGenerator` trait
  ```rust
  impl CodeGenerator for DotNetBackend {
      fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String> {
          // 从 LIR 生成 .NET 代码
          todo!()
      }
  }
  ```

- [ ] 映射 LIR → C# / IL
  - [ ] LIR 函数 → C# 方法
  - [ ] LIR 变量 → C# 变量
  - [ ] LIR 控制流 → C# if/while
  - [ ] LIR 内存操作 → C# 对象引用（或 unmanaged 代码）

- [ ] 支持多个 .NET 平台
  - [ ] .NET Framework
  - [ ] .NET Core
  - [ ] .NET 5+

- [ ] 编写单元测试
  - [ ] 测试基本功能的转换
  - [ ] 测试生成的 C# 代码能编译并运行
  - [ ] 测试输出与 Zig 后端的行为一致

**验收标准**:
- ✅ 实现完整的 `generate_from_lir()` 方法
- ✅ 单元测试通过
- ✅ 生成的代码能被 .NET 编译器接受并执行
- ✅ 行为与 Zig 后端一致

---

## 📊 Phase 3: 调试与测试（低优先级）

### Task 3.1: 完整的 `--emit` 输出

**文件位置**: `tools/x-cli/src/commands/compile.rs`

**目标**: 添加 `--emit hir`, `--emit mir`, `--emit lir` 选项，用于逐阶段调试

**待做项**:

- [ ] 修改 `emit_stage()` 函数
  ```rust
  fn emit_stage(source: &str, stage: &str) -> Result<(), String> {
      match stage {
          "tokens" => { /* 现有实现 */ }
          "ast" => { /* 现有实现 */ }
          "hir" => { /* 新增 */ }
          "mir" => { /* 新增 */ }
          "lir" => { /* 新增 */ }
          "zig" | "rust" | "c" | "dotnet" => { /* 现有实现 */ }
          _ => Err(format!("未知阶段: {}", stage))
      }
  }
  ```

- [ ] 实现 HIR 输出
  - [ ] 调用 `pipeline::run_pipeline()`
  - [ ] 输出 `pipeline_output.hir`
  - [ ] 格式化为可读的文本

- [ ] 实现 MIR 输出
  - [ ] 调用 `pipeline::run_pipeline()`
  - [ ] 输出 `pipeline_output.mir`
  - [ ] 格式化为可读的文本
  - [ ] 包含 Perceus 分析信息

- [ ] 实现 LIR 输出
  - [ ] 调用 `pipeline::run_pipeline()`
  - [ ] 输出 `pipeline_output.lir`
  - [ ] 格式化为可读的文本
  - [ ] 显示优化信息

- [ ] 编写测试
  - [ ] 测试各 `--emit` 选项的输出格式
  - [ ] 验证输出的完整性与正确性

**验收标准**:
- ✅ 所有 `--emit` 选项都能正确输出
- ✅ 输出格式清晰可读
- ✅ 用户能通过这些输出调试编译问题

**使用示例**:
```bash
x compile hello.x --emit hir     # 输出 HIR
x compile hello.x --emit mir     # 输出 MIR
x compile hello.x --emit lir     # 输出 LIR
```

---

### Task 3.2: 流水线文档与测试

**文件位置**: 
- `docs/` - 文档目录
- `compiler/x-codegen/tests/` - 集成测试

**目标**: 编写完整的编译器流水线文档与集成测试

**待做项**:

- [ ] 编写流水线文档
  - [ ] 各阶段的职责与数据结构
  - [ ] 数据流转说明
  - [ ] 后端集成指南
  - [ ] 调试指南（如何使用 `--emit` 调试）

- [ ] 编写阶段 → 阶段的转换说明
  - [ ] AST → HIR 的映射规则
  - [ ] HIR → MIR 的映射规则
  - [ ] MIR → LIR 的映射规则
  - [ ] LIR → 代码的映射规则

- [ ] 编写集成测试
  - [ ] 测试完整流水线（source → executable）
  - [ ] 测试各个中间阶段的输出
  - [ ] 测试所有后端的行为一致性
  - [ ] 测试错误处理与诊断

- [ ] 编写性能基准
  - [ ] 测量各阶段的耗时
  - [ ] 识别性能瓶颈
  - [ ] 验证性能无明显退化

- [ ] 编写最佳实践指南
  - [ ] 如何添加新的后端
  - [ ] 如何调试编译问题
  - [ ] 如何优化生成的代码

**验收标准**:
- ✅ 文档清晰、完整
- ✅ 集成测试覆盖全部流水线
- ✅ 所有后端行为一致
- ✅ 性能无明显退化

---

## 🔍 验收清单

**在宣称"编译器流水线合规"前，必须验证：**

### 架构合规性

- [ ] `run_pipeline()` 完整实现源代码 → LIR 的流程
- [ ] 所有后端都实现了 `CodeGenerator::generate_from_lir()`
- [ ] `compile` 命令完全使用 LIR（未绕过）
- [ ] 所有后端生成的代码都来自同一个 LIR 输入
- [ ] 流水线中没有副作用（I/O 由 CLI 层处理）

### 功能完整性

- [ ] `--emit tokens` - 词法分析输出 ✅
- [ ] `--emit ast` - 语法分析输出 ✅
- [ ] `--emit hir` - HIR 输出 ✅
- [ ] `--emit mir` - MIR 输出 ✅
- [ ] `--emit lir` - LIR 输出 ✅
- [ ] `--emit zig` - Zig 代码输出 ✅
- [ ] `--emit rust` - Rust 代码输出 ✅
- [ ] `--emit c` - C 代码输出 ✅
- [ ] `--emit dotnet` - .NET 代码输出 ✅

### 测试覆盖

- [ ] 单元测试覆盖所有阶段转换
- [ ] 集成测试验证完整流水线
- [ ] 各后端生成代码的行为一致性
- [ ] 错误情况下的诊断信息准确
- [ ] 性能基准达到要求

### 质量指标

- [ ] 编译器不会因为跳过中间阶段而崩溃
- [ ] 所有错误诊断都追踪到原始源代码
- [ ] 流水线性能在可接受范围内
- [ ] 代码生成结果的质量达到预期

---

## 📈 进度跟踪

### Phase 1 进度

| Task | 状态 | 完成度 | 备注 |
|------|------|--------|------|
| 1.1 创建统一接口 | ⬜ 未开始 | 0% | 预计 2-3 天 |
| 1.2 修复 Zig 后端 | ⬜ 未开始 | 0% | 预计 3-5 天 |
| 1.3 修复编译命令 | ⬜ 未开始 | 0% | 预计 1 天 |

### Phase 2 进度

| Task | 状态 | 完成度 | 备注 |
|------|------|--------|------|
| 2.1 JS 后端 | ⬜ 未开始 | 0% | 预计 2-3 天 |
| 2.2 JVM 后端 | ⬜ 未开始 | 0% | 预计 2-3 天 |
| 2.3 .NET 后端 | ⬜ 未开始 | 0% | 预计 2-3 天 |

### Phase 3 进度

| Task | 状态 | 完成度 | 备注 |
|------|------|--------|------|
| 3.1 `--emit` 输出 | ⬜ 未开始 | 0% | 预计 1 天 |
| 3.2 文档与测试 | ⬜ 未开始 | 0% | 预计 2-3 天 |

---

## 📚 相关文档

- [COMPILER_PIPELINE_AUDIT.md](./COMPILER_PIPELINE_AUDIT.md) - 完整审计报告
- [CLAUDE.md](./CLAUDE.md) - 编译器开发指南
- [DESIGN_GOALS.md](./DESIGN_GOALS.md) - 设计目标（第13条：多后端架构）
- `compiler/Cargo.toml` - Crate 结构
- `compiler/x-lir/src/lir.rs` - LIR 数据结构定义

---

## 📝 更新历史

| 日期 | 版本 | 更新内容 |
|------|------|---------|
| 2024-XX-XX | 1.0 | 初始版本，基于审计报告制定 |

---

## 🎯 目标

**成功标志**: 将合规性评分从 **65/100** 提升到 **95/100+**

**预期收益**:
- ✅ 符合设计目标（多后端统一中间表示）
- ✅ 启用 Perceus 内存优化
- ✅ 支持平台无关的编译器优化
- ✅ 为 LSP、增量编译等高级功能铺平道路

---

*最后更新：2024*
*负责人：[待指派]*