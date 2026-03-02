// Option 类型演示
// 展示如何使用 Option 替代 null

fun main() {
  println("=== Option 类型演示 ===")
  println()

  // ==========================================
  // 创建 Option
  // ==========================================
  println("1. 创建 Option")

  let some_int = Some(42)
  let some_str = Some("Hello")
  let none_value = None()

  println("  Some(42): " + to_string(some_int))
  println("  Some(\"Hello\"): " + to_string(some_str))
  println("  None: " + to_string(none_value))
  println()

  // ==========================================
  // 检查 Option 状态
  // ==========================================
  println("2. 检查 Option 状态")

  println("  is_some(Some(42)): " + to_string(is_some(some_int)))
  println("  is_none(Some(42)): " + to_string(is_none(some_int)))
  println("  is_some(None): " + to_string(is_some(none_value)))
  println("  is_none(None): " + to_string(is_none(none_value)))
  println()

  // ==========================================
  // 解包 Option
  // ==========================================
  println("3. 解包 Option")

  // unwrap - 有值时返回，无值时 panic
  println("  unwrap(Some(42)): " + to_string(unwrap(some_int)))

  // unwrap_or - 有值时返回，无值时返回默认值
  println("  unwrap_or(Some(42), 0): " + to_string(unwrap_or(some_int, 0)))
  println("  unwrap_or(None, 0): " + to_string(unwrap_or(none_value, 0)))

  // unwrap_or_else - 有值时返回，无值时调用函数
  let default_val = unwrap_or_else(none_value, () -> {
    println("    计算默认值...")
    100
  })
  println("  unwrap_or_else(None, ...): " + to_string(default_val))
  println()

  // ==========================================
  // 变换 Option
  // ==========================================
  println("4. 变换 Option")

  // map - 对值应用函数
  let doubled = map(some_int, (x) -> x * 2)
  println("  map(Some(42), (x) -> x * 2): " + to_string(doubled))
  println("  map(None, (x) -> x * 2): " + to_string(map(none_value, (x) -> x * 2)))

  // and_then - 对值应用返回 Option 的函数
  let positive = (x: Int) -> if x > 0 { Some(x) } else { None() }
  let result1 = and_then(some_int, positive)
  let result2 = and_then(Some(-10), positive)
  println("  and_then(Some(42), positive): " + to_string(result1))
  println("  and_then(Some(-10), positive): " + to_string(result2))

  // filter - 保留满足谓词的值
  let filtered = filter(some_int, (x) -> x > 10)
  println("  filter(Some(42), (x) -> x > 10): " + to_string(filtered))
  println("  filter(Some(42), (x) -> x > 50): " + to_string(filter(some_int, (x) -> x > 50)))
  println()

  // ==========================================
  // 组合 Option
  // ==========================================
  println("5. 组合 Option")

  let some1 = Some(10)
  let some2 = Some(20)
  let none1 = None()

  // or - 如果第一个有值则返回第一个，否则返回第二个
  println("  or(Some(10), Some(20)): " + to_string(or(some1, some2)))
  println("  or(None, Some(20)): " + to_string(or(none1, some2)))
  println("  or(None, None): " + to_string(or(none1, none1)))

  // and - 如果两个都有值则返回值对，否则返回 None
  println("  and(Some(10), Some(20)): " + to_string(and(some1, some2)))
  println("  and(None, Some(20)): " + to_string(and(none1, some2)))
  println()

  // ==========================================
  // 转换为 Result
  // ==========================================
  println("6. 转换为 Result")

  // ok_or - None 时使用给定的错误
  println("  ok_or(Some(42), \"error\"): " + to_string(ok_or(some_int, "error")))
  println("  ok_or(None, \"error\"): " + to_string(ok_or(none_value, "error")))

  // ok_or_else - None 时调用函数生成错误
  let err_result = ok_or_else(none_value, () -> "dynamic error")
  println("  ok_or_else(None, ...): " + to_string(err_result))
  println()

  // ==========================================
  // 实际示例：查找用户
  // ==========================================
  println("7. 实际示例：查找用户")

  // 模拟用户数据库
  let users = {
    "1": "Alice",
    "2": "Bob",
    "3": "Charlie"
  }

  fun find_user(id: String): Option<String> {
    map_get(users, id)
  }

  // 查找存在的用户
  let user1 = find_user("2")
  println("  find_user(\"2\"): " + to_string(user1))
  println("  用户名: " + unwrap_or(user1, "未知"))

  // 查找不存在的用户
  let user2 = find_user("99")
  println("  find_user(\"99\"): " + to_string(user2))
  println("  用户名: " + unwrap_or(user2, "未知"))
  println()

  println("=== Option 演示完成 ===")
}
