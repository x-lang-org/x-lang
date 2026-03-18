# x-cli 待办事项

## 当前状态

**整体完成度：约 90%**（核心命令 95%，其他命令 60%）

| 模块 | 状态 | 完成度 |
|------|------|--------|
| CLI 框架（clap 定义） | ✅ 完成 | 100% |
| 核心命令（run/check/compile） | ✅ 完成 | 95% |
| 项目结构（Project/Manifest） | ✅ 完成 | 95% |
| 包管理命令 | ✅ 完成 | 85% |
| 开发工具命令 | ✅ 完成 | 80% |
| 高级功能 | 🚧 部分实现 | 40% |
| 测试覆盖 | ✅ 单元+冒烟 | 5 单元 + 2 集成（smoke_check/smoke_run） |

## 命令完成度详情

### 核心命令（95% 完成）
- ✅ `run` - 运行程序
- ✅ `check` - 语法/类型检查
- ✅ `compile` - 完整编译流程
- ✅ `build` - 项目构建

### 包管理命令（85% 完成）
- ✅ `add` - 添加依赖
- ✅ `remove` - 移除依赖
- ✅ `update` - 更新依赖
- ✅ `vendor` - 依赖本地化
- ✅ `package` - 打包
- ✅ `publish` - 发布

### 开发工具命令（80% 完成）
- ✅ `fmt` - 格式化
- ✅ `lint` - 代码检查
- ✅ `fix` - 自动修复（框架已完成）
- ✅ `repl` - 交互式解释器
- ✅ `doc` - 文档生成
- ✅ `init`/`new` - 项目初始化
- ✅ `clean` - 清理

### 高级功能（40% 完成）
- ❌ 配置文件支持（.x/config.toml）
- ❌ 工作区支持
- ❌ 交叉编译
- ❌ 构建缓存
- ❌ 增量编译

---

## 🔴 高优先级

### 1. 完善 `compile` 命令的 Zig 后端集成
- [x] 实现 AST → XIR 转换，或让 Zig 后端直接支持 AST 输入
- [x] 调用 Zig 编译器生成可执行文件
- [x] 支持 `--release`、`--target` 等选项
- [x] Wasm 目标支持（wasm32-wasi, wasm32-freestanding）

### 2. 完善 `build` 命令
- [x] 启用 `#[cfg(feature = "codegen")]` 代码路径
- [x] 实现完整的代码生成和链接流程
- [x] 多目标支持（bin、lib、example）
- [x] 支持 `--examples` 选项

### 3. 实现 `test` 命令
- [x] 收集 `tests/` 目录下的测试文件
- [x] 编译并运行测试
- [x] 格式化测试输出（通过/失败统计）

---

## 🟡 中优先级

### 4. 实现 `fmt` 命令
- [x] 集成 x-parser 的 AST 格式化
- [x] 支持 `--check` 模式（仅检查不修改）
- [x] 递归格式化项目所有 .x 文件

### 5. 实现 `init` / `new` 命令
- [x] 创建项目目录结构
- [x] 生成默认 x.toml
- [x] 生成 src/main.x 或 src/lib.x 模板
- [x] Git 初始化（可选）

### 6. 实现 `clean` 命令
- [x] 删除 `target/` 目录
- [x] 支持 `--doc`、`--release` 选项

### 7. 添加单元测试
- [x] 测试命令行参数解析（通过 smoke_check/smoke_run 间接验证）
- [x] 测试 Project 查找逻辑（project_find_from_missing_manifest_has_hint、project_find_from_loads_manifest_and_root）
- [x] 测试 Manifest 解析（manifest_roundtrip_default_bin、find_manifest_path_walks_upwards）
- [x] 测试错误格式化（format_parse_error_includes_location_and_snippet）

---

## 🟢 低优先级

### 8. 包管理命令
- [x] `add` - 添加依赖到 x.toml（支持版本、路径、Git、特性等）
- [x] `remove` - 移除依赖
- [x] `update` - 更新 x.lock（支持 --dry-run、--aggressive）
- [x] `vendor` - 本地化依赖（支持递归复制目录）
- [x] `package` - 打包（支持 tar.gz 压缩、验证）
- [x] `publish` - 发布到注册表（支持 --dry-run、--allow-dirty）

### 9. 开发工具命令
- [x] `lint` - 代码检查（行尾空白、行长度、制表符、文件末尾换行）
- [x] `fix` - 自动修复（移除行尾空白、添加末尾换行、制表符转空格、移除多余空行）
- [x] `repl` - 交互式解释器（支持多行输入、解释器重置）
- [x] `doc` - 文档生成（生成 HTML 文档，支持浏览器打开）

### 10. 高级功能
- [ ] 配置文件支持（.x/config.toml）
- [ ] 工作区支持
- [ ] 交叉编译
- [ ] 构建缓存
- [ ] 增量编译

---

## 代码中的 TODO 标记

| 文件 | 位置 | 描述 | 状态 |
|------|------|------|------|
| `src/commands/compile.rs` | L36 | Zig backend currently only supports XIR input | 📝 已注释说明 |
| `src/commands/run.rs` | L56-57 | 已恢复类型检查；通过 pipeline::type_check_with_big_stack 及 main 大栈线程避免栈溢出 | ✅ 已解决 |
| `src/commands/fix.rs` | L42 | TODO: Apply automatic fixes | 🚧 待实现 |

---

## 待实现的功能

### `fix` 命令自动修复逻辑

当前 `fix` 命令框架已完成，需要实现以下自动修复功能：

1. **移除未使用的导入**
   - 分析 AST 找出未使用的 import 语句
   - 自动删除未使用的导入

2. **修复废弃语法**
   - 将旧语法转换为新语法
   - 例如：`fn` → `function`，`mut var` → `var`

3. **类型建议修正**
   - 根据类型检查器建议修复类型错误
   - 添加缺失的类型注解

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

---

## 最新更新

最后更新时间：2026-03-16

### 2026-03-16 更新
- ✅ 细化完成度：核心命令 95%，其他命令 60%
- ✅ 添加命令分类完成度表格
- ✅ 整体完成度从 95% 调整为 90%

### 2026-03-15 更新
- ✅ 更新 TODO.md 反映实际完成状态
- ✅ 确认所有包管理命令已实现
- ✅ 确认所有开发工具命令已实现（fix 命令框架已完成）
- ✅ 确认中优先级任务全部完成
