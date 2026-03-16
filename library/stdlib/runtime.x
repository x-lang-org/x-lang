// X语言标准库 - 运行时原语
//
// 这些函数在编译时由各后端内联展开为原生 API 调用
// 零函数调用开销，性能最高

// ==========================================
// 文件系统原语
// ==========================================

/// 读取文件全部内容
/// 返回 Result<String, String>，成功时为文件内容，失败时为错误信息
external function __rt_file_read(path: String) -> Result<String, String>

/// 写入字符串到文件
/// 返回 Result<Unit, String>
external function __rt_file_write(path: String, content: String) -> Result<Unit, String>

/// 检查文件是否存在
external function __rt_file_exists(path: String) -> Bool

/// 删除文件
/// 返回 Result<Unit, String>
external function __rt_file_delete(path: String) -> Result<Unit, String>

/// 复制文件
external function __rt_file_copy(from: String, to: String) -> Result<Unit, String>

/// 移动/重命名文件
external function __rt_file_move(from: String, to: String) -> Result<Unit, String>

/// 获取文件大小（字节）
external function __rt_file_size(path: String) -> Option<Int>

// ==========================================
// 目录操作原语
// ==========================================

/// 创建目录
external function __rt_dir_create(path: String) -> Result<Unit, String>

/// 创建目录（包括父目录）
external function __rt_dir_create_all(path: String) -> Result<Unit, String>

/// 列出目录内容
external function __rt_dir_list(path: String) -> Result<[String], String>

/// 检查目录是否存在
external function __rt_dir_exists(path: String) -> Bool

/// 删除空目录
external function __rt_dir_delete(path: String) -> Result<Unit, String>

/// 删除目录及其内容
external function __rt_dir_delete_all(path: String) -> Result<Unit, String>

// ==========================================
// 控制台原语
// ==========================================

/// 打印到标准输出（不换行）
external function __rt_print(s: String) -> Unit

/// 打印到标准输出（换行）
external function __rt_println(s: String) -> Unit

/// 打印到标准错误（不换行）
external function __rt_eprint(s: String) -> Unit

/// 打印到标准错误（换行）
external function __rt_eprintln(s: String) -> Unit

/// 从标准输入读取一行
external function __rt_readline() -> String

// ==========================================
// 系统原语
// ==========================================

/// 获取环境变量
external function __rt_get_env(name: String) -> Option<String>

/// 设置环境变量
external function __rt_set_env(name: String, value: String) -> Bool

/// 获取命令行参数
external function __rt_args() -> [String]

/// 获取当前工作目录
external function __rt_cwd() -> Result<String, String>

/// 改变工作目录
external function __rt_chdir(path: String) -> Result<Unit, String>

/// 退出程序
external function __rt_exit(code: Int) -> Unit

/// 获取当前时间戳（毫秒）
external function __rt_timestamp_ms() -> Int

/// 获取当前时间戳（纳秒）
external function __rt_timestamp_ns() -> Int

/// 休眠指定毫秒数
external function __rt_sleep(ms: Int) -> Unit

/// 获取进程ID
external function __rt_getpid() -> Int

// ==========================================
// 数学原语
// ==========================================

/// 平方根
external function __rt_sqrt(x: Float) -> Float

/// 幂运算
external function __rt_pow(base: Float, exp: Float) -> Float

/// 正弦函数（弧度）
external function __rt_sin(x: Float) -> Float

/// 余弦函数（弧度）
external function __rt_cos(x: Float) -> Float

/// 正切函数（弧度）
external function __rt_tan(x: Float) -> Float

/// 反正弦函数（返回弧度）
external function __rt_asin(x: Float) -> Float

/// 反余弦函数（返回弧度）
external function __rt_acos(x: Float) -> Float

/// 反正切函数（返回弧度）
external function __rt_atan(x: Float) -> Float

/// 双参数反正切函数
external function __rt_atan2(y: Float, x: Float) -> Float

/// 自然对数
external function __rt_log(x: Float) -> Float

/// 以2为底的对数
external function __rt_log2(x: Float) -> Float

/// 以10为底的对数
external function __rt_log10(x: Float) -> Float

/// e的x次幂
external function __rt_exp(x: Float) -> Float

/// 向下取整
external function __rt_floor(x: Float) -> Float

/// 向上取整
external function __rt_ceil(x: Float) -> Float

/// 四舍五入
external function __rt_round(x: Float) -> Float

/// 截断小数部分
external function __rt_trunc(x: Float) -> Float

/// 整数绝对值
external function __rt_abs_int(x: Int) -> Int

/// 浮点数绝对值
external function __rt_abs_float(x: Float) -> Float

/// 整数最小值
external function __rt_min_int(a: Int, b: Int) -> Int

/// 整数最大值
external function __rt_max_int(a: Int, b: Int) -> Int

/// 浮点数最小值
external function __rt_min_float(a: Float, b: Float) -> Float

/// 浮点数最大值
external function __rt_max_float(a: Float, b: Float) -> Float

// ==========================================
// 字符串原语
// ==========================================

/// 字符串长度
external function __rt_str_len(s: String) -> Int

/// 字符串拼接
external function __rt_str_concat(a: String, b: String) -> String

/// 字符串相等比较
external function __rt_str_eq(a: String, b: String) -> Bool

/// 字符串小于比较
external function __rt_str_lt(a: String, b: String) -> Bool

/// 字符串包含
external function __rt_str_contains(s: String, substr: String) -> Bool

/// 字符串以指定前缀开始
external function __rt_str_starts_with(s: String, prefix: String) -> Bool

/// 字符串以指定后缀结束
external function __rt_str_ends_with(s: String, suffix: String) -> Bool

/// 子字符串
external function __rt_str_substring(s: String, start: Int, len: Int) -> String

/// 字符串分割
external function __rt_str_split(s: String, sep: String) -> [String]

/// 字符串替换
external function __rt_str_replace(s: String, from: String, to: String) -> String

/// 字符串转小写
external function __rt_str_to_lower(s: String) -> String

/// 字符串转大写
external function __rt_str_to_upper(s: String) -> String

/// 去除首尾空白
external function __rt_str_trim(s: String) -> String

/// 整数转字符串
external function __rt_int_to_str(n: Int) -> String

/// 浮点数转字符串
external function __rt_float_to_str(f: Float) -> String

/// 字符串转整数
external function __rt_str_to_int(s: String) -> Option<Int>

/// 字符串转浮点数
external function __rt_str_to_float(s: String) -> Option<Float>

// ==========================================
// 数组原语
// ==========================================

/// 数组长度
external function __rt_array_len(arr: [dynamic]) -> Int

/// 数组是否为空
external function __rt_array_is_empty(arr: [dynamic]) -> Bool

/// 数组连接
external function __rt_array_concat(a: [dynamic], b: [dynamic]) -> [dynamic]

/// 数组切片
external function __rt_array_slice(arr: [dynamic], start: Int, end: Int) -> [dynamic]

/// 数组包含元素
external function __rt_array_contains(arr: [dynamic], elem: dynamic) -> Bool

// ==========================================
// TCP 网络原语（底层系统能力）
// ==========================================

/// 创建 TCP 服务器并开始监听
/// 返回 Result<Int, String>，成功时为服务器句柄
external function __rt_tcp_listen(host: String, port: Int) -> Result<Int, String>

/// 接受 TCP 连接
/// 返回 Result<Int, String>，成功时为连接句柄
external function __rt_tcp_accept(handle: Int) -> Result<Int, String>

/// 连接到 TCP 服务器
/// 返回 Result<Int, String>，成功时为连接句柄
external function __rt_tcp_connect(host: String, port: Int) -> Result<Int, String>

/// 从 TCP 连接读取数据
/// 返回 Result<[Int], String>，成功时为字节数组
external function __rt_tcp_read(handle: Int, max_size: Int) -> Result<[Int], String>

/// 向 TCP 连接写入数据
/// 返回 Result<Int, String>，成功时为写入的字节数
external function __rt_tcp_write(handle: Int, data: [Int]) -> Result<Int, String>

/// 关闭 TCP 连接或服务器
external function __rt_tcp_close(handle: Int) -> Unit

/// 检查 TCP 连接是否有数据可读
external function __rt_tcp_readable(handle: Int) -> Bool

/// 设置 TCP 连接为非阻塞模式
external function __rt_tcp_set_nonblocking(handle: Int, flag: Bool) -> Bool

// ==========================================
// 异步 I/O 原语（底层系统能力）
// ==========================================

/// 异步接受 TCP 连接
/// 返回操作 ID，用于轮询结果
external function __rt_tcp_accept_async(handle: Int) -> Int

/// 异步读取 TCP 数据
/// 返回操作 ID，用于轮询结果
external function __rt_tcp_read_async(handle: Int, max_size: Int) -> Int

/// 异步写入 TCP 数据
/// 返回操作 ID，用于轮询结果
external function __rt_tcp_write_async(handle: Int, data: [Int]) -> Int

/// 检查异步操作是否完成
external function __rt_async_poll(op_id: Int) -> Bool

/// 获取异步操作的结果
external function __rt_async_result(op_id: Int) -> Result<dynamic, String>

/// 事件循环迭代
external function __rt_event_loop_tick() -> Bool
