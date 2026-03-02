// 数学函数演示
// 展示数学标准库的使用

fun main() {
  println("=== 数学函数演示 ===")
  println()

  // ==========================================
  // 数学常量
  // ==========================================
  println("1. 数学常量")

  println("  π (pi)     = " + to_string(pi))
  println("  e          = " + to_string(e))
  println("  √2         = " + to_string(sqrt2))
  println("  √(1/2)     = " + to_string(sqrt1_2))
  println("  log2(e)    = " + to_string(log2e))
  println("  log10(e)   = " + to_string(log10e))
  println("  ln(2)      = " + to_string(ln2))
  println("  ln(10)     = " + to_string(ln10))
  println()

  // ==========================================
  // 基础函数
  // ==========================================
  println("2. 基础函数")

  println("  abs(-5)          = " + to_string(abs(-5.0)))
  println("  abs_int(-42)     = " + to_string(abs_int(-42)))
  println("  signum(-3.5)     = " + to_string(signum(-3.5)))
  println("  signum(0.0)      = " + to_string(signum(0.0)))
  println("  signum(7.2)      = " + to_string(signum(7.2)))
  println()

  // ==========================================
  // 幂函数和平方根
  // ==========================================
  println("3. 幂函数和平方根")

  println("  sqrt(16.0)       = " + to_string(sqrt(16.0)))
  println("  sqrt(2.0)        = " + to_string(sqrt(2.0)))
  println("  square(5.0)      = " + to_string(square(5.0)))
  println("  square_int(5)    = " + to_string(square_int(5)))
  println("  pow(2.0, 3.0)    = " + to_string(pow(2.0, 3.0)))
  println("  pow(10.0, 3.0)   = " + to_string(pow(10.0, 3.0)))
  println("  pow_int(2, 10)   = " + to_string(pow_int(2, 10)))
  println("  cbrt(8.0)        = " + to_string(cbrt(8.0)))
  println()

  // ==========================================
  // 指数和对数函数
  // ==========================================
  println("4. 指数和对数函数")

  println("  exp(1.0)         = " + to_string(exp(1.0)))
  println("  exp(0.0)         = " + to_string(exp(0.0)))
  println("  exp2(3.0)        = " + to_string(exp2(3.0)))
  println("  ln(e)            = " + to_string(ln(e)))
  println("  log2(8.0)        = " + to_string(log2(8.0)))
  println("  log10(1000.0)    = " + to_string(log10(1000.0)))
  println("  log(2.0, 8.0)    = " + to_string(log(2.0, 8.0)))
  println()

  // ==========================================
  // 三角函数
  // ==========================================
  println("5. 三角函数")

  println("  sin(0)           = " + to_string(sin(0.0)))
  println("  sin(π/2)         = " + to_string(sin(pi / 2.0)))
  println("  cos(0)           = " + to_string(cos(0.0)))
  println("  cos(π)           = " + to_string(cos(pi)))
  println("  tan(π/4)         = " + to_string(tan(pi / 4.0)))
  println("  asin(1.0)        = " + to_string(asin(1.0)))
  println("  acos(0.0)        = " + to_string(acos(0.0)))
  println("  atan(1.0)        = " + to_string(atan(1.0)))
  println("  atan2(1.0, 1.0)  = " + to_string(atan2(1.0, 1.0)))
  println()

  // ==========================================
  // 双曲函数
  // ==========================================
  println("6. 双曲函数")

  println("  sinh(0.0)        = " + to_string(sinh(0.0)))
  println("  sinh(1.0)        = " + to_string(sinh(1.0)))
  println("  cosh(0.0)        = " + to_string(cosh(0.0)))
  println("  cosh(1.0)        = " + to_string(cosh(1.0)))
  println("  tanh(0.0)        = " + to_string(tanh(0.0)))
  println("  tanh(1.0)        = " + to_string(tanh(1.0)))
  println()

  // ==========================================
  // 取整函数
  // ==========================================
  println("7. 取整函数")

  println("  floor(3.7)       = " + to_string(floor(3.7)))
  println("  floor(-3.7)      = " + to_string(floor(-3.7)))
  println("  ceil(3.2)        = " + to_string(ceil(3.2)))
  println("  ceil(-3.2)       = " + to_string(ceil(-3.2)))
  println("  round(3.5)       = " + to_string(round(3.5)))
  println("  round(3.4)       = " + to_string(round(3.4)))
  println("  trunc(3.9)       = " + to_string(trunc(3.9)))
  println("  trunc(-3.9)      = " + to_string(trunc(-3.9)))
  println("  fract(3.7)       = " + to_string(fract(3.7)))
  println()

  // ==========================================
  // 极值函数
  // ==========================================
  println("8. 极值函数")

  println("  min(5.0, 10.0)   = " + to_string(min(5.0, 10.0)))
  println("  min_int(5, 10)   = " + to_string(min_int(5, 10)))
  println("  max(5.0, 10.0)   = " + to_string(max(5.0, 10.0)))
  println("  max_int(5, 10)   = " + to_string(max_int(5, 10)))
  println("  clamp(5, 0, 10)  = " + to_string(clamp(5.0, 0.0, 10.0)))
  println("  clamp(-5, 0, 10) = " + to_string(clamp(-5.0, 0.0, 10.0)))
  println("  clamp(15, 0, 10) = " + to_string(clamp(15.0, 0.0, 10.0)))
  println()

  // ==========================================
  // 角度转换
  // ==========================================
  println("9. 角度转换")

  println("  radians(180)     = " + to_string(radians(180.0)))
  println("  degrees(π)       = " + to_string(degrees(pi)))
  println("  degrees(π/2)     = " + to_string(degrees(pi / 2.0)))
  println()

  // ==========================================
  // 插值和距离
  // ==========================================
  println("10. 插值和距离")

  println("  lerp(0.0, 10.0, 0.5)  = " + to_string(lerp(0.0, 10.0, 0.5)))
  println("  lerp(10.0, 20.0, 0.3) = " + to_string(lerp(10.0, 20.0, 0.3)))
  println("  distance(0, 0, 3, 4)  = " + to_string(distance(0.0, 0.0, 3.0, 4.0)))
  println("  manhattan_distance(0, 0, 3, 4) = " + to_string(manhattan_distance(0.0, 0.0, 3.0, 4.0)))
  println()

  // ==========================================
  // 最大公约数和最小公倍数
  // ==========================================
  println("11. 最大公约数和最小公倍数")

  println("  gcd(48, 18)      = " + to_string(gcd(48, 18)))
  println("  gcd(100, 75)     = " + to_string(gcd(100, 75)))
  println("  lcm(4, 6)        = " + to_string(lcm(4, 6)))
  println("  lcm(12, 18)      = " + to_string(lcm(12, 18)))
  println()

  // ==========================================
  // 因数和质数检查
  // ==========================================
  println("12. 因数和质数检查")

  println("  is_even(42)      = " + to_string(is_even(42)))
  println("  is_even(13)      = " + to_string(is_even(13)))
  println("  is_odd(42)       = " + to_string(is_odd(42)))
  println("  is_odd(13)       = " + to_string(is_odd(13)))
  println("  is_prime(2)      = " + to_string(is_prime(2)))
  println("  is_prime(17)     = " + to_string(is_prime(17)))
  println("  is_prime(100)    = " + to_string(is_prime(100)))
  println()

  // ==========================================
  // 阶乘和斐波那契
  // ==========================================
  println("13. 阶乘和斐波那契")

  println("  factorial(0)     = " + to_string(factorial(0)))
  println("  factorial(5)     = " + to_string(factorial(5)))
  println("  factorial(10)    = " + to_string(factorial(10)))
  println("  fibonacci(0)     = " + to_string(fibonacci(0)))
  println("  fibonacci(1)     = " + to_string(fibonacci(1)))
  println("  fibonacci(10)    = " + to_string(fibonacci(10)))
  println()

  // ==========================================
  // 随机数
  // ==========================================
  println("14. 随机数")

  srand(timestamp())  // 设置随机种子
  println("  rand()           = " + to_string(rand()))
  println("  rand()           = " + to_string(rand()))
  println("  rand_int(0, 10)  = " + to_string(rand_int(0, 10)))
  println("  rand_int(0, 10)  = " + to_string(rand_int(0, 10)))
  println()

  println("=== 数学函数演示完成 ===")
}
