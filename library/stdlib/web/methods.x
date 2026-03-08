// X语言Web Framework - HTTP方法模块
//
// 定义HTTP方法常量

/// HTTP方法
type Method = String

/// HTTP方法常量
export let GET: Method = "GET"
export let POST: Method = "POST"
export let PUT: Method = "PUT"
export let DELETE: Method = "DELETE"
export let PATCH: Method = "PATCH"
export let HEAD: Method = "HEAD"
export let OPTIONS: Method = "OPTIONS"
export let TRACE: Method = "TRACE"
export let CONNECT: Method = "CONNECT"

/// 方法集合
let methods = {
  get: GET,
  post: POST,
  put: PUT,
  delete: DELETE,
  patch: PATCH,
  head: HEAD,
  options: OPTIONS,
  trace: TRACE,
  connect: CONNECT,
}
