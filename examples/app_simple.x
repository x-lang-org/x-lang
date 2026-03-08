// X语言Web Framework简化示例应用
//
// 展示如何使用Web Framework构建API服务器

// 简化版示例，不使用import语句

// 创建一个简单的HTTP服务器
function handle_request(request) {
  // 模拟解析请求
  let method = "GET"
  let path = "/"
  
  // 模拟路由处理
  if path == "/" {
    return {
      status_code: 200,
      status_text: "OK",
      headers: { "Content-Type": "text/plain; charset=utf-8" },
      body: "Hello, X Language Web Framework!"
    }
  } else if path == "/api/users" {
    return {
      status_code: 200,
      status_text: "OK",
      headers: { "Content-Type": "application/json; charset=utf-8" },
      body: "[{\"id\": 1, \"name\": \"Alice\"}, {\"id\": 2, \"name\": \"Bob\"}]"
    }
  } else {
    return {
      status_code: 404,
      status_text: "Not Found",
      headers: { "Content-Type": "text/plain; charset=utf-8" },
      body: "Not Found"
    }
  }
}

// 模拟服务器运行
println("Starting server on http://localhost:3000")
println("Server started")

// 模拟处理请求
let request = {}
let response = handle_request(request)
println("Response:")
println("Status: " + response.status_code + " " + response.status_text)
println("Body: " + response.body)
