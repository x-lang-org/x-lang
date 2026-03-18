# Result 类型

`Result<T, E>` 是 X 语言中表示「可能成功或失败」操作结果的核心类型，用于替代异常。

## 类型定义

```x
enum Result<T, E> {
    Ok(T),   // 操作成功，包含返回值
    Err(E)   // 操作失败，包含错误信息
}
```

## 为什么不用异常？

X 语言没有异常机制。使用 `Result<T, E>` 的好处：

1. **显式性**：从函数签名就能看出可能失败
2. **类型安全**：编译器强制处理错误情况
3. **无隐藏控制流**：错误传播是显式的，不会意外跳转
4. **可组合**：错误是值，可以用函数组合

## 基本用法

### 创建 Result

```x
let success: Result<integer, string> = Ok(42)
let failure: Result<integer, string> = Err("出错了")
```

### 模式匹配

```x
function describe(result: Result<integer, string>): string {
    given result {
        is Ok(value) => "成功: " + to_string(value)
        is Err(e) => "失败: " + e
    }
}
```

## 常用函数

### 检查

```x
is_ok(result)    // 检查是否为 Ok
is_err(result)   // 检查是否为 Err
```

### 提取值

```x
unwrap(result)           // 提取 Ok 的值，Err 则 panic
unwrap_or(result, 0)     // 提取值，Err 则返回默认值
unwrap_err(result)       // 提取 Err 的错误，Ok 则 panic
```

### 变换

```x
map(result, fn(x) { x * 2 })       // 如果是 Ok，对值应用函数
map_err(result, fn(e) { "错误: " + e })  // 如果是 Err，对错误应用函数
and_then(result, fn(x) { Ok(x * 2) })    // 如果是 Ok，应用返回 Result 的函数
```

### 转换

```x
ok(result)   // Result<T, E> -> Option<T>
err(result)  // Result<T, E> -> Option<E>
```

## 示例：文件操作

```x
function read_file(path: string): Result<string, string> {
    // 调用平台特定的文件读取 API
    // 这里使用伪代码示意
    if file_exists(path) {
        Ok(file_content(path))
    } else {
        Err("文件不存在: " + path)
    }
}

function main() {
    let result = read_file("config.txt")

    given result {
        is Ok(content) => {
            println("文件内容:")
            println(content)
        }
        is Err(e) => {
            println("读取失败: " + e)
        }
    }
}
```

## 示例：解析操作

```x
function parse_int(s: string): Result<integer, string> {
    // 尝试解析字符串为整数
    // ...
}

function main() {
    let input = "42"

    let result = parse_int(input)
        |> fn(r) { map(r, fn(x) { x * 2 }) }

    given result {
        is Ok(value) => println("结果: " + to_string(value))
        is Err(e) => println("解析错误: " + e)
    }
}
```

## 错误传播（? 运算符）

X 语言支持 `?` 运算符，用于简洁地传播错误：

```x
function read_config(): Result<Config, string> {
    let content = read_file("config.txt")?    // 如果 Err，直接返回
    let config = parse_config(content)?        // 如果 Err，直接返回
    Ok(config)
}
```

`?` 运算符的工作原理：

```x
let value = result?
// 等价于
let value = given result {
    is Ok(v) => v
    is Err(e) => return Err(e)
}
```

## 链式错误处理

```x
function process_file(path: string): Result<integer, string> {
    read_file(path)
        |> fn(r) { and_then(r, parse_int) }
        |> fn(r) { map(r, fn(x) { x * 2 }) }
        |> fn(r) { map_err(r, fn(e) { "处理失败: " + e }) }
}
```

## 组合多个 Result

```x
function divide(a: integer, b: integer): Result<integer, string> {
    if b == 0 {
        Err("除数不能为零")
    } else {
        Ok(a / b)
    }
}

function safe_calculate(x: integer, y: integer, z: integer): Result<integer, string> {
    let a = divide(x, y)?
    let b = divide(a, z)?
    Ok(b)
}
```

## 与 Option 的转换

```x
// Option<T> -> Result<T, E>
let result = ok_or(some_option, "默认错误")

// Result<T, E> -> Option<T>
let opt = ok(some_result)  // 丢弃错误信息
```

## 最佳实践

1. **错误类型要具体**：使用有意义的错误类型，而不是笼统的 string
2. **尽早处理**：不要让 Result 到处传播而不处理
3. **提供上下文**：用 `map_err` 添加错误上下文
4. **区分可恢复与不可恢复**：可恢复错误用 Result，不可恢复用 panic
