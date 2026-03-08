// X语言Web Framework示例应用
//
// 展示如何使用Web Framework构建API服务器

// 由于X语言编译器还不支持import语句，我们直接使用相对路径引用模块
// import "../library/stdlib/web/http"

// 创建路由器
let app = router()

// 静态文件服务配置
let static_config = static_config("./public", "index.html", true)
let static_handler = static_handler(static_config)

// 注册静态文件服务路由
app = get(app, "/static/*", static_handler)

// 注册API路由
app = get(app, "/", fn(req: Request): Response {
  let headers = map()
  map_set(headers, "Content-Type", "text/plain; charset=utf-8")
  build_response(200, "Hello, X Language Web Framework!", headers)
})

app = get(app, "/api/users", fn(req: Request): Response {
  let users = list()
  push(users, map("id" => 1, "name" => "Alice"))
  push(users, map("id" => 2, "name" => "Bob"))
  push(users, map("id" => 3, "name" => "Charlie"))
  
  let headers = map()
  map_set(headers, "Content-Type", "application/json; charset=utf-8")
  build_response(200, to_json(users), headers)
})

app = get(app, "/api/users/:id", fn(req: Request): Response {
  // 这里应该从路由参数中获取id
  let id = "1" // 模拟
  let user = map("id" => id, "name" => "Alice")
  
  let headers = map()
  map_set(headers, "Content-Type", "application/json; charset=utf-8")
  build_response(200, to_json(user), headers)
})

app = post(app, "/api/users", fn(req: Request): Response {
  // 这里应该解析请求体
  let user = map("id" => 4, "name" => "David")
  
  let headers = map()
  map_set(headers, "Content-Type", "application/json; charset=utf-8")
  build_response(201, to_json(user), headers)
})

// 模板引擎示例
let template_engine = template_engine("./templates")
template_engine = load_template(template_engine, "index", "index.html")

template_engine = template_from_string(template_engine, "hello", "<h1>Hello, {{name}}!</h1>")

app = get(app, "/template", fn(req: Request): Response {
  let context = map()
  map_set(context, "name", "X Language")
  map_set(context, "users", list("Alice", "Bob", "Charlie"))
  map_set(context, "show_users", true)
  
  let content = render_template(template_engine, "hello", context)
  
  let headers = map()
  map_set(headers, "Content-Type", "text/html; charset=utf-8")
  build_response(200, content, headers)
})

// 数据库示例
app = get(app, "/db", fn(req: Request): Response {
  let db_config = db_config("mysql", "localhost", 3306, "test", "root", "", map())
  let conn = connect(db_config)
  
  match conn {
    Some(connection) => {
      let result = query(connection, "SELECT * FROM users", list())
      disconnect(connection)
      
      let headers = map()
      map_set(headers, "Content-Type", "text/plain; charset=utf-8")
      build_response(200, "Database query executed", headers)
    }
    None => {
      let headers = map()
      map_set(headers, "Content-Type", "text/plain; charset=utf-8")
      build_response(500, "Database connection failed", headers)
    }
  }
})

// 配置示例
app = get(app, "/config", fn(req: Request): Response {
  let config = load_config_from_env()
  let port = get_config(config, "PORT", 3000)
  let host = get_config(config, "HOST", "localhost")
  
  let content = "Host: " + host + ", Port: " + to_string(port)
  
  let headers = map()
  map_set(headers, "Content-Type", "text/plain; charset=utf-8")
  build_response(200, content, headers)
})

// 日志示例
let logger = default_logger()

app = get(app, "/log", fn(req: Request): Response {
  debug(logger, "Debug message")
  info(logger, "Info message")
  warn(logger, "Warn message")
  error(logger, "Error message")
  
  let headers = map()
  map_set(headers, "Content-Type", "text/plain; charset=utf-8")
  build_response(200, "Logs written", headers)
})

// 创建服务器
let server = quick_server("localhost", 3000, app)

// 启动服务器
println("Starting server on http://localhost:3000")
start(server)
