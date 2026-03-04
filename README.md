# X 语言规格说明书

> **本文档是 X 语言的权威规格说明。所有实现必须以本文档为准。**
> 若与其他文档冲突，以本文档为准。

---

## 设计哲学

### 核心原则

X 语言是一门现代通用编程语言，设计遵循以下核心原则：

- **可读性第一**：代码应该像散文一样可读，宁可多打几个字符也不牺牲可读性
- **类型安全**：编译通过就不应出现类型错误，无 null、无异常
- **内存安全**：Perceus 编译时引用计数，无 GC、无手动管理、无泄漏
- **多范式融合**：函数式、面向对象、过程式、声明式自由选择
- **英文全称关键字**：不使用缩写，含义自明
- **不使用奇怪符号**：只用键盘上常见的、一看就懂的符号
- **工具链完整**：`x` CLI 1:1 对标 Cargo，开箱即用

### 四种编程范式

X 语言支持四种编程范式，开发者可以根据场景选择最适合的方式：

#### 函数式（数学 + 管道）
```x
let topUsers = users |> filter(.active) |> sortBy(.score) |> take(10)
```

#### 声明式（自然语言 where/sort by）
```x
let topUsers = users
  where   .active and .score > 80
  sort by .score descending
  take    10
```

#### 面向对象（方法链）
```x
let topUsers = users.filter(.active).sortBy(.score).take(10)
```

#### 过程式（let mutable + for）
```x
function getTopUsers() {
  let mutable result = []
  for u in users {
    if u.active and u.score > 80 { result.add(u) }
  }
  result |> sortBy(.score) |> take(10)
}
```

---

## 1. 词法结构

### 1.1 标识符

- **命名规则**：标识符由字母、数字、下划线和连字符组成，但不能以数字或连字符开头
- **大小写敏感**：`userName` 和 `Username` 是不同的标识符
- **关键字**：不能使用关键字作为标识符
- **示例**：
  ```x
  validName      // 合法
  user_name      // 合法
  user-name      // 合法（推荐）
  123name        // 非法（以数字开头）
  -user          // 非法（以连字符开头）
  ```

### 1.2 数字字面量

- **整数**：支持十进制、十六进制（`0x`前缀）、八进制（`0o`前缀）、二进制（`0b`前缀）
- **浮点数**：支持小数点、科学计数法（`e`或`E`）
- **示例**：
  ```x
  42              // 十进制
  0x2A            // 十六进制
  0o52            // 八进制
  0b101010        // 二进制
  3.14            // 浮点数
  6.02e23         // 科学计数法
  1_000_000       // 数字分隔符（提高可读性）
  ```

### 1.3 字符串字面量

- **普通字符串**：使用双引号 `"`，支持转义字符
- **多行字符串**：使用三个双引号 `"""`，保留换行和缩进
- **插值字符串**：使用 `{}` 嵌入表达式
- **示例**：
  ```x
  "Hello, World!"             // 普通字符串
  """
  多行字符串
  保留格式
  """
  "Hello, {name}!"            // 字符串插值
  ```

### 1.4 布尔值

- `true` 和 `false` 是布尔类型的字面量
- **示例**：
  ```x
  let isActive = true
  let hasError = false
  ```

### 1.5 注释

- **单行注释**：`// 注释内容`
- **多行注释**：`/** 多行注释 */`

### 1.6 符号

X 只使用常见的、含义直观的符号：

| 符号 | 含义 |
|------|------|
| `+` `-` `*` `/` `%` | 算术运算 |
| `=` | 赋值/绑定 |
| `==` `!=` `<` `>` `<=` `>=` | 比较运算 |
| `(` `)` `{` `}` `[` `]` | 分组/块/列表 |
| `.` | 成员访问/路径分隔 |
| `,` `:` `;` | 分隔符 |
| `->` | 函数返回类型/Lambda |
| `=>` | 模式匹配分支 |
| `|>` | 管道运算符 |
| `?` | 错误传播 |
| `?.` | 可选链 |
| `??` | 默认值 |
| `..` `..=` | 范围（不包含/包含末尾） |
| `@` | 注解/装饰器 |

---

## 2. 类型系统

### 2.1 基本类型

```x
Integer     // 整数类型（任意精度）
Float       // 浮点数类型（双精度）
Boolean     // 布尔类型
Character   // 字符类型
String      // 字符串类型
Unit        // 单元类型（无返回值）
Never       // 永不存在的类型（永不返回）
```

### 2.2 复合类型

#### 记录类型（Record）

```x
type Point = {
  x: Float,
  y: Float
}

type User = { id: Integer, name: String, email: String }
```

#### 代数数据类型（ADT）

```x
type Shape =
  | Circle  { radius: Float }
  | Rect    { width: Float, height: Float }
  | Point

type Color = Red | Green | Blue | Custom { r: Integer, g: Integer, b: Integer }
```

#### 列表和字典

```x
List<T>      // 列表（Perceus 下大多数操作可原地执行）
Dictionary<K, V>  // 字典
```

#### 简写语法

```x
[T]           // List<T> 的简写
{K: V}        // Dictionary<K, V> 的简写
```

### 2.3 高级类型

#### 选项类型（代替 null）

```x
Option<T>    // Some(T) | None
```

#### 结果类型（代替异常）

```x
Result<T, E>   // Ok(T) | Err(E)
```

#### 函数类型

```x
(T1, T2) -> T3 // 接受 T1 和 T2，返回 T3 的函数
```

#### 异步类型

```x
Async<T>     // 异步计算结果
```

### 2.4 类型操作

#### copy-with 更新（不可变，产生新值）

```x
let p2 = point with { x: 5.0 }
```

#### Trait 接口

```x
trait Printable {
  function show(): String
}

trait Comparable<T> {
  function compare(other: T): Integer
}
```

---

## 3. 变量与绑定

### 3.1 变量声明

X 中所有绑定默认不可变：

```x
// 不可变绑定 - 默认不可变
let name = "Alice"
let age = 30

// 显式类型标注
let name: String = "Alice"
let age: Integer = 30

// 可变变量 - 使用 let mutable
let mutable count = 0
let mutable isActive: Boolean = true
```

### 3.2 变量解构

```x
// 元组解构
let (x, y) = (10, 20)

// 记录解构
let { name, age } = user

// 列表解构
let [first, ...rest] = [1, 2, 3, 4]
```

### 3.3 变量作用域

```x
function example() {
  let x = 10  // 函数作用域

  if true {
    let y = 20  // 块作用域
  }

  // 错误：y 在此处不可见
  // print(y)
}
```

---

## 4. 表达式

### 4.1 基本表达式

```x
// 字面量表达式
42
3.14
true
"Hello"

// 变量引用
x
name

// 成员访问
user.name
point.x
```

### 4.2 算术表达式

```x
// 基本运算
a + b   // 加法
a - b   // 减法
a * b   // 乘法
a / b   // 除法
a % b   // 取余

// 幂运算
x ^ 2   // x 的平方

// 复合赋值
x += 5  // x = x + 5
x -= 3  // x = x - 3
```

### 4.3 逻辑表达式

```x
// 逻辑运算
a and b   // 逻辑与
a or b    // 逻辑或
not a     // 逻辑非

// 短路评估
x > 0 and y < 10  // 如果 x <= 0，y < 10 不会计算
```

### 4.4 比较表达式

```x
a == b    // 相等
a != b    // 不等
a < b     // 小于
a > b     // 大于
a <= b    // 小于等于
a >= b    // 大于等于
```

### 4.5 类型检查与转换

```x
// 类型检查
x is Integer
y is String

// 类型转换
x as Float
```

### 4.6 函数调用

```x
// 普通函数调用
let result = add(2, 3)

// 方法调用
user.toString()

// 管道操作
data |> process |> filter |> sort
```

---

## 5. 函数

### 5.1 函数定义

X 使用 `function` 关键字（不用缩写）：

```x
// 简单函数（隐式返回）
function add(a: Integer, b: Integer) -> Integer = a + b

// 多行函数
function factorial(n: Integer) -> Integer {
  if n <= 1 {
    1
  } else {
    n * factorial(n - 1)
  }
}

// 类型推断（函数体内可省略类型注解）
function add(a, b) = a + b
```

### 5.2 分段函数（数学风格）

```x
fib(0) = 0
fib(1) = 1
fib(n) = fib(n-1) + fib(n-2)

sign(0) = 0
sign(n) = when n > 0 then 1 else -1
```

### 5.3 函数参数

```x
// 默认参数
function greet(name: String = "World") -> String = "Hello, {name}!"

// 可变参数
function sum(...numbers: List<Integer>) -> Integer {
  numbers |> reduce(0, (acc, x) -> acc + x)
}

// 具名参数调用
greet(name: "Alice")
sum(1, 2, 3)
```

### 5.4 函数返回

```x
// 隐式返回（最后一个表达式的值）
function add(a, b) = a + b

// 显式返回
function findUser(id: Integer) -> Option<User> {
  let user = database.query(id)
  if user != none {
    return Some(user)
  }
  None
}
```

### 5.5 Lambda 函数

```x
// 简单 lambda
(x) -> x * 2

// 多行 lambda
(x, y) -> {
  let z = x + y
  z * z
}

// 点语法简写
users |> map(.name)    // 等价于 map((u) -> u.name)
```

### 5.6 效果声明（副作用可见）

函数的副作用在类型签名中用 `with` 显式声明：

```x
// 有 IO 和抛出错误的效果
function readFile(path: String) -> String with IO, Throws<FileNotFound>

// 纯函数（无效果）
function add(a: Integer, b: Integer) -> Integer

// 多种效果
function processData(url: String) -> Data with Async, IO, Throws<NetworkError>
```

---

## 6. 控制流

### 6.1 条件语句

```x
if condition {
  // then
} else {
  // else
}

// 内联三元
let label = when x > 0 then "positive" else "non-positive"
```

### 6.2 循环

```x
// while 循环
while condition {
  // body
}

// for 循环
for item in iterable {
  // body
}

// 范围循环
for i in 0..10 {    // 0, 1, ..., 9（不包含 10）
  print(i)
}

for i in 0..=10 {   // 0, 1, ..., 10（包含 10）
  print(i)
}
```

### 6.3 模式匹配（必须穷尽）

```x
// 类型联合匹配（编译器保证穷尽）
function area(shape: Shape) -> Float =
  match shape {
    Circle { radius }        => pi * radius ^ 2
    Rect   { width, height } => width * height
    Point                    => 0.0
  }

// 守卫条件（where）
function grade(score: Integer) -> String =
  match score {
    s where s >= 90 => "A"
    s where s >= 75 => "B"
    s where s >= 60 => "C"
    _               => "F"
  }
```

---

## 7. 错误处理：无异常

X 语言没有异常机制。所有错误处理通过类型系统完成：

### 7.1 Option：表示"有或无"

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

### 7.2 Result：表示"成功或失败"

```x
// Result<T, E> = Ok(T) | Err(E)
function readFile(path: String) -> Result<String, IoError> {
  if not exists(path) {
    return Err(IoError.NotFound(path))
  }
  Ok(readBytes(path).decode())
}

// 模式匹配处理
match readFile("config.toml") {
  Ok(content) => parseConfig(content)
  Err(e)      => useDefault()
}

// ? 运算符：错误自动向上传播
function loadConfig() -> Result<Config, IoError> {
  let content = readFile("config.toml")?   // 失败时自动 return Err
  let config = parse(content)?
  Ok(config)
}

// or 运算符：提供默认值
let name = findUser(42) or "unknown"

// catch：捕获特定错误
let result = fetchUser(42)
  |> catch(NotFound, (_) -> User.guest)
```

---

## 8. 类与对象

### 8.1 类定义

```x
class Animal {
  name: String
  age: Integer

  // 构造函数
  new(name: String, age: Integer) {
    this.name = name
    this.age = age
  }

  // 方法
  function greet() -> String = "I'm {name}"

  // 可重写方法
  virtual function birthday() -> Animal =
    this with { age: age + 1 }
}

// 创建实例
let animal = Animal("Bob", 3)
```

### 8.2 继承

```x
class Dog extends Animal {
  breed: String

  // 子构造函数
  new(name: String, age: Integer, breed: String) {
    super(name, age)
    this.breed = breed
  }

  // 重写方法
  override function greet() -> String =
    "Woof! I'm {name}, a {breed}"
}
```

### 8.3 Trait（接口）

```x
trait Printable {
  function show(): String
}

trait Comparable<T> {
  function compare(other: T): Integer
}

// 实现 Trait
class User implements Printable, Comparable<User> {
  name: String
  age: Integer

  function show(): String = "User: {name}"

  function compare(other: User) -> Integer {
    if age < other.age { -1 }
    else if age > other.age { 1 }
    else { 0 }
  }
}
```

### 8.4 抽象类

```x
abstract class Shape {
  abstract function area(): Float
  abstract function perimeter(): Float

  function description() -> String = "A shape"
}

class Circle extends Shape {
  radius: Float

  function area(): Float = pi * radius ^ 2
  function perimeter(): Float = 2 * pi * radius
}
```

### 8.5 属性

```x
class Person {
  // 只读属性
  get fullName(): String = "{firstName} {lastName}"

  // 读写属性
  let mutable _age: Integer = 0
  get age(): Integer = _age
  set age(value: Integer) {
    if value >= 0 {
      _age = value
    }
  }

  // 计算属性
  get isAdult(): Boolean = age >= 18
}
```

---

## 9. 扩展

### 9.1 扩展函数

```x
// 为现有类型添加方法
extension String {
  function toInteger() -> Option<Integer> {
    // 解析字符串为整数
  }

  function isPalindrome() -> Boolean {
    this == reverse(this)
  }
}

// 使用扩展方法
"123".toInteger()       // 结果：Some(123)
"abcba".isPalindrome()  // 结果：true
```

### 9.2 扩展属性

```x
// 为现有类型添加属性
extension Integer {
  get isEven(): Boolean = this % 2 == 0
  get isOdd(): Boolean = this % 2 == 1
}

// 使用扩展属性
4.isEven  // 结果：true
5.isOdd   // 结果：true
```

---

## 10. 泛型

### 10.1 泛型类型

```x
// 泛型类
class Stack<T> {
  private items: List<T> = []

  function push(item: T) {
    items.append(item)
  }

  function pop() -> Option<T> {
    if items.isEmpty() {
      None
    } else {
      Some(items.removeLast())
    }
  }
}

// 使用泛型类
let intStack = Stack<Integer>()
let stringStack = Stack<String>()
```

### 10.2 泛型函数

```x
// 泛型函数
function identity<T>(value: T) -> T = value

function map<T, R>(list: List<T>, transform: (T) -> R) -> List<R> {
  [transform(x) for x in list]
}

// 使用泛型函数
let result = map([1, 2, 3], (x) -> x * 2)  // 结果：[2, 4, 6]
```

### 10.3 泛型 Trait

```x
// 泛型 Trait
trait Comparable<T> {
  function compare(other: T): Integer
}

class Person implements Comparable<Person> {
  function compare(other: Person) -> Integer {
    // 比较逻辑
  }
}
```

### 10.4 类型约束

```x
// 带约束的泛型
function printAll<T: Printable>(items: List<T>) {
  for item in items {
    print(item.show())
  }
}

// 使用带约束的泛型
printAll([Person(), Animal()])
```

---

## 11. 集合与推导式

### 11.1 列表推导（数学集合表示法）

```x
let evens      = [x       | x in 1..100, x mod 2 == 0]
let squares    = [x^2     | x in 1..10]
let names      = [u.name  | u in users, u.active]
let pairs      = [(x,y)   | x in 1..3, y in 1..3, x != y]
```

### 11.2 字典推导

```x
let scoreMap = {u.id: u.score | u in users}
```

### 11.3 范围

```x
0..10     // [0, 1, ..., 9]  不含末尾
0..=10    // [0, 1, ..., 10] 含末尾
```

### 11.4 常用操作

```x
users |> map(.name)
users |> filter(.active)
users |> sortBy(.score)
users |> groupBy(.department)
users |> reduce(0, (acc, u) -> acc + u.score)
```

---

## 12. 异步与并发

X 提供三种并发模型，开发者可根据场景选择：

### 12.1 Async/Await：结构化并发

```x
async function fetchData() -> Data {
  let a = await fetch("/api/users")
  let b = await fetch("/api/posts")
  combine(a, b)
}

// 结构化并发原语
let (users, posts) = await together {
  fetch("/api/users"),
  fetch("/api/posts")
}

// 取最快完成的结果
let result = await race { fetchPrimary(), fetchReplica() }

// 超时
let data = await heavyTask() timeout 5.seconds or fail(Timeout)
```

### 12.2 Go 风格：轻量级协程

```x
go function() {
  // 轻量级协程
  let result = computeHeavy()
  channel.send(result)
}
```

### 12.3 Actor 模型：消息传递

```x
actor Counter {
  let mutable count = 0
  receive Increment => count += 1
  receive GetCount(reply) => reply.send(count)
}
```

---

## 13. 包和模块

### 13.1 包定义

```toml
# x.toml（包描述文件，对标 Cargo.toml）
name = "com.example.utils"
version = "1.0.0"
description = "示例工具库"

[dependencies]
"com.example.core" = "2.0.0"
"org.json" = "1.0.0"

[exports]
"com.example.utils.string"
"com.example.utils.math"
```

### 13.2 模块声明

```x
// 模块声明（文件顶部）
module com.example.utils.string

// 导出符号
export function toCamelCase(s: String) -> String
export function toSnakeCase(s: String) -> String
```

### 13.3 导入模块

```x
// 导入整个模块
import com.example.utils.string

// 导入特定符号
import com.example.utils.string.toCamelCase
import com.example.utils.string.toSnakeCase as snakeCase

// 导入所有符号
import com.example.utils.string.*
```

---

## 14. 关键字速查

| 关键字 | 类别 | 含义 |
|--------|------|------|
| `function` | 结构 | 函数定义（不缩写） |
| `class` / `extends` | 结构 | 类 / 继承 |
| `trait` / `type` | 结构 | 接口 / 类型定义 |
| `let` / `let mutable` | 结构 | 不可变绑定 / 可变绑定 |
| `for` / `in` / `if` / `else` | 结构 | 控制流 |
| `match` | 结构 | 模式匹配 |
| `return` | 结构 | 函数返回 |
| `with` | 结构 | copy-with 更新 / 效果声明 |
| `where` | 结构 | 守卫条件 / 声明式过滤 |
| `async` / `await` | 结构 | 异步操作 |
| `together` / `race` | 并发 | 并发组合 |
| `go` | 并发 | 轻量级协程 |
| `actor` | 并发 | Actor 模型 |
| `import` / `export` | 模块 | 导入导出 |
| `module` | 模块 | 模块声明 |
| `new` | 结构 | 构造函数 |
| `virtual` / `override` | 结构 | 可重写 / 重写 |
| `abstract` | 结构 | 抽象成员 |
| `extension` | 扩展 | 扩展方法/属性 |
| `Some` / `None` | 类型 | Option 构造器 |
| `Ok` / `Err` | 类型 | Result 构造器 |
| `not` / `and` / `or` | 逻辑 | 逻辑运算 |
| `is` / `as` | 类型 | 类型检查 / 转换 |

---

## 15. 内存模型：Perceus

X 采用与 Koka 相同的 Perceus 算法实现内存管理：

### 核心机制

1. **编译时引用计数**：编译器在编译期精确插入 `dup`（引用复制）和 `drop`（引用释放）
2. **重用分析**：当值的引用计数为 1 时，函数式的"创建新值"可优化为原地更新
3. **无 GC**：无 stop-the-world 停顿，延迟确定
4. **无手动管理**：开发者不需要手写 `malloc`/`free`，不需要标注生命周期
5. **线程安全**：引用计数操作在必要时为原子操作

### FBIP：函数式但原地执行

纯函数式代码在对象唯一时自动原地执行，表现得像可变更新但语义上仍是纯函数。

### 循环引用处理

通过 `weak` 引用在类型系统层面打破循环：

```x
class TreeNode {
  value:    Integer
  children: List<TreeNode>
  parent:   weak Option<TreeNode>  // weak：不参与 RC
}
```

---

## 16. 工具链：`x` CLI

X 的工具链 `x` 在功能上 1:1 对标 Rust 的 Cargo：

| Cargo 命令 | X 等价命令 | 功能 |
|-----------|-----------|------|
| `cargo new` | `x new` | 创建新项目 |
| `cargo build` | `x build` | 构建项目 |
| `cargo run` | `x run` | 运行项目 |
| `cargo test` | `x test` | 运行测试 |
| `cargo check` | `x check` | 类型检查（不编译） |
| `cargo fmt` | `x fmt` | 代码格式化 |
| `cargo clippy` | `x lint` | 静态分析 |
| `cargo doc` | `x doc` | 生成文档 |
| `cargo publish` | `x publish` | 发布到仓库 |
| `cargo add` | `x add` | 添加依赖 |
| `cargo bench` | `x bench` | 运行基准测试 |
| `cargo install` | `x install` | 安装二进制工具 |

### 常用命令

```bash
# 构建
x build
x build --release

# 运行 .x 文件（解析 + 解释）
x run examples/hello.x

# 检查语法和类型
x check examples/hello.x

# 编译：完整流水线
x compile examples/hello.x -o hello
x compile examples/hello.x --emit tokens|ast|hir|pir|c|llvm-ir

# 指定后端
x build --target c       # C 后端（默认）
x build --target jvm     # JVM 后端
x build --target dotnet  # .NET 后端

# 运行测试
x test

# 格式化代码
x fmt
```

---

## 17. 多后端架构

X 编译器采用统一中间表示（X IR）+ 多后端架构：

```
                    ┌─── C23 ──→ GCC / Clang / MSVC ──→ Native
                    │
X Source → X IR ────┼─── JVM Bytecode ──→ Java 生态
                    │
                    └─── CIL ──→ .NET CLR 生态
```

### C 后端（主要后端）

- **目标**：X IR → C23 源码 → GCC / Clang / MSVC 编译
- **优势**：最大可移植性，利用成熟 C 编译器的优化
- **适用**：系统编程、嵌入式、性能关键场景

### JVM 后端

- **目标**：X IR → JVM 字节码（`.class` / `.jar`）
- **优势**：接入 Java 生态、跨平台、成熟的 GC 和 JIT
- **适用**：企业应用、Android 开发、大数据（Spark/Flink）

### .NET CLR 后端

- **目标**：X IR → CIL 字节码（`.dll` / `.exe`）
- **优势**：接入 .NET 生态、C# 互操作、Unity 游戏开发
- **适用**：Windows 应用、游戏开发、企业级 .NET 生态

---

## 18. 与 C 的 FFI

X 提供与 C 语言的零开销外部函数接口：

```x
// 声明外部 C 函数
extern "C" {
  function printf(format: CString, ...) -> Integer
  function malloc(size: USize) -> Pointer<Void>
  function free(ptr: Pointer<Void>)
}

// 在 X 中调用
function main() {
  let msg = "Hello from X!\n"
  printf(msg.asCstring())
}
```

---

## 19. 完整示例

### Hello World

```x
function main() {
  println("Hello, World!")
}
```

### Fibonacci

```x
function fibonacci(n: Integer) -> Integer {
  match n {
    0 => 0
    1 => 1
    _ => fibonacci(n - 1) + fibonacci(n - 2)
  }
}

function main() {
  for i in 0..10 {
    println("fib({i}) = {fibonacci(i)}")
  }
}
```

### 选项与结果

```x
type User = { id: Integer, name: String }

function findUser(users: List<User>, id: Integer) -> Option<User> {
  users |> filter(.id == id) |> first
}

type IoError = NotFound { path: String } | PermissionDenied

function readConfig(path: String) -> Result<String, IoError> {
  if not exists(path) {
    return Err(IoError.NotFound(path))
  }
  Ok(readFile(path))
}

function main() {
  let users = [
    User(1, "Alice"),
    User(2, "Bob")
  ]

  // 使用 Option
  let user = findUser(users, 1)
  let name = user?.name ?? "Guest"
  println("Name: {name}")

  // 使用 Result
  match readConfig("config.toml") {
    Ok(content) => println("Config: {content}")
    Err(e) => println("Error: {e}")
  }
}
```

---

## 20. 实现路线图

### Phase 1 · Bootstrap（当前阶段）

- ✅ 词法分析器（Lexer）
- ✅ 语法分析器（Parser）
- ✅ 抽象语法树（AST）表示
- ✅ 树遍历解释器
- ✅ C23 后端（核心功能）
- ✅ `x` CLI 框架

### Phase 2 · Type System

- 🚧 Hindley-Milner 类型推断
- 🚧 类型检查器
- 🚧 多态类型支持
- 🚧 效果系统

### Phase 3 · Perceus RC

- ❌ 实现线性资源演算分析器
- ❌ 插入基本 dup/drop
- ❌ 实现 drop_reuse/reuse 指令对
- ❌ 逃逸分析

### Phase 4 · FBIP

- ❌ 重用分析——配对 drop_reuse 与 reuse 位置
- ❌ 特化代码生成
- ❌ 性能优化

### Phase 5 · 完整后端

- 🚧 LLVM 后端
- 🚧 JVM 后端
- 🚧 .NET 后端

### Phase 6 · 并发与标准库

- ❌ Fiber 运行时
- ❌ wait together/race/timeout
- ❌ Channel 实现
- ❌ 标准库核心模块

---

## 总结

X 语言是一门现代通用编程语言，具有以下特色：

### 语言特色

- **可读性第一**：代码像散文一样可读，不使用缩写，不使用奇怪符号
- **类型安全**：Hindley-Milner 类型推断，无 null、无异常
- **内存安全**：Perceus 编译时引用计数，无 GC、无手动管理
- **多范式融合**：函数式、面向对象、过程式、声明式自由选择
- **效果系统**：副作用在类型签名中可见
- **异步编程**：async/await、together、race、timeout 等自然语法
- **多后端**：C23 / JVM / .NET，一次编写，多处运行
- **完整工具链**：`x` CLI 1:1 对标 Cargo

### 适用场景

- **系统编程**：C 后端 + 零开销 FFI
- **应用开发**：多范式 + 类型安全
- **Web 后端**：异步 + 并发支持
- **数据处理**：集合推导 + 函数式编程
- **教育编程**：自然语法，降低学习难度

X 语言旨在成为一门通用编程语言，既适合初学者学习，也适合专业开发人员构建复杂系统。

---

*本文档由 X 语言核心团队维护，是语言设计的最高准则。*
