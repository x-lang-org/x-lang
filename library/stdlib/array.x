// std.array — built-in array (`[T]`) operations via the list representation FFI.
// Language surface: `xs.length()`, `xs.push(x)` resolve here through UFCS / method desugar.

module std.array

import std.prelude
import std.types

/// Runtime list length (representation for `[T]`).
external "c" function x_list_len(list: any) -> integer

/// Runtime list push (element must already be representation-compatible).
external "c" function x_list_push(list: any, item: any) -> unit

/// Length of an array.
export function array_length(xs: any) -> integer {
    unsafe {
        x_list_len(xs)
    }
}

/// Append one element to an array (in place when uniquely owned).
export function array_push(xs: any, item: any) -> unit {
    unsafe {
        x_list_push(xs, item)
    }
}
