module std.net
import std.prelude
import std.types



// 地址族
const AF_INET: Int = 2;      // IPv4
const AF_INET6: Int = 10;    // IPv6
const AF_UNIX: Int = 1;      // Unix domain

// 套接字类型
const SOCK_STREAM: Int = 1;  // 流式 socket (TCP)
const SOCK_DGRAM: Int = 2;   // 数据报 socket (UDP)
const SOCK_RAW: Int = 3;     // 原始 socket

// 选项
const SOL_SOCKET: Int = 1;
const SO_REUSEADDR: Int = 4;

// 级别
const IPPROTO_TCP: Int = 6;

// TCP 选项
const TCP_NODELAY: Int = 1;

// 消息标志
const MSG_PEEK: Int = 2;
const MSG_OOB: Int = 1;

// 地址结构常量
const INADDR_ANY: u32 = 0;
const INADDR_LOOPBACK: u32 = 0x7F000001;

// shutdown 方式
const SHUT_RD: Int = 0;
const SHUT_WR: Int = 1;
const SHUT_RDWR: Int = 2;

// === 外部 C 库函数（POSIX socket）===

external "c" function socket(domain: signed 32-bit integer, type_: signed 32-bit integer, protocol: signed 32-bit integer) -> signed 32-bit integer
external "c" function bind(fd: signed 32-bit integer, addr: *(), addr_len: usize) -> signed 32-bit integer
external "c" function listen(fd: signed 32-bit integer, backlog: signed 32-bit integer) -> signed 32-bit integer
external "c" function accept(fd: signed 32-bit integer, addr: *(), addr_len: *usize) -> signed 32-bit integer
external "c" function connect(fd: signed 32-bit integer, addr: *(), addr_len: usize) -> signed 32-bit integer
external "c" function getsockname(fd: signed 32-bit integer, addr: *(), addr_len: *usize) -> signed 32-bit integer
external "c" function getpeername(fd: signed 32-bit integer, addr: *(), addr_len: *usize) -> signed 32-bit integer
external "c" function setsockopt(fd: signed 32-bit integer, level: signed 32-bit integer, optname: signed 32-bit integer, optval: *(), optlen: usize) -> signed 32-bit integer
external "c" function getsockopt(fd: signed 32-bit integer, level: signed 32-bit integer, optname: signed 32-bit integer, optval: *(), optlen: *usize) -> signed 32-bit integer
external "c" function send(fd: signed 32-bit integer, buf: *(), len: usize, flags: signed 32-bit integer) -> signed 32-bit integer
external "c" function recv(fd: signed 32-bit integer, buf: *(), len: usize, flags: signed 32-bit integer) -> signed 32-bit integer
external "c" function sendto(fd: signed 32-bit integer, buf: *(), len: usize, flags: signed 32-bit integer, to: *(), tolen: usize) -> signed 32-bit integer
external "c" function recvfrom(fd: signed 32-bit integer, buf: *(), len: usize, flags: signed 32-bit integer, from: *(), fromlen: *usize) -> signed 32-bit integer
external "c" function close(fd: signed 32-bit integer) -> signed 32-bit integer
external "c" function shutdown(fd: signed 32-bit integer, how: signed 32-bit integer) -> signed 32-bit integer
external "c" function htonl(host: unsigned 32-bit integer) -> unsigned 32-bit integer
external "c" function htons(host: unsigned 16-bit integer) -> unsigned 16-bit integer
external "c" function ntohl(net: unsigned 32-bit integer) -> unsigned 32-bit integer
external "c" function ntohs(net: unsigned 16-bit integer) -> unsigned 16-bit integer
external "c" function gethostbyname(name: *character) -> *()
external "c" function inet_addr(cp: *character) -> unsigned 32-bit integer

/// IPv4 地址
export record IpV4Addr {
    /// 网络字节序地址
    addr: u32,
}

/// 创建 IPv4 地址从四个字节
export fn ipv4(a: u8, b: u8, c: u8, d: u8) -> IpV4Addr {
    let addr = ((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32);
    IpV4Addr { addr: addr }
}

/// 解析 IPv4 地址字符串
export fn parse_ipv4(str: string) -> Result<IpV4Addr, string> {
    unsafe {
        let net_addr = inet_addr(str as *character);
        when net_addr == 0xFFFFFFFF {
            Err("invalid IPv4 address")
        } else {
            Ok(IpV4Addr { addr: net_addr })
        }
    }
}

/// 获取任意地址（0.0.0.0）
export fn ipv4_any() -> IpV4Addr {
    IpV4Addr { addr: INADDR_ANY }
}

/// 获取回环地址（127.0.0.1）
export fn ipv4_loopback() -> IpV4Addr {
    IpV4Addr { addr: INADDR_LOOPBACK }
}

/// TCP 监听器
export record TcpListener {
    fd: Int,
}

/// TCP 连接
export record TcpStream {
    fd: Int,
}

/// UDP 套接字
export record UdpSocket {
    fd: Int,
}

/// 创建 TCP 监听器绑定到指定地址端口
export fn tcp_bind(addr: IpV4Addr, port: Int) -> Result<TcpListener, string> {
    unsafe {
        let fd = socket(AF_INET as signed 32-bit integer, SOCK_STREAM as signed 32-bit integer, 0);
        when fd < 0 {
            Err("failed to create socket")
        } else {
            // 设置 SO_REUSEADDR
            let optval: signed 32-bit integer = 1;
            setsockopt(fd, SOL_SOCKET as signed 32-bit integer, SO_REUSEADDR as signed 32-bit integer, &optval as *(), size_of.<signed 32-bit integer>() as usize);

            // 构造 sockaddr_in
            let mut buffer: [u8] = [0; 16]; // 足够大小容纳 sockaddr_in
            // sin_family = AF_INET
            buffer[0] = (AF_INET >> 8) as u8;
            buffer[1] = (AF_INET & 0xFF) as u8;
            // sin_port
            let net_port = htons(port as unsigned 16-bit integer);
            buffer[2] = ((net_port >> 8) & 0xFF) as u8;
            buffer[3] = (net_port & 0xFF) as u8;
            // sin_addr
            let net_addr = addr.addr;
            buffer[4] = ((net_addr >> 24) & 0xFF) as u8;
            buffer[5] = ((net_addr >> 16) & 0xFF) as u8;
            buffer[6] = ((net_addr >> 8) & 0xFF) as u8;
            buffer[7] = (net_addr & 0xFF) as u8;

            let result = bind(fd, buffer as *(), 16 as usize);
            when result < 0 {
                close(fd);
                Err("failed to bind address")
            } else {
                let result = listen(fd, 128 as signed 32-bit integer);
                when result < 0 {
                    close(fd);
                    Err("failed to listen")
                } else {
                    Ok(TcpListener { fd: fd as Int })
                }
            }
        }
    }
}

/// 接受新的 TCP 连接
export fn accept(listener: &TcpListener) -> Result<TcpStream, string> {
    unsafe {
        let mut buffer: [u8] = [0; 16];
        let mut addr_len = 16 as usize;
        let client_fd = accept(listener.fd as signed 32-bit integer, buffer as *(), &addr_len as *usize);
        when client_fd < 0 {
            Err("failed to accept connection")
        } else {
            Ok(TcpStream { fd: client_fd as Int })
        }
    }
}

/// 连接到 TCP 服务器
export fn tcp_connect(addr: IpV4Addr, port: Int) -> Result<TcpStream, string> {
    unsafe {
        let fd = socket(AF_INET as signed 32-bit integer, SOCK_STREAM as signed 32-bit integer, 0);
        when fd < 0 {
            Err("failed to create socket")
        } else {
            let mut buffer: [u8] = [0; 16];
            buffer[0] = (AF_INET >> 8) as u8;
            buffer[1] = (AF_INET & 0xFF) as u8;
            let net_port = htons(port as unsigned 16-bit integer);
            buffer[2] = ((net_port >> 8) & 0xFF) as u8;
            buffer[3] = (net_port & 0xFF) as u8;
            let net_addr = addr.addr;
            buffer[4] = ((net_addr >> 24) & 0xFF) as u8;
            buffer[5] = ((net_addr >> 16) & 0xFF) as u8;
            buffer[6] = ((net_addr >> 8) & 0xFF) as u8;
            buffer[7] = (net_addr & 0xFF) as u8;

            let result = connect(fd, buffer as *(), 16 as usize);
            when result < 0 {
                close(fd);
                Err("failed to connect")
            } else {
                Ok(TcpStream { fd: fd as Int })
            }
        }
    }
}

/// 写入字节到 TCP 流
export fn write(stream: &mut TcpStream, bytes: [u8]) -> Result<Int, string> {
    unsafe {
        let len = bytes.len();
        when len == 0 {
            Ok(0)
        }
        let ptr = &bytes[0] as *u8 as *();
        let sent = send(stream.fd as signed 32-bit integer, ptr, len as usize, 0);
        when sent < 0 {
            Err("failed to send data")
        } else {
            Ok(sent as Int)
        }
    }
}

/// 写入字符串到 TCP 流
export fn write_string(stream: &mut TcpStream, text: string) -> Result<Int, string> {
    unsafe {
        let len = text.len();
        when len == 0 {
            Ok(0)
        }
        let ptr = text as *character as *();
        let sent = send(stream.fd as signed 32-bit integer, ptr, len as usize, 0);
        when sent < 0 {
            Err("failed to send data")
        } else {
            Ok(sent as Int)
        }
    }
}

/// 读取字节从 TCP 流
export fn read(stream: &mut TcpStream, buffer: &mut [u8]) -> Result<Int, string> {
    unsafe {
        let capacity = buffer.len() as usize;
        let ptr = buffer as *u8 as *();
        let received = recv(stream.fd as signed 32-bit integer, ptr, capacity, 0);
        when received < 0 {
            Err("failed to receive data")
        } else {
            Ok(received as Int)
        }
    }
}

/// 读取到缓冲区直到填满或 EOF
export fn read_exact(stream: &mut TcpStream, buffer: &mut [u8]) -> Result<unit, string> {
    let len = buffer.len();
    let mut offset = 0;
    while offset < len {
        let result = read(stream, &mut buffer[offset..]);
        match result {
            Err(e) => return Err(e),
            Ok(0) => {
                // EOF
                return Err("unexpected end of stream");
            },
            Ok(n) => {
                offset = offset + n;
            },
        }
    }
    Ok(unit)
}

/// 读取整行（以 \n 结束）
export fn read_line(stream: &mut TcpStream) -> Result<string, string> {
    let mut line = "";
    let mut byte: [u8] = [0];
    loop {
        match read(stream, &mut byte) {
            Err(e) => return Err(e),
            Ok(0) => {
                when line.len() > 0 {
                    return Ok(line);
                } else {
                    return Ok("");
                }
            },
            Ok(1) => {
                let c = byte[0] as character;
                when c == '\n' {
                    return Ok(line);
                }
                when c != '\r' {
                    line = line ++ c;
                }
            },
        }
    }
}

/// 关闭 TCP 流
export fn close(stream: TcpStream) -> Result<unit, string> {
    unsafe {
        let result = close(stream.fd as signed 32-bit integer);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to close socket")
        }
    }
}

/// 关闭监听器
export fn close_listener(listener: TcpListener) -> Result<unit, string> {
    unsafe {
        let result = close(listener.fd as signed 32-bit integer);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to close listener")
        }
    }
}

/// 获取文件描述符（用于 select/poll 等）
export fn fileno(stream: &TcpStream) -> Int {
    stream.fd
}

/// 设置 TCP_NODELAY（禁用 Nagle 算法）
export fn set_nodelay(stream: &mut TcpStream, nodelay: Bool) -> Result<unit, string> {
    unsafe {
        let optval: signed 32-bit integer = when nodelay { 1 } else { 0 };
        let result = setsockopt(
            stream.fd as signed 32-bit integer,
            IPPROTO_TCP as signed 32-bit integer,
            TCP_NODELAY as signed 32-bit integer,
            &optval as *(),
            size_of.<signed 32-bit integer>() as usize
        );
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to set TCP_NODELAY")
        }
    }
}

// === UDP ===

/// 创建 UDP 套接字绑定到地址端口
export fn udp_bind(addr: IpV4Addr, port: Int) -> Result<UdpSocket, string> {
    unsafe {
        let fd = socket(AF_INET as signed 32-bit integer, SOCK_DGRAM as signed 32-bit integer, 0);
        when fd < 0 {
            Err("failed to create UDP socket")
        } else {
            let mut buffer: [u8] = [0; 16];
            buffer[0] = (AF_INET >> 8) as u8;
            buffer[1] = (AF_INET & 0xFF) as u8;
            let net_port = htons(port as unsigned 16-bit integer);
            buffer[2] = ((net_port >> 8) & 0xFF) as u8;
            buffer[3] = (net_port & 0xFF) as u8;
            let net_addr = addr.addr;
            buffer[4] = ((net_addr >> 24) & 0xFF) as u8;
            buffer[5] = ((net_addr >> 16) & 0xFF) as u8;
            buffer[6] = ((net_addr >> 8) & 0xFF) as u8;
            buffer[7] = (net_addr & 0xFF) as u8;

            let result = bind(fd, buffer as *(), 16 as usize);
            when result < 0 {
                close(fd);
                Err("failed to bind UDP socket")
            } else {
                Ok(UdpSocket { fd: fd as Int })
            }
        }
    }
}

/// 发送 UDP 数据报到指定地址
export fn udp_send(socket: &UdpSocket, bytes: [u8], to: IpV4Addr, port: Int) -> Result<Int, string> {
    unsafe {
        let mut buffer: [u8] = [0; 16];
        buffer[0] = (AF_INET >> 8) as u8;
        buffer[1] = (AF_INET & 0xFF) as u8;
        let net_port = htons(port as unsigned 16-bit integer);
        buffer[2] = ((net_port >> 8) & 0xFF) as u8;
        buffer[3] = (net_port & 0xFF) as u8;
        let net_addr = to.addr;
        buffer[4] = ((net_addr >> 24) & 0xFF) as u8;
        buffer[5] = ((net_addr >> 16) & 0xFF) as u8;
        buffer[6] = ((net_addr >> 8) & 0xFF) as u8;
        buffer[7] = (net_addr & 0xFF) as u8;

        let len = bytes.len();
        when len == 0 {
            Ok(0)
        }
        let ptr = &bytes[0] as *u8 as *();
        let sent = sendto(
            socket.fd as signed 32-bit integer,
            ptr,
            len as usize,
            0,
            buffer as *(),
            16 as usize
        );
        when sent < 0 {
            Err("failed to send UDP datagram")
        } else {
            Ok(sent as Int)
        }
    }
}

/// 接收 UDP 数据报
export fn udp_recv(socket: &UdpSocket, buffer: &mut [u8]) -> Result<(Int, IpV4Addr, Int), string> {
    unsafe {
        let mut from_buf: [u8] = [0; 16];
        let mut from_len = 16 as usize;
        let capacity = buffer.len() as usize;
        let ptr = buffer as *u8 as *();
        let received = recvfrom(
            socket.fd as signed 32-bit integer,
            ptr,
            capacity,
            0,
            from_buf as *(),
            &from_len as *usize
        );
        when received < 0 {
            Err("failed to receive UDP datagram")
        } else {
            // 解析地址
            // from_buf 已经是 sockaddr_in，提取端口和地址
            let port: Int = ((from_buf[2] as Int) << 8) | (from_buf[3] as Int);
            let port = ntohs(port as unsigned 16-bit integer) as Int;
            let addr: u32 =
                ((from_buf[4] as u32) << 24) |
                ((from_buf[5] as u32) << 16) |
                ((from_buf[6] as u32) << 8) |
                (from_buf[7] as u32);
            Ok((received as Int, IpV4Addr { addr: addr }, port))
        }
    }
}

/// 关闭 UDP 套接字
export fn udp_close(socket: UdpSocket) -> Result<unit, string> {
    unsafe {
        let result = close(socket.fd as signed 32-bit integer);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to close UDP socket")
        }
    }
}

/// 主机字节序转网络字节序 32 位
export fn htonl(host: u32) -> u32 {
    unsafe { htonl(host) }
}

/// 主机字节序转网络字节序 16 位
export fn htons(host: u16) -> u16 {
    unsafe { htons(host) }
}

/// 网络字节序转主机字节序 32 位
export fn ntohl(net: u32) -> u32 {
    unsafe { ntohl(net) }
}

/// 网络字节序转主机字节序 16 位
export fn ntohs(net: u16) -> u16 {
    unsafe { ntohs(net) }
}
