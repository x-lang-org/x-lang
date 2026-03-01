-- n-body: N-body simulation (Benchmarks Game)
-- Minimal: 2 bodies, 5 steps via recursion
fun step(n, x1, v1, x2, v2) {
  if n <= 0 {
    return x1
  }
  val dx = x2 - x1
  val a = dx
  val nv1 = v1 + a
  val nv2 = v2 - a
  val nx1 = x1 + nv1
  val nx2 = x2 + nv2
  return step(n - 1, nx1, nv1, nx2, nv2)
}

fun main() {
  val x = step(5, 0, 0, 100, 0)
  print(x)
}
