# 类与对象

X 语言支持多种编程范式：函数式、声明式、过程式，以及面向对象编程（OOP）。在本章中，我们将探讨 X 语言中的面向对象特性：类、对象、继承等。

## 什么是面向对象编程？

面向对象的程序由对象组成。对象将数据和操作该数据的过程包装在一起。这些过程通常被称为方法或操作。

对于这个定义，X 语言是面向对象的：

- 结构体和枚举可以包含数据
- `impl` 块提供了可以在结构体和枚举上调用的方法
- 类（我们将在本章中讨论）提供了更传统的 OOP 特性

让我们首先看看 X 语言中的类是什么。

## 定义类

X 语言中的类使用 `class` 关键字定义。一个类可以包含字段（数据）和方法（函数）。让我们从一个简单的 `Animal` 类开始：

```x
class Animal {
  let name: String
  let age: integer

  constructor(name: String, age: integer) {
    this.name = name
    this.age = age
  }

  function make_sound(self: &Self) {
    println("动物发出声音！")
  }

  function get_name(self: &Self) -> String {
    self.name.clone()
  }

  function get_age(self: &Self) -> integer {
    self.age
  }
}
```

让我们分解这个类定义：

1. **字段声明**：`let name: String` 和 `let age: integer` 声明了类的字段（数据成员）。
2. **构造函数**：`constructor(name: String, age: integer)` 是一个特殊方法，用于初始化类的新实例。`this` 关键字指的是正在创建的实例。
3. **方法**：`make_sound`、`get_name` 和 `get_age` 是可以在类实例上调用的方法。`self: &Self` 参数指的是方法被调用的实例（类似于其他语言中的 `this` 或 `self`）。

## 创建对象

要创建类的实例（对象），我们像调用函数一样调用构造函数：

```x
let animal = Animal::new(String::from("Buddy"), 5)
println("动物的名字是: ", animal.get_name())
println("动物的年龄是: ", animal.get_age())
animal.make_sound()
```

这将打印：

```
动物的名字是: Buddy
动物的年龄是: 5
动物发出声音！
```

## 可变对象

默认情况下，对象是不可变的。要修改对象的字段，我们需要声明对象为可变的，并且在类中提供修改字段的方法：

```x
class MutableAnimal {
  let mutable name: String
  let mutable age: integer

  constructor(name: String, age: integer) {
    this.name = name
    this.age = age
  }

  function set_name(self: &mut Self, new_name: String) {
    self.name = new_name
  }

  function have_birthday(self: &mut Self) {
    self.age = self.age + 1
    println("生日快乐！现在 ", self.name, " 是 ", self.age, " 岁了！")
  }

  function get_name(self: &Self) -> String {
    self.name.clone()
  }

  function get_age(self: &Self) -> integer {
    self.age
  }
}
```

现在我们可以创建一个可变对象并修改它：

```x
let mutable mutable_animal = MutableAnimal::new(String::from("Buddy"), 5)
println("原始名字: ", mutable_animal.get_name())

mutable_animal.set_name(String::from("Max"))
println("新名字: ", mutable_animal.get_name())

mutable_animal.have_birthday()
```

这将打印：

```
原始名字: Buddy
新名字: Max
生日快乐！现在 Max 是 6 岁了！
```

## 封装

面向对象编程的一个关键原则是封装：对象的内部细节对外部代码是隐藏的。只有对象的公共方法是可访问的。

在 X 语言中，默认情况下，类字段和方法是私有的。要使它们公开，我们使用 `public` 关键字：

```x
class BankAccount {
  let mutable balance: integer  // 私有字段

  constructor(initial_balance: integer) {
    this.balance = initial_balance
  }

  public function deposit(self: &mut Self, amount: integer) {
    if amount > 0 {
      this.balance = this.balance + amount
    }
  }

  public function withdraw(self: &mut Self, amount: integer) -> boolean {
    if amount > 0 && this.balance >= amount {
      this.balance = this.balance - amount
      true
    } else {
      false
    }
  }

  public function get_balance(self: &Self) -> integer {
    this.balance
  }
}
```

在这个例子中：
- `balance` 字段是私有的，不能从类外部直接访问
- `deposit`、`withdraw` 和 `get_balance` 方法是公共的，可以从外部调用
- `withdraw` 方法确保在允许提款之前账户有足够的资金

这封装了银行账户的内部状态并强制执行业务规则：

```x
let mutable account = BankAccount::new(1000)
println("初始余额: ", account.get_balance())  // 1000

account.deposit(500)
println("存款后: ", account.get_balance())  // 1500

let success = account.withdraw(300)
println("提款成功: ", success)  // true
println("提款后: ", account.get_balance())  // 1200

// account.balance  // 错误！无法访问私有字段
```

## 类与结构体

你可能想知道什么时候应该使用类，什么时候应该使用结构体。这里有一些指导原则：

### 使用类的情况：
- 你需要继承
- 你需要封装（私有字段）
- 你正在建模具有身份和行为的对象
- 你需要传统的面向对象特性

### 使用结构体的情况：
- 你只需要简单的数据容器
- 你不需要继承
- 你想要更轻量级的东西
- 你主要使用函数式风格

## 总结

X 语言通过以下方式支持面向对象编程：
- 使用 `class` 关键字定义类
- 具有字段（数据）和方法（行为）的对象
- 使用构造函数初始化对象
- 使用 `this` 或 `self` 引用当前实例
- 封装与公共/私有可见性
- 可变对象与可变字段和方法

在下一章中，我们将探讨继承，这是另一个关键的面向对象特性！

