# Result：处理可恢复的错误

在上一章中，我们研究了 `Option`，它表示一个值可能存在也可能不存在。在本章中，我们将研究 `Result`，它类似于 `Option`，但它表示一个操作可能成功也可能失败。

`Result` 是处理可恢复错误的 X 语言方式——你通常希望向用户报告这种错误并重试操作。让我们看看 `Result` 是什么，以及如何使用它。

## 什么是 Result？

`Result` 类型表示操作的结果：每个 `Result` 要么是 `Ok`，表示成功并包含一个值，要么是 `Err`，表示失败并包含一个错误值。`Result` 在 X 语言标准库中定义，如下所示：

```x
type Result<T, E> = Ok(T) | Err(E)
```

`Result` 有两个泛型类型参数：`T` 表示成功时包含在 `Ok` 变体中的值的类型，`E` 表示失败时包含在 `Err` 变体中的错误的类型。

让我们看一个使用 `Result` 的简单示例。回想一下第 2 章，我们有一个从字符串解析数字的函数：

```x
function parse_number(s: String) -> Result<integer, String> {
  // 让我们假装这是从字符串解析整数
  if s == String::from("42") {
    Ok(42)
  } else {
    Err(String::from("无效的数字"))
  }
}
```

在这个例子中，`parse_number` 函数返回一个 `Result<integer, String>`。如果输入是字符串 "42"，它返回 `Ok(42)`。否则，它返回 `Err(String::from("无效的数字"))`。

## 为什么 Result 有用？

像 `Option` 一样，`Result` 比在其他语言中常见的使用异常或特殊值的方法更好，因为它强制你在编译时处理错误情况。编译器会确保你不会忘记处理 `Err` 情况，这有助于防止在运行时出现意外的错误。

让我们看看如何使用 `Result` 值。

## 使用 Result

与 `Option` 一样，我们可以使用 `when`/`is` 表达式来处理 `Result` 值：

```x
let result = parse_number(String::from("42"))
when result is {
  Ok(n) => println("解析的数字: ", n),
  Err(e) => println("解析错误: ", e)
}
```

在这个例子中，如果结果是 `Ok`，我们打印解析的数字。如果是 `Err`，我们打印错误消息。

### 匹配 Result 的简写：if let

与 `Option` 一样，我们可以使用 `if let` 语法作为使用 `when`/`is` 处理 `Result` 的简写：

```x
let result = parse_number(String::from("42"))
if let Ok(n) = result {
  println("解析的数字: ", n)
} else if let Err(e) = result {
  println("解析错误: ", e)
}
```

## Result 的常用方法

与 `Option` 一样，`Result` 类型有许多有用的方法可以让你的生活更轻松。让我们看看其中一些最常见的。

### is_ok 和 is_err

`is_ok` 方法在 `Result` 为 `Ok` 时返回 `true`，在为 `Err` 时返回 `false`。`is_err` 正好相反：

```x
let ok_result = Ok(42)
let err_result: Result<integer, String> = Err(String::from("出错了"))

println(ok_result.is_ok())   // true
println(ok_result.is_err())  // false
println(err_result.is_ok())  // false
println(err_result.is_err()) // true
```

### ok

`ok` 方法将 `Result<T, E>` 转换为 `Option<T>`，将 `Ok` 映射到 `Some`，将 `Err` 映射到 `None`：

```x
let ok_result = Ok(42)
let option_from_ok = Result::ok(ok_result)  // Some(42)

let err_result: Result<integer, String> = Err(String::from("出错了"))
let option_from_err = Result::ok(err_result)  // None
```

### err

`err` 方法将 `Result<T, E>` 转换为 `Option<E>`，将 `Err` 映射到 `Some`，将 `Ok` 映射到 `None`：

```x
let ok_result = Ok(42)
let option_from_ok = Result::err(ok_result)  // None

let err_result: Result<integer, String> = Err(String::from("出错了"))
let option_from_err = Result::err(err_result)  // Some(String::from("出错了"))
```

### unwrap

`unwrap` 方法返回 `Ok` 内部的值，但如果 `Result` 是 `Err`，它会 panic：

```x
let ok_result = Ok(42)
let value = ok_result.unwrap()  // 42

let err_result: Result<integer, String> = Err(String::from("出错了"))
// err_result.unwrap()  // 这会 panic！
```

与 `Option` 上的 `unwrap` 一样，通常最好在示例或快速原型设计之外避免使用它。

### expect

`expect` 方法与 `unwrap` 类似，但它允许你提供自定义的 panic 消息：

```x
let ok_result = Ok(42)
let value = ok_result.expect("应该有一个值")  // 42

let err_result: Result<integer, String> = Err(String::from("出错了"))
// err_result.expect("应该有一个值")  // 这会 panic 并显示我们的消息！
```

### map

`map` 方法接受一个函数，并将其应用于 `Result` 内部的成功值（如果存在）：

```x
let ok_result = Ok(42)
let new_result = Result::map(ok_result, function(x) { x * 2 })  // Ok(84)

let err_result: Result<integer, String> = Err(String::from("出错了"))
let new_err = Result::map(err_result, function(x) { x * 2 })  // Err(...)
```

### map_err

`map_err` 方法接受一个函数，并将其应用于 `Result` 内部的错误值（如果存在）：

```x
let ok_result = Ok(42)
let new_result = Result::map_err(ok_result, function(e) { String::from("错误: ") + e })  // Ok(42)

let err_result: Result<integer, String> = Err(String::from("出错了"))
let new_err = Result::map_err(err_result, function(e) { String::from("错误: ") + e })  // Err("错误: 出错了")
```

### and_then

`and_then` 方法类似于 `map`，但它接受一个返回 `Result` 的函数。当你有一个返回 `Result` 的函数链时，这很有用：

```x
function divide(x: integer, y: integer) -> Result<integer, String> {
  if y == 0 {
    Err(String::from("除以零"))
  } else {
    Ok(x / y)
  }
}

let ok_result = Ok(8)
let result = Result::and_then(ok_result, function(x) { divide(x, 2) })  // Ok(4)

let err_result: Result<integer, String> = Err(String::from("出错了"))
let no_result = Result::and_then(err_result, function(x) { divide(x, 2) })  // Err(...)
```

### or

`or` 方法返回第一个 `Result`（如果它是 `Ok`），否则返回第二个 `Result`：

```x
let ok_result = Ok(42)
let fallback = Ok(0)
let result = Result::or(ok_result, fallback)  // Ok(42)

let err_result: Result<integer, String> = Err(String::from("出错了"))
let fallback = Ok(0)
let result_with_fallback = Result::or(err_result, fallback)  // Ok(0)
```

### or_else

`or_else` 方法类似于 `or`，但它接受一个函数，该函数仅在第一个 `Result` 是 `Err` 时才被调用以计算 fallback `Result`：

```x
let ok_result = Ok(42)
let result = Result::or_else(ok_result, function() { Ok(0) })  // Ok(42)

let err_result: Result<integer, String> = Err(String::from("出错了"))
let result_with_fallback = Result::or_else(err_result, function() { Ok(0) })  // Ok(0)
```

当 fallback 的计算成本很高时，这很有用。

### unwrap_or

`unwrap_or` 方法返回 `Ok` 内部的值，或者如果是 `Err`，则返回提供的默认值：

```x
let ok_result = Ok(42)
let value = Result::unwrap_or(ok_result, 0)  // 42

let err_result: Result<integer, String> = Err(String::from("出错了"))
let value_with_default = Result::unwrap_or(err_result, 0)  // 0
```

### unwrap_or_else

`unwrap_or_else` 方法类似于 `unwrap_or`，但它接受一个函数，该函数仅在 `Result` 是 `Err` 时才被调用以计算默认值：

```x
let ok_result = Ok(42)
let value = Result::unwrap_or_else(ok_result, function(e) { 0 })  // 42

let err_result: Result<integer, String> = Err(String::from("出错了"))
let value_with_default = Result::unwrap_or_else(err_result, function(e) { 0 })  // 0
```

## 传播错误

在编写代码时，你经常会发现自己编写的函数调用可能会失败的其他函数。你可以让错误传播，而不是在函数内部处理错误，这样调用者就可以决定如何处理它。让我们看一个例子：

```x
function parse_and_double(s: String) -> Result<integer, String> {
  let result = parse_number(s)
  when result is {
    Ok(n) => Ok(n * 2),
    Err(e) => Err(e)
  }
}
```

在这个例子中，`parse_and_double` 调用 `parse_number`，如果成功，它将结果加倍。如果失败，它会将错误传播给调用者。

这种模式在 X 语言代码中非常常见。我们可以使用 `when`/`is` 模式匹配来传播错误，但这有点冗长。

## 组合 Result 和 Option

你经常会发现自己同时使用 `Result` 和 `Option`。幸运的是，它们有很好的组合方式。例如，你可以使用 `ok` 方法将 `Result` 转换为 `Option`，或者使用 `ok_or` 方法（如果可用）将 `Option` 转换为 `Result`。

## 总结

`Result` 类型是 X 语言处理可恢复错误的方式。它比异常更安全，因为编译器会强制你在编译时处理错误情况。以下是一些关键点：

- `Result<T, E>` 可以是 `Ok(T)` 或 `Err(E)`
- 使用 `when`/`is` 以类型安全的方式处理 `Result` 值
- `if let` 是简单情况的简写
- `Result` 有许多有用的方法，如 `map`、`and_then`、`unwrap_or` 等
- 你可以将错误传播给调用者
- 谨慎使用 `unwrap` 和 `expect`，因为它们可能会 panic

在本章中，我们介绍了 `Result`，它用于处理可恢复的错误。在下一章中，我们将介绍 `panic`，它用于处理不可恢复的错误。
