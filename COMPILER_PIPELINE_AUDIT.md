# X 语言编译器流水线审计报告

## 执行摘要

用户要求编译器流水线必须严格遵循以下阶段顺序：

**词法分析 → 语法分析 → 语义分析 → HIR → MIR → LIR → 目标后端 → 最终可运行产物**

本报告对当前实现进行了全面审计，发现了几个**关键问题**需要立即整改。

---

## 1. 现状分析

### 1.1 当前编译流水线（`tools/x-cli/src/pipeline.rs`）

```rust
pub fn run_pipeline(source: &str) -> Result<PipelineOutput, String> {
    // 阶段1: 词法分析（隐式，在解析器内部）
    let parser = x_parser::parser::XParser::new();
    
    // 阶段2: 语法分析 ✅
    let mut ast = parser.parse(source)?;
    
    // 自动导入标准库 prelude
    let prelude_decls = parse_std_prelude()?;
    ast.declarations = [prelude_decls, ast.declarations].concat();
    
    // 阶段3: 语义分析（类型检查）✅
    type_check_with_big_stack(&ast)?;
    
    // 阶段4: HIR 生成 ✅
    let hir = x_hir::ast_to_hir(&ast)?;
    
    // 阶段5: MIR 生成 ✅
    let mir = x_mir::lower_hir_to_mir(&hir)?;
    
    // 阶段6: LIR 生成 ✅
    let lir = x_lir::lower_mir_to_lir(&mir)?;
    
    Ok(PipelineOutput { ast, hir, mir, lir })
}
```

### 1.2 阶段映射表

| # | 要求阶段 | 当前实现 | 状态 | 备注 |
|----|---------|--------|------|------|
| 1 | 词法分析 | `x-lexer` | ✅ 完成 | 已在 x-parser 内部完整实现 |
| 2 | 语法分析 | `x-parser` | ✅ 完成 | 生成 AST |
| 3 | 语义分析 | `x-typechecker` | ✅ 完成 | 类型检查 |
| 4 | HIR | `x-hir` | ✅ 完成 | AST → HIR 降级 |
| 5 | MIR | `x-mir` | ✅ 完成 | HIR → MIR 降级 + Perceus 分析 |
| 6 | LIR | `x-lir` | ✅ 完成 | MIR → LIR 降级 |
| 7 | 目标后端 | `x-codegen*` | ⚠️ 问题 | **见下文** |
| 8 | 可运行产物 | `x-codegen-zig` | ⚠️ 问题 | **见下文** |

---

## 2. 发现的问题

### 问题 A: 代码生成后端**绕过了完整流水线**

**当前 `tools/x-cli/src/commands/compile.rs` 的实现:**

```rust
pub fn exec(file: &str, output: Option<&str>, ...) -> Result<(), String> {
    // ❌ 问题：直接使用 AST 生成代码，跳过了 HIR/MIR/LIR
    let program = parser.parse(&content)?;
    
    // ...类型检查...
    
    // ❌ 直接从 AST 生成 Zig 代码（未经过 MIR/LIR）
    let mut backend = x_codegen::zig_backend::ZigBackend::new(...);
    let codegen_output = backend.generate_from_ast(&program)?;
    
    // 编译 Zig 代码...
}
```

**问题分析：**
- ❌ `ZigBackend::generate_from_ast()` 直接从 AST 生成代码
- ❌ 未使用 `run_pipeline()` 中已生成的 HIR/MIR/LIR
- ❌ Perceus 内存管理分析（在 MIR 中）被完全跳过
- ❌ LIR 中的优化机制未被利用

**影响范围：**
- Zig 后端（当前实现最成熟）
- JavaScript 后端 `generate_from_ast()`
- JVM 后端 `generate_from_ast()`
- .NET 后端 `generate_from_ast()`

### 问题 B: 后端设计与流水线脱节

**当前后端实现现状：**

```rust
// ✅ 部分后端有多个代码生成入口
impl ZigBackend {
    pub fn generate_from_ast(&mut self, program: &AstProgram) -> Result<...> { ... }
    pub fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> Result<...> { ... }
    pub fn generate_from_pir(&mut self, pir: &x_mir::PerceusIR) -> Result<...> { ... }
    // ❌ 缺少 generate_from_lir()
}
```

**问题：**
- ✅ 有条件支持 HIR 和 MIR 的代码生成
- ❌ **完全缺少 LIR → 代码生成的入口**
- ❌ 无法利用 LIR 中的优化信息
- ❌ 没有统一的代码生成接口规范

### 问题 C: `emit` 选项实现不完整

在 `commands/compile.rs` 的 `emit_stage()` 函数中：

```rust
fn emit_stage(file: &str, content: &str, stage: &str) -> Result<(), String> {
    match stage {
        "tokens" => { /* ✅ 实现 */ }
        "ast" => { /* ✅ 实现 */ }
        "zig" => { /* ✅ 实现 */ }
        "dotnet" | "csharp" => { /* ⚠️ 部分实现 */ }
        // ❌ 缺失: hir, mir, lir 的输出选项
    }
}
```

---

## 3. 合规性评分

### 总体评分：**65/100**

| 评估项 | 得分 | 备注 |
|--------|------|------|
| 词法分析阶段 | 20/20 | ✅ 完整实现 |
| 语法分析阶段 | 20/20 | ✅ 完整实现 |
| 语义分析阶段 | 15/20 | ✅ 实现，但无错误恢复 |
| HIR 阶段 | 10/10 | ✅ 完整实现 |
| MIR 阶段 | 10/10 | ✅ 完整实现 + Perceus |
| LIR 阶段 | 10/10 | ✅ 完整实现 |
| **后端阶段** | **0/10** | ❌ **绕过流水线** |
| 可运行产物 | 0/10 | ❌ 依赖后端绕过 |
| 文档与测试 | 0/10 | ❌ 流水线文档缺失 |

---

## 4. 必要的整改方案

### 整改方案 A: 统一代码生成接口

**新建文件:** `compiler/x-codegen/src/unified_codegen.rs`

```rust
/// 统一的代码生成接口
/// 所有后端必须实现此特征
pub trait CodeGenerator {
    /// 从 LIR 生成代码（标准入口）
    fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String>;
}

impl CodeGenerator for ZigBackend {
    fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String> {
        // 根据 LIR 生成优化的 Zig 代码
        todo!()
    }
}

// 为其他后端实现同样的接口
```

**要求：**
- ✅ 所有后端必须实现 `generate_from_lir()`
- ✅ 禁止使用 `generate_from_ast()`（仅用于 `--emit` 调试）
- ✅ 后端内部可以：访问完整的控制流信息（来自 LIR）、应用内存优化（MIR 的 Perceus 分析结果在 LIR 中体现）、进行平台特定优化

### 整改方案 B: 修复编译命令流水线

**修改:** `tools/x-cli/src/commands/compile.rs`

```rust
pub fn exec(file: &str, output: Option<&str>, ...) -> Result<(), String> {
    let content = std::fs::read_to_string(file)?;
    
    // 使用完整的编译流水线
    let pipeline_output = pipeline::run_pipeline(&content)?;
    
    // 现在 pipeline_output 包含: AST, HIR, MIR, LIR
    let lir = &pipeline_output.lir;
    
    // 从 LIR 生成代码（不再是从 AST）
    let mut backend = x_codegen::zig_backend::ZigBackend::new(...);
    let codegen_output = backend.generate_from_lir(lir)?;
    
    // ... 编译和链接 ...
}
```

**要求：**
- ✅ 所有编译路径（`compile`、`build` 等）必须经过 `run_pipeline()`
- ✅ 后端直接接收 LIR，而不是 AST
- ✅ 移除所有 `generate_from_ast()` 的直接调用（除了 `--emit ast` 调试）

### 整改方案 C: 完整的 `--emit` 调试选项

**修改:** `tools/x-cli/src/commands/compile.rs`

```rust
fn emit_stage(source: &str, stage: &str) -> Result<(), String> {
    match stage {
        "tokens" => { /* 词法分析输出 */ }
        "ast" => { /* 语法分析输出 */ }
        "hir" => { /* HIR 输出 */ }
        "mir" => { /* MIR 输出 */ }
        "lir" => { /* LIR 输出 */ }
        "zig" | "llvm" | "js" | "jvm" | "dotnet" => { /* 各后端代码输出 */ }
        _ => Err(format!("未知阶段: {}", stage))
    }
}
```

**命令示例：**
```bash
x compile hello.x --emit tokens      # 词法分析输出
x compile hello.x --emit ast         # 语法树输出
x compile hello.x --emit hir         # HIR 输出
x compile hello.x --emit mir         # MIR 输出
x compile hello.x --emit lir         # LIR 输出
x compile hello.x --emit zig         # Zig 代码输出
```

---

## 5. 实施优先级

### Phase 1: 核心修复（高优先级）

1. **创建统一代码生成接口**
   - 文件：`compiler/x-codegen/src/unified_codegen.rs`
   - 定义 `CodeGenerator` trait，所有后端实现 `generate_from_lir()`
   - 工作量：2-3 天

2. **修复 Zig 后端实现**
   - 在 `ZigBackend` 中实现 `generate_from_lir()`
   - 从 LIR 到 Zig 代码的降级（而不是从 AST）
   - 工作量：3-5 天

3. **修复编译命令**
   - 修改 `tools/x-cli/src/commands/compile.rs`
   - 强制使用 `run_pipeline()` + `generate_from_lir()`
   - 工作量：1 天

### Phase 2: 其他后端适配（中优先级）

4. **适配 JavaScript 后端**
   - 实现 `generate_from_lir()`
   - 工作量：2-3 天

5. **适配 JVM 后端**
   - 实现 `generate_from_lir()`
   - 工作量：2-3 天

6. **适配 .NET 后端**
   - 实现 `generate_from_lir()`
   - 工作量：2-3 天

### Phase 3: 调试与测试（低优先级）

7. **完整的 `--emit` 输出**
   - 添加 `--emit hir`、`--emit mir`、`--emit lir`
   - 工作量：1 天

8. **流水线文档与测试**
   - 编写编译器流水线规范
   - 添加单元测试覆盖完整流水线
   - 工作量：2-3 天

---

## 6. 规范性要求

### 6.1 流水线阶段定义

每个编译阶段必须：

1. **有清晰的输入与输出类型**
   ```rust
   // ✅ 良好的设计
   pub fn lower_hir_to_mir(hir: &Hir) -> Result<MirModule, Error>
   pub fn lower_mir_to_lir(mir: &MirModule) -> Result<Program, Error>
   ```

2. **不产生副作用**
   - 不能写文件、不能调用外部工具
   - 所有 I/O 由 CLI 层处理

3. **支持完整的错误追踪**
   - 错误信息必须包含源代码位置（span）
   - 支持多错误收集

4. **提供标准化的输出函数**
   ```rust
   // ✅ 调试辅助函数
   pub fn mir_to_string(mir: &MirModule) -> String
   pub fn lir_to_string(lir: &Program) -> String
   ```

### 6.2 代码生成后端规范

每个后端必须：

1. **实现统一接口**
   ```rust
   impl CodeGenerator for ZigBackend {
       fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput, String>
   }
   ```

2. **严格遵循流水线**
   - 仅接收 LIR 作为输入
   - 禁止直接访问 AST
   - 利用 MIR 中的 Perceus 分析结果

3. **支持多目标配置**
   ```rust
   pub struct BackendConfig {
       pub target: TargetTriple,
       pub optimize_level: OptimizationLevel,
       pub debug_info: bool,
   }
   ```

4. **返回标准化输出**
   ```rust
   pub struct CodegenOutput {
       pub files: Vec<OutputFile>,
       pub diagnostics: Vec<Diagnostic>,
   }
   ```

---

## 7. 检查清单

**在宣称"编译器流水线合规"前，必须验证：**

- [ ] `run_pipeline()` 在源代码 → LIR 的完整路径上运行无误
- [ ] 所有后端都实现了 `CodeGenerator::generate_from_lir()`
- [ ] `compile` 命令的代码生成路径完全使用 LIR（未绕过）
- [ ] `--emit hir`, `--emit mir`, `--emit lir` 都能正确输出
- [ ] 存在单元测试验证完整流水线（包括各阶段的往返对称性）
- [ ] 编译器不会因为跳过某些中间阶段而崩溃
- [ ] 所有后端生成的代码都来自相同的 LIR 输入
- [ ] 流水线的性能在可接受范围内（无明显退化）
- [ ] 错误诊断信息正确追踪到原始源代码

---

## 8. 参考文档

- [DESIGN_GOALS.md](../DESIGN_GOALS.md) - 设计最高准则（第13条：多后端架构）
- [CLAUDE.md](../CLAUDE.md) - 编译器架构说明
- `compiler/Cargo.toml` - 当前 crate 结构
- `tools/x-cli/src/pipeline.rs` - 当前流水线实现

---

## 总结

**当前状态：** 流水线架构 70% 完成，但代码生成后端绕过了 MIR/LIR，造成设计目标无法实现。

**关键整改：** 
1. 创建统一代码生成接口
2. 所有后端从 LIR 生成（而不是 AST）
3. 修复编译命令使用完整流水线
4. 添加完整的调试选项支持

**预计工作量：** 2-3 周（按优先级分阶段实施）

**收益：** 
- ✅ 符合设计目标（多后端统一中间表示）
- ✅ 启用 Perceus 内存优化
- ✅ 支持平台无关的编译器优化
- ✅ 为 LSP、增量编译等高级功能铺平道路