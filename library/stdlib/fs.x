module std.fs

import std.types;

/// 读取整个文本文件内容，如果失败则 panic
export function read_file(path: string) -> string {
    let result = __file_read(path)
    // 强制转换为 string
    unwrap_ok(result) as string
}

/// 写入内容到文本文件，如果失败则 panic
export function write_file(path: string, content: string) -> unit {
    let result = __file_write(path, content)
    unwrap_ok(result)
    ()
}

/// 检查文件是否存在
export function exists(path: string) -> boolean {
    __file_exists(path)
}

/// 删除文件，如果失败则 panic
export function delete_file(path: string) -> unit {
    let result = __file_delete(path)
    unwrap_ok(result)
    ()
}
