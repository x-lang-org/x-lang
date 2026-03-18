// HTTP Request - 包装 Zig std.http.Request

use zig::std.http.{Method, Headers}
use ./method.{Method, from_zig_method}

/// HTTP Request
class Request {
    public var method: Method
    public var path: string
    public var query: string
    public var headers: Headers
    public var body: string

    new(method: Method, path: string, query: string, headers: Headers, body: string) {
        this.method = method
        this.path = path
        this.query = query
        this.headers = headers
        this.body = body
    }
}

/// 从 Zig HTTP 请求构建 Request
function build_request(
    method: zig::std.http.Method,
    target: string,
    headers: zig::std.http.Headers,
    body: string
) -> Request {
    // 解析 target 为 path 和 query
    let path_query = parse_target(target)

    Request.new(
        from_zig_method(method),
        path_query.path,
        path_query.query,
        headers,
        body
    )
}

/// 解析 target (path?query)
function parse_target(target: string) -> PathQuery {
    let query_index = string_index_of(target, "?")

    if query_index < 0 {
        PathQuery.new(target, "")
    } else {
        PathQuery.new(
            string_substring(target, 0, query_index),
            string_substring(target, query_index + 1, string_length(target))
        )
    }
}

record PathQuery {
    path: string,
    query: string
}

extern function string_index_of(s: string, substr: string) -> integer
extern function string_substring(s: string, start: integer, end: integer) -> string
extern function string_length(s: string) -> integer
