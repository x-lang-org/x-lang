# X 语言标准库概览

X 语言的标准库采用**核心优先、逐步扩展**的思路：一方面保留对目标平台能力的直接利用，另一方面也在仓库中提供越来越多的通用模块实现。换句话说，X 不是“只有极小 prelude、其余全部依赖平台库”的状态了；当前仓库里的 `library/stdlib/` 已经包含核心类型、I/O、文件系统、字符串、集合、数学等多类模块，但整体仍在持续演进中。

## 设计理念

### 为什么强调“核心优先”？

1. **不重复造轮子**：目标平台（如 Zig、JVM、.NET、JavaScript）已有成熟标准库时，X 倾向于优先复用而不是重复发明
2. **零开销互操作**：直接调用平台 API 仍然是重要能力，而不是被标准库完全遮蔽
3. **仓库内提供公共基础模块**：对于跨项目反复使用的核心能力，`library/stdlib` 逐步沉淀成可直接导入的 X 语言模块
4. **保持演进空间**：当前标准库已经不算“极小”，但也还远未到所有能力都稳定封装完毕的阶段

### 标准库内容

当前仓库中的标准库至少包含以下几类内容：

| 模块/能力 | 用途 |
|-----------|------|
| `std.prelude` | 自动导入的基础函数与常用符号 |
| `std.types` | `Option<T>`、`Result<T, E>`、列表等核心类型 |
| `std.io` | 控制台 I/O、读取输入、flush 等 |
| `std.fs` | 文件与目录相关基础操作 |
| `std.string` | 字符串查询、变换、分割、拼接 |
| `std.collections` | 集合类型与常见容器 |
| `std.math` / `std.random` / `std.time` | 数学、随机、时间等通用模块 |
| `std.encoding` / `std.hash` / `std.process` / `std.unsafe` | 编码、哈希、进程与底层能力 |

其中一部分属于“语言运行所需的核心基础”，另一部分则是仓库当前已经提供的、更偏实用化的标准库模块。

## Prelude

Prelude 是自动导入到每个 X 程序中的模块：

```x
// 自动可用，无需导入
let x: Option<integer> = Some(42)
let y: Result<string, string> = Ok("hello")

println("Hello, World!")
panic("something went wrong")
```

## 与目标平台标准库的关系

即使仓库内已经存在更丰富的标准库源码，X 仍然保留“直接利用目标平台库”的能力。对于某些能力，当前实现也可能仍然更依赖目标平台，而不是完全通过 X 标准库统一抽象。

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

如果需要跨平台代码，应优先评估 `library/stdlib/` 里是否已经存在对应模块；若没有，再考虑平台特定导入或条件编译。

## 标准库结构

```
library/stdlib/
├── README.md          # 标准库说明
├── prelude.x          # 自动导入的核心函数与符号
├── types.x            # 核心类型
├── collections.x      # 集合类型
├── io.x / fs.x / net.x
├── math.x / random.x / time.x
└── encoding.x / hash.x / process.x / unsafe.x
```

> 这份目录结构描述的是仓库当前已经存在的源码范围，不代表所有模块都已经达到同样成熟度。

## 与其他语言的对比

| 语言 | 标准库大小 | 理念 |
|------|-----------|------|
| Python | 庞大 | "Batteries included" |
| Go | 中等 | 常用功能全覆盖 |
| Rust | 中等 | 核心功能 + 生态包 |
| **X** | **核心优先、持续扩展** | **核心能力内建 + 仓库 stdlib 模块 + 必要时直接利用平台库** |
| C | 极小 | 仅标准类型 |
