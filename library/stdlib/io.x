module std.io
import std.prelude
import std.types



/// 外部 C 库函数：getline - 读取一行
external "c" function getline(line: **character, capacity: *signed 32-bit integer, stream: *()) -> signed 32-bit integer

/// 外部 C 库函数：stdin
external "c" variable stdin: *()

/// 读取一行从标准输入
export function read_line() -> Result<string, string> {
    unsafe {
        let buffer: *character = null;
        let capacity: signed 32-bit integer = 0;
        let result = getline(&buffer, &capacity, stdin);
        when result < 0 {
            Err("failed to read line")
        } else {
            // 转换 buffer 到 X 字符串
            // 这里需要复制内容到 X 管理的字符串
            let mut s = "";
            let mut i = 0;
            while i < result - 1 {
                let c = (buffer[i] as character);
                s = s ++ c;
                i = i + 1;
            }
            // 不需要释放，C stdio getline 分配了内存我们现在没有 free 绑定
            // TODO: 正确释放内存
            Ok(s)
        }
    }
}

/// 读取一行从标准输入，如果出错返回空字符串
export function read_line_or_empty() -> string {
    match read_line() {
        Ok(s) => s,
        Err(_) => "",
    }
}

/// 刷新标准输出缓冲区
export function flush() -> unit {
    unsafe {
        fflush(stdout);
    }
}
