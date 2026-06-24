# CLAUDE.md — x-cli

**命令行**：二进制名 **`x`**（`Cargo.toml` 中 `[[bin]] name = "x"`）。编排解析 → 类型检查 → HIR → MIR → LIR → 后端。全局规则见 [../../CLAUDE.md](../../CLAUDE.md)、[../../DESIGN_GOALS.md](../../DESIGN_GOALS.md)。

## 关键源文件

| 文件 | 作用 |
|------|------|
| `src/main.rs` | `clap` 子命令路由、`--verbose` / `-C` 等全局参数 |
| `src/pipeline.rs` | **`run_pipeline`** → `PipelineOutput { ast, hir, mir, lir }`；标准库路径、`ModuleResolver`、`type_check_with_big_stack*` |
| `src/commands/run.rs` | 解释执行路径（`x_interpreter::Interpreter::run`） |
| `src/commands/check.rs` | 仅解析 + 类型检查 |
| `src/commands/compile.rs` | **`--emit`**：`tokens`、`ast`、`hir`、`mir`、`lir`、`zig`、`c`、`rust`、`ts`、`js`、`dotnet`、`native`/`obj`（直出 `.o` 字节）等；**`--target`**：`native`（默认走 `x-codegen-native`，仅 x86_64 Linux）、`wasm*`、`zig`、`ts` 等 |

## 后端选择（`compile.rs` 摘要）

- **`Target::Native`**：`x_codegen_native::NativeBackend` 直出可重定位 ELF `.o`（仅 x86_64 Linux），再用 `link_object_linux`（`cc`/`clang`/`gcc`）链接；不调用外部汇编器。
- **Zig 路径**：`x_codegen_zig::ZigBackend` + 系统 **`zig`** 编译。
- 其他：`TypeScriptBackend`、`RustBackend`、`CSharpBackend` 等按 `x_codegen::Target` 分支。

## 构建与测试

```bash
cd tools/x-cli && cargo build
cd tools && cargo test -p x-cli
```

## 依赖提示

需要 **Zig 0.13+** 在 `PATH` 中才能走完 Zig/Wasm 完整链接；纯 `run`/`check` 不依赖 Zig。
