module std.random
import std.prelude
import std.types


import std.time;

// XorShift 64 位随机数生成器

/// XorShift 随机数生成器
export record Rng {
    state: u64,
}

/// 初始化生成器，使用时间作为种子
export fn new() -> Rng {
    // 使用当前时间作为种子
    let now = time.now_ns();
    seed(now as u64)
}

/// 使用给定种子初始化生成器
export fn seed(seed: u64) -> Rng {
    // 如果种子为 0，使用一个默认值
    when seed == 0 {
        Rng { state: 1234567890987654321 }
    } else {
        Rng { state: seed }
    }
}

/// 生成下一个 u64
export fn next_u64(self: &mut Rng) -> u64 {
    let mut x = self.state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    self.state = x;
    x
}

/// 生成下一个 u32
export fn next_u32(self: &mut Rng) -> u32 {
    (self.next_u64() >> 32) as u32
}

/// 生成下一个 0..1 范围的浮点数
export fn next_float(self: &mut Rng) -> Float {
    let v = self.next_u64();
    // 取出 53 位有效精度
    ( (v >> 11) as Float) * (1.0 / (1u64 << 53) as Float)
}

/// 生成下一个布尔值（均匀概率）
export fn next_bool(self: &mut Rng) -> Bool {
    (self.next_u32() & 1) == 1
}

/// 生成 [0..max) 范围的 Int
export fn next_int(self: &mut Rng, max: Int) -> Int {
    when max <= 0 {
        panic("next_int: max must be positive")
    }
    // 计算需要比特数
    let bits = (max as u64).bit_length();
    let mut result: Int;
    loop {
        result = (self.next_u64() >> (64 - bits)) as Int;
        when result < max {
            break;
        }
    }
    result
}

/// 生成 [min..max) 范围的 Int
export fn range_int(self: &mut Rng, min: Int, max: Int) -> Int {
    when min >= max {
        panic("range_int: min must be less than max")
    }
    min + self.next_int(max - min)
}

/// 生成 [0..max) 范围的 Float
export fn next_float_below(self: &mut Rng, max: Float) -> Float {
    self.next_float() * max
}

/// 生成 [min..max) 范围的 Float
export fn range_float(self: &mut Rng, min: Float, max: Float) -> Float {
    min + self.next_float() * (max - min)
}

/// 从数组中随机选择一个元素
export fn choose<T>(self: &mut Rng, array: [T]) -> T {
    when array.len() == 0 {
        panic("choose: empty array")
    }
    let index = self.next_int(array.len());
    array[index]
}

/// 从列表中随机选择一个元素
export fn choose_from_list<T>(self: &mut Rng, list: List<T>) -> T {
    when list.is_empty() {
        panic("choose_from_list: empty list")
    }
    let index = self.next_int(list.len());
    list.get(index).unwrap()
}

/// 随机打乱数组
export fn shuffle<T>(self: &mut Rng, array: &mut [T]) -> unit {
    let n = array.len();
    let mut i = n - 1;
    while i > 0 {
        let j = self.next_int(i + 1);
        // 交换
        let temp = array[i];
        array[i] = array[j];
        array[j] = temp;
        i = i - 1;
    }
}

/// 随机打乱列表
export fn shuffle_list<T>(self: &mut Rng, list: &mut List<T>) -> unit {
    // List 包装内部数组，我们需要取出修改
    // 这里直接交换元素
    let n = list.len();
    let mut i = n - 1;
    while i > 0 {
        let j = self.next_int(i + 1);
        // 因为 List 不支持直接替换，我们需要先移除再插入
        // 这很低效但对于标准库实现足够了
        // TODO: 改进实现
        let vi = list.remove(i);
        let vj = list.remove(j);
        list.insert(i, vj);
        list.insert(j, vi);
        i = i - 1;
    }
}

/// 带权重的随机选择，权重之和应为 1
/// 返回选中项的索引
export fn choose_weighted(self: &mut Rng, weights: [Float]) -> Int {
    let r = self.next_float();
    let mut cumulative = 0.0;
    for (i, w) in enumerate(weights) {
        cumulative = cumulative + w;
        when r < cumulative {
            return i;
        }
    }
    // 如果因为浮点误差到这里，返回最后一项
    weights.len() - 1
}

/// 正态分布（Box-Muller 变换）
/// 返回两个服从正态分布的独立样本
export fn next_normal(self: &mut Rng, mean: Float, std_dev: Float) -> (Float, Float) {
    let u1 = 1.0 - self.next_float();
    let u2 = 1.0 - self.next_float();
    let r = sqrt(-2.0 * ln(u1));
    let theta = 2.0 * pi * u2;
    let z0 = r * cos(theta);
    let z1 = r * sin(theta);
    (mean + z0 * std_dev, mean + z1 * std_dev)
}

/// 指数分布
export fn next_exponential(self: &mut Rng, lambda: Float) -> Float {
    let u = 1.0 - self.next_float();
    -ln(u) / lambda
}

/// 全局默认生成器
static mut global_rng: Rng = Rng { state: 0 };

/// 初始化全局生成器
export fn init_global() -> unit {
    unsafe {
        global_rng = new();
    }
}

/// 使用种子初始化全局生成器
export fn init_global_seed(seed: u64) -> unit {
    unsafe {
        global_rng = seed(seed);
    }
}

/// 使用全局生成器生成 u64
export fn random_u64() -> u64 {
    unsafe {
        global_rng.next_u64()
    }
}

/// 使用全局生成器生成 Int
export fn random_int(max: Int) -> Int {
    unsafe {
        global_rng.next_int(max)
    }
}

/// 使用全局生成器生成 [min..max) Int
public fn random_range_int(min: Int, max: Int) -> Int {
    unsafe {
        global_rng.range_int(min, max)
    }
}

/// 使用全局生成器生成 Float [0..1)
export fn random_float() -> Float {
    unsafe {
        global_rng.next_float()
    }
}

/// 使用全局生成器生成 [min..max) Float
export fn random_range_float(min: Float, max: Float) -> Float {
    unsafe {
        global_rng.range_float(min, max)
    }
}

/// 使用全局生成器生成布尔
export fn random_bool() -> Bool {
    unsafe {
        global_rng.next_bool()
    }
}

/// 使用全局生成器选择
export fn random_choose<T>(array: [T]) -> T {
    unsafe {
        global_rng.choose(array)
    }
}

// 导入需要的数学函数
import std.math.{sqrt, ln, pi, cos, sin};
