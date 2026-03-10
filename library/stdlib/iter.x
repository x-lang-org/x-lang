// X语言标准库 - 迭代器
//
// 迭代器抽象和操作

// ==========================================
// 迭代器类型
// ==========================================

/// 迭代器状态
type Iterator<T> = {
  next: () -> Option<T>,
  has_next: () -> Bool,
}

// ==========================================
// 迭代器创建
// ==========================================

/// 从列表创建迭代器
function iter_from_list<T>(list: [T]): Iterator<T> {
  let mut index = 0
  let len = list_len(list)
  {
    next: () -> {
      if index < len {
        let item = list_get(list, index)
        index = index + 1
        item
      } else {
        None()
      }
    },
    has_next: () -> index < len,
  }
}

/// 创建范围迭代器
function iter_range(start: Int, end: Int): Iterator<Int> {
  let mut current = start
  {
    next: () -> {
      if current < end {
        let value = current
        current = current + 1
        Some(value)
      } else {
        None()
      }
    },
    has_next: () -> current < end,
  }
}

/// 创建无限重复迭代器
function iter_repeat<T>(item: T): Iterator<T> {
  {
    next: () -> Some(item),
    has_next: () -> true,
  }
}

/// 创建重复 n 次的迭代器
function iter_repeat_n<T>(item: T, n: Int): Iterator<T> {
  let mut count = 0
  {
    next: () -> {
      if count < n {
        count = count + 1
        Some(item)
      } else {
        None()
      }
    },
    has_next: () -> count < n,
  }
}

// ==========================================
// 迭代器操作
// ==========================================

/// 映射迭代器
function iter_map<T, U>(iter: Iterator<T>, f: (T) -> U): Iterator<U> {
  {
    next: () -> {
      match iter.next() is
        Some { value: v } -> Some(f(v))
        None -> None()
    },
    has_next: () -> iter.has_next(),
  }
}

/// 过滤迭代器
function iter_filter<T>(iter: Iterator<T>, predicate: (T) -> Bool): Iterator<T> {
  {
    next: () -> {
      while iter.has_next() {
        match iter.next() is
          Some { value: v } ->
            if predicate(v) {
              return Some(v)
            }
          None -> break
      }
      None()
    },
    has_next: () -> {
      // 预先查找下一个满足条件的元素
      // 这里简化处理
      iter.has_next()
    },
  }
}

/// 过滤并映射迭代器
function iter_filter_map<T, U>(iter: Iterator<T>, f: (T) -> Option<U>): Iterator<U> {
  {
    next: () -> {
      while iter.has_next() {
        match iter.next() is
          Some { value: v } ->
            match f(v) is
              Some { value: u } -> return Some(u)
              None -> continue
          None -> break
      }
      None()
    },
    has_next: () -> iter.has_next(),
  }
}

/// 限制迭代器元素数量
function iter_take<T>(iter: Iterator<T>, n: Int): Iterator<T> {
  let mut count = 0
  {
    next: () -> {
      if count < n {
        count = count + 1
        iter.next()
      } else {
        None()
      }
    },
    has_next: () -> count < n && iter.has_next(),
  }
}

/// 跳过前 n 个元素
function iter_skip<T>(iter: Iterator<T>, n: Int): Iterator<T> {
  let mut skipped = 0
  while skipped < n && iter.has_next() {
    iter.next()
    skipped = skipped + 1
  }
  iter
}

/// 带索引的迭代器
function iter_enumerate<T>(iter: Iterator<T>): Iterator<(Int, T)> {
  let mut index = 0
  {
    next: () -> {
      match iter.next() is
        Some { value: v } ->
          let result = (index, v)
          index = index + 1
          Some(result)
        None -> None()
    },
    has_next: () -> iter.has_next(),
  }
}

// ==========================================
// 迭代器组合
// ==========================================

/// 链式连接两个迭代器
function iter_chain<T>(iter1: Iterator<T>, iter2: Iterator<T>): Iterator<T> {
  let mut first_done = false
  {
    next: () -> {
      if not first_done {
        match iter1.next() is
          Some { value: v } -> return Some(v)
          None -> first_done = true
      }
      iter2.next()
    },
    has_next: () -> (not first_done && iter1.has_next()) || iter2.has_next(),
  }
}

/// 交错两个迭代器
function iter_interleave<T>(iter1: Iterator<T>, iter2: Iterator<T>): Iterator<T> {
  let mut use_first = true
  {
    next: () -> {
      if use_first && iter1.has_next() {
        use_first = false
        iter1.next()
      } else if iter2.has_next() {
        use_first = true
        iter2.next()
      } else if iter1.has_next() {
        iter1.next()
      } else {
        None()
      }
    },
    has_next: () -> iter1.has_next() || iter2.has_next(),
  }
}

// ==========================================
// 迭代器消费
// ==========================================

/// 收集迭代器到列表
function iter_collect<T>(iter: Iterator<T>): [T] {
  let mut result = []
  while iter.has_next() {
    match iter.next() is
      Some { value: v } -> list_push(result, v)
      None -> break
  }
  result
}

/// 左折叠迭代器
function iter_fold<T, U>(iter: Iterator<T>, initial: U, f: (U, T) -> U): U {
  let mut accum = initial
  while iter.has_next() {
    match iter.next() is
      Some { value: v } -> accum = f(accum, v)
      None -> break
  }
  accum
}

/// 计算迭代器元素数量
function iter_count<T>(iter: Iterator<T>): Int {
  let mut count = 0
  while iter.has_next() {
    iter.next()
    count = count + 1
  }
  count
}

/// 查找第一个满足谓词的元素
function iter_find<T>(iter: Iterator<T>, predicate: (T) -> Bool): Option<T> {
  while iter.has_next() {
    match iter.next() is
      Some { value: v } ->
        if predicate(v) {
          return Some(v)
        }
      None -> break
  }
  None()
}

/// 检查是否所有元素都满足谓词
function iter_all<T>(iter: Iterator<T>, predicate: (T) -> Bool): Bool {
  while iter.has_next() {
    match iter.next() is
      Some { value: v } ->
        if not predicate(v) {
          return false
        }
      None -> break
  }
  true
}

/// 检查是否有元素满足谓词
function iter_any<T>(iter: Iterator<T>, predicate: (T) -> Bool): Bool {
  while iter.has_next() {
    match iter.next() is
      Some { value: v } ->
        if predicate(v) {
          return true
        }
      None -> break
  }
  false
}

/// 计算迭代器中整数的和
function iter_sum(iter: Iterator<Int>): Int {
  iter_fold(iter, 0, (acc, x) -> acc + x)
}

/// 计算迭代器中浮点数的和
function iter_sum_float(iter: Iterator<Float>): Float {
  iter_fold(iter, 0.0, (acc, x) -> acc + x)
}

/// 查找整数迭代器的最小值
function iter_min(iter: Iterator<Int>): Option<Int> {
  let mut result = None()
  while iter.has_next() {
    match iter.next() is
      Some { value: v } ->
        result = match result is
          Some { value: current } -> Some(min_int(current, v))
          None -> Some(v)
      None -> break
  }
  result
}

/// 查找整数迭代器的最大值
function iter_max(iter: Iterator<Int>): Option<Int> {
  let mut result = None()
  while iter.has_next() {
    match iter.next() is
      Some { value: v } ->
        result = match result is
          Some { value: current } -> Some(max_int(current, v))
          None -> Some(v)
      None -> break
  }
  result
}

/// 执行迭代器的每个元素
function iter_for_each<T>(iter: Iterator<T>, f: (T) -> Unit) {
  while iter.has_next() {
    match iter.next() is
      Some { value: v } -> f(v)
      None -> break
  }
}

// ==========================================
// 高级迭代器
// ==========================================

/// 按给定大小分块
function iter_chunks<T>(iter: Iterator<T>, size: Int): Iterator<[T]> {
  if size <= 0 {
    panic("iter_chunks: 块大小必须为正")
  }
  {
    next: () -> {
      let mut chunk = []
      let mut count = 0
      while count < size && iter.has_next() {
        match iter.next() is
          Some { value: v } ->
            list_push(chunk, v)
            count = count + 1
          None -> break
      }
      if list_is_empty(chunk) {
        None()
      } else {
        Some(chunk)
      }
    },
    has_next: () -> iter.has_next(),
  }
}

/// 滑动窗口
function iter_windows<T>(iter: Iterator<T>, size: Int): Iterator<[T]> {
  if size <= 0 {
    panic("iter_windows: 窗口大小必须为正")
  }
  // 预填充第一个窗口
  let mut window = []
  let mut i = 0
  while i < size && iter.has_next() {
    match iter.next() is
      Some { value: v } -> list_push(window, v)
      None -> break
    i = i + 1
  }
  let mut started = false
  {
    next: () -> {
      if not started {
        started = true
        if list_len(window) == size {
          Some(window)
        } else {
          None()
        }
      } else if iter.has_next() {
        // 滑动窗口
        let (_, new_window) = list_remove(window, 0)
        window = new_window
        match iter.next() is
          Some { value: v } ->
            list_push(window, v)
            Some(window)
          None -> None()
      } else {
        None()
      }
    },
    has_next: () -> not started || (list_len(window) == size && iter.has_next()),
  }
}

/// 扫描迭代器（保持中间状态）
function iter_scan<T, U>(iter: Iterator<T>, initial: U, f: (U, T) -> U): Iterator<U> {
  let mut state = initial
  {
    next: () -> {
      match iter.next() is
        Some { value: v } ->
          state = f(state, v)
          Some(state)
        None -> None()
    },
    has_next: () -> iter.has_next(),
  }
}

/// 逐个元素应用函数直到返回 None
function iter_take_while<T>(iter: Iterator<T>, predicate: (T) -> Bool): Iterator<T> {
  let mut done = false
  {
    next: () -> {
      if done {
        return None()
      }
      match iter.next() is
        Some { value: v } ->
          if predicate(v) {
            Some(v)
          } else {
            done = true
            None()
          }
        None -> None()
    },
    has_next: () -> not done && iter.has_next(),
  }
}

/// 跳过元素直到谓词返回 false
function iter_skip_while<T>(iter: Iterator<T>, predicate: (T) -> Bool): Iterator<T> {
  let mut skipping = true
  while skipping && iter.has_next() {
    match iter.next() is
      Some { value: v } ->
        if not predicate(v) {
          // 需要把这个元素放回去
          // 简化实现：创建一个新的迭代器
          let mut buffer = [v]
          return {
            next: () -> {
              if not list_is_empty(buffer) {
                let (item, rest) = list_remove(buffer, 0)
                buffer = rest
                item
              } else {
                iter.next()
              }
            },
            has_next: () -> not list_is_empty(buffer) || iter.has_next(),
          }
        }
      None -> break
  }
  iter
}

// ==========================================
// 配对迭代器
// ==========================================

/// 拉链两个迭代器
function iter_zip<T, U>(iter1: Iterator<T>, iter2: Iterator<U>): Iterator<(T, U)> {
  {
    next: () -> {
      match (iter1.next(), iter2.next()) is
        (Some { value: v1 }, Some { value: v2 }) -> Some((v1, v2))
        _ -> None()
    },
    has_next: () -> iter1.has_next() && iter2.has_next(),
  }
}

/// 带索引的迭代（简化版）
function iter_with_index<T>(iter: Iterator<T>): Iterator<(Int, T)> {
  iter_enumerate(iter)
}