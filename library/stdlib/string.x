// X语言标准库 - 字符串操作
//
// 常用的字符串处理函数

// ==========================================
// 字符串基本属性
// ==========================================

/// 获取字符串长度（字符数）
function str_len(s: String): Int {
  let mut length = 0
  let mut i = 0
  while i < str_byte_len(s) {
    let code = str_byte_at(s, i)
    if code < 0x80 {
      i = i + 1
    } else if code < 0xE0 {
      i = i + 2
    } else if code < 0xF0 {
      i = i + 3
    } else {
      i = i + 4
    }
    length = length + 1
  }
  length
}

/// 检查字符串是否为空
function str_is_empty(s: String): Bool {
  str_len(s) == 0
}

/// 获取字符串的字节长度
function str_byte_len(s: String): Int {
  let mut length = 0
  let mut i = 0
  while true {
    let code = str_byte_at(s, i)
    if code == 0 {
      break
    }
    length = length + 1
    i = i + 1
  }
  length
}

/// 获取字符串指定位置的字节
function str_byte_at(s: String, index: Int): Int {
  // 简单实现，实际需要底层支持
  0
}

// ==========================================
// 字符访问
// ==========================================

/// 获取字符串的所有字符
function str_chars(s: String): [Char] {
  let mut chars = []
  let mut i = 0
  let byte_length = str_byte_len(s)
  while i < byte_length {
    let code = str_byte_at(s, i)
    let c: Char
    if code < 0x80 {
      c = code as Char
      i = i + 1
    } else if code < 0xE0 {
      let code1 = str_byte_at(s, i + 1)
      c = ((code & 0x1F) << 6 | (code1 & 0x3F)) as Char
      i = i + 2
    } else if code < 0xF0 {
      let code1 = str_byte_at(s, i + 1)
      let code2 = str_byte_at(s, i + 2)
      c = ((code & 0x0F) << 12 | (code1 & 0x3F) << 6 | (code2 & 0x3F)) as Char
      i = i + 3
    } else {
      let code1 = str_byte_at(s, i + 1)
      let code2 = str_byte_at(s, i + 2)
      let code3 = str_byte_at(s, i + 3)
      c = ((code & 0x07) << 18 | (code1 & 0x3F) << 12 | (code2 & 0x3F) << 6 | (code3 & 0x3F)) as Char
      i = i + 4
    }
    chars = list_push(chars, c)
  }
  chars
}

/// 获取指定位置的字符
function str_get(s: String, index: Int): Option<Char> {
  let chars = str_chars(s)
  if index >= 0 && index < list_len(chars) {
    Some(list_get(chars, index))
  } else {
    None()
  }
}

/// 获取第一个字符
function str_first(s: String): Option<Char> {
  str_get(s, 0)
}

/// 获取最后一个字符
function str_last(s: String): Option<Char> {
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
function str_compare(a: String, b: String): Int {
  let len_a = str_len(a)
  let len_b = str_len(b)
  let min_len = if len_a < len_b { len_a } else { len_b }
  
  let chars_a = str_chars(a)
  let chars_b = str_chars(b)
  
  for i in 0..min_len {
    let c_a = list_get(chars_a, i)
    let c_b = list_get(chars_b, i)
    if c_a < c_b {
      return -1
    } else if c_a > c_b {
      return 1
    }
  }
  
  if len_a < len_b {
    -1
  } else if len_a > len_b {
    1
  } else {
    0
  }
}

/// 检查字符串是否相等
function str_eq(a: String, b: String): Bool {
  a == b
}

// ==========================================
// 字符串拼接
// ==========================================

/// 拼接两个字符串
function str_concat(a: String, b: String): String {
  a + b
}

/// 拼接多个字符串
function str_join(strings: [String], separator: String): String {
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
function str_repeat(s: String, n: Int): String {
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
function str_contains(s: String, substr: String): Bool {
  let s_len = str_len(s)
  let substr_len = str_len(substr)
  
  if substr_len == 0 {
    true
  } else if substr_len > s_len {
    false
  } else {
    let s_chars = str_chars(s)
    let substr_chars = str_chars(substr)
    
    for i in 0..(s_len - substr_len + 1) {
      let mut match_found = true
      for j in 0..substr_len {
        if list_get(s_chars, i + j) != list_get(substr_chars, j) {
          match_found = false
          break
        }
      }
      if match_found {
        return true
      }
    }
    false
  }
}

/// 检查字符串是否以指定前缀开头
function str_starts_with(s: String, prefix: String): Bool {
  let s_len = str_len(s)
  let prefix_len = str_len(prefix)
  
  if prefix_len == 0 {
    true
  } else if prefix_len > s_len {
    false
  } else {
    let s_chars = str_chars(s)
    let prefix_chars = str_chars(prefix)
    
    for i in 0..prefix_len {
      if list_get(s_chars, i) != list_get(prefix_chars, i) {
        return false
      }
    }
    true
  }
}

/// 检查字符串是否以指定后缀结尾
function str_ends_with(s: String, suffix: String): Bool {
  let s_len = str_len(s)
  let suffix_len = str_len(suffix)
  
  if suffix_len == 0 {
    true
  } else if suffix_len > s_len {
    false
  } else {
    let s_chars = str_chars(s)
    let suffix_chars = str_chars(suffix)
    
    for i in 0..suffix_len {
      if list_get(s_chars, s_len - suffix_len + i) != list_get(suffix_chars, i) {
        return false
      }
    }
    true
  }
}

// ==========================================
// 字符串提取
// ==========================================

/// 提取子字符串
function str_substring(s: String, start: Int, end: Int): String {
  let s_len = str_len(s)
  let start_idx = if start < 0 { 0 } else if start > s_len { s_len } else { start }
  let end_idx = if end < 0 { 0 } else if end > s_len { s_len } else { end }
  
  if start_idx >= end_idx {
    ""
  } else {
    let chars = str_chars(s)
    let mut result = ""
    for i in start_idx..end_idx {
      result = result + char_to_string(list_get(chars, i))
    }
    result
  }
}

/// 提取从 start 到末尾的子串
function str_slice(s: String, start: Int): String {
  str_substring(s, start, str_len(s))
}

/// 获取前 n 个字符
function str_take(s: String, n: Int): String {
  if n <= 0 {
    ""
  } else {
    str_substring(s, 0, min_int(n, str_len(s)))
  }
}

/// 去掉前 n 个字符
function str_drop(s: String, n: Int): String {
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
function str_replace(s: String, from: String, to: String): String {
  if str_is_empty(from) {
    s
  } else {
    let parts = str_split(s, from)
    str_join(parts, to)
  }
}

/// 替换第一个匹配的子字符串
function str_replace_first(s: String, from: String, to: String): String {
  if str_is_empty(from) {
    s
  } else {
    let s_len = str_len(s)
    let from_len = str_len(from)
    
    let s_chars = str_chars(s)
    let from_chars = str_chars(from)
    
    for i in 0..(s_len - from_len + 1) {
      let mut match_found = true
      for j in 0..from_len {
        if list_get(s_chars, i + j) != list_get(from_chars, j) {
          match_found = false
          break
        }
      }
      if match_found {
        let before = str_substring(s, 0, i)
        let after = str_substring(s, i + from_len, s_len)
        return before + to + after
      }
    }
    s
  }
}

// ==========================================
// 字符串大小写转换
// ==========================================

/// 转换为小写
function str_to_lowercase(s: String): String {
  let chars = str_chars(s)
  let mut result = ""
  for i in 0..list_len(chars) {
    let c = list_get(chars, i)
    if c >= 'A' && c <= 'Z' {
      result = result + char_to_string((c as Int + 32) as Char)
    } else {
      result = result + char_to_string(c)
    }
  }
  result
}

/// 转换为大写
function str_to_uppercase(s: String): String {
  let chars = str_chars(s)
  let mut result = ""
  for i in 0..list_len(chars) {
    let c = list_get(chars, i)
    if c >= 'a' && c <= 'z' {
      result = result + char_to_string((c as Int - 32) as Char)
    } else {
      result = result + char_to_string(c)
    }
  }
  result
}

/// 首字母大写
function str_capitalize(s: String): String {
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
function str_trim(s: String): String {
  str_trim_end(str_trim_start(s))
}

/// 去除开头空白
function str_trim_start(s: String): String {
  let chars = str_chars(s)
  let mut i = 0
  while i < list_len(chars) {
    let c = list_get(chars, i)
    if not char_is_whitespace(c) {
      break
    }
    i = i + 1
  }
  str_slice(s, i)
}

/// 去除结尾空白
function str_trim_end(s: String): String {
  let len = str_len(s)
  let chars = str_chars(s)
  let mut i = len - 1
  while i >= 0 {
    let c = list_get(chars, i)
    if not char_is_whitespace(c) {
      break
    }
    i = i - 1
  }
  str_substring(s, 0, i + 1)
}

/// 去除首尾指定字符
function str_trim_chars(s: String, chars: String): String {
  str_trim_start_chars(str_trim_end_chars(s, chars), chars)
}

/// 去除开头指定字符
function str_trim_start_chars(s: String, chars: String): String {
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
function str_trim_end_chars(s: String, chars: String): String {
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
function str_pad_left(s: String, width: Int, pad_char: Char): String {
  let len = str_len(s)
  if len >= width {
    s
  } else {
    str_repeat(char_to_string(pad_char), width - len) + s
  }
}

/// 右侧填充到指定长度
function str_pad_right(s: String, width: Int, pad_char: Char): String {
  let len = str_len(s)
  if len >= width {
    s
  } else {
    s + str_repeat(char_to_string(pad_char), width - len)
  }
}

/// 居中填充到指定长度
function str_center(s: String, width: Int, pad_char: Char): String {
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
function str_split(s: String, separator: String): [String] {
  let s_len = str_len(s)
  let sep_len = str_len(separator)
  
  if sep_len == 0 {
    let chars = str_chars(s)
    let mut result = []
    for i in 0..list_len(chars) {
      result = list_push(result, char_to_string(list_get(chars, i)))
    }
    result
  } else if sep_len > s_len {
    [s]
  } else {
    let mut result = []
    let mut start = 0
    let s_chars = str_chars(s)
    let sep_chars = str_chars(separator)
    
    while start <= s_len - sep_len {
      let mut match_found = true
      for i in 0..sep_len {
        if list_get(s_chars, start + i) != list_get(sep_chars, i) {
          match_found = false
          break
        }
      }
      
      if match_found {
        result = list_push(result, str_substring(s, 0, start))
        s = str_substring(s, start + sep_len, s_len)
        s_len = str_len(s)
        start = 0
      } else {
        start = start + 1
      }
    }
    
    result = list_push(result, s)
    result
  }
}

/// 按空白分割字符串
function str_split_whitespace(s: String): [String] {
  let trimmed = str_trim(s)
  if str_is_empty(trimmed) {
    []
  } else {
    let chars = str_chars(trimmed)
    let mut result = []
    let mut current = ""
    
    for i in 0..list_len(chars) {
      let c = list_get(chars, i)
      if char_is_whitespace(c) {
        if not str_is_empty(current) {
          result = list_push(result, current)
          current = ""
        }
      } else {
        current = current + char_to_string(c)
      }
    }
    
    if not str_is_empty(current) {
      result = list_push(result, current)
    }
    
    result
  }
}

/// 按行分割字符串
function str_lines(s: String): [String] {
  str_split(s, "\n")
}

// ==========================================
// 字符串解析
// ==========================================

/// 解析为整数
function str_parse_int(s: String): Option<Int> {
  let trimmed = str_trim(s)
  if str_is_empty(trimmed) {
    None()
  } else {
    let chars = str_chars(trimmed)
    let mut is_negative = false
    let mut start = 0
    
    if list_get(chars, 0) == '-' {
      is_negative = true
      start = 1
    } else if list_get(chars, 0) == '+' {
      start = 1
    }
    
    if start >= list_len(chars) {
      None()
    }
    
    let mut result = 0
    for i in start..list_len(chars) {
      let c = list_get(chars, i)
      if not char_is_digit(c) {
        return None()
      }
      result = result * 10 + (c as Int - '0' as Int)
    }
    
    if is_negative {
      Some(-result)
    } else {
      Some(result)
    }
  }
}

/// 解析为浮点数
function str_parse_float(s: String): Option<Float> {
  let trimmed = str_trim(s)
  if str_is_empty(trimmed) {
    None()
  } else {
    let chars = str_chars(trimmed)
    let mut is_negative = false
    let mut start = 0
    let mut has_decimal = false
    let mut decimal_places = 0
    
    if list_get(chars, 0) == '-' {
      is_negative = true
      start = 1
    } else if list_get(chars, 0) == '+' {
      start = 1
    }
    
    if start >= list_len(chars) {
      None()
    }
    
    let mut integer_part = 0
    let mut fractional_part = 0
    
    for i in start..list_len(chars) {
      let c = list_get(chars, i)
      if c == '.' {
        if has_decimal {
          return None()
        }
        has_decimal = true
      } else if char_is_digit(c) {
        if has_decimal {
          fractional_part = fractional_part * 10 + (c as Int - '0' as Int)
          decimal_places = decimal_places + 1
        } else {
          integer_part = integer_part * 10 + (c as Int - '0' as Int)
        }
      } else {
        return None()
      }
    }
    
    let mut result = integer_part as Float
    if has_decimal {
      let mut divisor = 1.0
      for _ in 0..decimal_places {
        divisor = divisor * 10.0
      }
      result = result + fractional_part as Float / divisor
    }
    
    if is_negative {
      Some(-result)
    } else {
      Some(result)
    }
  }
}

/// 解析为布尔值
function str_parse_bool(s: String): Option<Bool> {
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
function char_to_string(c: Char): String {
  // 简单实现，实际需要底层支持
  let code = c as Int
  if code < 0x80 {
    [c].join("")
  } else if code < 0x800 {
    let c1 = (0xC0 | (code >> 6)) as Char
    let c2 = (0x80 | (code & 0x3F)) as Char
    [c1, c2].join("")
  } else if code < 0x10000 {
    let c1 = (0xE0 | (code >> 12)) as Char
    let c2 = (0x80 | ((code >> 6) & 0x3F)) as Char
    let c3 = (0x80 | (code & 0x3F)) as Char
    [c1, c2, c3].join("")
  } else {
    let c1 = (0xF0 | (code >> 18)) as Char
    let c2 = (0x80 | ((code >> 12) & 0x3F)) as Char
    let c3 = (0x80 | ((code >> 6) & 0x3F)) as Char
    let c4 = (0x80 | (code & 0x3F)) as Char
    [c1, c2, c3, c4].join("")
  }
}

/// 数字转换为字符
function char_code(c: Char): Int {
  c as Int
}

/// 从字符码创建字符
function char_from_code(code: Int): Option<Char> {
  if code >= 0 && code <= 0x10FFFF {
    Some(code as Char)
  } else {
    None()
  }
}

// ==========================================
// 字符分类
// ==========================================

/// 检查字符是否是字母
function char_is_alpha(c: Char): Bool {
  (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

/// 检查字符是否是数字
function char_is_digit(c: Char): Bool {
  c >= '0' && c <= '9'
}

/// 检查字符是否是字母或数字
function char_is_alphanumeric(c: Char): Bool {
  char_is_alpha(c) || char_is_digit(c)
}

/// 检查字符是否是空白
function char_is_whitespace(c: Char): Bool {
  c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

/// 检查字符是否是小写字母
function char_is_lowercase(c: Char): Bool {
  c >= 'a' && c <= 'z'
}

/// 检查字符是否是大写字母
function char_is_uppercase(c: Char): Bool {
  c >= 'A' && c <= 'Z'
}

// ==========================================
// 字符串反转
// ==========================================

/// 反转字符串
function str_reverse(s: String): String {
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
function str_is_alpha(s: String): Bool {
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
function str_is_digit(s: String): Bool {
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
function str_is_alphanumeric(s: String): Bool {
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
function str_is_whitespace(s: String): Bool {
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
function format_int(n: Int, width: Int): String {
  str_pad_left(to_string(n), width, '0')
}

/// 将浮点数格式化为指定小数位数的字符串
function format_float(n: Float, decimals: Int): String {
  // 简单实现，实际需要更精确的处理
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

// ==========================================
// 辅助函数
// ==========================================

/// 计算两个数的最小值
function min_int(a: Int, b: Int): Int {
  if a < b { a } else { b }
}

/// 计算两个数的最大值
function max_int(a: Int, b: Int): Int {
  if a > b { a } else { b }
}
