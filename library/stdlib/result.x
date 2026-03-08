// X语言标准库 - Result 类型
//
// Result 类型表示操作可能成功或失败
// 用于替代异常，强制处理可能的错误

// ==========================================
// Result 类型定义
// ==========================================

/// Result 类型：Ok(value) 或 Err(error)
type Result<T, E> =
  | Ok  { value: T }
  | Err { error: E }

// ==========================================
// Result 构造函数
// ==========================================

/// 创建一个成功的 Result
function Ok<T, E>(value: T): Result<T, E> {
  Result::Ok { value: value }
}

/// 创建一个失败的 Result
function Err<T, E>(error: E): Result<T, E> {
  Result::Err { error: error }
}

// ==========================================
// Result 检查函数
// ==========================================

/// 检查 Result 是否是成功的
function is_ok<T, E>(result: Result<T, E>): Bool {
  when result is
    Ok { value } -> true
    Err { error } -> false
}

/// 检查 Result 是否是失败的
function is_err<T, E>(result: Result<T, E>): Bool {
  not is_ok(result)
}

// ==========================================
// Result 解包函数
// ==========================================

/// 解包 Result，成功时返回值，失败时 panic
function unwrap<T, E>(result: Result<T, E>, message: String = "unwrap on Err"): T {
  when result is
    Ok { value } -> value
    Err { error } -> panic(message)
}

/// 解包 Result，失败时返回默认值
function unwrap_or<T, E>(result: Result<T, E>, default: T): T {
  when result is
    Ok { value } -> value
    Err { error } -> default
}

/// 解包 Result，失败时调用函数生成默认值
function unwrap_or_else<T, E>(result: Result<T, E>, default_function: (E) -> T): T {
  when result is
    Ok { value } -> value
    Err { error } -> default_function(error)
}

/// 解包 Result，失败时 panic 并显示错误
function expect<T, E>(result: Result<T, E>, message: String): T {
  when result is
    Ok { value } -> value
    Err { error } -> panic(message + ": " + to_string(error))
}

// ==========================================
// Result 变换函数
// ==========================================

/// 对成功的 Result 中的值应用函数
function map<T, U, E>(result: Result<T, E>, f: (T) -> U): Result<U, E> {
  when result is
    Ok { value } -> Ok(f(value))
    Err { error } -> Err(error)
}

/// 对失败的 Result 中的错误应用函数
function map_err<T, E, F>(result: Result<T, E>, f: (E) -> F): Result<T, F> {
  when result is
    Ok { value } -> Ok(value)
    Err { error } -> Err(f(error))
}

/// 对成功的 Result 应用返回 Result 的函数
function and_then<T, U, E>(result: Result<T, E>, f: (T) -> Result<U, E>): Result<U, E> {
  when result is
    Ok { value } -> f(value)
    Err { error } -> Err(error)
}

/// 对失败的 Result 应用返回 Result 的函数
function or_else<T, E, F>(result: Result<T, E>, f: (E) -> Result<T, F>): Result<T, F> {
  when result is
    Ok { value } -> Ok(value)
    Err { error } -> f(error)
}

// ==========================================
// Result 组合函数
// ==========================================

/// 如果第一个 Result 成功则返回它，否则返回第二个 Result
function or<T, E>(result1: Result<T, E>, result2: Result<T, E>): Result<T, E> {
  when result1 is
    Ok { value } -> result1
    Err { error } -> result2
}

/// 如果两个 Result 都成功，返回包含值对的 Result
function and<T, U, E>(result1: Result<T, E>, result2: Result<U, E>): Result<(T, U), E> {
  when (result1, result2) is
    (Ok { value: v1 }, Ok { value: v2 }) -> Ok((v1, v2))
    (Err { error: e }, _) -> Err(e)
    (_, Err { error: e }) -> Err(e)
}

// ==========================================
// Result 转换为 Option
// ==========================================

/// 将 Result 转换为 Option，丢弃错误信息
function ok<T, E>(result: Result<T, E>): Option<T> {
  when result is
    Ok { value } -> Some(value)
    Err { error } -> None()
}

/// 将 Result 转换为 Option，丢弃成功值
function err<T, E>(result: Result<T, E>): Option<E> {
  when result is
    Ok { value } -> None()
    Err { error } -> Some(error)
}

// ==========================================
// 错误传播辅助（? 运算符的函数形式）
// ==========================================

/// 尝试解包 Result，失败时提前返回错误
/// 这是 ? 运算符的函数形式
function try<T, E>(result: Result<T, E>): T {
  // 这个函数由编译器特殊处理
  // 在实际代码中，使用 ? 运算符更简洁
  when result is
    Ok { value } -> value
    Err { error } -> return Err(error)  // 这个返回由编译器处理
}