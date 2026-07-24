# X 标准库

按 **Swift Foundation + Kotlin stdlib** 习惯设计：可读、一种写法、失败用 `Option`/`Result`（无异常）。

权威约束见仓库根目录 [DESIGN_GOALS.md](../../DESIGN_GOALS.md)。

## 约定

| 主题 | 规则 |
|------|------|
| 命名 | 英文全称关键字；集合动词偏 Kotlin（`get`/`put`/`remove`）；可选值偏 Swift（`Option`） |
| 错误 | 可能失败 → `Result<T, E>`；可能缺失 → `Option<T>` |
| 实现 | 表面纯 X；C FFI / runtime 表示层只出现在 `unsafe` 边界 |
| 禁止 | 用户代码依赖 `x_list_*` / `x_map_*` 等 runtime 符号 |
| Prelude | 仅预导入日常表面（打印、字符串/数组 UFCS、断言）；完整集合用 `import std.map` 等 |

## 模块

| 模块 | 职责 |
|------|------|
| `std.types` | `Option` / `Result` |
| `std.list` | 可变列表 |
| `std.map` | 键值映射 |
| `std.set` | 集合 |
| `std.string` | 字符串（`count`/`has_prefix`/…；与 prelude UFCS 对齐） |
| `std.math` | 数学常数 + libc 绑定 + `clamp_*`/`lerp` |
| `std.io` / `std.fs` / `std.net` / `std.process` | 系统能力（C 库封装；fs 提供 `Result` 与 panic 两套） |
| `std.time` / `std.encoding` / `std.hash` | 时间、编码、哈希（部分模块仍在整理） |
| `std.panic` / `std.error` | 恐慌与错误；`std.errors` 仅为兼容再导出 |
| `sqlite/` | 独立包，不进 prelude |

## 集合 API（Swift / Kotlin 对照）

| X | Swift 近似 | Kotlin 近似 |
|---|------------|-------------|
| `Map.empty` / `get` → `Option` / `put` / `remove` | `Dictionary` | `MutableMap` |
| `List.empty` / `push` / `get` → `Option` | `Array` | `MutableList` |
| `Set.empty` / `insert` / `contains` | `Set` | `MutableSet` |

## 状态

集合层（`list` / `map` / `set`）以 **class + 动态数组** 实现，优先正确与可编译；哈希表与完整 trait 体系后续增强。
