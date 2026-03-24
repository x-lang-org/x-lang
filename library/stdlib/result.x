// Result 类型 - 泛型版（简化版）
enum Result<T, E> {
    Ok(T),
    Err(E)
}
