// Server Configuration

/// 服务器配置
class ServerConfig {
    public var host: string
    public var port: integer
    public var max_body_size: integer

    new() {
        this.host = "127.0.0.1"
        this.port = 8080
        this.max_body_size = 1048576  // 1MB
    }

    new(host: string, port: integer) {
        this.host = host
        this.port = port
        this.max_body_size = 1048576
    }

    function with_host(host: string) -> ServerConfig {
        let config = ServerConfig.new()
        config.host = host
        config.port = this.port
        config.max_body_size = this.max_body_size
        config
    }

    function with_port(port: integer) -> ServerConfig {
        let config = ServerConfig.new()
        config.host = this.host
        config.port = port
        config.max_body_size = this.max_body_size
        config
    }

    function with_max_body_size(size: integer) -> ServerConfig {
        let config = ServerConfig.new()
        config.host = this.host
        config.port = this.port
        config.max_body_size = size
        config
    }
}
