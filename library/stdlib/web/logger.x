// X语言Web Framework - 日志系统模块
//
// 提供日志的记录和管理功能

import "../collections"

/// 日志级别
type LogLevel {
  name: String,
  priority: Int,
}

/// 日志记录器类型
type Logger {
  level: LogLevel,
  handlers: List<(String) -> Void>,
  name: String,
}

/// 日志级别常量
let DEBUG = LogLevel{ name: "DEBUG", priority: 10 }
let INFO = LogLevel{ name: "INFO", priority: 20 }
let WARN = LogLevel{ name: "WARN", priority: 30 }
let ERROR = LogLevel{ name: "ERROR", priority: 40 }
let FATAL = LogLevel{ name: "FATAL", priority: 50 }

/// 日志级别映射
let log_levels = {
  debug: DEBUG,
  info: INFO,
  warn: WARN,
  error: ERROR,
  fatal: FATAL,
}

/// 创建日志记录器
///
/// # 参数
/// - `name`: 日志记录器名称
/// - `level`: 日志级别
///
/// # 返回值
/// - `Logger`: 新创建的日志记录器对象
function logger(name: String, level: LogLevel): Logger {
  {
    level: level,
    handlers: list(),
    name: name,
  }
}

/// 创建默认日志记录器
///
/// # 返回值
/// - `Logger`: 默认日志记录器对象
function default_logger(): Logger {
  let logger = logger("app", INFO)
  add_console_handler(logger)
  logger
}

/// 添加控制台日志处理器
///
/// # 参数
/// - `logger`: 日志记录器对象
///
/// # 返回值
/// - `Logger`: 更新后的日志记录器对象
function add_console_handler(logger: Logger): Logger {
  let handler = fn(message: String) {
    println(message)
  }
  push(logger.handlers, handler)
  logger
}

/// 添加文件日志处理器
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `path`: 日志文件路径
///
/// # 返回值
/// - `Logger`: 更新后的日志记录器对象
function add_file_handler(logger: Logger, path: String): Logger {
  let handler = fn(message: String) {
    append_file(path, message + "\n")
  }
  push(logger.handlers, handler)
  logger
}

/// 记录调试级别日志
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `message`: 日志消息
function debug(logger: Logger, message: String) {
  log(logger, DEBUG, message)
}

/// 记录信息级别日志
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `message`: 日志消息
function info(logger: Logger, message: String) {
  log(logger, INFO, message)
}

/// 记录警告级别日志
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `message`: 日志消息
function warn(logger: Logger, message: String) {
  log(logger, WARN, message)
}

/// 记录错误级别日志
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `message`: 日志消息
function error(logger: Logger, message: String) {
  log(logger, ERROR, message)
}

/// 记录致命级别日志
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `message`: 日志消息
function fatal(logger: Logger, message: String) {
  log(logger, FATAL, message)
}

/// 记录日志
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `level`: 日志级别
/// - `message`: 日志消息
function log(logger: Logger, level: LogLevel, message: String) {
  if level.priority >= logger.level.priority {
    let timestamp = now()
    let log_message = "[" + timestamp + "] [" + level.name + "] [" + logger.name + "] " + message
    
    for handler in logger.handlers {
      handler(log_message)
    }
  }
}

/// 设置日志级别
///
/// # 参数
/// - `logger`: 日志记录器对象
/// - `level`: 日志级别
///
/// # 返回值
/// - `Logger`: 更新后的日志记录器对象
function set_log_level(logger: Logger, level: LogLevel): Logger {
  logger.level = level
  logger
}

/// 获取日志级别
///
/// # 参数
/// - `name`: 日志级别名称
///
/// # 返回值
/// - `Option<LogLevel>`: 日志级别对象
function get_log_level(name: String): Option<LogLevel> {
  if map_contains(log_levels, name) {
    Some(map_get(log_levels, name))
  } else {
    None
  }
}
