// Router - 路由匹配和分发

use zig::std.http.{Method}
use ../http/request.{Request}
use ../http/response.{Response, not_found}
use ./route.{Route}

/// 路由器
class Router {
    private var routes: List<Route>

    new() {
        this.routes = []
    }

    new(routes: List<Route>) {
        this.routes = routes
    }

    /// 添加 GET 路由
    function get(path: string, handler: function(Request) -> Response) -> Router {
        Router.new(this.routes + [Route.new("GET", path, handler)])
    }

    /// 添加 POST 路由
    function post(path: string, handler: function(Request) -> Response) -> Router {
        Router.new(this.routes + [Route.new("POST", path, handler)])
    }

    /// 添加 PUT 路由
    function put(path: string, handler: function(Request) -> Response) -> Router {
        Router.new(this.routes + [Route.new("PUT", path, handler)])
    }

    /// 添加 DELETE 路由
    function delete(path: string, handler: function(Request) -> Response) -> Router {
        Router.new(this.routes + [Route.new("DELETE", path, handler)])
    }

    /// 分发请求
    function dispatch(request: Request) -> Response {
        match find_route(this.routes, request.method, request.path) {
            is Some(route) => route.handler(request),
            is None => not_found("Route not found: " + request.path)
        }
    }
}

/// 查找匹配的路由
function find_route(routes: List<Route>, method: string, path: string) -> Option<Route> {
    find_route_recursive(routes, method, path, 0)
}

function find_route_recursive(
    routes: List<Route>,
    method: string,
    path: string,
    index: integer
) -> Option<Route> {
    if index >= list_length(routes) {
        None
    } else {
        let route = list_get(routes, index)
        if route.method == method && path_matches(route.path, path) {
            Some(route)
        } else {
            find_route_recursive(routes, method, path, index + 1)
        }
    }
}

/// 检查路径是否匹配
function path_matches(pattern: string, path: string) -> boolean {
    if pattern == path {
        true
    } else if starts_with(pattern, "/:") && starts_with(path, "/") {
        // 参数路由匹配: /:id 匹配 /123
        let pattern_parts = split_path(pattern)
        let path_parts = split_path(path)

        if list_length(pattern_parts) != list_length(path_parts) {
            false
        } else {
            paths_match(pattern_parts, path_parts, 0)
        }
    } else {
        false
    }
}

function paths_match(pattern: List<string>, path: List<string>, index: integer) -> boolean {
    if index >= list_length(pattern) {
        true
    } else {
        let p = list_get_string(pattern, index)
        let v = list_get_string(path, index)

        if starts_with(p, ":") {
            // 参数部分匹配任意值
            paths_match(pattern, path, index + 1)
        } else if p == v {
            paths_match(pattern, path, index + 1)
        } else {
            false
        }
    }
}

extern function list_length(list: List) -> integer
extern function list_get(list: List, index: integer) -> Route
extern function list_get_string(list: List, index: integer) -> string
extern function starts_with(s: string, prefix: string) -> boolean
extern function split_path(path: string) -> List<string>
