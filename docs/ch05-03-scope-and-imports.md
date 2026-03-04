# 引用模块树中项目的路径

在第 5-01 章中，我们介绍了模块树的基础。在本章中，我们将更深入地讨论如何使用路径（path）引用模块树中的项目。

## 路径语法

引用模块树中的项目有两种方式：

1. **绝对路径**（absolute path）从包根（crate root）开始，以包名或 `crate` 开头
2. **相对路径**（relative path）从当前模块开始，以 `self`、`super` 或当前模块中的标识符开头

绝对路径和相对路径后面都跟随着一个或多个由双冒号（`::`）分隔的标识符。

让我们回到第 5-01 章的例子：

```x
// package.x
name = "restaurant"

// src/main.x
mod front_of_house {
  mod hosting {
    function add_to_waitlist() {}
  }
}

function eat_at_restaurant() {
  // 绝对路径
  crate::front_of_house::hosting::add_to_waitlist()

  // 相对路径
  front_of_house::hosting::add_to_waitlist()
}
```

在 `eat_at_restaurant` 中，我们可以用两种方式调用 `add_to_waitlist`。第一种是绝对路径：我们从 `crate`（包根）开始，然后列出每个连续的模块，就像在文件系统中浏览目录一样。

第二种是相对路径：它从当前模块（`main.x` 的根）开始，然后 `front_of_house`、`hosting`、`add_to_waitlist` 跟在后面。这就像文件系统中的路径 `front_of_house/hosting/add_to_waitlist`。

选择使用相对路径还是绝对路径是一个取决于你的项目的决定，取决于你是更倾向于将项目的定义代码与使用代码一起移动还是分开移动。例如，如果我们将 `front_of_house` 模块和 `eat_at_restaurant` 函数一起移动到一个名为 `customer_experience` 的模块中，我们就需要更新绝对路径，但相对路径仍然有效！但是，如果我们将 `eat_at_restaurant` 函数单独移动到一个名为 `dining` 的模块中，绝对路径将保持不变，但相对路径需要更新。我们的偏好是优先使用绝对路径，因为我们更可能希望代码的定义和使用彼此独立地移动。

## 使用 `pub` 关键字控制可见性

在第 5-01 章中，我们简要介绍了 `pub` 关键字，但没有深入讨论它的细节。默认情况下，X 语言中的所有内容都是私有的（private），但父模块中的项不能使用子模块中的私有项，而子模块中的项可以使用其所有祖先模块中的项。

让我们再次看看 `hosting` 模块。默认情况下，它是私有的。我们可以在模块前添加 `pub` 关键字使其公开：

```x
mod front_of_house {
  pub mod hosting {
    function add_to_waitlist() {}
  }
}
```

但是，`add_to_waitlist` 函数仍然是私有的！使模块公开不会使其内容公开。模块上的 `pub` 关键字只允许其父模块引用它，而不能访问其内部代码。因为模块是一个容器，仅仅使模块公开并没有太大作用；我们还需要选择使其中的一些项公开。

让我们通过在 `add_to_waitlist` 函数前添加 `pub` 关键字使其公开：

```x
mod front_of_house {
  pub mod hosting {
    pub function add_to_waitlist() {}
  }
}

function eat_at_restaurant() {
  // 绝对路径
  crate::front_of_house::hosting::add_to_waitlist()

  // 相对路径
  front_of_house::hosting::add_to_waitlist()
}
```

现在代码可以编译了！让我们看看绝对路径和相对路径为什么能够访问 `add_to_waitlist`，这与私有性规则有关。

因为我们在 `hosting` 模块和 `add_to_waitlist` 函数上都使用了 `pub`，所以我们可以从 `eat_at_restaurant` 访问和调用 `add_to_waitlist`。

### `pub` 的其他用法

让我们看另一个例子：

```x
mod back_of_house {
  function fix_incorrect_order() {
    cook_order()
    super::serve_order()
  }

  function cook_order() {}
}

function serve_order() {}
```

`back_of_house` 模块及其函数都是私有的，但 `fix_incorrect_order` 可以调用同一模块中的 `cook_order`，因为它们在同一模块中。`fix_incorrect_order` 还可以使用 `super::` 调用 `serve_order`：`super::` 让我们从父模块开始，这类似于在文件系统中使用 `..` 开头。

## 使用 `use` 关键字将路径引入作用域

到目前为止，我们一直通过完整路径调用函数，这可能会重复和冗长。在 X 语言中，我们可以使用 `use` 关键字将路径引入作用域，这样我们就可以像使用本地项一样使用它们。

让我们看看如何使用 `use` 将 `front_of_house::hosting` 模块引入作用域，这样我们就可以直接调用 `hosting::add_to_waitlist`：

```x
mod front_of_house {
  pub mod hosting {
    pub function add_to_waitlist() {}
  }
}

use crate::front_of_house::hosting

function eat_at_restaurant() {
  hosting::add_to_waitlist()
}
```

将 `hosting` 模块引入作用域就像在文件系统中创建符号链接一样。通过在包根添加 `use crate::front_of_house::hosting`，`hosting` 现在在该作用域中是一个有效的名称，就像 `hosting` 模块是在包根中定义的一样。

我们也可以使用 `use` 和相对路径引入项：

```x
use front_of_house::hosting
```

### 创建惯用的 `use` 路径

在前面的例子中，你可能想知道为什么我们要写 `use crate::front_of_house::hosting` 然后在 `eat_at_restaurant` 中调用 `hosting::add_to_waitlist`，而不是直接引入 `add_to_waitlist` 函数：

```x
mod front_of_house {
  pub mod hosting {
    pub function add_to_waitlist() {}
  }
}

use crate::front_of_house::hosting::add_to_waitlist

function eat_at_restaurant() {
  add_to_waitlist()
}
```

虽然两者都可以工作，但前者（引入模块）是惯用的方式。引入模块而不是单个函数可以让我们调用 `hosting::add_to_waitlist()` 而不是直接 `add_to_waitlist()`，这使得 `add_to_waitlist` 来自哪个模块更加清晰。

对于结构体、枚举和其他项，习惯上引入完整路径：

```x
use std::collections::Map

function main() {
  let mut map = Map::new()
  map.insert(String::from("key"), String::from("value"))
}
```

这个习惯用法没有硬性规定：它只是一个人们习惯的约定。

### 使用 `as` 关键字提供新名称

使用 `use` 引入项时的另一个解决方案是在路径后使用 `as` 关键字和新名称：

```x
use std::collections::Map as HashMap

function main() {
  let mut map = HashMap::new()
}
```

### 使用 `pub use` 重新导出名称

当我们使用 `use` 关键字将名称引入作用域时，该名称在新作用域中是私有的。如果我们想让调用我们代码的代码能够像在该代码自己的作用域中定义那样使用该类型，我们可以将 `pub` 和 `use` 结合起来。这种技术称为"重新导出"（re-exporting），因为我们将一个项引入作用域，同时也使该项可供其他人引入他们的作用域。

```x
mod front_of_house {
  pub mod hosting {
    pub function add_to_waitlist() {}
  }
}

pub use crate::front_of_house::hosting

function eat_at_restaurant() {
  hosting::add_to_waitlist()
}
```

现在，外部代码可以使用 `restaurant::hosting::add_to_waitlist()` 了！如果我们没有指定 `pub use`，`eat_at_restaurant` 可以在其作用域内调用 `hosting::add_to_waitlist()`，但外部代码不能利用这个新路径。

## 使用嵌套路径清理大型 `use` 列表

如果我们使用同一个包或模块中的多个项，为每个项单独列出一行会占用我们文件中大量的垂直空间。

```x
use std::collections::List
use std::collections::Map
use std::collections::Set
```

相反，我们可以使用嵌套路径将相同的项引入作用域，但只用一行！我们可以通过指定路径的公共部分，然后是两个冒号，然后是花括号内路径的不同部分来实现，如下所示：

```x
use std::collections::{List, Map, Set}
```

### 全局引入操作符 `*`

如果我们想将一个路径下的所有公共项引入作用域，我们可以使用 `*` 操作符：

```x
use std::collections::*
```

这个 `use` 语句将 `std::collections` 中定义的所有公共项引入当前作用域。使用 `*` 时要小心：它会使更难分辨作用域中有哪些名称以及程序中使用的名称来自哪里。

全局引入操作符通常用于测试，用于将所有测试的内容引入 `tests` 模块；我们将在第 11 章"如何编写测试"中讨论这一点。它也有时用于 prelude 模式；查看标准库文档以获取该模式的更多详细信息。

## 总结

在本章中，我们学习了：

- 如何使用绝对路径或相对路径引用模块树中的项
- 如何使用 `pub` 使项公开
- 如何使用 `use` 将项引入作用域
- 惯用的引入方式
- 如何使用 `as` 重命名引入的项
- 如何使用 `pub use` 重新导出项
- 如何使用嵌套路径和全局引入简化 `use` 语句

这些是模块系统的核心概念！有了这些知识，你就可以组织代码并控制哪些内容是公开可见的了。
