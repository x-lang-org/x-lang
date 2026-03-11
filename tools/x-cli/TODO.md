# x-cli 待办事项

## 当前状态

**整体完成度：约 40%**

| 模块 | 状态 | 完成度 |
|------|------|--------|
| CLI 框架（clap 定义） | ✅ 完成 | 100% |
| 核心命令（run/check/compile） | ✅ 基本完成 | 80% |
| 项目结构（Project/Manifest） | ✅ 基本完成 | 70% |
| 其他命令（30+） | 🚧 桩实现 | 10% |
| 包管理 | ❌ 未实现 | 0% |
| 测试覆盖 | ✅ 单元+冒烟 | 5 单元 + 2 集成（smoke_check/smoke_run） |

---

## 🔴 高优先级

### 1. 完善 `compile` 命令的 Zig 后端集成
- [ ] 实现 AST → XIR 转换，或让 Zig 后端直接支持 AST 输入
- [ ] 调用 Zig 编译器生成可执行文件
- [ ] 支持 `--release`、`--target` 等选项
- [ ] Wasm 目标支持

### 2. 完善 `build` 命令
- [ ] 启用 `#[cfg(feature = "codegen")]` 代码路径
- [ ] 实现完整的代码生成和链接流程
- [ ] 多目标支持（bin、lib、example、test）

### 3. 实现 `test` 命令
- [ ] 收集 `tests/` 目录下的测试文件
- [ ] 编译并运行测试
- [ ] 格式化测试输出（通过/失败统计）

---

## 🟡 中优先级

### 4. 实现 `fmt` 命令
- [ ] 集成 x-parser 的 AST 格式化
- [ ] 支持 `--check` 模式（仅检查不修改）
- [ ] 递归格式化项目所有 .x 文件

### 5. 实现 `init` / `new` 命令
- [ ] 创建项目目录结构
- [ ] 生成默认 x.toml
- [ ] 生成 src/main.x 或 src/lib.x 模板
- [ ] Git 初始化（可选）

### 6. 实现 `clean` 命令
- [ ] 删除 `target/` 目录
- [ ] 支持 `--doc`、`--release` 选项

### 7. 添加单元测试
- [x] 测试命令行参数解析（通过 smoke_check/smoke_run 间接验证）
- [x] 测试 Project 查找逻辑（project_find_from_missing_manifest_has_hint、project_find_from_loads_manifest_and_root）
- [x] 测试 Manifest 解析（manifest_roundtrip_default_bin、find_manifest_path_walks_upwards）
- [x] 测试错误格式化（format_parse_error_includes_location_and_snippet）

---

## 🟢 低优先级

### 8. 包管理命令
- [ ] `add` - 添加依赖到 x.toml
- [ ] `remove` - 移除依赖
- [ ] `update` - 更新 x.lock
- [ ] `vendor` - 本地化依赖
- [ ] `package` - 打包
- [ ] `publish` - 发布到注册表

### 9. 开发工具命令
- [ ] `lint` - 代码检查
- [ ] `fix` - 自动修复
- [ ] `repl` - 交互式解释器
- [ ] `doc` - 文档生成

### 10. 高级功能
- [ ] 配置文件支持
- [ ] 工作区支持
- [ ] 交叉编译
- [ ] 构建缓存
- [ ] 增量编译

---

## 代码中的 TODO 标记

| 文件 | 位置 | 描述 |
|------|------|------|
| `src/commands/compile.rs` | L36 | Zig backend currently only supports XIR input |
| `src/commands/run.rs` | L56-57 | 已恢复类型检查；通过 pipeline::type_check_with_big_stack 及 main 大栈线程避免栈溢出 |

---

## 测试覆盖率估计

当前有 5 个单元测试（manifest/project/pipeline）和 2 个集成冒烟测试（smoke_check、smoke_run）。可用 `cargo llvm-cov -p x-cli --tests` 测量覆盖率。

## 质量门禁（可测试与可验证）

### 覆盖率目标

- **行覆盖率**：100%
- **分支覆盖率**：100%
- **测试通过率**：100%

### 必须具备的测试类型

- [x] **单元测试**：覆盖命令参数解析、命令 dispatch、`Project`/manifest/config/lockfile 逻辑与错误格式化（Manifest/Project/pipeline 单测已添加；tempfile 为 dev-dependency）
- [x] **集成测试**：覆盖 `run/check` 的关键路径（smoke_check、smoke_run 使用最小 .x 源码，避免与 examples/*.x 的规范语法差异）
- [ ] **回归用例**：每次修复 CLI 行为/错误输出差异都新增最小复现测试

### 验收步骤（本地一键验证）

```bash
cd tools
cargo test -p x-cli

# 覆盖率（line/branch）
cargo llvm-cov -p x-cli --tests
```
