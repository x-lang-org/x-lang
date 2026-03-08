// X语言标准库 - 系统功能
//
// 系统相关操作，如环境变量、命令行参数、进程操作等

// ==========================================
// 环境变量
// ==========================================

/// 获取环境变量
function get_env(name: String): Option<String> {
  // 由C后端实现
}

/// 设置环境变量
function set_env(name: String, value: String): Bool {
  // 由C后端实现
}

/// 删除环境变量
function unset_env(name: String): Bool {
  // 由C后端实现
}

/// 获取所有环境变量
function env_vars(): List<(String, String)> {
  // 由C后端实现
}

// ==========================================
// 命令行参数
// ==========================================

/// 获取命令行参数
function args(): List<String> {
  // 由C后端实现
}

/// 获取命令行参数数量
function arg_count(): Int {
  let args = args()
  len(args)
}

/// 获取指定索引的命令行参数
function arg(index: Int): Option<String> {
  let args = args()
  if index >= 0 && index < len(args) {
    Some(args[index])
  } else {
    None
  }
}

// ==========================================
// 进程操作
// ==========================================

/// 获取当前进程ID
function getpid(): Int {
  // 由C后端实现
}

/// 获取父进程ID
function getppid(): Int {
  // 由C后端实现
}

/// 终止当前进程
function exit(code: Int) {
  // 由C后端实现
}

/// 执行系统命令
function system(command: String): Int {
  // 由C后端实现
}

/// 执行命令并获取输出
function command_output(command: String): Result<String, String> {
  // 由C后端实现
}

// ==========================================
// 系统信息
// ==========================================

/// 获取操作系统类型
function os_type(): String {
  // 由C后端实现
}

/// 获取操作系统版本
function os_version(): String {
  // 由C后端实现
}

/// 获取主机名
function hostname(): String {
  // 由C后端实现
}

/// 获取系统架构
function arch(): String {
  // 由C后端实现
}

/// 获取可用内存（字节）
function free_memory(): Int {
  // 由C后端实现
}

/// 获取总内存（字节）
function total_memory(): Int {
  // 由C后端实现
}

/// 获取CPU核心数
function cpu_count(): Int {
  // 由C后端实现
}

// ==========================================
// 路径操作
// ==========================================

/// 获取当前工作目录
function current_dir(): String {
  // 由C后端实现
}

/// 改变工作目录
function chdir(path: String): Bool {
  // 由C后端实现
}

/// 拼接路径
function path_join(paths: List<String>): String {
  if len(paths) == 0 {
    ""
  } else if len(paths) == 1 {
    paths[0]
  } else {
    let sep = if os_type() == "Windows" { "\\" } else { "/" }
    let mut result = paths[0]
    for i in 1..len(paths) {
      let path = paths[i]
      if result != "" && !str_ends_with(result, sep) {
        result = result + sep
      }
      if str_starts_with(path, sep) {
        result = result + str_substring(path, 1)
      } else {
        result = result + path
      }
    }
    result
  }
}

/// 获取路径的目录部分
function path_dirname(path: String): String {
  // 由C后端实现
}

/// 获取路径的文件名部分
function path_basename(path: String): String {
  // 由C后端实现
}

/// 获取路径的扩展名
function path_extension(path: String): String {
  // 由C后端实现
}

/// 检查路径是否存在
function path_exists(path: String): Bool {
  // 由C后端实现
}

/// 检查路径是否为文件
function is_file(path: String): Bool {
  // 由C后端实现
}

/// 检查路径是否为目录
function is_dir(path: String): Bool {
  // 由C后端实现
}

// ==========================================
// 临时文件
// ==========================================

/// 创建临时文件
function temp_file(): Result<String, String> {
  // 由C后端实现
}

/// 创建临时目录
function temp_dir(): Result<String, String> {
  // 由C后端实现
}

/// 获取系统临时目录
function get_temp_dir(): String {
  // 由C后端实现
}

// ==========================================
// 信号处理
// ==========================================

/// 信号类型
type Signal = {
  name: String,
  number: Int,
}

/// 信号定义
let SIGINT: Signal = { name: "SIGINT", number: 2 }     // 中断信号
let SIGTERM: Signal = { name: "SIGTERM", number: 15 }   // 终止信号
let SIGKILL: Signal = { name: "SIGKILL", number: 9 }   // 强制终止信号
let SIGSEGV: Signal = { name: "SIGSEGV", number: 11 }  // 段错误

/// 注册信号处理器
function signal(signum: Int, handler: () -> ()): Bool {
  // 由C后端实现
}

/// 发送信号到进程
function kill(pid: Int, signum: Int): Bool {
  // 由C后端实现
}

// ==========================================
// 系统调用
// ==========================================

/// 系统调用（底层）
function syscall(number: Int, args: List<Int>): Int {
  // 由C后端实现
}

// ==========================================
// 时间相关系统函数
// ==========================================

/// 获取系统启动时间（秒）
function uptime(): Float {
  // 由C后端实现
}

// ==========================================
// 随机数
// ==========================================

/// 生成随机整数（0 到 max-1）
function random(max: Int): Int {
  // 由C后端实现
}

/// 生成随机浮点数（0.0 到 1.0）
function random_float(): Float {
  // 由C后端实现
}

/// 设置随机数种子
function srand(seed: Int) {
  // 由C后端实现
}

// ==========================================
// 其他系统函数
// ==========================================

/// 获取用户ID
function getuid(): Int {
  // 由C后端实现
}

/// 获取组ID
function getgid(): Int {
  // 由C后端实现
}

/// 获取用户名
function get_username(): String {
  // 由C后端实现
}

/// 获取组名
function get_groupname(): String {
  // 由C后端实现
}