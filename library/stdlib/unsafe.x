module std.unsafe

import std.prelude
import std.types


// === 外部 C 库函数绑定 ===

/// 内存分配
external "c" function malloc(size: usize) -> *unit

/// 内存释放
external "c" function free(ptr: *unit) -> unit

/// 内存分配，清零
external "c" function calloc(nmemb: usize, size: usize) -> *unit

/// 重新分配内存
external "c" function realloc(ptr: *unit, size: usize) -> *unit

/// 内存拷贝
external "c" function memcpy(dest: *unit, src: *const unit, n: usize) -> *unit

/// 内存移动
external "c" function memmove(dest: *unit, src: *const unit, n: usize) -> *unit

/// memset
external "c" function memset(ptr: *unit, c: signed 32-bit integer, n: usize) -> *unit

/// 原子比较交换
/// 如果 *ptr == old_val，则设置 *ptr = new_val
/// 返回原来的 *ptr
external "c" function atomic_compare_exchange_strong(ptr: *Int, old_val: Int, new_val: Int) -> Int

/// 原子加法，返回新值
external "c" function atomic_fetch_add(ptr: *Int, val: Int) -> Int

/// 原子减法，返回新值
external "c" function atomic_fetch_sub(ptr: *Int, val: Int) -> Int

/// Unsafe 遮罩：在不安全块中执行代码
export macro unsafe(block: Expr) -> Nothing {
    unsafe {
        block
    }
}
