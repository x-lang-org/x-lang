# 记录类型

在第 2 章中，我们简要介绍了记录类型作为 X 语言的基本复合类型之一。在本章中，我们将更深入地了解记录，它们与结构体的关系，以及何时你可能想要使用它们。

## 什么是记录？

记录是一种简单的数据结构，它将多个具有不同类型的值组合成一个类型。记录使用字段名来标识每个值。你可以将记录视为没有方法的轻量级结构体。

下面是我们在第 2 章中看到的记录类型定义的回顾：

```x
type Point = {
  x: Float,
  y: Float
}
```

记录类型定义以 `type` 关键字开头，后跟记录的名称，然后是大括号内的字段列表。每个字段都有一个名称（如 `x` 或 `y`）和一个类型（如 `Float`），用冒号分隔。

## 创建记录实例

要创建记录的实例，我们使用与定义记录相同的大括号语法，但我们为每个字段提供具体值：

```x
let p = { x: 0.0, y: 0.0 }
```

字段顺序不必与类型定义中的顺序匹配。以下代码也是有效的：

```x
let p = { y: 0.0, x: 0.0 }
```

## 访问记录字段

要访问记录的字段，我们使用点表示法：

```x
let p = { x: 1.0, y: 2.0 }
println("x = ", p.x)
println("y = ", p.y)
```

这将打印：

```
x = 1
y = 2
```

## 可变记录

如果要修改记录的字段，则需要将记录变量声明为可变的：

```x
let mutable p = { x: 1.0, y: 2.0 }
p.x = 5.0
p.y = 6.0
println("x = ", p.x)
println("y = ", p.y)
```

请注意，整个记录必须是可变的；X 语言不允许只将某些字段标记为可变。

## 记录更新语法

就像结构体一样，记录也支持更新语法，以从现有实例创建新实例：

```x
let p1 = { x: 1.0, y: 2.0 }
let p2 = { x: 5.0, ...p1 }
println("p2.x = ", p2.x)
println("p2.y = ", p2.y)
```

这将打印：

```
p2.x = 5
p2.y = 2
```

`...p1` 语法指定未显式设置的剩余字段应与 `p1` 中的字段具有相同的值。

## 记录与结构体

你可能想知道什么时候应该使用记录，什么时候应该使用结构体。以下是一些指导原则：

### 使用记录的情况

- 当你只需要一个简单的数据容器时
- 当你不需要任何方法时
- 当你想要轻量级的东西时
- 当你主要处理数据传输对象（DTOs）时

记录示例：

```x
// 简单的坐标点
type Point = { x: Float, y: Float }

// 表示用户数据
type UserData = {
  id: integer,
  name: String,
  email: String
}

// 配置选项
type Config = {
  debug: boolean,
  log_level: String,
  max_connections: integer
}
```

### 使用结构体的情况

- 当你需要方法时
- 当你想要封装行为时
- 当你需要更复杂的功能时
- 当你实现数据抽象时

结构体示例：

```x
// 带有区域计算方法的 Rectangle
type Rectangle = {
  width: integer,
  height: integer
}

function Rectangle::area(self: &Rectangle) -> integer {
  self.width * self.height
}

// 带有操作方法的 BankAccount
type BankAccount = {
  balance: integer,
  account_number: String
}

function BankAccount::deposit(self: &mut BankAccount, amount: integer) {
  self.balance = self.balance + amount
}

function BankAccount::withdraw(self: &mut BankAccount, amount: integer) -> boolean {
  if self.balance >= amount {
    self.balance = self.balance - amount
    true
  } else {
    false
  }
}
```

## 记录模式匹配

你也可以在模式匹配中使用记录：

```x
type Point = { x: Float, y: Float }

let p = { x: 1.0, y: 2.0 }

when p is {
  { x: 0.0, y: 0.0 } => println("原点"),
  { x: x, y: 0.0 } => println("在 x 轴上，x = ", x),
  { x: 0.0, y: y } => println("在 y 轴上，y = ", y),
  { x: x, y: y } => println("在 (", x, ", ", y, ")")
}
```

这允许你根据记录字段的值轻松地分支代码。

## 记录作为函数参数和返回值

记录作为函数参数和返回值非常有用：

```x
type Point = { x: Float, y: Float }

// 计算两点之间的距离
function distance(p1: Point, p2: Point) -> Float {
  let dx = p2.x - p1.x
  let dy = p2.y - p1.y
  sqrt(dx * dx + dy * dy)
}

// 创建一个新的点
function create_point(x: Float, y: Float) -> Point {
  { x: x, y: y }
}

// 使用这些函数
let p1 = create_point(0.0, 0.0)
let p2 = create_point(3.0, 4.0)
let dist = distance(p1, p2)
println("距离是 ", dist)
```

## 总结

记录是 X 语言中一种简单而强大的数据结构。它们：

- 使用字段名将多个值组合成一个类型
- 可以是可变的或不可变的
- 支持更新语法以方便实例化
- 可以与模式匹配一起使用
- 是简单数据容器的绝佳选择

对于更复杂的需求，你可以使用添加了方法的结构体。记录和结构体共同为你提供了在 X 语言中组织数据所需的灵活性。

既然我们已经很好地理解了结构体、枚举和记录，让我们转到模块系统，这将帮助我们组织更大的程序！
