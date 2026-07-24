module std.string
import std.prelude
import std.types

/// Character count (Swift `count` / Kotlin `length`).
export function count(s: string) -> integer {
    string_length(s)
}

export function length(s: string) -> integer {
    string_length(s)
}

export function is_empty(s: string) -> boolean {
    string_length(s) == 0
}

export function concat(a: string, b: string) -> string {
    a ++ b
}

export function contains(s: string, substring: string) -> boolean {
    string_contains(s, substring)
}

/// Swift `hasPrefix` / Kotlin `startsWith`.
export function has_prefix(s: string, prefix: string) -> boolean {
    let n = string_length(s)
    let m = string_length(prefix)
    if m > n {
        return false
    }
    string_substring(s, 0, m) == prefix
}

/// Swift `hasSuffix` / Kotlin `endsWith`.
export function has_suffix(s: string, suffix: string) -> boolean {
    let n = string_length(s)
    let m = string_length(suffix)
    if m > n {
        return false
    }
    string_substring(s, n - m, n) == suffix
}

export function substring(s: string, start: integer, end: integer) -> string {
    string_substring(s, start, end)
}

export function trim(s: string) -> string {
    string_trim(s)
}

/// Swift `uppercased` / Kotlin `uppercase`.
export function uppercased(s: string) -> string {
    let mutable out = ""
    let n = string_length(s)
    let mutable i = 0
    while i < n {
        let ch = string_substring(s, i, i + 1)
        if ch == "a" { out = out ++ "A" }
        else if ch == "b" { out = out ++ "B" }
        else if ch == "c" { out = out ++ "C" }
        else if ch == "d" { out = out ++ "D" }
        else if ch == "e" { out = out ++ "E" }
        else if ch == "f" { out = out ++ "F" }
        else if ch == "g" { out = out ++ "G" }
        else if ch == "h" { out = out ++ "H" }
        else if ch == "i" { out = out ++ "I" }
        else if ch == "j" { out = out ++ "J" }
        else if ch == "k" { out = out ++ "K" }
        else if ch == "l" { out = out ++ "L" }
        else if ch == "m" { out = out ++ "M" }
        else if ch == "n" { out = out ++ "N" }
        else if ch == "o" { out = out ++ "O" }
        else if ch == "p" { out = out ++ "P" }
        else if ch == "q" { out = out ++ "Q" }
        else if ch == "r" { out = out ++ "R" }
        else if ch == "s" { out = out ++ "S" }
        else if ch == "t" { out = out ++ "T" }
        else if ch == "u" { out = out ++ "U" }
        else if ch == "v" { out = out ++ "V" }
        else if ch == "w" { out = out ++ "W" }
        else if ch == "x" { out = out ++ "X" }
        else if ch == "y" { out = out ++ "Y" }
        else if ch == "z" { out = out ++ "Z" }
        else { out = out ++ ch }
        i = i + 1
    }
    out
}

/// Swift `lowercased` / Kotlin `lowercase`.
export function lowercased(s: string) -> string {
    let mutable out = ""
    let n = string_length(s)
    let mutable i = 0
    while i < n {
        let ch = string_substring(s, i, i + 1)
        if ch == "A" { out = out ++ "a" }
        else if ch == "B" { out = out ++ "b" }
        else if ch == "C" { out = out ++ "c" }
        else if ch == "D" { out = out ++ "d" }
        else if ch == "E" { out = out ++ "e" }
        else if ch == "F" { out = out ++ "f" }
        else if ch == "G" { out = out ++ "g" }
        else if ch == "H" { out = out ++ "h" }
        else if ch == "I" { out = out ++ "i" }
        else if ch == "J" { out = out ++ "j" }
        else if ch == "K" { out = out ++ "k" }
        else if ch == "L" { out = out ++ "l" }
        else if ch == "M" { out = out ++ "m" }
        else if ch == "N" { out = out ++ "n" }
        else if ch == "O" { out = out ++ "o" }
        else if ch == "P" { out = out ++ "p" }
        else if ch == "Q" { out = out ++ "q" }
        else if ch == "R" { out = out ++ "r" }
        else if ch == "S" { out = out ++ "s" }
        else if ch == "T" { out = out ++ "t" }
        else if ch == "U" { out = out ++ "u" }
        else if ch == "V" { out = out ++ "v" }
        else if ch == "W" { out = out ++ "w" }
        else if ch == "X" { out = out ++ "x" }
        else if ch == "Y" { out = out ++ "y" }
        else if ch == "Z" { out = out ++ "z" }
        else { out = out ++ ch }
        i = i + 1
    }
    out
}

export function split(s: string, delimiter: string) -> [string] {
    string_split(s, delimiter)
}

/// Join string parts with a separator (Swift `joined(separator:)`).
export function joined(parts: [string], separator: string) -> string {
    let n = parts.length()
    if n == 0 {
        return ""
    }
    let mutable out = parts[0] as string
    let mutable i = 1
    while i < n {
        out = out ++ separator ++ (parts[i] as string)
        i = i + 1
    }
    out
}

export function repeat(s: string, n: integer) -> string {
    let mutable out = ""
    let mutable i = 0
    while i < n {
        out = out ++ s
        i = i + 1
    }
    out
}
