// 测试HTTP模块

function test_http_basics() {
  println("=== 测试HTTP基础功能 ===")
  
  // 测试请求解析
  let raw_request = "GET /test?foo=bar&baz=qux HTTP/1.1\r\nHost: localhost:3000\r\nContent-Type: application/json\r\n\r\n{\"name\": \"test\"}"
  
  println("原始请求:")
  println(raw_request)
  println()
  
  // 测试响应构建
  let response = {
    status_code: 200,
    status_text: "OK",
    headers: {
      "Content-Type": "application/json",
      "Content-Length": "27"
    },
    body: "{\"message\": \"Hello, World!\"}"
  }
  
  println("响应对象:")
  println("状态码: " + to_string(response.status_code))
  println("状态文本: " + response.status_text)
  println("响应头: " + to_string(response.headers))
  println("响应体: " + response.body)
  
  println()
  println("测试完成!")
}

function main() {
  test_http_basics()
}

main()
