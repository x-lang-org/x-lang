// X语言标准库 - 字符串操作
//
// 常用的字符串处理函数

// ==========================================
// 字符串基本属性
// ==========================================

/// 获取字符串长度（字符数）
fun str_len(s: String): Int {
  // 内置函数
  "__builtin_str_len"
}

/// 检查字符串是否为空
fun str_is_empty(s: String): Bool {
  str_len(s) == 0
}

/// 获取字符串的字节长度
fun str_byte_len(s: String): Int {
  // 内置函数
  "__builtin_str_byte_len"
}

// ==========================================
// 字符访问
// ==========================================

/// 获取字符串的所有字符
fun str_chars(s: String): [Char] {
  // 内置函数
  "__builtin_str_chars"
}

/// 获取指定位置的字符
fun str_get(s: String, index: Int): Option<Char> {
  let chars = str_chars(s)
  if index >= 0 && index < list_len(chars) {
    Some(list_get(chars, index))
  } else {
    None()
  }
}

/// 获取第一个字符
fun str_first(s: String): Option<Char> {
  str_get(s, 0)
}

/// 获取最后一个字符
fun str_last(s: String): Option<Char> {
  let len = str_len(s)
  if len > 0 {
    str_get(s, len - 1)
  } else {
    None()
  }
}

// ==========================================
// 字符串比较
// ==========================================

/// 比较两个字符串（字典序）
fun str_compare(a: String, b: String): Int {
  // 内置函数
  "__builtin_str_compare"
}

/// 检查字符串是否相等
fun str_eq(a: String, b: String): Bool {
  a == b
}

// ==========================================
// 字符串拼接
// ==========================================

/// 拼接两个字符串
fun str_concat(a: String, b: String): String {
  a + b
}

/// 拼接多个字符串
fun str_join(strings: [String], separator: String): String {
  if list_is_empty(strings) {
    ""
  } else {
    let mut result = list_get(strings, 0)
    let mut i = 1
    while i < list_len(strings) {
      result = result + separator + list_get(strings, i)
      i = i + 1
    }
    result
  }
}

/// 重复字符串 n 次
fun str_repeat(s: String, n: Int): String {
  if n <= 0 {
    ""
  } else {
    let mut result = ""
    let mut i = 0
    while i < n {
      result = result + s
      i = i + 1
    }
    result
  }
}

// ==========================================
// 字符串包含检查
// ==========================================

/// 检查字符串是否包含子串
fun str_contains(s: String, substr: String): Bool {
  // 内置函数
  "__builtin_str_contains"
}

/// 检查字符串是否以指定前缀开头
fun str_starts_with(s: String, prefix: String): Bool {
  // 内置函数
  "__builtin_str_starts_with"
}

/// 检查字符串是否以指定后缀结尾
fun str_ends_with(s: String, suffix: String): Bool {
  // 内置函数
  "__builtin_str_ends_with"
}

// ==========================================
// 字符串提取
// ==========================================

/// 提取子字符串
fun str_substring(s: String, start: Int, end: Int): String {
  // 内置函数
  "__builtin_str_substring"
}

/// 提取从 start 到末尾的子串
fun str_slice(s: String, start: Int): String {
  str_substring(s, start, str_len(s))
}

/// 获取前 n 个字符
fun str_take(s: String, n: Int): String {
  if n <= 0 {
    ""
  } else {
    str_substring(s, 0, min_int(n, str_len(s)))
  }
}

/// 去掉前 n 个字符
fun str_drop(s: String, n: Int): String {
  if n <= 0 {
    s
  } else {
    str_slice(s, min_int(n, str_len(s)))
  }
}

// ==========================================
// 字符串替换
// ==========================================

/// 替换子字符串
fun str_replace(s: String, from: String, to: String): String {
  // 内置函数
  "__builtin_str_replace"
}

/// 替换第一个匹配的子字符串
fun str_replace_first(s: String, from: String, to: String): String {
  // 内置函数
  "__builtin_str_replace_first"
}

// ==========================================
// 字符串大小写转换
// ==========================================

/// 转换为小写
fun str_to_lowercase(s: String): String {
  // 内置函数
  "__builtin_str_to_lowercase"
}

/// 转换为大写
fun str_to_uppercase(s: String): String {
  // 内置函数
  "__builtin_str_to_uppercase"
}

/// 首字母大写
fun str_capitalize(s: String): String {
  if str_is_empty(s) {
    s
  } else {
    let first = str_get(s, 0)
    let rest = str_drop(s, 1)
    str_to_uppercase(char_to_string(first)) + rest
  }
}

// ==========================================
// 字符串修剪
// ==========================================

/// 去除首尾空白
fun str_trim(s: String): String {
  // 内置函数
  "__builtin_str_trim"
}

/// 去除开头空白
fun str_trim_start(s: String): String {
  // 内置函数
  "__builtin_str_trim_start"
}

/// 去除结尾空白
fun str_trim_end(s: String): String {
  // 内置函数
  "__builtin_str_trim_end"
}

/// 去除首尾指定字符
fun str_trim_chars(s: String, chars: String): String {
  str_trim_start_chars(str_trim_end_chars(s, chars), chars)
}

/// 去除开头指定字符
fun str_trim_start_chars(s: String, chars: String): String {
  let mut i = 0
  let len = str_len(s)
  while i < len {
    let c = str_get(s, i)
    if not str_contains(chars, char_to_string(c)) {
      break
    }
    i = i + 1
  }
  str_slice(s, i)
}

/// 去除结尾指定字符
fun str_trim_end_chars(s: String, chars: String): String {
  let mut i = str_len(s)
  while i > 0 {
    let c = str_get(s, i - 1)
    if not str_contains(chars, char_to_string(c)) {
      break
    }
    i = i - 1
  }
  str_substring(s, 0, i)
}

// ==========================================
// 字符串填充
// ==========================================

/// 左侧填充到指定长度
fun str_pad_left(s: String, width: Int, pad_char: Char): String {
  let len = str_len(s)
  if len >= width {
    s
  } else {
    str_repeat(char_to_string(pad_char), width - len) + s
  }
}

/// 右侧填充到指定长度
fun str_pad_right(s: String, width: Int, pad_char: Char): String {
  let len = str_len(s)
  if len >= width {
    s
  } else {
    s + str_repeat(char_to_string(pad_char), width - len)
  }
}

/// 居中填充到指定长度
fun str_center(s: String, width: Int, pad_char: Char): String {
  let len = str_len(s)
  if len >= width {
    s
  } else {
    let total_pad = width - len
    let left_pad = total_pad / 2
    let right_pad = total_pad - left_pad
    str_repeat(char_to_string(pad_char), left_pad) + s + str_repeat(char_to_string(pad_char), right_pad)
  }
}

// ==========================================
// 字符串分割
// ==========================================

/// 按分隔符分割字符串
fun str_split(s: String, separator: String): [String] {
  // 内置函数
  "__builtin_str_split"
}

/// 按空白分割字符串
fun str_split_whitespace(s: String): [String] {
  str_split(str_trim(s), " ")
}

/// 按行分割字符串
fun str_lines(s: String): [String] {
  str_split(s, "\n")
}

// ==========================================
// 字符串解析
// ==========================================

/// 解析为整数
fun str_parse_int(s: String): Option<Int> {
  // 内置函数
  "__builtin_str_parse_int"
}

/// 解析为浮点数
fun str_parse_float(s: String): Option<Float> {
  // 内置函数
  "__builtin_str_parse_float"
}

/// 解析为布尔值
fun str_parse_bool(s: String): Option<Bool> {
  let lower = str_to_lowercase(str_trim(s))
  if lower == "true" || lower == "yes" || lower == "1" {
    Some(true)
  } else if lower == "false" || lower == "no" || lower == "0" {
    Some(false)
  } else {
    None()
  }
}

// ==========================================
// 字符和字符串转换
// ==========================================

/// 字符转换为字符串
fun char_to_string(c: Char): String {
  // 内置函数
  "__builtin_char_to_string"
}

/// 数字转换为字符
fun char_code(c: Char): Int {
  // 内置函数
  "__builtin_char_code"
}

/// 从字符码创建字符
fun char_from_code(code: Int): Option<Char> {
  // 内置函数
  "__builtin_char_from_code"
}

// ==========================================
// 字符分类
// ==========================================

/// 检查字符是否是字母
fun char_is_alpha(c: Char): Bool {
  (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

/// 检查字符是否是数字
fun char_is_digit(c: Char): Bool {
  c >= '0' && c <= '9'
}

/// 检查字符是否是字母或数字
fun char_is_alphanumeric(c: Char): Bool {
  char_is_alpha(c) || char_is_digit(c)
}

/// 检查字符是否是空白
fun char_is_whitespace(c: Char): Bool {
  c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

/// 检查字符是否是小写字母
fun char_is_lowercase(c: Char): Bool {
  c >= 'a' && c <= 'z'
}

/// 检查字符是否是大写字母
fun char_is_uppercase(c: Char): Bool {
  c >= 'A' && c <= 'Z'
}

// ==========================================
// 字符串反转
// ==========================================

/// 反转字符串
fun str_reverse(s: String): String {
  let chars = str_chars(s)
  let mut result = ""
  let mut i = list_len(chars) - 1
  while i >= 0 {
    result = result + char_to_string(list_get(chars, i))
    i = i - 1
  }
  result
}

// ==========================================
// 字符串检查
// ==========================================

/// 检查字符串是否只包含字母
fun str_is_alpha(s: String): Bool {
  if str_is_empty(s) {
    false
  } else {
    let chars = str_chars(s)
    let mut i = 0
    while i < list_len(chars) {
      if not char_is_alpha(list_get(chars, i)) {
        return false
      }
      i = i + 1
    }
    true
  }
}

/// 检查字符串是否只包含数字
fun str_is_digit(s: String): Bool {
  if str_is_empty(s) {
    false
  } else {
    let chars = str_chars(s)
    let mut i = 0
    while i < list_len(chars) {
      if not char_is_digit(list_get(chars, i)) {
        return false
      }
      i = i + 1
    }
    true
  }
}

/// 检查字符串是否只包含字母或数字
fun str_is_alphanumeric(s: String): Bool {
  if str_is_empty(s) {
    false
  } else {
    let chars = str_chars(s)
    let mut i = 0
    while i < list_len(chars) {
      if not char_is_alphanumeric(list_get(chars, i)) {
        return false
      }
      i = i + 1
    }
    true
  }
}

/// 检查字符串是否只包含空白
fun str_is_whitespace(s: String): Bool {
  if str_is_empty(s) {
    true
  } else {
    let chars = str_chars(s)
    let mut i = 0
    while i < list_len(chars) {
      if not char_is_whitespace(list_get(chars, i)) {
        return false
      }
      i = i + 1
    }
    true
  }
}

// ==========================================
// 格式化辅助
// ==========================================

/// 将整数格式化为指定宽度的字符串
fun format_int(n: Int, width: Int): String {
  str_pad_left(to_string(n), width, '0')
}

/// 将浮点数格式化为指定小数位数的字符串
fun format_float(n: Float, decimals: Int): String {
  // 简单实现，实际需要内置函数支持
  let s = to_string(n)
  let parts = str_split(s, ".")
  if list_len(parts) == 1 {
    s + "." + str_repeat("0", decimals)
  } else {
    let int_part = list_get(parts, 0)
    let frac_part = list_get(parts, 1)
    int_part + "." + str_take(frac_part + str_repeat("0", decimals), decimals)
  }
}
