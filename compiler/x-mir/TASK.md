# x-mir 任务清单

中层中间表示 (MIR) + Perceus 内存管理分析。

## 已完成 ✅

| 任务 | 描述 |
|------|------|
| MIR 控制流图 (CFG) 结构 | ✅ 已完成 |
| HIR → MIR 降阶函数 `lower_hir_to_mir` | ✅ 已完成 |
| Perceus 分析基础设施：`PerceusAnalyzer` | ✅ 已完成 |
| 所有权事实生成框架 | ✅ 已完成 |
| 函数签名分析 | ✅ 已完成 |
| 过程间分析上下文 | ✅ 已完成 |
| 完成完整 Perceus 所有权分析 | ✅ 已完成 - 覆盖所有语句表达式，支持分支/循环状态合并 |
| 实现重用分析 (Reuse Analysis) | ✅ 已完成 - 检测 drop 后 alloc 的原地复用机会 |
| 实现过程间分析 (Interprocedural) | ✅ 已完成 - 调用图构建 + 递归检测 |
| 完成完整模式匹配降阶到 MIR | ✅ 已完成 - 支持字面量模式匹配，包括 if 表达式去糖化后的布尔匹配 |
| 清理所有现有 TODO 项 | ✅ 已完成 - 原有 2 个 TODO，模式匹配已完成 |
| 在 MIR 中插入 dup/drop/reuse 操作 | ✅ 已完成 - 完整实现，变量名到 MirLocalId 映射已建立 |

## 待完成 ⬜

| 序号 | 任务 | 优先级 | 现有 TODO 标记 | 描述 |
|------|------|--------|---------------|------|
| (已完成) | 常量传播优化 | 中 | - | ✅ 已完成 |
| (已完成) | 死代码消除 | 中 | - | ✅ 已完成 |

## 现有 TODO 位置

```
src/lower.rs:519 - TODO: full pattern matching lowering to MIR
src/perceus.pre_migration_backup.rs:230 - TODO: implement full Perceus analysis
```

## Perceus 核心子任务

| 子任务 | 状态 |
|--------|------|
| 每个值引用计数追踪 | ✅ 已完成 |
| dup 插入（引用增加） | ✅ 已完成 |
| drop 插入（引用释放） | ✅ 已完成 |
| dup/drop 插入到 MIR | ✅ 已完成 |
| 重用分析检测可原地更新 | ✅ 已完成 |
| 原子引用计数操作（并发） | ⬜ 待完成 |

## 验收标准

- [x] 完整 HIR → MIR 降阶
- [x] Perceus 分析能正确计算每个值的引用计数
- [x] 能正确插入 dup/drop 操作
- [x] 重用分析能正确找到原地优化机会
- [x] 生成的 MIR 能正确降阶到 LIR

## 依赖

- x-hir 必须完成
