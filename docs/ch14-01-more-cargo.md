# 关于 Cargo 和 Crates.io 的更多内容

到目前为止，我们只使用了最基本的 Cargo 功能来构建、运行和测试我们的代码，但它可以做更多的事情。在本章中，我们将讨论它的一些更高级的功能，向你展示如何做到以下几点：
- 使用发布配置文件自定义构建
- 将库发布到 Crates.io
- 使用工作空间组织大型项目
- 从 Crates.io 安装二进制文件
- 使用自定义命令扩展 Cargo

我们在本书前面介绍的功能只是 Cargo 能力的一小部分；虽然我们在这里没有完整的文档，但官方文档是关于其功能的最佳文档。

## 使用发布配置文件自定义构建

在 X 语言中，发布配置文件是预定义的、自定义的配置，允许程序员对编译选项进行更多控制。每个配置都独立于其他配置。

Cargo 有两个主要配置文件：`dev` 配置文件，Cargo 在运行 `x build` 时使用，以及 `release` 配置文件，Cargo 在运行 `x build --release` 时使用。

`dev` 配置文件定义为适合开发的良好默认值，`release` 配置文件定义为适合发布构建的良好默认值。

这些配置名称可能看起来很熟悉；它们出现在你的构建输出中：

```bash
$ x build
    Finished dev [unoptimized + debuginfo] target(s) in 0.0s
$ x build --release
    Finished release [optimized] target(s) in 0.0s
```

这里显示的 `dev` 和 `release` 表明编译器正在使用不同的配置文件。

当你的项目的 `x.toml` 中没有任何 `[profile.*]` 部分时，Cargo 对每个配置文件都有默认设置。通过将任何非默认设置添加到你想要自定义的配置文件的部分，你可以覆盖默认设置的任何子集。例如，这里是 `dev` 和 `release` 配置文件的 `opt-level` 设置的默认值：

```toml
[profile.dev]
opt-level = 0

[profile.release]
opt-level = 3
```

`opt-level` 设置控制 X 语言应该对代码应用多少优化，范围从 0 到 3。应用更多优化会延长编译时间，所以如果你在开发过程中并希望编译得快，你不希望优化太多，即使生成的代码运行得更慢。这就是为什么 `dev` 的 `opt-level` 默认是 0 的原因。当你准备发布时，最好花更多时间编译。你只会在发布模式下编译一次，但你会多次运行编译后的程序，所以发布模式会以更长的编译时间换取更快的代码运行时间。这就是为什么 `release` 的 `opt-level` 默认是 3 的原因。

你可以通过在 `x.toml` 中添加不同的值来覆盖任何默认设置。例如，假设我们想在开发配置文件中使用优化级别 1。我们可以将这两行添加到项目的 `x.toml` 中：

```toml
[profile.dev]
opt-level = 1
```

这将覆盖默认设置 `0`。现在，当我们运行 `x build` 时，Cargo 将使用 `dev` 的默认设置，除了我们覆盖的 `opt-level`。因为我们将 `opt-level` 设置为 `1`，Cargo 会比默认值多应用一些优化，但不如发布构建那么多。

有关每个配置文件的配置选项和默认值的更多信息，请参阅 [Cargo 文档](https://doc.rust-lang.org/cargo/)。

## 将 Crate 发布到 Crates.io

你可以将自己的包发布到 Crates.io，与世界其他地方分享！在发布之前，你需要先创建一个帐户并获取 API 令牌。为此，请访问 [crates.io](https://crates.io) 并使用 GitHub 帐户登录。（目前 GitHub 帐户是必需的，但将来可能会支持其他创建帐户的方式。）一旦你登录，请查看你的帐户设置并获取你的 API 密钥。然后，像这样使用该密钥登录：

```bash
$ x login abcdefghijklmnopqrstuvwxyz012345
    Login for abcdefghijklmnopqrstuvwxyz012345 stored in /Users/you/.x/credentials
```

此命令会通知 X 语言你的秘密 API 令牌，并将其本地存储在 `~/.x/credentials` 中。请确保不要与任何人共享此令牌；将其保密！如果你的令牌因任何原因被泄露，你应该立即在 Crates.io 上撤销并重新生成它。

现在你已登录，让我们看看发布需要什么！在发布之前，你需要在 crate 的 `x.toml` 文件中添加一些元数据。你需要确保你的 crate 有一个唯一的名称，设置一个描述，说明它的作用，选择一个许可证，并提供一个分类器（如果适用）。将这些内容添加到你的 `x.toml` 文件中：

```toml
[package]
name = "guessing_game"
version = "0.1.0"
authors = ["Your Name <you@example.com>"]
edition = "2024"
description = "一个有趣的猜数字游戏"
license = "MIT OR Apache-2.0"

# 这里可以添加更多键（查看文档！）

[dependencies]
```

`authors` 字段可能因你使用的 X 语言版本而异；请查看文档以获取详细信息。`description` 是对你的 crate 作用的简短描述。`license` 是一个许可证标识符值；你可以在 [SPDX 许可证列表](https://spdx.org/licenses/) 中找到你可以使用的标识符。如果你想在多个许可证下发布，请用 `OR` 分隔这些标识符。

现在我们已经配置好了所有内容，让我们发布！发布 crate 是永久性的；特定版本永远无法被覆盖，但可以发布更多版本。让我们运行 `x publish`：

```bash
$ x publish
    Updating registry `https://github.com/x-lang/crates.io-index`
   Packaging guessing_game v0.1.0 (file:///projects/guessing_game)
   Verifying guessing_game v0.1.0 (file:///projects/guessing_game)
   Compiling guessing_game v0.1.0 (file:///projects/guessing_game/target/package/guessing_game-0.1.0)
    Finished dev [unoptimized + debuginfo] target(s) in 0.43s
   Uploading guessing_game v0.1.0 (file:///projects/guessing_game)
```

注意，当你运行 `x publish` 时，`x publish` 会先打包你的 crate，然后将其上传到 [crates.io](https://crates.io)。如果其他人想使用你的 crate，他们可以像我们使用 `rand` crate 那样使用它：将其添加为依赖项并同意他们选择的任何许可证。

## Cargo 工作空间

随着项目的发展，你可能想将包拆分为多个包，以便它们更容易维护。为此，Cargo 有一个称为工作空间的功能，允许我们管理多个相关的包，这些包是一起开发的。

让我们创建一个包含二进制文件和两个库的工作空间。首先，我们将创建工作空间并添加一个二进制文件：

```bash
$ mkdir add
$ cd add
```

接下来，在 `add` 目录中，我们将创建包含工作空间配置的 `x.toml` 文件。此文件不会有 `[package]` 部分，也不会有我们在其他 `x.toml` 文件中看到的元数据。相反，它将以 `[workspace]` 开头，这将允许我们通过指定工作空间成员的路径来将成员添加到工作空间：

```toml
[workspace]

members = [
    "adder",
]
```

接下来，让我们使用 `x new` 在 `add` 目录中创建 `adder` 二进制包：

```bash
$ x new adder
     Created binary (application) `adder` package
```

此时，我们可以使用 `x build` 构建工作空间。`add` 目录中的文件应该如下所示：

```
├── Cargo.toml
├── adder
│   ├── Cargo.toml
│   └── src
│       └── main.x
└── target
```

工作空间在顶层有一个 `target` 目录；`adder` 包没有自己的 `target` 目录。即使我们从 `adder` 目录内部运行 `x build`，编译后的工件仍会在 `add/target` 中而不是 `add/adder/target` 中。Cargo 以这种方式在工作空间中构建包，因为工作空间中的包应该相互依赖。如果每个包都有自己的 `target` 目录，那么它们必须在每次想要使用另一个包时重新编译。通过共享一个 `target` 目录，包可以避免不必要的重新构建。

### 在工作空间中创建第二个包

让我们创建另一个工作空间成员包，称为 `add_one`。让我们在顶层 `x.toml` 中调整 `members` 以包含 `add-one` 路径：

```toml
[workspace]

members = [
    "adder",
    "add-one",
]
```

然后生成一个包含函数的新库包 `add-one`：

```bash
$ x new add-one --lib
     Created library `add-one` package
```

现在我们的目录应该如下所示：

```
├── Cargo.toml
├── add-one
│   ├── Cargo.toml
│   └── src
│       └── lib.x
├── adder
│   ├── Cargo.toml
│   └── src
│       └── main.x
└── target
```

让我们在 `add-one/src/lib.x` 中添加一个函数：

```x
export function add_one(x: integer) -> integer {
  x + 1
}
```

现在我们的 `adder` 包可以依赖我们的 `add-one` 包了。首先，我们需要通过在 `adder/Cargo.toml` 中添加路径依赖项来告诉 Cargo：

```toml
[dependencies]
add-one = { path = "../add-one" }
```

Cargo 不假设工作空间中的包会相互依赖，所以我们需要明确依赖关系。

接下来，让我们在 `adder` 包中使用 `add_one` 包中的 `add_one` 函数。打开 `adder/src/main.x` 并在顶部添加一个 `import` 行，将新的 `add-one` 库包引入作用域。然后修改 `main` 函数调用 `add_one` 函数。

```x
import add_one::add_one

function main() {
  let num = 10
  println("Hello, world! {} plus one is {}!", num, add_one(num))
}
```

让我们通过在顶层 `add` 目录中运行 `x build` 来构建工作空间！

```bash
$ x build
   Compiling add-one v0.1.0 (file:///projects/add/add-one)
   Compiling adder v0.1.0 (file:///projects/add/adder)
    Finished dev [unoptimized + debuginfo] target(s) in 0.68s
```

要从顶层 `add` 目录运行 `adder` 包，我们可以使用 `-p` 参数和包名来指定我们要在工作空间中运行哪个包：

```bash
$ x run -p adder
    Finished dev [unoptimized + debuginfo] target(s) in 0.0s
     Running `target/debug/adder`
Hello, world! 10 plus one is 11!
```

这运行了 `adder/src/main.x` 中的代码，它依赖于 `add-one` 包。

### 工作空间的外部依赖

注意，工作空间在顶层只有一个 `Cargo.lock`，而不是在每个包中都有。这确保所有包都使用相同版本的所有依赖项。如果我们将 `rand` 包同时添加到 `adder/Cargo.toml` 和 `add-one/Cargo.toml` 中，Cargo 会将这两个包解析为同一个版本的 `rand` 并将其记录在一个 `Cargo.lock` 中。让工作空间中的所有包使用相同的依赖项确保工作空间中的包相互兼容。

### 在工作空间中添加测试

让我们在 `add_one` 包中添加 `add_two` 函数的测试：

```x
export function add_one(x: integer) -> integer {
  x + 1
}

export function add_two(x: integer) -> integer {
  x + 2
}

test it_works {
  assert_eq!(4, add_two(2))
}
```

要运行工作空间中特定包的测试，我们可以使用 `-p` 参数并指定我们要测试的包的名称：

```bash
$ x test -p add-one
    Finished test [unoptimized + debuginfo] target(s) in 0.0s
     Running unittests (target/debug/deps/add_one-abcabcabc)

running 1 test
test it_works ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

如果我们在没有指定包的情况下在顶层目录运行 `x test`，所有测试都会运行！

如你所见，工作空间是一种组织包的便捷方式。它们可以帮助保持相关包在一起，同时允许它们保持独立。你可以选择是否在一个工作空间中拥有所有包。

## 从 Crates.io 安装二进制文件

`x install` 命令允许你在本地安装和使用二进制 crate。这不打算取代你的系统包管理器；它旨在作为一种方便的工具，供 X 语言开发者安装他们在 [crates.io](https://crates.io) 上共享的工具。注意你只能安装具有二进制目标的包。二进制目标是 crate 在其 `src/main.x` 或其他被指定为二进制的文件中有一个可执行程序，而不是仅打算作为库使用的库目标。

默认情况下，`x install` 会将已安装的二进制文件存储在你的系统的配置目录下的 `bin` 文件夹中。如果你使用 `xup` 安装了 X 语言并且没有任何非默认配置，这个目录将是 `$HOME/.x/bin`。确保该目录在你的 `$PATH` 中，以便能够运行 `x install` 安装的程序！

例如，在第 12 章中我们提到有一个 X 语言版本的 `grep` 工具，叫做 `ripgrep`，用于搜索文件。要安装 `ripgrep`，我们可以运行：

```bash
$ x install ripgrep
    Updating crates.io index
  Downloaded ripgrep v11.0.2
  Downloaded ...
   Compiling ...
    Finished release [optimized] target(s) in ...
  Installing /Users/you/.x/bin/rg
   Installed package `ripgrep v11.0.2` (executable `rg`)
```

安装输出显示了安装的位置和命令的名称，在这种情况下是 `rg`。然后，只要你的 `$PATH` 配置正确，就可以运行 `rg --help` 并开始使用更快的、用 X 语言编写的搜索文件工具！

## 使用自定义命令扩展 Cargo

Cargo 的设计使得你可以在不修改 Cargo 本身的情况下用新的子命令扩展它。如果 `$PATH` 中有名为 `x-something` 的二进制文件，你可以像 Cargo 子命令一样运行它，就像 `x something` 一样。这些自定义命令也会在你运行 `x --list` 时显示出来。能够使用 `x install` 安装插件，然后像内置的 Cargo 工具一样运行它们，这是让 Cargo 非常好扩展的一个不错的便利功能！

## 总结

在本章中，我们讨论了一些更高级的 Cargo 功能，包括：
- 发布配置文件，让你自定义构建选项
- 在 [crates.io](https://crates.io) 上发布包
- 使用工作空间管理多个相关包
- 用 `x install` 安装二进制 crate
- 用自定义命令扩展 Cargo

这些只是 Cargo 能力的一小部分；有关其所有功能的更多信息，请查看 [Cargo 文档](https://doc.rust-lang.org/cargo/)。
