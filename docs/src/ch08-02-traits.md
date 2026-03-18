# Trait：定义共享行为

Trait 类似于其他语言中通常称为"接口"的特性，尽管有一些差异。Trait 允许我们以一种抽象的方式定义共享行为。我们可以使用 trait 约束来指定泛型类型是具有特定行为的任何类型。

注意：Trait 类似于其他语言中的接口，但有一些差异。

## 定义 Trait

类型的行为由我们可以对该类型调用的方法组成。不同类型共享相同的行为，如果我们可以对所有这些类型调用相同的方法。Trait 定义是一种将方法签名分组在一起的方法，用于定义实现某些目的所需的一组行为。

例如，假设我们有多个类型，它们都持有某种文本：`NewsArticle` 类型持有新闻故事，`Tweet` 类型持有最多 280 个字符的推文，以及可能有一些元数据。我们希望对这些类型中的每一个都能够显示摘要。因此，我们希望能够对每个类型调用 `summarize` 方法来获取该摘要。让我们看看如何在 trait 中表达这一点。

```x
trait Summary {
  function summarize(self: &Self) -> String
}
```

在这里，我们使用 `trait` 关键字声明一个 trait，后面跟着 trait 的名称，在这种情况下是 `Summary`。在大括号内部，我们声明描述实现这个 trait 的类型的行为的方法签名，在这种情况下是 `function summarize(self: &Self) -> String`。

在方法签名之后，而不是在大括号内提供一个实现，我们使用分号。每个实现这个 trait 的类型都必须提供自己的 `summarize` 方法的自定义行为。编译器将强制任何具有 `Summary` trait 的类型都具有完全此签名的 `summarize` 方法。

## 实现 Trait 在类型上

现在我们已经定义了 `Summary` trait，我们可以在我们的媒体聚合器类型上实现它。

```x
type NewsArticle = {
  headline: String,
  location: String,
  author: String,
  content: String
}

impl Summary for NewsArticle {
  function summarize(self: &NewsArticle) -> String {
    String::format("{}-{}, by {} ({})", self.headline, self.location, self.author, self.content)
  }
}

type Tweet = {
  username: String,
  content: String,
  reply: boolean,
  retweet: boolean
}

impl Summary for Tweet {
  function summarize(self: &Tweet) -> String {
    String::format("{}: {}", self.username, self.content)
  }
}
```

在类型上实现 trait 类似于实现常规方法，只是我们在 `impl` 之后添加 trait 名称，然后使用 `for` 关键字，后面跟着我们正在为其实现 trait 的类型的名称。在 `impl` 块中，我们放置 trait 定义中定义的方法签名。我们不是在每个签名后添加分号，而是在大括号中填写方法体，以指定我们希望 trait 的方法对特定类型具有的行为。

一旦我们实现了 trait，我们就可以像调用非 trait 方法一样在 `NewsArticle` 和 `Tweet` 的实例上调用 trait 方法：

```x
let tweet = {
  username: String::from("horse_ebooks"),
  content: String::from("当然，伙计们，你可能已经知道了"),
  reply: false,
  retweet: false
}
println("1 条新推文: {}", tweet.summarize())
```

这个代码打印 `1 条新推文: horse_ebooks: 当然，伙计们，你可能已经知道了`。

## Trait 作为参数

现在我们知道如何定义和实现 trait 了，让我们看看如何使用 trait 来定义接受许多不同类型的函数。例如，我们可以定义一个 `notify` 函数，该函数在其 `item` 参数上调用 `summarize` 方法，该参数是实现 `Summary` trait 的某种类型。为此，我们使用 `impl Trait` 语法：

```x
function notify(item: impl Summary) {
  println("突发新闻！{}", item.summarize())
}
```

我们可以使用 `impl` 语法，而不是具体类型作为参数的类型，而是使用 trait 名称。这个参数接受实现了我们指定的 trait 的任何类型。在 `notify` 函数体中，我们可以调用来自 `Summary` trait 的任何方法，包括 `summarize`。我们可以调用 `notify` 并传入 `NewsArticle` 或 `Tweet` 的实例。使用具体类型（如 `String` 或 `integer`）调用此函数的代码将无法编译，因为这些类型不实现 `Summary`。

## Trait Bound 语法

`impl Trait` 语法是 trait bound 的语法糖，看起来像这样：

```x
function notify<T: Summary>(item: T) {
  println("突发新闻！{}", item.summarize())
}
```

这与上一个示例等效，但稍微冗长一些。我们将 trait bound 与泛型类型参数的声明放在一起，放在尖括号内，在冒号之后。

使用 trait bound 语法的 `impl Trait` 语法很方便，在简单的情况下使代码更简洁。trait bound 语法可以在更复杂的情况下表达更多内容，例如，我们可以强制两个参数具有相同的类型。这只有在使用 trait bound 时才有可能：

```x
function notify<T: Summary>(item1: T, item2: T) {
  // ...
}
```

我们指定的泛型类型 `T` 同时指定了 `item1` 和 `item2` 的类型，它们必须是实现 `Summary` trait 的相同具体类型。如果我们使用 `impl Trait` 语法，我们不能这样做。

## 通过 + 指定多个 Trait Bound

我们也可以指定多个 trait bound。例如，如果我们想要求 `notify` 中的 `item` 既具有 `summarize` 方法又具有 `Display` trait，我们可以使用 `+` 语法：

```x
function notify(item: impl Summary + Display) {
  // ...
}
```

`+` 语法也与 trait bound 上的泛型类型参数一起使用：

```x
function notify<T: Summary + Display>(item: T) {
  // ...
}
```

通过这两个 trait bound，`notify` 的主体可以调用 `summarize` 并使用 `{}` 格式化 `item`。

## 通过 where 子句简化 Trait Bound

使用多个 trait bound 可能会有很多括号，每个泛型类型的 trait bound 列表可能会变得很长且难以阅读。出于这个原因，X 语言在函数签名之后有一个可选的 `where` 子句用于 trait bound。所以与其这样写：

```x
function some_function<T: Display + Clone, U: Clone + Debug>(t: T, u: U) -> integer {
  // ...
}
```

我们可以使用 `where` 子句：

```x
function some_function<T, U>(t: T, u: U) -> integer
  where T: Display + Clone,
        U: Clone + Debug
{
  // ...
}
```

这个函数签名不那么杂乱：函数名、参数列表和返回类型彼此靠近，类似于没有许多 trait bound 的函数。

## 返回实现 Trait 的类型

我们还可以在返回位置使用 `impl Trait` 语法，以返回实现 trait 的某种类型的值：

```x
function returns_summarizable() -> impl Summary {
  {
    username: String::from("horse_ebooks"),
    content: String::from("当然，伙计们，你可能已经知道了"),
    reply: false,
    retweet: false
  }
}
```

通过使用 `impl Summary` 作为返回类型，我们指定 `returns_summarizable` 函数返回实现 `Summary` trait 的某种类型，但没有指定具体类型。在这个例子中，`returns_summarizable` 返回一个 `Tweet`，但调用该函数的代码不需要知道这一点。

能够在闭包和迭代器的上下文中指定仅通过 `impl Trait` 语法知道实现某个 trait 的返回类型特别有用，我们将在第 10 章中介绍。闭包和迭代器创建只有编译器知道或指定起来非常冗长的类型。`impl Trait` 语法允许你简洁地指定一个函数返回实现 `Iterator` trait 的某种类型。

但是，你只能在返回单个类型时使用 `impl Trait`。例如，如果你有返回 `NewsArticle` 或 `Tweet` 的代码，两者都实现 `Summary`，那么你不能使用 `impl Summary` 作为返回类型。

## 使用 Trait Bound 有条件地实现方法

通过在 `impl` 块上使用带有泛型类型参数的 trait bound，我们可以有条件地仅针对实现指定 trait 的类型实现方法。

```x
type Pair<T> = {
  x: T,
  y: T
}

function Pair::new<T>(x: T, y: T) -> Pair<T> {
  { x: x, y: y }
}

impl<T: Display + PartialOrd> Pair<T> {
  function cmp_display(self: &Pair<T>) {
    if self.x >= self.y {
      println("最大的成员是 x = {}", self.x)
    } else {
      println("最大的成员是 y = {}", self.y)
    }
  }
}
```

`Pair<T>` 类型总是实现 `new` 函数。但是，只有当 `T` 实现了允许比较的 `PartialOrd` trait 和允许打印的 `Display` trait 时，它才会实现 `cmp_display` 方法。

我们也可以有条件地为实现另一个 trait 的任何类型实现 trait。在实现 trait 时，在满足 trait bound 的任何类型上实现 trait 称为全覆盖实现，它们在 X 语言标准库中被广泛使用。例如，标准库为实现 `Display` trait 的任何类型实现 `ToString` trait。标准库中的这个 `impl` 块看起来类似于以下代码：

```x
impl<T: Display> ToString for T {
  // --snip--
}
```

因为标准库有这个全覆盖实现，我们可以在实现 `Display` trait 的任何类型上调用 `ToString` trait 中定义的 `to_string` 方法。例如，我们可以将整数变成字符串，因为整数实现了 `Display`：

```x
let s = 3.to_string()
```

## 总结

Trait 和 trait bound 允许我们以抽象的方式定义共享行为，并让我们在不导致代码重复的情况下利用这种共享行为。它们还允许我们指定泛型类型将具有特定行为，而不仅仅是任何类型。

现在我们已经讨论了 X 语言中泛型和 trait 的一些主要特性，让我们继续讨论类和面向对象编程！

