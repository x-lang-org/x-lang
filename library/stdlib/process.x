module std.process
import std.prelude
import std.types



// === 外部 C 库函数 ===

external "c" function exit(code: signed 32-bit integer) -> never
external "c" function system(command: *character) -> signed 32-bit integer
external "c" function getpid() -> signed 32-bit integer
external "c" function getppid() -> signed 32-bit integer
external "c" function sleep(seconds: signed 32-bit integer) -> signed 32-bit integer
external "c" function abort() -> never

/// 以指定退出码退出当前进程
export fn exit(code: Int) -> never {
    unsafe {
        exit(code as signed 32-bit integer);
    }
}

/// 正常退出（退出码 0）
export fn exit_success() -> never {
    exit(0);
}

/// 异常退出（退出码 1）
export fn exit_failure() -> never {
    exit(1);
}

/// 异常中止进程（SIGABRT）
export fn abort() -> never {
    unsafe {
        abort();
    }
}

/// 执行系统命令
/// 返回退出码
export fn system(command: string) -> Int {
    unsafe {
        system(command as *character) as Int
    }
}

/// 执行系统命令并返回是否成功
export fn system_ok(command: string) -> Bool {
    system(command) == 0
}

/// 获取当前进程 ID
export fn get_pid() -> Int {
    unsafe {
        getpid() as Int
    }
}

/// 获取父进程 ID
export fn get_ppid() -> Int {
    unsafe {
        getppid() as Int
    }
}

/// 环境变量
/// 获取环境变量的值，如果不存在返回 None
external "c" function getenv(name: *character) -> *character

/// 获取环境变量
export fn get_env(name: string) -> Option<string> {
    unsafe {
        let ptr = getenv(name as *character);
        when ptr == null {
            None
        } else {
            // 计算字符串长度
            let mut len = 0;
            while (*(ptr + len) as character != '\0') {
                len = len + 1;
            }
            // 构造字符串
            let mut s = "";
            let mut i = 0;
            while i < len {
                s = s ++ (*(ptr + i) as character);
                i = i + 1;
            }
            Some(s)
        }
    }
}

/// 设置环境变量
/// 1 = overwrite, 0 = don't overwrite if already exists
external "c" function setenv(name: *character, value: *character, overwrite: signed 32-bit integer) -> signed 32-bit integer

/// 设置环境变量
export fn set_env(name: string, value: string, overwrite: Bool) -> Result<unit, string> {
    unsafe {
        let over = when overwrite { 1 } else { 0 };
        let result = setenv(name as *character, value as *character, over);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to set environment variable")
        }
    }
}

/// 移除环境变量
external "c" function unsetenv(name: *character) -> signed 32-bit integer

/// 移除环境变量
export fn unset_env(name: string) -> Result<unit, string> {
    unsafe {
        let result = unsetenv(name as *character);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to unset environment variable")
        }
    }
}

/// 当前工作目录
external "c" function getcwd(buf: *character, size: usize) -> *character

/// 获取当前工作目录
export fn get_current_dir() -> Result<string, string> {
    unsafe {
        // 分配一个合理大小的缓冲区
        let capacity = 4096;
        let mut buf: [character] = ['\0'; capacity];
        let result = getcwd(buf as *character, capacity as usize);
        when result == null {
            Err("failed to get current working directory")
        } else {
            // 计算长度
            let mut len = 0;
            while len < capacity && buf[len] != '\0' {
                len = len + 1;
            }
            let mut s = "";
            let mut i = 0;
            while i < len {
                s = s ++ buf[i];
                i = i + 1;
            }
            Ok(s)
        }
    }
}

/// 改变当前工作目录
external "c" function chdir(path: *character) -> signed 32-bit integer

/// 改变当前工作目录
export fn change_dir(path: string) -> Result<unit, string> {
    unsafe {
        let result = chdir(path as *character);
        when result == 0 {
            Ok(unit)
        } else {
            Err("failed to change directory to: " ++ path)
        }
    }
}

/// 睡眠指定秒数
export fn sleep_sec(seconds: Int) -> unit {
    unsafe {
        sleep(seconds as signed 32-bit integer);
    }
}

/// 检查程序是否在 POSIX 系统上运行
export fn is_posix() -> Bool {
    // 大部分系统都是 POSIX，这里返回 true，实际由运行时决定
    true
}
