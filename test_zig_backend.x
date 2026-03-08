// 测试Zig后端
// 这个文件用于测试Zig后端的编译和运行功能

function main() {
  println("Hello from Zig backend!");
  
  // 测试基本类型和表达式
  let a: Int = 10;
  let b: Float = 3.14;
  let c: Bool = true;
  
  println("a = " + to_string(a));
  println("b = " + to_string(b));
  println("c = " + to_string(c));
  
  // 测试函数调用
  let result = add(a, 5);
  println("add(" + to_string(a) + ", 5) = " + to_string(result));
  
  // 测试条件语句
  if a > 5 {
    println("a is greater than 5");
  } else {
    println("a is not greater than 5");
  }
  
  // 测试循环
  for i in 0..5 {
    println("i = " + to_string(i));
  }
}

function add(x: Int, y: Int) -> Int {
  return x + y;
}
