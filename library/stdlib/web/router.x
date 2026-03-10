// X语言Web Framework - 路由模块
//
// 提供HTTP路由的匹配和处理功能

import "../collections"

/// 路由处理器类型
type Handler = (Request) -> Response

/// 路由类型
type Route {
  method: String,
  path: String,
  handler: Handler,
  params: List<String>,
  pattern: String,
}

/// 路由器类型
type Router {
  routes: List<Route>,
}

/// 创建路由器
///
/// # 返回值
/// - `Router`: 新创建的路由器对象
function router(): Router {
  {
    routes: list(),
  }
}

/// 添加GET路由
///
/// # 参数
/// - `router`: 路由器对象
/// - `path`: 路由路径
/// - `handler`: 路由处理器
///
/// # 返回值
/// - `Router`: 更新后的路由器对象
function get(router: Router, path: String, handler: Handler): Router {
  add_route(router, "GET", path, handler)
}

/// 添加POST路由
///
/// # 参数
/// - `router`: 路由器对象
/// - `path`: 路由路径
/// - `handler`: 路由处理器
///
/// # 返回值
/// - `Router`: 更新后的路由器对象
function post(router: Router, path: String, handler: Handler): Router {
  add_route(router, "POST", path, handler)
}

/// 添加PUT路由
///
/// # 参数
/// - `router`: 路由器对象
/// - `path`: 路由路径
/// - `handler`: 路由处理器
///
/// # 返回值
/// - `Router`: 更新后的路由器对象
function put(router: Router, path: String, handler: Handler): Router {
  add_route(router, "PUT", path, handler)
}

/// 添加DELETE路由
///
/// # 参数
/// - `router`: 路由器对象
/// - `path`: 路由路径
/// - `handler`: 路由处理器
///
/// # 返回值
/// - `Router`: 更新后的路由器对象
function delete(router: Router, path: String, handler: Handler): Router {
  add_route(router, "DELETE", path, handler)
}

/// 添加路由
///
/// # 参数
/// - `router`: 路由器对象
/// - `method`: HTTP方法
/// - `path`: 路由路径
/// - `handler`: 路由处理器
///
/// # 返回值
/// - `Router`: 更新后的路由器对象
function add_route(router: Router, method: String, path: String, handler: Handler): Router {
  let (params, pattern) = parse_path(path)
  let route = {
    method: method,
    path: path,
    handler: handler,
    params: params,
    pattern: pattern,
  }
  push(router.routes, route)
  router
}

/// 解析路径，提取参数和生成匹配模式
///
/// # 参数
/// - `path`: 路由路径
///
/// # 返回值
/// - `(List<String>, String)`: 参数列表和匹配模式
function parse_path(path: String): (List<String>, String) {
  let parts = split(path, "/")
  let mut params = list()
  let mut pattern = ""
  
  for part in parts {
    if starts_with(part, ":") {
      let param_name = substring(part, 1)
      push(params, param_name)
      pattern = pattern + "/([^/]+)"
    } else {
      pattern = pattern + "/" + part
    }
  }
  
  (params, pattern)
}

/// 匹配路由
///
/// # 参数
/// - `router`: 路由器对象
/// - `method`: HTTP方法
/// - `path`: 请求路径
///
/// # 返回值
/// - `Option<(Handler, Map<String, String>)>`: 匹配成功返回Some(handler, params)，失败返回None
function match_route(router: Router, method: String, path: String): Option<(Handler, Map<String, String>)> {
  for route in router.routes {
    if route.method != method {
      continue
    }
    
    let (matched, params) = match_path(route, path)
    if matched {
      return Some((route.handler, params))
    }
  }
  
  None
}

/// 匹配路径
///
/// # 参数
/// - `route`: 路由对象
/// - `path`: 请求路径
///
/// # 返回值
/// - `(Bool, Map<String, String>)`: 是否匹配成功和提取的参数
function match_path(route: Route, path: String): (Bool, Map<String, String>) {
  let route_parts = split(route.path, "/")
  let path_parts = split(path, "/")
  
  if len(route_parts) != len(path_parts) {
    return (false, map())
  }
  
  let params = map()
  for i in 0..len(route_parts) {
    let route_part = route_parts[i]
    let path_part = path_parts[i]
    
    if starts_with(route_part, ":") {
      let param_name = substring(route_part, 1)
      map_set(params, param_name, path_part)
    } else if route_part != path_part {
      return (false, map())
    }
  }
  
  (true, params)
}
