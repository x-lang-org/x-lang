# 闭包

X 语言支持函数式编程特性，包括闭包——可以捕获其环境中值的匿名函数。在本章中，我们将了解闭包是什么、它们如何工作以及何时使用它们。

## 什么是闭包？

闭包是可以存储在变量中或作为参数传递给其他函数的匿名函数。与函数不同，闭包可以捕获它们定义的作用域中的值。

让我们从一个简单的闭包示例开始：

```x
let add_one = function(x) { x + 1 }

println(add_one(5))  // 6
```

这里，`add_one` 是一个接受一个参数 `x` 并返回 `x + 1` 的闭包。闭包的语法类似于函数，但有一些区别：

- `function` 关键字（可选的，在某些上下文中）
- 参数周围的括号
- 箭头 `=>`（可选的，取决于语法）
- 没有显式类型注解（通常可以推断）

## 闭包类型推断

与函数不同，闭包通常不需要你注解参数或返回值的类型。类型是从闭包的使用方式推断出来的。

```x
// 闭包不强制类型注解
let add = function(x, y) { x + y }

// 第一次使用确定类型
let result = add(5, 10)  // add 现在是 integer -> integer -> integer
println(result)  // 15
```

但是，如果你想显式注解类型，你可以这样做：

```x
let add: function(integer, integer) -> integer = function(x, y) { x + y }
```

## 捕获环境

闭包的一个强大特性是它们可以捕获其环境——它们定义的作用域中的变量。

```x
function main() {
  let x = 4
  let equal_to_x = function(z) { z == x }
  let y = 4
  println(equal_to_x(y))  // true
}
```

这里，`equal_to_x` 闭包从其环境中捕获了变量 `x`。它将 `x` 的值与它接受的参数 `z` 进行比较。

## 捕获方式：借用与移动

闭包可以通过三种方式捕获它们的环境，对应于函数获取参数的三种方式：不可变借用、可变借用和获取所有权。闭包会自动确定使用哪种方式，具体取决于它如何使用捕获的值。

### 不可变借用

如果闭包只读取值，它将通过不可变借用捕获它：

```x
let list = [1, 2, 3]
let only_borrows = function() { println("我借用了列表: {:?}", list) }
only_borrows()  // 我借用了列表: [1, 2, 3]
```

### 可变借用

如果闭包修改值，它将通过可变借用捕获它：

```x
let mutable list = [1, 2, 3]
let mutable borrows_mutably = function() { list = list + [4] }
mut_borrows_mutably()
println(list)  // [1, 2, 3, 4]
```

### 移动

如果闭包获取值的所有权（例如，如果它将值返回或将其移动到别处），它将通过移动捕获值：

```x
let list = [1, 2, 3]
let takes_ownership = function() {
  let moved_list = list
  println("我拥有了列表: {:?}", moved_list)
}
takes_ownership()
// println(list)  // 错误！list 已经被移动
```

我们也可以使用 `move` 关键字强制闭包获取它使用的环境值的所有权，即使它在技术上不需要这样做：

```x
let list = [1, 2, 3]
let owns_list = move function() {
  println("我拥有了列表: {:?}", list)
}
owns_list()
// println(list)  // 错误！list 已经被移动
```

当我们将闭包传递给新线程时，`move` 关键字最常用，因为我们想将数据的所有权从一个线程转移到另一个线程。我们将在关于并发的章节中看到这方面的例子。

## 将闭包作为参数

闭包作为函数参数非常有用。让我们创建一个接受闭包作为参数的函数：

```x
function apply_twice(f: function(integer) -> integer, x: integer) -> integer {
  f(f(x))
}

let add_two = function(x) { x + 2 }
let result = apply_twice(add_two, 5)
println(result)  // 5 + 2 + 2 = 9
```

在这里，`apply_twice` 接受一个函数 `f` 和一个整数 `x`，并将 `f` 应用于 `x` 两次。

## 返回闭包

我们也可以从函数返回闭包。但是，闭包类型是匿名的，所以我们需要使用 `impl` 语法或 trait 对象：

```x
function create_adder(x: integer) -> impl function(integer) -> integer {
  function(y) { x + y }
}

let add_five = create_adder(5)
println(add_five(3))  // 8
```

`create_adder` 接受一个整数 `x` 并返回一个将 `x` 添加到其参数的闭包。返回的闭包捕获 `x`。

## 闭包作为迭代器适配器

闭包在迭代器中特别有用，我们将在下一章讨论。这是一个预览：

```x
let numbers = [1, 2, 3, 4, 5]
let doubled: List<integer> = numbers
  .iter()
  .map(function(n) { n * 2 })
  .collect()
println(doubled)  // [2, 4, 6, 8, 10]
```

## 闭包与函数指针

你也可以使用普通函数代替闭包，当你想要的逻辑不需要捕获环境中的任何内容时：

```x
function add_one(x: integer) -> integer {
  x + 1
}

let result = apply_twice(add_one, 5)
println(result)  // 7
```

## 闭包的性能

你可能想知道闭包是否有性能成本。好消息是，X 语言中的闭包被编译为高效的代码——通常与手写函数一样高效！每个闭包都有自己独特的类型，即使两个闭包具有相同的签名，因此编译器可以专门化并优化每个闭包的使用。

## 总结

X 语言中的闭包：
- 是可以捕获其环境的匿名函数
- 可以存储在变量中并作为参数传递
- 可以通过不可变借用、可变借用或移动捕获值
- 可以使用 `move` 关键字强制获取所有权
- 可以作为函数参数和返回值
- 对于迭代器和高阶函数特别有用
- 编译为高效代码，没有运行时开销

闭包是 X 语言函数式编程工具包的重要组成部分。在下一章中，我们将讨论迭代器，它们经常与闭包一起使用！
