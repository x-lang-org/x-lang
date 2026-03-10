// X语言标准库 - Prelude（自动导入）
//
// 此模块包含最常用的函数和类型，会自动导入到所有X程序中

// ==========================================
// 断言函数
// ==========================================

/// 断言条件为真，否则 panic
function assert(condition: Bool, message: String = "断言失败") {
  if not condition {
    panic(message)
  }
}

/// 断言两个值相等
function assert_eq(a, b, message: String = "值不相等") {
  if a != b {
    let msg = message + ": " + to_string(a) + " != " + to_string(b)
    panic(msg)
  }
}

/// 断言两个值不相等
function assert_neq(a, b, message: String = "值相等") {
  if a == b {
    let msg = message + ": " + to_string(a) + " == " + to_string(b)
    panic(msg)
  }
}

// ==========================================
// 转换函数
// ==========================================

/// 将值转换为字符串表示
function to_string(value): String {
  // 这个函数是内置的，由解释器提供特殊处理
  // 这里只是声明签名
  "__builtin_to_string"
}

/// 获取值的类型名称
function type_of(value): String {
  // 内置函数
  "__builtin_type_of"
}

// ==========================================
// 调试辅助
// ==========================================

/// 打印调试信息（带前缀）
function dbg(value) {
  print("[DEBUG] " + to_string(value))
  value
}

/// 带标签的调试打印
function dbg_label(label: String, value) {
  print("[DEBUG " + label + "] " + to_string(value))
  value
}