// 测试Zig标准库导入和函数调用
import zig::std;

function main() {
    // 测试数学函数
    let x: Int = 42;
    let sqrt_x = std.math.sqrt(f64(x));
    println("Square root of " + to_string(x) + " is " + to_string(sqrt_x));
    
    // 测试字符串函数
    let s = "Hello, Zig!";
    let len = s.len;
    println("String length: " + to_string(len));
    
    // 测试内存分配
    let allocator = std.heap.page_allocator;
    let buffer = allocator.alloc(u8, 1024);
    println("Allocated buffer of size: " + to_string(buffer.len));
    allocator.free(buffer);
    
    println("Zig standard library test completed!");
}