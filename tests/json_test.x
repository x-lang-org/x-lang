// JSON 功能测试

// 测试基本类型序列化
print("测试基本类型序列化:")
print("数字: " + x_to_json(42))
print("浮点数: " + x_to_json(3.14))
print("布尔值: " + x_to_json(true))
print("字符串: " + x_to_json("hello"))
print("空值: " + x_to_json(null))

// 测试解析基本类型
print("\n测试解析基本类型:")
let num = x_json_parse("42")
print("解析数字: " + to_string(num))
let str = x_json_parse("\"hello\"")
print("解析字符串: " + str)
let bool = x_json_parse("true")
print("解析布尔值: " + to_string(bool))
let null_val = x_json_parse("null")
print("解析null: " + to_string(null_val))

print("\n所有测试完成！")
