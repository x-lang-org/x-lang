# 第0章 设计哲学

> 本章直接映射 [DESIGN_GOALS.md](../../DESIGN_GOALS.md) 中定义的 17 条设计目标。
> DESIGN_GOALS.md 是 X 语言的最高准则与宪法性文件，本章是其在规格说明书中的具体阐释。

---

## 0.1 核心设计原则

X 语言的设计由以下 17 条原则驱动，按编号排列，其中**第 15 条（可读性第一）具有最高优先级**——当其他原则与可读性冲突时，可读性胜出。

| # | 原则 | 一句话描述 |
|---|------|-----------|
| 1 | 通用性 | 一门语言，从系统到应用 |
| 2 | 类型安全 | 编译通过 ≈ 无类型错误 |
| 3 | 内存安全 | Perceus：无 GC、无手动管理、无泄漏 |
| 4 | 多范式 | FP + OOP + 过程式 + 声明式，按需选择 |
| 5 | 博采众长 | 站在 Python/Rust/Go/Kotlin/Haskell/... 的肩膀上 |
| 6 | HM 推断 | 少写类型，多靠推断 |
| 7 | 默认不可变 | `let` 不可变，`let mutable` 可变；值/引用小写/大写；`const` 编译期常量 |
| 8 | 多种并发 | goroutine + Actor + async/await，按需选择 |
| 9 | 完整工具链 | `x` CLI 以 Cargo 级体验为目标，并持续迭代实现 |
| 10 | 效果系统 | 副作用在类型中可见（`with`） |
| 11 | C FFI | 与 C 零开销互操作 |
| 12 | 多后端 | C / LLVM / JS / JVM / .NET，一次编写，多处运行 |
| 13 | 无异常 | `Option` + `Result` + `?`，错误即值 |
| 14 | 关键字全称 | 英文全称，含义自明，不缩写 |
| 15 | 可读性第一 | 写一次读百次，可读性永远胜出（最高优先级） |
| 16 | 不用奇怪符号 | 只用常见符号，看一眼就懂 |
| 17 | AI 友好 | 预编译补全，机器可读，人机同一套语法 |

---

## 0.2 可读性第一（Goal 15）

> **Code is read far more often than it is written.**

可读性是 X 语言**最高优先级的设计约束**。当可读性与简洁性、性能、灵活性冲突时，**可读性胜出**。

### 核心准则

1. **一眼看懂**：任何代码片段，一个合格的开发者应当在几秒内理解其意图
2. **只有一种显而易见的写法**：避免提供多种等价但风格迥异的语法（Python 之禅）
3. **新手可读，专家可写**：语法对初学者友好，同时不限制高级用法

### 可读性如何指导设计

| 设计决策 | 可读性优先的选择 | 被拒绝的替代方案 |
|---------|----------------|----------------|
| 关键字 | `function`（完整英文） | `fn`、`func`、`def` |
| 可变性 | `let mutable x = 0` | `var x`（含义不够显式） |
| 模式匹配 | `match value { ... }` | 运算符重载式匹配 |
| 管道 | `data \|> filter() \|> map()` | 深层嵌套 `map(filter(data))` |
| 错误处理 | `Result<T, E>` + `?` | 隐式异常 |
| 效果 | 类型签名中显式声明 | 隐藏副作用 |
| 导入 | `import std.collections.HashMap` | `use std::collections::HashMap` |

### 可读性检验清单

在引入任何新语法或特性时，必须通过以下检验：

- [ ] **大声朗读测试**：把代码读出来，是否像在说一句通顺的英语？
- [ ] **六个月测试**：半年后重新看这段代码，能否立即理解？
- [ ] **新手测试**：一个没学过 X 的程序员，能否猜出这段代码的含义？
- [ ] **无需注释测试**：代码本身是否已经足够清晰，不需要注释来解释"做了什么"？

### 示例对比

```x
// X：可读性优先
function fibonacci(n: Integer) -> Integer {
    match n {
        0 => 0
        1 => 1
        _ => fibonacci(n - 1) + fibonacci(n - 2)
    }
}

let results = numbers
    |> filter(is_even)
    |> map(square)
    |> take(10)
```

```rust
// 对比 Rust：简洁但符号密度高
fn fib(n: i32) -> i32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

let res: Vec<_> = nums.iter().filter(|&&x| x % 2 == 0).map(|&x| x * x).take(10).collect();
```

**设计准则**：宁可多打几个字符，也不要牺牲可读性。写代码花一次时间，读代码花一百次。

---

## 0.3 关键字设计（Goal 14）

X 的关键字**必须使用英文全称**，且能**准确表达其语义**，不使用缩写或生造词。

### 设计规则

1. **全称，不缩写**：`function` 而非 `fn`/`func`，`Integer` 而非 `int`/`i32`，`Boolean` 而非 `bool`
2. **含义自明**：读到关键字就知道它做什么，无需查文档
3. **来自自然英语**：优先使用普通英语词汇，而非编程术语黑话

### 关键字对照表

| 类别 | X 关键字 | 含义 | 对比其他语言 |
|------|---------|------|-------------|
| 声明 | `let` | 不可变绑定 | Rust `let`, JS `const` |
| 声明 | `mutable` | 可变绑定标记 | Rust `mut`, Kotlin `var` |
| 函数 | `function` | 函数定义 | Rust `fn`, Go `func`, Python `def` |
| 返回 | `return` | 函数返回 | 通用 |
| 条件 | `if` / `else` | 条件分支 | 通用 |
| 循环 | `for` / `while` | 循环 | 通用 |
| 匹配 | `match` | 模式匹配 | Rust `match`, Scala `match` |
| 类型 | `type` | 类型别名/定义 | Go `type`, TS `type` |
| 类 | `class` | 类定义 | 通用 |
| 接口 | `trait` | 接口/行为约束 | Rust `trait`, Scala `trait` |
| 实现 | `implement` | 实现 trait | Rust `impl`, Java `implements` |
| 模块 | `module` | 模块声明 | — |
| 导入 | `import` / `export` | 导入/导出符号 | Python/TS/Kotlin |
| 异步 | `async` / `await` | 异步操作 | 通用 |
| 并发 | `together` / `race` | 并发组合 | X 独创 |
| 访问 | `public` / `private` / `protected` / `internal` | 访问修饰符 | Java/C#（全称） |

### 类型名称使用全称

| X 类型名 | 含义 | 其他语言对比 |
|---------|------|-------------|
| `Integer` | 整数类型 | Rust `i32`/`i64`, C `int`, Go `int` |
| `Float` | 浮点数类型 | 通用（已是全称） |
| `Boolean` | 布尔类型 | Rust `bool`, C `bool`, Go `bool` |
| `Character` | 字符类型 | Rust `char`, C `char` |
| `String` | 字符串类型 | 通用（已是全称） |
| `Unit` | 单元类型 | Kotlin `Unit`, Scala `Unit` |
| `Never` | 永不返回类型 | Rust `!`, TS `never` |

### 反面示例（X 不会采用）

| 缩写/生造 | 问题 | X 的选择 |
|-----------|------|---------|
| `fn` | 不是英文单词 | `function` |
| `func` | 截断 | `function` |
| `def` | 含义模糊（define?） | `function` |
| `impl` | 缩写 | `implement` |
| `pub` | 缩写 | `public` |
| `mod` | 缩写，且与取模冲突 | `module` |
| `mut` | 缩写 | `mutable` |
| `i32` / `u8` | 密码式命名 | `Integer` / `Byte` |
| `bool` | 截断 | `Boolean` |
| `char` | 截断 | `Character` |
| `Int` | 截断 | `Integer` |

**设计准则**：代码应该像散文一样可读。关键字是语言的词汇表——每个词都必须是真正的英语单词，且含义精确。

---

## 0.4 类型安全与无异常（Goals 2, 13）

### 类型安全（Goal 2）

X 拥有**完整的、健全的类型系统**：

- **Hindley-Milner 类型推断**：绝大多数类型注解可省略，编译器自动推断
- **代数数据类型（ADT）**：`enum`（sum type）+ `record`（product type）
- **参数多态**：泛型函数与泛型类型
- **无 null**：用 `Option<T>` 代替 null，编译器强制处理
- **无异常**：用 `Result<T, E>` 代替异常，错误路径显式可见
- **穷尽匹配**：`match` 模式匹配必须覆盖所有情况

```x
// 类型推断——无需注解，编译器自动推断
let x = 42                      // x : Integer
let f = (a, b) -> a + b         // f : (Integer, Integer) -> Integer
let xs = [1, 2, 3]              // xs : List<Integer>
let result = xs |> map(f(_, 1)) // result : List<Integer>
```

**设计准则**：如果编译通过，程序就不应出现类型错误。

### 无异常（Goal 13）

X 语言**没有异常机制**。没有 `try`、`catch`、`finally`、`throw`。所有错误处理通过类型系统完成。

#### Option：表示"有或无"

```x
// Option<T> = Some(T) | None
function find(users: List<User>, id: Integer) -> Option<User> {
    users |> filter(.id == id) |> first
}

let user = find(users, 42)
match user {
    Some(u) => println("Found: {u.name}")
    None    => println("Not found")
}

// 便捷运算符
let name = user?.name ?? "anonymous"   // 可选链 + 默认值
```

#### Result：表示"成功或失败"

```x
// Result<T, E> = Ok(T) | Err(E)
function read_file(path: String) -> Result<String, IoError> {
    if not exists(path) {
        return Err(IoError.NotFound(path))
    }
    Ok(read_bytes(path).decode())
}

// 模式匹配处理
match read_file("config.toml") {
    Ok(content) => parse_config(content)
    Err(e)      => use_default()
}

// ? 运算符：错误自动向上传播
function load_config() -> Result<Config, IoError> {
    let content = read_file("config.toml")?
    let config = parse(content)?
    Ok(config)
}
```

#### 设计理由

| 异常机制 | Option/Result |
|---------|--------------|
| 隐式控制流，调用者不知道函数会抛什么 | 错误路径在类型签名中**显式可见** |
| 容易遗漏 catch，运行时崩溃 | 编译器**强制**处理每种情况 |
| 性能不可预测（栈展开开销） | **零成本**抽象，与正常返回相同开销 |
| 难以组合（try-catch 嵌套地狱） | 用 `?` 和 `\|>` **链式组合** |

**设计准则**：错误是值，不是控制流。所有可能失败的操作都必须在返回类型中体现。

---

## 0.5 内存安全（Goal 3）

X 采用与 **Koka** 相同的 **Perceus** 算法实现内存管理：

- **编译时引用计数**：编译器在编译期精确插入 `dup`（引用复制）和 `drop`（引用释放）
- **重用分析（Reuse Analysis）**：当值的引用计数为 1 时，函数式的"创建新值"可优化为原地更新
- **无垃圾回收器（No GC）**：无 stop-the-world 停顿，延迟确定
- **无手动管理**：开发者不需要手写 `malloc`/`free`，不需要标注生命周期
- **线程安全**：引用计数操作在必要时为原子操作

### FBIP：函数式但原地执行

```x
// 纯函数式代码——语义上创建新值
function map(f, xs) {
    match xs {
        []       => []
        [x, ...rest] => [f(x), ...map(f, rest)]
    }
}

// Perceus 优化：当 xs 是唯一引用时，零 malloc，原地改写
// 当 xs 是共享引用时，自动 COW（写时复制）
```

### 循环引用处理

通过 `weak` 引用在类型系统层面打破循环，不需要运行时循环检测器：

```x
class TreeNode {
    value:    Integer
    children: List<TreeNode>
    parent:   weak Option<TreeNode>  // 不参与 RC，不形成强引用循环
}
```

**设计准则**：安全的内存管理不应以牺牲性能或开发体验为代价。

---

## 0.6 多范式（Goal 4）

X 支持**函数式编程（FP）** 与 **面向对象编程（OOP）** 的融合，以及过程式和声明式风格。同一个问题，四种写法：

### 函数式（数学 + 管道）

```x
let top_users = users |> filter(.active) |> sort_by(.score) |> take(10)
```

### 声明式（自然语言 where/sort by）

```x
let top_users = users
    where   .active and .score > 80
    sort by .score descending
    take    10
```

### 面向对象（方法链）

```x
let top_users = users.filter(.active).sort_by(.score).take(10)
```

### 过程式（let mutable + for）

```x
function get_top_users(users: List<User>) -> List<User> {
    let mutable result = []
    for u in users {
        if u.active and u.score > 80 {
            result.add(u)
        }
    }
    result |> sort_by(.score) |> take(10)
}
```

| 范式 | 核心特性 |
|------|---------|
| **函数式** | 纯函数、不可变数据、高阶函数、管道运算符 `\|>`、模式匹配 |
| **面向对象** | 类、继承、接口/trait、方法链、封装 |
| **过程式** | 可变变量 `let mutable`、`for`/`while` 循环、顺序执行 |
| **声明式** | `where`/`sort by` 查询语法、表达式导向 |

**设计准则**：为问题选择最合适的范式，而非强迫开发者适应单一范式。

---

## 0.7 参考语言（Goal 5）

X 的设计广泛参考以下语言的优秀实践：

| 参考语言 | 借鉴领域 |
|---------|---------|
| **Python** | 可读性、简洁语法、开发体验 |
| **Rust** | 所有权与安全性、模式匹配、枚举、`Result`/`Option`、cargo 工具链 |
| **Go** | goroutine 并发模型、简洁性、快速编译 |
| **Kotlin** | 表达式导向、空安全、扩展函数、协程 |
| **TypeScript** | 结构化类型、渐进类型化、联合类型 |
| **Swift** | 值类型、可选类型、协议导向编程 |
| **Haskell** | HM 类型推断、类型类、纯函数式、Effect 系统思想 |
| **Scala 3** | FP+OOP 融合、ADT、上下文参数、元编程 |
| **F#** | 管道运算符、类型推断、FP-first 设计、计算表达式 |
| **Zig** | 编译期计算、无隐藏控制流、手动底层控制 |
| **Koka** | Perceus 内存管理、代数效果系统 |

### X 语言的独特之处

- **Perceus 内存管理**：编译时精确引用计数 + 重用分析 + FBIP，无 GC
- **效果系统**：`-> T with Effects` 显式、可组合的副作用追踪，`needs`/`given` 依赖注入
- **全称关键字**：`function`、`mutable`、`implement`、`Integer`、`Boolean`——每个关键字都是真正的英语单词
- **结构化并发**：`together`、`race`、`atomic`/`retry`
- **无异常**：`Option<T>` + `Result<T, E>` + `?`/`?.`/`??` 运算符

---

## 0.8 非目标

X 语言**不**追求：

1. **成为一切语言**：专注于系统编程、应用开发、工具编写，不是 DSL
2. **零学习成本**：好的设计需要一定学习投入，但可读性优先确保门槛尽可能低
3. **完全向后兼容**：愿意在必要时打破兼容性（通过 Preview 标志渐进过渡）
4. **与 C++ 完全 ABI 兼容**：提供 C FFI，但不追求 C++ ABI 兼容
5. **在每个微基准上最快**：追求足够快，同时保持安全性和生产力
6. **最少字符数**：宁可多打几个字符（`function` vs `fn`），也不牺牲可读性
7. **隐式魔法**：所有行为（副作用、错误、可变性）必须显式可见

---

## 0.9 演进原则

### 语言演进策略

```
Stability > Experimentation > Deprecation > Removal
```

- **新特性通过 Preview 标志 gated**：`--preview feature_name` 启用实验性特性
- **收集反馈后稳定化**：经过足够使用后移除 Preview 标志
- **缓慢废弃，长期支持**：废弃的特性给予充分的迁移期
- **重要变更有迁移路径**：提供自动化工具辅助代码迁移

### 版本策略

```
Major.Minor.Patch

Major：不兼容变更（极少）
Minor：新特性（向后兼容）
Patch：bug 修复
```

**设计准则**：稳定性是信任的基础。变更必须谨慎、透明、有迁移路径。

---

**X 语言：为思考的程序员设计——可读、安全、高效。**
