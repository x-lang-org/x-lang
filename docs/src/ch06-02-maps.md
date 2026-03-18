# 使用 Map 存储键值对

最后一个我们将讨论的常用集合是 `Map`。`Map<K, V>` 类型存储了 `K` 类型的键与 `V` 类型的值之间的映射。它通过一个称为哈希的过程来实现这一点，该过程决定了如何在内存中放置这些键和值。许多编程语言都支持这种数据结构，但通常使用不同的名称，例如 hash、map、object、hash table、dictionary 或 associative array，仅举几例。

当你想使用键（可以是任何类型，而不仅仅是整数）查找数据时，Map 很有用，而不是像列表那样使用索引。例如，在游戏中，你可以将每个团队的分数保存在 Map 中，其中每个键是团队的名称，值是团队的分数。

## 创建新 Map

创建空 Map 的一种方法是使用 `Map::new`。让我们创建一个空 Map 来存储团队名称和分数：

```x
let scores: Map<String, integer> = Map::new()
```

请注意，我们需要显式注解类型，因为我们还没有插入任何值。

另一种创建 Map 的方法是使用键值对语法：

```x
let scores = {
  String::from("蓝队") => 10,
  String::from("红队") => 50
}
```

这将创建一个包含两个条目的 Map："蓝队" 的分数为 10，"红队" 的分数为 50。这种语法类似于我们用于记录的语法，但有一个重要区别：Map 可以在运行时增长和缩小，而记录具有固定的字段集，在编译时已知。

## 读取 Map 的值

我们可以通过向 `get` 方法提供其键来从 Map 中获取值，如清单所示。

```x
let scores = {
  String::from("蓝队") => 10,
  String::from("红队") => 50
}

let team_name = String::from("蓝队")
let score = scores.get(team_name)
when score is {
  Some(s) => println("分数: ", s),
  None => println("未找到团队。")
}
```

在这里，`score` 将具有与蓝队相关联的值，结果将是 `Some(10)`。如果 Map 中没有该键的条目，`get` 将返回 `None`。

我们也可以像遍历列表元素那样使用 `for` 循环遍历 Map 中的每个键值对：

```x
for (key, value) in scores {
  println(key, ": ", value)
}
```

这将以任意顺序打印每个键值对：

```
蓝队: 10
红队: 50
```

## 更新 Map

虽然 Map（像列表一样）通常是不可变的，但我们可以创建包含新条目的新 Map。让我们看看如何更新 Map。

### 添加新条目

我们可以使用 `Map::insert` 方法向 Map 添加新条目：

```x
let scores = {
  String::from("蓝队") => 10,
  String::from("红队") => 50
}
let scores2 = Map::insert(scores, String::from("黄队"), 30)
```

这将创建一个包含所有原始条目加上新条目的新 Map。

### 覆盖值

如果我们插入一个键已经存在的条目，该键的旧值将被替换。即使我们调用 `Map::insert` 两次，使用相同的键但不同的值，Map 中该键只会有一个值：

```x
let scores = { String::from("蓝队") => 10 }
let scores2 = Map::insert(scores, String::from("蓝队"), 25)
```

此时，`scores2` 将只有一个键 "蓝队"，值为 25。原始值 10 已被覆盖。

### 仅在键没有值时插入

通常，检查特定键是否已有值，如果没有，则为其插入一个值是很有用的。Map 为此有一个特殊的 API，称为 `Map::entry`，它将你要检查的键作为参数。让我们看看如何使用 `entry`：

```x
let scores = { String::from("蓝队") => 10 }
let scores2 = Map::entry(scores, String::from("蓝队"), 50)  // 不会覆盖
let scores3 = Map::entry(scores2, String::from("黄队"), 50)  // 会插入新值
```

`entry` 方法检查键是否存在；如果存在，它会保留现有值；如果不存在，它会插入新值。

## 常见的 Map 操作

`Map` 类型有许多有用的方法。让我们看看其中一些。

### 获取 Map 大小

我们可以使用 `len` 方法获取 Map 中的条目数：

```x
let scores = {
  String::from("蓝队") => 10,
  String::from("红队") => 50
}
println("Map 大小: ", scores.len())  // 打印 2
```

### 检查键是否存在

我们可以使用 `contains_key` 方法检查 Map 是否包含特定键：

```x
let scores = { String::from("蓝队") => 10 }
println("有蓝队吗? ", scores.contains_key(String::from("蓝队")))  // true
println("有黄队吗? ", scores.contains_key(String::from("黄队")))  // false
```

### 删除条目

我们可以使用 `Map::remove` 方法从 Map 中删除条目：

```x
let scores = {
  String::from("蓝队") => 10,
  String::from("红队") => 50
}
let scores2 = Map::remove(scores, String::from("蓝队"))
```

### 获取所有键或值

我们可以使用 `keys` 和 `values` 方法分别获取所有键或所有值的列表：

```x
let scores = {
  String::from("蓝队") => 10,
  String::from("红队") => 50
}
let team_names = scores.keys()  // [String::from("蓝队"), String::from("红队")]
let score_values = scores.values()  // [10, 50]
```

## Map 和所有权

对于实现 `Copy` trait 的类型，如 `integer`，值会被复制到 Map 中。对于拥有的值，如 `String`，值会被移动，Map 将成为这些值的所有者，如清单所示。

```x
let field_name = String::from("最喜欢的颜色")
let field_value = String::from("蓝色")

let map = { field_name => field_value }

// field_name 和 field_value 在这里不再有效，
// 因为它们已被移动到 map 中
```

如果我们将对值的引用插入到 Map 中，这些值不会被移动到 Map 中。引用指向的值必须在 Map 有效的至少同一时间内有效。我们将在第 10 章讨论这些问题。

## 哈希函数

默认情况下，`Map` 使用密码学上强大的哈希函数，该函数可以抵抗拒绝服务（DoS）攻击。这不是可用的最快哈希算法，但为了安全性而放弃一点性能是值得的。如果你发现默认哈希函数对于你的目的太慢，你可以通过指定不同的 hasher 来切换到另一个函数。Hasher 是实现 `Hasher` trait 的类型。我们将在第 8 章讨论 trait 以及如何实现它们。

## 总结

列表、Map 和 Set 将涵盖你在编程中需要存储、访问和修改数据的许多常见情况。以下是一些练习，让你有机会实践我们在本章中学到的知识：

1. 给定一个整数列表，使用 Map 返回它们的平均值（mean）、中位数（当排序后位于中间位置的值）和众数（出现频率最高的值）。
2. 将字符串转换为 pig latin。每个单词的第一个辅音字母移到单词末尾并添加 "ay"，因此 "first" 变成 "irst-fay"。以元音开头的单词则在末尾添加 "hay"（"apple" 变成 "apple-hay"）。
3. 使用 Map 和 Set，创建一个文本界面，允许用户将员工姓名添加到公司的部门中；例如，"Add Sally to Engineering" 或 "Add Amir to Sales"。然后让用户检索部门中所有人员的列表，或按部门检索公司中所有人员的列表，并进行字母排序。

标准库 API 文档中描述了这些集合类型提供的方法，因此一定要检查一下！

现在我们已经介绍了 X 语言中一些比较常见的集合，让我们继续讨论 Set。

