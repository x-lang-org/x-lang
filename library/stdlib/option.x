// Option 类型
// 表示可能存在或不存在的值，替代 null

/// Option<T> 表示一个可能存在或不存在的值
enum Option<T> {
    /// 值存在
    Some(T),
    /// 值不存在
    None
}

/// 检查 Option 是否为 Some
function is_some<T>(opt: Option<T>): boolean {
    given opt {
        is Some(_) => true
        is None => false
    }
}

/// 检查 Option 是否为 None
function is_none<T>(opt: Option<T>): boolean {
    given opt {
        is Some(_) => false
        is None => true
    }
}

/// 从 Some 中提取值，如果是 None 则 panic
function unwrap<T>(opt: Option<T>): T {
    given opt {
        is Some(value) => value
        is None => panic("unwrap called on None")
    }
}

/// 从 Some 中提取值，如果是 None 则返回默认值
function unwrap_or<T>(opt: Option<T>, default: T): T {
    given opt {
        is Some(value) => value
        is None => default
    }
}

/// 从 Some 中提取值，如果是 None 则调用函数生成默认值
function unwrap_or_else<T>(opt: Option<T>, f: function(): T): T {
    given opt {
        is Some(value) => value
        is None => f()
    }
}

/// 如果是 Some，对其值应用函数
function map<T, U>(opt: Option<T>, f: function(T): U): Option<U> {
    given opt {
        is Some(value) => Some(f(value))
        is None => None
    }
}

/// 如果是 Some，对其值应用返回 Option 的函数
function and_then<T, U>(opt: Option<T>, f: function(T): Option<U>): Option<U> {
    given opt {
        is Some(value) => f(value)
        is None => None
    }
}

/// 如果是 None，返回另一个 Option；否则返回自身
function or<T>(opt: Option<T>, other: Option<T>): Option<T> {
    given opt {
        is Some(_) => opt
        is None => other
    }
}

/// 如果是 None，调用函数生成 Option；否则返回自身
function or_else<T>(opt: Option<T>, f: function(): Option<T>): Option<T> {
    given opt {
        is Some(_) => opt
        is None => f()
    }
}

/// 将两个 Option 组合成一个元组 Option
function zip<T, U>(opt1: Option<T>, opt2: Option<U>): Option<(T, U)> {
    given opt1 {
        is Some(v1) => given opt2 {
            is Some(v2) => Some((v1, v2))
            is None => None
        }
        is None => None
    }
}

/// 过滤 Option：如果是 Some 且满足谓词，返回自身；否则返回 None
function filter<T>(opt: Option<T>, predicate: function(T): boolean): Option<T> {
    given opt {
        is Some(value) => if predicate(value) { opt } else { None }
        is None => None
    }
}

/// 如果是 Some 且值等于目标，返回 Some(())；否则返回 None
function contains<T>(opt: Option<T>, target: T): Option<()> where T: Eq {
    given opt {
        is Some(value) => if value == target { Some(()) } else { None }
        is None => None
    }
}
