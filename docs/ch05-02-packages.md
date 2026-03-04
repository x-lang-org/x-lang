# 包和 Crate

在本章中，我们将讨论包和 crate——X 语言中代码组织的两个基本概念。让我们首先看看包和 crate 是什么，然后看看它们如何协同工作。

## 什么是 Crate？

Crate 是 X 语言中编译的最小单位。每次运行 `x compile something.x` 时，那个 `something.x` 文件都被视为一个 crate。Crate 可以是二进制 crate 或库 crate。

### 二进制 Crate

二进制 crate 是可以编译为可以运行的可执行文件的程序。它们必须有一个名为 `main` 的函数，该函数定义了可执行文件运行时发生的事情。我们到目前为止创建的所有 crate 都是二进制 crate。

### 库 Crate

库 crate 没有 `main` 函数，也不会编译为可执行文件。相反，它们定义了旨在在多个项目之间共享的功能。例如，如果我们写了一个提供有用数学函数的 crate，我们可以在多个不同的项目中使用该 crate。

我们使用 crate 的方式是将它们用作库。大多数时候，当人们说"crate"时，他们指的是库 crate，并且他们几乎可以互换使用"crate"和"library"。

## 什么是包？

包是提供一组功能的一个或多个 crate。包包含一个 `x.toml` 文件，该文件描述了如何构建这些 crate。让我们创建一个包！

### 创建包

让我们创建一个名为 `hello_world` 的新包。为此，请运行以下命令：

```bash
x new hello_world
```

这将创建一个名为 `hello_world` 的目录，其中包含以下文件：

```
hello_world/
├── x.toml
└── src/
    └── main.x
```

让我们看看 `x.toml` 包含什么：

```toml
[package]
name = "hello_world"
version = "0.1.0"
edition = "2024"

[dependencies]
```

此文件采用 TOML（Tom's Obvious, Minimal Language）格式，这是 X 包的配置格式。

第一行 `[package]` 表示以下部分正在配置包。接下来的三行设置我们的包构建所需的配置：名称、版本和要使用的 X 语言 edition。最后一部分 `[dependencies]` 是我们将为包添加任何依赖项的部分。

现在让我们看看 `src/main.x`：

```x
function main() {
  println("Hello, world!")
}
```

`x new` 已经为我们生成了一个"Hello, world!"程序！让我们运行它：

```bash
cd hello_world
x run
```

你应该会看到输出：

```
Hello, world!
```

## 包的规则

包可以包含多个二进制 crate，但最多只能包含一个库 crate。让我们看看 `x new` 生成的包：在 `src` 目录中，我们有：

- `src/main.x` - 这是一个与包同名的二进制 crate 的根
- `src/lib.x` - 如果我们添加这个文件，包将有一个与包同名的库 crate

我们也可以通过将文件放在 `src/bin` 目录中来拥有多个二进制 crate：`src/bin` 目录中的每个文件都是一个单独的二进制 crate。

### 库 Crate

让我们向我们的包中添加一个库 crate。创建一个名为 `src/lib.x` 的文件，内容如下：

```x
// src/lib.x
export function add(a: integer, b: integer) -> integer {
  a + b
}
```

现在我们的包同时包含一个库 crate 和一个二进制 crate！我们可以在二进制 crate 中使用我们的库 crate。修改 `src/main.x`：

```x
// src/main.x
import hello_world::add

function main() {
  println("2 + 3 = {}", add(2, 3))
}
```

运行它：

```bash
x run
```

你应该会看到输出：

```
2 + 3 = 5
```

## 使用外部 Crate

让我们看看如何使用外部 crate。假设我们想使用一个名为 `rand` 的 crate，它提供随机数生成功能。首先，我们需要将它添加到我们的 `x.toml` 中：

```toml
[dependencies]
rand = "0.8.5"
```

现在我们可以在代码中使用 `rand` crate：

```x
import rand::random

function main() {
  let secret_number = random::<integer>()
  println("随机数: {}", secret_number)
}
```

运行 `x build`，X 会自动下载 `rand` 及其所有依赖项，构建它们，然后构建我们的包。

## 构建和运行包

X 提供了几个用于处理包的命令：

- **`x new`** - 创建一个新包
- **`x build`** - 构建包
- **`x run`** - 构建并运行二进制 crate
- **`x test`** - 运行包的测试
- **`x doc`** - 构建包的文档
- **`x check`** - 快速检查包的错误而不生成可执行文件

## 总结

在本章中，我们介绍了：

- **Crate** - X 语言中编译的最小单位（二进制或库）
- **包** - 一个或多个 crate，带有 `x.toml` 描述如何构建它们
- **`x.toml`** - 包的配置文件
- **`src/main.x`** - 二进制 crate 的根
- **`src/lib.x`** - 库 crate 的根
- **`src/bin/`** - 额外二进制 crate 的目录
- **依赖项** - 在 `x.toml` 中指定，并由 X 自动管理

包和 crate 是组织 X 语言代码的基础！
