// HTTP Status - 映射到 Zig std.http.Status

/// HTTP Status
enum Status {
    // 2xx Success
    Ok,
    Created,
    Accepted,
    NoContent,
    ResetContent,
    PartialContent,

    // 3xx Redirection
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    TemporaryRedirect,
    PermanentRedirect,

    // 4xx Client Error
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    RequestTimeout,
    Conflict,
    Gone,
    PayloadTooLarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    ImATeapot,
    UnprocessableEntity,
    TooManyRequests,

    // 5xx Server Error
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported
}

/// 转换为 Zig Status
function to_zig_status(status: Status) -> zig::std.http.Status {
    match status {
        is Status.Ok => .ok,
        is Status.Created => .created,
        is Status.Accepted => .accepted,
        is Status.NoContent => .no_content,
        is Status.ResetContent => .reset_content,
        is Status.PartialContent => .partial_content,
        is Status.MovedPermanently => .moved_permanently,
        is Status.Found => .found,
        is Status.SeeOther => .see_other,
        is Status.NotModified => .not_modified,
        is Status.TemporaryRedirect => .temporary_redirect,
        is Status.PermanentRedirect => .permanent_redirect,
        is Status.BadRequest => .bad_request,
        is Status.Unauthorized => .unauthorized,
        is Status.Forbidden => .forbidden,
        is Status.NotFound => .not_found,
        is Status.MethodNotAllowed => .method_not_allowed,
        is Status.NotAcceptable => .not_acceptable,
        is Status.RequestTimeout => .request_timeout,
        is Status.Conflict => .conflict,
        is Status.Gone => .gone,
        is Status.PayloadTooLarge => .payload_too_large,
        is Status.URITooLong => .uri_too_long,
        is Status.UnsupportedMediaType => .unsupported_media_type,
        is Status.RangeNotSatisfiable => .range_not_satisfiable,
        is Status.ExpectationFailed => .expectation_failed,
        is Status.ImATeapot => .teapot,
        is Status.UnprocessableEntity => .unprocessable_entity,
        is Status.TooManyRequests => .too_many_requests,
        is Status.InternalServerError => .internal_server_error,
        is Status.NotImplemented => .not_implemented,
        is Status.BadGateway => .bad_gateway,
        is Status.ServiceUnavailable => .service_unavailable,
        is Status.GatewayTimeout => .gateway_timeout,
        is Status.HTTPVersionNotSupported => .http_version_not_supported
    }
}

/// 获取状态码数字
function status_code(status: Status) -> integer {
    match status {
        is Status.Ok => 200,
        is Status.Created => 201,
        is Status.Accepted => 202,
        is Status.NoContent => 204,
        is Status.ResetContent => 205,
        is Status.PartialContent => 206,
        is Status.MovedPermanently => 301,
        is Status.Found => 302,
        is Status.SeeOther => 303,
        is Status.NotModified => 304,
        is Status.TemporaryRedirect => 307,
        is Status.PermanentRedirect => 308,
        is Status.BadRequest => 400,
        is Status.Unauthorized => 401,
        is Status.Forbidden => 403,
        is Status.NotFound => 404,
        is Status.MethodNotAllowed => 405,
        is Status.NotAcceptable => 406,
        is Status.RequestTimeout => 408,
        is Status.Conflict => 409,
        is Status.Gone => 410,
        is Status.PayloadTooLarge => 413,
        is Status.URITooLong => 414,
        is Status.UnsupportedMediaType => 415,
        is Status.RangeNotSatisfiable => 416,
        is Status.ExpectationFailed => 417,
        is Status.ImATeapot => 418,
        is Status.UnprocessableEntity => 422,
        is Status.TooManyRequests => 429,
        is Status.InternalServerError => 500,
        is Status.NotImplemented => 501,
        is Status.BadGateway => 502,
        is Status.ServiceUnavailable => 503,
        is Status.GatewayTimeout => 504,
        is Status.HTTPVersionNotSupported => 505
    }
}
