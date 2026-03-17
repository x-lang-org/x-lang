# Option 类型

`Option<T>` 是 X 语言中表示「可能存在或不存在」值的核心类型，用于替代 null。

## 类型定义

```x
enum Option<T> {
    Some(T),  // 值存在
    None      // 值不存在
}
```

## 为什么不用 null？

X 语言没有 null 值。使用 `Option<T>` 的好处：

1. **类型安全**：编译器强制处理「值不存在」的情况
2. **显式性**：从类型签名就能看出值可能不存在
3. **无空指针异常**：不可能在运行时遇到空指针

## 基本用法

### 创建 Option

```x
let some_value: Option<integer> = Some(42)
let no_value: Option<integer> = None
```

### 模式匹配

```x
function describe(opt: Option<integer>): string {
    given opt {
        is Some(value) => "值为: " + to_string(value)
        is None => "没有值"
    }
}
```

### 使用 given 表达式

```x
let result = given some_value {
    is Some(v) => v * 2
    is None => 0
}
```

## 常用函数

### 检查

```x
is_some(opt)  // 检查是否为 Some
is_none(opt)  // 检查是否为 None
```

### 提取值

```x
unwrap(opt)           // 提取值，None 则 panic
unwrap_or(opt, 0)     // 提取值，None 则返回默认值
unwrap_or_else(opt, fn() { 0 })  // 提取值，None 则调用函数
```

### 变换

```x
map(opt, fn(x) { x * 2 })      // 如果是 Some，对值应用函数
and_then(opt, fn(x) { Some(x * 2) })  // 如果是 Some，应用返回 Option 的函数
```

### 组合

```x
or(opt1, opt2)      // 如果 opt1 是 None，返回 opt2
zip(opt1, opt2)     // 如果两个都是 Some，返回 Some((v1, v2))
```

### 过滤

```x
filter(opt, fn(x) { x > 0 })  // 如果值满足谓词，返回自身；否则返回 None
```

## 示例：安全除法

```x
function divide(a: integer, b: integer): Option<integer> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

function main() {
    let result = divide(10, 2)

    given result {
        is Some(value) => println("结果: " + to_string(value))
        is None => println("除数不能为零")
    }
}
```

## 示例：查找操作

```x
function find_user(id: integer): Option<string> {
    if id == 1 {
        Some("Alice")
    } else if id == 2 {
        Some("Bob")
    } else {
        None
    }
}

function main() {
    let user = find_user(1)

    let name = unwrap_or(user, "未知用户")
    println("用户名: " + name)
}
```

## 链式操作

```x
function parse_int(s: string): Option<integer> {
    // 尝试解析字符串为整数
    // ...
}

function main() {
    let input = "42"

    let result = parse_int(input)
        |> fn(opt) { map(opt, fn(x) { x * 2 }) }
        |> fn(opt) { filter(opt, fn(x) { x > 50 }) }

    given result {
        is Some(value) => println("结果: " + to_string(value))
        is None => println("无效或数值太小")
    }
}
```

## 与 Result 的转换

```x
// Option<T> -> Result<T, E>
function ok_or<T, E>(opt: Option<T>, err: E): Result<T, E> {
    given opt {
        is Some(value) => Ok(value)
        is None => Err(err)
    }
}

// Result<T, E> -> Option<T>
function ok<T, E>(result: Result<T, E>): Option<T> {
    given result {
        is Ok(value) => Some(value)
        is Err(_) => None
    }
}
```
