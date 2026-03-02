// X语言标准库 - 时间处理
//
// 时间获取、格式化、睡眠等操作

// ==========================================
// 时间类型
// ==========================================

/// 时间点（自 Unix 纪元以来的秒数和纳秒数）
type Time = {
  seconds: Int,
  nanoseconds: Int,
}

/// 持续时间
type Duration = {
  seconds: Int,
  nanoseconds: Int,
}

/// 日历日期时间
type DateTime = {
  year: Int,
  month: Int,      // 1-12
  day: Int,        // 1-31
  hour: Int,       // 0-23
  minute: Int,     // 0-59
  second: Int,     // 0-59
  weekday: Int,    // 0=Sunday, 1=Monday, ..., 6=Saturday
  yearday: Int,    // 1-366
  is_dst: Bool,    // 是否夏令时
}

// ==========================================
// 时间常量
// ==========================================

/// 1秒 = 1_000_000_000 纳秒
let NANOS_PER_SECOND: Int = 1_000_000_000

/// 1毫秒 = 1_000_000 纳秒
let NANOS_PER_MILLISECOND: Int = 1_000_000

/// 1微秒 = 1_000 纳秒
let NANOS_PER_MICROSECOND: Int = 1_000

/// 1分钟 = 60秒
let SECONDS_PER_MINUTE: Int = 60

/// 1小时 = 3600秒
let SECONDS_PER_HOUR: Int = 3600

/// 1天 = 86400秒
let SECONDS_PER_DAY: Int = 86400

// ==========================================
// 当前时间
// ==========================================

/// 获取当前时间（自 Unix 纪元以来的秒数）
fun timestamp(): Int {
  // 内置函数
  "__builtin_timestamp"
}

/// 获取当前时间（自 Unix 纪元以来的毫秒数）
fun timestamp_millis(): Int {
  // 内置函数
  "__builtin_timestamp_millis"
}

/// 获取当前时间（自 Unix 纪元以来的微秒数）
fun timestamp_micros(): Int {
  // 内置函数
  "__builtin_timestamp_micros"
}

/// 获取当前时间（自 Unix 纪元以来的纳秒数）
fun timestamp_nanos(): Int {
  // 内置函数
  "__builtin_timestamp_nanos"
}

/// 获取当前时间点
fun now(): Time {
  let secs = timestamp()
  let nanos = timestamp_nanos() % NANOS_PER_SECOND
  { seconds: secs, nanoseconds: nanos }
}

// ==========================================
// 睡眠
// ==========================================

/// 睡眠指定秒数
fun sleep(seconds: Float) {
  // 内置函数
  "__builtin_sleep"
}

/// 睡眠指定毫秒数
fun sleep_ms(milliseconds: Int) {
  sleep(milliseconds as Float / 1000.0)
}

/// 睡眠指定微秒数
fun sleep_us(microseconds: Int) {
  sleep(microseconds as Float / 1_000_000.0)
}

/// 睡眠指定纳秒数
fun sleep_ns(nanoseconds: Int) {
  sleep(nanoseconds as Float / 1_000_000_000.0)
}

/// 睡眠指定持续时间
fun sleep_duration(duration: Duration) {
  let total_seconds = duration.seconds as Float + duration.nanoseconds as Float / 1_000_000_000.0
  sleep(total_seconds)
}

// ==========================================
// Duration 构造函数
// ==========================================

/// 创建持续时间（秒）
fun duration_seconds(seconds: Int): Duration {
  { seconds: seconds, nanoseconds: 0 }
}

/// 创建持续时间（毫秒）
fun duration_millis(milliseconds: Int): Duration {
  let secs = milliseconds / 1000
  let nanos = (milliseconds % 1000) * NANOS_PER_MILLISECOND
  { seconds: secs, nanoseconds: nanos }
}

/// 创建持续时间（微秒）
fun duration_micros(microseconds: Int): Duration {
  let secs = microseconds / 1_000_000
  let nanos = (microseconds % 1_000_000) * NANOS_PER_MICROSECOND
  { seconds: secs, nanoseconds: nanos }
}

/// 创建持续时间（纳秒）
fun duration_nanos(nanoseconds: Int): Duration {
  let secs = nanoseconds / NANOS_PER_SECOND
  let nanos = nanoseconds % NANOS_PER_SECOND
  { seconds: secs, nanoseconds: nanos }
}

/// 创建持续时间（分钟）
fun duration_minutes(minutes: Int): Duration {
  duration_seconds(minutes * SECONDS_PER_MINUTE)
}

/// 创建持续时间（小时）
fun duration_hours(hours: Int): Duration {
  duration_seconds(hours * SECONDS_PER_HOUR)
}

/// 创建持续时间（天）
fun duration_days(days: Int): Duration {
  duration_seconds(days * SECONDS_PER_DAY)
}

// ==========================================
// Duration 操作
// ==========================================

/// 获取持续时间的总秒数
fun duration_as_seconds(d: Duration): Float {
  d.seconds as Float + d.nanoseconds as Float / 1_000_000_000.0
}

/// 获取持续时间的总毫秒数
fun duration_as_millis(d: Duration): Int {
  d.seconds * 1000 + d.nanoseconds / NANOS_PER_MILLISECOND
}

/// 获取持续时间的总微秒数
fun duration_as_micros(d: Duration): Int {
  d.seconds * 1_000_000 + d.nanoseconds / NANOS_PER_MICROSECOND
}

/// 获取持续时间的总纳秒数
fun duration_as_nanos(d: Duration): Int {
  d.seconds * NANOS_PER_SECOND + d.nanoseconds
}

/// 两个持续时间相加
fun duration_add(a: Duration, b: Duration): Duration {
  let mut secs = a.seconds + b.seconds
  let mut nanos = a.nanoseconds + b.nanoseconds
  if nanos >= NANOS_PER_SECOND {
    secs = secs + 1
    nanos = nanos - NANOS_PER_SECOND
  }
  { seconds: secs, nanoseconds: nanos }
}

/// 两个持续时间相减
fun duration_sub(a: Duration, b: Duration): Duration {
  let mut secs = a.seconds - b.seconds
  let mut nanos = a.nanoseconds - b.nanoseconds
  if nanos < 0 {
    secs = secs - 1
    nanos = nanos + NANOS_PER_SECOND
  }
  { seconds: secs, nanoseconds: nanos }
}

/// 比较两个持续时间
fun duration_compare(a: Duration, b: Duration): Int {
  if a.seconds != b.seconds {
    if a.seconds < b.seconds { -1 }
    else { 1 }
  } else {
    if a.nanoseconds < b.nanoseconds { -1 }
    else if a.nanoseconds > b.nanoseconds { 1 }
    else { 0 }
  }
}

// ==========================================
// Time 操作
// ==========================================

/// 计算两个时间点的差
fun time_diff(a: Time, b: Time): Duration {
  let mut secs = a.seconds - b.seconds
  let mut nanos = a.nanoseconds - b.nanoseconds
  if nanos < 0 {
    secs = secs - 1
    nanos = nanos + NANOS_PER_SECOND
  }
  { seconds: secs, nanoseconds: nanos }
}

/// 给时间点加上持续时间
fun time_add(t: Time, d: Duration): Time {
  let mut secs = t.seconds + d.seconds
  let mut nanos = t.nanoseconds + d.nanoseconds
  if nanos >= NANOS_PER_SECOND {
    secs = secs + 1
    nanos = nanos - NANOS_PER_SECOND
  }
  { seconds: secs, nanoseconds: nanos }
}

/// 给时间点减去持续时间
fun time_sub(t: Time, d: Duration): Time {
  let mut secs = t.seconds - d.seconds
  let mut nanos = t.nanoseconds - d.nanoseconds
  if nanos < 0 {
    secs = secs - 1
    nanos = nanos + NANOS_PER_SECOND
  }
  { seconds: secs, nanoseconds: nanos }
}

/// 比较两个时间点
fun time_compare(a: Time, b: Time): Int {
  if a.seconds != b.seconds {
    if a.seconds < b.seconds { -1 }
    else { 1 }
  } else {
    if a.nanoseconds < b.nanoseconds { -1 }
    else if a.nanoseconds > b.nanoseconds { 1 }
    else { 0 }
  }
}

// ==========================================
// 日历时间
// ==========================================

/// 将时间戳转换为本地日历时间
fun to_local_datetime(seconds: Int): DateTime {
  // 内置函数
  "__builtin_to_local_datetime"
}

/// 将时间戳转换为 UTC 日历时间
fun to_utc_datetime(seconds: Int): DateTime {
  // 内置函数
  "__builtin_to_utc_datetime"
}

/// 将日历时间转换为时间戳
fun from_datetime(dt: DateTime): Int {
  // 内置函数
  "__builtin_from_datetime"
}

/// 获取当前本地时间
fn local_now(): DateTime {
  to_local_datetime(timestamp())
}

/// 获取当前 UTC 时间
fun utc_now(): DateTime {
  to_utc_datetime(timestamp())
}

// ==========================================
// 时间格式化
// ==========================================

/// 格式化日期时间为字符串
fun format_datetime(dt: DateTime, format: String): String {
  // 简单实现，支持基本占位符
  let mut result = format
  result = str_replace(result, "%Y", format_int(dt.year, 4))
  result = str_replace(result, "%m", format_int(dt.month, 2))
  result = str_replace(result, "%d", format_int(dt.day, 2))
  result = str_replace(result, "%H", format_int(dt.hour, 2))
  result = str_replace(result, "%M", format_int(dt.minute, 2))
  result = str_replace(result, "%S", format_int(dt.second, 2))
  result
}

/// 格式化为 ISO 8601 格式
fun format_iso8601(dt: DateTime): String {
  format_int(dt.year, 4) + "-" +
  format_int(dt.month, 2) + "-" +
  format_int(dt.day, 2) + "T" +
  format_int(dt.hour, 2) + ":" +
  format_int(dt.minute, 2) + ":" +
  format_int(dt.second, 2)
}

/// 简单的日期时间字符串表示
fun datetime_to_string(dt: DateTime): String {
  format_iso8601(dt)
}

// ==========================================
// 工作日名称
// ==========================================

/// 获取工作日名称
fun weekday_name(weekday: Int): String {
  let names = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
  if weekday >= 0 && weekday < 7 {
    names[weekday]
  } else {
    "Unknown"
  }
}

/// 获取工作日缩写
fun weekday_abbr(weekday: Int): String {
  let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
  if weekday >= 0 && weekday < 7 {
    names[weekday]
  } else {
    "???"
  }
}

// ==========================================
// 月份名称
// ==========================================

/// 获取月份名称
fun month_name(month: Int): String {
  let names = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
  ]
  if month >= 1 && month <= 12 {
    names[month - 1]
  } else {
    "Unknown"
  }
}

/// 获取月份缩写
fun month_abbr(month: Int): String {
  let names = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
  ]
  if month >= 1 && month <= 12 {
    names[month - 1]
  } else {
    "???"
  }
}

// ==========================================
// 性能测量
// ==========================================

/// 测量函数执行时间（秒）
fun time_it<T>(f: () -> T): (T, Float) {
  let start = timestamp_nanos()
  let result = f()
  let end = timestamp_nanos()
  let elapsed = (end - start) as Float / 1_000_000_000.0
  (result, elapsed)
}

/// 测量函数执行时间并打印
fun time_it_print<T>(label: String, f: () -> T): T {
  let (result, elapsed) = time_it(f)
  println(label + ": " + format_float(elapsed, 6) + "s")
  result
}
