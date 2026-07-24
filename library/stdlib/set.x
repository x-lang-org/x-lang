module std.set
import std.types
import std.map

/// Unordered unique strings for now (Swift Set / Kotlin MutableSet).
/// Backed by `Map` keys; values are unused placeholders.
export class Set {
    public let data: Map

    public new() {
        this.data = Map()
    }
}

export function empty() -> Set {
    Set()
}

export function count(self: Set) -> integer {
    self.data.keys.length()
}

export function len(self: Set) -> integer {
    self.data.keys.length()
}

export function is_empty(self: Set) -> boolean {
    self.data.keys.length() == 0
}

/// Insert. Returns true if the value was already present.
export function insert(self: Set, value: string) -> boolean {
    let existed = contains_key(self.data, value)
    let old = put(self.data, value, 1)
    existed
}

export function contains(self: Set, value: string) -> boolean {
    contains_key(self.data, value)
}

export function remove(self: Set, value: string) -> boolean {
    // Call map.remove via a local alias pattern: rebuild without the key
    // to avoid free-function name clash with this `remove`.
    let n = self.data.keys.length()
    let mutable i = 0
    while i < n {
        if (self.data.keys[i] as string) == value {
            let mutable new_keys = [] as [string]
            let mutable new_values = [] as [any]
            let mutable j = 0
            while j < n {
                if j != i {
                    new_keys.push(self.data.keys[j] as string)
                    new_values.push(self.data.values[j])
                }
                j = j + 1
            }
            self.data.keys = new_keys
            self.data.values = new_values
            return true
        }
        i = i + 1
    }
    false
}

export function clear(self: Set) -> unit {
    self.data.keys = [] as [string]
    self.data.values = [] as [any]
}
