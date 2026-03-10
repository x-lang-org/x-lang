// X语言Web Framework - HTTP状态码模块
//
// 定义HTTP状态码常量

/// HTTP状态码
///
/// # 1xx 信息性状态码
/// - `CONTINUE`: 100 - 继续
/// - `SWITCHING_PROTOCOLS`: 101 - 切换协议
///
/// # 2xx 成功状态码
/// - `OK`: 200 - 请求成功
/// - `CREATED`: 201 - 资源创建成功
/// - `ACCEPTED`: 202 - 请求已接受
/// - `NO_CONTENT`: 204 - 无内容
///
/// # 3xx 重定向状态码
/// - `MOVED_PERMANENTLY`: 301 - 永久重定向
/// - `FOUND`: 302 - 临时重定向
/// - `SEE_OTHER`: 303 - 查看其他位置
/// - `NOT_MODIFIED`: 304 - 未修改
/// - `TEMPORARY_REDIRECT`: 307 - 临时重定向
/// - `PERMANENT_REDIRECT`: 308 - 永久重定向
///
/// # 4xx 客户端错误状态码
/// - `BAD_REQUEST`: 400 - 错误请求
/// - `UNAUTHORIZED`: 401 - 未授权
/// - `FORBIDDEN`: 403 - 禁止访问
/// - `NOT_FOUND`: 404 - 资源不存在
/// - `METHOD_NOT_ALLOWED`: 405 - 方法不允许
/// - `NOT_ACCEPTABLE`: 406 - 不可接受
/// - `REQUEST_TIMEOUT`: 408 - 请求超时
/// - `CONFLICT`: 409 - 冲突
/// - `GONE`: 410 - 资源已删除
/// - `LENGTH_REQUIRED`: 411 - 需要内容长度
/// - `PAYLOAD_TOO_LARGE`: 413 - 负载过大
/// - `URI_TOO_LONG`: 414 - URI过长
/// - `UNSUPPORTED_MEDIA_TYPE`: 415 - 不支持的媒体类型
/// - `TOO_MANY_REQUESTS`: 429 - 请求过多
///
/// # 5xx 服务器错误状态码
/// - `INTERNAL_SERVER_ERROR`: 500 - 服务器内部错误
/// - `NOT_IMPLEMENTED`: 501 - 未实现
/// - `BAD_GATEWAY`: 502 - 错误的网关
/// - `SERVICE_UNAVAILABLE`: 503 - 服务不可用
/// - `GATEWAY_TIMEOUT`: 504 - 网关超时
type StatusCode = Int

/// 1xx 信息性状态码
export let CONTINUE: StatusCode = 100
export let SWITCHING_PROTOCOLS: StatusCode = 101

/// 2xx 成功状态码
export let OK: StatusCode = 200
export let CREATED: StatusCode = 201
export let ACCEPTED: StatusCode = 202
export let NO_CONTENT: StatusCode = 204

/// 3xx 重定向状态码
export let MOVED_PERMANENTLY: StatusCode = 301
export let FOUND: StatusCode = 302
export let SEE_OTHER: StatusCode = 303
export let NOT_MODIFIED: StatusCode = 304
export let TEMPORARY_REDIRECT: StatusCode = 307
export let PERMANENT_REDIRECT: StatusCode = 308

/// 4xx 客户端错误状态码
export let BAD_REQUEST: StatusCode = 400
export let UNAUTHORIZED: StatusCode = 401
export let FORBIDDEN: StatusCode = 403
export let NOT_FOUND: StatusCode = 404
export let METHOD_NOT_ALLOWED: StatusCode = 405
export let NOT_ACCEPTABLE: StatusCode = 406
export let REQUEST_TIMEOUT: StatusCode = 408
export let CONFLICT: StatusCode = 409
export let GONE: StatusCode = 410
export let LENGTH_REQUIRED: StatusCode = 411
export let PAYLOAD_TOO_LARGE: StatusCode = 413
export let URI_TOO_LONG: StatusCode = 414
export let UNSUPPORTED_MEDIA_TYPE: StatusCode = 415
export let TOO_MANY_REQUESTS: StatusCode = 429

/// 5xx 服务器错误状态码
export let INTERNAL_SERVER_ERROR: StatusCode = 500
export let NOT_IMPLEMENTED: StatusCode = 501
export let BAD_GATEWAY: StatusCode = 502
export let SERVICE_UNAVAILABLE: StatusCode = 503
export let GATEWAY_TIMEOUT: StatusCode = 504

/// 状态码集合
let status_codes = {
  continue: CONTINUE,
  switching_protocols: SWITCHING_PROTOCOLS,
  ok: OK,
  created: CREATED,
  accepted: ACCEPTED,
  no_content: NO_CONTENT,
  moved_permanently: MOVED_PERMANENTLY,
  found: FOUND,
  see_other: SEE_OTHER,
  not_modified: NOT_MODIFIED,
  temporary_redirect: TEMPORARY_REDIRECT,
  permanent_redirect: PERMANENT_REDIRECT,
  bad_request: BAD_REQUEST,
  unauthorized: UNAUTHORIZED,
  forbidden: FORBIDDEN,
  not_found: NOT_FOUND,
  method_not_allowed: METHOD_NOT_ALLOWED,
  not_acceptable: NOT_ACCEPTABLE,
  request_timeout: REQUEST_TIMEOUT,
  conflict: CONFLICT,
  gone: GONE,
  length_required: LENGTH_REQUIRED,
  payload_too_large: PAYLOAD_TOO_LARGE,
  uri_too_long: URI_TOO_LONG,
  unsupported_media_type: UNSUPPORTED_MEDIA_TYPE,
  too_many_requests: TOO_MANY_REQUESTS,
  internal_server_error: INTERNAL_SERVER_ERROR,
  not_implemented: NOT_IMPLEMENTED,
  bad_gateway: BAD_GATEWAY,
  service_unavailable: SERVICE_UNAVAILABLE,
  gateway_timeout: GATEWAY_TIMEOUT,
}
