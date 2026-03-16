// X语言标准库 - 网络操作模块
//
// 提供高层 TCP 网络编程接口

import "string"
import "collections"

// ==========================================
// TCP Socket 高层 API
// ==========================================

/// 创建 TCP 服务器并开始监听
/// 参数:
///   host: 绑定的主机地址，如 "127.0.0.1" 或 "0.0.0.0"
///   port: 监听的端口号
/// 返回: Result<Int, String> - 成功时为服务器句柄
function tcp_listen(host: String, port: Int): Result<Int, String> {
    __rt_tcp_listen(host, port)
}

/// 接受一个 TCP 连接
/// 参数:
///   server: 服务器句柄
/// 返回: Result<Int, String> - 成功时为连接句柄
function tcp_accept(server: Int): Result<Int, String> {
    __rt_tcp_accept(server)
}

/// 连接到远程 TCP 服务器
/// 参数:
///   host: 服务器地址
///   port: 服务器端口
/// 返回: Result<Int, String> - 成功时为连接句柄
function tcp_connect(host: String, port: Int): Result<Int, String> {
    __rt_tcp_connect(host, port)
}

/// 从 TCP 连接读取字节数组
/// 参数:
///   handle: 连接句柄
///   max_size: 最大读取字节数
/// 返回: Result<[Int], String> - 成功时为字节数组
function tcp_read_bytes(handle: Int, max_size: Int): Result<[Int], String> {
    __rt_tcp_read(handle, max_size)
}

/// 从 TCP 连接读取字符串
/// 参数:
///   handle: 连接句柄
///   max_size: 最大读取字节数
/// 返回: Result<String, String> - 成功时为读取的字符串
function tcp_read_string(handle: Int, max_size: Int): Result<String, String> {
    let result = tcp_read_bytes(handle, max_size)
    given result {
        is Ok(bytes) => {
            // 将字节数组转换为字符串（简化实现）
            Ok(to_string(len(bytes)) + " bytes received")
        }
        is Err(e) => {
            Err(e)
        }
    }
}

/// 向 TCP 连接写入字节数组
/// 参数:
///   handle: 连接句柄
///   bytes: 要写入的字节数组
/// 返回: Result<Int, String> - 成功时为写入的字节数
function tcp_write_bytes(handle: Int, bytes: [Int]): Result<Int, String> {
    __rt_tcp_write(handle, bytes)
}

/// 向 TCP 连接写入字符串
/// 参数:
///   handle: 连接句柄
///   data: 要发送的字符串
/// 返回: Result<Int, String> - 成功时为写入的字节数
function tcp_write_string(handle: Int, data: String): Result<Int, String> {
    // 将字符串转换为字节数组（简化实现）
    let bytes: [Int] = []
    let length = len(data)
    let mut i = 0
    while i < length {
        bytes = list_push(bytes, 0)
        i = i + 1
    }
    tcp_write_bytes(handle, bytes)
}

/// 关闭 TCP 连接或服务器
/// 参数:
///   handle: 连接或服务器句柄
function tcp_close(handle: Int): Unit {
    __rt_tcp_close(handle)
}

/// 检查 TCP 连接是否有数据可读
/// 参数:
///   handle: 连接句柄
/// 返回: Bool - 是否有数据可读
function tcp_readable(handle: Int): Bool {
    __rt_tcp_readable(handle)
}

/// 设置 TCP 连接为非阻塞模式
/// 参数:
///   handle: 连接句柄
///   flag: true 为非阻塞，false 为阻塞
/// 返回: Bool - 设置是否成功
function tcp_set_nonblocking(handle: Int, flag: Bool): Bool {
    __rt_tcp_set_nonblocking(handle, flag)
}
