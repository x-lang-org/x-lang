// X语言标准库 - 数学函数
//
// 常用的数学常量和函数

// ==========================================
// 数学常量
// ==========================================

/// 圆周率 π ≈ 3.141592653589793
let pi: Float = 3.141592653589793

/// 自然对数的底 e ≈ 2.718281828459045
let e: Float = 2.718281828459045

/// 2 的平方根 ≈ 1.4142135623730951
let sqrt2: Float = 1.4142135623730951

/// 1/2 的平方根 ≈ 0.7071067811865476
let sqrt1_2: Float = 0.7071067811865476

/// 以 2 为底 e 的对数 ≈ 1.4426950408889634
let log2e: Float = 1.4426950408889634

/// 以 10 为底 e 的对数 ≈ 0.4342944819032518
let log10e: Float = 0.4342944819032518

/// 以 e 为底 2 的对数 ≈ 0.6931471805599453
let ln2: Float = 0.6931471805599453

/// 以 e 为底 10 的对数 ≈ 2.302585092994046
let ln10: Float = 2.302585092994046

/// 正无穷大
let infinity: Float = 1.0 / 0.0

/// 负无穷大
let neg_infinity: Float = -1.0 / 0.0

// ==========================================
// 基础函数
// ==========================================

/// 绝对值
function abs(x: Float): Float {
  if x >= 0.0 { x }
  else { -x }
}

/// 整数绝对值
function abs_int(x: Int): Int {
  if x >= 0 { x }
  else { -x }
}

/// 符号函数：返回 -1, 0, 或 1
function signum(x: Float): Float {
  if x > 0.0 { 1.0 }
  else if x < 0.0 { -1.0 }
  else { 0.0 }
}

/// 整数符号函数
function signum_int(x: Int): Int {
  if x > 0 { 1 }
  else if x < 0 { -1 }
  else { 0 }
}

// ==========================================
// 幂函数和平方根
// ==========================================

/// 平方根
function sqrt(x: Float): Float {
  // 内置函数
  "__builtin_sqrt"
}

/// 平方
function square(x: Float): Float {
  x * x
}

/// 整数平方
function square_int(x: Int): Int {
  x * x
}

/// 幂运算：x^y
function pow(x: Float, y: Float): Float {
  // 内置函数
  "__builtin_pow"
}

/// 整数幂运算（只支持非负指数）
function pow_int(x: Int, y: Int): Int {
  if y < 0 {
    panic("pow_int: 指数不能为负")
  }
  let mut result = 1
  let mut base = x
  let mut exp = y
  while exp > 0 {
    if exp % 2 == 1 {
      result = result * base
    }
    base = base * base
    exp = exp / 2
  }
  result
}

/// 立方根
function cbrt(x: Float): Float {
  pow(x, 1.0 / 3.0)
}

/// 平方根的倒数
function rsqrt(x: Float): Float {
  1.0 / sqrt(x)
}

// ==========================================
// 指数和对数函数
// ==========================================

/// e 的 x 次幂
function exp(x: Float): Float {
  // 内置函数
  "__builtin_exp"
}

/// 2 的 x 次幂
function exp2(x: Float): Float {
  pow(2.0, x)
}

/// 自然对数（以 e 为底）
function ln(x: Float): Float {
  // 内置函数
  "__builtin_ln"
}

/// 以 2 为底的对数
function log2(x: Float): Float {
  ln(x) / ln2
}

/// 以 10 为底的对数
function log10(x: Float): Float {
  ln(x) / ln10
}

/// 以任意数为底的对数
function log(base: Float, x: Float): Float {
  ln(x) / ln(base)
}

// ==========================================
// 三角函数
// ==========================================

/// 正弦函数（弧度）
function sin(x: Float): Float {
  // 内置函数
  "__builtin_sin"
}

/// 余弦函数（弧度）
function cos(x: Float): Float {
  // 内置函数
  "__builtin_cos"
}

/// 正切函数（弧度）
function tan(x: Float): Float {
  sin(x) / cos(x)
}

/// 反正弦函数（返回弧度）
function asin(x: Float): Float {
  // 内置函数
  "__builtin_asin"
}

/// 反余弦函数（返回弧度）
function acos(x: Float): Float {
  // 内置函数
  "__builtin_acos"
}

/// 反正切函数（返回弧度）
function atan(x: Float): Float {
  // 内置函数
  "__builtin_atan"
}

/// 反正切函数，返回 (x, y) 的角度
function atan2(y: Float, x: Float): Float {
  // 内置函数
  "__builtin_atan2"
}

// ==========================================
// 双曲函数
// ==========================================

/// 双曲正弦
function sinh(x: Float): Float {
  (exp(x) - exp(-x)) / 2.0
}

/// 双曲余弦
function cosh(x: Float): Float {
  (exp(x) + exp(-x)) / 2.0
}

/// 双曲正切
function tanh(x: Float): Float {
  sinh(x) / cosh(x)
}

// ==========================================
// 取整函数
// ==========================================

/// 向下取整
function floor(x: Float): Float {
  // 内置函数
  "__builtin_floor"
}

/// 向上取整
function ceil(x: Float): Float {
  // 内置函数
  "__builtin_ceil"
}

/// 四舍五入
function round(x: Float): Float {
  floor(x + 0.5)
}

/// 截断小数部分
function trunc(x: Float): Float {
  if x >= 0.0 { floor(x) }
  else { ceil(x) }
}

/// 小数部分
function fract(x: Float): Float {
  x - trunc(x)
}

// ==========================================
// 极值函数
// ==========================================

/// 两个数中的较小值
function min(a: Float, b: Float): Float {
  if a < b { a }
  else { b }
}

/// 两个整数中的较小值
function min_int(a: Int, b: Int): Int {
  if a < b { a }
  else { b }
}

/// 两个数中的较大值
function max(a: Float, b: Float): Float {
  if a > b { a }
  else { b }
}

/// 两个整数中的较大值
function max_int(a: Int, b: Int): Int {
  if a > b { a }
  else { b }
}

/// 限制值在 [min, max] 范围内
function clamp(x: Float, min_val: Float, max_val: Float): Float {
  min(max(x, min_val), max_val)
}

/// 限制整数在 [min, max] 范围内
function clamp_int(x: Int, min_val: Int, max_val: Int): Int {
  min_int(max_int(x, min_val), max_val)
}

// ==========================================
// 角度转换
// ==========================================

/// 将角度转换为弧度
function radians(degrees: Float): Float {
  degrees * pi / 180.0
}

/// 将弧度转换为角度
function degrees(radians: Float): Float {
  radians * 180.0 / pi
}

// ==========================================
// 距离和插值
// ==========================================

/// 线性插值：在 a 和 b 之间按 t 插值
function lerp(a: Float, b: Float, t: Float): Float {
  a + t * (b - a)
}

/// 计算两个点之间的欧几里得距离
function distance(x1: Float, y1: Float, x2: Float, y2: Float): Float {
  sqrt(square(x2 - x1) + square(y2 - y1))
}

/// 曼哈顿距离
function manhattan_distance(x1: Float, y1: Float, x2: Float, y2: Float): Float {
  abs(x2 - x1) + abs(y2 - y1)
}

// ==========================================
// 随机数（伪随机，简单实现）
// ==========================================

let mut rng_seed: Int = 12345

/// 设置随机种子
function srand(seed: Int) {
  rng_seed = seed
}

/// 生成 0 到 1 之间的随机浮点数
function rand(): Float {
  // 简单的 LCG 随机数生成器
  rng_seed = (rng_seed * 1103515245 + 12345) % 2147483648
  abs_int(rng_seed) as Float / 2147483648.0
}

/// 生成指定范围内的随机整数 [min, max)
function rand_int(min: Int, max: Int): Int {
  min + (rand() * (max - min) as Float) as Int
}

/// 生成指定范围内的随机浮点数 [min, max)
function rand_float(min: Float, max: Float): Float {
  min + rand() * (max - min)
}

// ==========================================
// 除法和余数
// ==========================================

/// 欧几里得除法（总是向负无穷方向舍入）
function div_euclid(a: Int, b: Int): Int {
  let q = a / b
  if (a % b < 0 && b > 0) { q - 1 }
  else if (a % b > 0 && b < 0) { q - 1 }
  else { q }
}

/// 欧几里得余数（总是非负）
function rem_euclid(a: Int, b: Int): Int {
  a - div_euclid(a, b) * b
}

// ==========================================
// 最大公约数和最小公倍数
// ==========================================

/// 最大公约数
function gcd(a: Int, b: Int): Int {
  let mut x = abs_int(a)
  let mut y = abs_int(b)
  while y != 0 {
    let temp = y
    y = x % y
    x = temp
  }
  x
}

/// 最小公倍数
function lcm(a: Int, b: Int): Int {
  if a == 0 || b == 0 { 0 }
  else { abs_int(a * b) / gcd(a, b) }
}

// ==========================================
// 因数和质数检查
// ==========================================

/// 检查是否是偶数
function is_even(n: Int): Bool {
  n % 2 == 0
}

/// 检查是否是奇数
function is_odd(n: Int): Bool {
  n % 2 != 0
}

/// 检查是否是质数
function is_prime(n: Int): Bool {
  if n <= 1 { false }
  else if n == 2 { true }
  else if is_even(n) { false }
  else {
    let mut i = 3
    while i * i <= n {
      if n % i == 0 {
        return false
      }
      i = i + 2
    }
    true
  }
}

// ==========================================
// 阶乘
// ==========================================

/// 阶乘
function factorial(n: Int): Int {
  if n < 0 {
    panic("factorial: 参数不能为负")
  }
  let mut result = 1
  let mut i = 2
  while i <= n {
    result = result * i
    i = i + 1
  }
  result
}

/// 斐波那契数列
function fibonacci(n: Int): Int {
  if n < 0 {
    panic("fibonacci: 参数不能为负")
  }
  if n == 0 { 0 }
  else if n == 1 { 1 }
  else {
    let mut a = 0
    let mut b = 1
    let mut i = 2
    while i <= n {
      let temp = b
      b = a + b
      a = temp
      i = i + 1
    }
    b
  }
}