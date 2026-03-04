---
layout: page
---

# 介绍

## 什么是 X 语言？

X 语言是一门现代通用编程语言，设计遵循以下核心原则：

| 原则 | 描述 |
|------|------|
| **可读性第一** | 代码应该像散文一样可读，宁可多打几个字符也不牺牲可读性 |
| **类型安全** | 编译通过就不应出现类型错误，无 null、无异常 |
| **内存安全** | Perceus 编译时引用计数，无 GC、无手动管理、无泄漏 |
| **多范式融合** | 函数式、面向对象、过程式、声明式自由选择 |
| **英文全称关键字** | 不使用缩写，含义自明 |
| **基础类型小写** | `integer`、`float`、`boolean`、`string`、`character` |
| **不使用奇怪符号** | 只用键盘上常见的、一看就懂的符号 |
| **工具链完整** | `x` CLI 1:1 对标 Cargo，开箱即用 |

## 四种编程范式

X 语言支持四种编程范式，开发者可以根据场景选择最适合的方式：

### 函数式（数学 + 管道）

```x
let topUsers = users |> filter(.active) |> sortBy(.score) |> take(10)
```

### 声明式（自然语言 where/sort by）

```x
let topUsers = users
  where   .active and .score > 80
  sort by .score descending
  take    10
```

### 面向对象（方法链）

```x
let topUsers = users.filter(.active).sortBy(.score).take(10)
```

### 过程式（let mutable + for）

```x
function getTopUsers() {
  let mutable result = []
  for u in users {
    if u.active and .score > 80 {
      result = result + [u]
    }
  }
  result |> sortBy(.score) |> take(10)
}
```

## 快速预览

让我们通过一个简单的例子来看看 X 语言的样子：

```x
// 计算斐波那契数列的函数
function fib(n) {
  if n <= 1 {
    return n
  }
  return fib(n - 1) + fib(n - 2)
}

let i = 0
while i < 10 {
  print("fib(")
  print(i)
  print(") = ")
  print(fib(i))
  print("\n")
  i = i + 1
}
```

注意，在这个例子中我们没有使用 `main` 函数！在 X 语言中，`main` 函数不是必须的——你可以像 Swift 一样，直接在文件顶层编写代码。

这个例子展示了 X 语言的几个关键特性：

- 使用 `function` 关键字定义函数
- 使用 `let` 声明不可变变量
- 使用 `let mutable` 声明可变变量
- `if` 和 `while` 语句不需要圆括号
- 使用 `print` 和 `println` 输出
- 支持递归函数
- `main` 函数是可选的

## 下一步

在下一章中，我们将学习如何安装 X 语言，然后编写你的第一个 "Hello, World!" 程序。让我们开始吧！
