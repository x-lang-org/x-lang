# 模块系统

随着项目的增长，你需要通过将代码拆分为多个文件，然后拆分为多个模块来组织你的代码。当代码库变大时，将相关功能分组并分离不同的功能变得至关重要。

X 语言的模块系统包括：

- **模块**：允许你组织代码并控制路径的隐私
- **路径**：一种命名项目的方式，如函数、类型或模块
- **import**：将路径引入作用域
- **export**：使项目可从外部访问

让我们按顺序讨论所有这些概念！

## 模块定义

你可以使用 `module` 关键字声明一个新模块，后跟模块的名称和大括号，这些大括号包含模块的内容。让我们从定义一些模块开始，以组织我们的示例代码。

```x
// 在名为 garden.x 的文件中
module plants {
  // 植物相关的代码
}

module animals {
  // 动物相关的代码
}
```

模块可以嵌套：

```x
module garden {
  module plants {
    function grow() {
      println("植物正在生长！")
    }
  }

  module animals {
    function eat() {
      println("动物正在吃东西！")
    }
  }
}
```

## 路径

要引用模块中的项目，我们需要知道它的路径。路径有两种形式：

1. **绝对路径**：从根模块开始，使用模块名或字面量路径。
2. **相对路径**：从当前模块开始，使用 `self`、`super` 或当前模块中的标识符。

绝对路径和相对路径后面跟着一个或多个由双冒号（`::`）分隔的标识符。

让我们看一下我们的花园模块示例：

```x
module garden {
  module plants {
    function grow() {
      println("植物正在生长！")
    }
  }
}

function main() {
  // 绝对路径
  garden::plants::grow()

  // 相对路径（如果在适当的模块中）
  // plants::grow()
}
```

我们可以使用绝对路径从 `main` 函数调用 `grow` 函数，该路径从根模块开始并导航到 `garden`，然后到 `plants`，最后到 `grow`。

### 使用 super 开始相对路径

我们还可以通过使用 `super` 开头来构建从父模块开始的相对路径。这就像在文件系统中使用 `..` 语法从父目录开始一样。使用 `super` 允许我们引用父模块中的项目，当模块彼此靠近时，这有助于重新组织模块树，而不必重写大量路径。

```x
module garden {
  function water() {
    println("给花园浇水！")
  }

  module plants {
    function grow() {
      super::water()  // 调用父模块中的 water 函数
      println("植物正在生长！")
    }
  }
}
```

## 导出项目

默认情况下，模块中的所有内容都是私有的（private）。父模块中的代码不能使用子模块中的私有代码，但子模块中的代码可以使用其祖先模块中的代码。这是因为模块应该封装其实现细节。

要使模块中的项目公开，我们可以使用 `export` 关键字。让我们看一个例子：

```x
module garden {
  export module plants {
    export function grow() {
      println("植物正在生长！")
    }
  }
}

function main() {
  // 现在可以访问，因为 plants 模块和 grow 函数都已导出
  garden::plants::grow()
}
```

在这个例子中，我们导出了 `plants` 模块和 `grow` 函数，使它们可以从 `garden` 模块外部访问。

## 导入项目

使用模块的完整路径调用函数可能很长并且会重复。在 X 语言中，我们有一个方法可以使用 `import` 关键字将路径引入作用域，从而使这个过程更短。让我们看看如何将 `garden::plants::grow` 路径引入作用域：

```x
module garden {
  export module plants {
    export function grow() {
      println("植物正在生长！")
    }
  }
}

import garden::plants::grow

function main() {
  grow()  // 现在我们可以直接调用 grow！
}
```

我们也可以使用 `import` 来导入整个模块：

```x
import garden::plants

function main() {
  plants::grow()
}
```

### 导入多个项目

我们可以使用大括号导入多个项目：

```x
import garden::plants::{ grow, water }
```

### 使用 as 重命名导入

有时，你可能想要导入两个具有相同名称的项目，或者你可能想要为导入的项目赋予不同的名称。我们可以使用 `as` 关键字来做到这一点：

```x
import garden::plants::grow as grow_plant
import garden::animals::grow as grow_animal

function main() {
  grow_plant()
  grow_animal()
}
```

## 将模块拆分为多个文件

到目前为止，我们已经在单个文件中定义了所有模块。当模块变大时，你可能希望将它们的定义移动到单独的文件中，以便代码更易于导航。

让我们重构我们的花园示例，将每个模块放在自己的文件中。首先，让我们创建一个项目结构，如下所示：

```
garden.x
plants.x
animals.x
```

在 `garden.x` 中：

```x
// garden.x
import "plants.x" as plants
import "animals.x" as animals

function main() {
  plants::grow()
  animals::eat()
}
```

在 `plants.x` 中：

```x
// plants.x
export function grow() {
  println("植物正在生长！")
}
```

在 `animals.x` 中：

```x
// animals.x
export function eat() {
  println("动物正在吃东西！")
}
```

通过使用 `import` 和文件路径，我们可以将代码拆分为多个文件，同时仍然能够在它们之间引用项目。

## 总结

X 语言的模块系统允许你：

- 使用 `module` 关键字组织代码到模块中
- 使用绝对路径或相对路径引用项目
- 使用 `export` 关键字使项目公开
- 使用 `import` 关键字将路径引入作用域
- 将模块拆分为多个文件以提高可读性

模块系统是构建大型代码库的强大工具。通过适当使用模块，你可以保持代码的组织性和可维护性！

