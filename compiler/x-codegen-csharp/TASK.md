# x-codegen-csharp 任务清单

C# 后端 - 生成 C# 源代码，供 .NET CLR 运行。

## 已完成 ✅

| 任务 | 状态 |
|------|------|
| 框架结构完成 | ✅ 已完成 |
| 基本类型映射框架 | ✅ 已完成 |
| **LIR → C# 代码生成** | ✅ 已完成 (2026-03-29) |

## 完成度

**85%** - generate_from_lir 已实现

## 已实现功能

- [x] CodeGenerator trait 实现
- [x] LIR 函数 → C# 方法
- [x] LIR 变量 → C# 变量
- [x] LIR 控制流 → C# if/while
- [x] LIR 字面量 (Integer, Float, String, Char, Bool)
- [x] LIR 二元运算符 (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, &, |, ^, <<, >>)
- [x] LIR 一元运算符 (-, !, ~)
- [x] LIR 表达式 (Call, Member, Index)
- [x] 所有现有单元测试通过

## 待完成 ⬜

| 序号 | 任务 | 优先级 | 描述 |
|------|------|--------|------|
| 1 | For 循环支持 | 中 | For 循环语句 |
| 2 | Match 模式匹配支持 | 中 | Match 语句 |
| 3 | 支持多个 .NET 平台 | 低 | .NET Framework/Core/5+ |
| 4 | Perceus 引用计数集成 | 中 | 转为 .NET GC |

## 验收标准

- [x] generate_from_lir 方法已实现
- [x] 单元测试通过
- [ ] 生成的 C# 代码能被 `dotnet build` 编译
- [ ] CLR 中能正确运行

## 依赖

- x-lir ✅ 已完成
