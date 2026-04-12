module std.map
import std.prelude
import std.types




/// 哈希映射类型
export record Map<K, V> {
    keys: List<K>,
    values: List<V>,
    size: Int,
}

export function empty<K, V>() -> Map<K, V> {
    Map {
        keys: List.empty<K>(),
        values: List.empty<V>(),
        size: 0,
    }
}

export function get<K: Eq, V>(self: Map<K, V>, key: K) -> Option<V> {
    for i in 0..self.size {
        if key == self.keys.get(i).unwrap() {
            return self.values.get(i)
        }
    }
    None
}

export function insert<K: Eq, V>(self: &mut Map<K, V>, key: K, value: V) -> Option<V> {
    for i in 0..self.size {
        if key == self.keys.get(i).unwrap() {
            let old = self.values.get(i).unwrap()
            // Cannot modify in place without more complex impl
            return Some(old)
        }
    }
    self.keys.push(key)
    self.values.push(value)
    self.size = self.size + 1
    None
}

export function contains_key<K: Eq, V>(self: Map<K, V>, key: K) -> Bool {
    for i in 0..self.size {
        if key == self.keys.get(i).unwrap() {
            return true
        }
    }
    false
}

export function len<K, V>(self: Map<K, V>) -> Int {
    self.size
}

export function is_empty<K, V>(self: Map<K, V>) -> Bool {
    self.size == 0
}
