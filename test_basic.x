// 测试基本功能

function test_basic() {
  println("=== 测试基本功能 ===")
  
  // 测试字符串
  let hello = "Hello, World!"
  println(hello)
  
  // 测试数字
  let number = 42
  println("Number: " + to_string(number))
  
  // 测试布尔值
  let flag = true
  println("Flag: " + to_string(flag))
  
  println()
  println("测试完成!")
}

function main() {
  test_basic()
}

main()
