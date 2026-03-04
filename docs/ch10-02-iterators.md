# 迭代器

迭代器是处理一系列元素的模式。迭代器本身负责管理序列的逻辑，并确定何时处理完每个元素。当你使用迭代器时，你不必自己重新实现这些逻辑。

在 X 语言中，迭代器是惰性的——这意味着在你调用消耗迭代器的方法之前，它们不会有任何效果。让我们来看看迭代器的实际应用！

## Iterator trait

迭代器模式的核心是 `Iterator` trait，它有一个名为 `next` 的方法，返回 `Option<Self::Item>`。让我们看看如何实现 `Iterator` trait。

但首先，让我们看看如何使用标准库提供的迭代器。

## 使用迭代器处理列表

我们可以通过调用 `iter()` 方法从列表创建迭代器：

```x
let v = [1, 2, 3]
let iter = v.iter()
```

迭代器存储了我们需要处理的序列的所有状态。一旦我们有了迭代器，我们就可以用多种方式使用它。

### 使用 for 循环与迭代器

我们可以使用 `for` 循环来遍历迭代器中的元素：

```x
let v = [1, 2, 3]
for element in v {
  println("得到: {}", element)
}
```

这将打印：

```
得到: 1
得到: 2
得到: 3
```

### 调用 next 方法

我们也可以直接在迭代器上调用 `next` 方法：

```x
let v = [1, 2, 3]
let mutable iter = v.iter()

println(iter.next())  // Some(1)
println(iter.next())  // Some(2)
println(iter.next())  // Some(3)
println(iter.next())  // None
```

注意，我们需要将 `iter` 设为可变的：在迭代器上调用 `next` 方法会更改迭代器用来跟踪它在序列中的位置的内部状态。换句话说，这个代码消耗（使用）了迭代器。每次对 `next` 的调用都从迭代器中消耗一个元素。

## 消耗迭代器的方法

定义在 `Iterator` trait 上的各种方法（我们称之为迭代器适配器）有不同的用途。一些方法调用 `next`，因此它们被称为消耗适配器，因为调用它们会消耗迭代器。

例如，`sum` 方法，它消耗迭代器并通过反复调用 `next` 遍历所有元素，并在遍历过程中将每个元素相加：

```x
let v = [1, 2, 3]
let total: integer = v.iter().sum()
println(total)  // 6
```

`sum` 方法获取迭代器的所有权并通过遍历元素来消耗它。它将每个元素添加到运行总和中，并在迭代完成时返回总和。

## 生成其他迭代器的方法

定义在 `Iterator` trait 上的其他方法（我们称之为迭代器适配器）允许你将迭代器更改为不同类型的迭代器。你可以通过链接多个迭代器适配器调用来执行复杂的操作，并且仍然具有可读性。但请记住，因为所有迭代器都是惰性的，你需要调用一个消耗适配器来从适配器调用中获得结果。

### map

`map` 适配器接受一个闭包，并对每个元素调用该闭包，生成一个新的迭代器：

```x
let v = [1, 2, 3]
let mapped: List<integer> = v.iter()
  .map(function(x) { x + 1 })
  .collect()
println(mapped)  // [2, 3, 4]
```

这里，`map` 适配器将闭包应用于每个元素，`collect` 消耗迭代器并将结果收集到列表中。

### filter

`filter` 适配器接受一个闭包，该闭包对每个元素返回 `true` 或 `false`，生成一个只包含闭包返回 `true` 的元素的新迭代器：

```x
let v = [1, 2, 3, 4, 5, 6]
let evens: List<integer> = v.iter()
  .filter(function(x) { x % 2 == 0 })
  .collect()
println(evens)  // [2, 4, 6]
```

### 链接多个适配器

迭代器适配器的强大之处在于你可以将它们链接在一起以可读的方式执行复杂操作：

```x
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let result: List<integer> = numbers.iter()
  .filter(function(x) { x % 2 == 0 })      // [2, 4, 6, 8, 10]
  .map(function(x) { x * x })               // [4, 16, 36, 64, 100]
  .filter(function(x) { x > 10 })           // [16, 36, 64, 100]
  .take(3)                                   // [16, 36, 64]
  .collect()
println(result)  // [16, 36, 64]
```

这是对偶数进行平方，筛选出大于 10 的，取前 3 个，并收集结果。

## 其他常见的迭代器适配器

这里有一些你可能会发现有用的其他迭代器适配器：

### take

`take` 适配器从迭代器中获取前 n 个元素：

```x
let v = [1, 2, 3, 4, 5]
let first_three: List<integer> = v.iter().take(3).collect()
println(first_three)  // [1, 2, 3]
```

### skip

`skip` 适配器跳过前 n 个元素：

```x
let v = [1, 2, 3, 4, 5]
let after_two: List<integer> = v.iter().skip(2).collect()
println(after_two)  // [3, 4, 5]
```

### enumerate

`enumerate` 适配器返回元素及其索引：

```x
let v = ["a", "b", "c"]
for (i, element) in v.iter().enumerate() {
  println("索引 {}: {}", i, element)
}
```

这将打印：

```
索引 0: a
索引 1: b
索引 2: c
```

### fold

`fold` 适配器使用累加器对迭代器中的所有元素进行归约：

```x
let v = [1, 2, 3, 4, 5]
let sum = v.iter().fold(0, function(acc, x) { acc + x })
println(sum)  // 15

let product = v.iter().fold(1, function(acc, x) { acc * x })
println(product)  // 120
```

### any 和 all

`any` 检查是否有任何元素满足条件，而 `all` 检查是否所有元素都满足条件：

```x
let v = [1, 2, 3, 4, 5]
let has_even = v.iter().any(function(x) { x % 2 == 0 })
let all_positive = v.iter().all(function(x) { x > 0 })
println(has_even)       // true
println(all_positive)   // true
```

## 实现 Iterator trait

你可以实现 `Iterator` trait 来创建自己的迭代器。让我们创建一个从 1 计数到某个数字的迭代器：

```x
type Counter = {
  current: integer,
  max: integer
}

function Counter::new(max: integer) -> Counter {
  { current: 1, max: max }
}

impl Iterator for Counter {
  type Item = integer

  function next(self: &mut Self) -> Option<integer> {
    if self.current <= self.max {
      let result = Some(self.current)
      self.current = self.current + 1
      result
    } else {
      None
    }
  }
}

// 使用我们的迭代器
let counter = Counter::new(5)
for num in counter {
  println(num)
}
```

这将打印：

```
1
2
3
4
5
```

## 迭代器的性能

你可能会想知道迭代器是否有性能成本。答案是否定的！X 语言中的迭代器被编译为高效的代码——通常与手写循环一样快，甚至更快！迭代器是零成本抽象的一个例子，这意味着使用它们不会增加运行时开销。

## 总结

X 语言中的迭代器：
- 实现 `Iterator` trait，其中有 `next` 方法
- 是惰性的——在你调用消耗它们的方法之前不会有效果
- 有消耗适配器，如 `sum` 和 `collect`
- 有迭代器适配器，如 `map` 和 `filter`，它们将迭代器转换为其他迭代器
- 可以链接在一起以复杂但可读的方式处理数据
- 你可以通过实现 `Iterator` trait 来创建自己的迭代器
- 编译为高效代码，没有运行时开销

迭代器是 X 语言函数式编程工具包的另一个重要组成部分。在下一章中，我们将讨论管道运算符，这是一种将值传递给函数的优雅方式！
