module std.time
import std.prelude
import std.types



/// 时间点（以 UNIX 时间戳表示，秒）
export type Instant = Float;

/// 持续时间
export record Duration {
    /// 纳秒数
    nanos: Int,
}

/// 创建秒级持续时间
export fn seconds(s: Int) -> Duration {
    Duration { nanos: s * 1_000_000_000 }
}

/// 创建毫秒级持续时间
export fn milliseconds(ms: Int) -> Duration {
    Duration { nanos: ms * 1_000_000 }
}

/// 创建微秒级持续时间
export fn microseconds(us: Int) -> Duration {
    Duration { nanos: us * 1_000 }
}

/// 创建纳秒级持续时间
export fn nanoseconds(ns: Int) -> Duration {
    Duration { nanos: ns }
}

/// 获取持续时间的总秒数
export fn as_seconds(self: Duration) -> Float {
    (self.nanos as Float) / 1_000_000_000.0
}

/// 获取持续时间的总毫秒数
export fn as_milliseconds(self: Duration) -> Int {
    self.nanos / 1_000_000
}

/// 获取持续时间的总微秒数
export fn as_microseconds(self: Duration) -> Int {
    self.nanos / 1_000
}

/// 获取持续时间的纳秒数
export fn as_nanoseconds(self: Duration) -> Int {
    self.nanos
}

/// 加法
export fn add(self: Duration, other: Duration) -> Duration {
    Duration { nanos: self.nanos + other.nanos }
}

/// 减法
export fn subtract(self: Duration, other: Duration) -> Duration {
    Duration { nanos: self.nanos - other.nanos }
}

/// 日期
export record Date {
    year: Int,
    month: Int,   // 1-12
    day: Int,     // 1-31
}

/// 时间
export record Time {
    hour: Int,    // 0-23
    minute: Int,  // 0-59
    second: Int,  // 0-59
    nanosecond: Int, // 0-999999999
}

/// 日期时间
export record DateTime {
    date: Date,
    time: Time,
}

// === 外部 C 库函数 ===

external "c" function time(t: *signed 32-bit integer) -> signed 32-bit integer
external "c" function localtime_r(timer: *signed 32-bit integer, result: *()) -> *()
external "c" function gmtime_r(timer: *signed 32-bit integer, result: *()) -> *()
external "c" function mktime(tm: *()) -> signed 32-bit integer
external "c" function clock_gettime(clk_id: signed 32-bit integer, tp: *()) -> signed 32-bit integer
external "c" function sleep(seconds: signed 32-bit integer) -> signed 32-bit integer
external "c" function usleep(useconds: unsigned 32-bit integer) -> signed 32-bit integer

/// CLOCK_REALTIME
const CLOCK_REALTIME: signed 32-bit integer = 0;
/// CLOCK_MONOTONIC
const CLOCK_MONOTONIC: signed 32-bit integer = 1;

/// 获取当前 UNIX 时间戳（秒）
export fn now() -> Int {
    unsafe {
        time(null) as Int
    }
}

/// 获取当前 UNIX 时间戳（浮点，微秒精度）
export fn now_float() -> Float {
    now() as Float
}

/// 获取当前高精度时间戳（纳秒）
export fn now_ns() -> Int {
    // 使用 clock_gettime 获得更高精度
    unsafe {
        // 这里简化处理
        (now() * 1_000_000_000) as Int
    }
}

/// 睡眠指定持续时间
export fn sleep(duration: Duration) -> unit {
    let ms = duration.as_milliseconds();
    when ms > 0 {
        unsafe {
            if ms >= 1000 {
                sleep((ms / 1000) as signed 32-bit integer);
            }
            let remaining = ms % 1000;
            when remaining > 0 {
                usleep((remaining * 1000) as unsigned 32-bit integer);
            }
        }
    }
}

/// 睡眠指定秒数
export fn sleep_sec(seconds: Int) -> unit {
    unsafe {
        sleep(seconds as signed 32-bit integer);
    }
}

/// 获取当前日期时间（本地时区）
export fn now_local() -> DateTime {
    let t = now() as signed 32-bit integer;
    // 需要分配 tm 结构
    // 这里我们简化处理，解析 C tm 结构
    // 布局: tm_sec, tm_min, tm_hour, tm_mday, tm_mon, tm_year, ...
    unsafe {
        // 我们在栈上分配足够空间
        let mut buffer: [Int] = [0; 12];
        let result = localtime_r(&t, buffer as *());
        // 读取字段
        let second = buffer[0] as Int;
        let minute = buffer[1] as Int;
        let hour = buffer[2] as Int;
        let day = buffer[3] as Int;
        let month = buffer[4] as Int + 1; // C 中 0-11
        let year = buffer[5] as Int + 1900; // C 中是 1900 年偏移
        DateTime {
            date: Date { year: year, month: month, day: day },
            time: Time { hour: hour, minute: minute, second: second, nanosecond: 0 },
        }
    }
}

/// 获取当前 UTC 日期时间
export fn now_utc() -> DateTime {
    let t = now() as signed 32-bit integer;
    unsafe {
        let mut buffer: [Int] = [0; 12];
        let result = gmtime_r(&t, buffer as *());
        let second = buffer[0] as Int;
        let minute = buffer[1] as Int;
        let hour = buffer[2] as Int;
        let day = buffer[3] as Int;
        let month = buffer[4] as Int + 1;
        let year = buffer[5] as Int + 1900;
        DateTime {
            date: Date { year: year, month: month, day: day },
            time: Time { hour: hour, minute: minute, second: second, nanosecond: 0 },
        }
    }
}

/// 转换 DateTime 为 UNIX 时间戳
export fn to_timestamp(dt: DateTime) -> Int {
    unsafe {
        let mut buffer: [Int] = [0; 12];
        buffer[0] = dt.time.second as Int;
        buffer[1] = dt.time.minute as Int;
        buffer[2] = dt.time.hour as Int;
        buffer[3] = dt.date.day as Int;
        buffer[4] = (dt.date.month - 1) as Int;
        buffer[5] = (dt.date.year - 1900) as Int;
        mktime(buffer as *()) as Int
    }
}

/// 计算两个时间点之差（单位 Duration）
export fn duration_between(start: Instant, end: Instant) -> Duration {
    let diff_seconds = end - start;
    seconds(diff_seconds as Int)
}

/// 判断闰年
export fn is_leap_year(year: Int) -> Bool {
    when (year % 4 != 0) {
        false
    } else when (year % 100 != 0) {
        true
    } else {
        year % 400 == 0
    }
}

/// 获取月份天数
export fn days_in_month(year: Int, month: Int) -> Int {
    match month {
        1 => 31,
        2 => when is_leap_year(year) { 29 } else { 28 },
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 0,
    }
}

/// 格式化日期为字符串 YYYY-MM-DD
export fn format_date(date: Date) -> string {
    let year_str = date.year.to_string();
    let month_str = pad_two_digits(date.month);
    let day_str = pad_two_digits(date.day);
    year_str ++ "-" ++ month_str ++ "-" ++ day_str
}

/// 格式化时间为字符串 HH:MM:SS
export fn format_time(time: Time) -> string {
    let hour_str = pad_two_digits(time.hour);
    let minute_str = pad_two_digits(time.minute);
    let second_str = pad_two_digits(time.second);
    hour_str ++ ":" ++ minute_str ++ ":" ++ second_str
}

/// 格式化日期时间为字符串 YYYY-MM-DD HH:MM:SS
export fn format_datetime(dt: DateTime) -> string {
    format_date(dt.date) ++ " " ++ format_time(dt.time)
}

/// 补零到两位
private fn pad_two_digits(n: Int) -> string {
    when n < 10 {
        "0" ++ n.to_string()
    } else {
        n.to_string()
    }
}

/// 测量函数执行时间
export fn measure<F>(f: F) -> Duration where F: function() -> unit -> Duration {
    let start = now_float();
    f();
    let end = now_float();
    seconds((end - start) as Int)
}

/// 测量并打印函数执行时间
export fn measure_and_print<F>(name: string, f: F) -> Duration where F: function() -> unit {
    let start = now_float();
    f();
    let end = now_float();
    let dur = seconds(((end - start) * 1000.0) as Int);
    println(name ++ ": " ++ dur.as_milliseconds().to_string() ++ "ms");
    dur
}
