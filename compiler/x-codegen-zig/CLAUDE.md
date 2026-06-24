# CLAUDE.md — x-codegen-zig

**Zig 后端**：从 **AST**（及 trait 中的 HIR/LIR 路径）生成 **Zig 0.16** 风格源码，再由系统 **`zig`** 编译为本机或 Wasm。全局规则见 [../../CLAUDE.md](../../CLAUDE.md)、[../../DESIGN_GOALS.md](../../DESIGN_GOALS.md)。

## 主要类型（`src/lib.rs`）

- **`ZigBackend`** + **`ZigBackendConfig`**（`output_dir`、`optimize`、`debug_info`、**`target: ZigTarget`**）。
- **`ZigTarget`**：`Native`、`Wasm32Wasi`、`Wasm32Freestanding` → `as_zig_target()` / `output_extension()`。
- 实现 **`x_codegen::CodeGenerator`**（文件末尾 `impl CodeGenerator for ZigBackend`）：`generate_from_ast` 为主路径；HIR/LIR 方法以实现为准。

内部大量使用 **`x_codegen::CodeBuffer`**、`x_codegen::headers`，并匹配 **`x_parser::ast`** 的 `ExpressionKind`、`StatementKind` 等。

## 环境与 CLI

- 本机需 **Zig 0.16**（根 CLAUDE）；CLI 在 `tools/x-cli/src/commands/compile.rs` 中在 Zig 目标下构造 `ZigBackend` 并调用 `zig build-exe` 等。生成代码已对齐 0.16 stdlib API（如小写 `@typeInfo` 标签 `.pointer`、`std.net.Server`）。

## 测试

```bash
cd compiler && cargo test -p x-codegen-zig
```
