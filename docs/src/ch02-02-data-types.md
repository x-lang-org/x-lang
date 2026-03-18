# 数据类型

在 X 语言中，每个值都属于某种数据类型，这告诉 X 语言正在指定哪种类型的数据，以便它知道如何处理这些数据。我们将介绍两种数据类型子集：基本类型和复合类型。

在继续之前，请记住 X 语言是静态类型语言，这意味着它必须在编译时知道所有变量的类型。通常编译器可以根据值及其使用方式推断出我们想要使用的类型。

## 基本类型

基本类型是由语言核心定义的简单类型。X 语言有以下基本类型：

### 整数类型

整数是没有小数部分的数字。X 语言中的基础整数类型对外有一对名称：

- **`integer`**：值类型（primitive），用于绝大多数计算场景
- **`Integer`**：引用类型（boxed），在需要以对象形式存在（如放入统一的对象容器）时使用

抽象上表示数学意义上的整数（…，-2，-1，0，1，2，…），规格上定义为 **任意精度整数**：理论上只受内存限制，不会像传统 32/64 位整型那样静默溢出。

```x
let a: integer = 42
let big: integer = 1_000_000_000_000_000

let sum: integer = a + big
let diff = big - a      // 类型推断为 integer
```

#### 固定位宽整数：完整英文短语形式

在需要与底层平台或其他语言精确对齐时，X 也提供了 **固定位宽整数类型的内置别名**，并且名称使用**完整英文短语 + 空格**，避免 `i8` / `u64` 这类缩写和符号化命名：

- 有符号：如
  - `signed 8bit integer`
  - `signed 16bit integer`
  - `signed 32bit integer`
  - `signed 64bit integer`
  - `signed 128bit integer`
- 无符号：如
  - `unsigned 8bit integer`
  - `unsigned 16bit integer`
  - `unsigned 32bit integer`
  - `unsigned 64bit integer`
  - `unsigned 128bit integer`

示例：

```x
let small: signed 8bit integer    = 127
let port:  unsigned 16bit integer = 8080
let size:  unsigned 64bit integer = 1_000_000_000
let mask:  unsigned 128bit integer = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF
```

整数可以写成以下形式：

| 数字字面量 | 示例 |
|-----------|------|
| 十进制 | `98_222` |
| 十六进制 | `0xff` |
| 八进制 | `0o77` |
| 二进制 | `0b1111_0000` |

注意：你可以使用下划线 `_` 作为分隔符以便于阅读，例如 `1_000_000`。

### 浮点类型

X 语言中的基础浮点类型也有一对名称：

- **`float`**：值类型（primitive），默认对应 64 位双精度
- **`Float`**：引用类型（boxed），用于需要对象语义的场合

默认对应 **双精度浮点数**（与大多数现代语言的 `double` 类似），用于近似实数计算：物理量、评分、概率、统计等。

```x
let pi: float = 3.1415926535
let radius: float = 2.5

let area: float = pi * radius ^ 2
```

#### 固定位宽浮点：`32bit float` / `64bit float` 与十进制 decimal

类似整数，X 为浮点数提供了使用简洁完整短语的内置别名（值类型），分为二进制浮点和十进制浮点两类：

- **二进制浮点**
  - `32bit float`：对应 32 位单精度
  - `64bit float`：对应 64 位双精度
- **十进制浮点**
  - `32bit decimal`：32 位十进制浮点
  - `64bit decimal`：64 位十进制浮点
  - `128bit decimal`：128 位十进制浮点，适合高精度金融 / 结算场景

示例：

```x
let x: 32bit float = 1.0
let y: 64bit float = 3.1415926535

let price: 64bit decimal    = 123.45
let amount: 128bit decimal  = 1_000_000_000_000.0001
```

### 布尔类型

#### 语义

布尔类型为 **`boolean`**，只有两个字面量：

- `true`
- `false`

```x
let is-active: boolean = true
let has-error: boolean = false

if is-active and not has-error {
  start()
}
```

与所有主流语言保持一致，**禁止** 以 `0` / `1` 充当真假，减少隐式转换带来的歧义。配合 `not` / `and` / `or` 这些关键字式逻辑运算符，让布尔表达式读起来更接近自然语言。

### 字符类型

#### 语义

字符类型分为：

- **`character`**：值类型，代表单个 Unicode 字符
- **`Character`**：引用类型，用于需要对象封装时

```x
let ch: character = '中'
let letter: character = 'A'
```

可用于：

- 解析文本、编写词法分析器
- 单字符处理（如分类、过滤）

注意：X 语言使用单引号 `'` 来表示字符字面量，使用双引号 `"` 来表示字符串字面量。

### 字符串类型

#### 语义

字符串类型分为：

- **`string`**：值类型，用于绝大多数文本数据场景
- **`String`**：引用类型，提供面向对象的字符串接口

- 普通字符串：`"..."`（支持转义）
- 多行字符串：`""" ... """`（保留缩进和换行）
- 插值字符串：`"Hello, {name}!"`

```x
let greeting: string = "Hello, X"

let multi = """
多行字符串
保留格式
"""

let name: string = "Alice"
let msg: string = "Hello, {name}!"   // 插值
```

字符串可以包含任何 Unicode 字符，包括表情符号。我们将在后面的章节中详细讨论字符串。

### Unit 类型

Unit 类型是一个特殊类型，它只有一个值：`Unit`（有时也写为 `()`）。当没有其他有意义的值可以返回时，Unit 类型用作函数的返回类型。

```x
function do_something() {
  print("做了一些事情")
  // 隐式返回 Unit
}
```

### Never 类型

Never 类型（`Never`）是一个永远不会有任何值的类型。它用于表示永远不会返回的表达式（例如 `panic` 调用或无限循环）。

## 复合类型

复合类型可以将多个值组合成一个类型。X 语言有两种基本的复合类型：数组/列表和记录。

### 列表类型

列表是一个值的集合，所有值都具有相同的类型。X 语言使用 `List<T>` 表示列表类型，其中 `T` 是元素的类型。列表类型也有一个简写语法 `[T]`。

```x
let a = [1, 2, 3, 4, 5]
let months = ["一月", "二月", "三月", "四月", "五月", "六月",
              "七月", "八月", "九月", "十月", "十一月", "十二月"]
```

你可以通过索引访问列表元素：

```x
let first = a[0]
let second = a[1]
```

我们将在第 6 章中详细讨论列表。

### 记录类型

记录类型可以将多个具有不同类型的值组合成一个类型。记录使用字段名来标识每个值：

```x
type Point = {
  x: float,
  y: float
}

let p = { x: 0.0, y: 0.0 }
```

我们将在第 4 章中详细讨论记录和结构体。

## 类型注解

虽然 X 语言通常可以推断类型，但你也可以显式注解类型：

```x
let guess: integer = 42
let price: float = 3.99
let is_active: boolean = true
let initial: character = 'A'
let message: string = "Hello"
```

## 总结

我们已经介绍了 X 语言的基本数据类型。让我们继续学习函数！

