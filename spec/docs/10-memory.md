# 第10章 内存管理

X 采用与 Koka 相同的 **Perceus** 算法实现内存管理——编译时精确插入引用计数操作，结合重用分析实现零开销的函数式编程。无垃圾回收器，无手动内存管理，无生命周期标注。

## 10.1 Perceus 内存管理

### 核心思想

```
Perceus: Compile-time precise reference counting + reuse analysis

Properties:
  - No garbage collector (no stop-the-world pauses)
  - No manual memory management (no malloc/free)
  - No lifetime annotations (unlike Rust's borrow checker)
  - Deterministic deallocation (resource release is predictable)
  - Thread-safe (atomic RC when shared across threads)
```

Perceus 的核心观察：编译器能精确知道每个值在何处被最后一次使用，因此可以在编译期自动插入引用计数操作，而非依赖运行时 GC 或开发者手动管理。

### 与其他方案的对比

| 方案 | 运行时开销 | 开发者负担 | 确定性释放 | 安全性 |
|------|-----------|-----------|-----------|--------|
| **Perceus (X)** | 极低 | 无 | ✓ | 编译期保证 |
| GC (Java/Go) | STW 停顿 | 无 | ✗ | 运行时保证 |
| 借用检查 (Rust) | 零 | 高（生命周期标注） | ✓ | 编译期保证 |
| 手动管理 (C) | 零 | 极高 | ✓ | 无保证 |
| ARC (Swift) | 低 | 低 | ✓ | 运行时保证 |

---

## 10.2 精确 dup/drop 插入

编译器在编译期分析值的使用模式，精确插入 `dup`（增加引用计数）和 `drop`（减少引用计数并可能释放）操作。

### Dup 操作

```
dup : ∀T. T → T
dup(v) = v'
  where refcount(v) += 1
        v' shares the same underlying data as v
```

当一个值需要在多个位置使用时，编译器在第二次（及后续）使用前插入 `dup`。

### Drop 操作

```
drop : ∀T. T → ()
drop(v) =
  refcount(v) -= 1
  if refcount(v) == 0 then
    for each child c of v:
      drop(c)
    deallocate(v)
```

当一个值不再被使用时，编译器在该点插入 `drop`。

### 插入规则

```
对于每个变量 x 在作用域 S 中：

  设 uses(x, S) = {u₁, u₂, ..., uₙ}  (x 的所有使用点，按执行顺序)

  若 n = 0:
    insert drop(x) at the beginning of S

  若 n = 1:
    x is consumed at u₁ (no dup needed, drop after use if not returned)

  若 n > 1:
    insert dup(x) before u₁, u₂, ..., uₙ₋₁
    uₙ consumes the original reference
```

### 示例

```x
function example() {
    let xs = [1, 2, 3]
    let a = length(xs)       // 编译器插入: dup(xs); length(xs)
    let b = sum(xs)          // 最后一次使用，消耗 xs
    a + b
}
// 编译器转换后（伪代码）：
// let xs = [1, 2, 3]
// dup(xs)
// let a = length(xs)
// let b = sum(xs)        ← 消耗最后一份引用
// let result = a + b
// drop(a); drop(b)       ← 标量类型可优化掉
// result
```

---

## 10.3 重用分析 (Reuse Analysis)

重用分析是 Perceus 的关键优化：当编译器检测到一个值的引用计数为 1（唯一引用）时，可以将 "释放旧值 + 分配新值" 优化为 "原地更新"，实现零分配。

### 形式化定义

```
ReuseOpportunity = (DropSite, AllocationSite)
  where typeof(dropped) ≈ typeof(allocated)
        size(dropped) ≥ size(allocated)
        no aliases exist to dropped

reuse(v, fields...) =
  if refcount(v) == 1 then
    update_in_place(v, fields...)      // zero allocation
  else
    drop(v)
    allocate_new(fields...)            // normal allocation
```

### 重用转换

```
源代码模式:
  drop(x)
  let y = Constructor(v₁, ..., vₙ)

转换为（若 x 与 y 类型兼容）:
  if is_unique(x) then
    let y = reuse(x, v₁, ..., vₙ)     // 原地更新
  else
    decrement(x)
    let y = Constructor(v₁, ..., vₙ)   // 新分配
```

### 示例：函数式列表操作

```x
function map(list: List<Integer>, f: (Integer) -> Integer) -> List<Integer> {
    match list {
        Nil          => Nil
        Cons(x, xs)  => Cons(f(x), map(xs, f))
    }
}
```

当 `list` 为唯一引用时，`Cons(f(x), map(xs, f))` 可重用原 `Cons` 节点的内存，不需要新分配。这使得函数式风格的链表操作与命令式原地修改具有相同的性能。

---

## 10.4 FBIP (Functional But In-Place)

FBIP 是 Perceus 使函数式编程达到命令式性能的核心范式：编写纯函数式代码，编译器自动优化为原地更新。

### 原理

```
FBIP 保证:
  若函数 f 是某数据结构的唯一消费者，
  则 f 中的"构造新值"操作自动变为"原地修改"。

  Formally:
    f(x) where refcount(x) == 1
    ⟹ all allocations in f that match the shape of x
       are rewritten to in-place mutations
```

### 经典示例：红黑树插入

```x
function insert(tree: Tree<Integer>, value: Integer) -> Tree<Integer> {
    match tree {
        Leaf => Node(Red, Leaf, value, Leaf)
        Node(color, left, v, right) => {
            if value < v {
                balance(color, insert(left, value), v, right)
            } else if value > v {
                balance(color, left, v, insert(right, value))
            } else {
                tree
            }
        }
    }
}
```

此函数是纯函数式的，但当 `tree` 为唯一引用时，`Node(...)` 的构造自动重用被解构的节点内存，性能等同于命令式的原地插入。

---

## 10.5 特化 (Specialization)

编译器对引用计数为 1 的值（唯一引用）生成特化代码路径，跳过引用计数检查。

### 特化规则

```
对于函数 f(x: T):
  若编译器能证明 refcount(x) == 1 在调用处恒成立，
  则生成特化版本 f_unique(x: T)，其中：
    - 所有 dup(x) 被消除
    - 所有 drop(x) + alloc 被替换为 reuse
    - 引用计数检查被消除
```

### 特化策略

```
specialize(f, x) =
  let f_unique = clone(f)
  in f_unique:
    replace all dup(x) with nop
    replace all (drop(x); alloc(T, ...)) with reuse(x, ...)
    remove all refcount checks on x
  emit:
    if is_unique(x) then f_unique(x) else f(x)
```

在实践中，唯一引用是常见情况（局部变量、函数返回值等），因此特化能显著提升性能。

---

## 10.6 循环引用处理

Perceus 的引用计数无法自动回收循环引用。X 提供 `weak` 引用来打破循环。

### weak 引用

```
weak T : 弱引用类型
  不增加引用计数
  不阻止被引用对象的释放
  访问前必须升级为 Option<T>
```

### 使用方式

```x
class Node {
    let value: Integer
    let children: List<Node>
    let parent: weak Node       // 弱引用打破循环
}

function accessParent(node: Node) -> Option<Node> {
    upgrade(node.parent)        // 返回 Some(parent) 或 None
}
```

### weak 引用规则

```
weak(v):
  does not call dup(v)
  refcount(v) is unchanged

upgrade(w: weak T) -> Option<T>:
  if target is alive then
    dup(target)
    Some(target)
  else
    None

drop(w: weak T):
  removes weak reference record
  does not affect target's refcount
```

### 设计指导

- 树形结构中的父指针使用 `weak`
- 缓存、观察者模式中的反向引用使用 `weak`
- 尽量设计单向数据流的数据结构以避免循环

---

## 10.7 安全保证

Perceus 在编译期提供以下内存安全保证：

### 定理 1：无释放后使用 (No Use-After-Free)

```
Theorem (No Use-After-Free):
  ∀ well-typed program P,
  ∀ variable x in P,
  if x is accessed at point p, then refcount(x) > 0 at p.

Proof sketch:
  The compiler inserts dup before each use except the last,
  and drop only after the last use. Therefore no access
  occurs after the reference count reaches zero.
```

### 定理 2：无重复释放 (No Double-Free)

```
Theorem (No Double-Free):
  ∀ well-typed program P,
  ∀ value v in P,
  drop(v) reducing refcount to 0 occurs at most once.

Proof sketch:
  Each dup increments by exactly 1, each drop decrements by exactly 1.
  The total number of drops = 1 (initial) + number of dups.
  Only the final drop triggers deallocation.
```

### 定理 3：无内存泄漏 (No Memory Leaks)

```
Theorem (No Memory Leaks — acyclic):
  ∀ well-typed program P with no cyclic references,
  ∀ allocated value v in P,
  v is eventually deallocated.

Proof sketch:
  Every owned value has exactly one drop inserted at end of scope.
  For acyclic data, the refcount will reach 0 and trigger deallocation.

Note: Cyclic references require explicit use of `weak` to break cycles.
      The compiler may emit warnings for potential cycles.
```

### 安全保证总结

| 保证 | 机制 |
|------|------|
| 无释放后使用 | 编译期 dup/drop 精确插入 |
| 无重复释放 | 引用计数严格配对 |
| 无内存泄漏（无环） | 每个值恰好一个最终 drop |
| 循环引用安全 | `weak` 引用打破循环 |
| 线程安全 | 跨线程共享时使用原子引用计数 |

---

**本章定义了 X 语言的 Perceus 内存管理系统。编译器在编译期精确插入 dup/drop 操作，通过重用分析和 FBIP 实现函数式代码的零分配优化，通过特化进一步消除运行时开销。`weak` 引用用于处理循环引用。整个系统在编译期保证无释放后使用、无重复释放和无内存泄漏（无环情况下）。**
