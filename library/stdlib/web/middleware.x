// X语言Web Framework - 中间件模块
//
// 提供中间件的定义和处理功能

import "../collections"

/// 中间件类型
type Middleware = (Request, Next) -> Response

/// 下一个中间件函数类型
type Next = () -> Response

/// 中间件链类型
type MiddlewareChain = List<Middleware>

/// 创建中间件链
///
/// # 返回值
/// - `MiddlewareChain`: 新创建的中间件链
function middleware_chain(): MiddlewareChain {
  list()
}

/// 添加中间件到链
///
/// # 参数
/// - `chain`: 中间件链
/// - `middleware`: 中间件函数
///
/// # 返回值
/// - `MiddlewareChain`: 更新后的中间件链
function add_middleware(chain: MiddlewareChain, middleware: Middleware): MiddlewareChain {
  push(chain, middleware)
  chain
}

/// 执行中间件链
///
/// # 参数
/// - `chain`: 中间件链
/// - `request`: 请求对象
/// - `final_handler`: 最终的请求处理器
///
/// # 返回值
/// - `Response`: 响应对象
function execute_middleware(chain: MiddlewareChain, request: Request, final_handler: (Request) -> Response): Response {
  if len(chain) == 0 {
    return final_handler(request)
  }
  
  let middleware = chain[0]
  let remaining_chain = list()
  for i in 1..len(chain) {
    push(remaining_chain, chain[i])
  }
  
  let next = fn() {
    execute_middleware(remaining_chain, request, final_handler)
  }
  
  middleware(request, next)
}

/// 日志中间件
///
/// # 返回值
/// - `Middleware`: 日志中间件函数
function logger_middleware(): Middleware {
  fn(request: Request, next: Next): Response {
    let start = timestamp()
    let response = next()
    let end = timestamp()
    let duration = end - start
    
    println(request.method + " " + request.path + " " + to_string(response.status_code) + " " + to_string(duration) + "ms")
    
    response
  }
}

/// CORS中间件
///
/// # 返回值
/// - `Middleware`: CORS中间件函数
function cors_middleware(): Middleware {
  fn(request: Request, next: Next): Response {
    let response = next()
    
    // 添加CORS头
    map_set(response.headers, "Access-Control-Allow-Origin", "*")
    map_set(response.headers, "Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
    map_set(response.headers, "Access-Control-Allow-Headers", "Content-Type, Authorization")
    map_set(response.headers, "Access-Control-Max-Age", "86400")
    
    response
  }
}

/// 错误处理中间件
///
/// # 返回值
/// - `Middleware`: 错误处理中间件函数
function error_middleware(): Middleware {
  fn(request: Request, next: Next): Response {
    try {
      next()
    } catch (error) {
      let headers = map()
      map_set(headers, "Content-Type", "text/plain; charset=utf-8")
      
      build_response(500, "Internal Server Error: " + to_string(error), headers)
    }
  }
}

/// 认证中间件
///
/// # 参数
/// - `realm`: 认证域
///
/// # 返回值
/// - `Middleware`: 认证中间件函数
function auth_middleware(realm: String): Middleware {
  fn(request: Request, next: Next): Response {
    let auth_header = map_get(request.headers, "Authorization")
    
    match auth_header {
      Some(token) => {
        // 这里可以验证token
        // 简化实现，实际应该验证token的有效性
        next()
      }
      None => {
        let headers = map()
        map_set(headers, "Content-Type", "text/plain; charset=utf-8")
        map_set(headers, "WWW-Authenticate", "Basic realm=\"" + realm + "\"")
        
        build_response(401, "Unauthorized", headers)
      }
    }
  }
}

/// 压缩中间件
///
/// # 返回值
/// - `Middleware`: 压缩中间件函数
function compression_middleware(): Middleware {
  fn(request: Request, next: Next): Response {
    let response = next()
    
    // 检查请求头是否支持压缩
    let accept_encoding = map_get(request.headers, "Accept-Encoding")
    
    match accept_encoding {
      Some(encoding) => {
        if str_contains(encoding, "gzip") {
          map_set(response.headers, "Content-Encoding", "gzip")
          // 实际实现应该压缩响应体
        } else if str_contains(encoding, "deflate") {
          map_set(response.headers, "Content-Encoding", "deflate")
          // 实际实现应该压缩响应体
        }
      }
      None => {}
    }
    
    response
  }
}

/// 静态文件中间件
///
/// # 参数
/// - `root`: 静态文件根目录
///
/// # 返回值
/// - `Middleware`: 静态文件中间件函数
function static_middleware(root: String): Middleware {
  fn(request: Request, next: Next): Response {
    // 检查请求路径是否指向静态文件
    let path = request.path
    
    // 简化实现，实际应该检查文件是否存在
    if str_starts_with(path, "/static/") {
      let headers = map()
      map_set(headers, "Content-Type", "text/plain; charset=utf-8")
      
      build_response(200, "Static file: " + path, headers)
    } else {
      next()
    }
  }
}

/// 速率限制中间件
///
/// # 参数
/// - `max_requests`: 最大请求数
/// - `window_ms`: 时间窗口（毫秒）
///
/// # 返回值
/// - `Middleware`: 速率限制中间件函数
function rate_limit_middleware(max_requests: Int, window_ms: Int): Middleware {
  // 简化实现，实际应该使用更复杂的速率限制算法
  let requests = map()
  
  fn(request: Request, next: Next): Response {
    let ip = request.ip // 假设Request对象有ip字段
    let now = timestamp()
    
    let user_requests = map_get(requests, ip)
    let current_requests = match user_requests {
      Some(list) => {
        // 过滤掉时间窗口外的请求
        let filtered = list_filter(list, fn(t) {
          now - t < window_ms
        })
        filtered
      }
      None => list()
    }
    
    if len(current_requests) >= max_requests {
      let headers = map()
      map_set(headers, "Content-Type", "text/plain; charset=utf-8")
      map_set(headers, "Retry-After", to_string(window_ms / 1000))
      
      build_response(429, "Too Many Requests", headers)
    } else {
      push(current_requests, now)
      map_set(requests, ip, current_requests)
      next()
    }
  }
}

