# 第2章 语言基础

X 语言的设计理念是"可读性第一"，同时兼顾安全性和表达力。本章介绍 X 语言的基础概念，包括基本语法、变量与绑定、数据类型、运算符和控制流。

## 2.1 基本语法

### 2.1.1 词法结构

X 语言使用 Unicode 字符集，源文件必须使用 UTF-8 编码。代码由词法记号（token）组成，包括关键字、标识符、字面量、运算符和标点符号。

#### 关键字

X 语言使用英文全称作为关键字，避免缩写以提高可读性：

| 类别 | 关键字 |
|------|--------|
| 声明 | `let`, `mutable`, `function`, `async`, `class`, `trait`, `type`, `module`, `const` |
| 控制流 | `if`, `else`, `for`, `in`, `while`, `return`, `match`, `break`, `continue` |
| 效果 | `needs`, `given`, `await`, `with`, `together`, `race`, `atomic`, `retry` |
| 字面量 | `true`, `false`, `None`, `Some`, `Ok`, `Err` |
| 修饰符 | `public`, `private`, `protected`, `internal`, `static`, `abstract`, `final`, `override`, `virtual` |
| 其他 | `import`, `export`, `with`, `where`, `and`, `or`, `not`, `is`, `as`, `weak`, `implement`, `extends`, `new`, `this`, `super` |

#### 标识符

标识符用于命名变量、函数、类型等程序实体：

- 以 Unicode 字母或下划线 `_` 开头
- 可包含字母、数字和连字符 `-`
- 大小写敏感
- 不能与关键字同名

推荐的命名约定：
- 变量、函数、参数：`snake_case` 或 `kebab-case`
- 类型、类、trait：`PascalCase`
- 常量：`SCREAMING_SNAKE_CASE`
- 模块：小写点分隔

#### 注释

X 语言支持三种注释形式：

- 单行注释：`// 注释内容`
- 多行注释：`/** 注释内容 */`（支持嵌套）
- 文档注释：`/// 注释内容` 或 `/*** 注释内容 */`

```x
// 这是单行注释

/**
 * 这是多行注释
 * 可以跨行
 */

/// 这是文档注释
/// 用于生成 API 文档
function add(a: Integer, b: Integer) -> Integer {
    a + b
}
```

### 2.1.2 程序结构

X 语言的程序由模块组成，每个模块包含声明和表达式。程序的执行从 `main` 函数开始（如果存在）。

```x
// 简单的 X 程序
function main() {
    println("Hello, X!")
}
```

## 2.2 变量与绑定

X 语言使用 `let` 关键字声明变量，默认创建不可变绑定。要创建可变绑定，使用 `let mutable`。

### 2.2.1 不可变绑定

不可变绑定一旦创建，其值不能被修改：

```x
let name = "Alice"          // 类型推断为 String
let age: Integer = 30       // 显式类型注解
let pi = 3.14159            // 类型推断为 Float
let is_valid = true         // 类型推断为 Boolean
let numbers = [1, 2, 3]     // 类型推断为 [Integer]
```

### 2.2.2 可变绑定

可变绑定允许后续修改其值：

```x
let mutable count = 0              // 可变 Integer
let mutable name: String = "Bob"  // 可变 String，显式注解
let mutable items: [Integer] = []  // 可变列表

// 修改可变绑定
count += 1
name = "Charlie"
items.push(42)
```

### 2.2.3 解构绑定

`let` 绑定支持模式解构，可以从复杂类型中提取值：

```x
// 元组解构
let (x, y) = get_position()

// 记录解构
let { name, age } = get_person()

// 列表解构
let [first, second, ..rest] = numbers

// 可变解构
let mutable (a, b) = (1, 2)
a += 1
```

### 2.2.4 作用域与遮蔽

X 语言使用词法作用域，变量的可见性由其在源码中的位置决定：

```x
let x = 10

function foo() -> Integer {
    x               // 可以访问外层的 x
}

{
    let y = 20      // y 仅在此块内可见
    println(x + y)  // 可以访问外层的 x
}
// println(y)       // 编译错误：y 不在作用域内
```

内部作用域可以声明与外部作用域同名的变量，遮蔽外部变量：

```x
let x = 10
{
    let x = 20      // 遮蔽外层的 x
    println(x)      // 输出 20
}
println(x)          // 输出 10（外层 x 未被修改）

let value = "hello"
let value = value.length()    // 允许：用新类型遮蔽旧绑定
```

## 2.3 数据类型

X 语言拥有完整的类型系统，基于 Hindley-Milner 类型推断与代数数据类型。

### 2.3.1 基本类型

| 类型 | 描述 | 示例 |
|------|------|------|
| `integer` | 有符号整数（默认机型友好大小） | `42`, `0xFF`, `0b1010` |
| `integer n` | 有符号整数（指定位宽） | `integer 32`, `integer 64` |
| `unsigned integer n` | 无符号整数（指定位宽） | `unsigned integer 32` |
| `float` | 双精度浮点数（默认） | `3.14159`, `1.0e-10` |
| `float n` | 浮点数（指定位宽） | `float 32`, `float 64` |
| `boolean` | 布尔值 | `true`, `false` |
| `string` | UTF-8 字符串 | `"Hello"`, `"${name}"` |
| `character` | Unicode 字符 | `'A'`, `'🎉'` |
| `unit` | 单位类型（无值） | `()` |
| `never` | 永无类型（无返回） | - |

### 2.3.2 复合类型

#### 列表（List）

同构元素的有序集合：

```x
let numbers: [Integer] = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob"]    // [String]
let empty: [Float] = []
```

#### 字典（Dictionary）

键值对集合：

```x
let ages: {String: Integer} = {"Alice": 30, "Bob": 25}
let config: {String: String} = {"host": "localhost", "port": "8080"}
```

#### 元组（Tuple）

固定长度、异构类型的有序集合：

```x
let pair: (Integer, String) = (42, "answer")
let triple: (Float, Float, Float) = (1.0, 2.0, 3.0)
```

#### 记录（Record）

具名字段的积类型：

```x
type Point = {
    x: Float,
    y: Float
}

type Person = {
    name: String,
    age: Integer,
    email: String
}

let origin: Point = { x: 0.0, y: 0.0 }
let alice = Person { name: "Alice", age: 30, email: "alice@example.com" }
```

使用 `with` 语法从现有记录创建新记录：

```x
let p1 = Point { x: 1.0, y: 2.0 }
let p2 = p1 with { x: 5.0 }           // p2 = { x: 5.0, y: 2.0 }
```

#### 联合类型（Union）

和类型，使用 `type` 关键字和 `|` 定义：

```x
type Shape =
    | Circle { radius: Float }
    | Rect { width: Float, height: Float }
    | Point

type Color =
    | Red
    | Green
    | Blue
    | Custom(Integer, Integer, Integer)
```

#### Option 类型

表示可能缺失的值，替代 `null`：

```x
// Option<T> = Some(T) | None

function find(users: [User], id: Integer) -> Option<User> {
    users |> filter(function(u) => u.id == id) |> first
}

let user = find(users, 42)
match user {
    Some(u) => println("Found: ${u.name}")
    None    => println("Not found")
}
```

#### Result 类型

表示可能失败的操作，替代异常：

```x
// Result<T, E> = Ok(T) | Err(E)

function read_file(path: String) -> Result<String, IoError> {
    if not exists(path) {
        return Err(IoError.NotFound(path))
    }
    Ok(read_bytes(path).decode())
}

match read_file("config.toml") {
    Ok(content) => parse_config(content)
    Err(e)      => use_default()
}
```

## 2.4 运算符

X 语言提供丰富的运算符，按优先级从高到低排列：

| 优先级 | 运算符 | 描述 | 示例 |
|--------|--------|------|------|
| 1 | `.` `()` `[]` | 成员访问、函数调用、索引 | `user.name`, `add(1, 2)`, `arr[0]` |
| 2 | `-` `not` `~` | 一元取负、逻辑非、位取反 | `-42`, `not true`, `~0xFF` |
| 3 | `*` `/` `%` | 乘法、除法、取模 | `a * b`, `a / b`, `a % b` |
| 4 | `+` `-` | 加法、减法 | `a + b`, `a - b` |
| 5 | `<<` `>>` | 位左移、位右移 | `a << 2`, `a >> 2` |
| 6 | `&` | 位与 | `a & b` |
| 7 | `^` | 位异或 | `a ^ b` |
| 8 | `|` | 位或 | `a | b` |
| 9 | `..` `..=` | 范围（左闭右开、左闭右闭） | `0..10`, `1..=100` |
| 10 | `<` `>` `<=` `>=` `is` `as` | 比较、类型检查、类型转换 | `a < b`, `x is Integer`, `x as Float` |
| 11 | `==` `!=` | 相等、不等 | `a == b`, `a != b` |
| 12 | `and` | 逻辑与（短路求值） | `a and b` |
| 13 | `or` | 逻辑或（短路求值） | `a or b` |
| 14 | `??` `?.` | 默认值、可选链 | `x ?? 0`, `user?.name` |
| 15 | `|>` | 管道 | `data |> process` |
| 16 | `=` `+=` `-=` `*=` `/=` `%=` `^=` | 赋值、复合赋值 | `x = 42`, `x += 1` |

### 2.4.1 逻辑运算符

X 使用英文关键字作为逻辑运算符，均支持短路求值：

- `and`：逻辑与
- `or`：逻辑或
- `not`：逻辑非

```x
let both = is_valid and is_active
let either = is_admin or has_permission
let inverted = not is_valid
```

### 2.4.2 错误处理运算符

X 提供三个特殊运算符用于错误处理：

- `?`：错误传播（自动返回 `Err` 或 `None`）
- `?.`：可选链（安全访问 `Option` 内部成员）
- `??`：默认值（`Option` 为 `None` 时提供默认值）

```x
// 错误传播
function load_config() -> Result<Config, IoError> {
    let content = read_file("config.toml")?
    let parsed = parse_toml(content)?
    Ok(parsed)
}

// 可选链
let name = user?.name                  // Option<String>
let city = user?.address?.city         // Option<String>（链式）

// 默认值
let name = user?.name ?? "anonymous"
let port = config.get("port")?.parse_integer() ?? 8080
```

### 2.4.3 管道运算符

管道运算符 `|>` 将左侧表达式的结果作为右侧函数的第一个参数传入，使代码更具可读性：

```x
let result = [1, 2, 3, 4, 5]
    |> filter(function(n) => n % 2 == 0)
    |> map(function(n) => n * n)
    |> sum

let processed = raw_data
    |> parse
    |> validate
    |> transform
    |> serialize
```

## 2.5 控制流

### 2.5.1 条件语句

`if` 语句在 X 中是表达式，具有值：

```x
let max = if a > b { a } else { b }

let category = if age < 13 {
    "child"
} else if age < 18 {
    "teen"
} else {
    "adult"
}
```

### 2.5.2 循环语句

#### While 循环

```x
let mutable i = 0
while i < 10 {
    println("i = ${i}")
    i += 1
}
```

#### For 循环

```x
for item in items {
    println(item)
}

for i in 0..10 {
    println("index: ${i}")
}

for (key, value) in dictionary {
    println("${key} = ${value}")
}
```

#### 循环控制

- `break`：立即退出最内层循环
- `continue`：跳过当前迭代的剩余部分，进入下一次迭代

```x
for i in 0..100 {
    if i % 2 == 0 {
        continue
    }
    if i > 50 {
        break
    }
    println(i)
}
```

### 2.5.3 模式匹配

`match` 语句用于模式匹配，支持穷尽性检查：

```x
match command {
    "quit" => {
        println("Goodbye!")
        exit(0)
    }
    "help" => show_help()
    "version" => println("v1.0.0")
    _ => println("Unknown command: ${command}")
}

match shape {
    Circle { radius } => {
        let area = 3.14159 * radius * radius
        println("Circle area: ${area}")
    }
    Rect { width, height } => {
        let area = width * height
        println("Rectangle area: ${area}")
    }
    Point => println("Point has no area")
}

match score {
    n if n >= 90 => "A"
    n if n >= 80 => "B"
    n if n >= 70 => "C"
    n if n >= 60 => "D"
    _ => "F"
}
```

### 2.5.4 返回语句

`return` 语句用于从函数返回：

```x
function find_index(items: [String], target: String) -> Option<Integer> {
    for i in 0..items.length() {
        if items[i] == target {
            return Some(i)
        }
    }
    None
}

function greet(name: String) {
    if name.is_empty() {
        return
    }
    println("Hello, ${name}!")
}
```

## 2.6 表达式与语句

在 X 语言中，大部分构造都是表达式，具有返回值：

- 条件表达式：`if e1 { e2 } else { e3 }`
- 匹配表达式：`match e { p1 => e1, p2 => e2, ... }`
- 块表达式：`{ e1; e2; ...; en }`（值为最后一个表达式）
- 函数调用：`f(e1, e2, ...)`
- 二元运算：`a + b`, `a == b`, `a and b`

语句是不产生值或值被丢弃的构造：
- 声明语句：`let x = e`, `let mutable x = e`
- 赋值语句：`x = e`, `x += e`
- 表达式语句：`e;`（值被丢弃）
- 循环语句：`while e { b }`, `for p in e { b }`
- 返回语句：`return e`

这种设计使得 X 语言更加简洁和表达力强，同时保持了代码的可读性。
