# 附录 C - 可派生的 Trait

在整个书中，我们使用了 `derive` 属性，它可以在类型上生成 trait 的默认实现。在本附录中，我们提供了所有可用于 `derive` 的所有标准库 trait 的参考。每个部分涵盖：

- 该 trait 提供了哪些操作符和方法
- derive 生成什么
- trait 做什么
- 为什么你可能想要或不想要实现该 trait

## Debug

`Debug` trait 支持使用 `{:?}` 格式说明符进行调试格式化。

### 使用 `derive` 时，`Debug` trait 使你能够打印类型的实例以进行调试。

### 示例

```x
[derive(Debug)]
type Point = {
  x: integer,
  y: integer
}

let p = { x: 3, y: 5 }
println("{:?}", p)  // Point { x: 3, y: 5 }
```

### 手动实现

如果你想自定义调试输出，你可以手动实现 `Debug` trait 而不是使用 `derive`：

```x
type Point = {
  x: integer,
  y: integer
}

impl Debug for Point {
  function fmt(self: &Self, f: &mut Formatter) -> Result<(), Error> {
    write!(f, "({}, {})", self.x, self.y)
  }
}

let p = { x: 3, y: 5 }
println("{:?}", p)  // (3, 5)
```

### 何时使用

- 当你想要能够使用 `println!("{:?}", value)` 打印你的类型进行调试时，使用 `derive(Debug)`。几乎所有类型都应该派生这个 trait。

## Display

`Display` trait 支持使用 `{}` 格式说明符进行用户友好的格式化。

### 注意：`Display` 不能使用 `derive`；你必须手动实现它。

### 示例

```x
type Point = {
  x: integer,
  y: integer
}

impl Display for Point {
  function fmt(self: &Self, f: &mut Formatter) -> Result<(), Error> {
    write!(f, "({}, {})", self.x, self.y)
  }
}

let p = { x: 3, y: 5 }
println("{}", p)  // (3, 5)
```

### 何时实现

当你想要能够使用 `println!("{}", value)` 以面向用户的方式打印你的类型时，实现 `Display`。

## Clone

`Clone` trait 明确创建值的深拷贝。

### 使用 `derive` 时，`Clone` trait 使你能够克隆一个值。

### 示例

```x
[derive(Clone)]
type Point = {
  x: integer,
  y: integer
}

let p1 = { x: 3, y: 5 }
let p2 = p1.clone()
println("p1 = {:?}, p2 = {:?}", p1, p2)
```

### 何时使用

- 当你想要能够显式复制你的类型的值时，使用 `derive(Clone)`。

## Copy

`Copy` trait 允许值被隐式复制，而不是移动。

### 使用 `derive` 时，`Copy` trait 使你的类型可以被复制而不是移动。

### 示例

```x
[derive(Copy, Clone)]
type Point = {
  x: integer,
  y: integer
}

let p1 = { x: 3, y: 5 }
let p2 = p1  // p1 被复制，而不是移动！
println("p1 仍然有效！")
```

### 何时使用

- 当你的类型简单且足够小，可以通过复制而不是移动时，使用 `derive(Copy)`。
- 注意：如果一个类型实现了 `Copy`，它也必须实现 `Clone`。
- 通常，只有完全由 `Copy` 类型组成的类型才可以是 `Copy`。
- 拥有堆分配数据的类型（如 `String` 或 `List`）不应是 `Copy`。

## PartialEq 和 Eq

`PartialEq` trait 允许你使用 `==` 和 `!=` 操作符比较值的相等性。

`Eq` trait 是一个标记 trait，表示对于类型的所有值，相等性是自反的（`a == a`）、对称的（`a == b` 意味着 `b == a`）和传递的（`a == b` 和 `b == c` 意味着 `a == c`）。

### 使用 `derive` 时，`PartialEq` trait 允许你比较你的类型的值相等或不相等。

### 示例

```x
[derive(PartialEq, Eq)]
type Point = {
  x: integer,
  y: integer
}

let p1 = { x: 3, y: 5 }
let p2 = { x: 3, y: 5 }
let p3 = { x: 1, y: 2 }
println("p1 == p2: {}", p1 == p2)  // true
println("p1 == p3: {}", p1 == p3)  // false
```

### 何时使用

- 当你想要能够比较你的类型的值相等性时，使用 `derive(PartialEq)`。
- 当相等性对于你的类型是自反的、对称的和传递的时，也使用 `derive(Eq)`。
- 大多数具有相等性概念的类型都应该派生这些 trait。

## PartialOrd 和 Ord

`PartialOrd` trait 允许你使用 `<`、`>`、`<=` 和 `>=` 比较符比较值的排序。

`Ord` trait 是一个标记 trait，表示对于类型的所有值，排序是完全的（对于任意两个值，`a < b`、`a == b` 或 `a > b` 中恰好有一个为真）。

### 使用 `derive` 时，`PartialOrd` trait 允许你比较你的类型的值的排序。

### 示例

```x
[derive(PartialOrd, Ord, PartialEq, Eq)]
type Point = {
  x: integer,
  y: integer
}

let p1 = { x: 1, y: 2 }
let p2 = { x: 3, y: 4 }
println("p1 < p2: {}", p1 < p2)  // true
println("p1 > p2: {}", p1 > p2)  // false
```

### 何时使用

- 当你想要能够排序或比较你的类型的值的排序时，使用 `derive(PartialOrd)`。
- 当排序对于你的类型是完全的时，也使用 `derive(Ord)`。

## Hash

`Hash` trait 允许你将类型的值哈希为整数。

### 使用 `derive` 时，`Hash` trait 允许你哈希你的类型的值。

### 示例

```x
[derive(Hash, PartialEq, Eq)]
type Point = {
  x: integer,
  y: integer
}

let p = { x: 3, y: 5 }
let h = hash(&p)
println("p 的哈希: {}", h)
```

### 何时使用

- 当你想要能够将你的类型的值作为 `Map` 中的键或 `Set` 中的值时，使用 `derive(Hash)`。
- 如果派生 `Hash`，还必须同时派生 `PartialEq` 和 `Eq`。

## Default

`Default` trait 为类型提供默认值。

### 使用 `derive` 时，`Default` trait 为你的类型提供默认值。

### 示例

```x
[derive(Default)]
type Point = {
  x: integer,
  y: integer
}

let p: Point = Default::default()
println("默认点: {:?}", p)  // Point { x: 0, y: 0 }
```

### 何时使用

- 当你的类型有一个合理的默认值时，使用 `derive(Default)`。
- 例如，数值类型默认为 0，可选类型默认为 `None`，等等。

## 总结

这是所有标准库 trait 的总结，你可以使用 `derive` 属性派生它们：

- **Debug** - 调试格式化
- **Display** - 用户友好的格式化（不能派生，必须手动实现）
- **Clone** - 显式复制
- **Copy** - 隐式复制
- **PartialEq** - 相等性比较
- **Eq** - 完全相等性
- **PartialOrd** - 排序比较
- **Ord** - 完全排序
- **Hash** - 哈希值
- **Default** - 默认值

大多数这些 trait 都可以使用 `derive` 自动派生，但有些（如 `Display`）需要手动实现。
