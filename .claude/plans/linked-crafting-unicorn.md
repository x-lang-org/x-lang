# TODO 完成计划

## 探索发现

### 已实现但 TODO 未标记的项目

经过代码探索，发现许多 CLI 命令已经实现：

| 命令 | 状态 | 描述 |
|------|------|------|
| `run` | ✅ 已实现 | 解析 + 类型检查 + 解释执行 |
| `check` | ✅ 已实现 | 语法和类型检查 |
| `compile` | ✅ 已实现 | 编译，支持 --emit |
| `build` | ✅ 部分实现 | 项目构建 |
| `test` | ✅ 已实现 | 收集 tests/ 目录，解析+类型检查+解释执行 |
| `fmt` | ✅ 已实现 | 基础格式化（缩进、trim） |
| `init` | ✅ 已实现 | 创建项目结构（x.toml, src/, .gitignore） |
| `clean` | ✅ 已实现 | 删除 target/ 目录 |
| `lint` | ✅ 已实现 | 检查尾随空白、行长度、制表符、换行符 |

### 实际待完成的项目

#### 1. 类型检查器（80%）
- [ ] 类/接口/trait 完整类型检查
- [ ] 递归类型定义检查
- [ ] 增量类型检查和错误恢复
- [ ] 错误携带 span 位置信息

#### 2. Perceus 内存管理（80%）
- [ ] 弱引用（weak）支持
- [ ] 循环引用检测

#### 3. Zig 后端
- [ ] 类/接口/trait 支持

#### 4. 并发和异步
- [ ] async/await 支持
- [ ] together/race/atomic 并发原语

#### 5. 工具链
- [ ] `fix` 命令：自动修复代码问题
- [ ] `repl` 命令增强
- [ ] `package`/`publish` 命令
- [ ] 依赖管理（add/remove/update/vendor）
- [ ] 调试器支持
- [ ] LSP 增强

---

## 实施计划

### 阶段 1：完善类型检查器

1. **完成类/接口/trait 类型检查**
   - 文件：`compiler/x-typechecker/src/lib.rs`
   - 添加 `ClassDecl`、`TraitDecl`、`ImplDecl` 的类型检查
   - 验证类字段类型、方法签名

2. **添加错误位置信息**
   - 修改 `TypeCheckError` 携带 span
   - 在各检查点添加位置信息

### 阶段 2：完善 Perceus

1. **添加 weak 引用支持**
   - 文件：`compiler/x-perceus/src/lib.rs`
   - 弱引用不参与引用计数
   - 自动降级为 None 当引用对象被释放

### 阶段 3：完善 Zig 后端

1. **添加类/接口支持**
   - 文件：`compiler/x-codegen/src/zig_backend.rs`
   - 生成 Zig struct/class 代码
   - 处理方法调度

### 阶段 4：增强工具链

1. **实现 `fix` 命令自动修复**
   - 基于 lint 结果自动修复
   - 常见问题：格式化、添加分号

2. **增强 `repl`**
   - 命令历史
   - 代码补全

---

## 验证方法

每个阶段完成后运行：
```bash
# 单元测试
cd compiler && cargo test

# 规格测试
cargo run -p x-spec

# 示例运行
cd tools/x-cli && cargo run -- run ../../examples/fib.x
```
