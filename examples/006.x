enum Option<T> {
    /// 值存在
    Some(T),
    /// 值不存在
    None
}

let opt = Option.Some(42)

match opt {
    Option.Some(value) => println(value),
    Option.None => println("No value"),
}
