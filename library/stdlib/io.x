module std.io
import std.prelude
import std.types



/// 外部 C 库函数：getline - 读取一行
external "c" function getline(line: **character, capacity: *CSize, stream: *()) -> CLong

/// 外部 C 库函数：stdin
external "c" variable stdin: *()

/// 外部 C 库变量：stdout
external "c" variable stdout: *()

/// 外部 C 库函数：fflush - 刷新缓冲区
external "c" function fflush(stream: *()) -> signed 32-bit integer

/// 外部 C 库函数：free - 释放 getline 分配的内存
external "c" function free(ptr: *()) -> unit

/// 读取一行从标准输入
export function read_line() -> Result<string, string> {
    unsafe {
        let buffer: *character = null;
        let capacity: CSize = 0 as CSize;
        let result = getline(&buffer as **character, &capacity as *CSize, stdin);
        let read_len = result as signed 64-bit integer;
        when read_len < 0 {
            free(buffer as *());
            Err("failed to read line")
        } else {
            // 转换 buffer 到 X 字符串
            // 这里需要复制内容到 X 管理的字符串
            let end = when read_len > 0 and buffer[read_len - 1] as character == '\n' {
                read_len - 1
            } else {
                read_len
            };
            let mut s = "";
            let mut i = 0;
            while i < end {
                let c = (buffer[i] as character);
                s = s ++ c;
                i = i + 1;
            }
            free(buffer as *());
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
        let result = fflush(stdout);
    }
}
