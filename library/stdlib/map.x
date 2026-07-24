module std.map
import std.types

/// Key-value map (Swift Dictionary / Kotlin MutableMap).
/// Linear lookup over parallel arrays — correct first; hashing later.
export class Map {
    public let keys: [string]
    public let values: [any]

    public new() {
        this.keys = [] as [string]
        this.values = [] as [any]
    }
}

/// Create an empty map.
export function empty() -> Map {
    Map()
}

/// Number of entries.
export function count(self: Map) -> integer {
    self.keys.length()
}

/// Alias for `count` (Kotlin `size` / Swift `count`).
export function len(self: Map) -> integer {
    count(self)
}

export function is_empty(self: Map) -> boolean {
    count(self) == 0
}

/// Look up `key`. Missing → `None`.
export function get(self: Map, key: string) -> Option<any> {
    let n = self.keys.length()
    let mutable i = 0
    while i < n {
        if (self.keys[i] as string) == key {
            return Some(self.values[i])
        }
        i = i + 1
    }
    None
}

/// Insert or replace. Returns previous value if the key existed.
export function put(self: Map, key: string, value: any) -> Option<any> {
    let n = self.keys.length()
    let mutable i = 0
    while i < n {
        if (self.keys[i] as string) == key {
            let old = self.values[i]
            self.values[i] = value
            return Some(old)
        }
        i = i + 1
    }
    self.keys.push(key)
    self.values.push(value)
    None
}

/// Remove `key`. Returns the removed value if present.
export function remove(self: Map, key: string) -> Option<any> {
    let n = self.keys.length()
    let mutable i = 0
    while i < n {
        if (self.keys[i] as string) == key {
            let old = self.values[i]
            // Compact by swapping with last (order of keys is not significant).
            let last = n - 1
            if i != last {
                self.keys[i] = self.keys[last]
                self.values[i] = self.values[last]
            }
            // Shrink by rebuilding without the last slot via pop-equivalent:
            // push-only arrays: rebuild.
            let mutable new_keys = [] as [string]
            let mutable new_values = [] as [any]
            let mutable j = 0
            while j < last {
                new_keys.push(self.keys[j] as string)
                new_values.push(self.values[j])
                j = j + 1
            }
            self.keys = new_keys
            self.values = new_values
            return Some(old)
        }
        i = i + 1
    }
    None
}

export function contains_key(self: Map, key: string) -> boolean {
    let n = self.keys.length()
    let mutable i = 0
    while i < n {
        if (self.keys[i] as string) == key {
            return true
        }
        i = i + 1
    }
    false
}

/// Value for `key`, or `default` when missing (Kotlin `getOrDefault`).
export function get_or_default(self: Map, key: string, default: any) -> any {
    match get(self, key) {
        Some(v) => v,
        None => default,
    }
}

export function clear(self: Map) -> unit {
    self.keys = [] as [string]
    self.values = [] as [any]
}
