module std.list
import std.types

/// Growable list (Swift Array / Kotlin MutableList).
export class List {
    public let items: [any]

    public new() {
        this.items = [] as [any]
    }
}

export function empty() -> List {
    List()
}

export function count(self: List) -> integer {
    self.items.length()
}

export function len(self: List) -> integer {
    count(self)
}

export function is_empty(self: List) -> boolean {
    count(self) == 0
}

export function push(self: List, value: any) -> unit {
    self.items.push(value)
}

export function get(self: List, index: integer) -> Option<any> {
    let n = self.items.length()
    if index < 0 or index >= n {
        None
    } else {
        Some(self.items[index])
    }
}

export function first(self: List) -> Option<any> {
    get(self, 0)
}

export function last(self: List) -> Option<any> {
    let n = self.items.length()
    if n == 0 {
        None
    } else {
        Some(self.items[n - 1])
    }
}

export function pop(self: List) -> Option<any> {
    let n = self.items.length()
    if n == 0 {
        None
    } else {
        let v = self.items[n - 1]
        let mutable new_items = [] as [any]
        let mutable i = 0
        while i < n - 1 {
            new_items.push(self.items[i])
            i = i + 1
        }
        self.items = new_items
        Some(v)
    }
}

export function contains(self: List, value: any) -> boolean {
    let n = self.items.length()
    let mutable i = 0
    while i < n {
        if self.items[i] == value {
            return true
        }
        i = i + 1
    }
    false
}

export function clear(self: List) -> unit {
    self.items = [] as [any]
}

/// Copy out the underlying array.
export function to_array(self: List) -> [any] {
    self.items
}
