### 说明

这篇文章专门介绍 **X 语言的关键字及其用法示例**，并参考了 `languages.md` 中主流语言（如 Python、JavaScript、C、Rust、Kotlin 等）的关键字设计，从中选取了 **可读性最强、跨语言最容易理解** 的风格来统一 X 的关键字体系。

选择这些关键字的总体原则：

- **使用完整英文单词**（`function`、`not`、`and`、`or`），避免缩写（对比 Python 的 `def`、Rust 的 `fn`）
- **优先选用“行业共识”的单词**（如 `if` / `else`、`class`、`async` / `await`、`import` / `export`）
- **尽量避免符号化操作符**（如 `&&` / `||`），用自然语言单词提高可读性
- **关键字语义“顾名思义”**，不需要记忆额外规则

下文会按类别依次介绍核心关键字，并配上简短示例。

---

### 1. 变量与绑定：`let` / `let mutable` / `const`

#### 1.1 设计理由

- 很多语言使用 `var` / `let` / `const`（参见 JavaScript、Kotlin、Swift），`let` 已经是事实标准。
- X 延续 `let` 的习惯，但改用 **“完整短语”** `let mutable` 明确可变语义，比 C/C++ 中在类型前加 `const`/省略更直观。
- X 额外提供 **`const`** 表示“**编译期常量**”，其值在编译阶段就完全确定，便于优化与语义清晰。

#### 1.2 用法示例：运行期绑定

```x
// 不可变绑定（默认）
let name = "Alice"
let age: Integer = 30

// 可变变量
let mutable count = 0
count += 1
```

也可以配合解构使用：

```x
// 元组解构
let (x, y) = (10, 20)

// 记录解构
let { name, age } = user
```

#### 1.3 `const`：编译期常量

- **含义**：`const` 声明的绑定在 **编译期即确定值**，不能依赖运行时输入或 I/O。
- **典型用途**：版本号、协议标识、固定缓冲区大小等永远不会变化的配置。

```x
const MAX-RETRY-COUNT: Integer = 3
const APP-NAME: String = "x-lang"

function connect() {
  for i in 0..MAX-RETRY-COUNT {
    // 尝试连接，使用编译期常量控制重试次数
  }
}
```

与 `let` 的关系：

- `const`：值在编译期就必须可计算出来，不能修改，编译器可以进行内联、折叠等优化。
- `let`：不可变绑定，但值可以在运行期计算（比如来自函数调用或 I/O）。
- `let mutable`：运行期可修改的变量。

---

### 2. 类型与抽象：`type` / `trait`

#### 2.1 `type`

- **对比**：很多语言有 `type`（TypeScript、Haskell）或 `typedef`（C），而 X 采用简洁的 `type`。
- **用途**：定义别名、记录类型和代数数据类型，语义一目了然。

```x
// 记录类型
type Point = {
  x: Float,
  y: Float
}

// 代数数据类型（ADT）
type Shape =
  | Circle { radius: Float }
  | Rect   { width: Float, height: Float }
  | Point
```

#### 2.2 `trait`

- **对比**：等价于许多语言中的 interface（Java、TypeScript、Go）的概念。
- **命名理由**：选择 Rust 等语言中 `trait` 这一 **语义清晰的词**，强调“可被类型实现的一组行为”。

```x
trait Printable {
  function show(): String
}

trait Comparable<T> {
  function compare(other: T): Integer
}
```

---

### 3. 函数：`function` / `return` / `->`

#### 3.1 `function`

- **对比**：
  - Python / Ruby 使用 `def`
  - Rust 使用 `fn`
  - JavaScript / C 使用 `function` / 函数声明
- **设计选择**：X 采用 **完整单词 `function`**，对初学者和跨语言读者都更直观。

```x
// 单行函数（隐式返回）
function add(a: Integer, b: Integer) -> Integer = a + b

// 多行函数
function factorial(n: Integer) -> Integer {
  if n <= 1 {
    1
  } else {
    n * factorial(n - 1)
  }
}
```

#### 3.2 `return`

- 与 C / Java / Python 等主流语言一致，用于 **提前返回**，语义无歧义。

```x
function findUser(id: Integer) -> Option<User> {
  let user = database.query(id)
  if user != None {
    return Some(user)
  }
  None
}
```

#### 3.3 箭头 `->`（返回类型与 Lambda）

- 参考 TypeScript / Kotlin / Scala，使用 `->` 表示“从参数到返回值”的映射，更接近数学函数记号。

```x
// Lambda
let double = (x) -> x * 2

let sumOfSquares =
  numbers |> map((n) -> n * n)
          |> reduce(0, (acc, x) -> acc + x)
```

---

### 4. 控制流：`if` / `else` / `when ... then ... else ...` / `while` / `for ... in`

#### 4.1 `if` / `else`

- 这些关键字在 C、Java、Python、JavaScript 等几乎所有主流语言中都存在，是 **跨语言最容易识别** 的控制流。

```x
let label =
  if x > 0 {
    "positive"
  } else {
    "non-positive"
  }
```

#### 4.2 `when ... then ... else ...`

- 受数学“条件表达式”启发，使用完整单词 `when` / `then`，比三元运算符 `?:` 更可读。

```x
let label = when x > 0 then "positive" else "non-positive"
```

#### 4.3 `while`

```x
let mutable i = 0
while i < 5 {
  println(i)
  i += 1
}
```

#### 4.4 `for ... in`

- 对应 Python / Rust / Kotlin 等语言的 `for ... in`，相比传统 C 风格 `for(;;)` 更直观。

```x
for user in users {
  println(user.name)
}

for i in 0..=10 {   // 含 10
  println(i)
}
```

---

### 5. 模式匹配与守卫：`match` / `where` / `_`

#### 5.1 `match`

- 借鉴 Rust、Haskell、Scala 的 `match`，命名清晰地表达“匹配多个分支”的含义。

```x
function area(shape: Shape) -> Float =
  match shape {
    Circle { radius }        => pi * radius ^ 2
    Rect   { width, height } => width * height
    Point                    => 0.0
  }
```

#### 5.2 `where`

- 参考 SQL 和数学中的 where 子句，在 `match` 和声明式查询中用于添加条件，**语义接近自然语言**。

```x
// 守卫条件
function grade(score: Integer) -> String =
  match score {
    s where s >= 90 => "A"
    s where s >= 75 => "B"
    s where s >= 60 => "C"
    _               => "F"
  }
```

#### 5.3 `_`（通配符）

- 与 Haskell、Rust、Scala 等保持一致，用 `_` 作为“我不在乎这个值”的统一标记。

```x
match option {
  Some(v) => println(v)
  _       => println("none")
}
```

---

### 6. 类型检查与转换：`is` / `as`

#### 6.1 `is`

- 类似 C#、Python 中的 `is` 语义，用自然语言单词而不是符号进行类型判断。

```x
if value is String {
  println("it's a String")
}
```

#### 6.2 `as`

- 对应许多语言中的类型转换（C# 的 `as`、TypeScript 的 `as`），统一采用自然语言关键字。

```x
let x: Integer = 42
let y: Float   = x as Float
```

---

### 7. 布尔逻辑：`not` / `and` / `or`

#### 7.1 设计理由

- 大多数语言使用 `&&` / `||` / `!`，对初学者不够直观。
- 借鉴 Python、SQL 等，采用完整单词 `not` / `and` / `or`，与英语逻辑表达一致。

```x
let ok = not hasError and isReady

if isAdmin or isOwner {
  grantAccess()
}
```

---

### 8. 异步与并发：`async` / `await` / `together` / `race` / `go` / `actor`

#### 8.1 `async` / `await`

- 参考 C#、JavaScript、Rust、Kotlin Dart 等主流语言，`async` / `await` 已经形成事实标准。
- 直接沿用这两个关键字，降低跨语言迁移成本。

```x
async function fetchData() -> Data {
  let users = await fetch("/api/users")
  let posts = await fetch("/api/posts")
  combine(users, posts)
}
```

#### 8.2 `together` / `race`

- 使用自然语言短语代替库函数名（例如很多语言用 `Promise.all` / `race`，或 `Task.WhenAll`）。
- X 使用 `together` 强调“一起执行”，`race` 强调“比谁先完成”，语义非常直观。

```x
let (users, posts) = await together {
  fetch("/api/users"),
  fetch("/api/posts")
}

let fastest = await race { fetchPrimary(), fetchReplica() }
```

#### 8.3 `go`

- 致敬 Go 语言中的 `go` 关键字，表示启动轻量级协程。由于已经有广泛认知，沿用该短关键字即可。

```x
go function() {
  let result = computeHeavy()
  channel.send(result)
}
```

#### 8.4 `actor`

- 直接用领域术语 `actor`，与 Akka、Erlang/Elixir 的 Actor 模型概念对齐。

```x
actor Counter {
  let mutable count = 0
  receive Increment        => count += 1
  receive GetCount(reply)  => reply.send(count)
}
```

---

### 9. 模块与导入导出：`module` / `import` / `export`

#### 9.1 `module`

- 与许多语言（F#, OCaml、Python 的 module 概念）保持一致，用 `module` 标记模块声明。

```x
module com.example.utils.string
```

#### 9.2 `import` / `export`

- 借鉴 JavaScript / TypeScript、Python 等语言，使用非常直观的两个单词说明“导入 / 导出”。

```x
// 导出符号
export function toCamelCase(s: String) -> String
export function toSnakeCase(s: String) -> String

// 导入整个模块
import com.example.utils.string

// 导入特定函数
import com.example.utils.string.toCamelCase
import com.example.utils.string.toSnakeCase as snakeCase
```

---

### 10. 类与对象：`class` / `extends` / `new` / `virtual` / `override` / `abstract` / `extension`

#### 10.1 `class` / `extends` / `new`

- 完全沿用 Java / C# / TypeScript 等主流语言的命名，降低理解门槛。

```x
class Animal {
  name: String
  age: Integer

  new(name: String, age: Integer) {
    this.name = name
    this.age = age
  }

  function greet() -> String = "I'm {name}"
}

class Dog extends Animal {
  breed: String
}
```

#### 10.2 `virtual` / `override` / `abstract`

- 与 C#、C++ 等语言相同的术语，用来描述多态行为，语义清晰。

```x
abstract class Shape {
  abstract function area(): Float
  abstract function perimeter(): Float
}

class Circle extends Shape {
  radius: Float
  override function area(): Float = pi * radius ^ 2
}
```

#### 10.3 `extension`

- 借鉴 C# 和 Kotlin 的扩展方法概念，用 `extension` 作为统一关键字，替代魔法语法或装饰器。

```x
extension String {
  function isPalindrome() -> Boolean {
    this == reverse(this)
  }
}
```

---

### 11. 推导式与管道：`in` / 管道运算符 `|>`

#### 11.1 推导式中的 `in`

- 与 Python 列表推导、SQL 语义一致，`in` 表示“从某个集合中取值”。

```x
let evens   = [x       | x in 1..100, x mod 2 == 0]
let scores  = {u.id: u.score | u in users}
```

#### 11.2 管道运算符 `|>`

- 不是关键字，但在 X 中非常重要，借鉴 F#、Elixir 等语言，使数据流表达更接近“从左到右”的自然阅读顺序。

```x
let topUsers = users
  |> filter(.active)
  |> sortBy(.score)
  |> take(10)
```

---

### 12. 小结：从主流语言中“借鉴可读性最强的关键字”

综合 `languages.md` 中各语言的关键字设计，可以看到几条共识：

- **控制流关键字高度收敛**：几乎所有语言都使用 `if` / `else`、`for` / `while`，X 直接沿用这些“行业标准”。
- **命名更偏自然语言**：与大量使用缩写的语言（`def`、`fn`、`sub`）相比，X 选择 `function`、`not` / `and` / `or`、`match` / `where` 等完整单词，使代码接近英文散文。
- **异步与模块关键字与现代生态对齐**：`async` / `await`、`import` / `export`、`module` 与 C#、JavaScript、TypeScript 等现代语言保持一致，方便开发者迁移。
- **并发与 OOP 使用领域通用术语**：`actor`、`trait`、`extension` 等直接采用在多门语言中已经广泛使用的专业术语，避免发明新的名词。

因此，X 语言的关键字表是在 **不改变现有规格的前提下**，从 Python、JavaScript、C#、Rust、Kotlin 等语言的实践中，挑选出 **最易读、最直观、跨语言迁移成本最低** 的单词作为统一的关键字体系，并配合自然语言风格的控制流（如 `when ... then ... else ...`、`where`）和管道 `|>`，让代码尽可能接近人类日常阅读习惯。


