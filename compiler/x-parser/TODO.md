# x-parser TODO

## 当前状态

- 基本功能已实现
- 支持变量和函数声明
- 支持基本语句（if, while, for）
- 支持复杂表达式解析
- 提供完整的位置信息（Span）
- **新增**: 支持类和接口声明的完整修饰符解析

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
- [x] 完善类声明解析（支持 abstract/final 修饰符、字段可见性、方法修饰符）
- [x] 完善接口声明解析（支持 extends 继承）
- [x] 完善模块声明解析（顶级 `module <name>;`）
- [ ] 完善类型别名解析
- [ ] 支持泛型声明

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

**整体完成度**：约 90%
- 基本功能：95%（函数/变量/语句/表达式解析完整）
- 高级功能：90%（match/try/module/export/类/接口已完成；泛型仍待）
- 错误处理：70%（错误携带 span，多错误恢复待完善）
- 测试覆盖：65%（覆盖核心语法分支）

**当前测试覆盖**：本 crate 含 18 个单元测试，`cargo test -p x-parser` 全绿。

**已实现功能清单**：
- ✅ 程序解析（声明 + 语句）
- ✅ 变量声明（let/const/var，含可见性修饰符）
- ✅ 函数声明（含类型注解、参数、方法修饰符）
- ✅ 类声明（含 abstract/final 修饰符、extends/implements、字段、方法、构造函数）
- ✅ 接口声明（含 extends 继承、方法签名）
- ✅ 控制流语句（if/while/for/do-while/break/continue）
- ✅ Match 语句（模式匹配 + when guard + or-pattern）
- ✅ Try 语句（try/catch/finally）
- ✅ 模块声明（`module <name>;`）
- ✅ 导入声明（`import ...;`）
- ✅ 导出声明（`export <symbol>;`）
- ✅ 表达式解析（优先级攀爬算法）
- ✅ Lambda 表达式
- ✅ 数组/字典字面量
- ✅ 记录表达式
- ✅ 类型注解解析
