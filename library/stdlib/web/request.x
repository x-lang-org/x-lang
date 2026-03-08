// X语言Web Framework - HTTP请求模块
//
// 提供HTTP请求的解析和处理功能

import "../collections"

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
///
/// # 参数
/// - `raw_request`: 原始HTTP请求字符串
///
/// # 返回值
/// - `Option<Request>`: 解析成功返回Some(Request)，失败返回None
function parse_request(raw_request: String): Option<Request> {
  let lines = split(raw_request, "\r\n")
  if len(lines) == 0 {
    return None
  }

  // 解析请求行
  let request_line = lines[0]
  let parts = split(request_line, " ")
  if len(parts) != 3 {
    return None
  }

  let method = parts[0]
  let path_query = parts[1]
  let (path, query_params) = parse_path_and_query(path_query)

  // 解析请求头
  let mut headers = map()
  let mut i = 1
  while i < len(lines) && lines[i] != "" {
    let header_line = lines[i]
    let header_parts = split(header_line, ": ")
    if len(header_parts) == 2 {
      let key = header_parts[0]
      let value = header_parts[1]
      map_set(headers, key, value)
    }
    i = i + 1
  }

  // 解析请求体
  let mut body = ""
  if i + 1 < len(lines) {
    body = lines[i + 1]
  }

  // 创建Request对象
  let request = {
    method: method,
    path: path,
    query_params: query_params,
    headers: headers,
    body: body,
    remote_addr: "127.0.0.1", // 暂时硬编码
  }

  Some(request)
}

/// 解析路径和查询参数
function parse_path_and_query(path_query: String): (String, Map<String, String>) {
  let parts = split(path_query, "?")
  let path = parts[0]
  let mut query_params = map()

  if len(parts) > 1 {
    let query_string = parts[1]
    let query_pairs = split(query_string, "&")
    for pair in query_pairs {
      let key_value = split(pair, "=")
      if len(key_value) == 2 {
        let key = key_value[0]
        let value = key_value[1]
        map_set(query_params, key, value)
      }
    }
  }

  (path, query_params)
}
