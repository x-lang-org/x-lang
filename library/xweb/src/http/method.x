// HTTP Method - 映射到 Zig std.http.Method

/// HTTP Method
enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
    Connect
}

/// 从 Zig Method 转换
function from_zig_method(method: zig::std.http.Method) -> Method {
    match method {
        is .GET => Method.Get,
        is .POST => Method.Post,
        is .PUT => Method.Put,
        is .DELETE => Method.Delete,
        is .PATCH => Method.Patch,
        is .HEAD => Method.Head,
        is .OPTIONS => Method.Options,
        is .TRACE => Method.Trace,
        is .CONNECT => Method.Connect,
        is _ => Method.Get
    }
}

/// 转换为 Zig Method
function to_zig_method(method: Method) -> zig::std.http.Method {
    match method {
        is Method.Get => .GET,
        is Method.Post => .POST,
        is Method.Put => .PUT,
        is Method.Delete => .DELETE,
        is Method.Patch => .PATCH,
        is Method.Head => .HEAD,
        is Method.Options => .OPTIONS,
        is Method.Trace => .TRACE,
        is Method.Connect => .CONNECT
    }
}

/// 从字符串解析
function parse_method(s: string) -> Option<Method> {
    match s {
        is "GET" => Some(Method.Get),
        is "POST" => Some(Method.Post),
        is "PUT" => Some(Method.Put),
        is "DELETE" => Some(Method.Delete),
        is "PATCH" => Some(Method.Patch),
        is "HEAD" => Some(Method.Head),
        is "OPTIONS" => Some(Method.Options),
        is "TRACE" => Some(Method.Trace),
        is "CONNECT" => Some(Method.Connect),
        is _ => None
    }
}

/// 转换为字符串
function method_to_string(method: Method) -> string {
    match method {
        is Method.Get => "GET",
        is Method.Post => "POST",
        is Method.Put => "PUT",
        is Method.Delete => "DELETE",
        is Method.Patch => "PATCH",
        is Method.Head => "HEAD",
        is Method.Options => "OPTIONS",
        is Method.Trace => "TRACE",
        is Method.Connect => "CONNECT"
    }
}
