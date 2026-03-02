// Hello World 示例
// 演示基本的标准库使用

fun main() {
  // 打印问候
  println("Hello, World!")
  println("欢迎使用 X 语言标准库！")
  println()

  // 打印标准库版本
  println("标准库版本: " + stdlib_version())
  println()

  // 使用 Option
  let optional_value = Some(42)
  println("Option 示例:")
  println("  Some(42) 是否有值: " + to_string(is_some(optional_value)))
  println("  解包的值: " + to_string(unwrap(optional_value)))
  println()

  // 使用 Result
  let result_value = Ok("成功!")
  println("Result 示例:")
  println("  Ok(\"成功!\") 是否成功: " + to_string(is_ok(result_value)))
  println("  解包的值: " + unwrap(result_value))
  println()

  // 数学计算
  println("数学示例:")
  println("  π = " + to_string(pi))
  println("  e = " + to_string(e))
  println("  sin(π/2) = " + to_string(sin(pi / 2)))
  println("  sqrt(16) = " + to_string(sqrt(16)))
  println()

  // 字符串操作
  let s = "Hello, X!"
  println("字符串示例:")
  println("  字符串: " + s)
  println("  长度: " + to_string(str_len(s)))
  println("  大写: " + str_to_uppercase(s))
  println()

  // 列表操作
  let numbers = list_range(1, 6)
  println("列表示例:")
  println("  列表: " + to_string(numbers))
  println("  长度: " + to_string(list_len(numbers)))
  println("  和: " + to_string(list_sum(numbers)))
  println()

  println("示例运行完成！")
}
