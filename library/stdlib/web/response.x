// X语言Web Framework - HTTP响应模块
//
// 提供HTTP响应的构建和生成功能（纯 X 语言实现）

import "string"
import "collections"

/// HTTP响应类型
type Response {
  status_code: Int,
  status_text: String,
  headers: Map<String, String>,
  body: String,
}

/// 构建HTTP响应
function response(status_code: Int, body: String): Response {
  let status_text = get_status_text(status_code)
  let mut headers = map_new()
  headers = map_insert(headers, "Content-Type", "text/plain; charset=utf-8")
  headers = map_insert(headers, "Content-Length", to_string(len(body)))

  {
    status_code: status_code,
    status_text: status_text,
    headers: headers,
    body: body,
  }
}

/// 构建带自定义头的响应
function response_with_headers(status_code: Int, body: String, headers: Map<String, String>): Response {
  let status_text = get_status_text(status_code)

  if !map_contains_key(headers, "Content-Type") {
    headers = map_insert(headers, "Content-Type", "text/plain; charset=utf-8")
  }
  headers = map_insert(headers, "Content-Length", to_string(len(body)))

  {
    status_code: status_code,
    status_text: status_text,
    headers: headers,
    body: body,
  }
}

/// 创建文本响应
function text(s: String): Response {
  let mut headers = map_new()
  headers = map_insert(headers, "Content-Type", "text/plain; charset=utf-8")
  response_with_headers(200, s, headers)
}

/// 创建HTML响应
function html(s: String): Response {
  let mut headers = map_new()
  headers = map_insert(headers, "Content-Type", "text/html; charset=utf-8")
  response_with_headers(200, s, headers)
}

/// 创建JSON响应
function json(data: String): Response {
  let mut headers = map_new()
  headers = map_insert(headers, "Content-Type", "application/json; charset=utf-8")
  response_with_headers(200, data, headers)
}

/// 创建成功响应
function ok(body: String): Response {
  response(200, body)
}

/// 创建创建成功响应
function created(body: String): Response {
  response(201, body)
}

/// 创建无内容响应
function no_content(): Response {
  response(204, "")
}

/// 创建错误请求响应
function bad_request(message: String): Response {
  response(400, message)
}

/// 创建未授权响应
function unauthorized(message: String): Response {
  response(401, message)
}

/// 创建禁止访问响应
function forbidden(message: String): Response {
  response(403, message)
}

/// 创建未找到响应
function not_found(message: String): Response {
  response(404, message)
}

/// 创建服务器错误响应
function internal_server_error(message: String): Response {
  response(500, message)
}

/// 重定向响应
function redirect(url: String): Response {
  let mut headers = map_new()
  headers = map_insert(headers, "Location", url)
  response_with_headers(302, "", headers)
}

/// 永久重定向响应
function redirect_permanent(url: String): Response {
  let mut headers = map_new()
  headers = map_insert(headers, "Location", url)
  response_with_headers(301, "", headers)
}

/// 生成HTTP响应字符串
function serialize_response(resp: Response): String {
  let mut result = "HTTP/1.1 " + to_string(resp.status_code) + " " + resp.status_text + "\r\n"

  let entries = map_entries(resp.headers)
  let mut i = 0
  while i < len(entries) {
    let entry = entries[i]
    result = result + entry.0 + ": " + entry.1 + "\r\n"
    i = i + 1
  }

  result = result + "\r\n"
  result = result + resp.body

  result
}

/// 设置响应头
function set_header(mut resp: Response, key: String, value: String): Response {
  resp.headers = map_insert(resp.headers, key, value)
  resp
}

/// 设置Content-Type
function set_content_type(mut resp: Response, content_type: String): Response {
  resp.headers = map_insert(resp.headers, "Content-Type", content_type)
  resp
}

/// 设置状态码
function set_status(mut resp: Response, status_code: Int): Response {
  resp.status_code = status_code
  resp.status_text = get_status_text(status_code)
  resp
}

/// 获取HTTP状态码对应的文本
function get_status_text(status_code: Int): String {
  if status_code == 200 { return "OK" }
  else if status_code == 201 { return "Created" }
  else if status_code == 202 { return "Accepted" }
  else if status_code == 204 { return "No Content" }
  else if status_code == 301 { return "Moved Permanently" }
  else if status_code == 302 { return "Found" }
  else if status_code == 303 { return "See Other" }
  else if status_code == 304 { return "Not Modified" }
  else if status_code == 307 { return "Temporary Redirect" }
  else if status_code == 308 { return "Permanent Redirect" }
  else if status_code == 400 { return "Bad Request" }
  else if status_code == 401 { return "Unauthorized" }
  else if status_code == 403 { return "Forbidden" }
  else if status_code == 404 { return "Not Found" }
  else if status_code == 405 { return "Method Not Allowed" }
  else if status_code == 406 { return "Not Acceptable" }
  else if status_code == 408 { return "Request Timeout" }
  else if status_code == 409 { return "Conflict" }
  else if status_code == 410 { return "Gone" }
  else if status_code == 411 { return "Length Required" }
  else if status_code == 413 { return "Payload Too Large" }
  else if status_code == 414 { return "URI Too Long" }
  else if status_code == 415 { return "Unsupported Media Type" }
  else if status_code == 429 { return "Too Many Requests" }
  else if status_code == 500 { return "Internal Server Error" }
  else if status_code == 501 { return "Not Implemented" }
  else if status_code == 502 { return "Bad Gateway" }
  else if status_code == 503 { return "Service Unavailable" }
  else if status_code == 504 { return "Gateway Timeout" }
  else { return "Unknown Status" }
}
