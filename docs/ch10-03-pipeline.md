# 管道运算符

X 语言的一个独特特性是管道运算符 `|>`，它允许你以一种从左到右的方式将值传递给函数，这可以使代码更具可读性。管道运算符是 X 语言函数式编程工具包的重要组成部分。

## 什么是管道运算符？

管道运算符 `|>` 接受左侧的值并将其作为参数传递给右侧的函数。

让我们从一个简单的例子开始：

```x
function double(x: integer) -> integer {
  x * 2
}

function add_one(x: integer) -> integer {
  x + 1
}

// 不使用管道的常规函数调用
let result1 = add_one(double(5))
println(result1)  // 11

// 使用管道的相同调用
let result2 = 5 |> double |> add_one
println(result2)  // 11
```

两种方式都做同样的事情，但使用管道，我们从值开始，然后按操作发生的顺序应用操作：先 `double`，然后 `add_one`。

## 为什么使用管道？

管道运算符在以下几个方面使代码更具可读性：

1. **从左到右的顺序**：操作按视觉上发生的顺序排列
2. **减少嵌套**：避免了深层嵌套的函数调用
3. **关注点分离**：每个步骤在自己的行上清晰可见

让我们看一个更复杂的例子，展示了好处：

```x
function process_data(data: List<integer>) -> List<integer> {
  data
    |> List::filter(function(x) { x > 0 })
    |> List::map(function(x) { x * 2 })
    |> List::take(5)
}

// 不使用管道的相同代码
function process_data_without_pipe(data: List<integer>) -> List<integer> {
  List::take(
    List::map(
      List::filter(data, function(x) { x > 0 }),
      function(x) { x * 2 }
    ),
    5
  )
}
```

管道版本更容易阅读！每个操作都按发生的顺序清晰可见。

## 带多个参数的管道

如果函数接受多个参数怎么办？管道会将值作为第一个参数传递。让我们看看：

```x
function add(a: integer, b: integer) -> integer {
  a + b
}

function multiply(a: integer, b: integer) -> integer {
  a * b
}

// 5 作为第一个参数传递给 add
let result = 5 |> add(3) |> multiply(2)
println(result)  // (5 + 3) * 2 = 16
```

这里，`5 |> add(3)` 等价于 `add(5, 3)`。

## 带闭包的管道

管道与闭包配合得非常好。让我们看看：

```x
let result = "hello world"
  |> String::to_uppercase
  |> function(s) { s + "!" }
  |> println

// 等价于:
// println(function(s) { s + "!" }(String::to_uppercase("hello world")))
```

## 带方法的管道

你也可以在管道中使用方法。请记住，方法接受 `self` 作为第一个参数：

```x
type Person = {
  name: String,
  age: integer
}

function Person::new(name: String, age: integer) -> Person {
  { name: name, age: age }
}

function Person::celebrate_birthday(self: &mut Person) {
  self.age = self.age + 1
}

function Person::greet(self: &Person) -> String {
  String::format("你好，我是 {}，我 {} 岁了", self.name, self.age)
}

let mutable person = Person::new(String::from("Alice"), 30)
  |> Person::celebrate_birthday
  |> Person::greet
  |> println

// 输出: 你好，我是 Alice，我 31 岁了
```

## 带 Option 和 Result 的管道

管道与 `Option` 和 `Result` 类型配合得特别好，使错误处理代码更具可读性：

```x
function parse_number(s: String) -> Option<integer> {
  if s == String::from("42") {
    Some(42)
  } else {
    None
  }
}

function double_if_some(opt: Option<integer>) -> Option<integer> {
  Option::map(opt, function(x) { x * 2 })
}

let result = String::from("42")
  |> parse_number
  |> double_if_some
  |> Option::unwrap_or(0)

println(result)  // 84
```

## 链式多个管道

管道运算符的真正威力来自于链接多个操作。让我们看一个数据处理的例子：

```x
type Order = {
  id: integer,
  product: String,
  quantity: integer,
  price: Float
}

let orders = [
  { id: 1, product: String::from("Apple"), quantity: 5, price: 0.50 },
  { id: 2, product: String::from("Banana"), quantity: 3, price: 0.30 },
  { id: 3, product: String::from("Orange"), quantity: 10, price: 0.60 },
  { id: 4, product: String::from("Apple"), quantity: 2, price: 0.50 }
]

let total_revenue = orders
  |> List::filter(function(o) { o.quantity > 2 })
  |> List::map(function(o) { o.quantity * o.price })
  |> List::sum()

println("总收入: ${}", total_revenue)
```

这个管道：
1. 筛选出数量大于 2 的订单
2. 将每个订单映射到其收入（数量 * 价格）
3. 对所有收入求和

## 最佳实践

这里有一些有效使用管道的最佳实践：

1. **保持每个步骤简单**：如果步骤变得复杂，请使用命名函数而不是内联闭包
2. **每行一个步骤**：这使管道易于阅读
3. **必要时使用注释**：对于不明显的复杂管道，添加解释性注释
4. **不要过度使用**：对于简单的函数调用，普通调用语法可能更好

## 总结

X 语言中的管道运算符：
- 使用 `|>` 语法将值从左传递到右
- 使数据转换链更具可读性
- 减少嵌套括号
- 按逻辑发生顺序显示操作
- 与函数、闭包、方法和泛型配合良好
- 对于数据处理和函数式编程特别有用

管道运算符是使 X 语言代码更清晰、更具声明性的强大工具！

现在我们已经介绍了 X 语言的函数式编程特性，让我们继续讨论测试！
