# 函数与闭包

## 函数定义

函数是 X 语言中最基本的抽象单元，用于封装可重用的代码逻辑。在 X 中，函数使用 `function` 关键字声明，遵循以下语法：

```x
function add(a: Integer, b: Integer) -> Integer = a + b

function greet(name: String) -> String {
    let message = "Hello, {name}!"
    message
}
```

函数定义由以下部分组成：

1. **函数名**：遵循标识符命名规则，绑定在当前作用域中。
2. **参数列表**：定义函数的输入参数，每个参数可以包含类型注解和默认值。
3. **返回类型**：可选，使用 `-> Type` 语法显式声明，也可由编译器自动推导。
4. **函数体**：有两种形式：
   - 表达式形式：使用 `= expression` 直接返回表达式的值
   - 块形式：使用 `{ statements }` 执行多个语句，最后一个表达式为返回值

### 函数声明语法

```
FunctionDeclaration ::= 'function' Identifier Parameters ('->' Type)? ('with' EffectList)? FunctionBody

Parameters ::= '(' (Parameter (',' Parameter)*)? ')'

Parameter ::= Identifier (':' Type)? ('=' Expression)?

FunctionBody ::= '=' Expression
               | Block
```

## 函数参数

X 语言支持多种参数形式，提供灵活的函数调用方式：

### 位置参数

按顺序传递的参数，数量和类型必须与函数声明匹配：

```x
function add(a: Integer, b: Integer) -> Integer = a + b
add(1, 2)  // 3
```

### 命名参数

通过参数名指定，可以任意顺序传递：

```x
function formatDate(year: Integer, month: Integer, day: Integer) -> String {
    "{year}-{month}-{day}"
}

formatDate(year = 2026, month = 3, day = 7)  // "2026-3-7"
```

### 默认参数

为参数提供默认值，调用时可省略：

```x
function increment(x: Integer, step: Integer = 1) -> Integer = x + step

increment(5)      // 6
increment(5, 2)   // 7
```

### 管道运算符

使用 `|>` 运算符将左侧表达式作为第一个参数传入右侧函数，支持函数链式调用：

```x
let result = data
    |> filter(is_valid)
    |> map(transform)
    |> take(10)
```

## 函数返回值

函数可以显式声明返回类型，也可以由编译器通过 Hindley-Milner 类型推断自动推导：

### 显式返回类型

```x
function add(a: Integer, b: Integer) -> Integer = a + b
```

### 隐式返回类型

```x
function add(a: Integer, b: Integer) = a + b  // 编译器推断返回类型为 Integer
```

### 块形式的返回值

在块形式的函数体中，最后一个表达式的结果作为函数返回值，也可以使用 `return` 语句显式返回：

```x
function max(a: Integer, b: Integer) -> Integer {
    if a > b {
        return a
    } else {
        b
    }
}
```

### 效果注解

函数可以通过 `with` 关键字声明可能产生的效果，如 IO、异步操作、异常等：

```x
function printMessage(message: String) -> () with IO {
    print(message)
}

function readFile(path: String) -> String with IO, Throws<FileNotFound> {
    let file = open(path)?
    file.readAll()
}
```

## 闭包

闭包是可以捕获周围作用域变量的函数值。在 X 中，匿名函数（Lambda）会创建闭包：

```x
function make_counter() -> () -> Integer {
    let mutable count = 0
    () -> {
        count = count + 1
        count
    }
}

let counter = make_counter()
counter()  // 1
counter()  // 2
counter()  // 3
```

### 闭包的特性

1. **变量捕获**：闭包在定义时捕获周围作用域的变量。
2. **环境关联**：被捕获的变量保持与外部作用域的关联。
3. **可变变量**：若外部变量为 `let mutable`，闭包内外的修改相互可见。

### 匿名函数语法

```
Lambda ::= '(' (Identifier (',' Identifier)*)? ')' '->' Expression
         | '(' (Identifier (',' Identifier)*)? ')' '->' Block
         | '.' Identifier                            // 点缩写
```

### 点缩写语法

点缩写提供简洁的字段/方法访问语法，等价于以对象为参数的单参数 lambda：

```x
let names = users |> map(.name)             // 等价于 map((u) -> u.name)
let adults = users |> filter(.age >= 18)    // 等价于 filter((u) -> u.age >= 18)
```

## 高阶函数

高阶函数是指以函数为参数或以函数为返回值的函数。在 X 中，函数是一等公民，可以作为参数传递、作为返回值返回、赋值给变量。

### 函数作为参数

```x
function apply_twice(f: (Integer) -> Integer, x: Integer) -> Integer {
    f(f(x))
}

let result = apply_twice((x) -> x * 2, 5)  // 20
```

### 函数作为返回值

```x
function make_adder(n: Integer) -> (Integer) -> Integer {
    (x) -> x + n
}

let add5 = make_adder(5)
add5(10)  // 15
```

### 柯里化与部分应用

```x
function multiply(a: Integer, b: Integer) -> Integer = a * b

let double = multiply(2, _)   // 部分应用
let triple = multiply(3, _)   // 部分应用

double(5)  // 10
triple(5)  // 15
```

### 常见高阶函数

X 语言提供了丰富的高阶函数，如 `map`、`filter`、`reduce` 等：

```x
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers |> map((x) -> x * 2)      // [2, 4, 6, 8, 10]
let evens = numbers |> filter((x) -> x % 2 == 0) // [2, 4]
let sum = numbers |> reduce((acc, x) -> acc + x, 0) // 15
```

## 递归函数

函数可以在其体内引用自身实现递归。`rec` 关键字是可选的，主要用于明确表示递归意图：

```x
function factorial(n: Integer) -> Integer {
    match n {
        0 => 1
        _ => n * factorial(n - 1)
    }
}

function rec fibonacci(n: Integer) -> Integer {
    match n {
        0 => 0
        1 => 1
        _ => fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```

### 分段定义

X 支持分段定义，允许将多个 case 写成独立的函数子句：

```x
function fibonacci(0) -> Integer = 0
function fibonacci(1) -> Integer = 1
function fibonacci(n) -> Integer = fibonacci(n - 1) + fibonacci(n - 2)
```

## 多态函数

函数可以有类型参数（泛型），使用 `<>` 语法。类型参数在调用时由编译器根据实参类型推断，也可以显式指定：

```x
function identity<T>(x: T) -> T = x

function max<T: Comparable>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

function swap<A, B>(pair: (A, B)) -> (B, A) = (pair.1, pair.0)

// 调用：类型由推断确定
let x = identity(42)         // T = Integer
let m = max(3, 7)            // T = Integer
let s = swap(("hello", 42))  // A = String, B = Integer
```

## 总结

X 语言的函数系统提供了丰富的特性，包括：

- 灵活的函数定义语法，支持表达式形式和块形式
- 多种参数形式：位置参数、命名参数、默认参数
- 强大的类型系统，支持类型推断和泛型
- 闭包机制，允许函数捕获和修改外部变量
- 高阶函数支持，使函数成为一等公民
- 递归和分段定义，方便实现复杂算法
- 效果注解，使副作用在类型层面可见

这些特性使得 X 语言能够表达从简单到复杂的各种计算逻辑，同时保持代码的清晰性和可维护性。
