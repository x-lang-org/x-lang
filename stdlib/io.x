// X语言标准库 - 输入输出
//
// 文件操作、控制台输入输出等

// ==========================================
// 标准输入输出
// ==========================================

/// 从标准输入读取一行
fun input(): String {
  // 内置函数
  "__builtin_input"
}

/// 从标准输入读取一行，带提示
fun input_prompt(prompt: String): String {
  print(prompt)
  input()
}

/// 打印到标准输出（不换行）
fun print(...values) {
  // 内置函数，解释器特殊处理
  "__builtin_print"
}

/// 打印到标准输出（换行）
fun println(...values) {
  // 内置函数
  "__builtin_println"
}

/// 格式化字符串
fun format(template: String, ...args): String {
  // 内置函数
  "__builtin_format"
}

// ==========================================
// 文件操作
// ==========================================

/// 读取文件全部内容为字符串
fun read_file(path: String): Result<String, String> {
  // 内置函数
  "__builtin_read_file"
}

/// 写入字符串到文件
fun write_file(path: String, content: String): Result<Unit, String> {
  // 内置函数
  "__builtin_write_file"
}

/// 追加内容到文件
fun append_file(path: String, content: String): Result<Unit, String> {
  // 内置函数
  "__builtin_append_file"
}

/// 检查文件是否存在
fun file_exists(path: String): Bool {
  // 内置函数
  "__builtin_file_exists"
}

/// 删除文件
fun delete_file(path: String): Result<Unit, String> {
  // 内置函数
  "__builtin_delete_file"
}

/// 复制文件
fun copy_file(from: String, to: String): Result<Unit, String> {
  match read_file(from) is
    Ok { value: content } -> write_file(to, content)
    Err { error: e } -> Err(e)
}

/// 移动/重命名文件
fun move_file(from: String, to: String): Result<Unit, String> {
  match copy_file(from, to) is
    Ok { value: _ } -> delete_file(from)
    Err { error: e } -> Err(e)
}

// ==========================================
// 目录操作
// ==========================================

/// 创建目录
fun create_dir(path: String): Result<Unit, String> {
  // 内置函数
  "__builtin_create_dir"
}

/// 创建目录（包括父目录）
fun create_dir_all(path: String): Result<Unit, String> {
  // 内置函数
  "__builtin_create_dir_all"
}

/// 列出目录内容
fun list_dir(path: String): Result<[String], String> {
  // 内置函数
  "__builtin_list_dir"
}

/// 检查目录是否存在
fun dir_exists(path: String): Bool {
  // 内置函数
  "__builtin_dir_exists"
}

/// 删除空目录
fun delete_dir(path: String): Result<Unit, String> {
  // 内置函数
  "__builtin_delete_dir"
}

/// 删除目录及其内容
fun delete_dir_all(path: String): Result<Unit, String> {
  // 内置函数
  "__builtin_delete_dir_all"
}

/// 获取当前工作目录
fun current_dir(): Result<String, String> {
  // 内置函数
  "__builtin_current_dir"
}

/// 改变当前工作目录
fun set_current_dir(path: String): Result<Unit, String> {
  // 内置函数
  "__builtin_set_current_dir"
}

// ==========================================
// 路径操作
// ==========================================

/// 连接路径
fun path_join(parts: [String]): String {
  if list_is_empty(parts) {
    return ""
  }
  let mut result = parts[0]
  let mut i = 1
  while i < list_len(parts) {
    let part = parts[i]
    if str_starts_with(part, "/") || str_contains(part, ":") {
      // 绝对路径，替换前面的
      result = part
    } else if str_ends_with(result, "/") {
      result = result + part
    } else {
      result = result + "/" + part
    }
    i = i + 1
  }
  result
}

/// 获取路径的目录部分
fun path_dirname(path: String): String {
  let parts = str_split(path, "/")
  if list_len(parts) <= 1 {
    return "."
  }
  let dir_parts = list_slice(parts, 0, list_len(parts) - 1)
  str_join(dir_parts, "/")
}

/// 获取路径的文件名部分
fun path_basename(path: String): String {
  let parts = str_split(path, "/")
  if list_is_empty(parts) {
    return ""
  }
  list_last(parts)
}

/// 获取文件扩展名
fun path_extension(path: String): Option<String> {
  let basename = path_basename(path)
  let parts = str_split(basename, ".")
  if list_len(parts) > 1 {
    Some(list_last(parts))
  } else {
    None()
  }
}

/// 去除文件扩展名
fun path_without_extension(path: String): String {
  let dir = path_dirname(path)
  let basename = path_basename(path)
  let parts = str_split(basename, ".")
  if list_len(parts) <= 1 {
    path
  } else {
    let name_parts = list_slice(parts, 0, list_len(parts) - 1)
    let name = str_join(name_parts, ".")
    if dir == "." {
      name
    } else {
      dir + "/" + name
    }
  }
}

/// 检查路径是否是绝对路径
fun path_is_absolute(path: String): Bool {
  str_starts_with(path, "/") || str_contains(path, ":")
}

/// 检查路径是否是相对路径
fun path_is_relative(path: String): Bool {
  not path_is_absolute(path)
}

// ==========================================
// 文件元数据
// ==========================================

/// 获取文件大小（字节）
fun file_size(path: String): Option<Int> {
  // 内置函数
  "__builtin_file_size"
}

/// 检查是否是文件
fun is_file(path: String): Bool {
  // 内置函数
  "__builtin_is_file"
}

/// 检查是否是目录
fun is_dir(path: String): Bool {
  // 内置函数
  "__builtin_is_dir"
}

// ==========================================
// 逐行读取
// ==========================================

/// 读取文件行
fun read_lines(path: String): Result<[String], String> {
  match read_file(path) is
    Ok { value: content } -> Ok(str_lines(content))
    Err { error: e } -> Err(e)
}

/// 写入行到文件
fun write_lines(path: String, lines: [String]): Result<Unit, String> {
  let content = str_join(lines, "\n") + "\n"
  write_file(path, content)
}

/// 追加行到文件
fun append_lines(path: String, lines: [String]): Result<Unit, String> {
  let content = str_join(lines, "\n") + "\n"
  append_file(path, content)
}

// ==========================================
// 临时文件
// ==========================================

/// 创建临时文件
fun temp_file(): Result<String, String> {
  // 内置函数
  "__builtin_temp_file"
}

/// 创建临时目录
fun temp_dir(): Result<String, String> {
  // 内置函数
  "__builtin_temp_dir"
}

// ==========================================
// 环境变量
// ==========================================

/// 获取环境变量
fun env_var(name: String): Option<String> {
  // 内置函数
  "__builtin_env_var"
}

/// 设置环境变量
fun set_env_var(name: String, value: String): Result<Unit, String> {
  // 内置函数
  "__builtin_set_env_var"
}

/// 获取所有环境变量
fn env_vars(): Result<{String: String}, String> {
  // 内置函数
  "__builtin_env_vars"
}

// ==========================================
// 进程操作
// ==========================================

/// 退出程序
fun exit(code: Int): Unit {
  // 内置函数
  "__builtin_exit"
}

/// 获取命令行参数
fn args(): [String] {
  // 内置函数
  "__builtin_args"
}

/// 获取程序名
fn program_name(): String {
  let argv = args()
  if list_is_empty(argv) {
    ""
  } else {
    path_basename(argv[0])
  }
}

// ==========================================
// 调试和日志
// ==========================================

/// 打印错误信息到标准错误
fun eprint(...values) {
  // 内置函数
  "__builtin_eprint"
}

/// 打印错误信息到标准错误（带换行）
fun eprintln(...values) {
  // 内置函数
  "__builtin_eprintln"
}

/// 格式化并打印调试信息
fun dbg_fmt(template: String, ...args) {
  println(format(template, ...args))
}
