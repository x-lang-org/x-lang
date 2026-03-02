// X语言标准库 - 系统功能
//
// 系统信息、进程、环境等

// ==========================================
// 进程退出
// ==========================================

/// 正常退出
fun exit_success() {
  exit(0)
}

/// 失败退出
fun exit_failure() {
  exit(1)
}

/// 退出并打印错误信息
fun exit_with_error(message: String, code: Int = 1) {
  eprintln(message)
  exit(code)
}

// ==========================================
// 环境变量
// ==========================================

/// 获取环境变量，带默认值
fun env_var_or(name: String, default: String): String {
  match env_var(name) is
    Some { value: v } -> v
    None -> default
}

/// 检查环境变量是否存在
fun has_env_var(name: String): Bool {
  is_some(env_var(name))
}

/// 设置环境变量（如不存在）
fun set_env_var_if_missing(name: String, value: String): Result<Unit, String> {
  if not has_env_var(name) {
    set_env_var(name, value)
  } else {
    Ok(())
  }
}

// ==========================================
// 操作系统信息
// ==========================================

/// 获取操作系统名称
fun os_name(): String {
  // 内置函数
  "__builtin_os_name"
}

/// 获取操作系统版本
fun os_version(): String {
  // 内置函数
  "__builtin_os_version"
}

/// 获取架构
fn arch(): String {
  // 内置函数
  "__builtin_arch"
}

/// 获取主机名
fun hostname(): String {
  // 内置函数
  "__builtin_hostname"
}

/// 获取用户名
fn username(): String {
  // 内置函数
  "__builtin_username"
}

/// 获取用户主目录
fun home_dir(): Option<String> {
  // 内置函数
  "__builtin_home_dir"
}

/// 获取临时目录
fun temp_dir(): String {
  // 内置函数
  "__builtin_temp_dir"
}

/// 获取当前工作目录
fn cwd(): Option<String> {
  match current_dir() is
    Ok { value: dir } -> Some(dir)
    Err { error: _ } -> None()
}

// ==========================================
// 平台检查
// ==========================================

/// 检查是否是 Windows
fun is_windows(): Bool {
  let os = str_to_lowercase(os_name())
  str_contains(os, "windows") || str_contains(os, "win32")
}

/// 检查是否是 Linux
fun is_linux(): Bool {
  str_contains(str_to_lowercase(os_name()), "linux")
}

/// 检查是否是 macOS
fun is_macos(): Bool {
  let os = str_to_lowercase(os_name())
  str_contains(os, "mac") || str_contains(os, "darwin")
}

/// 检查是否是 Unix-like 系统
fun is_unix(): Bool {
  is_linux() || is_macos()
}

// ==========================================
// CPU 信息
// ==========================================

/// 获取 CPU 核心数
fn cpu_count(): Int {
  // 内置函数
  "__builtin_cpu_count"
}

// ==========================================
// 内存信息
// ==========================================

/// 获取总内存（字节）
fun total_memory(): Int {
  // 内置函数
  "__builtin_total_memory"
}

/// 获取可用内存（字节）
fun available_memory(): Int {
  // 内置函数
  "__builtin_available_memory"
}

/// 获取已用内存（字节）
fun used_memory(): Int {
  total_memory() - available_memory()
}

// ==========================================
// 命令行参数
// ==========================================

/// 获取命令行参数（不包括程序名）
fn args_rest(): [String] {
  let argv = args()
  if list_len(argv) > 1 {
    list_drop(argv, 1)
  } else {
    []
  }
}

/// 检查是否有指定的参数
fn has_arg(name: String): Bool {
  list_contains(args(), name)
}

/// 获取参数的值（如 --key=value）
fun arg_value(key: String): Option<String> {
  let prefix = key + "="
  let argv = args()
  let mut i = 0
  while i < list_len(argv) {
    let arg = argv[i]
    if str_starts_with(arg, prefix) {
      return Some(str_slice(arg, str_len(prefix)))
    }
    i = i + 1
  }
  None()
}

/// 获取参数的值，带默认值
fun arg_value_or(key: String, default: String): String {
  match arg_value(key) is
    Some { value: v } -> v
    None -> default
}

// ==========================================
// 路径查找
// ==========================================

/// 获取 PATH 环境变量中的目录
fn path_dirs(): [String] {
  match env_var("PATH") is
    Some { value: path } ->
      let separator = if is_windows() { ";" } else { ":" }
      str_split(path, separator)
    None -> []
}

/// 在 PATH 中查找可执行文件
fun which(executable: String): Option<String> {
  let dirs = path_dirs()
  let extensions = if is_windows() {
    [".exe", ".cmd", ".bat", ""]
  } else {
    [""]
  }

  let mut i = 0
  while i < list_len(dirs) {
    let dir = dirs[i]
    let mut j = 0
    while j < list_len(extensions) {
      let ext = extensions[j]
      let full_path = path_join([dir, executable + ext])
      if file_exists(full_path) {
        return Some(full_path)
      }
      j = j + 1
    }
    i = i + 1
  }
  None()
}

/// 检查命令是否存在
fun command_exists(command: String): Bool {
  is_some(which(command))
}

// ==========================================
// 随机数（系统级）
// ==========================================

/// 获取系统随机字节
fun random_bytes(count: Int): [Int] {
  // 内置函数
  "__builtin_random_bytes"
}

/// 生成随机整数 [0, max)
fun random_int(max: Int): Int {
  let bytes = random_bytes(4)
  let mut result = 0
  let mut i = 0
  while i < list_len(bytes) {
    result = (result * 256 + bytes[i]) % max
    i = i + 1
  }
  result
}

/// 生成随机整数 [min, max)
fun random_range(min: Int, max: Int): Int {
  if min >= max {
    panic("random_range: min 必须小于 max")
  }
  min + random_int(max - min)
}

// ==========================================
// 日志级别
// ==========================================

type LogLevel = Int
let LOG_LEVEL_DEBUG: LogLevel = 0
let LOG_LEVEL_INFO: LogLevel = 1
let LOG_LEVEL_WARN: LogLevel = 2
let LOG_LEVEL_ERROR: LogLevel = 3

let mut current_log_level: LogLevel = LOG_LEVEL_INFO

/// 设置日志级别
fun set_log_level(level: LogLevel) {
  current_log_level = level
}

/// 获取日志级别
fun get_log_level(): LogLevel {
  current_log_level
}

/// 调试日志
fun log_debug(message: String) {
  if current_log_level <= LOG_LEVEL_DEBUG {
    println("[DEBUG] " + message)
  }
}

/// 信息日志
fun log_info(message: String) {
  if current_log_level <= LOG_LEVEL_INFO {
    println("[INFO] " + message)
  }
}

/// 警告日志
fun log_warn(message: String) {
  if current_log_level <= LOG_LEVEL_WARN {
    eprintln("[WARN] " + message)
  }
}

/// 错误日志
fun log_error(message: String) {
  if current_log_level <= LOG_LEVEL_ERROR {
    eprintln("[ERROR] " + message)
  }
}

// ==========================================
//  panic 和断言
// ==========================================

/// 终止程序并显示错误信息
fun panic(message: String) {
  eprintln("PANIC: " + message)
  // 这里可以添加堆栈跟踪
  exit_failure()
}

/// 终止程序并显示格式化错误信息
fun panic_fmt(template: String, ...args) {
  panic(format(template, ...args))
}

///  unreachable! 宏的函数版本
fun unreachable(message: String = "unreachable code reached") {
  panic(message)
}

/// todo! 宏的函数版本
fun todo(message: String = "not implemented") {
  panic("TODO: " + message)
}

// ==========================================
// 系统信息摘要
// ==========================================

/// 获取系统信息摘要
fun system_info(): {String: String} {
  let mut info = map_new()
  info = map_insert(info, "os", os_name())
  info = map_insert(info, "os_version", os_version())
  info = map_insert(info, "arch", arch())
  info = map_insert(info, "hostname", hostname())
  info = map_insert(info, "username", username())
  info = map_insert(info, "cpu_cores", to_string(cpu_count()))
  info
}

/// 打印系统信息
fn print_system_info() {
  let info = system_info()
  println("系统信息:")
  println("  OS: " + map_get(info, "os") + " " + map_get(info, "os_version"))
  println("  架构: " + map_get(info, "arch"))
  println("  主机名: " + map_get(info, "hostname"))
  println("  用户名: " + map_get(info, "username"))
  println("  CPU 核心数: " + map_get(info, "cpu_cores"))
}
