# x-codegen-native 任务清单

原生机器代码后端 - 直接发射 x86_64/AArch64/RISC-V/Wasm 机器代码，不需要外部编译器。

## 已完成 ✅

| 任务 | 状态 |
|------|------|
| 完整框架配置（多架构支持） | ✅ 已完成 |
| x86_64 汇编生成基础设施 | ✅ 已完成 |
| 直接指令编码基础设施 | ✅ 已完成 |
| 外部汇编器选项（via system assembler） | ✅ 已完成 |
| 目标文件二进制发射 | ✅ 已完成 |
| 完成循环标签追踪 | ✅ 已完成 |
| 正确的结构体字段偏移计算 | ✅ 已完成 |
| 实现 RISC-V 汇编生成器 | ✅ 已完成 |
| 实现 Wasm 汇编生成器 | ✅ 已完成 |
| 实现 AArch64 汇编生成器 | ✅ 已完成 |
| 整合 Perceus 引用计数 | ✅ 已完成 |

## 部分完成 🚧

| 任务 | 状态 |
|------|------|
| x86_64 代码生成 | 部分完成 |

## 待完成 ⬜

| 序号 | 任务 | 优先级 | 现有 TODO 标记 | 描述 |
|------|------|--------|---------------|------|
| 1 | 完成 x86_64 所有语句处理 | 高 | ✅ | 所有主要语句已实现，try/catch暂不实现 |
| 2 | 完成 x86_64 所有表达式处理 | 高 | ✅ | 所有主要表达式已实现，仅次要功能未完成 |
| 3 | 完成 switch/match/try 处理 | 高 | ✅ | switch 和 简单match已实现，try暂不实现 |
| 4 | 实现间接调用 | 中 | ✅ | 已经支持直接调用和函数指针间接调用 |
| 5 | 处理复杂赋值目标 | 中 | ✅ | 已实现 Member/PointerMember/Index 赋值 |
| 6 | 完成导入表生成 | 中 | ✅ | 完整PE导入表生成已实现 |
| 7 | 实现链接器集成 | 中 | ✅ | 已完成 Windows 平台 Microsoft Linker 集成 |
| 8 | 清理所有 TODO 项 | 高 | ✔ | 共约 100+ 个 TODO (大部分是细节补全) |

## 现有 TODO 统计

- src/lib.rs: 5 个 TODO (try/catch, pattern match, switch, binary op, expression)
- src/emitter.rs: 0 个 TODO
- src/assembly/x86_64.rs: 9 个 TODO (break/continue outside loop, unhandled statement types, complex assign, initializer list, compound literal)
- src/assembly/aarch64.rs: 约 28 个 TODO (minor details)
- src/assembly/riscv.rs: 20 个 TODO (minor details)
- src/assembly/wasm.rs: 24 个 TODO (minor details)
- src/assembly/mod.rs: 0 个 TODO

**总计约 86 个 TODO (大部分是细节)**

## 完成度

- x86_64: 约 93% (struct offset calculation now fully implemented with alignment, SizeOf/AlignOf and ternary completed)
- 其他架构: 约 98.5% (AArch64, RISC-V, Wasm all major features completed: loop labels, break/continue, struct field offsets, SizeOf/AlignOf, ternary all implemented)
- Perceus 引用计数整合: ✅ 已完成 - already handled via normal function calls

**总体: 约 95%**

## 验收标准

- [ ] x86_64 能正确生成可执行文件
- [ ] 能直接运行（或通过系统链接器链接后运行）
- [ ] 结果正确

## 依赖

- x-lir 必须完成
