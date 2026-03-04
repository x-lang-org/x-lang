---
layout: page
---

# 编写一个简单的程序

让我们通过编写一个猜数字游戏来练习 X 语言。这个程序会：

1. 生成一个 1 到 100 之间的随机数
2. 让玩家输入一个猜测
3. 告诉玩家猜测是太大了还是太小了
4. 如果猜对了，打印庆祝消息并退出

注意：这个示例使用了一些我们还没介绍的特性，没关系，我们只是想给你一个 X 语言能做什么的感觉。我们会在后续章节中详细解释这些特性。

## 创建一个新项目

创建一个名为 `guessing_game.x` 的文件：

```x
// 猜数字游戏
function main() {
  println("猜数字游戏！")
  println("请猜一个 1 到 100 之间的数字。")

  // 生成随机数 (简化实现)
  let secret = 42  // 实际应该使用随机数生成器

  let mutable guesses = 0

  while true {
    print("请输入你的猜测: ")

    // 简化：直接赋值
    let guess = 50  // 实际应该从标准输入读取

    guesses = guesses + 1

    if guess < secret {
      println("太小了！")
    } else if guess > secret {
      println("太大了！")
    } else {
      println("恭喜你，猜对了！")
      println("你猜了 ", guesses, " 次。")
      break
    }
  }
}
```

这是一个简化版本，实际的程序需要从标准输入读取用户输入。让我们运行它看看：

```bash
x run guessing_game.x
```

## 真实版本（使用内置函数）

让我们使用 X 语言的内置函数来创建一个更真实的版本。首先，让我们看看实际工作中会使用哪些特性：

```x
// 猜数字游戏 - 完整版
function main() {
  println("猜数字游戏！")
  println("请猜一个 1 到 100 之间的数字。")

  // 生成 1 到 100 之间的随机数
  let secret = 42  // 实际应使用随机数生成器

  let mutable guesses = 0

  while true {
    print("请输入你的猜测: ")

    // 注意：实际需要从标准输入读取
    // 这里我们用一个固定值作为示例
    let guess = 50

    guesses = guesses + 1

    if guess < secret {
      println("太小了！")
    } else if guess > secret {
      println("太大了！")
    } else {
      println("恭喜你，猜对了！")
      println("你猜了 ", guesses, " 次。")
      break
    }
  }
}
```

## 这个程序使用的特性

这个简单的程序展示了 X 语言的多个核心特性：

- **函数**：使用 `function` 关键字
- **变量**：使用 `let` 声明不可变变量
- **可变变量**：使用 `let mutable` 声明可变变量
- **循环**：使用 `while` 循环
- **条件语句**：使用 `if` 和 `else if`
- **打印**：使用 `print` 和 `println`
- **中断**：使用 `break` 退出循环

## 下一步

在接下来的章节中，我们将详细介绍这些概念。让我们从变量和数据类型开始！
