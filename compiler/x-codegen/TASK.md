# x-codegen 任务清单

代码生成核心 - 公共接口和工具函数。

## 已完成 ✅

| 任务 | 状态 |
|------|------|
| `CodeGenerator` trait 定义 | ✅ 已完成 |
| `Target` 枚举（所有支持的后端） | ✅ 已完成 |
| `CodeGenConfig` 和 `CodegenOutput` | ✅ 已完成 |
| 错误类型定义 | ✅ 已完成 |
| 工具模块：buffer/escape/operators/symbols | ✅ 已完成 |
| XIR（旧 IR）兼容保留 | ✅ 已完成 |

## 待完成 ⬜

目前核心基础设施已完成，没有重大待完成任务。等待各后端实现需求。

## 验收标准

- [ ] 所有后端都能实现 CodeGenerator trait
- [ ] 工具函数满足所有后端需求

## 依赖

- x-lir 必须完成
