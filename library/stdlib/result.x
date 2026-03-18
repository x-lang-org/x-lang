// Result 类型
// 表示可能成功或失败的操作结果

/// Result<T, E> 表示一个可能成功或失败的结果
enum Result<T, E> {
    /// 操作成功，包含值
    Ok(T),
    /// 操作失败，包含错误
    Err(E)
}

/// 检查 Result 是否为 Ok
function is_ok<T, E>(result: Result<T, E>): boolean {
    given result {
        is Ok(_) => true
        is Err(_) => false
    }
}

/// 检查 Result 是否为 Err
function is_err<T, E>(result: Result<T, E>): boolean {
    given result {
        is Ok(_) => false
        is Err(_) => true
    }
}

/// 从 Ok 中提取值，如果是 Err 则 panic
function unwrap<T, E>(result: Result<T, E>): T {
    given result {
        is Ok(value) => value
        is Err(e) => panic("unwrap called on Err: " + to_string(e))
    }
}

/// 从 Ok 中提取值，如果是 Err 则返回默认值
function unwrap_or<T, E>(result: Result<T, E>, default: T): T {
    given result {
        is Ok(value) => value
        is Err(_) => default
    }
}

/// 从 Err 中提取错误，如果是 Ok 则 panic
function unwrap_err<T, E>(result: Result<T, E>): E {
    given result {
        is Ok(_) => panic("unwrap_err called on Ok")
        is Err(e) => e
    }
}

/// 如果是 Ok，对其值应用函数
function map<T, U, E>(result: Result<T, E>, f: function(T): U): Result<U, E> {
    given result {
        is Ok(value) => Ok(f(value))
        is Err(e) => Err(e)
    }
}

/// 如果是 Err，对其错误应用函数
function map_err<T, E, F>(result: Result<T, E>, f: function(E): F): Result<T, F> {
    given result {
        is Ok(value) => Ok(value)
        is Err(e) => Err(f(e))
    }
}

/// 如果是 Ok，对其值应用返回 Result 的函数
function and_then<T, U, E>(result: Result<T, E>, f: function(T): Result<U, E>): Result<U, E> {
    given result {
        is Ok(value) => f(value)
        is Err(e) => Err(e)
    }
}

/// 如果是 Err，对其错误应用返回 Result 的函数
function or_else<T, E, F>(result: Result<T, E>, f: function(E): Result<T, F>): Result<T, F> {
    given result {
        is Ok(value) => Ok(value)
        is Err(e) => f(e)
    }
}

/// 将 Result<T, E> 转换为 Option<T>
function ok<T, E>(result: Result<T, E>): Option<T> {
    given result {
        is Ok(value) => Some(value)
        is Err(_) => None
    }
}

/// 将 Result<T, E> 转换为 Option<E>
function err<T, E>(result: Result<T, E>): Option<E> {
    given result {
        is Ok(_) => None
        is Err(e) => Some(e)
    }
}

/// 将 Result<Result<T, E>, E> 展平为 Result<T, E>
function flatten<T, E>(result: Result<Result<T, E>, E>): Result<T, E> {
    given result {
        is Ok(inner) => inner
        is Err(e) => Err(e)
    }
}

/// 将 Result<T, E> 转换为 Option<T>，如果是 Err 则执行错误处理函数
function ok_or_else<T, E>(result: Result<T, E>, f: function(E)): Option<T> {
    given result {
        is Ok(value) => Some(value)
        is Err(e) => {
            f(e)
            None
        }
    }
}
