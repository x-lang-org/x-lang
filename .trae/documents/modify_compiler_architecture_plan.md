# 修改 X 语言编译器架构文档计划

## 任务概述

修改 `c:\Users\Administrator\Documents\x-lang\docs\src\compiler-architecture.md` 文档，实现以下变更：

1. 移除 C23 后端，统一使用 Zig
2. Native 和 Wasm 由 Zig 提供
3. 其他 C 语言实现统一使用 Zig 实现
4. 新增 Python，直接编译为 Python 源码
5. 移除 JavaScript 和 TypeScript，由 Wasm 提供
6. 保留 JVM 字节码和 .NET 字节码

## 具体修改步骤

### 步骤 1: 更新总体架构图

修改文档开头的 Mermaid 流程图：

- 移除 `C23[C23 后端]`
- 添加 `Zig[Zig 后端]`
- 保留 `LLVM[LLVM 后端]`
- 移除 `JSRuntime[JavaScript 后端]`
- 更新 `Native` 和 `Wasm` 的来源，改为来自 Zig

### 步骤 2: 更新 AOT 编译管道

修改 AOT 配置部分：

- 枚举 `Target` 中移除 `C`
- 添加 `Zig` 到原生目标
- 移除 `JavaScript` 和 `TypeScript` 源码目标

### 步骤 3: 更新 JIT 编译架构

修改 JIT 核心设计部分：

- 从平台后端移除 `JS[JavaScript 后端]`
- 更新 `JitTarget` 枚举，移除 `JavaScript`

### 步骤 4: 更新 Crate 组织结构

修改 crate 列表：

- 移除 `x-codegen-c` (C23 AOT 后端)
- 保留 `x-codegen-llvm` (LLVM AOT 后端)
- 新增 `x-codegen-zig` (Zig AOT 后端)
- 移除 `x-codegen-js` (JavaScript AOT 后端)
- 保留 `x-codegen-jvm` (JVM AOT 后端)
- 保留 `x-codegen-dotnet` (.NET AOT 后端)
- 更新 `x-runtime-js` 相关内容为 Wasm

### 步骤 5: 更新 JIT 后端

修改 JIT 后端部分：

- 移除 JavaScript 后端实现
- 保留 JVM 后端实现
- 保留 CLR (.NET) 后端实现
- 保留 Python 后端实现

### 步骤 6: 更新 CLI 扩展

修改 CLI 命令示例：

- 移除 `--target js` 相关命令
- 添加 `--target wasm` 命令
- 保留 `--target jvm` 和 dotnet 相关命令

### 步骤 7: 更新实现阶段

修改 Phase 2：

- 移除 JavaScript JIT 后端相关任务
- 将 Wasm 后端提前，使用 Zig 实现

## 需要修改的文件

- `c:\Users\Administrator\Documents\x-lang\docs\src\compiler-architecture.md`

## 预期结果

修改后的架构应该支持：

- **AOT 后端**：
  - Zig（Native + Wasm）
  - LLVM（Native）
  - JVM 字节码
  - .NET CIL 字节码
  - Python 源码

- **JIT 后端**：
  - JVM
  - .NET CLR
  - Python
  - Wasm（由 Zig 提供）

- **移除的内容**：
  - C23 后端
  - JavaScript/TypeScript 后端
