// Route - 路由定义

use ../http/request.{Request}
use ../http/response.{Response}

/// 路由
class Route {
    public var method: string
    public var path: string
    public var handler: function(Request) -> Response

    new(method: string, path: string, handler: function(Request) -> Response) {
        this.method = method
        this.path = path
        this.handler = handler
    }
}
