module std.types

/// Option 类型 - 表示可能存在或不存在的值
/// 由标准库提供，不是编译器内置
pub enum Option<T> {
    /// 没有值
    None,
    /// 有值
    Some(T),
}

/// Result 类型 - 表示成功或失败的结果
/// 由标准库提供，不是编译器内置
pub enum Result<T, E> {
    /// 成功结果
    Ok(T),
    /// 错误结果
    Err(E),
}

/// Option 类型扩展方法
import std::prelude::*;

/// 判断 Option 是否为 None
export function is_none<T>(self: Option<T>) -> Bool {
    match self {
        None => true,
        Some(_) => false,
    }
}

/// 判断 Option 是否为 Some
export function is_some<T>(self: Option<T>) -> Bool {
    match self {
        None => false,
        Some(_) => true,
    }
}

///  unwrap 获取值，如果是 None 则 panic
export function unwrap<T>(self: Option<T>) -> T {
    match self {
        None => panic("called `Option.unwrap()` on a `None` value"),
        Some(value) => value,
    }
}

/// unwrap_or 获取值，默认值提供默认
export function unwrap_or<T>(self: Option<T>, default: T) -> T {
    match self {
        None => default,
        Some(value) => value,
    }
}

/// map 将 Option 映射为另一种类型
export function map<T, U>(self: Option<T>, f: function(T) -> U) -> Option<U> {
    match self {
        None => None,
        Some(value) => Some(f(value)),
    }
}

/// and_then 链式调用返回 Option
export function and_then<T, U>(self: Option<T>, f: function(T) -> Option<U>) -> Option<U> {
    match self {
        None => None,
        Some(value) => f(value),
    }
}

/// Result 类型扩展方法
/// 判断 Result 是否为 Ok
export function is_ok<T, E>(self: Result<T, E>) -> Bool {
    match self {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// 判断 Result 是否为 Err
export function is_err<T, E>(self: Result<T, E>) -> Bool {
    match self {
        Ok(_) => false,
        Err(_) => true,
    }
}

/// unwrap 获取 Ok 值，如果是 Err 则 panic
export function unwrap<T, E>(self: Result<T, E>) -> T {
    match self {
        Ok(value) => value,
        Err(err) => panic("called `Result.unwrap()` on an `Err` value: " ++ to_string(err)),
    }
}

/// unwrap_err 获取 Err 值，如果是 Ok 则 panic
export function unwrap_err<T, E>(self: Result<T, E>) -> E {
    match self {
        Ok(_) => panic("called `Result.unwrap_err()` on an `Ok` value"),
        Err(err) => err,
    }
}

/// unwrap_or 获取 Ok 值，默认值提供默认
export function unwrap_or<T, E>(self: Result<T, E>, default: T) -> T {
    match self {
        Ok(value) => value,
        Err(_) => default,
    }
}

/// map 将 Result 映射为另一种类型
export function map<T, E, U>(self: Result<T, E>, f: function(T) -> U) -> Result<U, E> {
    match self {
        Ok(value) => Ok(f(value)),
        Err(err) => Err(err),
    }
}

/// map_err 将错误映射为另一种类型
export function map_err<T, E, F>(self: Result<T, E>, f: function(E) -> F) -> Result<T, F> {
    match self {
        Ok(value) => Ok(value),
        Err(err) => Err(f(err)),
    }
}

/// and_then 链式调用返回 Result
export function and_then<T, E, U>(self: Result<T, E>, f: function(T) -> Result<U, E>) -> Result<U, E> {
    match self {
        Ok(value) => f(value),
        Err(err) => Err(err),
    }
}

/// List (动态数组) 类型基础操作
export record List<T> {
    /// 内部存储
    data: [T],
}

/// 创建空列表
export function empty<T>() -> List<T> {
    List { data: [] }
}

/// 创建列表从数组
export function from_array<T>(data: [T]) -> List<T> {
    List { data: data }
}

/// 获取列表长度
export function len<T>(self: List<T>) -> Int {
    self.data.len()
}

/// 判断列表是否为空
export function is_empty<T>(self: List<T>) -> Bool {
    self.len() == 0
}

/// 获取指定索引处元素
export function get<T>(self: List<T>, index: Int) -> Option<T> {
    when index < 0 || index >= self.len() {
        None
    }
    Some(self.data[index])
}

/// 获取第一个元素
export function first<T>(self: List<T>) -> Option<T> {
    when self.is_empty() {
        None
    }
    Some(self.data[0])
}

/// 获取最后一个元素
export function last<T>(self: List<T>) -> Option<T> {
    when self.is_empty() {
        None
    }
    Some(self.data[self.len() - 1])
}

/// 推送元素到列表末尾
export function push<T>(self: &mut List<T>, value: T) -> Unit {
    self.data.push(value)
}

/// 弹出最后一个元素
export function pop<T>(self: &mut List<T>) -> Option<T> {
    self.data.pop()
}

/// 插入元素到指定位置
export function insert<T>(self: &mut List<T>, index: Int, value: T) -> Unit {
    self.data.insert(index, value)
}

/// 移除指定位置元素
export function remove<T>(self: &mut List<T>, index: Int) -> T {
    self.data.remove(index)
}

/// 替换指定位置元素
export function replace<T>(self: &mut List<T>, index: Int, value: T) -> T {
    let old = self.data[index];
    self.data[index] = value;
    old
}

/// 清空列表
export function clear<T>(self: &mut List<T>) -> Unit {
    self.data.clear()
}

/// 反转列表
export function reverse<T>(self: &mut List<T>) -> Unit {
    self.data.reverse()
}

/// 映射列表每个元素
export function map<T, U>(self: List<T>, f: function(T) -> U) -> List<U> {
    let mut result = empty();
    for element in self {
        result.push(f(element));
    }
    result
}

/// 过滤列表元素
export function filter<T>(self: List<T>, predicate: function(T) -> Bool) -> List<T> {
    let mut result = empty();
    for element in self {
        when predicate(element) {
            result.push(element);
        }
    }
    result
}

/// 折叠列表
export function fold<T, A>(self: List<T>, initial: A, f: function(A, T) -> A) -> A {
    let mut acc = initial;
    for element in self {
        acc = f(acc, element);
    }
    acc
}

/// Map (字典) 类型 - 键值对集合
export record Map<K, V> {
    /// 内部存储 - 简单实现基于列表存储键值对
    entries: List<(K, V)>,
}

/// 创建空字典
export function empty<K, V>() -> Map<K, V> {
    Map { entries: empty() }
}

/// 获取字典大小
export function len<K, V>(self: Map<K, V>) -> Int {
    self.entries.len()
}

/// 判断字典是否为空
export function is_empty<K, V>(self: Map<K, V>) -> Bool {
    self.len() == 0
}

/// 获取键对应的值
export function get<K: Eq, V>(self: Map<K, V>, key: K) -> Option<V> {
    for (k, v) in self.entries {
        when k == key {
            return Some(v);
        }
    }
    None
}

/// 插入键值对
export function insert<K, V>(self: &mut Map<K, V>, key: K, value: V) -> Option<V> {
    // 查找是否已存在
    for (index, (k, _)) in enumerate(self.entries) {
        when k == key {
            // 替换已有值
            let (_, old_value) = self.entries.remove(index);
            self.entries.insert(index, (key, value));
            return Some(old_value);
        }
    }
    // 不存在，插入新值
    self.entries.push((key, value));
    None
}

/// 移除键对应的值
export function remove<K: Eq, V>(self: &mut Map<K, V>, key: K) -> Option<V> {
    for (index, (k, v)) in enumerate(self.entries) {
        when k == key {
            return Some(self.entries.remove(index).1);
        }
    }
    None
}

/// 检查键是否存在
export function contains_key<K: Eq, V>(self: Map<K, V>, key: K) -> Bool {
    self.get(key).is_some()
}
