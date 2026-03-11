# x-parser TODO

## 当前状态

- 基本功能已实现
- 支持变量和函数声明
- 支持基本语句（if, while, for）
- 支持复杂表达式解析
- 提供完整的位置信息（Span）

## 待办事项

### 1. 语句解析
- [x] 完善 Match 语句（模式匹配：statement 级 match + when guard + `a | b` or-pattern）
- [x] 完善 Try 语句（异常处理：try + catch + finally；支持 `catch (Type var)`）
- [ ] 完善 For 循环（更复杂的模式）
- [x] 支持 Do-While 循环
- [x] 支持 Break 和 Continue 语句

### 2. 表达式解析
- [ ] 完善管道操作符解析
- [ ] 完善异步操作（Wait）解析
- [ ] 完善 Effect 操作（Needs/Given）解析
- [ ] 支持更复杂的 lambda 表达式
- [ ] 完善范围表达式
- [ ] 支持模式匹配表达式

### 3. 声明解析
- [ ] 完善类声明解析
- [ ] 完善接口声明解析
- [x] 完善模块声明解析（顶级 `module <name>;`）
- [ ] 完善类型别名解析
- [ ] 支持泛型声明
- [x] 完善导入/导出声明（顶级 `import ...;` 已有；补 `export <symbol>;`）

### 4. 类型系统
- [ ] 完善类型注解解析
- [ ] 支持泛型类型参数
- [ ] 支持类型约束
- [ ] 完善类型定义解析

### 5. 错误处理
- [ ] 优化错误恢复机制
- [ ] 支持多错误报告
- [ ] 提供更详细的错误信息
- [ ] 添加错误代码和分类

### 6. 性能优化
- [ ] 实现增量解析
- [ ] 优化内存分配
- [ ] 支持语法预测
- [ ] 并行解析（如适用）

### 7. 测试覆盖
- [x] 添加单元测试模块（覆盖 module/import/export、match、try/catch/finally）
- [ ] 测试所有边界条件
- [ ] 测试错误场景
- [ ] 性能基准测试

### 8. 其他
- [ ] 完善文档和示例
- [ ] 添加语法验证工具
- [ ] 支持语法高亮
- [ ] 实现 LSP 接口基础

## 质量门禁（可测试与可验证）

### 覆盖率目标

- **行覆盖率**：100%
- **分支覆盖率**：100%
- **测试通过率**：100%

### 必须具备的测试类型

- [x] **单元测试**：覆盖解析入口（`parse_program`）以及关键语法分支（module/import/export、match、try）
- [ ] **错误场景**：对每个 `ParseError` 分支添加最小输入（含 span 校验）
- [ ] **回归用例**：每个修复的解析 bug 都新增一个最小复现测试

### 验收步骤（本地一键验证）

```bash
cd compiler
cargo test -p x-parser

# 覆盖率（line/branch）
cargo llvm-cov -p x-parser --tests
```

## 完成度估计

**整体完成度**：约 70%
- 基本功能：85%
- 高级功能：55%（match/try/module/export 已补；类/接口/更多控制流仍待）
- 错误处理：45%
- 测试覆盖：35%

**当前测试覆盖**：本 crate 含单元测试（`src/lib.rs`），可用 `cargo test -p x-parser` 运行；覆盖率可用 `cargo llvm-cov -p x-parser --tests` 查看。
