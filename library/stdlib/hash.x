module std.hash
import std.prelude
import std.types



/// 哈希 trait
/// 类型可以实现这个来提供自定义哈希
export trait Hash<T> {
    /// 计算值的哈希
    fn hash(self: &T) -> u64;
}

/// 默认哈希 - 使用 FNV-1a
export fn default_hash(bytes: [u8]) -> u64 {
    fnv1a(bytes)
}

/// FNV-1a 哈希参数
const FNV_OFFSET: u64 = 14695981039346313089;
const FNV_PRIME: u64 = 1099511628211;

/// FNV-1a 哈希算法
/// 快速非加密哈希，适合哈希表使用
export fn fnv1a(bytes: [u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash = hash ^ (b as u64);
        hash = hash * FNV_PRIME;
    }
    hash
}

/// FNV-1 哈希算法
export fn fnv1(bytes: [u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash = hash * FNV_PRIME;
        hash = hash ^ (b as u64);
    }
    hash
}

/// 计算字符串的 FNV-1a 哈希
export fn fnv1a_string(s: string) -> u64 {
    let mut hash = FNV_OFFSET;
    for c in s {
        hash = hash ^ (c as u64);
        hash = hash * FNV_PRIME;
    }
    hash
}

/// DJB2 哈希算法
export fn djb2(bytes: [u8]) -> u32 {
    let mut hash: u32 = 5381;
    for b in bytes {
        hash = ((hash << 5) + hash) ^ (b as u32);
    }
    hash
}

/// SDBM 哈希算法
export fn sdbm(bytes: [u8]) -> u32 {
    let mut hash: u32 = 0;
    for b in bytes {
        hash = (b as u32) + (hash << 6) + (hash << 16) - hash;
    }
    hash
}

/// Jenkins 一次哈希
fn jenkins_mix(a: u32, b: u32, c: u32) -> (u32, u32, u32) {
    let mut a = a;
    let mut b = b;
    let mut c = c;
    a = a - b; a = a - c; a = a ^ (c >> 13);
    b = b - c; b = b - a; b = b ^ (a << 8);
    c = c - a; c = c - b; c = c ^ (b >> 13);
    a = a - b; a = a - c; a = a ^ (c >> 12);
    b = b - c; b = b - a; b = b ^ (a << 16);
    c = c - a; c = c - b; c = c ^ (b >> 5);
    a = a - b; a = a - c; a = a ^ (c >> 3);
    b = b - c; b = b - a; b = b ^ (a << 10);
    c = c - a; c = c - b; c = c ^ (b >> 15);
    (a, b, c)
}

/// Jenkins one-at-a-time 哈希
export fn jenkins_one_at_a_time(bytes: [u8]) -> u32 {
    let mut hash: u32 = 0;
    for b in bytes {
        hash = hash + (b as u32);
        hash = hash + (hash << 10);
        hash = hash ^ (hash >> 6);
    }
    hash = hash + (hash << 3);
    hash = hash ^ (hash >> 11);
    hash = hash + (hash << 15);
    hash
}

/// MurmurHash3 32 位（简化实现）
export fn murmurhash3_32(bytes: [u8], seed: u32) -> u32 {
    let c1: u32 = 0xcc9e2d51;
    let c2: u32 = 0x1b873593;
    let mut h1 = seed;
    let n_blocks = (bytes.len() / 4) * 4;
    let mut i = 0;

    while i < n_blocks {
        let mut k1: u32 =
            (bytes[i] as u32) |
            ((bytes[i + 1] as u32) << 8) |
            ((bytes[i + 2] as u32) << 16) |
            ((bytes[i + 3] as u32) << 24);
        k1 = k1 * c1;
        k1 = (k1 << 15) | (k1 >> 17);
        k1 = k1 * c2;
        h1 = h1 ^ k1;
        h1 = (h1 << 13) | (h1 >> 19);
        h1 = h1 * 5 + 0xe6546b64;
        i = i + 4;
    }

    // 处理尾部
    let remaining = bytes.len() - i;
    when remaining > 0 {
        let mut k1: u32 = 0;
        match remaining {
            3 => {
                k1 = k1 ^ ((bytes[i + 2] as u32) << 16);
                k1 = k1 ^ ((bytes[i + 1] as u32) << 8);
                k1 = k1 ^ (bytes[i] as u32);
                k1 = k1 * c1;
                k1 = (k1 << 15) | (k1 >> 17);
                k1 = k1 * c2;
                h1 = h1 ^ k1;
            }
            2 => {
                k1 = k1 ^ ((bytes[i + 1] as u32) << 8);
                k1 = k1 ^ (bytes[i] as u32);
                k1 = k1 * c1;
                k1 = (k1 << 15) | (k1 >> 17);
                k1 = k1 * c2;
                h1 = h1 ^ k1;
            }
            1 => {
                k1 = k1 ^ (bytes[i] as u32);
                k1 = k1 * c1;
                k1 = (k1 << 15) | (k1 >> 17);
                k1 = k1 * c2;
                h1 = h1 ^ k1;
            }
        }
    }

    // 最终混合
    h1 = h1 ^ (bytes.len() as u32);
    h1 = h1 ^ (h1 >> 16);
    h1 = h1 * 0x85ebca6b;
    h1 = h1 ^ (h1 >> 13);
    h1 = h1 * 0xc2b2ae35;
    h1 = h1 ^ (h1 >> 16);
    h1
}

/// 简化的 MurmurHash3，使用默认种子
export fn murmurhash3_32_default(bytes: [u8]) -> u32 {
    murmurhash3_32(bytes, 0);
}

/// 计算哈希组合值
/// 用于组合多个字段的哈希
export fn combine(seed: u64, hash: u64) -> u64 {
    seed ^ (hash + 0x9e3779b97f4a7c15 + (seed << 12) + (seed >> 12))
}

/// 默认哈希实现 for Int
export fn hash_int(x: Int) -> u64 {
    // 简单转换，FNV 处理单个值
    let mut hash = FNV_OFFSET;
    // 把 Int 分成字节并哈希
    let mut v = x as u64;
    let mut i = 0;
    while i < 8 {
        let byte = (v & 0xFF) as u8;
        hash = hash ^ (byte as u64);
        hash = hash * FNV_PRIME;
        v = v >> 8;
        i = i + 1;
    }
    hash
}

/// 默认哈希实现 for Float
export fn hash_float(x: Float) -> u64 {
    // 将浮点数按位转换
    (x as u64)
}

/// 默认哈希实现 for Bool
export fn hash_bool(b: Bool) -> u64 {
    when b { 12345 } else { 67890 }
}

/// 默认哈希实现 for string
export fn hash_string(s: string) -> u64 {
    fnv1a_string(s)
}

/// 哈希表使用的哈希函数选择
/// 默认使用 FNV-1a，因为它快且分布良好
export fn hash_for_table<T>(value: &T) -> u64 where T: Hash {
    value.hash()
}
