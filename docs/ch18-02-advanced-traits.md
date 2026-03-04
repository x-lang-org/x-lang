# 高级 Trait

我们在第 8 章中第一次讨论了 trait，但和我们生命周期一样，我们没有讨论一些更高级的细节。现在我们对 X 语言有了更多了解，我们可以深入研究。

## 关联类型在 trait 定义中指定占位符类型

关联类型（Associated types）将类型占位符与 trait 连接起来，以便 trait 的方法签名可以在其签名中使用这些占位符类型。trait 的实现者将为他们正在实现的 trait 的特定用例指定要用于占位符类型的具体类型。这样，我们就可以定义一个使用某些类型的 trait，而不必确切知道这些类型是什么，直到 trait 被实现。

我们在本书前面提到的一个具有关联类型的 trait 是标准库中的 `Iterator` trait。它有一个名为 `Item` 的关联类型，代表迭代器正在迭代的值的类型。`Iterator` trait 的定义如下所示：

```x
trait Iterator {
  type Item

  function next(self: &mut Self) -> Option<Self::Item>
}
```

`Item` 类型是一个占位符类型，而 `next` 方法的定义表明它将返回 `Option<Self::Item>` 类型的值。`Iterator` trait 的实现者将为 `Item` 指定一个具体类型，并且 `next` 方法将返回一个包含该具体类型值的 `Option`。

关联类型与泛型的不同之处在于，使用关联类型时，我们不需要为每个实现注解类型，因为我们不能为一个类型多次实现该 trait。让我们看看这看起来像什么。

让我们看一下 `Iterator` trait 在一个名为 `Counter` 的类型上的实现，我们在第 13 章中定义了该类型，它迭代 `1` 到 `5` 的值：

```x
type Counter = {
  count: integer
}

impl Iterator for Counter {
  type Item = integer

  function next(self: &mut Self) -> Option<integer> {
    if self.count < 6 {
      let current = self.count
      self.count = self.count + 1
      Some(current)
    } else {
      None
    }
  }
}
```

我们为 `Item` 关联类型指定了 `integer` 类型，并实现了 `next` 方法，使其返回 `Option<integer>`。

如果 trait 是泛型的，会是什么样子？`Iterator` trait 可能已经用泛型定义了，如下所示：

```x
trait Iterator<T> {
  function next(self: &mut Self) -> Option<T>
}
```

那么我们需要这样实现：

```x
impl Iterator<integer> for Counter {
  function next(self: &mut Self) -> Option<integer> {
    // --snip--
  }
}
```

然后我们也可以为 `Counter` 实现 `Iterator<&str>`、`Iterator<Float>` 等等，这样 `Counter` 就有多个 `next` 方法的实现，每个都有自己的类型。

使用关联类型而不是泛型 trait 的好处是，我们不需要为每个实现注解类型，因为我们不能为一个类型多次实现该 trait。使用关联类型，一旦我们在 `impl Iterator for Counter` 中选择了 `Item` 的类型，我们就不必再次指定我们正在迭代 `integer` 值。

## 默认泛型类型参数和运算符重载

我们可以为泛型类型参数指定默认类型。如果默认类型足够，这消除了 trait 用户为泛型类型指定具体类型的需要。指定泛型类型的默认类型的语法是在声明泛型类型时使用 `<PlaceholderType=ConcreteType>`。

这种技术的一个很好的例子是一个用于运算符重载的 trait。运算符重载允许我们在某些情况下自定义运算符（如 `+`）的行为。

X 语言不允许你创建自己的运算符或重载任意运算符。但是你可以通过实现与该运算符对应的 trait 来重载 `std::ops` 中列出的操作和相应的 trait。例如，在示例中，我们通过实现 `Add` trait 来重载 `+` 运算符，以将两个 `Point` 实例加在一起。

```x
use std::ops::Add

type Point = {
  x: integer,
  y: integer
}

impl Add for Point {
  type Output = Point

  function add(self: Self, other: Self) -> Point {
    {
      x: self.x + other.x,
      y: self.y + other.y
    }
  }
}

function main() {
  assert_eq!({x: 1, y: 0} + {x: 2, y: 3}, {x: 3, y: 3})
}
```

`add` 方法将两个 `Point` 实例的 `x` 值和 `y` 值分别相加，以创建一个新的 `Point`。`Add` trait 有一个名为 `Output` 的关联类型，它确定从 `add` 方法返回的类型。

让我们更仔细地看看 `Add` trait 是如何工作的：

```x
trait Add<Rhs=Self> {
  type Output

  function add(self: Self, rhs: Rhs) -> Self::Output
}
```

这段代码看起来应该很熟悉：它是一个带有一个方法和一个关联类型的 trait。新的部分是 `Rhs=Self`：这个语法称为默认类型参数。`Rhs` 泛型类型参数（用于"右手边"）定义了 `add` 方法中 `rhs` 参数的类型。如果我们在实现 `Add` trait 时没有为 `Rhs` 指定具体类型，`Rhs` 的类型将默认为 `Self`，这将是我们正在实现 `Add` 的类型。

当我们为 `Point` 实现 `Add` 时，我们使用了默认的 `Rhs`，因为我们想将两个 `Point` 实例加在一起。让我们看一个实现 `Add` trait 的例子，在这个例子中，我们想要自定义 `Rhs` 类型而不是使用默认类型。

我们有两个结构，`Millimeters` 和 `Meters`，分别持有以毫米和米为单位的值。我们想通过实现 `Add` 来将毫米加到米上，其中 `Add` 在 `Millimeters` 上的实现将 `Meters` 作为 `Rhs`。

```x
use std::ops::Add

type Millimeters = { value: integer }
type Meters = { value: integer }

impl Add<Meters> for Millimeters {
  type Output = Millimeters

  function add(self: Self, other: Meters) -> Millimeters {
    { value: self.value + (other.value * 1000) }
  }
}
```

要将毫米加到米上，我们需要设置 `impl Add<Meters>` 以赋予 `Rhs` 类型参数一个值，而不是使用默认的 `Self`。

默认类型参数主要用于两种方式：
1. 扩展一个类型而不破坏现有代码
2. 允许在大多数用户不需要的特定情况下进行自定义

标准库的 `Add` trait 是第二种方式的一个例子：通常你会将两个相似的类型加在一起，但 `Add` trait 提供了自定义该功能的能力。使用 `Add` trait 的默认类型参数意味着大多数时候你不必指定额外的参数，从而减少了必须指定的样板代码。

好的，关于关联类型和默认类型参数就讲到这里！让我们继续关于 trait 的另一个高级特性：完全限定语法以消除歧义。
