# 测试组织

随着项目的发展，测试套件也会增长，你可能需要组织测试，以便更容易导航和运行。在本章中，我们将讨论如何组织测试，以及如何运行测试的某些子集。

## 测试目录结构

对于较大的项目，推荐的目录结构如下：

```
my_project/
├── src/
│   ├── main.x
│   ├── lib.x
│   ├── math.x
│   └── utils/
│       ├── string_utils.x
│       └── num_utils.x
└── tests/
    ├── integration_tests.x
    ├── math_tests.x
    └── utils/
        ├── string_tests.x
        └── num_tests.x
```

- **src/**：包含源代码
- **tests/**：包含集成测试
- 单元测试与它们测试的代码放在同一个文件中

## 单元测试

单元测试放在与它们测试的代码相同的文件中。它们旨在测试代码的各个部分的隔离。

```x
// src/math.x
function add(a: integer, b: integer) -> integer {
  a + b
}

function subtract(a: integer, b: integer) -> integer {
  a - b
}

export function multiply(a: integer, b: integer) -> integer {
  a * b
}

// 单元测试
test add {
  assert_eq!(add(2, 3), 5)
}

test subtract {
  assert_eq!(subtract(5, 3), 2)
}

test multiply {
  assert_eq!(multiply(2, 3), 6)
}
```

## 集成测试

集成测试放在 `tests` 目录中，它们像任何其他外部代码一样使用你的代码。它们只能测试公共接口。

```x
// tests/math_tests.x
import math::*

test multiply_integration {
  assert_eq!(multiply(2, 3), 6)
  assert_eq!(multiply(0, 5), 0)
  assert_eq!(multiply(-2, 3), -6)
}

// 集成测试还可以测试多个函数协同工作
test multiple_operations {
  let result = multiply(add(2, 3), subtract(10, 5))
  assert_eq!(result, 25)
}
```

注意：集成测试只能访问公共（导出）函数。

## 测试模块

对于相关测试组，你可以使用测试模块来组织它们：

```x
// tests/math_tests.x
module math_tests {
  import math::*

  test basic_multiplication {
    assert_eq!(multiply(2, 3), 6)
  }

  test zero_multiplication {
    assert_eq!(multiply(0, 5), 0)
    assert_eq!(multiply(5, 0), 0)
  }

  test negative_multiplication {
    assert_eq!(multiply(-2, 3), -6)
    assert_eq!(multiply(2, -3), -6)
    assert_eq!(multiply(-2, -3), 6)
  }
}
```

## 测试辅助函数

你可以创建辅助函数来帮助测试：

```x
// tests/helpers.x
export function create_test_list() -> List<integer> {
  [1, 2, 3, 4, 5]
}

export function assert_list_eq<T: Eq>(a: List<T>, b: List<T>) {
  assert_eq!(a.len(), b.len())
  for (i, element) in a.iter().enumerate() {
    assert_eq!(element, b[i])
  }
}

// tests/list_tests.x
import helpers::*
import list_utils::*

test reverse_list {
  let original = create_test_list()
  let reversed = reverse(original)
  let expected = [5, 4, 3, 2, 1]
  assert_list_eq(reversed, expected)
}
```

## 按名称过滤测试

你可以通过将测试名称的一部分传递给 `x test` 来运行测试的子集：

```bash
# 运行所有名称中包含 "multiply" 的测试
x test --filter multiply

# 运行所有名称中包含 "negative" 的测试
x test --filter negative
```

## 测试模块

如果你将测试组织成模块，你可以运行特定模块中的所有测试：

```bash
# 运行 math_tests 模块中的所有测试
x test --module math_tests
```

## 忽略测试

如前一章所述，你可以使用 `ignore` 属性忽略测试：

```x
test slow_test ignore {
  // 运行时间很长的测试
}
```

要运行被忽略的测试：

```bash
x test --ignored
```

## 并行运行测试

默认情况下，X 语言并行运行测试以加快速度。如果你需要按顺序运行测试（例如，如果它们共享资源），你可以使用 `--test-threads=1` 标志：

```bash
x test --test-threads=1
```

## 显示输出

默认情况下，X 语言只显示失败测试的输出。要查看所有测试的输出：

```bash
x test --show-output
```

## 测试设置和拆卸

有时你需要在测试前设置一些东西，然后在测试后清理。虽然 X 语言没有内置的 `setup` 和 `teardown` 函数，但你可以使用辅助函数：

```x
function setup_database() -> DatabaseConnection {
  let conn = Database::connect(":memory:")
  conn.execute("CREATE TABLE users (id INTEGER, name TEXT)")
  conn
}

function teardown_database(conn: DatabaseConnection) {
  conn.close()
}

test database_insert {
  let conn = setup_database()
  conn.execute("INSERT INTO users VALUES (1, 'Alice')")
  let result = conn.query("SELECT name FROM users WHERE id = 1")
  assert_eq!(result, "Alice")
  teardown_database(conn)
}
```

## 属性测试

对于更高级的测试，你可以使用属性测试（也称为基于属性的测试），它生成随机输入来测试你的代码：

```x
// 假设的属性测试库
property addition_commutative forall a: integer, b: integer {
  assert_eq!(add(a, b), add(b, a))
}

property addition_associative forall a: integer, b: integer, c: integer {
  assert_eq!(add(add(a, b), c), add(a, add(b, c)))
}
```

## 基准测试

除了测试正确性之外，你可能还想测试性能。基准测试测量代码运行的速度：

```x
benchmark vector_sort {
  let mutable v = [5, 3, 1, 4, 2]
  v.sort()
}

benchmark matrix_multiplication(size: integer = 100) {
  let a = create_matrix(size, size)
  let b = create_matrix(size, size)
  multiply_matrices(a, b)
}
```

## 总结

组织 X 语言测试：
- 单元测试与被测试的代码放在同一个文件中
- 集成测试放在 `tests/` 目录中
- 使用模块组织相关测试
- 创建辅助函数进行设置/拆卸和共享测试代码
- 使用 `--filter` 按名称运行测试的子集
- 使用 `--ignore` 忽略测试，使用 `--ignored` 运行被忽略的测试
- 使用 `--test-threads` 控制并行性
- 使用 `--show-output` 查看所有测试的输出
- 对于更高级的测试，考虑属性测试和基准测试

良好组织的测试使项目更容易维护和自信地更改！

现在我们已经介绍了测试，让我们继续讨论标准库！

