module std.fs
import std.prelude
import std.types



/// 文件打开模式
export enum OpenMode {
    /// 只读模式
    Read,
    /// 只写模式（会覆盖）
    Write,
    /// 追加模式
    Append,
    /// 读写模式
    ReadWrite,
    /// 读写模式（创建文件不存在则创建）
    ReadWriteCreate,
}

/// 将打开模式转换为 C 字符串
function mode_to_c_string(mode: OpenMode) -> string {
    match mode {
        OpenMode.Read => "r",
        OpenMode.Write => "w",
        OpenMode.Append => "a",
        OpenMode.ReadWrite => "r+",
        OpenMode.ReadWriteCreate => "w+",
    }
}

/// 文件句柄
export record File {
    /// 底层 C FILE 指针
    handle: *(),
}

/// 文件信息
export record FileInfo {
    /// 文件大小（字节）
    size: Int,
    /// 是否为目录
    is_dir: Bool,
    /// 是否为普通文件
    is_file: Bool,
    /// 是否为符号链接
    is_symlink: Bool,
}

// === 外部 C 库函数绑定 ===

/// 打开文件
external "c" function fopen(path: *character, mode: *character) -> *()

/// 关闭文件
external "c" function fclose(stream: *()) -> signed 32-bit integer

/// 读取文件内容
external "c" function fread(ptr: *(), size: usize, nmemb: usize, stream: *()) -> usize

/// 写入文件内容
external "c" function fwrite(ptr: *(), size: usize, nmemb: usize, stream: *()) -> usize

/// 获取文件大小
external "c" function fseek(stream: *(), offset: signed 32-bit integer, whence: signed 32-bit integer) -> signed 32-bit integer

/// 获取当前位置
external "c" function ftell(stream: *()) -> signed 32-bit integer

/// 刷新缓冲区
external "c" function fflush(stream: *()) -> signed 32-bit integer

/// 删除文件
external "c" function remove(path: *character) -> signed 32-bit integer

/// 重命名文件
external "c" function rename(old_path: *character, new_path: *character) -> signed 32-bit integer

/// 获取文件信息
external "c" function stat(path: *character, buf: *()) -> signed 32-bit integer

/// 创建目录
external "c" function mkdir(path: *character, mode: unsigned 32-bit integer) -> signed 32-bit integer

/// 删除目录
external "c" function rmdir(path: *character) -> signed 32-bit integer

/// 检查文件是否存在
/// 返回 true 如果文件存在，false 如果不存在
export function exists(path: string) -> Bool {
    unsafe {
        // 简化实现：尝试打开文件
        let handle = fopen(path as *character, "r" as *character);
        when handle == null {
            false
        } else {
            fclose(handle);
            true
        }
    }
}

/// 打开文件
/// 返回 Ok(File) 成功，Err(string) 失败
export function open(path: string, mode: OpenMode) -> Result<File, string> {
    let mode_str = mode_to_c_string(mode);
    unsafe {
        let handle = fopen(path as *character, mode_str as *character);
        when handle == null {
            Err("failed to open file: " ++ path)
        } else {
            Ok(File { handle: handle })
        }
    }
}

/// 创建并打开文件用于写入
/// 如果文件已存在则覆盖
export function create(path: string) -> Result<File, string> {
    open(path, OpenMode.Write)
}

/// 关闭文件
export function close(self: File) -> Result<unit, string> {
    unsafe {
        let result = fclose(self.handle);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to close file")
        }
    }
}

/// 获取文件大小（字节）
export function size(self: File) -> Result<Int, string> {
    unsafe {
        // SEEK_SET = 0
        when fseek(self.handle, 0, 2) != 0 {
            Err("fseek failed")
        } else {
            let size = ftell(self.handle);
            Ok(size as Int)
        }
    }
}

/// 读取整个文件内容为字符串
export function read_to_string(self: File) -> Result<string, string> {
    let size_result = self.size();
    match size_result {
        Err(err) => Err(err),
        Ok(size) => unsafe {
            when size < 0 {
                Err("invalid file size")
            }
            when size == 0 {
                Ok("")
            }
            // 分配缓冲区并读取
            // 注意：这里我们依赖 X 的字符串构造，逐个字符读取
            // 在实际实现中，这应该更高效
            // 回到开头
            fseek(self.handle, 0, 0);
            let mut result = "";
            let mut buffer: character = '\0';
            let mut bytes_read: signed 32-bit integer = 0;
            while bytes_read < (size as signed 32-bit integer) {
                let read = fread(&buffer as *(), 1, 1, self.handle);
                when read == 0 {
                    break;
                }
                result = result ++ buffer;
                bytes_read = bytes_read + 1;
            }
            Ok(result)
        },
    }
}

/// 读取整个文件内容为字节数组
export function read_to_bytes(self: File) -> Result<[u8], string> {
    let size_result = self.size();
    match size_result {
        Err(err) => Err(err),
        Ok(size) => unsafe {
            when size < 0 {
                Err("invalid file size")
            }
            when size == 0 {
                Ok([])
            }
            fseek(self.handle, 0, 0);
            let mut bytes: [u8] = [];
            let mut byte: u8 = 0;
            let mut bytes_read = 0;
            while bytes_read < size {
                let read = fread(&byte as *(), 1, 1, self.handle);
                when read == 0 {
                    break;
                }
                bytes.push(byte);
                bytes_read = bytes_read + 1;
            }
            Ok(bytes)
        },
    }
}

/// 将字符串写入文件
export function write_string(self: &mut File, content: string) -> Result<Int, string> {
    unsafe {
        let len = content.len();
        let written = fwrite(content as *character, 1, len as usize, self.handle);
        fflush(self.handle);
        Ok(written as Int)
    }
}

/// 将字节数组写入文件
export function write_bytes(self: &mut File, bytes: [u8]) -> Result<Int, string> {
    unsafe {
        let len = bytes.len();
        when len == 0 {
            Ok(0)
        }
        // 获取第一个元素的指针
        let ptr = &bytes[0] as *u8 as *();
        let written = fwrite(ptr, 1, len as usize, self.handle);
        fflush(self.handle);
        Ok(written as Int)
    }
}

/// 删除文件
export function remove_file(path: string) -> Result<unit, string> {
    unsafe {
        let result = remove(path as *character);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to remove file: " ++ path)
        }
    }
}

/// 重命名/移动文件
export function rename(old_path: string, new_path: string) -> Result<unit, string> {
    unsafe {
        let result = rename(old_path as *character, new_path as *character);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to rename file")
        }
    }
}

/// 创建目录
/// mode: 权限模式，默认 0o755
export function create_dir(path: string, mode: Option<unsigned 32-bit integer>) -> Result<unit, string> {
    let actual_mode = match mode {
        None => 493, // 0o755
        Some(m) => m,
    };
    unsafe {
        let result = mkdir(path as *character, actual_mode);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to create directory: " ++ path)
        }
    }
}

/// 删除空目录
export function remove_dir(path: string) -> Result<unit, string> {
    unsafe {
        let result = rmdir(path as *character);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to remove directory: " ++ path)
        }
    }
}

/// 读取整个文件内容为字符串（便捷函数）
export function read_to_string_at(path: string) -> Result<string, string> {
    match open(path, OpenMode.Read) {
        Err(err) => Err(err),
        Ok(mut file) => {
            let result = file.read_to_string();
            file.close();
            result
        },
    }
}

/// 将字符串写入文件（便捷函数）
export function write_string_to(path: string, content: string) -> Result<unit, string> {
    match create(path) {
        Err(err) => Err(err),
        Ok(mut file) => {
            match file.write_string(content) {
                Err(err) => {
                    file.close();
                    Err(err)
                },
                Ok(_) => {
                    file.close();
                    Ok(unit)
                },
            }
        },
    }
}

/// 将字节数组写入文件（便捷函数）
export function write_bytes_to(path: string, bytes: [u8]) -> Result<unit, string> {
    match create(path) {
        Err(err) => Err(err),
        Ok(mut file) => {
            match file.write_bytes(bytes) {
                Err(err) => {
                    file.close();
                    Err(err)
                },
                Ok(_) => {
                    file.close();
                    Ok(unit)
                },
            }
        },
    }
}
