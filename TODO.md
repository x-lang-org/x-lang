# X 语言项目待办事项 (TODO)

本文件列出了 X 语言项目所有未完成的功能和待实现特性，按优先级和模块分类。

## 优先级定义
- 🔴 高优先级：核心功能，必须首先实现
- 🟡 中优先级：重要功能，短期需要实现
- 🟢 低优先级：长期规划，未来版本实现

---

## 1. 编译器核心阶段 🔴

### 1.1 类型检查器 (`x-typechecker`) - 进度: 70% ✅
- [x] 变量/函数未定义检查
- [x] 类型兼容检查和类型推断
- [x] 函数签名校验（参数类型、返回类型）
- [x] 完整类型比较支持（Option, Result, Tuple, Record, Union, Async, Generic）
- [x] Lambda 表达式类型推断
- [x] Record 表达式类型推断
- [x] Pipe 操作类型推断
- [x] Wait 操作类型推断
- [x] Needs/Given Effect 表达式支持
- [x] 类型兼容性检查（is_type_compatible、common_supertype）
- [x] Match/Try/For 语句类型检查
- [x] Break/Continue 类型检查
- [x] 作用域检查（函数/分支/循环/try-catch 等引入新作用域）
- [x] 错误格式化模块（format.rs）
- [ ] 类、接口、特征的完整类型检查
- [ ] 类型参数约束检查
- [ ] 递归类型定义检查
- [ ] 增量类型检查和错误恢复

### 1.2 HIR 生成 (`x-hir`) - 进度: 80% ✅
- [x] 完整的 HIR 数据结构定义
- [x] 从 AST 到 HIR 的转换逻辑
- [x] 类型环境（符号表）
- [x] 支持所有声明、语句、表达式类型
- [x] 语法糖消除（如管道操作展开）
- [x] 语义分析（变量解析、函数解析、作用域处理）
- [x] 常量折叠优化
- [x] 死代码消除优化
- [x] 错误分类和代码
- [ ] HIR 优化和简化（内联优化、循环优化）
- [ ] 类型参数化支持（部分实现）
- [ ] 与 Perceus 集成（所有权信息编码）
- [ ] 增量 HIR 更新

### 1.3 Perceus 内存管理 (`x-perceus`) - 进度: 60% ✅
- [x] PerceusIR 完整数据结构定义
- [x] 分析函数接口
- [x] Perceus 分析算法实现
- [x] 循环和条件语句的分支分析
- [x] 引用计数优化（dup/drop 插入）- 基础框架完成
- [x] 内存复用分析
- [x] 与代码生成后端集成（Zig 后端已集成）
- [x] 跨函数内存分析（函数签名收集、调用图构建、递归检测、参数所有权推断）
- [x] Copy/Consume/Borrow/BorrowMut 所有权行为枚举
- [ ] 更精确的复用分析算法
- [ ] 增量分析支持
- [ ] 复杂数据结构分析支持

---

## 2. 代码生成后端

### 2.1 Zig 后端 (最成熟) 🔴 - 进度: 85% ✅
- [x] 函数声明和变量声明
- [x] 基本类型和表达式
- [x] If/While/For 语句
- [x] 数组和字典的完整支持
- [x] match 模式匹配实现
- [x] try 语句和异常处理实现
- [x] HIR Match/Try 语句代码生成
- [x] Option/Result 类型支持
- [x] 结构体/记录类型支持
- [x] Perceus 引用计数集成（基础框架完成）
- [x] generate_from_hir 和 generate_from_pir 方法
- [ ] 类/接口/特征支持
- [ ] Lambda 表达式完整支持
- [ ] 错误处理和调试信息生成优化

### 2.2 JavaScript/TypeScript 后端 🟡 - 进度: 50%
- [x] 函数声明和变量声明
- [x] 类声明支持
- [x] Lambda 表达式支持
- [x] for 循环实现
- [x] match 语句实现
- [x] try/catch/finally 语句实现
- [x] 数组和字典支持
- [x] 内置函数映射（print、to_string、len 等）
- [ ] 完整的类型系统支持
- [ ] TypeScript 类型注解生成
- [ ] 类和接口完整实现
- [ ] 错误处理和效果系统
- [ ] 代码优化和压缩
- [ ] 从 HIR/PIR 生成代码

### 2.3 JVM 后端 🟡 - 进度: 45%
- [x] Java 源代码生成器实现
- [x] 未初始化变量的类型注解处理
- [x] 基本类型映射（X 到 Java）
- [x] 变量和函数声明翻译
- [x] 表达式和运算符翻译
- [x] 控制流语句翻译（if、while、for、match、try、do-while）
- [x] 数组类型翻译
- [x] try/catch/finally 异常处理
- [ ] 完整 JVM 字节码生成器实现
- [ ] 类和接口的翻译
- [ ] 与 Java 互操作支持

### 2.4 .NET 后端 🟡 - 进度: 50%
- [x] C# 源代码生成器实现
- [x] 类型映射（X 语言类型 → .NET 类型）
- [x] 基本块代码生成
- [x] 函数调用和变量绑定
- [x] 整数/浮点/布尔/字符/字符串类型支持
- [x] if/else、while/for 循环
- [x] match 模式匹配语句
- [x] try/catch/finally 语句
- [x] do-while 语句
- [ ] 完整 CIL 字节码生成器实现
- [ ] 类和接口支持
- [ ] 闭包支持
- [ ] 与 .NET 语言互操作支持

### 2.5 Python 后端 🟢 - 进度: 50%
- [x] for 循环实现
- [x] match 语句实现
- [x] try 语句实现
- [ ] 完整类型系统支持
- [ ] 异常处理完善

### 2.6 Java 后端 🟢 - 进度: 40%
- [x] 未初始化变量的类型注解处理
- [ ] 完整类型系统支持
- [ ] 数组和字典支持
- [ ] 类和接口支持
- [ ] 错误处理

---

## 3. 标准库 (`x-stdlib`) 🟡

### 3.1 核心模块
- [ ] 完整集合数据结构实现（数组、字典、链表、队列、栈等）
- [ ] 网络编程支持（TCP/UDP、HTTP 客户端/服务端）
- [ ] 文件系统操作完整实现
- [ ] 并发和多线程支持
- [ ] 加密和哈希函数
- [ ] 正则表达式
- [ ] 日期时间处理高级功能
- [ ] 科学计算和统计函数
- [ ] 异步 I/O 支持

---

## 4. 工具链 🟡 - 进度: 95% ✅

### 4.1 CLI 命令 - 进度: 95% ✅
- [x] `run` 命令：解析 → 类型检查 → 解释执行
- [x] `check` 命令：语法和类型检查
- [x] `compile` 命令：完整流水线，支持 --emit tokens|ast|hir|pir|zig
- [x] `build` 命令：项目构建，支持 --examples
- [x] `test` 命令：测试运行
- [x] `init`/`new` 命令：项目初始化
- [x] `clean` 命令：清理构建产物
- [x] `fmt` 命令：完整格式化器实现，支持 --check
- [x] `lint` 命令：代码检查规则（行尾空白、行长度、制表符检查）
- [x] `package` 命令：打包功能（tar.gz 压缩、验证）
- [x] `publish` 命令：包发布（支持 --dry-run、--allow-dirty）
- [x] 依赖管理命令：`add`/`remove`/`update`/`vendor`（完整实现）
- [x] `fix` 命令：自动修复代码问题（移除行尾空白、制表符转空格、移除多余空行）
- [x] `repl` 命令：交互式解释器（支持多行输入、命令）
- [x] `doc` 命令：文档生成（生成 HTML 文档，支持浏览器打开）
- [ ] 配置文件支持（.x/config.toml）
- [ ] 工作区支持
- [ ] 交叉编译
- [ ] 构建缓存
- [ ] 增量编译

### 4.2 开发工具 - 进度: 85%
- [x] LSP (语言服务器) 实现：代码导航、自动补全、诊断
  - 文档符号、悬停提示、跳转定义、查找引用、诊断
  - 类型检查器诊断集成
  - 符号位置查找
- [x] 语法高亮生成器：
  - Emacs major mode
  - Neovim Tree-sitter
  - Sublime Text
  - JetBrains (IntelliJ IDEA)
- [x] 文档生成工具（`doc` 命令）
- [ ] 调试器支持
- [ ] 增量编译支持
- [ ] 代码覆盖率分析工具
- [ ] 性能分析工具

---

## 5. 测试和规格 🟡

### 5.1 单元测试
- [x] 类型检查器测试（32 个单元测试）
- [x] HIR 测试（28 个单元测试）
- [x] Perceus 测试（15 个单元测试）
- [x] 解释器测试（覆盖核心执行路径）
- [x] 代码生成后端测试（Zig/Python/Java/C#/TypeScript）
- [x] CLI 测试（5 个单元测试 + 2 个集成冒烟测试）
- [ ] 各模块完整边界条件测试
- [ ] 标准库测试覆盖

### 5.2 规格测试
- [ ] 覆盖完整语言特性的 TOML 测试用例
- [ ] 所有测试用例与规范文档章节关联
- [ ] 性能基准测试
- [ ] 内存使用测试

### 5.3 基准测试
- [x] examples/ 目录下的基准测试程序（10 个来自计算机语言基准测试游戏）
- [x] build_benchmarks.sh 脚本支持多后端构建
- [ ] 与其他语言的全面性能对比
- [ ] 多后端性能比较
- [ ] 编译时间基准
- [ ] 内存使用基准

---

## 6. 语言特性 🔴 - 进度: 65%

### 6.1 核心特性
- [x] 函数声明和调用
- [x] 变量声明（let/const）
- [x] 基本类型（Int、Float、Bool、String、Char）
- [x] 数组和字典基础支持
- [x] 记录/结构体类型（基础支持）
- [x] Option/Result 类型
- [x] match 模式匹配（when guard、or-pattern）
- [x] try/catch/finally 异常处理
- [x] for/while 循环
- [x] Lambda 表达式（基础支持）
- [x] Pipe 操作符
- [ ] 类和接口系统
- [ ] 效果系统（R·E·A）
- [ ] 并发和异步支持
- [ ] 泛型和 trait 系统
- [ ] 元编程和反射

### 6.2 语法特性
- [x] 自然语言风格关键字（needs、given、wait、when/is、can、atomic）
- [x] 数学函数表示法
- [ ] 完整 Unicode 支持
- [ ] 高级模式匹配语法
- [ ] 多行表达式和语句
- [ ] 自然语言语法优化

---

## 7. 诊断和错误处理 🔴

- [x] 类型检查错误携带 span 位置信息
- [x] 运行时错误携带位置信息
- [x] 多错误收集和恢复
- [x] 代码修复建议
- [x] 详细的错误分类和严重程度
- [x] 友好的错误信息格式

---

## 8. 文档 🟡

- [ ] 完整 API 参考文档
- [ ] 标准库详细文档
- [ ] 编译器架构深度解析
- [ ] 语言特性完整示例
- [ ] 调试和优化指南
- [ ] 性能调优手册
- [ ] 与其他语言对比分析

---

## 9. 性能优化 🟢

### 9.1 编译期性能
- [ ] 增量解析和编译
- [ ] 并行编译
- [ ] 编译器内存使用优化
- [ ] 大文件/大 AST 处理优化

### 9.2 运行时性能
- [ ] 代码生成优化
- [ ] 内存管理优化
- [ ] 垃圾回收优化
- [ ] 并行执行支持

---

## 10. 工业级特性 🟢

- [ ] IDE 集成（VS Code 插件）
- [ ] 代码导航和跳转
- [ ] 代码自动完成
- [ ] 调试支持
- [ ] 交叉编译支持
- [ ] 平台特定优化
- [ ] 构建缓存优化

---

## 代码中现存 TODO/FIXME 标记

| 文件 | 位置 | 描述 |
|------|------|------|
| `x-hir/src/lib.rs` | L2278 | 说明性注释：annotate_ownership 函数已移至 x-perceus 模块 |
| `x-cli/src/commands/fix.rs` | L42 | fix 命令自动修复框架（移除未使用导入、修复废弃语法等） |

> **注**：以上代码中的 TODO 标记为说明性注释或待实现功能。

---

## 最新更新
最后更新时间：2026-03-16

### 2026-03-16 更新
- ✅ 统一更新所有 TODO.md 完成度数字，反映实际代码状态
  - 类型检查器：75% → 70%
  - HIR：75% → 80%
  - Perceus：75% → 60%
  - Zig 后端：95% → 85%
  - JavaScript 后端：65% → 50%
  - JVM 后端：55% → 45%
  - .NET 后端：55% → 50%
  - 解释器：添加明确的 75% 完成度
  - 语法分析器：70% → 85%
  - 词法分析器：95% → 90%

### 2026-03-15 更新（续10）
- ✅ 更新根 TODO.md 反映各模块实际完成状态
- ✅ 类型检查器：确认完成作用域检查、Match/Try/For 语句检查
- ✅ HIR：确认完成常量折叠、死代码消除优化
- ✅ Perceus：确认完成跨函数内存分析和所有权行为枚举
- ✅ 代码生成后端：更新各后端完成度评估
  - Zig 后端：95%（最成熟）
  - JavaScript 后端：65%
  - JVM 后端：55%
  - .NET 后端：55%
  - Python 后端：50%
- ✅ 工具链：确认完成语法高亮生成器（Emacs/Neovim/Sublime/JetBrains）
- ✅ CLI：确认完成度达 95%

### 2026-03-15 更新（续9）
- ✅ 更新 x-cli TODO.md 反映实际完成状态
- ✅ 更新根 TODO.md 工具链部分状态
- ✅ 修复多个编译警告（未使用的导入、参数等）
- ✅ 确认所有包管理命令已实现
- ✅ 确认所有开发工具命令已实现
- ✅ 实现 `fix` 命令自动修复功能
  - 移除行尾空白字符
  - 确保文件末尾有换行符
  - 将制表符转换为 4 个空格
  - 移除多余的连续空行（超过 2 行）
  - 添加 5 个单元测试
- ✅ 实现 Emacs major mode 语法定义生成器
- ✅ 实现 Neovim Tree-sitter 语法定义生成器

### 2026-03-15 更新（续8）
- ✅ 实现 Sublime Text 语法定义生成器
- ✅ 实现 JetBrains (IntelliJ IDEA) 语法定义生成器

### 2026-03-15 更新（续7）
- ✅ 实现 LSP 类型检查器诊断（添加类型错误到诊断输出）
- ✅ 实现 LSP 符号位置查找（find_at_position 方法）
- ✅ 修复 x-lsp 编译错误（Statement 类型匹配、临时值生命周期）

### 2026-03-15 更新（续6）
- ✅ 更新 JVM 后端 TODO.md 反映已实现的功能
- ✅ 更新 .NET 后端 TODO.md 反映已实现的功能

### 2026-03-15 更新（续5）
- ✅ 完善 JVM 后端 Match/Try/DoWhile 语句支持
- ✅ 完善 .NET 后端 DoWhile 语句支持
- ✅ 消除 JVM 和 .NET 后端的 TODO 标记

### 2026-03-15 更新（续4）
- ✅ 实现 x-codegen-wasm 类型检查
- ✅ 完善 JavaScript 后端模式匹配（match 语句完整实现）
- ✅ 完善 JavaScript 后端 try/catch/finally 支持
- ✅ 完善 JVM 后端基础代码生成（generate_from_ast 实现）
- ✅ 完善 .NET 后端基础代码生成（generate_from_ast 实现）

### 2026-03-15 更新（续3）
- ✅ 增强类型检查器错误系统
  - 添加错误分类（NameResolution, TypeMismatch, Declaration 等）
  - 添加错误严重程度（Error, Warning, Info）
  - 添加错误代码（E0001 - E0015）
  - 添加修复建议功能（FixSuggestion）
  - 增强格式化模块（format_type_error_with_suggestions）
- ✅ 增强解释器错误系统
  - 添加更多错误类型（UndefinedVariable, UndefinedFunction, TypeError, DivisionByZero 等）
  - 所有错误类型携带 Span 位置信息
- ✅ 增强 HIR 错误系统
  - 添加更多错误类型（UnresolvedReference, InvalidOperation, SemanticError）
  - 添加便捷构造方法

### 2026-03-15 更新（续2）
- ✅ 修复 x-hir 循环依赖问题
  - 移除 x-hir 对 x-perceus 的依赖，避免循环依赖
  - 将 `annotate_ownership` 函数标记为 TODO（应移至 x-perceus）
- ✅ 修复 Hir 结构体初始化缺少 perceus_info 字段的编译错误
  - 更新 x-perceus、x-codegen、x-codegen-js 中的测试代码
- ✅ 确认诊断与错误处理已实现
  - 类型检查错误携带 span 位置信息（TypeError 枚举）
  - 运行时错误携带位置信息（InterpreterError 结构体）
  - 多错误收集和恢复（TypeCheckResult 结构体）
  - 友好的错误信息格式（format_type_error 函数）

### 2026-03-15 更新（续）
- ✅ 实现类型兼容性检查
- ✅ 实现类型兼容性检查
  - 添加 `is_type_compatible` 函数，支持 Never 子类型、Int→Float 隐式转换、Union 类型成员检查、Option 类型兼容、数组协变等
  - 添加 `common_supertype` 函数，用于分支合并等场景的类型推导
  - 新增 12 个测试用例覆盖类型兼容性功能
- ✅ 确认 PerceusIR 到 Zig 代码生成接口已实现并测试通过

### 2026-03-15 更新
- ✅ 实现 Perceus 跨函数内存分析
  - 添加 FunctionSignature 结构用于记录函数参数和返回值的所有权行为
  - 实现 InterproceduralContext 用于管理函数签名和调用图
  - 实现递归函数检测
  - 支持参数所有权行为推断（Copy、Consume、Borrow、BorrowMut）
  - 新增 9 个测试用例覆盖跨函数分析功能

### 2026-03-11 更新
- ✅ 实现 Perceus 与 Zig 后端集成
- ✅ 添加 `generate_from_hir` 和 `generate_from_pir` 方法到 ZigBackend
- ✅ 实现内存操作代码生成（dup/drop/reuse/alloc）
- ✅ 添加 HIR 类型到 Zig 类型的映射
- ✅ 新增 3 个测试用例覆盖 Perceus 集成
