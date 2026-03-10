// X语言标准库 - 集合类型
//
// 列表(List)、映射(Map)、集合(Set)等集合操作

// ==========================================
// 列表 (List) 操作
// ==========================================

/// 创建一个新的空列表
function list_new<T>(): [T] {
  []
}

/// 创建一个包含单个元素的列表
function list_of<T>(item: T): [T] {
  [item]
}

/// 创建一个包含重复元素的列表
function list_repeat<T>(item: T, count: Int): [T] {
  let mut result = []
  let mut i = 0
  while i < count {
    list_push(result, item)
    i = i + 1
  }
  result
}

/// 获取列表长度
function list_len<T>(list: [T]): Int {
  let mut length = 0
  for item in list {
    length = length + 1
  }
  length
}

/// 检查列表是否为空
function list_is_empty<T>(list: [T]): Bool {
  list_len(list) == 0
}

// ==========================================
// 列表元素访问
// ==========================================

/// 获取指定位置的元素
function list_get<T>(list: [T], index: Int): Option<T> {
  if index >= 0 && index < list_len(list) {
    // 内置访问
    Some(list[index])
  } else {
    None()
  }
}

/// 获取第一个元素
function list_first<T>(list: [T]): Option<T> {
  list_get(list, 0)
}

/// 获取最后一个元素
function list_last<T>(list: [T]): Option<T> {
  let len = list_len(list)
  if len > 0 {
    list_get(list, len - 1)
  } else {
    None()
  }
}

// ==========================================
// 列表修改操作
// ==========================================

/// 在列表末尾添加元素
function list_push<T>(list: [T], item: T): [T] {
  // 简单实现，实际需要底层支持
  list + [item]
}

/// 移除并返回列表末尾的元素
function list_pop<T>(list: [T]): (Option<T>, [T]) {
  let len = list_len(list)
  if len == 0 {
    (None(), list)
  } else {
    let item = list_get(list, len - 1)
    let mut new_list = []
    let mut i = 0
    while i < len - 1 {
      list_push(new_list, list_get(list, i))
      i = i + 1
    }
    (item, new_list)
  }
}

/// 在指定位置插入元素
function list_insert<T>(list: [T], index: Int, item: T): [T] {
  let len = list_len(list)
  if index < 0 || index > len {
    panic("list_insert: 索引越界")
  }
  let mut result = []
  let mut i = 0
  while i < index {
    list_push(result, list_get(list, i))
    i = i + 1
  }
  list_push(result, item)
  while i < len {
    list_push(result, list_get(list, i))
    i = i + 1
  }
  result
}

/// 移除指定位置的元素
function list_remove<T>(list: [T], index: Int): (Option<T>, [T]) {
  let len = list_len(list)
  if index < 0 || index >= len {
    (None(), list)
  } else {
    let item = list_get(list, index)
    let mut result = []
    let mut i = 0
    while i < index {
      list_push(result, list_get(list, i))
      i = i + 1
    }
    i = i + 1
    while i < len {
      list_push(result, list_get(list, i))
      i = i + 1
    }
    (item, result)
  }
}

/// 清空列表
function list_clear<T>(list: [T]): [T] {
  []
}

// ==========================================
// 列表连接和分割
// ==========================================

/// 连接两个列表
function list_append<T>(list1: [T], list2: [T]): [T] {
  let mut result = []
  let mut i = 0
  while i < list_len(list1) {
    list_push(result, list_get(list1, i))
    i = i + 1
  }
  i = 0
  while i < list_len(list2) {
    list_push(result, list_get(list2, i))
    i = i + 1
  }
  result
}

/// 连接多个列表
function list_concat<T>(lists: [[T]]): [T] {
  let mut result = []
  let mut i = 0
  while i < list_len(lists) {
    result = list_append(result, list_get(lists, i))
    i = i + 1
  }
  result
}

/// 分割列表为两部分
function list_split_at<T>(list: [T], index: Int): ([T], [T]) {
  let len = list_len(list)
  let split_index = clamp_int(index, 0, len)
  let mut left = []
  let mut right = []
  let mut i = 0
  while i < split_index {
    list_push(left, list_get(list, i))
    i = i + 1
  }
  while i < len {
    list_push(right, list_get(list, i))
    i = i + 1
  }
  (left, right)
}

// ==========================================
// 列表变换
// ==========================================

/// 对列表中的每个元素应用函数
function list_map<T, U>(list: [T], f: (T) -> U): [U] {
  let mut result = []
  let mut i = 0
  while i < list_len(list) {
    list_push(result, f(list_get(list, i)))
    i = i + 1
  }
  result
}

/// 过滤列表，只保留满足谓词的元素
function list_filter<T>(list: [T], predicate: (T) -> Bool): [T] {
  let mut result = []
  let mut i = 0
  while i < list_len(list) {
    let item = list_get(list, i)
    if predicate(item) {
      list_push(result, item)
    }
    i = i + 1
  }
  result
}

/// 过滤并映射列表
function list_filter_map<T, U>(list: [T], f: (T) -> Option<U>): [U] {
  let mut result = []
  let mut i = 0
  while i < list_len(list) {
    match f(list_get(list, i)) is
      Some { value } -> list_push(result, value)
      None -> {}
    i = i + 1
  }
  result
}

/// 左折叠（从左到右累积）
function list_fold<T, U>(list: [T], initial: U, f: (U, T) -> U): U {
  let mut accum = initial
  let mut i = 0
  while i < list_len(list) {
    accum = f(accum, list_get(list, i))
    i = i + 1
  }
  accum
}

/// 右折叠（从右到左累积）
function list_fold_right<T, U>(list: [T], initial: U, f: (T, U) -> U): U {
  let mut accum = initial
  let mut i = list_len(list) - 1
  while i >= 0 {
    accum = f(list_get(list, i), accum)
    i = i - 1
  }
  accum
}

// ==========================================
// 列表搜索
// ==========================================

/// 检查列表是否包含指定元素
function list_contains<T>(list: [T], item: T): Bool {
  let mut i = 0
  while i < list_len(list) {
    if list_get(list, i) == item {
      return true
    }
    i = i + 1
  }
  false
}

/// 查找第一个满足谓词的元素
function list_find<T>(list: [T], predicate: (T) -> Bool): Option<T> {
  let mut i = 0
  while i < list_len(list) {
    let item = list_get(list, i)
    if predicate(item) {
      return Some(item)
    }
    i = i + 1
  }
  None()
}

/// 查找第一个满足谓词的元素的索引
function list_position<T>(list: [T], predicate: (T) -> Bool): Option<Int> {
  let mut i = 0
  while i < list_len(list) {
    if predicate(list_get(list, i)) {
      return Some(i)
    }
    i = i + 1
  }
  None()
}

/// 检查是否所有元素都满足谓词
function list_all<T>(list: [T], predicate: (T) -> Bool): Bool {
  let mut i = 0
  while i < list_len(list) {
    if not predicate(list_get(list, i)) {
      return false
    }
    i = i + 1
  }
  true
}

/// 检查是否有元素满足谓词
function list_any<T>(list: [T], predicate: (T) -> Bool): Bool {
  let mut i = 0
  while i < list_len(list) {
    if predicate(list_get(list, i)) {
      return true
    }
    i = i + 1
  }
  false
}

/// 统计满足谓词的元素数量
function list_count<T>(list: [T], predicate: (T) -> Bool): Int {
  let mut count = 0
  let mut i = 0
  while i < list_len(list) {
    if predicate(list_get(list, i)) {
      count = count + 1
    }
    i = i + 1
  }
  count
}

// ==========================================
// 列表排序
// ==========================================

/// 反转列表
function list_reverse<T>(list: [T]): [T] {
  let mut result = []
  let mut i = list_len(list) - 1
  while i >= 0 {
    list_push(result, list_get(list, i))
    i = i - 1
  }
  result
}

/// 排序整数列表（升序）
function list_sort_int(list: [Int]): [Int] {
  // 简单的冒泡排序
  let len = list_len(list)
  if len <= 1 {
    return list
  }
  let mut result = list
  let mut i = 0
  while i < len - 1 {
    let mut j = 0
    while j < len - 1 - i {
      if result[j] > result[j + 1] {
        // 交换
        let temp = result[j]
        result[j] = result[j + 1]
        result[j + 1] = temp
      }
      j = j + 1
    }
    i = i + 1
  }
  result
}

/// 使用比较函数排序
function list_sort_with<T>(list: [T], compare: (T, T) -> Int): [T] {
  // 简单的冒泡排序
  let len = list_len(list)
  if len <= 1 {
    return list
  }
  let mut result = list
  let mut i = 0
  while i < len - 1 {
    let mut j = 0
    while j < len - 1 - i {
      if compare(result[j], result[j + 1]) > 0 {
        // 交换
        let temp = result[j]
        result[j] = result[j + 1]
        result[j + 1] = temp
      }
      j = j + 1
    }
    i = i + 1
  }
  result
}

// ==========================================
// 列表数值操作
// ==========================================

/// 计算整数列表的和
function list_sum(list: [Int]): Int {
  list_fold(list, 0, (acc, x) -> acc + x)
}

/// 计算浮点数列表的和
function list_sum_float(list: [Float]): Float {
  list_fold(list, 0.0, (acc, x) -> acc + x)
}

/// 计算整数列表的积
function list_product(list: [Int]): Int {
  list_fold(list, 1, (acc, x) -> acc * x)
}

/// 计算浮点数列表的积
function list_product_float(list: [Float]): Float {
  list_fold(list, 1.0, (acc, x) -> acc * x)
}

/// 查找整数列表的最小值
function list_min_int(list: [Int]): Option<Int> {
  if list_is_empty(list) {
    None()
  } else {
    Some(list_fold(list, list[0], (acc, x) -> min_int(acc, x)))
  }
}

/// 查找整数列表的最大值
function list_max_int(list: [Int]): Option<Int> {
  if list_is_empty(list) {
    None()
  } else {
    Some(list_fold(list, list[0], (acc, x) -> max_int(acc, x)))
  }
}

// ==========================================
// 范围生成
// ==========================================

/// 创建整数范围 [start, end)
function list_range(start: Int, end: Int): [Int] {
  let mut result = []
  let mut i = start
  while i < end {
    list_push(result, i)
    i = i + 1
  }
  result
}

/// 创建整数范围 [start, end]
function list_range_inclusive(start: Int, end: Int): [Int] {
  list_range(start, end + 1)
}

/// 创建带步长的范围
function list_range_step(start: Int, end: Int, step: Int): [Int] {
  if step == 0 {
    panic("list_range_step: 步长不能为0")
  }
  let mut result = []
  if step > 0 {
    let mut i = start
    while i < end {
      list_push(result, i)
      i = i + step
    }
  } else {
    let mut i = start
    while i > end {
      list_push(result, i)
      i = i + step
    }
  }
  result
}

// ==========================================
// 列表切片
// ==========================================

/// 获取列表切片 [start, end)
function list_slice<T>(list: [T], start: Int, end: Int): [T] {
  let len = list_len(list)
  let real_start = clamp_int(start, 0, len)
  let real_end = clamp_int(end, real_start, len)
  let mut result = []
  let mut i = real_start
  while i < real_end {
    list_push(result, list_get(list, i))
    i = i + 1
  }
  result
}

/// 获取前 n 个元素
function list_take<T>(list: [T], n: Int): [T] {
  list_slice(list, 0, n)
}

/// 去掉前 n 个元素
function list_drop<T>(list: [T], n: Int): [T] {
  list_slice(list, n, list_len(list))
}

// ==========================================
// 映射 (Map) 操作
// ==========================================

/// 创建一个新的空映射
function map_new<K, V>(): {K: V} {
  {}
}

/// 检查映射是否为空
function map_is_empty<K, V>(map: {K: V}): Bool {
  map_len(map) == 0
}

/// 获取映射的大小
function map_len<K, V>(map: {K: V}): Int {
  let mut count = 0
  for key in map {
    count = count + 1
  }
  count
}

/// 获取键对应的值
function map_get<K, V>(map: {K: V}, key: K): Option<V> {
  // 简单实现，实际需要底层支持
  if key in map {
    Some(map[key])
  } else {
    None()
  }
}

/// 插入键值对
function map_insert<K, V>(map: {K: V}, key: K, value: V): {K: V} {
  // 简单实现，实际需要底层支持
  let mut result = map
  result[key] = value
  result
}

/// 移除键值对
function map_remove<K, V>(map: {K: V}, key: K): (Option<V>, {K: V}) {
  let value = map_get(map, key)
  let mut result = map_new()
  for k in map {
    if k != key {
      result = map_insert(result, k, map[k])
    }
  }
  (value, result)
}

/// 检查映射是否包含键
function map_contains_key<K, V>(map: {K: V}, key: K): Bool {
  is_some(map_get(map, key))
}

/// 获取所有键
function map_keys<K, V>(map: {K: V}): [K] {
  let mut keys = []
  for key in map {
    list_push(keys, key)
  }
  keys
}

/// 获取所有值
function map_values<K, V>(map: {K: V}): [V] {
  let mut values = []
  for key in map {
    list_push(values, map[key])
  }
  values
}

/// 获取所有键值对
function map_entries<K, V>(map: {K: V}): [(K, V)] {
  let mut entries = []
  for key in map {
    list_push(entries, (key, map[key]))
  }
  entries
}

/// 从键值对列表创建映射
function map_from_entries<K, V>(entries: [(K, V)]): {K: V} {
  let mut result = map_new()
  let mut i = 0
  while i < list_len(entries) {
    let (k, v) = entries[i]
    result = map_insert(result, k, v)
    i = i + 1
  }
  result
}

/// 合并两个映射
function map_merge<K, V>(map1: {K: V}, map2: {K: V}): {K: V} {
  let mut result = map1
  let entries = map_entries(map2)
  let mut i = 0
  while i < list_len(entries) {
    let (k, v) = entries[i]
    result = map_insert(result, k, v)
    i = i + 1
  }
  result
}

// ==========================================
// 集合 (Set) 操作
// ==========================================

/// 创建一个新的空集合
function set_new<T>(): [T] {
  []
}

/// 创建一个包含元素的集合
function set_of<T>(items: [T]): [T] {
  let mut result = set_new()
  let mut i = 0
  while i < list_len(items) {
    result = set_insert(result, items[i])
    i = i + 1
  }
  result
}

/// 检查集合是否包含元素
function set_contains<T>(set: [T], item: T): Bool {
  list_contains(set, item)
}

/// 向集合添加元素
function set_insert<T>(set: [T], item: T): [T] {
  if set_contains(set, item) {
    set
  } else {
    list_push(set, item)
  }
}

/// 从集合移除元素
function set_remove<T>(set: [T], item: T): [T] {
  list_filter(set, (x) -> x != item)
}

/// 获取集合大小
function set_len<T>(set: [T]): Int {
  list_len(set)
}

/// 检查集合是否为空
function set_is_empty<T>(set: [T]): Bool {
  list_is_empty(set)
}

/// 集合的并集
function set_union<T>(set1: [T], set2: [T]): [T] {
  let mut result = set1
  let mut i = 0
  while i < list_len(set2) {
    result = set_insert(result, set2[i])
    i = i + 1
  }
  result
}

/// 集合的交集
function set_intersection<T>(set1: [T], set2: [T]): [T] {
  list_filter(set1, (x) -> set_contains(set2, x))
}

/// 集合的差集
function set_difference<T>(set1: [T], set2: [T]): [T] {
  list_filter(set1, (x) -> not set_contains(set2, x))
}

// ==========================================
// 辅助函数
// ==========================================

/// 计算两个数的最小值
function min_int(a: Int, b: Int): Int {
  if a < b { a } else { b }
}

/// 计算两个数的最大值
function max_int(a: Int, b: Int): Int {
  if a > b { a } else { b }
}

/// 限制值在指定范围内
function clamp_int(value: Int, min: Int, max: Int): Int {
  if value < min {
    min
  } else if value > max {
    max
  } else {
    value
  }
}
