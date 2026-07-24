// std.prelude — automatically imported into every module.
// These are standard-library APIs (with C FFI where needed), not compiler builtins.

import std.types;

/// libc
external function puts(message: *character) -> signed 32-bit integer
external function putchar(c: signed 32-bit integer) -> signed 32-bit integer
external function abort() -> never
external function strlen(s: *character) -> unsigned 64-bit integer

/// Array representation used for `[T]` (linked from runtime).
external "c" function x_list_len(list: any) -> integer
external "c" function x_list_push(list: any, item: any) -> unit

/// libc math (UFCS: `x.sqrt()`, `x.floor()`, …)
external "c" function sqrt(x: float) -> float
external "c" function floor(x: float) -> float
external "c" function ceil(x: float) -> float
external "c" function fabs(x: float) -> float
external "c" function pow(x: float, y: float) -> float

/// Integer absolute value (`n.abs()`).
export function abs(x: integer) -> integer {
    if x < 0 {
        -x
    } else {
        x
    }
}

export function println(message: string) -> unit {
    unsafe {
        let result = puts(message as *character);
    }
}

export function print(message: string) -> unit {
    for c in message {
        unsafe {
            let result = putchar(c as signed 32-bit integer);
        }
    }
}

export function panic(message: string) -> never {
    println(message);
    unsafe {
        abort()
    }
}

export function assert(condition: boolean) -> unit {
    if not condition {
        panic("assertion failed")
    }
}

/// `s.length()` → string_length(s)
export function string_length(s: string) -> integer {
    unsafe {
        strlen(s as *character) as integer
    }
}

/// `xs.length()` → array_length(xs)
export function array_length(xs: any) -> integer {
    unsafe {
        x_list_len(xs)
    }
}

/// `xs.push(x)` → array_push(xs, x)
export function array_push(xs: any, item: any) -> unit {
    unsafe {
        x_list_push(xs, item)
    }
}

/// `s.substring(start, end)` → string_substring(s, start, end)
export function string_substring(s: string, start: integer, end: integer) -> string {
    let mut i = start
    let mut out = ""
    while i < end {
        if i >= 0 {
            out = out ++ (s[i] as string)
        }
        i = i + 1
    }
    out
}

/// `s.contains(sub)` → string_contains(s, sub)
export function string_contains(s: string, substring: string) -> boolean {
    let n = string_length(s)
    let m = string_length(substring)
    if m == 0 {
        return true
    }
    if m > n {
        return false
    }
    let mut i = 0
    while i + m <= n {
        let mut matched = true
        let mut j = 0
        while j < m {
            if s[i + j] != substring[j] {
                matched = false
                break
            }
            j = j + 1
        }
        if matched {
            return true
        }
        i = i + 1
    }
    false
}

/// `s.trim()` → string_trim(s)
export function string_trim(s: string) -> string {
    let n = string_length(s)
    let mut start = 0
    let mut end = n
    while start < end {
        let c = s[start]
        if c != ' ' and c != '\t' and c != '\n' and c != '\r' {
            break
        }
        start = start + 1
    }
    while end > start {
        let c = s[end - 1]
        if c != ' ' and c != '\t' and c != '\n' and c != '\r' {
            break
        }
        end = end - 1
    }
    string_substring(s, start, end)
}

/// `s.to_upper()` / `s.toUpperCase()` → string_to_upper(s)
export function string_to_upper(s: string) -> string {
    let mut out = ""
    let mut i = 0
    let n = string_length(s)
    while i < n {
        let c = s[i]
        if c >= 'a' and c <= 'z' {
            out = out ++ (((c as signed 32-bit integer) - 32) as character as string)
        } else {
            out = out ++ (c as string)
        }
        i = i + 1
    }
    out
}

/// `s.to_lower()` / `s.toLowerCase()` → string_to_lower(s)
export function string_to_lower(s: string) -> string {
    let mut out = ""
    let mut i = 0
    let n = string_length(s)
    while i < n {
        let c = s[i]
        if c >= 'A' and c <= 'Z' {
            out = out ++ (((c as signed 32-bit integer) + 32) as character as string)
        } else {
            out = out ++ (c as string)
        }
        i = i + 1
    }
    out
}

/// `s.split(delim)` → string_split(s, delim)
export function string_split(s: string, delimiter: string) -> [string] {
    let mut result: [string] = []
    let n = string_length(s)
    let dlen = string_length(delimiter)
    if dlen == 0 {
        result.push(s)
        return result
    }
    let mut start = 0
    let mut i = 0
    while i + dlen <= n {
        let mut matched = true
        let mut j = 0
        while j < dlen {
            if s[i + j] != delimiter[j] {
                matched = false
                break
            }
            j = j + 1
        }
        if matched {
            result.push(string_substring(s, start, i))
            i = i + dlen
            start = i
            continue
        }
        i = i + 1
    }
    result.push(string_substring(s, start, n))
    result
}

export function enumerate<T>(items: [T]) -> [(Int, T)] {
    let mut result: [(Int, T)] = [];
    let mut index = 0;
    for item in items {
        result.push((index as Int, item));
        index = index + 1;
    }
    result
}

/// Platform / process helpers (declared here; backends may supply them).
external function __file_read(path: string) -> Result<string, string>
external function __file_write(path: string, content: string) -> Result<unit, string>
external function __file_exists(path: string) -> boolean
external function __file_delete(path: string) -> Result<unit, string>
external function unwrap_ok<T, E>(res: Result<T, E>) -> T
external function __args() -> Array<string>
external function x_json_parse(json: string) -> string
external function __get_env(name: string) -> Option<string>
