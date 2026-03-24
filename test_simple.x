// 简单测试 - 不依赖 prelude
println("Hello, X!");

let x = 42;
println("x = 42");

let sum = 1 + 2 + 3;
println("sum = 6");

if sum > 5 {
    println("sum is greater than 5");
} else {
    println("sum is not greater than 5");
}

let i = 0;
while i < 3 {
    println("loop");
    i = i + 1;
}

println("Test completed!");
