# X 语言规范文档

> 本文档是 X 语言的正式语法与语义定义。所有语言行为、语法规则和类型系统规则以本文档为准。

---

## 语言概述

X 是一门**现代的、通用的编程语言**，适用于从底层系统编程到上层应用开发的全栈场景。

### 核心设计特征

| 特性 | 描述 |
|------|------|
| **可读性第一** | 所有关键字使用英文全称（`function`、`mutable`、`match`、`implement`） |
| **类型安全** | Hindley-Milner 类型推断、代数数据类型、穷尽模式匹配 |
| **无 null、无异常** | 用 `Option<T>` 代替 null，用 `Result<T, E>` 代替异常 |
| **内存安全** | Perceus 编译时引用计数——无 GC 停顿、无手动管理 |
| **多范式** | 函数式 + 面向对象 + 过程式 + 声明式 |
| **效果系统** | 函数副作用在类型签名中显式声明（`with Effects`） |
| **结构化并发** | `async`/`await`、`together`、`race`、`atomic`/`retry` |
| **多后端** | Zig → 原生/Wasm、JVM 字节码、.NET CIL、JavaScript |

---

## 1. 词法结构

### 1.1 字符集

- 源代码使用 **UTF-8** 编码
- 标识符支持 Unicode 字母和数字
- 注释：`//` 单行注释，`/* */` 多行注释

### 1.2 标识符

```x
let name = "X"
let snake_case = 1
let camelCase = 2
let _private = 3
```

### 1.3 关键字（全称原则）

| 类别 | 关键字 |
|------|--------|
| 声明 | `let`, `mutable`, `const` |
| 函数 | `function`, `return`, `async`, `await` |
| 控制流 | `if`, `else`, `match`, `for`, `while`, `loop` |
| 类型 | `type`, `class`, `trait`, `implement`, `enum`, `record` |
| 模块 | `module`, `import`, `export`, `pub` |
| 异常处理 | `try`, `throw` (内置效果) |
| 效果系统 | `needs`, `given`, `with` |
| 并发 | `together`, `race`, `atomic`, `retry` |
| 其他 | `where`, `sort by`, `is`, `can`, `wait` |

### 1.4 字面量

```x
let int = 42              // 整数
let float = 3.14         // 浮点数
let bool = true          // 布尔值
let str = "Hello"        // 字符串
let char = 'A'           // 字符
let list = [1, 2, 3]     // 数组
let dict = {a: 1, b: 2}  // 字典
```

---

## 2. 类型系统

### 2.1 基本类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `Integer` | 整数 | `42` |
| `Float` | 浮点数 | `3.14` |
| `Boolean` | 布尔值 | `true` / `false` |
| `Character` | 单个字符 | `'A'` |
| `String` | 字符串 | `"Hello"` |
| `Unit` | 空值 | `()` |
| `Never` | 永不返回 | `panic()` 返回 |

### 2.2 复合类型

```x
// 数组
let arr: List<Integer> = [1, 2, 3]

// 字典
let dict: Map<String, Integer> = {a: 1, b: 2}

// 元组
let tuple: (Integer, String) = (42, "hello")

// 函数类型
let add: (Integer, Integer) -> Integer = (a, b) -> a + b
```

### 2.3 Option 和 Result

```x
// Option<T> - 表示"有或无"
type Option<T> = Some(T) | None

// Result<T, E> - 表示"成功或失败"
type Result<T, E> = Ok(T) | Err(E)
```

### 2.4 代数数据类型

```x
// 枚举（sum type）
enum Color {
    Red
    Green
    Blue
    RGB(Integer, Integer, Integer)
}

// 记录（product type）
record Person {
    name: String
    age: Integer
}
```

### 2.5 泛型

```x
function first<T>(list: List<T>) -> Option<T> {
    match list {
        [] => None
        [x, ...] => Some(x)
    }
}
```

---

## 3. 表达式

### 3.1 算术与逻辑

```x
let sum = a + b
let diff = a - b
let product = a * b
let quotient = a / b
let modulo = a % b

let and_cond = a and b
let or_cond = a or b
let not_cond = not a
```

### 3.2 管道运算符

```x
let result = numbers
    |> filter(is_even)
    |> map(square)
    |> take(10)
```

### 3.3 函数调用

```x
let result = foo(arg1, arg2)
let method_result = obj.method()
```

### 3.4 成员访问

```x
let x = record.field
let elem = array[0]
```

---

## 4. 语句与声明

### 4.1 变量绑定

```x
let x = 42              // 不可变
let mutable y = 0       // 可变
y = 10                  // 赋值
```

### 4.2 控制流

```x
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// 三元表达式
let max = if a > b then a else b
```

### 4.3 循环

```x
for item in collection {
    println(item)
}

while condition {
    // ...
}

loop {
    // 无限循环，需配合 break
    if done { break }
}
```

---

## 5. 函数

### 5.1 函数定义

```x
function greet(name: String) -> String {
    return "Hello, " + name
}

// 单表达式函数
function square(x: Integer) -> Integer = x * x

// 多返回值
function div_mod(a: Integer, b: Integer) -> (Integer, Integer) {
    return (a / b, a % b)
}
```

### 5.2 Lambda 表达式

```x
let add = (a: Integer, b: Integer) -> Integer = a + b
let apply = (f, x) -> f(x)
```

### 5.3 数学风格函数

```x
function f(x) = x * x  // 数学风格，单表达式
function g(x, y) = x + y
```

---

## 6. 类与接口

### 6.1 类定义

```x
class Person {
    name: String
    age: Integer

    function new(name: String, age: Integer) -> Self {
        Self { name, age }
    }

    function greet(self) -> String {
        "Hello, I'm " + self.name
    }
}
```

### 6.2 接口（trait）

```x
trait Printable {
    function to_string(self) -> String
}

trait Comparable<T> {
    function compare(self, other: T) -> Integer
}
```

### 6.3 实现

```x
implement Printable for Person {
    function to_string(self) -> String {
        "Person({name: " + self.name + ", age: " + self.age + "})"
    }
}
```

---

## 7. 效果系统

### 7.1 效果声明

```x
effect Io {
    function read_file(path: String) -> String
    function write_file(path: String, content: String) -> Unit
}
```

### 7.2 效果处理

```x
function read_config() -> String with Io {
    needs Io.read_file("config.toml")
}
```

### 7.3 needs/given 依赖注入

```x
function process_data(data: String) -> Result<String, Error> with Io {
    let content = needs Io.read_file("data.txt")
    // ...
}
```

---

## 8. 模块系统

### 8.1 模块声明

```x
module math.utils

export function add(a: Integer, b: Integer) -> Integer = a + b
export const PI = 3.14159
```

### 8.2 导入

```x
import std.collections.HashMap
import my.module { func_a, func_b }
```

---

## 9. 模式匹配

### 9.1 match 表达式

```x
let result = match value {
    0 => "zero"
    1 => "one"
    n if n > 10 => "large"
    _ => "other"
}
```

### 9.2 解构

```x
match point {
    (0, 0) => "origin"
    (x, 0) => "on x-axis"
    (0, y) => "on y-axis"
    (x, y) => "elsewhere"
}

match list {
    [] => "empty"
    [first, ...rest] => "first: " + first
}
```

### 9.3 穷尽检查

编译器确保 `match` 表达式覆盖所有可能的情况。

---

## 10. 内存模型

### 10.1 Perceus 引用计数

- **dup**：复制引用，增加引用计数
- **drop**：释放引用，减少引用计数
- **reuse**：当引用计数为 1 时，原地更新

### 10.2 弱引用

```x
class Node {
    value: Integer
    next: Option<Node>
    parent: weak Option<Node>  // 不参与 RC
}
```

### 10.3 FBIP（Functionally But In Place）

函数式代码在唯一引用时可原地更新，无需分配新内存。

---

## 11. 错误处理

### 11.1 Option 用法

```x
function find(users: List<User>, id: Integer) -> Option<User> {
    users |> filter(.id == id) |> first
}

match find(users, 42) {
    Some(u) => println(u.name)
    None    => println("Not found")
}

// 便捷运算符
let name = user?.name ?? "anonymous"
```

### 11.2 Result 用法

```x
function read_file(path: String) -> Result<String, IoError> {
    // ...
}

// ? 运算符传播错误
function load() -> Result<Config, IoError> {
    let content = read_file("config.toml")?
    parse(content)?
}
```

---

## 12. 并发

### 12.1 async/await

```x
async function fetch_data(url: String) -> Result<String, NetworkError> {
    let response = await http_get(url)
    Ok(response.body)
}
```

### 12.2 并发组合

```x
// 同时执行多个任务
let results = together {
    task_a()
    task_b()
    task_c()
}

// 竞态
let winner = race {
    fast_task()
    slow_task()
}
```

### 12.3 原子操作

```x
let counter = atomic 0
atomic { counter += 1 }
```

---

## 语法速查表

```x
// 变量
let x = 42
let mutable y = 0

// 函数
function add(a: Integer, b: Integer) -> Integer = a + b
let add = (a, b) -> a + b

// 控制流
if condition { } else { }
for item in items { }
while condition { }
match value { pattern => result }

// 类
class Point { x: Integer, y: Integer }
trait Drawable { function draw(self) -> Unit }

// 效果
function foo() -> T with Io { needs Io.read_file("...") }
```

---

## 与其他语言对比

| 特性 | X | Rust | Python | TypeScript |
|------|---|------|--------|------------|
| 关键字风格 | 全称 (`function`) | 缩写 (`fn`) | 简洁 (`def`) | 简洁 |
| 空安全 | Option<T> | Option<T> | None | undefined |
| 错误处理 | Result<T, E> | Result<T, E> | 异常 | 异常 |
| 内存管理 | Perceus | 所有权 | GC | GC |
| 类型推断 | HM | HM | 局部 | 渐进 |

---

## 参考

- 完整规范：[spec/](spec/)
- 设计目标：[DESIGN_GOALS.md](../DESIGN_GOALS.md)
- 示例程序：[examples/](examples/)

---

*最后更新：2026-03-13*
