module std.list
import std.prelude
import std.types




/// 动态数组类型
export record List<T> {
    data: [T],
    length: Int,
    capacity: Int,
}

export function empty<T>() -> List<T> {
    List {
        data: [],
        length: 0,
        capacity: 0,
    }
}

export function from_array<T>(arr: [T]) -> List<T> {
    List {
        data: arr,
        length: arr.len(),
        capacity: arr.len(),
    }
}

export function get<T>(self: List<T>, index: Int) -> Option<T> {
    if index < 0 or index >= self.length {
        None
    } else {
        Some(self.data[index])
    }
}

export function first<T>(self: List<T>) -> Option<T> {
    if self.is_empty() {
        None
    } else {
        Some(self.data[0])
    }
}

export function last<T>(self: List<T>) -> Option<T> {
    if self.is_empty() {
        None
    } else {
        Some(self.data[self.length - 1])
    }
}

export function push<T>(self: &mut List<T>, value: T) -> unit {
    if self.length >= self.capacity {
        if self.capacity == 0 {
            self.capacity = 1
        } else {
            self.capacity = self.capacity * 2
        }
    }
    self.data.push(value)
    self.length = self.length + 1
}

export function pop<T>(self: &mut List<T>) -> Option<T> {
    if self.is_empty() {
        None
    } else {
        self.length = self.length - 1
        self.data.pop()
    }
}

export function insert<T>(self: &mut List<T>, index: Int, value: T) -> unit {
    if index < 0 or index > self.length {
        panic("List.insert: index out of bounds")
    }
    self.data.insert(index, value)
    self.length = self.length + 1
}

export function remove<T>(self: &mut List<T>, index: Int) -> T {
    if index < 0 or index >= self.length {
        panic("List.remove: index out of bounds")
    }
    self.length = self.length - 1
    self.data.remove(index)
}

export function clear<T>(self: &mut List<T>) -> unit {
    self.data.clear()
    self.length = 0
}

export function len<T>(self: List<T>) -> Int {
    self.length
}

export function is_empty<T>(self: List<T>) -> Bool {
    self.length == 0
}

export function contains<T: Eq>(self: List<T>, value: T) -> Bool {
    for i in 0..self.length {
        if value == self.data[i] {
            return true
        }
    }
    false
}

export function to_array<T>(self: List<T>) -> [T] {
    self.data
}
