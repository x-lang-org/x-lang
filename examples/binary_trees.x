-- binary-trees: allocator-heavy trees (Benchmarks Game)
fun count(depth) {
  if depth <= 0 {
    return 1
  }
  val l = count(depth - 1)
  val r = count(depth - 1)
  return 1 + l + r
}

fun main() {
  print(count(2))
}
