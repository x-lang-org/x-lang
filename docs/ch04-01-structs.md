# 结构体

结构体（struct）是一种自定义数据类型，允许你命名和包装多个相关的值，这些值构成一个有意义的组。如果你熟悉面向对象的语言，结构体就像对象的数据属性。在本章中，我们将比较和对比记录与结构体，以建立你已经知道的知识。

## 定义和实例化结构体

结构体类似于我们在第 2 章中讨论的记录类型，但它们有一些额外的功能。让我们首先回顾一下记录类型，然后看看结构体是如何扩展它们的。

### 快速回顾：记录类型

在第 2 章中，我们看到了如何定义记录类型：

```x
type Point = {
  x: Float,
  y: Float
}

let p = { x: 0.0, y: 0.0 }
```

记录对于分组相关数据非常有用。结构体通过添加方法和其他功能来扩展这个概念。

### 定义结构体

我们使用与定义记录类型相同的语法来定义结构体，但我们可以选择添加方法。让我们定义一个存储关于用户帐户信息的结构体：

```x
type User = {
  username: String,
  email: String,
  sign_in_count: integer,
  active: boolean
}
```

这个结构体的定义有四个字段：`username` 是 `String` 类型，`email` 是 `String` 类型，`sign_in_count` 是 `integer` 类型，`active` 是 `boolean` 类型。

### 实例化结构体

要使用这个结构体，我们通过为每个字段指定具体值来创建该结构体的实例。我们通过声明结构体名称，然后在大括号内添加 `key: value` 对来创建实例，其中键是字段的名称，值是我们要存储在这些字段中的数据。我们不必按照我们在结构体中声明它们的相同顺序指定字段。换句话说，结构体定义就像该类型的通用模板，实例用特定数据填充该模板。例如，我们可以声明一个特定的用户，如下所示：

```x
let user1 = {
  username: String::from("someusername123"),
  email: String::from("someone@example.com"),
  sign_in_count: 1,
  active: true
}
```

为了从结构体中获取特定值，我们使用点表示法。如果我们只想要这个用户的电子邮件地址，我们可以在需要该值的任何地方使用 `user1.email`。如果实例是可变的，我们可以通过使用点表示法并赋值到特定字段来更改值。清单显示了如何更改可变 `User` 实例的 `email` 字段中的值。

```x
let mutable user2 = {
  username: String::from("someusername123"),
  email: String::from("someone@example.com"),
  sign_in_count: 1,
  active: true
}

user2.email = String::from("anotheremail@example.com")
```

请注意，整个实例必须是可变的；X 语言不允许我们只将某些字段标记为可变。与任何表达式一样，我们可以在函数体的最后一个表达式中构造并隐式返回该结构体的新实例。

清单显示了一个 `build_user` 函数，它返回一个 `User` 实例，其中包含给定的电子邮件和用户名。`active` 字段的值为 `true`，`sign_in_count` 的值为 `1`。

```x
function build_user(email: String, username: String) -> User {
  {
    username: username,
    email: email,
    sign_in_count: 1,
    active: true
  }
}
```

让函数参数与结构体字段同名是有意义的，但必须重复 `email` 和 `username` 字段名和变量有点乏味。如果结构体有更多字段，重复每个名称会变得更加烦人。幸运的是，有一个方便的简写！

### 使用字段初始化简写语法

因为参数名和字段名完全相同，我们可以使用字段初始化简写语法重写 `build_user`，使其行为完全相同，但不必重复 `username` 和 `email`，如下所示：

```x
function build_user(email: String, username: String) -> User {
  {
    username,
    email,
    sign_in_count: 1,
    active: true
  }
}
```

在这里，我们正在创建一个 `User` 类型的新实例，它有一个名为 `username` 的字段。我们想将 `username` 字段的值设置为 `build_user` 函数的 `username` 参数中的值。因为 `username` 字段和 `username` 参数具有相同的名称，我们只需要写 `username` 而不是 `username: username`。

### 使用结构体更新语法从另一个实例创建实例

通常，从现有实例创建新实例并重用其大部分值但更改某些值是很有用的。你可以使用结构体更新语法来做到这一点。

首先，清单显示了如何在不使用更新语法的情况下从 `user1` 创建一个新的 `User` 实例 `user2`。我们在 `user2` 中为 `email` 设置了一个新值，但其他值与我们在 `user1` 中创建的实例相同。

```x
let user2 = {
  username: user1.username,
  email: String::from("another@example.com"),
  sign_in_count: user1.sign_in_count,
  active: user1.active
}
```

使用结构体更新语法，我们可以用更少的代码实现相同的效果，如清单所示。`..` 语法指定未显式设置的剩余字段应与给定实例中的字段具有相同的值。

```x
let user2 = {
  email: String::from("another@example.com"),
  ...user1
}
```

清单中的代码也创建了一个 `user2` 实例，它的 `email` 与 `user1` 中的不同，但 `username`、`sign_in_count` 和 `active` 字段的值与 `user1` 中的相同。`...user1` 必须放在最后，以指定任何剩余字段应从 `user1` 中的字段获取它们的值，但我们可以选择以任何顺序为任意数量的字段指定值，而不管结构体定义中字段的顺序如何。

请注意，结构更新语法会像赋值一样移动数据，就像我们在第 3 章中讨论的那样。在这种情况下，我们在创建 `user2` 后不能再使用 `user1`，因为 `user1` 的 `username` 字段中的 `String` 已移动到 `user2`。如果我们给 `user2` 赋予新的 `email` 和 `username` 值，并且只使用 `user1` 中的 `sign_in_count` 和 `active` 值，那么 `user1` 在创建 `user2` 后仍然有效。`sign_in_count` 和 `active` 是实现 `Copy` trait 的类型，因此我们在第 3 章中讨论的行为适用。

### 没有命名字段的元组结构体

你还可以定义看起来类似于元组的结构体，称为元组结构体。元组结构体具有结构体名称提供的含义，但没有与其字段关联的名称；相反，它们只有字段的类型。当你想给整个元组一个名称并使元组与其他元组具有不同类型时，元组结构体很有用，并且当你像常规结构体那样命名每个字段时会显得冗长或多余。

要定义元组结构体，请以 `type` 关键字开头，后跟结构体名称和元组中的类型。例如，这里我们定义并使用两个名为 `Color` 和 `Point` 的元组结构体：

```x
type Color = (integer, integer, integer)
type Point = (integer, integer, integer)

let black = Color(0, 0, 0)
let origin = Point(0, 0, 0)
```

请注意，`black` 和 `origin` 值是不同的类型，因为它们是不同元组结构体的实例。你定义的每个结构体都是它自己的类型，即使结构体中的字段具有相同的类型。例如，一个以 `Color` 作为参数的函数不能接受 `Point` 作为参数，即使这两种类型都由三个 `integer` 值组成。否则，元组结构体实例的行为类似于元组：你可以将它们解构为它们的单独部分，你可以使用 `.` 后跟索引来访问单个值，依此类推。

## 结构体的方法语法

方法类似于函数：我们用 `function` 关键字声明它们，它们可以有参数和返回值，并且它们包含一些代码，当从其他地方调用该方法时会运行这些代码。与函数不同，方法是在结构体（或枚举或 trait 对象，我们将在第 4 章和第 13 章分别介绍）的上下文中定义的，并且它们的第一个参数总是 `self`，它代表调用该方法的结构体的实例。

### 定义方法

让我们将以 `Rectangle` 实例为参数的 `area` 函数改为定义在 `Rectangle` 结构体上的 `area` 方法。

首先，让我们定义一个 `Rectangle` 类型：

```x
type Rectangle = {
  width: integer,
  height: integer
}
```

要在 `Rectangle` 的上下文中定义函数，我们需要为这个结构体实现方法。让我们看看如何添加一个计算矩形面积的 `area` 方法：

```x
function Rectangle::area(self: &Rectangle) -> integer {
  self.width * self.height
}
```

要在 `Rectangle` 的上下文中定义函数，我们使用 `Rectangle::` 前缀将函数与 `Rectangle` 类型关联起来。然后，我们可以使用点语法在 `Rectangle` 实例上调用这个方法：

```x
let rect = { width: 30, height: 50 }
println(
  "矩形的面积是 ",
  rect.area(),
  " 平方单位。"
)
```

在 `area` 的签名中，我们使用 `self: &Rectangle` 而不是 `rectangle: &Rectangle`，因为该方法位于 `Rectangle::` 命名空间中，所以我们知道 `self` 的类型是 `Rectangle`。

注意，我们可以选择将 `self` 作为第一个参数，或者像我们在这里做的那样借用 `self`，就像我们可以使用任何其他参数一样。在这里，我们选择了借用 `self`，就像我们在函数版本中所做的那样。我们不希望方法获取所有权，我们只想读取结构体中的数据，而不是写入它。如果我们想在我们作为方法调用的一部分的实例中改变某些东西，我们会使用 `self: &mut Rectangle` 作为第一个参数。

通过在第一个参数中使用 `self` 而不是借用结构体的所有权，方法可以选择获取所有权、不可变借用或可变借用，就像它们可以对任何其他参数一样。

在主要调用 `area` 时，我们使用 `rect.area()` 来调用 `rect` 上的 `area` 方法。这比 `area(&rect)` 好得多，因为它更符合我们将方法附加到我们的值的直觉。

### 具有更多参数的方法

让我们通过实现 `Rectangle` 类型的第二个方法来练习使用方法。这一次，我们希望一个 `Rectangle` 实例接受另一个 `Rectangle` 实例，如果第二个 `Rectangle` 可以完全放入 `self`（第一个 `Rectangle`）中，则返回 `true`；否则，它应该返回 `false`。也就是说，一旦我们定义了 `can_hold` 方法，我们就想能够编写程序。

```x
let rect1 = { width: 30, height: 50 }
let rect2 = { width: 10, height: 40 }
let rect3 = { width: 60, height: 45 }
println("rect1 能容纳 rect2 吗？", rect1.can_hold(&rect2))
println("rect1 能容纳 rect3 吗？", rect1.can_hold(&rect3))
```

我们希望看到这样的输出，因为 `rect2` 的两个维度都小于 `rect1` 的维度，但 `rect3` 比 `rect1` 宽：

```
rect1 能容纳 rect2 吗？ true
rect1 能容纳 rect3 吗？ false
```

我们知道我们想要定义一个方法，它将在 `Rectangle::` 命名空间中。对于参数，我们将使用 `other: &Rectangle`，它将是对我们传递给 `can_hold` 的第二个 `Rectangle` 实例的不可变借用。该方法将返回一个 `boolean`。实现将检查 `self` 的宽度是否大于 `other` 的宽度，以及 `self` 的高度是否大于 `other` 的高度。让我们添加新的 `can_hold` 方法。

```x
function Rectangle::can_hold(self: &Rectangle, other: &Rectangle) -> boolean {
  self.width > other.width && self.height > other.height
}
```

当我们在 `rect1` 上使用 `rect2` 和 `rect3` 作为参数运行这段代码时，我们将看到所需的输出。方法可以接受多个参数，我们在 `self` 参数之后将其添加到签名中，这些参数的工作方式与函数中的参数完全相同。

### 关联函数

在 `Rectangle::` 命名空间中定义的不将 `self` 作为第一个参数的函数称为关联函数，因为它们与该类型关联。它们仍然是函数，而不是方法，因为它们没有可以处理的结构体的实例。你已经使用了 `String::from` 关联函数。

关联函数通常用于将返回该结构体新实例的构造函数。例如，我们可以提供一个接受一个维度参数并将其用作宽度和高度的关联函数，这样我们就可以轻松创建正方形的 `Rectangle`，而不必指定相同的值两次：

```x
function Rectangle::square(size: integer) -> Rectangle {
  { width: size, height: size }
}
```

要调用这个关联函数，我们使用结构体名称和 `::` 语法；例如 `let sq = Rectangle::square(3)`。这个函数由结构体命名：`::` 语法既用于关联函数，也用于模块创建的命名空间。我们将在第 5 章讨论模块。

## 总结

结构体允许你创建自定义类型，这些类型对于你的域有意义。通过使用结构体，你可以保持相关的数据片段相互关联，并为每个数据片段命名，使代码更清晰。在结构体的命名空间中，你可以定义方法并指定关联函数，这些方法指定你的自定义类型可以具有的行为。

但是结构体并不是创建自定义类型的唯一方法：让我们转向 X 语言的枚举特性，为你的工具箱添加另一个工具。
