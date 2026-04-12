module std.types

/// Option 类型 - 表示可能存在或不存在的值
export enum Option<T> {
    None,
    Some(T),
}

/// Result 类型 - 表示成功或失败的结果
export enum Result<T, E> {
    Ok(T),
    Err(E),
}
