-- fannkuch-redux: index/permutation (Benchmarks Game)
fun flips(n, a, b) {
  if n <= 1 {
    return 0
  }
  if a <= 0 {
    return b
  }
  return flips(n - 1, a - 1, b + 1)
}

fun fannkuch(n) {
  if n <= 0 {
    return 0
  }
  return flips(n, n, 0)
}

fun main() {
  print(fannkuch(5))
}
