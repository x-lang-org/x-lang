// X语言Web Framework最小示例应用
//
// 展示如何使用Web Framework构建API服务器

// 最小化示例，只使用基本的函数和变量

// 创建一个简单的HTTP服务器
function handle_request() {
  // 模拟解析请求
  let method = "GET"
  let path = "/"
  
  // 模拟路由处理
  if path == "/" {
    println("Status: 200 OK")
    println("Content-Type: text/plain; charset=utf-8")
    println("")
    println("Hello, X Language Web Framework!")
  } else if path == "/api/users" {
    println("Status: 200 OK")
    println("Content-Type: application/json; charset=utf-8")
    println("")
    println("[{\"id\": 1, \"name\": \"Alice\"}, {\"id\": 2, \"name\": \"Bob\"}]")
  } else {
    println("Status: 404 Not Found")
    println("Content-Type: text/plain; charset=utf-8")
    println("")
    println("Not Found")
  }
}

// 模拟服务器运行
println("Starting server on http://localhost:3000")
println("Server started")

// 模拟处理请求
handle_request()
