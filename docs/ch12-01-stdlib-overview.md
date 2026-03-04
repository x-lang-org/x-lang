# 标准库概览

X 语言的标准库（`stdlib`）提供了丰富的核心功能，你可以在 X 程序中使用。它包括常见的数据结构、有用的宏、I/O 和多线程、错误处理、字符串处理等。标准库是 X 编程的基础，因此熟悉它的功能很重要。

在本章中，我们将概述标准库中可用的内容，以便你了解在自己的项目中可以使用什么。我们不会深入探讨所有内容，但会提供一个起点，以便你在需要时可以了解更多。

## 标准库的组织

标准库组织成模块，每个模块包含相关的功能。主要模块包括：

- **prelude** - 自动导入到每个 X 程序中的内容
- **collections** - 常见的数据结构（List、Map、Set 等）
- **option** - Option 类型，表示可选值
- **result** - Result 类型，表示成功或失败
- **string** - 字符串处理
- **iter** - 迭代器
- **fmt** - 格式化和打印
- **io** - 输入和输出
- **fs** - 文件系统操作
- **path** - 路径处理
- **net** - 网络
- **time** - 时间和日期
- **thread** - 线程和并发
- **sync** - 同步原语
- **math** - 数学函数
- **convert** - 类型转换
- **default** - 默认值
- **cmp** - 比较
- **hash** - 哈希

让我们更详细地了解其中一些模块。

## Prelude：自动导入

X 语言的 prelude 是自动导入到每个 X 程序中的模块。它包含最常用的类型和函数，因此你不必手动导入它们。

Prelude 包括：
- 基本类型：`integer`、`Float`、`boolean`、`character`、`String`
- 容器类型：`List`、`Map`、`Set`、`Option`、`Result`
- 常用函数：`print`、`println`、`panic`
- 常用宏（如果有）
- 常用 trait：`Eq`、`Ord`、`Show`、`Clone`

由于 prelude 是自动导入的，你可以直接使用这些类型和函数，而无需显式导入它们。

## 集合

collections 模块提供了常见的数据结构：

- **List** - 可增长的数组（也写为 `[T]`）
- **Map** - 键值对（哈希表）
- **Set** - 唯一值的集合
- 其他可能的集合，如 `VecDeque`、`LinkedList`、`BinaryHeap` 等

我们在第 6 章中介绍了 List、Map 和 Set。这些是你在 X 编程中最常使用的集合。

## Option 和 Result

option 和 result 模块定义了 `Option` 和 `Result` 类型，我们在第 7 章中介绍过：

- **Option** - 表示可选值（`Some(T)` 或 `None`）
- **Result** - 表示操作的结果（`Ok(T)` 或 `Err(E)`）

这些类型是 X 语言错误处理哲学的核心，你会在大多数 X 程序中看到它们。

## 字符串

string 模块提供了 `String` 类型和处理字符串的函数。String 是一个可增长、可变、拥有的 UTF-8 编码字符串类型。

常见的字符串操作：
- 创建字符串：`String::from("hello")`
- 连接字符串：`s + " world"`
- 获取长度：`s.len()`
- 检查是否为空：`s.is_empty()`
- 子字符串：`s.substring(start, end)`
- 分割：`s.split(separator)`
- 修剪：`s.trim()`
- 大小写转换：`s.to_lowercase()`、`s.to_uppercase()`

## 迭代器

iter 模块提供了 `Iterator` trait 和许多迭代器适配器。我们在第 10 章中介绍了迭代器。

常见的迭代器操作：
- `map` - 转换元素
- `filter` - 筛选元素
- `fold` - 归约为单个值
- `collect` - 收集到集合中
- `take` - 取前 n 个元素
- `skip` - 跳过前 n 个元素
- `enumerate` - 获取索引和值
- `any` - 检查是否有任何元素满足条件
- `all` - 检查是否所有元素都满足条件

## 格式化和打印

fmt 模块提供了格式化和打印值的功能。这包括 `print` 和 `println` 函数。

格式化字符串可以包含占位符：
- `{}` - 默认格式
- `{:?}` - 调试格式
- `{:x}` - 十六进制
- `{:b}` - 二进制
- `{:o}` - 八进制

## I/O

io 模块提供了输入和输出功能。这包括从 stdin 读取、写入 stdout 和 stderr 以及处理 I/O 错误。

常见的 I/O 操作：
- 从标准输入读取：`io::stdin().read_line()`
- 写入标准输出：`print!`、`println!`
- 写入标准错误：`eprint!`、`eprintln!`

## 文件系统

fs 模块提供了文件系统操作，如读取和写入文件、创建目录等。

常见的文件系统操作：
- 读取文件：`fs::read_to_string()`、`fs::read()`
- 写入文件：`fs::write()`
- 创建目录：`fs::create_dir()`、`fs::create_dir_all()`
- 删除文件：`fs::remove_file()`
- 元数据：`fs::metadata()`

## 时间

time 模块提供了处理时间和日期的功能。

常见的时间操作：
- 获取当前时间：`time::now()`
- 时间戳：`time::UNIX_EPOCH`
- 持续时间：`Duration`
- 睡眠：`thread::sleep()`

## 线程和并发

thread 和 sync 模块提供了多线程和同步原语。

常见的并发原语：
- 生成线程：`thread::spawn()`
- 通道：`sync::mpsc::channel()`
- 互斥锁：`sync::Mutex`
- 读写锁：`sync::RwLock`
- 原子引用计数：`sync::Arc`

我们将在关于并发的章节中更详细地讨论线程和并发。

## 数学

math 模块提供了数学函数和常量。

常见的数学函数：
- 基本运算：`abs`、`signum`
- 幂运算：`pow`、`sqrt`、`cbrt`
- 三角函数：`sin`、`cos`、`tan`、`asin`、`acos`、`atan`
- 指数和对数：`exp`、`ln`、`log2`、`log10`
- 舍入：`floor`、`ceil`、`round`、`trunc`
- 最小值/最大值：`min`、`max`
- 常量：`PI`、`E`

## 如何了解更多

标准库比我们在这里涵盖的要大得多。要了解更多信息：

1. **查看标准库文档** - 官方文档是最好的资源
2. **阅读源代码** - 标准库是用 X 编写的，所以你可以看到它是如何工作的
3. **使用它！** - 学习任何库的最好方法是在自己的项目中使用它

## 总结

X 语言的标准库：
- 组织成包含相关功能的模块
- 包括 prelude，它是自动导入的
- 提供常见的数据结构（集合）
- 提供 Option 和 Result 用于错误处理
- 包括字符串处理、迭代器、I/O、文件系统、时间、线程等
- 有很好的文档记录和广泛的测试

标准库是 X 编程的基础——熟悉它会让你成为更高效的 X 程序员！

在下一章中，我们将更详细地查看 prelude！
