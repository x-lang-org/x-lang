# CLAUDE.md — x-codegen-native

**Native 后端（直出机器码）**：**LIR → 机器码字节 + 重定位 → 可重定位 ELF (`.o`)**，再交系统链接器（`cc`）链接为可执行文件。**不依赖外部汇编器**。当前仅支持 **x86_64 Linux**（System V AMD64 ABI）。全局规则见 [../../CLAUDE.md](../../CLAUDE.md)、[../../DESIGN_GOALS.md](../../DESIGN_GOALS.md)。

## 模块布局（`src/lib.rs` 导出）

| 模块 | 作用 |
|------|------|
| `arch` | 架构相关常量与 ABI 辅助、寄存器/指令定义 |
| `encoding` | `X86_64Encoder`：把单条 x86-64 指令编码为字节 |
| `machine` | `machine/x86_64.rs` 的 `MachineCodeGen`：LIR → `.text` 字节，函数内 label fixup、字符串入 `.rodata`、全局入 `.bss`、收集符号与重定位（`machine/mod.rs` 为共享模型） |
| `emitter` | `write_relocatable_elf`：把 `MachineObject` 写为 `ET_REL` ELF64 目标文件 |

## 主要类型

- **`NativeBackend`**、**`NativeBackendConfig`**：`TargetArch`、`TargetOS`、`OutputFormat`。
- **`MachineCodeGen`** / **`MachineObject`**：直出机器码核心与产物（`text`/`rodata`/`bss_size`/`symbols`/`relocations`）。
- **`impl x_codegen::CodeGenerator for NativeBackend`**：以 **`generate_from_lir`** 为主；CLI `compile` 默认 **`Target::Native`** 走此后端，返回 `FileType::ObjectFile`。

## 维护注意

- 仅支持 x86_64 Linux；非 x86_64 / 非 Linux 返回 `NativeError::Unimplemented`。
- 重定位约定：字符串/全局用 `R_X86_64_PC32`（指向 section 符号，addend = 段内偏移 − 4）；外部调用用 `R_X86_64_PLT32`（addend = −4）；函数内跳转/内部调用直接回填 rel32（标签为函数局部，按函数清空避免同名冲突）。
- 链接在 **`tools/x-cli/src/commands/compile.rs`** 的 `link_object_linux`（仅 `cc`/`clang`/`gcc`，无 `as`/nasm 步骤）。

## 测试

```bash
cd compiler && cargo test -p x-codegen-native
```
