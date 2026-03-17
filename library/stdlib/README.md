# X 语言精简标准库

X 语言采用**精简标准库**设计理念：只提供语言核心特性必须的类型和函数，其他功能由目标平台的标准库提供。

## 设计理念

1. **语言特性优先**：标准库只包含语言特性必须的类型（如 `Option`、`Result`）
2. **平台寄生**：I/O、网络、文件系统等功能直接使用目标平台的库
3. **零封装**：不重复造轮子，不封装目标平台已有的功能

## 核心类型

### Option<T>

表示可能存在或不存在的值，替代 null：

```x
enum Option<T> {
    Some(T),
    None
}
```

### Result<T, E>

表示可能成功或失败的操作结果：

```x
enum Result<T, E> {
    Ok(T),
    Err(E)
}
```

## Prelude

以下内容自动导入到每个 X 程序：

- `Option<T>`, `Some`, `None` - 可选值类型
- `Result<T, E>`, `Ok`, `Err` - 结果类型
- `print`, `println` - 基本输出
- `panic` - 不可恢复错误
- `assert`, `assert_eq`, `assert_ne` - 断言
- `todo`, `unimplemented`, `unreachable` - 标记函数

## 平台特定功能

X 语言不提供以下功能的标准库实现，而是直接使用目标平台的库：

| 功能 | Zig 后端 | JavaScript 后端 | JVM 后端 | .NET 后端 |
|------|----------|-----------------|----------|-----------|
| 文件系统 | `std.fs` | `fs` module | `java.nio.file` | `System.IO` |
| 网络 | `std.net` | `net` module | `java.net` | `System.Net` |
| 并发 | `std.Thread` | Worker / Promise | `java.util.concurrent` | `System.Threading` |
| 集合 | `std.ArrayList` 等 | Array / Map / Set | `java.util.*` | `System.Collections.*` |
| JSON | `std.json` | `JSON.parse/stringify` | Jackson/Gson | `System.Text.Json` |

## 使用示例

```x
// 使用标准库的 Option
function divide(a: integer, b: integer): Option<integer> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

// 使用标准库的 Result
function read_file(path: string): Result<string, string> {
    // 调用平台特定函数（如 Zig 的 std.fs.readFile）
    // ...
}

// 导入平台特定库（示例：JVM 后端）
import java.util.ArrayList

function main() {
    let list = ArrayList<string>()
    list.add("hello")
    println(list.get(0))
}
```

## 文件结构

```
library/stdlib/
├── README.md     # 本文档
├── prelude.x     # 自动导入的核心类型和函数
├── option.x      # Option 类型定义
└── result.x      # Result 类型定义
```
