// X 语言 TCP Echo Server 示例
// 演示基本的 TCP 网络编程

import "stdlib/net"

function main() {
    println("Starting TCP Echo Server on 127.0.0.1:8080")

    // 创建 TCP 服务器
    let result = tcp_listen("127.0.0.1", 8080)

    given result {
        is Ok(server) => {
            println("Server listening on port 8080")

            // 接受连接循环
            let mut running = true
            while running {
                let conn_result = tcp_accept(server)

                given conn_result {
                    is Ok(conn) => {
                        println("Client connected")

                        // 读取数据
                        let read_result = __rt_tcp_read(conn, 1024)

                        given read_result {
                            is Ok(bytes) => {
                                println("Received data")
                                // 回显数据
                                let write_result = __rt_tcp_write(conn, bytes)

                                given write_result {
                                    is Ok(_) => println("Echo sent")
                                    is Err(e) => println("Write error: " + e)
                                }
                            }
                            is Err(e) => {
                                println("Read error: " + e)
                            }
                        }

                        // 关闭连接
                        tcp_close(conn)
                    }
                    is Err(e) => {
                        println("Accept error: " + e)
                    }
                }
            }

            tcp_close(server)
        }
        is Err(e) => {
            println("Failed to start server: " + e)
        }
    }
}
