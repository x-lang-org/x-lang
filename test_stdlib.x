// 测试标准库功能
// 这个文件用于测试X语言标准库的功能，特别是通过Zig标准库实现的部分

function main() {
  // 测试print函数
  println("测试标准库功能");
  
  // 测试数学函数
  println("数学函数测试:");
  println("PI = " + to_string(PI));
  println("E = " + to_string(E));
  println("abs(-10) = " + to_string(abs(-10)));
  println("max(5, 10) = " + to_string(max(5, 10)));
  println("min(5, 10) = " + to_string(min(5, 10)));
  println("sqrt(16) = " + to_string(sqrt(16)));
  
  // 测试字符串函数
  println("\n字符串函数测试:");
  let s = "Hello, World!";
  println("字符串: " + s);
  println("长度: " + to_string(len(s)));
  println("首字符: " + char_at(s, 0));
  println("子字符串: " + substring(s, 0, 5));
  println("是否包含 'World': " + to_string(contains(s, "World")));
  println("转大写: " + to_upper(s));
  println("转小写: " + to_lower(s));
  
  // 测试类型转换
  println("\n类型转换测试:");
  let num = 42;
  let num_str = to_string(num);
  println("数字转字符串: " + num_str);
  let parsed_num = parse_int(num_str);
  println("字符串转数字: " + to_string(parsed_num));
  
  // 测试Option类型
  println("\nOption类型测试:");
  let some_value = Some(42);
  let none_value = None;
  println("Some值: " + to_string(unwrap(some_value)));
  println("None值是否为None: " + to_string(is_none(none_value)));
  
  // 测试panic函数
  println("\n测试完成！");
}
