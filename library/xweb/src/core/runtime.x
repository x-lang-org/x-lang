// X-Web Runtime - Direct Zig stdlib usage
// 使用 zig:: 前缀直接导入 Zig 标准库

use zig::std.http.{Server, Request, Response, Method}
use zig::std.json.{parseFromSlice, Value, stringify}
use zig::std.net.{Address}
use zig::std.io.{Reader, Writer}

// ========================================
// HTTP Server Types (Zig std.http 包装)
// ========================================

/// HTTP 服务器
class HttpServer {
    private var address: Address
    private var port: integer

    new(host: string, port: integer) {
        this.address = Address.parseIp(host, port)
        this.port = port
    }

    /// 启动服务器
    async function start(handler: function(Request) -> Response) -> Result<Unit, string> {
        let server = Server.init(this.address, .{})

        println("X-Web server listening on http://127.0.0.1:" + int_to_string(this.port))

        // 主循环
        loop {
            let response = wait server.wait()
            handle_request(response, handler)
        }
    }
}

/// 处理单个请求
function handle_request(response: Server.Response, handler: function(Request) -> Response) {
    // 读取请求
    let request = response.wait()
    let body = request.reader().readAllAlloc(allocator, 1024 * 1024)

    // 调用用户处理器
    let result = handler(request)

    // 发送响应
    response.status = result.status
    for result.headers {
        response.headers.append(key, value)
    }
    response.writer().writeAll(result.body)
    response.finish()
}

// ========================================
// 便捷函数
// ========================================

/// 创建 HTTP 服务器
function create_server(host: string, port: integer) -> HttpServer {
    HttpServer.new(host, port)
}

/// 解析地址
function parse_address(host: string, port: integer) -> Result<Address, string> {
    Address.parseIp(host, port)
}

extern function println(message: string)
extern function int_to_string(n: integer) -> string
