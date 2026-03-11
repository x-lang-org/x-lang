# x-hir

X语言高级中间表示（High-Level Intermediate Representation）。

## 功能定位

- 编译器的中间表示层
- 位于AST和代码生成之间
- 提供语义分析后的规范化表示
- 便于后续的优化和代码生成

## 依赖关系

**外部依赖**：
- im = "15.1" - 不可变数据结构
- thiserror = "1.0" - 错误处理
- log = "0.4" - 日志记录

**内部依赖**：
- x-parser = { workspace = true } - 语法分析器（获取AST）
- x-typechecker = { workspace = true } - 类型检查器（类型信息）

## 主要结构

### 核心类型

1. **Hir** - 高级中间表示根结构
   - 极简实现：目前为桩代码
   - 实际应包含模块、声明、语句、表达式等

2. **HirError** - HIR转换错误
   - 转换错误：包含消息文本

## 使用方法

```rust
use x_hir::ast_to_hir;
use x_parser::parse_program;

let source = "let x = 42;";
let ast = parse_program(source).expect("Parsing failed");

match ast_to_hir(&ast) {
    Ok(hir) => println!("HIR: {:?}", hir),
    Err(e) => eprintln!("Error: {}", e),
}
```

## 实现状态

**已实现功能**：
- 基本Hir类型定义
- 简化的AST到HIR转换函数
- 错误类型定义

**待实现功能**：
- 完整的Hir结构定义
- 语义分析和规范化
- 类型检查结果的整合
- 优化机会识别
- 完善的错误处理
- 与Perceus内存管理的集成

## 架构设计

### 设计目标

1. **规范化**：消除语法差异，提供标准化表示
2. **类型信息**：包含完整的类型信息
3. **优化友好**：便于后续优化（如常量传播、死代码消除）
4. **代码生成友好**：简化多后端代码生成
5. **语义保留**：保持与源代码的语义一致性

### 计划结构

```
Hir {
    modules: Vec<Module>,
    declarations: Vec<Declaration>,
    statements: Vec<Statement>,
    expressions: Vec<Expression>,
    type_env: TypeEnvironment,
}
```

## 测试覆盖

无（目前为桩代码）。

## Testing & Verification

### 最小验证（只验证本 crate）

```bash
cd compiler
cargo test -p x-hir
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

```bash
cd compiler
cargo llvm-cov -p x-hir --tests --lcov --output-path target/coverage/x-hir.lcov
```

### 集成验证（与下游联动）

```bash
cd compiler
cargo test -p x-perceus
```

## 代码生成后端支持

所有后端最终将依赖HIR而非AST。

## 未来规划

1. 完善Hir结构
2. 实现完整的AST到Hir转换
3. 集成类型检查结果
4. 支持增量编译
5. 实现基本优化 passes
