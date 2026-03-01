-- fasta: repeated FASTA (Benchmarks Game)
fun repeat(n, s, acc) {
  if n <= 0 {
    return acc
  }
  return repeat(n - 1, s, acc + 1)
}

fun main() {
  val n = 3
  print(repeat(n, 0, 0))
  print(1)
  print(2)
}
