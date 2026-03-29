# x-codegen-java 任务清单

Java 后端 - 生成 Java 源代码，供 JVM 运行。

## 已完成 ✅

| 任务 | 状态 |
|------|------|
| 框架结构完成 | ✅ 已完成 |
| 基本类型映射框架 | 🚧 部分完成 |

## 待完成 ⬜

| 序号 | 任务 | 优先级 | 现有 TODO 标记 | 描述 |
|------|------|--------|---------------|------|
| 1 | 实现 HIR → Java 代码生成 | 高 | ✔ | TODO 已标记 |
| 2 | 实现 LIR → Java 代码生成 | 高 | ✔ | TODO 已标记 |
| 3 | 完成所有语句和表达式生成 | 高 | 需要完整实现 |
| 4 | 完成类型映射 X → Java | 中 | 需要实现 |
| 5 | Perceus 引用计数集成（转为 JVM GC） | 中 | 需要适配 |
| 6 | 清理 TODO 项 | 高 | ✔ | 共 2 个 TODO |

## 现有 TODO 位置

```
src/lib.rs:1072 - TODO: implement HIR -> Java code generation
src/lib.rs:1077 - TODO: implement LIR -> Java code generation
```

## 完成度

约 20%

## 验收标准

- [ ] 生成的 Java 代码能被 `javac` 编译
- [ ] JVM 中能正确运行

## 依赖

- x-lir 必须完成
