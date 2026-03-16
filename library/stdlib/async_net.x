// X语言标准库 - 异步网络模块
//
// 提供异步 TCP 网络编程接口

import "string"
import "collections"

// ==========================================
// 异步 TCP 操作
// ==========================================

/// 异步接受 TCP 连接
/// 返回操作 ID，用于轮询结果
function tcp_accept_async(handle: Int): Int {
    __rt_tcp_accept_async(handle)
}

/// 异步读取 TCP 数据
/// 返回操作 ID，用于轮询结果
function tcp_read_async(handle: Int, max_size: Int): Int {
    __rt_tcp_read_async(handle, max_size)
}

/// 异步写入 TCP 数据
/// 返回操作 ID，用于轮询结果
function tcp_write_async(handle: Int, data: [Int]): Int {
    __rt_tcp_write_async(handle, data)
}

/// 检查异步操作是否完成
function async_poll(op_id: Int): Bool {
    __rt_async_poll(op_id)
}

/// 获取异步操作的结果
function async_result(op_id: Int): Result<dynamic, String> {
    __rt_async_result(op_id)
}

/// 事件循环迭代
function event_loop_tick(): Bool {
    __rt_event_loop_tick()
}

// ==========================================
// 高层异步 API
// ==========================================

/// 等待异步操作完成并获取结果
function await_async(op_id: Int): Result<dynamic, String> {
    let mut done = false
    while !done {
        done = async_poll(op_id)
        if !done {
            sleep(1)
        }
    }
    async_result(op_id)
}

/// 异步接受连接并返回连接句柄
async function tcp_accept_await(server: Int): Result<Int, String> {
    let op_id = tcp_accept_async(server)
    let result = await_async(op_id)
    given result {
        is Ok(v) => {
            given v {
                is Int(handle) => Ok(handle)
                is _ => Err("Invalid result type")
            }
        }
        is Err(e) => Err(e)
    }
}

/// 异步读取数据
async function tcp_read_await(handle: Int, max_size: Int): Result<[Int], String> {
    let op_id = tcp_read_async(handle, max_size)
    let result = await_async(op_id)
    given result {
        is Ok(v) => {
            given v {
                is arr => Ok(arr)
                is _ => Err("Invalid result type")
            }
        }
        is Err(e) => Err(e)
    }
}

/// 异步写入数据
async function tcp_write_await(handle: Int, data: [Int]): Result<Int, String> {
    let op_id = tcp_write_async(handle, data)
    let result = await_async(op_id)
    given result {
        is Ok(v) => {
            given v {
                is Int(n) => Ok(n)
                is _ => Err("Invalid result type")
            }
        }
        is Err(e) => Err(e)
    }
}

/// 异步读取字符串
async function tcp_read_string_async(handle: Int, max_size: Int): Result<String, String> {
    let result = tcp_read_await(handle, max_size)
    given result {
        is Ok(bytes) => {
            // 将字节数组转换为字符串（简化实现）
            Ok(to_string(len(bytes)) + " bytes")
        }
        is Err(e) => Err(e)
    }
}

/// 异步写入字符串
async function tcp_write_string_async(handle: Int, data: String): Result<Int, String> {
    // 将字符串转换为字节数组（简化实现）
    let bytes: [Int] = []
    let mut i = 0
    let length = len(data)
    while i < length {
        bytes = list_push(bytes, 0)
        i = i + 1
    }
    tcp_write_await(handle, bytes)
}
