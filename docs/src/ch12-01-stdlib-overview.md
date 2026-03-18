# X 语言精简标准库概览

X 语言采用**精简标准库**设计理念：只提供语言核心特性必须的类型和函数，其他功能由目标平台的标准库提供。

## 设计理念

### 为什么精简？

1. **不重复造轮子**：目标平台（Zig、JVM、.NET、JavaScript）已有成熟、经过充分测试的标准库
2. **零开销互操作**：直接调用平台 API，无需封装层
3. **平台最优**：每个后端都能充分利用平台特性
4. **简化编译器**：无需维护庞大的标准库代码

### 标准库内容

X 语言标准库只包含以下内容：

| 类型/函数 | 用途 |
|-----------|------|
| `Option<T>` | 替代 null，表示可能存在或不存在的值 |
| `Result<T, E>` | 替代异常，表示可能成功或失败的操作 |
| `print` / `println` | 基本输出 |
| `panic` | 不可恢复错误 |
| `assert` 系列 | 断言 |
| `todo` / `unimplemented` / `unreachable` | 标记函数 |

这些是语言特性的核心部分，必须由标准库提供。

## Prelude

Prelude 是自动导入到每个 X 程序中的模块：

```x
// 自动可用，无需导入
let x: Option<integer> = Some(42)
let y: Result<string, string> = Ok("hello")

println("Hello, World!")
panic("something went wrong")
```

## 平台寄生

以下功能不在标准库中，直接使用目标平台的库：

### 文件系统

```x
// Zig 后端
import std.fs

// JavaScript 后端
import fs

// JVM 后端
import java.nio.file.Files

// .NET 后端
import System.IO.File
```

### 网络

```x
// Zig 后端
import std.net

// JavaScript 后端
import net

// JVM 后端
import java.net.ServerSocket

// .NET 后端
import System.Net.Sockets
```

### 集合

```x
// Zig 后端
import std.ArrayList

// JavaScript 后端
// 直接使用 Array, Map, Set

// JVM 后端
import java.util.ArrayList

// .NET 后端
import System.Collections.Generic.List
```

## 平台特定代码

如果 X 代码导入了特定平台的库，则该代码只能编译运行于对应平台：

```x
// 这段代码只能在 JVM 后端编译运行
import java.util.ArrayList

function main() {
    let list = ArrayList<string>()
    list.add("hello")
    println(list.get(0))
}
```

如果需要跨平台代码，应避免使用平台特定导入，或者使用条件编译。

## 标准库结构

```
library/stdlib/
├── README.md     # 标准库说明
├── prelude.x     # 自动导入的核心类型和函数
├── option.x      # Option 类型定义
└── result.x      # Result 类型定义
```

## 与其他语言的对比

| 语言 | 标准库大小 | 理念 |
|------|-----------|------|
| Python | 庞大 | "Batteries included" |
| Go | 中等 | 常用功能全覆盖 |
| Rust | 中等 | 核心功能 + 生态包 |
| **X** | **精简** | **语言特性必须 + 平台寄生** |
| C | 极小 | 仅标准类型 |
