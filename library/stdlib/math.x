module std.math
import std.prelude
import std.types

/// Mathematical constants (Swift / Kotlin style).
export const pi: float = 3.14159265358979323846
export const e: float = 2.71828182845904523536
export const ln2: float = 0.693147180559945309417
export const ln10: float = 2.30258509299404568402

// libc math — call directly (UFCS: `x.sqrt()`, `x.sin()`, … via prelude aliases where wired).
external "c" function sqrt(x: float) -> float
external "c" function cbrt(x: float) -> float
external "c" function pow(x: float, y: float) -> float
external "c" function exp(x: float) -> float
external "c" function exp2(x: float) -> float
external "c" function log(x: float) -> float
external "c" function log10(x: float) -> float
external "c" function log2(x: float) -> float
external "c" function sin(x: float) -> float
external "c" function cos(x: float) -> float
external "c" function tan(x: float) -> float
external "c" function asin(x: float) -> float
external "c" function acos(x: float) -> float
external "c" function atan(x: float) -> float
external "c" function atan2(y: float, x: float) -> float
external "c" function sinh(x: float) -> float
external "c" function cosh(x: float) -> float
external "c" function tanh(x: float) -> float
external "c" function hypot(x: float, y: float) -> float
external "c" function ceil(x: float) -> float
external "c" function floor(x: float) -> float
external "c" function round(x: float) -> float
external "c" function trunc(x: float) -> float
external "c" function fabs(x: float) -> float
external "c" function fmod(x: float, y: float) -> float
external "c" function fmax(x: float, y: float) -> float
external "c" function fmin(x: float, y: float) -> float
external "c" function copysign(x: float, y: float) -> float

/// Absolute value (floating). Integer `abs` lives in prelude.
export function abs_f(x: float) -> float {
    fabs(x)
}

export function max_f(a: float, b: float) -> float {
    fmax(a, b)
}

export function min_f(a: float, b: float) -> float {
    fmin(a, b)
}

export function max_i(a: integer, b: integer) -> integer {
    if a > b { a } else { b }
}

export function min_i(a: integer, b: integer) -> integer {
    if a < b { a } else { b }
}

export function clamp_f(x: float, lo: float, hi: float) -> float {
    fmax(lo, fmin(x, hi))
}

export function clamp_i(x: integer, lo: integer, hi: integer) -> integer {
    max_i(lo, min_i(x, hi))
}

export function degrees_to_radians(degrees: float) -> float {
    degrees * pi / 180.0
}

export function radians_to_degrees(radians: float) -> float {
    radians * 180.0 / pi
}

export function is_nan(x: float) -> boolean {
    x != x
}

export function lerp(a: float, b: float, t: float) -> float {
    a + t * (b - a)
}
