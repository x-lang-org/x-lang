module std.set
import std.prelude
import std.types





/// 哈希集合类型
export record Set<T> {
    items: List<T>,
}

export function empty<T>() -> Set<T> {
    Set {
        items: List.empty<T>(),
    }
}

export function from_array<T: Eq>(arr: [T]) -> Set<T> {
    let mut set = Set.empty<T>()
    for elem in arr {
        set.insert(elem)
    }
    set
}

export function insert<T: Eq>(self: &mut Set<T>, value: T) -> Bool {
    if self.contains(value) {
        return false
    }
    self.items.push(value)
    true
}

export function remove<T: Eq>(self: &mut Set<T>, value: T) -> Bool {
    for i in 0..self.items.len() {
        if value == self.items.get(i).unwrap() {
            self.items.remove(i)
            return true
        }
    }
    false
}

export function contains<T: Eq>(self: Set<T>, value: T) -> Bool {
    for i in 0..self.items.len() {
        if value == self.items.get(i).unwrap() {
            return true
        }
    }
    false
}

export function len<T>(self: Set<T>) -> Int {
    self.items.len()
}

export function is_empty<T>(self: Set<T>) -> Bool {
    self.items.is_empty()
}

export function to_list<T>(self: Set<T>) -> List<T> {
    self.items
}
