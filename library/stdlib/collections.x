module std.collections

/// Advanced collections built on `std.list` / `std.map`.
/// Prefer importing `std.list`, `std.map`, `std.set` directly for everyday use.

import std.list
import std.map
import std.set

export function empty_list() -> List {
    List()
}

export function empty_map() -> Map {
    Map()
}

export function empty_set() -> Set {
    Set()
}
