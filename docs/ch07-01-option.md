# Option：表示可能不存在的值

在第 4 章中，我们简要介绍了 `Option` 枚举作为一种编码一个值可以存在或不存在的方式的方法。在本章中，我们将更详细地探讨 `Option`，包括它是什么、如何使用以及如何在你的代码中充分利用它。

## 什么是 Option？

`Option` 类型表示可选值：每个 `Option` 要么是 `Some`，其中包含一个值，要么是 `None`，表示没有值。`Option` 在 X 语言标准库中定义，如下所示：

```x
type Option<T> = Some(T) | None
```

`<T>` 语法表示 `Option` 枚举是泛型的，这意味着 `Some` 变体可以保存任何类型的数据。我们将在第 8 章更详细地介绍泛型。

让我们看一个使用 `Option` 的简单示例：

```x
let some_number = Some(5)
let some_string = Some(String::from("一个字符串"))
let absent_number: Option<integer> = None
```

在第一行中，我们创建了一个 `Option<integer>` 类型的 `Some` 值，并将其绑定到变量 `some_number`。在第二行中，我们创建了一个 `Option<String>` 类型的 `Some` 值。在第三行中，我们创建了一个 `None` 值，它表示没有任何值。对于 `None`，我们需要告诉 X 语言我们想要什么类型的 `Option`，因为编译器无法仅通过查看 `None` 值就推断出 `Some` 变体将持有的类型。

## 为什么 Option 有用？

你可能想知道为什么我们需要 `Option` 类型。其他语言使用空值或 nil 来表示没有值，这有什么问题呢？

问题在于，如果你尝试将空值用作非空值，你会得到某种错误。因为这个空或非空属性无处不在，很容易犯这种错误。空值的发明者 Tony Hoare 将其称为"十亿美元的错误"。

X 语言没有空值，但它有一个可以编码存在或不存在概念的枚举。这个枚举是 `Option<T>`，它由标准库定义。`Option<T>` 如此常用，以至于它甚至包含在 prelude 中。它的变体也包含在 prelude 中：你可以直接使用 `Some` 和 `None`，而不需要 `Option::` 前缀。

`Option<T>` 比空值好在哪里？简而言之，因为 `Option<T>` 和 `T`（其中 `T` 可以是任何类型）是不同的类型，编译器不会让我们使用 `Option<T>` 值，就好像它绝对是一个有效值一样。例如，这段代码不会编译，因为它试图将一个 `Option<integer>` 加到一个 `integer` 上：

```x
let x: integer = 5
let y: Option<integer> = Some(5)
let sum = x + y  // 错误！不能将 integer 和 Option<integer> 相加
```

实际上，这个错误消息意味着 X 语言不理解如何将 `Option<integer>` 和 `integer` 相加，因为它们是不同的类型。当我们在 X 语言中有一个像 `integer` 这样类型的值时，编译器将确保我们始终有一个有效值。我们可以放心地进行操作，而不必在使用该值之前检查空值。只有当我们有一个 `Option<integer>`（或我们正在使用的任何类型的 `Option`）时，我们才必须担心可能没有值，编译器会确保我们在使用该值之前处理这种情况。

换句话说，在你可以对 `Option<T>` 进行 `T` 操作之前，你必须将其转换为 `T`。一般来说，这有助于捕获空值最常见的问题之一：假设某事不是空的，而实际上它是空的。

## 使用 Option

现在我们知道 `Option` 是什么以及为什么它有用，让我们看看如何实际使用它！我们可以使用 `when`/`is` 表达式来处理 `Option` 值，就像我们在第 4 章看到的那样：

```x
function plus_one(x: Option<integer>) -> Option<integer> {
  when x is {
    None => None,
    Some(i) => Some(i + 1)
  }
}

let five = Some(5)
let six = plus_one(five)
let none = plus_one(None)
```

在这个例子中，`plus_one` 函数接受一个 `Option<integer>` 并返回一个 `Option<integer>`。如果输入是 `None`，它返回 `None`。如果输入是 `Some(i)`，它返回 `Some(i + 1)`。

### 匹配 Option 的简写：if let

`when`/`is` 表达式对于处理 `Option` 值非常强大，但有时对于简单情况可能有点冗长。对于只想在值为 `Some` 时执行某些操作的情况，X 语言提供了 `if let` 语法作为简写：

```x
let some_value = Some(3)
if let Some(i) = some_value {
  println("值是 ", i)
}
```

这段代码与使用 `when`/`is` 相同，但更简洁：

```x
let some_value = Some(3)
when some_value is {
  Some(i) => println("值是 ", i),
  None => ()
}
```

我们也可以在 `if let` 中包含 `else` 子句，以在值为 `None` 时执行某些操作：

```x
let some_value: Option<integer> = None
if let Some(i) = some_value {
  println("值是 ", i)
} else {
  println("没有值")
}
```

## Option 的常用方法

`Option` 类型有许多有用的方法可以让你的生活更轻松。让我们看看其中一些最常见的。

### is_some 和 is_none

`is_some` 方法在 `Option` 为 `Some` 时返回 `true`，在为 `None` 时返回 `false`。`is_none` 正好相反：

```x
let some_value = Some(3)
let no_value: Option<integer> = None

println(some_value.is_some())  // true
println(some_value.is_none())   // false
println(no_value.is_some())    // false
println(no_value.is_none())     // true
```

### unwrap

`unwrap` 方法返回 `Some` 内部的值，但如果 `Option` 是 `None`，它会 panic：

```x
let some_value = Some(3)
let value = some_value.unwrap()  // 3

let no_value: Option<integer> = None
// no_value.unwrap()  // 这会 panic！
```

因为 `unwrap` 可能会 panic，所以通常最好在示例或快速原型设计之外避免使用它。对于生产代码，你应该更喜欢显式处理 `None` 情况，或者使用 `expect` 方法，它允许你提供自定义的 panic 消息。

### expect

`expect` 方法与 `unwrap` 类似，但它允许你提供自定义的 panic 消息：

```x
let some_value = Some(3)
let value = some_value.expect("应该有一个值")  // 3

let no_value: Option<integer> = None
// no_value.expect("应该有一个值")  // 这会 panic 并显示我们的消息！
```

像 `unwrap` 一样，`expect` 应该谨慎使用。

### map

`map` 方法接受一个函数，并将其应用于 `Option` 内部的值（如果存在）：

```x
let some_value = Some(3)
let new_value = Option::map(some_value, function(x) { x * 2 })  // Some(6)

let no_value: Option<integer> = None
let new_none = Option::map(no_value, function(x) { x * 2 })  // None
```

### and_then

`and_then` 方法类似于 `map`，但它接受一个返回 `Option` 的函数。当你有一个返回 `Option` 的函数链时，这很有用：

```x
function divide_two(x: integer) -> Option<integer> {
  if x % 2 == 0 {
    Some(x / 2)
  } else {
    None
  }
}

let some_value = Some(8)
let result = Option::and_then(some_value, divide_two)  // Some(4)

let odd_value = Some(7)
let no_result = Option::and_then(odd_value, divide_two)  // None
```

### or

`or` 方法返回第一个 `Option`（如果它是 `Some`），否则返回第二个 `Option`：

```x
let some_value = Some(3)
let fallback = Some(5)
let result = Option::or(some_value, fallback)  // Some(3)

let no_value: Option<integer> = None
let fallback = Some(5)
let result_with_fallback = Option::or(no_value, fallback)  // Some(5)
```

### unwrap_or

`unwrap_or` 方法返回 `Some` 内部的值，或者如果是 `None`，则返回提供的默认值：

```x
let some_value = Some(3)
let value = Option::unwrap_or(some_value, 0)  // 3

let no_value: Option<integer> = None
let value_with_default = Option::unwrap_or(no_value, 0)  // 0
```

### unwrap_or_else

`unwrap_or_else` 方法类似于 `unwrap_or`，但它接受一个函数，该函数仅在 `Option` 为 `None` 时才被调用以计算默认值：

```x
let some_value = Some(3)
let value = Option::unwrap_or_else(some_value, function() { 0 })  // 3

let no_value: Option<integer> = None
let value_with_default = Option::unwrap_or_else(no_value, function() { 0 })  // 0
```

当默认值的计算成本很高时，这很有用，因为你只想在需要时计算它。

## 总结

`Option` 类型是 X 语言处理可能不存在的值的方式。它比空值更安全，因为编译器会强制你在使用值之前处理 `None` 情况。以下是一些关键点：

- `Option<T>` 可以是 `Some(T)` 或 `None`
- 使用 `when`/`is` 以类型安全的方式处理 `Option` 值
- `if let` 是简单情况的简写
- `Option` 有许多有用的方法，如 `map`、`and_then`、`unwrap_or` 等
- 谨慎使用 `unwrap` 和 `expect`，因为它们可能会 panic

在本章中，我们介绍了 `Option`，它用于表示可能不存在的值。在下一章中，我们将介绍 `Result`，它用于处理可能失败的操作。
