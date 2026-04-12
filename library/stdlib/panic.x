module std.panic
import std.prelude
import std.types


import std.io;

/// Panic 处理函数类型
export type PanicHandler = function(string) -> never;

/// 当前的 panic 处理函数
/// 默认使用内置处理
static mut panic_handler: PanicHandler = default_panic_handler;

/// 设置自定义 panic 处理函数
export fn set_handler(handler: PanicHandler) -> unit {
    unsafe {
        panic_handler = handler;
    }
}

/// 获取当前的 panic 处理函数
export fn get_handler() -> PanicHandler {
    unsafe {
        panic_handler
    }
}

/// 默认 panic 处理
/// 输出错误信息并退出
export fn default_panic_handler(message: string) -> never {
    println("panic: " ++ message);
    // 尝试输出栈回溯（如果支持）
    print_stack_trace();
    // 退出程序
    exit_with_code(1);
}

/// 触发 panic
/// 使用当前注册的处理函数
export fn panic(message: string) -> never {
    let handler = get_handler();
    handler(message);
}

/// 外部 C 函数：退出
external "c" function exit(code: signed 32-bit integer) -> never;

/// 以指定退出码退出程序
export fn exit_with_code(code: Int) -> never {
    unsafe {
        exit(code as signed 32-bit integer);
    }
}

/// 正常退出
export fn exit_success() -> never {
    exit_with_code(0);
}

/// 异常退出（非 panic）
export fn exit_failure() -> never {
    exit_with_code(1);
}

/// 打印栈回溯
/// 这个是占位实现，实际需要平台特定代码
/// 在支持栈回溯的系统上会打印实际栈信息
export fn print_stack_trace() -> unit {
    // 目前只打印提示，实际实现需要平台特定的 unwind 代码
    println("note: stack trace not available in this build");
}

/// 检查条件，如果为 false 则 panic
export fn assert(condition: Bool, message: string) -> unit {
    when not condition {
        panic("assertion failed: " ++ message);
    }
}

/// 检查条件，如果为 false 则 panic（无消息）
export fn assert(condition: Bool) -> unit {
    when not condition {
        panic("assertion failed");
    }
}

/// 解包 Option，如果是 None 则 panic
export fn unwrap<T>(opt: Option<T>, msg: string) -> T {
    match opt {
        None => panic(msg),
        Some(v) => v,
    }
}

/// 解包 Result，如果是 Err 则 panic
export fn unwrap<T, E>(res: Result<T, E>, msg: string) -> T where E: to_string {
    match res {
        Err(e) => panic(msg ++ ": " ++ to_string(e)),
        Ok(v) => v,
    }
}

/// 如果条件不为真则恐慌，带有位置信息
/// 通常由宏展开使用
export fn assert_with_location(condition: Bool, file: string, line: Int) -> unit {
    when not condition {
        panic("assertion failed at " ++ file ++ ":" ++ line.to_string());
    }
}

/// 未实现代码路径
export fn todo(reason: string) -> never {
    panic("not implemented: " ++ reason);
}

/// 不可达代码路径
export fn unreachable() -> never {
    panic("entered unreachable code");
}
