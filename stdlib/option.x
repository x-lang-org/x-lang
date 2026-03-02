// X语言标准库 - Option 类型
//
// Option 类型表示一个可能存在也可能不存在的值
// 用于替代 null，强制处理可能缺失的情况

// ==========================================
// Option 类型定义
// ==========================================

/// Option 类型：Some(value) 或 None
type Option<T> =
  | Some { value: T }
  | None

// ==========================================
// Option 构造函数
// ==========================================

/// 创建一个包含值的 Option
fun Some<T>(value: T): Option<T> {
  Option::Some { value: value }
}

/// 创建一个空的 Option
fun None<T>(): Option<T> {
  Option::None
}

// ==========================================
// Option 检查函数
// ==========================================

/// 检查 Option 是否包含值
fun is_some<T>(opt: Option<T>): Bool {
  when opt is
    Some { value } -> true
    None -> false
}

/// 检查 Option 是否为空
fun is_none<T>(opt: Option<T>): Bool {
  not is_some(opt)
}

// ==========================================
// Option 解包函数
// ==========================================

/// 解包 Option，包含值时返回该值，否则 panic
fun unwrap<T>(opt: Option<T>, message: String = "unwrap on None"): T {
  when opt is
    Some { value } -> value
    None -> panic(message)
}

/// 解包 Option，包含值时返回该值，否则返回默认值
fun unwrap_or<T>(opt: Option<T>, default: T): T {
  when opt is
    Some { value } -> value
    None -> default
}

/// 解包 Option，包含值时返回该值，否则调用函数生成默认值
fun unwrap_or_else<T>(opt: Option<T>, default_func: () -> T): T {
  when opt is
    Some { value } -> value
    None -> default_func()
}

// ==========================================
// Option 变换函数
// ==========================================

/// 对 Option 中的值应用函数
fun map<T, U>(opt: Option<T>, f: (T) -> U): Option<U> {
  when opt is
    Some { value } -> Some(f(value))
    None -> None()
}

/// 对 Option 中的值应用返回 Option 的函数（flat map）
fun and_then<T, U>(opt: Option<T>, f: (T) -> Option<U>): Option<U> {
  when opt is
    Some { value } -> f(value)
    None -> None()
}

/// 如果 Option 包含值且满足谓词，返回该 Option，否则返回 None
fun filter<T>(opt: Option<T>, predicate: (T) -> Bool): Option<T> {
  when opt is
    Some { value } ->
      if predicate(value) { opt }
      else { None() }
    None -> None()
}

// ==========================================
// Option 组合函数
// ==========================================

/// 如果第一个 Option 有值则返回它，否则返回第二个 Option
fun or<T>(opt1: Option<T>, opt2: Option<T>): Option<T> {
  when opt1 is
    Some { value } -> opt1
    None -> opt2
}

/// 如果两个 Option 都有值，返回包含值对的 Option，否则返回 None
fun and<T, U>(opt1: Option<T>, opt2: Option<U>): Option<(T, U)> {
  when (opt1, opt2) is
    (Some { value: v1 }, Some { value: v2 }) -> Some((v1, v2))
    _ -> None()
}

// ==========================================
// Option 转换为 Result
// ==========================================

/// 将 Option 转换为 Result，None 时使用给定的错误
fun ok_or<T, E>(opt: Option<T>, err: E): Result<T, E> {
  when opt is
    Some { value } -> Ok(value)
    None -> Err(err)
}

/// 将 Option 转换为 Result，None 时调用函数生成错误
fun ok_or_else<T, E>(opt: Option<T>, err_func: () -> E): Result<T, E> {
  when opt is
    Some { value } -> Ok(value)
    None -> Err(err_func())
}
