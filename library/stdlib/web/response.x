// X语言Web Framework - HTTP响应模块
//
// 提供HTTP响应的构建和生成功能

import "../collections"

/// HTTP响应类型
type Response {
  status_code: Int,
  status_text: String,
  headers: Map<String, String>,
  body: String,
}

/// 构建HTTP响应
///
/// # 参数
/// - `status_code`: HTTP状态码
/// - `body`: 响应体
/// - `headers`: 响应头
///
/// # 返回值
/// - `Response`: 构建好的响应对象
function build_response(status_code: Int, body: String, headers: Map<String, String>): Response {
  let status_text = get_status_text(status_code)
  
  // 添加默认的Content-Type头
  if !map_contains(headers, "Content-Type") {
    map_set(headers, "Content-Type", "text/plain; charset=utf-8")
  }
  
  // 添加Content-Length头
  map_set(headers, "Content-Length", to_string(len(body)))
  
  {
    status_code: status_code,
    status_text: status_text,
    headers: headers,
    body: body,
  }
}

/// 生成HTTP响应字符串
///
/// # 参数
/// - `response`: 响应对象
///
/// # 返回值
/// - `String`: 生成的HTTP响应字符串
function generate_response(response: Response): String {
  let mut result = "HTTP/1.1 " + to_string(response.status_code) + " " + response.status_text + "\r\n"
  
  // 添加响应头
  for (key, value) in entries(response.headers) {
    result = result + key + ": " + value + "\r\n"
  }
  
  // 添加空行
  result = result + "\r\n"
  
  // 添加响应体
  result = result + response.body
  
  result
}

/// 获取HTTP状态码对应的文本
function get_status_text(status_code: Int): String {
  match status_code {
    200 => "OK",
    201 => "Created",
    202 => "Accepted",
    204 => "No Content",
    301 => "Moved Permanently",
    302 => "Found",
    303 => "See Other",
    304 => "Not Modified",
    307 => "Temporary Redirect",
    308 => "Permanent Redirect",
    400 => "Bad Request",
    401 => "Unauthorized",
    403 => "Forbidden",
    404 => "Not Found",
    405 => "Method Not Allowed",
    406 => "Not Acceptable",
    408 => "Request Timeout",
    409 => "Conflict",
    410 => "Gone",
    411 => "Length Required",
    413 => "Payload Too Large",
    414 => "URI Too Long",
    415 => "Unsupported Media Type",
    429 => "Too Many Requests",
    500 => "Internal Server Error",
    501 => "Not Implemented",
    502 => "Bad Gateway",
    503 => "Service Unavailable",
    504 => "Gateway Timeout",
    _ => "Unknown Status"
  }
}
