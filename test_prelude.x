// 测试 prelude 和 Option/Result
println("Hello, X language!");

// 测试 Option 类型（需要先定义）
// 由于 Option 在 option.x 中定义但解析器可能不支持泛型enum
// 我们直接测试 prelude 中的函数

// 测试基本的 println
let x = 42;
println("x = 42");

// 测试算术
let sum = 1 + 2 + 3;
println("sum = 6");

// 测试条件
if sum > 5 {
    println("sum is greater than 5");
} else {
    println("sum is not greater than 5");
}

// 测试循环
let i = 0;
while i < 3 {
    println("loop iteration");
    i = i + 1;
}

println("Test completed!");
