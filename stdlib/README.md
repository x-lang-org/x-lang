# X 语言标准库

这是用 X 语言本身实现的标准库。

## 模块结构

```
stdlib/
├── stdlib.x       # 主入口
├── prelude.x      # 核心函数（自动导入）
├── option.x       # Option 类型
├── result.x       # Result 类型
├── math.x         # 数学函数
├── string.x       # 字符串操作
├── collections.x  # 集合类型（List, Map, Set）
├── iter.x         # 迭代器
├── io.x           # 输入输出
├── time.x         # 时间处理
└── sys.x          # 系统功能
```

## 使用示例

### Option 类型

```x
// 创建 Option
let some_value = Some(42)
let no_value = None()

// 检查
is_some(some_value)  // true
is_none(no_value)    // true

// 解包
unwrap(some_value)              // 42
unwrap_or(no_value, 0)          // 0
unwrap_or_else(no_value, () -> 0)  // 0

// 变换
map(some_value, (x) -> x * 2)   // Some(84)
filter(some_value, (x) -> x > 10)  // Some(42)
```

### Result 类型

```x
// 创建 Result
let ok_value = Ok(42)
let err_value = Err("something wrong")

// 检查
is_ok(ok_value)     // true
is_err(err_value)   // true

// 解包
unwrap(ok_value)                 // 42
unwrap_or(err_value, 0)          // 0
expect(err_value, "failed")      // panic

// 变换
map(ok_value, (x) -> x * 2)     // Ok(84)
map_err(err_value, (e) -> "ERR: " + e)  // Err("ERR: ...")
and_then(ok_value, (x) -> Ok(x * 2))  // Ok(84)
```

### 数学函数

```x
// 常量
pi      // 3.14159...
e       // 2.71828...

// 基本运算
abs(-5)         // 5
sqrt(16)        // 4
pow(2, 3)       // 8

// 三角函数
sin(pi / 2)     // 1
cos(0)          // 1
tan(pi / 4)     // 1

// 取整
floor(3.7)      // 3
ceil(3.2)       // 4
round(3.5)      // 4

// 极值
min(5, 10)      // 5
max(5, 10)      // 10
clamp(5, 0, 10) // 5

// 随机数
rand()          // 0.0 ~ 1.0
rand_int(0, 10) // 0 ~ 9
```

### 字符串操作

```x
// 基本属性
str_len("hello")        // 5
str_is_empty("")        // true

// 拼接
str_concat("a", "b")    // "ab"
str_join(["a", "b"], ",")  // "a,b"
str_repeat("x", 3)      // "xxx"

// 包含检查
str_contains("hello", "ell")  // true
str_starts_with("hello", "he") // true
str_ends_with("hello", "lo")   // true

// 提取
str_substring("hello", 1, 4)  // "ell"
str_take("hello", 3)           // "hel"
str_drop("hello", 2)           // "llo"

// 替换
str_replace("hello", "l", "x")  // "hexxo"

// 大小写
str_to_lowercase("HELLO")  // "hello"
str_to_uppercase("hello")  // "HELLO"

// 修剪
str_trim("  hello  ")  // "hello"

// 分割
str_split("a,b,c", ",")  // ["a", "b", "c"]

// 解析
str_parse_int("42")    // Some(42)
str_parse_float("3.14") // Some(3.14)
```

### 列表操作

```x
// 创建
let list = list_new()
let list = list_of(42)
let list = list_repeat(0, 5)  // [0, 0, 0, 0, 0]
let list = list_range(0, 5)    // [0, 1, 2, 3, 4]

// 访问
list_len([1, 2, 3])      // 3
list_is_empty([])         // true
list_get([1, 2, 3], 1)   // Some(2)
list_first([1, 2, 3])    // Some(1)
list_last([1, 2, 3])     // Some(3)

// 修改
list_push([1, 2], 3)         // [1, 2, 3]
list_insert([1, 3], 1, 2)    // [1, 2, 3]
list_remove([1, 2, 3], 1)    // (Some(2), [1, 3])

// 变换
list_map([1, 2, 3], (x) -> x * 2)      // [2, 4, 6]
list_filter([1, 2, 3, 4], (x) -> x % 2 == 0)  // [2, 4]
list_fold([1, 2, 3], 0, (acc, x) -> acc + x)   // 6

// 搜索
list_contains([1, 2, 3], 2)  // true
list_find([1, 2, 3], (x) -> x > 1)  // Some(2)
list_all([1, 2, 3], (x) -> x > 0)  // true
list_any([1, 2, 3], (x) -> x > 2)  // true

// 排序
list_reverse([1, 2, 3])  // [3, 2, 1]
list_sort_int([3, 1, 2])  // [1, 2, 3]

// 数值
list_sum([1, 2, 3])       // 6
list_min_int([3, 1, 2])   // Some(1)
list_max_int([3, 1, 2])   // Some(3)
```

### 映射操作

```x
// 创建
let map = map_new()

// 插入和访问
map = map_insert(map, "a", 1)
map_get(map, "a")  // Some(1)

// 检查
map_contains_key(map, "a")  // true
map_is_empty(map)            // false
map_len(map)                  // 1

// 移除
let (value, new_map) = map_remove(map, "a")

// 键和值
map_keys(map)    // ["a"]
map_values(map)  // [1]
map_entries(map) // [("a", 1)]
```

### 输入输出

```x
// 打印
print("Hello")
println("Hello, World!")

// 格式化
format("Hello, {0}!", "World")  // "Hello, World!"

// 读取输入
let name = input()
let name = input_prompt("Name: ")

// 文件操作
match read_file("file.txt") is
  Ok { value: content } -> println(content)
  Err { error: e } -> eprintln("Error: " + e)

write_file("file.txt", "Hello, World!")

// 路径
path_join(["dir", "file.txt"])  // "dir/file.txt"
path_dirname("dir/file.txt")    // "dir"
path_basename("dir/file.txt")   // "file.txt"
path_extension("file.txt")      // Some("txt")
```

### 时间处理

```x
// 当前时间
let now_sec = timestamp()       // Unix 时间戳（秒）
let now_ms = timestamp_millis() // 毫秒
let now = now()                  // Time 类型

// 睡眠
sleep(1.0)        // 睡眠 1 秒
sleep_ms(1000)    // 睡眠 1000 毫秒

// 持续时间
let d = duration_seconds(5)
let d = duration_millis(500)
duration_as_seconds(d)  // 0.5

// 日历时间
let dt = local_now()
dt.year    // 年
dt.month   // 月 (1-12)
dt.day     // 日 (1-31)
dt.hour    // 时 (0-23)
dt.minute  // 分 (0-59)
dt.second  // 秒 (0-59)

// 格式化
format_iso8601(dt)  // "2024-01-15T10:30:00"

// 性能测量
let (result, time) = time_it(() -> {
  // 要测量的代码
})
time_it_print("label", () -> {
  // 要测量的代码
})
```

### 系统功能

```x
// 退出程序
exit(0)       // 正常退出
exit_success() // 同上
exit_failure() // 异常退出

// 环境变量
env_var("HOME")               // Some("/home/user")
env_var_or("HOME", "/tmp")   // "/home/user"
set_env_var("KEY", "value")

// 命令行参数
args()        // ["./program", "arg1", "arg2"]
args_rest()   // ["arg1", "arg2"]

// 系统信息
os_name()     // "Windows", "Linux", "Darwin"
arch()        // "x86_64", "aarch64"
hostname()    // 主机名
username()    // 用户名
cpu_count()   // CPU 核心数

// 平台检查
is_windows()  // true/false
is_linux()    // true/false
is_macos()    // true/false

// 日志
set_log_level(LOG_LEVEL_DEBUG)
log_debug("debug message")
log_info("info message")
log_warn("warning message")
log_error("error message")

// 错误处理
panic("something wrong")
todo()  // 未实现
unreachable()  // 不可达代码
```

## 许可证

MIT License
