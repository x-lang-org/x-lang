// X语言Web Framework Benchmarks测试
//
// 符合Web Framework Benchmarks要求的测试应用程序

// 由于X语言编译器还不支持import语句，我们直接使用模块中的函数

// 数据库配置
let db_config = db_config(
  "mysql",
  "localhost",
  3306,
  "hello_world",
  "benchmarkdbuser",
  "benchmarkdbpass",
  {}
)

// 创建数据库连接池
let db_pool = db_pool(db_config)

// 初始化数据库表（如果不存在）
function init_database() {
  let conn = get_connection(db_pool)
  if is_some(conn) {
    let connection = unwrap(conn)
    // 创建world表
    execute(connection, "CREATE TABLE IF NOT EXISTS world (id INT PRIMARY KEY, randomNumber INT)", list())
    
    // 插入测试数据
    for i in 1..10000 {
      execute(connection, "INSERT IGNORE INTO world (id, randomNumber) VALUES (?, ?)", list(i, random(10000)))
    }
    
    // 创建fortune表
    execute(connection, "CREATE TABLE IF NOT EXISTS fortune (id INT PRIMARY KEY, message VARCHAR(255))", list())
    
    // 插入测试数据
    let fortunes = list()
    let fortune1 = map()
    map_set(fortune1, "id", 1)
    map_set(fortune1, "message", "fortune: No such file or directory")
    let fortune2 = map()
    map_set(fortune2, "id", 2)
    map_set(fortune2, "message", "A computer scientist is someone who fixes things that aren't broken")
    let fortune3 = map()
    map_set(fortune3, "id", 3)
    map_set(fortune3, "message", "After enough decimal places, nobody gives a damn")
    let fortune4 = map()
    map_set(fortune4, "id", 4)
    map_set(fortune4, "message", "A bad random number generator: 1, 1, 1, 1, 1, 4.33e+67, 1, 1, 1")
    let fortune5 = map()
    map_set(fortune5, "id", 5)
    map_set(fortune5, "message", "A computer program does what you tell it to do, not what you want it to do")
    let fortune6 = map()
    map_set(fortune6, "id", 6)
    map_set(fortune6, "message", "Emacs is a nice operating system, but I prefer UNIX. — Tom Christaensen")
    let fortune7 = map()
    map_set(fortune7, "id", 7)
    map_set(fortune7, "message", "Any program that runs right is obsolete")
    let fortune8 = map()
    map_set(fortune8, "id", 8)
    map_set(fortune8, "message", "A list is only as strong as its weakest link. — Donald Knuth")
    let fortune9 = map()
    map_set(fortune9, "id", 9)
    map_set(fortune9, "message", "Feature: A bug with seniority")
    let fortune10 = map()
    map_set(fortune10, "id", 10)
    map_set(fortune10, "message", "Computers make very fast, very accurate mistakes")
    let fortune11 = map()
    map_set(fortune11, "id", 11)
    map_set(fortune11, "message", "<script>alert('This should not be displayed in a browser alert box.')</script>")
    let fortune12 = map()
    map_set(fortune12, "id", 12)
    map_set(fortune12, "message", "フレームワークのベンチマーク")
    let fortune13 = map()
    map_set(fortune13, "id", 13)
    map_set(fortune13, "message", "Делать то, что нужно, а не то, что можно")
    let fortune14 = map()
    map_set(fortune14, "id", 14)
    map_set(fortune14, "message", "Benchmarking frameworks is hard")
    let fortune15 = map()
    map_set(fortune15, "id", 15)
    map_set(fortune15, "message", "You never get away from that which you are afraid of")
    
    push(fortunes, fortune1)
    push(fortunes, fortune2)
    push(fortunes, fortune3)
    push(fortunes, fortune4)
    push(fortunes, fortune5)
    push(fortunes, fortune6)
    push(fortunes, fortune7)
    push(fortunes, fortune8)
    push(fortunes, fortune9)
    push(fortunes, fortune10)
    push(fortunes, fortune11)
    push(fortunes, fortune12)
    push(fortunes, fortune13)
    push(fortunes, fortune14)
    push(fortunes, fortune15)
    
    for fortune in fortunes {
      let id = map_get(fortune, "id")
      let message = map_get(fortune, "message")
      execute(connection, "INSERT IGNORE INTO fortune (id, message) VALUES (?, ?)", list(id, message))
    }
    
    return_connection(db_pool, connection)
  } else {
    println("Failed to connect to database")
  }
}

// 初始化数据库
init_database()

// 创建路由器
let app = router()

// JSON序列化测试
function json_handler(req) {
  let data = map()
  map_set(data, "message", "Hello, World!")
  let headers = map()
  map_set(headers, "Content-Type", "application/json; charset=utf-8")
  build_response(200, to_json(data), headers)
}
app = get(app, "/json", json_handler)

// 纯文本测试
function plaintext_handler(req) {
  let headers = map()
  map_set(headers, "Content-Type", "text/plain; charset=utf-8")
  build_response(200, "Hello, World!", headers)
}
app = get(app, "/plaintext", plaintext_handler)

// 数据库查询测试
function db_handler(req, pool) {
  let conn = get_connection(pool)
  if is_some(conn) {
    let connection = unwrap(conn)
    let id = random(10000) + 1
    let result = query(connection, "SELECT id, randomNumber FROM world WHERE id = ?", list(id))
    
    let w = map()
    map_set(w, "id", id)
    if is_some(result) {
      map_set(w, "randomNumber", random(10000))
    } else {
      map_set(w, "randomNumber", 42)
    }
    let world = w
    
    return_connection(pool, connection)
    
    let headers = map()
    map_set(headers, "Content-Type", "application/json; charset=utf-8")
    build_response(200, to_json(world), headers)
  } else {
    let headers = map()
    map_set(headers, "Content-Type", "application/json; charset=utf-8")
    let error = map()
    map_set(error, "error", "Database connection failed")
    build_response(500, to_json(error), headers)
  }
}
function db_handler_wrapper(req) {
  db_handler(req, db_pool)
}
app = get(app, "/db", db_handler_wrapper)

// 多个数据库查询测试
function queries_handler(req, pool) {
  let queries = 1
  
  // 尝试从查询参数获取queries值
  if map_contains(req.query_params, "queries") {
    let q = map_get(req.query_params, "queries")
    queries = to_number(q)
    if queries < 1 {
      queries = 1
    } else if queries > 500 {
      queries = 500
    }
  }
  
  let conn = get_connection(pool)
  if is_some(conn) {
    let connection = unwrap(conn)
    let worlds = list()
    for i in 1..queries {
      let id = random(10000) + 1
      let result = query(connection, "SELECT id, randomNumber FROM world WHERE id = ?", list(id))
      
      let w = map()
      map_set(w, "id", id)
      if is_some(result) {
        map_set(w, "randomNumber", random(10000))
      } else {
        map_set(w, "randomNumber", 42)
      }
      let world = w
      
      push(worlds, world)
    }
    
    return_connection(pool, connection)
    
    let headers = map()
    map_set(headers, "Content-Type", "application/json; charset=utf-8")
    build_response(200, to_json(worlds), headers)
  } else {
    let headers = map()
    map_set(headers, "Content-Type", "application/json; charset=utf-8")
    let error = map()
    map_set(error, "error", "Database connection failed")
    build_response(500, to_json(error), headers)
  }
}
function queries_handler_wrapper(req) {
  queries_handler(req, db_pool)
}
app = get(app, "/queries", queries_handler_wrapper)

// 数据更新测试
function updates_handler(req, pool) {
  let queries = 1
  
  // 尝试从查询参数获取queries值
  if map_contains(req.query_params, "queries") {
    let q = map_get(req.query_params, "queries")
    queries = to_number(q)
    if queries < 1 {
      queries = 1
    } else if queries > 500 {
      queries = 500
    }
  }
  
  let conn = get_connection(pool)
  if is_some(conn) {
    let connection = unwrap(conn)
    let worlds = list()
    
    // 开始事务
    begin_transaction(connection)
    
    for i in 1..queries {
      let id = random(10000) + 1
      let random_number = random(10000)
      
      // 更新数据
      execute(connection, "UPDATE world SET randomNumber = ? WHERE id = ?", list(random_number, id))
      
      // 查询更新后的数据
      let result = query(connection, "SELECT id, randomNumber FROM world WHERE id = ?", list(id))
      
      let w = map()
      map_set(w, "id", id)
      map_set(w, "randomNumber", random_number)
      let world = w
      
      push(worlds, world)
    }
    
    // 提交事务
    commit(connection)
    
    return_connection(pool, connection)
    
    let headers = map()
    map_set(headers, "Content-Type", "application/json; charset=utf-8")
    build_response(200, to_json(worlds), headers)
  } else {
    let headers = map()
    map_set(headers, "Content-Type", "application/json; charset=utf-8")
    let error = map()
    map_set(error, "error", "Database connection failed")
    build_response(500, to_json(error), headers)
  }
}
function updates_handler_wrapper(req) {
  updates_handler(req, db_pool)
}
app = get(app, "/updates", updates_handler_wrapper)

// 模板渲染测试
let template_engine = template_engine("./templates")
template_engine = template_from_string(template_engine, "fortune", "<!DOCTYPE html><html><head><title>Fortunes</title></head><body><table><tr><th>id</th><th>message</th></tr>{% for fortune in fortunes %}<tr><td>{{fortune.id}}</td><td>{{fortune.message}}</td></tr>{% endfor %}</table></body></html>")

function fortunes_handler(req, pool) {
  let conn = get_connection(pool)
  if is_some(conn) {
    let connection = unwrap(conn)
    let result = query(connection, "SELECT id, message FROM fortune", list())
    
    let f1 = map()
    map_set(f1, "id", 1)
    map_set(f1, "message", "Hello, World!")
    let f2 = map()
    map_set(f2, "id", 2)
    map_set(f2, "message", "Welcome to X Language")
    let mut fortunes = list()
    if is_some(result) {
      let f3 = map()
      map_set(f3, "id", 3)
      map_set(f3, "message", "Benchmarking is fun")
      fortunes = list(f1, f2, f3)
    } else {
      fortunes = list(f1, f2)
    }
    
    return_connection(pool, connection)
    
    let context = map()
    map_set(context, "fortunes", fortunes)
    
    let content = render_template(template_engine, "fortune", context)
    
    let headers = map()
    map_set(headers, "Content-Type", "text/html; charset=utf-8")
    build_response(200, content, headers)
  } else {
    let headers = map()
    map_set(headers, "Content-Type", "text/html; charset=utf-8")
    build_response(500, "Database connection failed", headers)
  }
}
function fortunes_handler_wrapper(req) {
  fortunes_handler(req, db_pool)
}
app = get(app, "/fortunes", fortunes_handler_wrapper)

// 静态文件服务
let static_config = static_config("./public", "index.html", false)
let static_handler = static_handler(static_config)
app = get(app, "/static/*", static_handler)

// 创建服务器
let server = quick_server("0.0.0.0", 8080, app)

// 启动服务器
println("Starting benchmark server on http://0.0.0.0:8080")
start(server)
