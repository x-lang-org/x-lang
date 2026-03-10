// X语言Web Framework - 配置系统模块
//
// 提供配置的加载和管理功能

import "../collections"

/// 配置类型
type Config {
  values: Map<String, Any>,
}

/// 创建配置对象
///
/// # 返回值
/// - `Config`: 新创建的配置对象
function config(): Config {
  {
    values: map(),
  }
}

/// 从文件加载配置
///
/// # 参数
/// - `path`: 配置文件路径
///
/// # 返回值
/// - `Option<Config>`: 加载成功返回Some(Config)，失败返回None
function load_config(path: String): Option<Config> {
  // 检查文件是否存在
  if !exists(path) {
    return None
  }
  
  // 读取文件内容
  let content = read_file(path)
  
  // 解析配置（这里使用简单的键值对格式）
  let config = config()
  let lines = split(content, "\n")
  
  for line in lines {
    // 跳过空行和注释
    let trimmed = trim(line)
    if trimmed == "" || starts_with(trimmed, "#") {
      continue
    }
    
    // 解析键值对
    let parts = split(trimmed, "=")
    if len(parts) >= 2 {
      let key = trim(parts[0])
      let value = trim(parts[1])
      
      // 尝试解析为不同类型
      if value == "true" {
        map_set(config.values, key, true)
      } else if value == "false" {
        map_set(config.values, key, false)
      } else if is_number(value) {
        map_set(config.values, key, to_number(value))
      } else {
        map_set(config.values, key, value)
      }
    }
  }
  
  Some(config)
}

/// 从环境变量加载配置
///
/// # 返回值
/// - `Config`: 加载的配置对象
function load_config_from_env(): Config {
  let config = config()
  
  // 这里需要实现从环境变量加载配置
  // 由于X语言还没有环境变量相关的系统函数，这里使用模拟实现
  println("Loading config from environment variables")
  
  // 模拟环境变量
  map_set(config.values, "PORT", 3000)
  map_set(config.values, "HOST", "localhost")
  map_set(config.values, "DEBUG", true)
  
  config
}

/// 获取配置值
///
/// # 参数
/// - `config`: 配置对象
/// - `key`: 配置键
/// - `default`: 默认值
///
/// # 返回值
/// - `Any`: 配置值或默认值
function get_config(config: Config, key: String, default: Any): Any {
  if map_contains(config.values, key) {
    map_get(config.values, key)
  } else {
    default
  }
}

/// 设置配置值
///
/// # 参数
/// - `config`: 配置对象
/// - `key`: 配置键
/// - `value`: 配置值
///
/// # 返回值
/// - `Config`: 更新后的配置对象
function set_config(config: Config, key: String, value: Any): Config {
  map_set(config.values, key, value)
  config
}

/// 检查配置是否存在
///
/// # 参数
/// - `config`: 配置对象
/// - `key`: 配置键
///
/// # 返回值
/// - `Bool`: 配置存在返回true，否则返回false
function has_config(config: Config, key: String): Bool {
  map_contains(config.values, key)
}

/// 合并配置
///
/// # 参数
/// - `config`: 目标配置对象
/// - `other`: 源配置对象
///
/// # 返回值
/// - `Config`: 合并后的配置对象
function merge_config(config: Config, other: Config): Config {
  for (key, value) in entries(other.values) {
    map_set(config.values, key, value)
  }
  config
}

/// 保存配置到文件
///
/// # 参数
/// - `config`: 配置对象
/// - `path`: 配置文件路径
///
/// # 返回值
/// - `Bool`: 保存成功返回true，失败返回false
function save_config(config: Config, path: String): Bool {
  let mut content = ""
  
  for (key, value) in entries(config.values) {
    content = content + key + "=" + to_string(value) + "\n"
  }
  
  // 写入文件
  write_file(path, content)
  true
}
