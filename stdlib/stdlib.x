// X语言标准库 - 主入口
//
// 导入这个文件来使用完整的标准库
//
// 模块列表:
//   - prelude:  自动导入的核心函数
//   - option:   Option 类型（代替 null）
//   - result:   Result 类型（代替异常）
//   - math:     数学函数和常量
//   - string:   字符串操作
//   - collections: 集合操作（列表、映射、集合）
//   - iter:     迭代器
//   - io:       输入输出和文件操作
//   - time:     时间处理
//   - sys:      系统功能

// ==========================================
// 标准库版本
// ==========================================

/// 标准库主版本
let STDLIB_VERSION_MAJOR: Int = 0

/// 标准库次版本
let STDLIB_VERSION_MINOR: Int = 1

/// 标准库补丁版本
let STDLIB_VERSION_PATCH: Int = 0

/// 标准库版本字符串
let STDLIB_VERSION: String =
  to_string(STDLIB_VERSION_MAJOR) + "." +
  to_string(STDLIB_VERSION_MINOR) + "." +
  to_string(STDLIB_VERSION_PATCH)

/// 获取标准库版本
fun stdlib_version(): String {
  STDLIB_VERSION
}

// ==========================================
// 重新导出核心模块
// ==========================================

// 这些模块在实际使用时会被解释器特殊处理
// 作为标准库的公共 API

// Prelude - 自动导入
// export * from prelude

// Option
// export Some, None, is_some, is_none, unwrap, unwrap_or from option

// Result
// export Ok, Err, is_ok, is_err from result

// Math
// export pi, e, sqrt, sin, cos, tan, abs, min, max, ... from math

// String
// export str_len, str_concat, str_split, ... from string

// Collections
// export list_new, list_push, list_map, ... from collections

// Iter
// export iter_range, iter_map, iter_filter, ... from iter

// IO
// export print, println, read_file, write_file, ... from io

// Time
// export now, timestamp, sleep, ... from time

// Sys
// export exit, env_var, args, ... from sys

// ==========================================
// 标准库初始化
// ==========================================

/// 初始化标准库
fun init_stdlib() {
  // 这里可以进行标准库的初始化工作
  // 例如设置随机种子、初始化日志等
  srand(timestamp())
}

// 自动初始化
init_stdlib()
