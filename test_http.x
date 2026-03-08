// 测试HTTP模块

// 暂时不使用import语句，直接使用函数名

function test_parse_request() {
  println("=== 测试请求解析 ===")
  
  let raw_request = "GET /test?foo=bar&baz=qux HTTP/1.1\r\nHost: localhost:3000\r\nContent-Type: application/json\r\n\r\n{\"name\": \"test\"}"
  
  let request = parse_request(raw_request)
  
  match request {
    Some(req) => {
      println("方法: " + req.method)
      println("路径: " + req.path)
      println("查询参数: " + to_string(req.query_params))
      println("请求头: " + to_string(req.headers))
      println("请求体: " + req.body)
      println("远程地址: " + req.remote_addr)
    }
    None => {
      println("请求解析失败")
    }
  }
  
  println()
}

function test_build_response() {
  println("=== 测试响应构建 ===")
  
  let headers = map()
  map_set(headers, "Content-Type", "application/json")
  
  let response = build_response(200, "{\"message\": \"Hello, World!\"}", headers)
  
  println("状态码: " + to_string(response.status_code))
  println("状态文本: " + response.status_text)
  println("响应头: " + to_string(response.headers))
  println("响应体: " + response.body)
  
  let response_str = generate_response(response)
  println("生成的响应:")
  println(response_str)
  
  println()
}

function test_status_codes() {
  println("=== 测试状态码 ===")
  
  println("OK: " + to_string(OK))
  println("NOT_FOUND: " + to_string(NOT_FOUND))
  println("INTERNAL_SERVER_ERROR: " + to_string(INTERNAL_SERVER_ERROR))
  
  println()
}

function test_methods() {
  println("=== 测试HTTP方法 ===")
  
  println("GET: " + GET)
  println("POST: " + POST)
  println("PUT: " + PUT)
  println("DELETE: " + DELETE)
  
  println()
}

function main() {
  test_parse_request()
  test_build_response()
  test_status_codes()
  test_methods()
  println("所有测试完成!")
}

main()
