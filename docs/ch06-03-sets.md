# 使用 Set 存储唯一值

我们将在本章讨论的最后一个常用集合是 `Set`。`Set<T>` 类型是唯一值的集合。与列表不同，Set 保证不包含重复元素。与 Map 一样，Set 是一个用于存储值的集合，但在 Set 中，每个值都是自己的键，并且没有关联的值。

当你想确保没有重复值时，Set 很有用。例如，你可以使用 Set 来跟踪访问过的网站、事件中的唯一访客，或单词在文档中出现的唯一单词。

## 创建新 Set

创建空 Set 的一种方法是使用 `Set::new`：

```x
let unique_numbers: Set<integer> = Set::new()
```

请注意，我们需要显式注解类型，因为我们还没有插入任何值。

另一种创建 Set 的方法是使用从列表或值序列创建 Set 的语法：

```x
let unique_numbers = Set::from([1, 2, 3, 4, 5])
```

或者使用花括号语法：

```x
let unique_numbers = { 1, 2, 3, 4, 5 }
```

这两种方法都会创建一个包含五个唯一整数的 Set。

## Set 自动处理重复项

Set 最有用的特性之一是它们会自动删除重复项。让我们看看这是如何工作的：

```x
let numbers = [1, 2, 2, 3, 3, 3, 4, 4, 4, 4]
let unique_numbers = Set::from(numbers)
```

此时，`unique_numbers` 将只包含 `{1, 2, 3, 4}`。所有重复项都已自动删除。

## 读取 Set 的值

我们可以检查 Set 中是否存在某个值，使用 `contains` 方法：

```x
let numbers = { 1, 2, 3, 4, 5 }
println("有 3 吗? ", numbers.contains(3))  // true
println("有 6 吗? ", numbers.contains(6))  // false
```

我们也可以像遍历列表元素那样使用 `for` 循环遍历 Set 中的每个元素：

```x
let numbers = { 1, 2, 3, 4, 5 }
for number in numbers {
  println(number)
}
```

这将以任意顺序打印每个数字。

## 更新 Set

与其他集合一样，Set 通常是不可变的，但我们可以创建包含新元素的新 Set。

### 添加元素

我们可以使用 `Set::insert` 方法向 Set 添加元素：

```x
let numbers = { 1, 2, 3 }
let numbers2 = Set::insert(numbers, 4)
let numbers3 = Set::insert(numbers2, 4)  // 不会改变 Set，因为 4 已经存在
```

`Set::insert` 返回一个新 Set，如果元素已存在，则新 Set 与原始 Set 相同。

### 删除元素

我们可以使用 `Set::remove` 方法从 Set 中删除元素：

```x
let numbers = { 1, 2, 3, 4, 5 }
let numbers2 = Set::remove(numbers, 3)
```

此时，`numbers2` 将包含 `{1, 2, 4, 5}`。

## 集合操作

Set 对集合操作有很好的支持，例如并集、交集、差集和对称差集。让我们看看这些如何工作。

### 并集

两个 Set 的并集是一个包含任一 Set 中所有元素的新 Set：

```x
let a = { 1, 2, 3, 4 }
let b = { 3, 4, 5, 6 }
let union = Set::union(a, b)  // {1, 2, 3, 4, 5, 6}
```

### 交集

两个 Set 的交集是一个包含同时出现在两个 Set 中的所有元素的新 Set：

```x
let a = { 1, 2, 3, 4 }
let b = { 3, 4, 5, 6 }
let intersection = Set::intersection(a, b)  // {3, 4}
```

### 差集

两个 Set 的差集是一个包含第一个 Set 中但不在第二个 Set 中的所有元素的新 Set：

```x
let a = { 1, 2, 3, 4 }
let b = { 3, 4, 5, 6 }
let difference = Set::difference(a, b)  // {1, 2}
```

### 对称差集

两个 Set 的对称差集是一个包含任一 Set 中但不同时在两个 Set 中的所有元素的新 Set：

```x
let a = { 1, 2, 3, 4 }
let b = { 3, 4, 5, 6 }
let symmetric_difference = Set::symmetric_difference(a, b)  // {1, 2, 5, 6}
```

## 常见的 Set 操作

`Set` 类型有许多有用的方法。让我们看看其中一些。

### 获取 Set 大小

我们可以使用 `len` 方法获取 Set 中的元素数：

```x
let numbers = { 1, 2, 3, 4, 5 }
println("Set 大小: ", numbers.len())  // 打印 5
```

### 检查 Set 是否为空

我们可以使用 `is_empty` 方法检查 Set 是否为空：

```x
let empty_set: Set<integer> = {}
println("Set 为空: ", empty_set.is_empty())  // true
```

### 子集和超集

我们可以检查一个 Set 是否是另一个 Set 的子集（包含在其中）或超集（包含另一个）：

```x
let a = { 1, 2, 3 }
let b = { 1, 2, 3, 4, 5 }

println("a 是 b 的子集吗? ", Set::is_subset(a, b))  // true
println("b 是 a 的超集吗? ", Set::is_superset(b, a))  // true
```

### 转换为列表

我们可以使用 `to_list` 方法将 Set 转换为列表：

```x
let numbers = { 1, 2, 3, 4, 5 }
let number_list = Set::to_list(numbers)  // [1, 2, 3, 4, 5] 或其他顺序
```

## Set 实际应用示例

让我们看一个 Set 在实际中有用的示例。假设我们正在构建一个网站跟踪器，想要跟踪访问过我们网站的唯一用户：

```x
type Visitor = {
  id: String,
  name: String
}

// 创建一些访问者
let alice = { id: String::from("1"), name: String::from("Alice") }
let bob = { id: String::from("2"), name: String::from("Bob") }
let charlie = { id: String::from("3"), name: String::from("Charlie") }

// 跟踪唯一访问者
let mutable unique_visitors = { alice, bob }

// Alice 再次访问 - 不会添加重复项
unique_visitors = Set::insert(unique_visitors, alice)

// Charlie 第一次访问
unique_visitors = Set::insert(unique_visitors, charlie)

println("唯一访问者数量: ", unique_visitors.len())  // 3
```

在这个示例中，即使 Alice 访问了两次，她在 Set 中也只被计算一次。

## 总结

Set 是 X 语言标准库中一个强大的集合类型。它们：

- 自动处理重复值
- 支持快速查找操作
- 提供标准集合操作，如并集、交集和差集
- 对于跟踪唯一值或执行集合数学运算非常有用

列表、Map 和 Set 共同为你提供了在 X 语言中管理数据集合所需的大多数工具。

现在我们已经介绍了 X 语言中的常见集合，让我们继续讨论 X 语言处理错误的方式！
