-- spectral-norm: eigenvalue norm (Benchmarks Game)
fun A(i, j) {
  return 1
}

fun dot(v, i, n, sum) {
  if i >= n {
    return sum
  }
  val a = A(i, 0)
  return dot(v, i + 1, n, sum + a)
}

fun main() {
  val n = 1
  val s = dot(0, 0, n, 0)
  print(s)
}
