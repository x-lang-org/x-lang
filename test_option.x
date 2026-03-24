// 测试泛型 enum 定义
enum Option<T> {
    Some(T),
    None
}

// 测试变量
let x = 1;
println("x = 1");

// 测试 match 语句（使用字面量）
match x {
    1 => println("one"),
    _ => println("other")
}

println("Test completed!");
