// X 语言 ByteArray 标准库 - 简化版本

module std.bytes

import std.prelude;
import std.types;

/// 字节数组类型
export type ByteArray = [integer];

/// 创建指定大小的字节数组
export fn create(size: integer) -> ByteArray {
    let mut arr: ByteArray = [];
    let mut i = 0;
    while i < size {
        arr.push(0);
        i = i + 1;
    }
    arr
}

/// 获取字节数组长度
export fn length(self: ByteArray) -> integer {
    self.length()
}

/// 获取指定索引的字节
export fn get(self: ByteArray, index: integer) -> integer {
    self[index]
}

/// 设置指定索引的字节
export fn set(self: ByteArray, index: integer, value: integer) -> unit {
    self[index] = value
}
