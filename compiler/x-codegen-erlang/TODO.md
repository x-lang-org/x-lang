# TODO - x-codegen-erlang

## 待完成功能

### 高优先级

---

## 完成状态

- [x] 基本框架
- [x] AST：`while`/`do-while`/`loop` 尾递归辅助函数；`for` 使用 `lists:foreach`；`try` 使用 `begin/end` + 合法 `catch`；成员/赋值偏向 `maps:get` / `maps:put`；补全 `Cast`、`Tuple`、`Record`、`Range`、`Pipe`、`Await`、`TryPropagate`、`OptionalChain`、`NullCoalescing` 等表达式
- [x] LIR：`generate_from_lir` 与模块导出、`-spec`、语句序列、`if`/`while`/`do-while`、表达式（含 `Call`→`io:format`、`lists:nth` 1-based 索引等）
- [ ] HIR 直出（仍为 `Unimplemented`，可后续接 AST 或 LIR）

---

## 后续可增强

- 更完整的 LIR `switch` / `match` / `for` / `try`
- 记录类型与 `Member` 的 Dialyzer 友好 `-spec`
- Lambda / 块体的完整闭包生成
