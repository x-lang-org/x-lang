### 说明

本文专门讲解 **X 语言中的表达式体系**，以及这些设计背后的原因。

- 侧重回答三个问题：
  - **什么算“表达式”**，能在什么位置出现？
  - **有哪些核心表达式形态**（算术、逻辑、控制流、集合、异步等）？
  - **为什么 X 要把这么多东西设计成“表达式而不是语句”？**
- 如与总规格说明书 `README.md` 有冲突，以 `README.md` 为准。

---

### 1. 表达式 vs 语句：X 更偏“表达式语言”

在 X 中，只要能产生值，就尽量设计成 **表达式（expression）**，而不是“只能单独成行”的语句：

- **表达式**：有结果，可以嵌套、组合、赋值给变量、作为参数传递。
- **语句**：主要承担结构和作用域，数量尽量减少（如 `function` 定义、`class` 定义、`let` 绑定等）。

对比示例：

```x
// if 作为表达式使用
let label =
  if score >= 60 {
    "pass"
  } else {
    "fail"
  }

// when-then-else 完全是表达式
let level = when score >= 90 then "A"
            else when score >= 75 then "B"
            else "C"
```

**这么做的原因**：

- 让代码更接近数学函数式写法，减少“中间变量 + 多行 if”的样板代码。
- 组合性更强：任何控制流都可以内联在更大的表达式里，而无需拆成多段语句。

---

### 2. 字面量与基础表达式

#### 2.1 数字、字符串、布尔值

```x
42            // Integer
3.14          // Float
true, false   // Boolean
"Hello, X"    // String
"""
多行字符串
保留格式
"""
```

设计理由：

- 与主流语言的字面量形式兼容，降低迁移成本。
- 提供多行 / 插值字符串，方便直接表达文本模板。

#### 2.2 记录、列表、字典等复合字面量

```x
let point = { x: 1.0, y: 2.0 }
let nums  = [1, 2, 3, 4]
let map   = { "alice": 10, "bob": 20 }
```

设计理由：

- 首选 **“所见即所得”** 的直观字面量，而不是必须通过构造函数调用。

---

### 3. 变量、成员与函数调用表达式

#### 3.1 变量与成员访问

```x
user           // 变量引用
user.name      // 成员访问
point.x        // 记录字段
```

设计理由：

- 与 C/Java/JavaScript 等保持一致，`.` 统一表示“从某个值中取出某个部分”，减少符号种类。

#### 3.2 函数与方法调用

```x
let sum = add(1, 2)
let s   = user.toString()

// 链式调用
let result = users
  .filter(.active)
  .sortBy(.score)
  .take(10)
```

设计理由：

- 保留传统“函数名 + 括号 + 实参”形式，符合所有主流语言习惯。
- 同时支持链式风格（类似 JavaScript / Kotlin / C#），方便面向对象 / Fluent API。

---

### 4. 运算符表达式：算术、比较、逻辑与管道

#### 4.1 算术与比较

```x
let a = 1 + 2 * 3
let ok = a >= 0 and a < 10
```

运算符：

- 算术：`+ - * / %`
- 比较：`== != < > <= >=`

设计理由：

- 与 C 家族、Python、JavaScript 等完全一致，避免重新学习。

#### 4.2 逻辑：`not` / `and` / `or`

```x
let ready = not hasError and isConnected
if ready or isAdmin {
  start()
}
```

设计理由：

- 放弃 `!` / `&&` / `||` 这些符号，改用自然语言单词，类似 Python / SQL，语义更清晰，尤其在复杂条件中可读性更高。

#### 4.3 管道运算符 `|>`

```x
let topUsers =
  users
    |> filter(.active)
    |> sortBy(.score)
    |> take(10)
```

设计理由：

- 借鉴 F#、Elixir 等语言的 `|>`，把“数据流向”从左到右写出来，符合人眼阅读习惯。
- 避免深层嵌套括号，例如 `take(10, sortBy(score, filter(active, users)))`。

---

### 5. 函数与 Lambda 表达式

#### 5.1 Lambda：`(参数) -> 表达式`

```x
let double = (x) -> x * 2

let scores =
  users |> map((u) -> u.score)
```

设计理由：

- 使用 `->` 表示“从参数到结果”，接近数学函数写法，且在 TypeScript / Kotlin / Scala 等中已经广泛使用。

#### 5.2 多行 Lambda

```x
let sumOfSquares =
  numbers |> reduce(0, (acc, x) -> {
    let y = x * x
    acc + y
  })
```

设计理由：

- 允许在表达式内就地写出小块逻辑，而不必提升为顶层函数，兼顾局部性与可读性。

---

### 6. 控制流表达式：`if` / `when` / `match`

#### 6.1 `if` 作为表达式

```x
let label =
  if x > 0 {
    "positive"
  } else {
    "non-positive"
  }
```

设计理由：

- 对齐 Rust、Kotlin 等“表达式导向”语言，使分支可以出现在任何需要值的地方。

#### 6.2 `when ... then ... else ...`

```x
let level =
  when score >= 90 then "A"
  else when score >= 75 then "B"
  else "C"
```

设计理由：

- 避免传统三元运算符 `condition ? a : b` 的符号负担。
- 读起来更像自然语言：“当…时，然后…，否则…”，适合作为更长表达式的一部分。

#### 6.3 `match` 模式匹配

```x
function area(shape: Shape) -> Float =
  match shape {
    Circle { radius }        => pi * radius ^ 2
    Rect   { width, height } => width * height
    Point                    => 0.0
  }
``>

设计理由：

- 借鉴 Rust、Haskell、Scala 的 `match`，用 **穷尽匹配** 代替大量 `if-else` 链。
- 使代数数据类型（`type Shape = ...`）在语法层面一等公民，表达错误 / 状态 / 业务分支更安全。

---

### 7. 集合与推导式表达式

#### 7.1 列表 / 字典推导

```x
let evens   = [x       | x in 1..100, x mod 2 == 0]
let squares = [x^2     | x in 1..10]
let names   = [u.name  | u in users, u.active]

let scoreMap = {u.id: u.score | u in users}
```

设计理由：

- 直接参考数学集合表示和 Python 的列表推导式，适合描述“从一个集合变成另一个集合”的变换。
- 比链式 `map/filter` 在简单场景下更短、更直观。

#### 7.2 范围表达式

```x
0..10    // [0, 1, ..., 9]
0..=10   // [0, 1, ..., 10]
```

设计理由：

- 用 `..` / `..=` 明确是否包含末尾，比“魔法数字” + `< / <=` 组合更容易一眼看出边界。

---

### 8. Option / Result 与错误处理表达式

虽然 `Option` 和 `Result` 不是关键字，但在 X 的表达式世界里非常重要：

```x
type User = { id: Integer, name: String }

function findUser(users: List<User>, id: Integer) -> Option<User> {
  users |> filter(.id == id) |> first
}

let user  = findUser(users, 42)
let name  = user?.name ?? "Guest"
```

以及：

```x
type IoError = NotFound { path: String } | PermissionDenied

function readConfig(path: String) -> Result<String, IoError> {
  if not exists(path) {
    return Err(IoError.NotFound(path))
  }
  Ok(readFile(path))
}

function loadConfig() -> Result<Config, IoError> {
  let content = readConfig("config.toml")?   // ? 表达式：失败时向上传播
  parseConfig(content)?
}
```

设计理由：

- **无异常**：错误处理完全通过表达式和类型组合完成，避免隐藏控制流。
- `?` / `??` / `?.` 等操作符让 Option/Result 表达式在多数场景下依然保持简洁。

---

### 9. 声明式风格表达式：管道 + where + sort by

```x
let topUsers =
  users
    where   .active and .score > 80
    sort by .score descending
    take    10
```

设计理由：

- 受 SQL / LINQ 启发，为 X 提供 **接近自然语言的声明式管道**：
  - `where` 对应过滤条件
  - `sort by`、`take` 等都是表达式链的一部分
- 让“描述要什么结果”比“详细写循环和 if”更自然，特别适合数据处理、业务逻辑代码。

---

### 10. 设计总结：为什么 X 这么“偏爱表达式”

综合来看，X 在表达式设计上的几个核心目标是：

- **1. 代码像散文一样可读**  
  尽可能用完整单词（`function`、`not`、`and`、`or`、`match`、`where`、`async` / `await`），而不是大量缩写与符号。阅读体验更接近英文说明书而非“符号谜题”。

- **2. 尽量一切皆表达式，减少语句种类**  
  `if` / `when` / `match`、推导式、错误传播 `?`、管道 `|>` 等都可以组成更大的表达式，使逻辑可以局部化表达，不必打碎到多处。

- **3. 对齐主流语言的直觉，降低学习成本**  
  算术 / 比较 / 调用 / 类 / 模块等基础部分与 Python、C、JavaScript、Rust、Kotlin 等高度相似，只在“可读性确实更好”的地方（逻辑运算、声明式 where、管道、match）做了有意识的改进。

- **4. 让类型与控制流紧密结合**  
  通过 `match` + ADT、`Option` / `Result`、`where` 守卫等，让“所有可能分支”在语法上被清晰枚举，避免隐式异常和隐藏控制流。

这套表达式设计，是让 X 同时具备：

- 函数式语言的表达力与组合性，
- 命令式 / 面向对象语言的熟悉感，
- 以及接近自然语言的可读性。


