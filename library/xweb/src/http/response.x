// HTTP Response - 包装 Zig std.http.Response

use zig::std.http.{Status, Headers}
use ./status.{Status, to_zig_status}

/// HTTP Response
class Response {
    public var status: Status
    public var headers: Map<string, string>
    public var body: string

    new(status: Status, headers: Map<string, string>, body: string) {
        this.status = status
        this.headers = headers
        this.body = body
    }

    /// 设置状态码
    function with_status(status: Status) -> Response {
        Response.new(status, this.headers, this.body)
    }

    /// 添加 header
    function header(key: string, value: string) -> Response {
        let new_headers = map_insert(this.headers, key, value)
        Response.new(this.status, new_headers, this.body)
    }

    /// 设置 Content-Type
    function content_type(content_type: string) -> Response {
        this.header("Content-Type", content_type)
    }

    /// 设置 body
    function with_body(body: string) -> Response {
        Response.new(this.status, this.headers, body)
    }
}

// ========================================
// 响应构造器
// ========================================

/// 创建空响应
function new_response() -> Response {
    Response.new(Status.Ok, map_new(), "")
}

/// HTML 响应
function html(content: string) -> Response {
    Response.new(
        Status.Ok,
        map_insert(map_new(), "Content-Type", "text/html; charset=utf-8"),
        content
    )
}

/// JSON 响应
function json(content: string) -> Response {
    Response.new(
        Status.Ok,
        map_insert(map_new(), "Content-Type", "application/json"),
        content
    )
}

/// 文本响应
function text(content: string) -> Response {
    Response.new(
        Status.Ok,
        map_insert(map_new(), "Content-Type", "text/plain; charset=utf-8"),
        content
    )
}

/// 200 OK
function ok(body: string) -> Response {
    Response.new(Status.Ok, map_new(), body)
}

/// 201 Created
function created(body: string) -> Response {
    Response.new(Status.Created, map_new(), body)
}

/// 204 No Content
function no_content() -> Response {
    Response.new(Status.NoContent, map_new(), "")
}

/// 400 Bad Request
function bad_request(message: string) -> Response {
    Response.new(Status.BadRequest, map_new(), message)
}

/// 404 Not Found
function not_found(message: string) -> Response {
    Response.new(Status.NotFound, map_new(), message)
}

/// 500 Internal Server Error
function internal_server_error(message: string) -> Response {
    Response.new(Status.InternalServerError, map_new(), message)
}

// ========================================
// IntoResponse Trait
// ========================================

trait IntoResponse {
    function into_response(self) -> Response
}

impl IntoResponse for Response {
    function into_response(self) -> Response {
        self
    }
}

impl IntoResponse for string {
    function into_response(self) -> Response {
        text(self)
    }
}

// ========================================
// 辅助函数
// ========================================

extern function map_new() -> Map<string, string>
extern function map_insert(map: Map, key: string, value: string) -> Map
