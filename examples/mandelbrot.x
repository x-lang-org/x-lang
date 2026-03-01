-- mandelbrot: Mandelbrot set (Benchmarks Game)
fun mandel(cr, ci, zr, zi, n) {
  if n <= 0 {
    return 0
  }
  val zr2 = zr * zr
  val zi2 = zi * zi
  if zr2 + zi2 > 4 {
    return n
  }
  val nzr = zr2 - zi2 + cr
  val nzi = 2 * zr * zi + ci
  return mandel(cr, ci, nzr, nzi, n - 1)
}

fun main() {
  val k = mandel(0, 0, 0, 0, 20)
  print(k)
}
