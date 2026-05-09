module std.error
import std.prelude
import std.types



// ============================================================================
// Error Trait - 错误接口
// ============================================================================

/// 错误特征 - 所有错误类型必须实现此特征
/// 用于统一的错误处理和转换
export trait Error {
    /// 获取错误消息
    function message(self) -> string;
    
    /// 获取错误代码（可选）
    function code(self) -> Option<Int>;
    
    /// 获取错误来源（可选）
    function source(self) -> Option<Error>;
    
    /// 获取栈回溯（可选）
    function stack_trace(self) -> Option<StackTrace>;
}

// ============================================================================
// StackTrace - 栈回溯信息
// ============================================================================

/// 栈帧信息
export record StackFrame {
    /// 函数名
    public function_name: string,
    /// 文件名
    public file: string,
    /// 行号
    public line: Int,
    /// 列号
    public column: Int,
}

/// 栈回溯信息
export record StackTrace {
    /// 栈帧列表
    public frames: [StackFrame],
    /// 创建时间戳
    public timestamp: Int,
}

/// 创建空栈回溯
export function empty_stack_trace() -> StackTrace {
    StackTrace {
        frames: [],
        timestamp: 0,
    }
}

/// 获取当前栈回溯
/// 这是一个外部函数，需要平台特定的实现
external "c" function __capture_stack_trace(frames: *mut StackFrame, max_frames: Int) -> Int

/// 捕获当前栈回溯
export function capture_stack_trace() -> StackTrace {
    // 预分配 64 个栈帧的空间
    let mut frames: [StackFrame] = [];
    let mut i = 0;
    while i < 64 {
        frames.push(StackFrame {
            function_name: "",
            file: "",
            line: 0,
            column: 0,
        });
        i = i + 1;
    }
    
    unsafe {
        let count = __capture_stack_trace(&frames[0] as *mut StackFrame, 64);
        // 截取实际捕获的帧数
        frames = frames.slice(0, count);
    }
    
    StackTrace {
        frames: frames,
        timestamp: current_time_millis(),
    }
}

/// 格式化栈回溯为字符串
export function format_stack_trace(trace: StackTrace) -> string {
    let mut result = "Stack trace:\n";
    for (index, frame) in enumerate(trace.frames) {
        result = result ++ "  " ++ (index as string) ++ ": ";
        result = result ++ frame.function_name;
        result = result ++ " at " ++ frame.file;
        result = result ++ ":" ++ (frame.line as string);
        result = result ++ ":" ++ (frame.column as string);
        result = result ++ "\n";
    }
    result
}

/// 获取当前时间戳（毫秒）
external "c" function __time_millis() -> Int

function current_time_millis() -> Int {
    unsafe {
        __time_millis()
    }
}

// ============================================================================
// ErrorStack - 错误栈追踪
// ============================================================================

/// 错误栈追踪 - 记录错误传播路径
export record ErrorStack {
    /// 错误消息
    public message: string,
    /// 错误代码
    public error_code: Option<Int>,
    /// 栈回溯
    public trace: Option<StackTrace>,
    /// 来源错误
    public source: Option<ErrorStack>,
    /// 文件名
    public file: string,
    /// 行号
    public line: Int,
}

/// 创建简单错误栈
export function simple_error(message: string) -> ErrorStack {
    ErrorStack {
        message: message,
        error_code: None,
        trace: None,
        source: None,
        file: "",
        line: 0,
    }
}

/// 创建带位置的错误栈
export function error_at(message: string, file: string, line: Int) -> ErrorStack {
    ErrorStack {
        message: message,
        error_code: None,
        trace: Some(capture_stack_trace()),
        source: None,
        file: file,
        line: line,
    }
}

/// 创建带错误代码的错误栈
export function error_with_code(message: string, code: Int) -> ErrorStack {
    ErrorStack {
        message: message,
        error_code: Some(code),
        trace: None,
        source: None,
        file: "",
        line: 0,
    }
}

/// 创建完整错误栈
export function full_error(
    message: string,
    code: Option<Int>,
    file: string,
    line: Int,
    source: Option<ErrorStack>
) -> ErrorStack {
    ErrorStack {
        message: message,
        error_code: code,
        trace: Some(capture_stack_trace()),
        source: source,
        file: file,
        line: line,
    }
}

/// 添加上下文到错误栈
export function add_context(self: ErrorStack, context: string) -> ErrorStack {
    ErrorStack {
        message: context ++ ": " ++ self.message,
        error_code: self.error_code,
        trace: self.trace,
        source: Some(self),
        file: self.file,
        line: self.line,
    }
}

/// 格式化错误栈为字符串
export function format_error_stack(error: ErrorStack) -> string {
    let mut result = "Error: " ++ error.message;
    
    match error.error_code {
        Some(code) => {
            result = result ++ " (code: " ++ (code as string) ++ ")";
        },
        None => {},
    }
    
    when error.file != "" {
        result = result ++ "\n  at " ++ error.file ++ ":" ++ (error.line as string);
    }
    
    match error.trace {
        Some(trace) => {
            result = result ++ "\n" ++ format_stack_trace(trace);
        },
        None => {},
    }
    
    match error.source {
        Some(source) => {
            result = result ++ "\n\nCaused by:\n" ++ format_error_stack(source);
        },
        None => {},
    }
    
    result
}

// ============================================================================
// 错误传播辅助函数
// ============================================================================

/// 解包 Result，如果错误则返回带上下文的错误
export function unwrap_or_propagate<T>(result: Result<T, ErrorStack>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic(format_error_stack(err)),
    }
}

/// 解包 Result，如果错误则返回默认值
export function unwrap_or_default<T>(result: Result<T, ErrorStack>, default: T) -> T {
    match result {
        Ok(value) => value,
        Err(_) => default,
    }
}

/// 解包 Option，如果 None 则返回带上下文的错误
export function unwrap_option<T>(opt: Option<T>, message: string) -> T {
    match opt {
        Some(value) => value,
        None => panic(message),
    }
}

/// 将 Option 转换为 Result
export function option_to_result<T>(opt: Option<T>, error: ErrorStack) -> Result<T, ErrorStack> {
    match opt {
        Some(value) => Ok(value),
        None => Err(error),
    }
}

// ============================================================================
// 错误链构建器
// ============================================================================

/// 错误链构建器 - 用于逐步构建复杂错误
export record ErrorBuilder {
    public message: string,
    public code: Option<Int>,
    public file: string,
    public line: Int,
    public source: Option<ErrorStack>,
}

/// 创建错误构建器
export function error_builder(message: string) -> ErrorBuilder {
    ErrorBuilder {
        message: message,
        code: None,
        file: "",
        line: 0,
        source: None,
    }
}

/// 设置错误代码
export function with_code(self: ErrorBuilder, code: Int) -> ErrorBuilder {
    ErrorBuilder {
        message: self.message,
        code: Some(code),
        file: self.file,
        line: self.line,
        source: self.source,
    }
}

/// 设置位置
export function at(self: ErrorBuilder, file: string, line: Int) -> ErrorBuilder {
    ErrorBuilder {
        message: self.message,
        code: self.code,
        file: file,
        line: line,
        source: self.source,
    }
}

/// 设置来源错误
export function caused_by(self: ErrorBuilder, source: ErrorStack) -> ErrorBuilder {
    ErrorBuilder {
        message: self.message,
        code: self.code,
        file: self.file,
        line: self.line,
        source: Some(source),
    }
}

/// 构建错误栈
export function build(self: ErrorBuilder) -> ErrorStack {
    ErrorStack {
        message: self.message,
        error_code: self.code,
        trace: Some(capture_stack_trace()),
        source: self.source,
        file: self.file,
        line: self.line,
    }
}

// ============================================================================
// 错误类型转换
// ============================================================================

/// 将字符串转换为错误
export function string_to_error(s: string) -> ErrorStack {
    simple_error(s)
}

/// 将 Result<string, E> 转换为 Result<T, ErrorStack>
export function map_err<T, E>(result: Result<T, E>, mapper: function(E) -> ErrorStack) -> Result<T, ErrorStack> {
    match result {
        Ok(value) => Ok(value),
        Err(err) => Err(mapper(err)),
    }
}
