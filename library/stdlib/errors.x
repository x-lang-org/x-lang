module std.errors
import std.prelude
import std.types




// ============================================================================
// 基础错误类型
// ============================================================================

/// 错误分类
export enum ErrorCategory {
    /// IO 错误
    Io,
    /// 解析错误
    Parse,
    /// 类型错误
    Type,
    /// 运行时错误
    Runtime,
    /// 逻辑错误
    Logic,
    /// 配置错误
    Config,
    /// 网络错误
    Network,
    /// 数据库错误
    Database,
    /// 验证错误
    Validation,
    /// 权限错误
    Permission,
    /// 超时错误
    Timeout,
    /// 资源错误
    Resource,
    /// 未知错误
    Unknown,
}

/// 基础错误记录
export record BaseError {
    public category: ErrorCategory,
    public message: string,
    public code: Int,
    public file: string,
    public line: Int,
}

/// 创建基础错误
export function base_error(category: ErrorCategory, message: string) -> BaseError {
    BaseError {
        category: category,
        message: message,
        code: 0,
        file: "",
        line: 0,
    }
}

/// 设置错误位置
export function at_location(self: BaseError, file: string, line: Int) -> BaseError {
    BaseError {
        category: self.category,
        message: self.message,
        code: self.code,
        file: file,
        line: line,
    }
}

/// 设置错误代码
export function with_error_code(self: BaseError, code: Int) -> BaseError {
    BaseError {
        category: self.category,
        message: self.message,
        code: code,
        file: self.file,
        line: self.line,
    }
}

/// 转换为错误栈
export function to_error_stack(self: BaseError) -> ErrorStack {
    ErrorStack {
        message: format_category(self.category) ++ ": " ++ self.message,
        error_code: Some(self.code),
        trace: None,
        source: None,
        file: self.file,
        line: self.line,
    }
}

/// 格式化错误分类
function format_category(category: ErrorCategory) -> string {
    match category {
        ErrorCategory.Io => "IoError",
        ErrorCategory.Parse => "ParseError",
        ErrorCategory.Type => "TypeError",
        ErrorCategory.Runtime => "RuntimeError",
        ErrorCategory.Logic => "LogicError",
        ErrorCategory.Config => "ConfigError",
        ErrorCategory.Network => "NetworkError",
        ErrorCategory.Database => "DatabaseError",
        ErrorCategory.Validation => "ValidationError",
        ErrorCategory.Permission => "PermissionError",
        ErrorCategory.Timeout => "TimeoutError",
        ErrorCategory.Resource => "ResourceError",
        ErrorCategory.Unknown => "UnknownError",
    }
}

// ============================================================================
// IoError - IO 错误
// ============================================================================

/// IO 错误类型
export enum IoErrorKind {
    /// 文件未找到
    FileNotFound,
    /// 权限被拒绝
    PermissionDenied,
    /// 连接重置
    ConnectionReset,
    /// 连接断开
    ConnectionAborted,
    /// 没有连接
    NotConnected,
    /// 地址已被使用
    AddrInUse,
    /// 地址不可用
    AddrNotAvailable,
    /// 管道破裂
    BrokenPipe,
    /// 已有数据
    AlreadyExists,
    /// 无效输入
    InvalidInput,
    /// 无效数据
    InvalidData,
    /// 超时
    TimedOut,
    /// 写入零字节
    WriteZero,
    /// 中断
    Interrupted,
    /// 其他
    Other,
}

/// IO 错误
export record IoError {
    public kind: IoErrorKind,
    public message: string,
    public path: string,
}

/// 创建 IO 错误
export function io_error(kind: IoErrorKind, message: string) -> IoError {
    IoError {
        kind: kind,
        message: message,
        path: "",
    }
}

/// 创建文件未找到错误
export function file_not_found(path: string) -> IoError {
    IoError {
        kind: IoErrorKind.FileNotFound,
        message: "file not found: " ++ path,
        path: path,
    }
}

/// 创建权限被拒绝错误
export function permission_denied(path: string) -> IoError {
    IoError {
        kind: IoErrorKind.PermissionDenied,
        message: "permission denied: " ++ path,
        path: path,
    }
}

/// 创建连接错误
export function connection_error(message: string) -> IoError {
    IoError {
        kind: IoErrorKind.ConnectionReset,
        message: message,
        path: "",
    }
}

/// 转换为错误栈
export function io_to_error_stack(self: IoError) -> ErrorStack {
    ErrorStack {
        message: "IoError: " ++ self.message,
        error_code: Some(io_error_code(self.kind)),
        trace: None,
        source: None,
        file: "",
        line: 0,
    }
}

/// IO 错误代码
function io_error_code(kind: IoErrorKind) -> Int {
    match kind {
        IoErrorKind.FileNotFound => 1,
        IoErrorKind.PermissionDenied => 2,
        IoErrorKind.ConnectionReset => 3,
        IoErrorKind.ConnectionAborted => 4,
        IoErrorKind.NotConnected => 5,
        IoErrorKind.AddrInUse => 6,
        IoErrorKind.AddrNotAvailable => 7,
        IoErrorKind.BrokenPipe => 8,
        IoErrorKind.AlreadyExists => 9,
        IoErrorKind.InvalidInput => 10,
        IoErrorKind.InvalidData => 11,
        IoErrorKind.TimedOut => 12,
        IoErrorKind.WriteZero => 13,
        IoErrorKind.Interrupted => 14,
        IoErrorKind.Other => 15,
    }
}

// ============================================================================
// ParseError - 解析错误
// ============================================================================

/// 解析错误
export record ParseError {
    public message: string,
    public file: string,
    public line: Int,
    public column: Int,
    public source_line: string,
}

/// 创建解析错误
export function parse_error(message: string) -> ParseError {
    ParseError {
        message: message,
        file: "",
        line: 0,
        column: 0,
        source_line: "",
    }
}

/// 创建带位置的解析错误
export function parse_error_at(
    message: string,
    file: string,
    line: Int,
    column: Int,
    source_line: string
) -> ParseError {
    ParseError {
        message: message,
        file: file,
        line: line,
        column: column,
        source_line: source_line,
    }
}

/// 转换为错误栈
export function parse_to_error_stack(self: ParseError) -> ErrorStack {
    let mut msg = "ParseError: " ++ self.message;
    when self.file != "" {
        msg = msg ++ " at " ++ self.file ++ ":" ++ (self.line as string) ++ ":" ++ (self.column as string);
    }
    ErrorStack {
        message: msg,
        error_code: Some(100),
        trace: None,
        source: None,
        file: self.file,
        line: self.line,
    }
}

/// 格式化解析错误（带源码指针）
export function format_parse_error(self: ParseError) -> string {
    let mut result = "ParseError: " ++ self.message ++ "\n";
    when self.file != "" {
        result = result ++ "  --> " ++ self.file ++ ":" ++ (self.line as string) ++ ":" ++ (self.column as string) ++ "\n";
    }
    when self.source_line != "" {
        result = result ++ "   | " ++ self.source_line ++ "\n";
        result = result ++ "   | ";
        let mut i = 0;
        while i < self.column - 1 {
            result = result ++ " ";
            i = i + 1;
        }
        result = result ++ "^\n";
    }
    result
}

// ============================================================================
// TypeError - 类型错误
// ============================================================================

/// 类型错误
export record TypeError {
    public expected: string,
    public actual: string,
    public message: string,
    public file: string,
    public line: Int,
}

/// 创建类型错误
export function type_error(expected: string, actual: string) -> TypeError {
    TypeError {
        expected: expected,
        actual: actual,
        message: "expected " ++ expected ++ ", got " ++ actual,
        file: "",
        line: 0,
    }
}

/// 创建带消息的类型错误
export function type_error_msg(message: string) -> TypeError {
    TypeError {
        expected: "",
        actual: "",
        message: message,
        file: "",
        line: 0,
    }
}

/// 转换为错误栈
export function type_to_error_stack(self: TypeError) -> ErrorStack {
    ErrorStack {
        message: "TypeError: " ++ self.message,
        error_code: Some(200),
        trace: None,
        source: None,
        file: self.file,
        line: self.line,
    }
}

// ============================================================================
// RuntimeError - 运行时错误
// ============================================================================

/// 运行时错误
export record RuntimeError {
    public message: string,
    public code: Int,
    public recoverable: Bool,
}

/// 创建运行时错误
export function runtime_error(message: string) -> RuntimeError {
    RuntimeError {
        message: message,
        code: 0,
        recoverable: false,
    }
}

/// 创建可恢复的运行时错误
export function recoverable_runtime_error(message: string, code: Int) -> RuntimeError {
    RuntimeError {
        message: message,
        code: code,
        recoverable: true,
    }
}

/// 创建不可恢复的运行时错误
export function fatal_runtime_error(message: string, code: Int) -> RuntimeError {
    RuntimeError {
        message: message,
        code: code,
        recoverable: false,
    }
}

/// 转换为错误栈
export function runtime_to_error_stack(self: RuntimeError) -> ErrorStack {
    ErrorStack {
        message: "RuntimeError: " ++ self.message,
        error_code: Some(self.code),
        trace: None,
        source: None,
        file: "",
        line: 0,
    }
}

// ============================================================================
// 验证错误
// ============================================================================

/// 验证错误
export record ValidationError {
    public field: string,
    public message: string,
    public value: string,
}

/// 创建验证错误
export function validation_error(field: string, message: string) -> ValidationError {
    ValidationError {
        field: field,
        message: message,
        value: "",
    }
}

/// 创建带值的验证错误
export function validation_error_value(field: string, message: string, value: string) -> ValidationError {
    ValidationError {
        field: field,
        message: message,
        value: value,
    }
}

/// 转换为错误栈
export function validation_to_error_stack(self: ValidationError) -> ErrorStack {
    let mut msg = "ValidationError: " ++ self.field ++ " - " ++ self.message;
    when self.value != "" {
        msg = msg ++ " (value: " ++ self.value ++ ")";
    }
    ErrorStack {
        message: msg,
        error_code: Some(300),
        trace: None,
        source: None,
        file: "",
        line: 0,
    }
}

// ============================================================================
// 网络错误
// ============================================================================

/// 网络错误
export record NetworkError {
    public message: string,
    public code: Int,
    public url: string,
    public retryable: Bool,
}

/// 创建网络错误
export function network_error(message: string) -> NetworkError {
    NetworkError {
        message: message,
        code: 0,
        url: "",
        retryable: false,
    }
}

/// 创建带 URL 的网络错误
export function network_error_url(message: string, url: string) -> NetworkError {
    NetworkError {
        message: message,
        code: 0,
        url: url,
        retryable: false,
    }
}

/// 创建可重试的网络错误
export function network_error_retryable(message: string, code: Int) -> NetworkError {
    NetworkError {
        message: message,
        code: code,
        url: "",
        retryable: true,
    }
}

/// 转换为错误栈
export function network_to_error_stack(self: NetworkError) -> ErrorStack {
    let mut msg = "NetworkError: " ++ self.message;
    when self.url != "" {
        msg = msg ++ " (url: " ++ self.url ++ ")";
    }
    ErrorStack {
        message: msg,
        error_code: Some(self.code),
        trace: None,
        source: None,
        file: "",
        line: 0,
    }
}

// ============================================================================
// 超时错误
// ============================================================================

/// 超时错误
export record TimeoutError {
    public operation: string,
    public timeout_ms: Int,
}

/// 创建超时错误
export function timeout_error(operation: string, timeout_ms: Int) -> TimeoutError {
    TimeoutError {
        operation: operation,
        timeout_ms: timeout_ms,
    }
}

/// 转换为错误栈
export function timeout_to_error_stack(self: TimeoutError) -> ErrorStack {
    ErrorStack {
        message: "TimeoutError: " ++ self.operation ++ " timed out after " ++ (self.timeout_ms as string) ++ "ms",
        error_code: Some(400),
        trace: None,
        source: None,
        file: "",
        line: 0,
    }
}

// ============================================================================
// 错误消息格式化
// ============================================================================

/// 格式化错误为用户友好的消息
export function format_user_error(error: ErrorStack) -> string {
    let mut result = "Error: " ++ error.message ++ "\n";
    
    match error.error_code {
        Some(code) => {
            result = result ++ "  Code: " ++ (code as string) ++ "\n";
        },
        None => {},
    }
    
    when error.file != "" {
        result = result ++ "  Location: " ++ error.file ++ ":" ++ (error.line as string) ++ "\n";
    }
    
    match error.trace {
        Some(trace) => {
            result = result ++ "\nStack trace:\n";
            for (i, frame) in enumerate(trace.frames) {
                when frame.function_name != "" {
                    result = result ++ "  " ++ (i as string) ++ ". " ++ frame.function_name ++ "\n";
                    when frame.file != "" {
                        result = result ++ "     at " ++ frame.file ++ ":" ++ (frame.line as string) ++ "\n";
                    }
                }
            }
        },
        None => {},
    }
    
    match error.source {
        Some(source) => {
            result = result ++ "\nCaused by:\n" ++ format_user_error(source);
        },
        None => {},
    }
    
    result
}

/// 获取错误的简短描述
export function error_summary(error: ErrorStack) -> string {
    error.message
}
