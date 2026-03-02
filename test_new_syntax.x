// 测试X语言新语法
// 这个文件展示了新的 let/let mut 语法和新的注释语法

/**
 * 这是一个多行注释
 * 可以跨行写
 */

// 简单的函数示例
fun add(a, b) {
  return a + b
}

// 递归函数示例（来自 binary_trees）
fun count(depth) {
  if depth <= 0 {
    return 1
  }
  let l = count(depth - 1)
  let r = count(depth - 1)
  return 1 + l + r
}

// 使用可变变量的示例
fun counter() {
  let mut count = 0
  count = count + 1
  count = count + 1
  return count
}

// 主函数
fun main() {
  // 不可变绑定
  let x = 10
  let y = 20
  let sum = add(x, y)
  print(sum)

  // 递归调用
  let tree_count = count(2)
  print(tree_count)

  // 可变变量
  let c = counter()
  print(c)
}
