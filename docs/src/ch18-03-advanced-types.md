# 高级类型

在本节中，我们将探索一些关于类型的更高级功能：类型别名、`never` 类型和动态大小类型。让我们开始！

## 使用类型别名创建类型同义词

除了使用泛型，我们还可以讨论类型别名（type alias）——这是一种为现有类型赋予另一个名称的方法，称为同义词。我们将使用 `type` 关键字来实现。例如，我们可以像这样别名 `integer`：

```x
type Kilometers = integer

let x: integer = 5
let y: Kilometers = 5

println("x + y = {}", x + y)
```

因为 `Kilometers` 是 `integer` 的同义词，它们是同一类型。所以我们可以将 `integer` 和 `Kilometers` 相加，并且我们可以将 `Kilometers` 类型的值传递给接受 `integer` 类型参数的函数。但是通过使用这种技术，我们不会获得我们在第 4 章中讨论的类型检查的好处。换句话说，如果我们不小心将 `Kilometers` 和 `integer` 的值混合在一起，编译器不会给我们错误。

类型别名的主要用例是减少重复。例如，我们可能有一个如下所示的长类型：

```x
let f: Box<Fn() + Send + 'static> = Box::new(|| println!("hi"))
```

在每个函数签名中写出这个长长的类型会很累人且容易出错。想象一下，必须在函数定义中多次写出这个。谢天谢地，我们可以使用类型别名来缩短它：

```x
type Thunk = Box<Fn() + Send + 'static>

let f: Thunk = Box::new(|| println!("hi"))

function takes_long_type(f: Thunk) {
  // --snip--
}

function returns_long_type() -> Thunk {
  // --snip--
}
```

好多了！类型别名允许我们编写更简洁、更清晰的代码。类型别名通常也与 `Result<T, E>` 一起使用，以减少重复。考虑一下标准库中的 `std::io` 模块。I/O 操作通常返回 `Result<T, E>` 来处理操作失败的情况。`std::io` 定义了 `Error` 结构体，表示所有可能的 I/O 错误。`std::io` 中的许多函数返回 `Result<T, E>`，其中 `E` 是 `std::io::Error`，例如 `Write` trait 中的这些函数：

```x
use std::io::Error
use std::fmt

trait Write {
  function write(self: &mut Self, buf: &[u8]) -> Result<integer, Error>
  function flush(self: &mut Self) -> Result<(), Error>
}
```

因为 `std::io::Error` 经常被使用，`std::io` 提供了这个类型别名 `Result<T>` 作为 `Result<T, std::io::Error>` 的简写！所以 `std::io::Result<T>` 只是 `Result<T, E>` 的别名，其中 `E` 填充了 `std::io::Error`。这最终意味着我们需要输入更少，并且我们可以拥有更一致的接口。因为它在 `std::io` 模块中，可用的类型别名是 `std::io::Result<T>`，这正是我们想要的！

这就是类型别名！它们并不复杂，但很方便。

