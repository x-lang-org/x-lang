// Minimal prelude with C FFI

// Re-export Option and Result from std.types
// These are now user-defined enums, not compiler built-ins
import std.types.*;

/// 外部 C 库函数：puts - 输出字符串并换行
external function puts(message: *character) -> signed 32-bit integer

/// 外部 C 库函数：putchar - 输出单个字符
external function putchar(c: signed 32-bit integer) -> signed 32-bit integer

/// println 函数 - 打印字符串并换行
export function println(message: string) -> unit {
    unsafe {
        puts(message as *character)
    }
}

/// print 函数 - 打印字符串不换行
export function print(message: string) -> unit {
    for c in message {
        unsafe {
            putchar(c as signed 32-bit integer)
        }
    }
}

/// panic 宏 - 终止程序并输出错误信息
export function panic(message: string) -> unit {
    println(message)
}

/// assert 断言 - 如果条件不满足则panic
export function assert(condition: boolean) -> unit {
    if not condition {
        panic("assertion failed")
    }
}

/// Builtin: 读取文件内容
/// __file_read(path: string) -> Result<string, string>
external function __file_read(path: string) -> Result<string, string>

/// Builtin: 获取命令行参数
/// __args() -> [string]
external function __args() -> [string]

/// Builtin: 解析JSON字符串
/// x_json_parse(json: string) -> Dynamic
external function x_json_parse(json: string) -> Dynamic

/// Builtin: 获取环境变量
/// __get_env(name: string) -> Option<string>
external function __get_env(name: string) -> Option<string>
