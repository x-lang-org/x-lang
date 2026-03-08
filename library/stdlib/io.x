// X语言标准库 - 输入输出
//
// 文件操作、控制台输入输出等

// ==========================================
// 标准输入输出
// ==========================================

/// 从标准输入读取一行
function input(): String {
  // 简单实现，实际需要底层支持
  ""
}

/// 从标准输入读取一行，带提示
function input_prompt(prompt: String): String {
  print(prompt)
  input()
}

/// 打印到标准输出（不换行）
function print(...values) {
  // 简单实现，实际需要底层支持
  let mut output = ""
  for value in values {
    output = output + to_string(value)
  }
  output
}

/// 打印到标准输出（换行）
function println(...values) {
  // 简单实现，实际需要底层支持
  let mut output = ""
  for value in values {
    output = output + to_string(value)
  }
  output + "\n"
}

/// 格式化字符串
function format(template: String, ...args): String {
  // 简单实现，实际需要底层支持
  let mut result = template
  let mut i = 0
  while i < list_len(args) {
    let placeholder = "{" + to_string(i) + "}"
    result = str_replace(result, placeholder, to_string(args[i]))
    i = i + 1
  }
  result
}

// ==========================================
// 文件操作
// ==========================================

/// 读取文件全部内容为字符串
function read_file(path: String): Result<String, String> {
  // 简单实现，实际需要底层支持
  Err("File operation not implemented")
}

/// 写入字符串到文件
function write_file(path: String, content: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("File operation not implemented")
}

/// 追加内容到文件
function append_file(path: String, content: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("File operation not implemented")
}

/// 检查文件是否存在
function file_exists(path: String): Bool {
  // 简单实现，实际需要底层支持
  false
}

/// 删除文件
function delete_file(path: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("File operation not implemented")
}

/// 复制文件
function copy_file(from: String, to: String): Result<Unit, String> {
  match read_file(from) is
    Ok { value: content } -> write_file(to, content)
    Err { error: e } -> Err(e)
}

/// 移动/重命名文件
function move_file(from: String, to: String): Result<Unit, String> {
  match copy_file(from, to) is
    Ok { value: _ } -> delete_file(from)
    Err { error: e } -> Err(e)
}

// ==========================================
// 目录操作
// ==========================================

/// 创建目录
function create_dir(path: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("Directory operation not implemented")
}

/// 创建目录（包括父目录）
function create_dir_all(path: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("Directory operation not implemented")
}

/// 列出目录内容
function list_dir(path: String): Result<[String], String> {
  // 简单实现，实际需要底层支持
  Err("Directory operation not implemented")
}

/// 检查目录是否存在
function dir_exists(path: String): Bool {
  // 简单实现，实际需要底层支持
  false
}

/// 删除空目录
function delete_dir(path: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("Directory operation not implemented")
}

/// 删除目录及其内容
function delete_dir_all(path: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("Directory operation not implemented")
}

/// 获取当前工作目录
function current_dir(): Result<String, String> {
  // 简单实现，实际需要底层支持
  Err("Directory operation not implemented")
}

/// 改变当前工作目录
function set_current_dir(path: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("Directory operation not implemented")
}

// ==========================================
// 路径操作
// ==========================================

/// 连接路径
function path_join(parts: [String]): String {
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
function path_dirname(path: String): String {
  let parts = str_split(path, "/")
  if list_len(parts) <= 1 {
    return "."
  }
  let dir_parts = list_slice(parts, 0, list_len(parts) - 1)
  str_join(dir_parts, "/")
}

/// 获取路径的文件名部分
function path_basename(path: String): String {
  let parts = str_split(path, "/")
  if list_is_empty(parts) {
    return ""
  }
  list_last(parts)
}

/// 获取文件扩展名
function path_extension(path: String): Option<String> {
  let basename = path_basename(path)
  let parts = str_split(basename, ".")
  if list_len(parts) > 1 {
    Some(list_last(parts))
  } else {
    None()
  }
}

/// 去除文件扩展名
function path_without_extension(path: String): String {
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
function path_is_absolute(path: String): Bool {
  str_starts_with(path, "/") || str_contains(path, ":")
}

/// 检查路径是否是相对路径
function path_is_relative(path: String): Bool {
  not path_is_absolute(path)
}

// ==========================================
// 文件元数据
// ==========================================

/// 获取文件大小（字节）
function file_size(path: String): Option<Int> {
  // 简单实现，实际需要底层支持
  None()
}

/// 检查是否是文件
function is_file(path: String): Bool {
  // 简单实现，实际需要底层支持
  false
}

/// 检查是否是目录
function is_dir(path: String): Bool {
  // 简单实现，实际需要底层支持
  false
}

// ==========================================
// 逐行读取
// ==========================================

/// 读取文件行
function read_lines(path: String): Result<[String], String> {
  match read_file(path) is
    Ok { value: content } -> Ok(str_lines(content))
    Err { error: e } -> Err(e)
}

/// 写入行到文件
function write_lines(path: String, lines: [String]): Result<Unit, String> {
  let content = str_join(lines, "\n") + "\n"
  write_file(path, content)
}

/// 追加行到文件
function append_lines(path: String, lines: [String]): Result<Unit, String> {
  let content = str_join(lines, "\n") + "\n"
  append_file(path, content)
}

// ==========================================
// 临时文件
// ==========================================

/// 创建临时文件
function temp_file(): Result<String, String> {
  // 简单实现，实际需要底层支持
  Err("Temporary file operation not implemented")
}

/// 创建临时目录
function temp_dir(): Result<String, String> {
  // 简单实现，实际需要底层支持
  Err("Temporary directory operation not implemented")
}

// ==========================================
// 环境变量
// ==========================================

/// 获取环境变量
function env_var(name: String): Option<String> {
  // 简单实现，实际需要底层支持
  None()
}

/// 设置环境变量
function set_env_var(name: String, value: String): Result<Unit, String> {
  // 简单实现，实际需要底层支持
  Err("Environment variable operation not implemented")
}

/// 获取所有环境变量
function env_vars(): Result<{String: String}, String> {
  // 简单实现，实际需要底层支持
  Err("Environment variable operation not implemented")
}

// ==========================================
// 进程操作
// ==========================================

/// 退出程序
function exit(code: Int): Unit {
  // 简单实现，实际需要底层支持
  code
}

/// 获取命令行参数
function args(): [String] {
  // 简单实现，实际需要底层支持
  []
}

/// 获取程序名
function program_name(): String {
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
function eprint(...values) {
  // 简单实现，实际需要底层支持
  let mut output = ""
  for value in values {
    output = output + to_string(value)
  }
  output
}

/// 打印错误信息到标准错误（带换行）
function eprintln(...values) {
  // 简单实现，实际需要底层支持
  let mut output = ""
  for value in values {
    output = output + to_string(value)
  }
  output + "\n"
}

/// 格式化并打印调试信息
function dbg_fmt(template: String, ...args) {
  println(format(template, ...args))
}
