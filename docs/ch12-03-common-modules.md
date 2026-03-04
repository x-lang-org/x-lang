# 常用模块

在前面的章节中，我们看到了标准库的概述和 prelude。在本章中，我们将更详细地查看一些最常用的标准库模块。

这些是你在 X 编程中可能经常使用的模块。虽然我们无法涵盖所有内容，但我们将提供足够的信息来帮助你入门。

## fmt：格式化和打印

`fmt` 模块处理格式化和打印值。我们之前已经见过 `print` 和 `println`，但这里有更多功能。

### 格式化占位符

格式化字符串可以包含各种占位符：

```x
// 默认格式
println("Hello, {}", "world")  // Hello, world

// 多个占位符
println("{} + {} = {}", 2, 3, 5)  // 2 + 3 = 5

// 命名占位符
println("{greeting}, {name}", greeting="Hello", name="Alice")

// 调试格式（用于调试）
println("{:?}", [1, 2, 3])  // [1, 2, 3]

// 数字格式
println("{:x}", 255)  // ff (hexadecimal)
println("{:b}", 5)    // 101 (binary)
println("{:o}", 9)    // 11 (octal)

// 宽度和对齐
println("{:>10}", "right")  //      right
println("{:<10}", "left")   // left
println("{:^10}", "center") //   center

// 精度
println("{:.2}", 3.14159)  // 3.14
```

### 自定义格式化

你可以通过实现 `Show` trait 为自己的类型自定义格式化：

```x
type Point = { x: integer, y: integer }

impl Show for Point {
  function show(self: &Self) -> String {
    String::format("({}, {})", self.x, self.y)
  }
}

let p = { x: 3, y: 5 }
println(p)  // (3, 5)
```

## io：输入/输出

`io` 模块处理输入和输出操作。

### 从标准输入读取

```x
// 读取一行
print("输入你的名字: ")
let name = io::stdin().read_line()?
println("你好, {}!", name)

// 读取整个输入
let input = io::stdin().read_to_string()?
```

### 写入标准输出/错误

```x
// 我们已经知道 print 和 println
print("这个")
println("那个")

// 写入标准错误
eprintln("这是一个错误消息")
```

### I/O 错误

I/O 操作返回 `Result`，因为它们可能失败：

```x
when io::stdin().read_line() is {
  Ok(line) => println("读取: {}", line),
  Err(e) => eprintln!("读取行失败: {}", e)
}
```

## fs：文件系统

`fs` 模块处理文件系统操作。

### 读取文件

```x
// 将整个文件读取为字符串
let contents = fs::read_to_string("hello.txt")?
println("文件内容:\n{}", contents)

// 读取为字节
let bytes = fs::read("data.bin")?
```

### 写入文件

```x
// 写入字符串
fs::write("output.txt", "Hello, file!")?

// 写入字节
let data = [0, 1, 2, 3]
fs::write("data.bin", data)?
```

### 文件元数据

```x
let metadata = fs::metadata("file.txt")?
println("大小: {} 字节", metadata.len())
println("是文件吗? {}", metadata.is_file())
println("是目录吗? {}", metadata.is_dir())
```

### 目录操作

```x
// 创建目录
fs::create_dir("new_dir")?

// 创建目录及其所有父目录
fs::create_dir_all("path/to/nested/dir")?

// 删除文件
fs::remove_file("unwanted.txt")?

// 删除空目录
fs::remove_dir("empty_dir")?

// 删除目录及其所有内容（小心使用！）
fs::remove_dir_all("dir_to_delete")?
```

## path：路径处理

`path` 模块处理文件系统路径。

```x
// 从字符串创建路径
let p = path::Path::new("/home/user/file.txt")

// 连接路径
let full_path = path::Path::new("/home/user") + "file.txt"

// 获取组件
println("父目录: {:?}", p.parent())
println("文件名: {:?}", p.file_name())
println("扩展名: {:?}", p.extension())

// 检查路径
println("存在吗? {}", p.exists())
println("是文件吗? {}", p.is_file())
println("是目录吗? {}", p.is_dir())
```

## time：时间和日期

`time` 模块处理时间和日期。

### 当前时间

```x
// 获取当前时间
let now = time::now()
println("当前时间: {:?}", now)

// 自 UNIX 纪元以来的持续时间
let duration = now.duration_since(time::UNIX_EPOCH)?
println("自 1970 年以来的秒数: {}", duration.as_seconds())
```

### 持续时间

```x
// 创建持续时间
let five_seconds = time::Duration::from_seconds(5)
let hundred_millis = time::Duration::from_millis(100)

// 持续时间运算
let total = five_seconds + hundred_millis
let difference = five_seconds - hundred_millis

// 睡眠一段时间
thread::sleep(five_seconds)
```

## thread：线程

`thread` 模块处理多线程。

### 生成线程

```x
// 生成新线程
let handle = thread::spawn(function() {
  for i in 1..10 {
    println("线程数字: {}", i)
    thread::sleep(time::Duration::from_millis(1))
  }
})

// 主线程中做一些事情
for i in 1..5 {
  println("主线程数字: {}", i)
  thread::sleep(time::Duration::from_millis(1))
}

// 等待线程完成
handle.join()?
```

### 线程本地存储

```x
// 每个线程都有自己的计数器副本
thread_local! {
  static COUNTER: integer = 0
}

thread::spawn(function() {
  COUNTER.with(|c| {
    println("线程中的计数器: {}", c)
  })
})
```

我们将在关于并发的章节中更详细地讨论线程。

## sync：同步

`sync` 模块提供同步原语。

### 互斥锁（Mutex）

```x
let counter = sync::Mutex::new(0)

// 在多线程中使用...
let guard = counter.lock()?
*guard = *guard + 1
// guard 被丢弃时自动解锁
```

### 原子引用计数（Arc）

```x
// Arc 是原子的、线程安全的引用计数指针
let data = sync::Arc::new([1, 2, 3])

// 跨线程共享...
let data_clone = data.clone()  // 增加引用计数
thread::spawn(function() {
  println("线程中的数据: {:?}", data_clone)
})
```

### 通道（Channel）

```x
// 创建通道
let (sender, receiver) = sync::mpsc::channel()

// 在线程中发送
thread::spawn(function() {
  sender.send("你好来自线程!")?
})

// 在主线程中接收
let message = receiver.recv()?
println("收到: {}", message)
```

同样，我们将在关于并发的章节中更详细地讨论这些内容。

## math：数学函数

`math` 模块提供数学函数和常量。

### 基本函数

```x
println("绝对值: {}", math::abs(-5))  // 5
println("符号: {}", math::signum(-3))  // -1
```

### 幂运算和根

```x
println("平方: {}", math::pow(2, 3))      // 8
println("平方根: {}", math::sqrt(16.0))  // 4.0
println("立方根: {}", math::cbrt(8.0))   // 2.0
```

### 三角函数

```x
let angle = math::PI / 4.0  // 45度
println("sin: {}", math::sin(angle))
println("cos: {}", math::cos(angle))
println("tan: {}", math::tan(angle))
```

### 指数和对数

```x
println("e^2: {}", math::exp(2.0))
println("ln(10): {}", math::ln(10.0))
println("log2(8): {}", math::log2(8.0))
println("log10(100): {}", math::log10(100.0))
```

### 舍入

```x
println("floor: {}", math::floor(3.7))   // 3.0
println("ceil: {}", math::ceil(3.2))     // 4.0
println("round: {}", math::round(3.5))   // 4.0
println("trunc: {}", math::trunc(3.9))   // 3.0
```

### 最小值和最大值

```x
println("min: {}", math::min(5, 3))   // 3
println("max: {}", math::max(5, 3))   // 5
```

### 常量

```x
println("π: {}", math::PI)
println("e: {}", math::E)
```

## convert：类型转换

`convert` 模块帮助在类型之间进行转换。

```x
// 字符串转数字
let num: integer = "42".parse()?
let float: Float = "3.14".parse()?

// 数字转字符串
let s1 = 42.to_string()
let s2 = 3.14.to_string()

// 整数转换
let i: integer = 5
let f: Float = i as Float

// 浮点数转换（小心！）
let f: Float = 3.9
let i: integer = f as integer  // 3（截断）
```

## rand：随机数

（如果 `rand` 是单独的模块或在 `std` 中）

```x
// 生成随机整数
let n = rand::random<integer>()
let n_in_range = rand::range(0, 10)  // 0 <= n < 10

// 生成随机浮点数
let f = rand::random<Float>()  // 0.0 <= f < 1.0

// 从列表中选择
let items = ["a", "b", "c"]
let chosen = rand::choice(items)
```

## 总结

常用标准库模块：
- **fmt** - 格式化和打印，有很多占位符选项
- **io** - 标准输入/输出
- **fs** - 文件系统操作
- **path** - 路径处理
- **time** - 时间和日期，持续时间
- **thread** - 多线程
- **sync** - 同步原语（Mutex、Arc、通道）
- **math** - 数学函数和常量
- **convert** - 类型转换
- **rand** - 随机数生成（如果可用）

这些模块涵盖了你在日常 X 编程中需要的大部分功能。与往常一样，官方文档是完整 API 的最佳资源！

现在我们已经涵盖了标准库，让我们继续讨论高级特性！
