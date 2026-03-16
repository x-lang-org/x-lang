// X语言Web Framework - HTTP请求模块
//
// 提供HTTP请求的解析和处理功能（纯 X 语言实现）

import "string"
import "collections"

/// HTTP请求类型
type Request {
  method: String,
  path: String,
  query_params: Map<String, String>,
  headers: Map<String, String>,
  body: String,
  remote_addr: String,
}

/// 解析HTTP请求
function parse_request(raw_request: String): Option<Request> {
  let lines = str_split(raw_request, "\r\n")
  if len(lines) == 0 {
    return None
  }

  let request_line = lines[0]
  let parts = str_split(request_line, " ")
  if len(parts) != 3 {
    return None
  }

  let method = parts[0]
  let path_query = parts[1]
  let result = parse_path_and_query(path_query)
  let path = result.0
  let query_params = result.1

  let mut headers = map_new()
  let mut i = 1
  while i < len(lines) && lines[i] != "" {
    let header_line = lines[i]
    if str_contains(header_line, ": ") {
      let header_parts = str_split(header_line, ": ")
      if len(header_parts) >= 2 {
        let key = str_to_lowercase(header_parts[0])
        let value = header_parts[1]
        headers = map_insert(headers, key, value)
      }
    }
    i = i + 1
  }

  let mut body = ""
  let body_start = i + 1
  if body_start < len(lines) {
    let body_parts: [String] = []
    let mut j = body_start
    while j < len(lines) {
      body_parts = list_push(body_parts, lines[j])
      j = j + 1
    }
    body = str_join(body_parts, "\r\n")
  }

  Some({
    method: method,
    path: path,
    query_params: query_params,
    headers: headers,
    body: body,
    remote_addr: "127.0.0.1",
  })
}

/// 解析路径和查询参数
function parse_path_and_query(path_query: String): (String, Map<String, String>) {
  let parts = str_split(path_query, "?")
  let path = url_decode(parts[0])
  let mut query_params = map_new()

  if len(parts) > 1 {
    let query_string = parts[1]
    let query_pairs = str_split(query_string, "&")
    let mut i = 0
    while i < len(query_pairs) {
      let pair = query_pairs[i]
      let key_value = str_split(pair, "=")
      if len(key_value) == 2 {
        query_params = map_insert(query_params, url_decode(key_value[0]), url_decode(key_value[1]))
      } else if len(key_value) == 1 {
        query_params = map_insert(query_params, url_decode(key_value[0]), "")
      }
      i = i + 1
    }
  }

  (path, query_params)
}

/// URL解码
function url_decode(s: String): String {
  let mut result = ""
  let mut i = 0
  let length = len(s)

  while i < length {
    let c = str_substring(s, i, i + 1)
    if c == "%" && i + 2 < length {
      let hex = str_substring(s, i + 1, i + 3)
      result = result + hex_to_char(hex)
      i = i + 3
    } else if c == "+" {
      result = result + " "
      i = i + 1
    } else {
      result = result + c
      i = i + 1
    }
  }

  result
}

/// 将十六进制字符串转换为字符
function hex_to_char(hex: String): String {
  let v1 = hex_digit_to_int(str_substring(hex, 0, 1))
  let v2 = hex_digit_to_int(str_substring(hex, 1, 2))
  let code = v1 * 16 + v2

  // 简化实现 - 返回原字符或常见字符
  if code == 32 { return " " }
  else if code >= 48 && code <= 57 {
    // 数字 0-9
    let digits = "0123456789"
    return str_substring(digits, code - 48, code - 47)
  } else if code >= 65 && code <= 90 {
    // 大写字母 A-Z
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    return str_substring(letters, code - 65, code - 64)
  } else if code >= 97 && code <= 122 {
    // 小写字母 a-z
    let letters = "abcdefghijklmnopqrstuvwxyz"
    return str_substring(letters, code - 97, code - 96)
  }
  hex
}

/// 将十六进制数字字符转换为整数
function hex_digit_to_int(c: String): Int {
  if c == "0" { return 0 }
  else if c == "1" { return 1 }
  else if c == "2" { return 2 }
  else if c == "3" { return 3 }
  else if c == "4" { return 4 }
  else if c == "5" { return 5 }
  else if c == "6" { return 6 }
  else if c == "7" { return 7 }
  else if c == "8" { return 8 }
  else if c == "9" { return 9 }
  else if c == "a" || c == "A" { return 10 }
  else if c == "b" || c == "B" { return 11 }
  else if c == "c" || c == "C" { return 12 }
  else if c == "d" || c == "D" { return 13 }
  else if c == "e" || c == "E" { return 14 }
  else if c == "f" || c == "F" { return 15 }
  else { return 0 }
}

/// 解析查询字符串
function parse_query_string(qs: String): Map<String, String> {
  let mut params = map_new()
  let pairs = str_split(qs, "&")
  let mut i = 0
  while i < len(pairs) {
    let pair = pairs[i]
    let kv = str_split(pair, "=")
    if len(kv) == 2 {
      params = map_insert(params, url_decode(kv[0]), url_decode(kv[1]))
    } else if len(kv) == 1 {
      params = map_insert(params, url_decode(kv[0]), "")
    }
    i = i + 1
  }
  params
}

/// 获取请求头
function get_header(req: Request, name: String): Option<String> {
  map_get(req.headers, str_to_lowercase(name))
}

/// 获取查询参数
function get_query_param(req: Request, name: String): Option<String> {
  map_get(req.query_params, name)
}

/// 检查是否为GET请求
function is_get(req: Request): Bool {
  req.method == "GET"
}

/// 检查是否为POST请求
function is_post(req: Request): Bool {
  req.method == "POST"
}

/// 检查是否为JSON请求
function is_json(req: Request): Bool {
  given get_header(req, "content-type") {
    is Some(ct) => str_contains(str_to_lowercase(ct), "application/json")
    is None => false
  }
}
