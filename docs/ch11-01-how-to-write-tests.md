# 如何编写测试

测试是确保代码按预期工作的关键部分。X 语言内置了对测试的支持，因此你可以直接在代码中编写测试并运行它们。

在本章中，我们将介绍如何使用 X 语言内置的测试功能编写测试。我们将讨论用于编写测试的语法，以及用于运行测试和查看测试输出的选项。我们还将讨论如何组织测试以及如何编写单元测试和集成测试。

## 测试函数的剖析

在最简单的形式中，X 语言中的测试是一个用 `test` 属性注释的函数，用于验证某些代码是否按预期工作。让我们看一个简单的例子：

```x
test it_works {
  let result = 2 + 2
  assert_eq!(result, 4)
}
```

这是一个测试函数，用于验证 2 + 2 是否等于 4。让我们分解一下：

1. **`test`**：声明这是一个测试函数
2. **`it_works`**：测试的名称
3. **`assert_eq!`**：一个断言宏，用于检查两个值是否相等

如果我们运行这个测试，它会通过，因为 2 + 2 确实等于 4。

## 断言宏

X 语言提供了几个用于测试的断言宏：

### assert!

`assert!` 宏检查条件是否为 true：

```x
test is_true {
  assert!(true)
  assert!(2 + 2 == 4)
}
```

### assert_eq! 和 assert_ne!

`assert_eq!` 检查两个值是否相等，`assert_ne!` 检查它们是否不相等：

```x
test equality {
  assert_eq!(2 + 2, 4)
  assert_ne!(2 + 2, 5)
}
```

### 自定义消息

你可以向任何断言宏添加自定义消息：

```x
test with_message {
  let result = 2 + 2
  assert_eq!(result, 4, "加法出了问题：{} + {} != {}", 2, 2, result)
}
```

## 使用 panic! 测试

你可以测试代码在某些情况下是否 panic。使用 `should_panic` 属性：

```x
test this_panics should_panic {
  panic!("这个测试应该 panic")
}
```

你还可以指定 panic 消息：

```x
test panic_with_message should_panic("特定消息") {
  panic!("特定消息")
}
```

## 测试结果

让我们看看一些测试通过和失败的例子：

```x
test it_passes {
  assert_eq!(2 + 2, 4)
}

test it_fails {
  assert_eq!(2 + 2, 5)
}
```

当我们运行这些测试时，我们会看到：

```
running 2 tests
test it_passes ... ok
test it_fails ... FAILED

failures:

---- it_fails stdout ----
assertion failed: `(left == right)`
  left: `4`,
 right: `5`

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured
```

## 组织测试

测试通常放在与被测试代码相同的文件中，放在一个特殊的测试模块中：

```x
// 我们要测试的代码
function add(a: integer, b: integer) -> integer {
  a + b
}

function subtract(a: integer, b: integer) -> integer {
  a - b
}

// 测试
test add_works {
  assert_eq!(add(2, 3), 5)
}

test subtract_works {
  assert_eq!(subtract(5, 3), 2)
}
```

## 测试模块

对于较大的项目，你可能希望将测试组织成测试模块：

```x
// src/math.x
module math {
  export function add(a: integer, b: integer) -> integer {
    a + b
  }

  export function multiply(a: integer, b: integer) -> integer {
    a * b
  }
}

// tests/math_tests.x
import math::*

test math_add {
  assert_eq!(add(2, 3), 5)
}

test math_multiply {
  assert_eq!(multiply(2, 3), 6)
}
```

## 单元测试与集成测试

在 X 语言中，通常将测试分为两类：

1. **单元测试**：测试各个函数和模块的隔离
2. **集成测试**：测试多个模块协同工作

### 单元测试

单元测试放在与被测试代码相同的文件中，目的是测试代码的各个部分的隔离。它们可以测试私有接口。

### 集成测试

集成测试在你的库之外，它们以与任何其他代码相同的方式使用你的代码。它们只能测试公共接口，目的是测试库的多个部分是否协同工作。

## 测试私有函数

X 语言的测试哲学是，你应该主要通过公共接口测试代码，但如果你需要测试私有函数，你也可以这样做。由于测试只是另一个模块，它们可以访问同一模块中的私有项：

```x
function internal_helper(a: integer) -> integer {
  a * 2
}

export function public_function(a: integer) -> integer {
  internal_helper(a) + 1
}

test test_internal_helper {
  assert_eq!(internal_helper(5), 10)
}

test test_public_function {
  assert_eq!(public_function(5), 11)
}
```

## 运行测试

要运行测试，你使用 `x test` 命令：

```bash
x test my_file.x
```

你也可以运行特定测试：

```bash
x test my_file.x --test add_works
```

或者过滤测试：

```bash
x test my_file.x --filter add
```

## 忽略测试

你可以使用 `ignore` 属性忽略测试：

```x
test expensive_test ignore {
  // 运行时间长的测试
  for i in 0..1000000 {
    // 做一些事情
  }
}
```

如果你想运行被忽略的测试，你可以使用 `--ignored` 标志：

```bash
x test my_file.x --ignored
```

## 测试输出

默认情况下，X 语言只显示失败测试的输出。如果你想查看通过测试的输出，你可以使用 `--show-output` 标志：

```bash
x test my_file.x --show-output
```

## 总结

在 X 语言中编写测试：
- 使用 `test` 属性注释测试函数
- 使用 `assert!`、`assert_eq!` 和 `assert_ne!` 宏
- 使用 `should_panic` 测试 panic
- 可以测试公共和私有函数
- 组织为单元测试（同一文件）或集成测试（单独文件）
- 可以用 `ignore` 忽略
- 用 `x test` 运行

测试是编写可靠软件的重要组成部分，X 语言使其变得简单！

在下一章中，我们将讨论如何组织测试项目！
