// HTTP Server - 使用 Zig std.http

use zig::std.http.{Server, Request, Response, Method, Header}
use zig::std.net.{Address}
use ./config.{ServerConfig}
use ../http/request.{Request, build_request}
use ../http/response.{Response, to_zig_response}
use ../routing/router.{Router}

/// HTTP 服务器
class Server {
    private var config: ServerConfig
    private var router: Router

    new() {
        this.config = ServerConfig.new()
        this.router = Router.new()
    }

    new(config: ServerConfig) {
        this.config = config
        this.router = Router.new()
    }

    /// 设置端口
    function port(port: integer) -> Server {
        this.config = this.config.with_port(port)
        this
    }

    /// 设置主机
    function host(host: string) -> Server {
        this.config = this.config.with_host(host)
        this
    }

    /// 设置路由器
    function router(router: Router) -> Server {
        this.router = router
        this
    }

    /// 添加 GET 路由
    function get(path: string, handler: function(Request) -> Response) -> Server {
        this.router = this.router.get(path, handler)
        this
    }

    /// 添加 POST 路由
    function post(path: string, handler: function(Request) -> Response) -> Server {
        this.router = this.router.post(path, handler)
        this
    }

    /// 添加 PUT 路由
    function put(path: string, handler: function(Request) -> Response) -> Server {
        this.router = this.router.put(path, handler)
        this
    }

    /// 添加 DELETE 路由
    function delete(path: string, handler: function(Request) -> Response) -> Server {
        this.router = this.router.delete(path, handler)
        this
    }

    /// 启动服务器
    async function start() -> Result<Unit, string> {
        // 解析地址
        let address = Address.parseIp(this.config.host, this.config.port)

        // 创建服务器
        let server = Server.init(address, .{
            .reuse_address = true
        })

        println("X-Web server listening on http://" + this.config.host + ":" + int_to_string(this.config.port))

        // 主循环
        loop {
            // 等待连接
            wait run_server_loop(server, this.router, this.config)
        }
    }
}

/// 服务器主循环
async function run_server_loop(
    server: zig::std.http.Server,
    router: Router,
    config: ServerConfig
) {
    // 接受请求
    let response = wait server.wait()

    // 读取请求体
    let body = response.request.reader().readAllAlloc(
        allocator,
        config.max_body_size
    )

    // 构建 X-Web Request
    let request = build_request(
        response.request.method,
        response.request.target,
        response.request.headers,
        body
    )

    // 路由分发
    let result = router.dispatch(request)

    // 写入响应
    response.status = to_status_code(result.status)

    // 写入 headers
    let header_it = result.headers.iterator()
    while header_it.next() |entry| {
        response.headers.append(entry.key, entry.value)
    }

    // 写入 body
    response.writer().writeAll(result.body)

    // 完成响应
    response.finish()
}

/// 将 Status 转换为 Zig HTTP 状态码
function to_status_code(status: Status) -> zig::std.http.Status {
    match status {
        is Status.Ok => .ok,
        is Status.Created => .created,
        is Status.NoContent => .no_content,
        is Status.BadRequest => .bad_request,
        is Status.NotFound => .not_found,
        is Status.InternalServerError => .internal_server_error,
        is _ => .custom
    }
}

extern function println(message: string)
extern function int_to_string(n: integer) -> string
