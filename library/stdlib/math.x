module std.math
import std.prelude
import std.types



/// 数学常数 π
export const pi: Float = 3.14159265358979323846;

/// 数学常数 e
export const e: Float = 2.71828182845904523536;

/// 欧拉常数 γ
export const euler_gamma: Float = 0.577215664901532860606;

/// 自然对数的 2
export const ln2: Float = 0.693147180559945309417;

/// 自然对数的 10
export const ln10: Float = 2.30258509299404568402;

// === 外部 C 库函数绑定 ===

external "c" function sqrt(x: Float) -> Float
external "c" function cbrt(x: Float) -> Float
external "c" function pow(x: Float, y: Float) -> Float
external "c" function exp(x: Float) -> Float
external "c" function exp2(x: Float) -> Float
external "c" function log(x: Float) -> Float
external "c" function log10(x: Float) -> Float
external "c" function log2(x: Float) -> Float
external "c" function sin(x: Float) -> Float
external "c" function cos(x: Float) -> Float
external "c" function tan(x: Float) -> Float
external "c" function asin(x: Float) -> Float
external "c" function acos(x: Float) -> Float
external "c" function atan(x: Float) -> Float
external "c" function atan2(y: Float, x: Float) -> Float
external "c" function sinh(x: Float) -> Float
external "c" function cosh(x: Float) -> Float
external "c" function tanh(x: Float) -> Float
external "c" function asinh(x: Float) -> Float
external "c" function acosh(x: Float) -> Float
external "c" function atanh(x: Float) -> Float
external "c" function hypot(x: Float, y: Float) -> Float
external "c" function ceil(x: Float) -> Float
external "c" function floor(x: Float) -> Float
external "c" function round(x: Float) -> Float
external "c" function trunc(x: Float) -> Float
external "c" function fabs(x: Float) -> Float
external "c" function fmod(x: Float, y: Float) -> Float
external "c" function remainder(x: Float, y: Float) -> Float
external "c" function copysign(x: Float, y: Float) -> Float
external "c" function nextafter(x: Float, y: Float) -> Float
external "c" function fdim(x: Float, y: Float) -> Float
external "c" function fmax(x: Float, y: Float) -> Float
external "c" function fmin(x: Float, y: Float) -> Float

// === 基础函数 ===

/// 平方根
export function sqrt(x: Float) -> Float {
    unsafe { sqrt(x) }
}

/// 立方根
export function cbrt(x: Float) -> Float {
    unsafe { cbrt(x) }
}

/// x 的 y 次幂
export function pow(x: Float, y: Float) -> Float {
    unsafe { pow(x, y) }
}

/// e 的 x 次幂
export function exp(x: Float) -> Float {
    unsafe { exp(x) }
}

/// 2 的 x 次幂
export function exp2(x: Float) -> Float {
    unsafe { exp2(x) }
}

/// 自然对数 (以 e 为底)
export function log(x: Float) -> Float {
    unsafe { log(x) }
}

/// 常用对数 (以 10 为底)
export function log10(x: Float) -> Float {
    unsafe { log10(x) }
}

/// 二进制对数 (以 2 为底)
export function log2(x: Float) -> Float {
    unsafe { log2(x) }
}

// === 三角函数 ===

/// 正弦
export function sin(x: Float) -> Float {
    unsafe { sin(x) }
}

/// 余弦
export function cos(x: Float) -> Float {
    unsafe { cos(x) }
}

/// 正切
export function tan(x: Float) -> Float {
    unsafe { tan(x) }
}

/// 反正弦
export function asin(x: Float) -> Float {
    unsafe { asin(x) }
}

/// 反余弦
export function acos(x: Float) -> Float {
    unsafe { acos(x) }
}

/// 反正切
export function atan(x: Float) -> Float {
    unsafe { atan(x) }
}

/// y/x 的反正切
export function atan2(y: Float, x: Float) -> Float {
    unsafe { atan2(y, x) }
}

// === 双曲函数 ===

/// 双曲正弦
export function sinh(x: Float) -> Float {
    unsafe { sinh(x) }
}

/// 双曲余弦
export function cosh(x: Float) -> Float {
    unsafe { cosh(x) }
}

/// 双曲正切
export function tanh(x: Float) -> Float {
    unsafe { tanh(x) }
}

/// 反双曲正弦
export function asinh(x: Float) -> Float {
    unsafe { asinh(x) }
}

/// 反双曲余弦
export function acosh(x: Float) -> Float {
    unsafe { acosh(x) }
}

/// 反双曲正切
export function atanh(x: Float) -> Float {
    unsafe { atanh(x) }
}

// === 其他几何函数 ===

/// 计算直角三角形斜边 sqrt(x^2 + y^2)
export function hypot(x: Float, y: Float) -> Float {
    unsafe { hypot(x, y) }
}

// === 取整函数 ===

/// 向上取整
export function ceil(x: Float) -> Float {
    unsafe { ceil(x) }
}

/// 向下取整
export function floor(x: Float) -> Float {
    unsafe { floor(x) }
}

/// 四舍五入
export function round(x: Float) -> Float {
    unsafe { round(x) }
}

/// 截断小数部分
export function trunc(x: Float) -> Float {
    unsafe { trunc(x) }
}

/// 绝对值
export function abs(x: Float) -> Float {
    unsafe { fabs(x) }
}

/// 绝对值（整数版本）
export function abs(x: Int) -> Int {
    when x < 0 {
        -x
    } else {
        x
    }
}

// === 余数函数 ===

/// 浮点数取余
export function fmod(x: Float, y: Float) -> Float {
    unsafe { fmod(x, y) }
}

/// 余数
export function remainder(x: Float, y: Float) -> Float {
    unsafe { remainder(x, y) }
}

// === 比较函数 ===

/// 最大值
export function max(a: Float, b: Float) -> Float {
    unsafe { fmax(a, b) }
}

/// 最小值
export function min(a: Float, b: Float) -> Float {
    unsafe { fmin(a, b) }
}

/// 正差：如果 x > y 返回 x - y，否则返回 0
export function fdim(x: Float, y: Float) -> Float {
    unsafe { fdim(x, y) }
}

// === 整数版本比较函数 ===

/// 最大值（整数）
export function max(a: Int, b: Int) -> Int {
    when a > b {
        a
    } else {
        b
    }
}

/// 最小值（整数）
export function min(a: Int, b: Int) -> Int {
    when a < b {
        a
    } else {
        b
    }
}

// === 其他工具函数 ===

/// 复制符号：返回 x 的大小，但使用 y 的符号
export function copysign(x: Float, y: Float) -> Float {
    unsafe { copysign(x, y) }
}

/// 下一个可表示的浮点数
export function nextafter(x: Float, y: Float) -> Float {
    unsafe { nextafter(x, y) }
}

/// 角度转弧度
export function degrees_to_radians(degrees: Float) -> Float {
    degrees * pi / 180.0
}

/// 弧度转角度
export function radians_to_degrees(radians: Float) -> Float {
    radians * 180.0 / pi
}

/// 检查是否为 NaN
export function is_nan(x: Float) -> Bool {
    // NaN 不等于任何值，包括自身
    x != x
}

/// 检查是否为无穷大
export function is_infinite(x: Float) -> Bool {
    abs(x) > 1e308
}

/// 检查是否为有限值（不是 NaN 也不是无穷大）
export function is_finite(x: Float) -> Bool {
    not (is_nan(x) or is_infinite(x))
}

/// 阶乘（迭代实现）
export function factorial(n: Int) -> Int {
    when n < 0 {
        panic("factorial: negative input")
    }
    when n == 0 or n == 1 {
        1
    } else {
        let mut result = 1;
        let mut i = 2;
        while i <= n {
            result = result * i;
            i = i + 1;
        }
        result
    }
}

/// 组合数 C(n, k)
export function combination(n: Int, k: Int) -> Int {
    when k < 0 or k > n {
        0
    }
    when k == 0 or k == n {
        1
    }
    // 优化：计算较小的那个
    let k = min(k, n - k);
    let mut result = 1;
    let mut i = 1;
    while i <= k {
        result = result * (n - k + i) / i;
        i = i + 1;
    }
    result
}

/// 勾股定理：计算斜边长度
export function pythagorean(a: Float, b: Float) -> Float {
    hypot(a, b)
}

/// 线性插值
export function lerp(a: Float, b: Float, t: Float) -> Float {
    a + t * (b - a)
}

/// 钳制值到范围 [min, max]
export function clamp(x: Float, min_val: Float, max_val: Float) -> Float {
    max(min_val, min(x, max_val))
}

/// 钳制整数值到范围 [min, max]
export function clamp(x: Int, min_val: Int, max_val: Int) -> Int {
    max(min_val, min(x, max_val))
}
