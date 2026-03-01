-- reverse-complement: DNA reverse complement (Benchmarks Game)
fun revlen(s, i) {
  if i <= 0 {
    return 0
  }
  return 1 + revlen(s, i - 1)
}

fun main() {
  val n = revlen(0, 6)
  print(n)
}
