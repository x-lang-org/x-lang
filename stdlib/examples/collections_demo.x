// 集合操作演示
// 展示列表、映射和集合的使用

fun main() {
  println("=== 集合操作演示 ===")
  println()

  // ==========================================
  // 列表创建
  // ==========================================
  println("1. 列表创建")

  let empty = list_new()
  let single = list_of(42)
  let repeated = list_repeat("x", 5)
  let range = list_range(1, 6)
  let inclusive = list_range_inclusive(1, 5)

  println("  list_new(): " + to_string(empty))
  println("  list_of(42): " + to_string(single))
  println("  list_repeat(\"x\", 5): " + to_string(repeated))
  println("  list_range(1, 6): " + to_string(range))
  println("  list_range_inclusive(1, 5): " + to_string(inclusive))
  println()

  // ==========================================
  // 列表基本属性
  // ==========================================
  println("2. 列表基本属性")

  let numbers = [1, 2, 3, 4, 5]
  println("  列表: " + to_string(numbers))
  println("  list_len: " + to_string(list_len(numbers)))
  println("  list_is_empty: " + to_string(list_is_empty(numbers)))
  println("  list_is_empty([]): " + to_string(list_is_empty([])))
  println()

  // ==========================================
  // 列表元素访问
  // ==========================================
  println("3. 列表元素访问")

  println("  list_get(numbers, 0): " + to_string(list_get(numbers, 0)))
  println("  list_get(numbers, 2): " + to_string(list_get(numbers, 2)))
  println("  list_get(numbers, 4): " + to_string(list_get(numbers, 4)))
  println("  list_get(numbers, 10): " + to_string(list_get(numbers, 10)))
  println("  list_first: " + to_string(list_first(numbers)))
  println("  list_last: " + to_string(list_last(numbers)))
  println()

  // ==========================================
  // 列表修改
  // ==========================================
  println("4. 列表修改")

  let list1 = [1, 2, 3]
  let pushed = list_push(list1, 4)
  println("  list_push([1,2,3], 4): " + to_string(pushed))

  let inserted = list_insert(list1, 1, 99)
  println("  list_insert([1,2,3], 1, 99): " + to_string(inserted))

  let (popped_item, popped_list) = list_pop(pushed)
  println("  list_pop([1,2,3,4]): " + to_string(popped_item) + ", " + to_string(popped_list))

  let (removed_item, removed_list) = list_remove(list1, 1)
  println("  list_remove([1,2,3], 1): " + to_string(removed_item) + ", " + to_string(removed_list))
  println()

  // ==========================================
  // 列表连接和分割
  // ==========================================
  println("5. 列表连接和分割")

  let list_a = [1, 2, 3]
  let list_b = [4, 5, 6]
  let appended = list_append(list_a, list_b)
  println("  list_append([1,2,3], [4,5,6]): " + to_string(appended))

  let (left, right) = list_split_at(appended, 3)
  println("  list_split_at(..., 3): " + to_string(left) + ", " + to_string(right))
  println()

  // ==========================================
  // 列表变换
  // ==========================================
  println("6. 列表变换")

  let nums = [1, 2, 3, 4, 5]
  let mapped = list_map(nums, (x) -> x * 2)
  println("  list_map([1,2,3,4,5], (x)->x*2): " + to_string(mapped))

  let filtered = list_filter(nums, (x) -> x % 2 == 0)
  println("  list_filter([1,2,3,4,5], even): " + to_string(filtered))

  let sum = list_fold(nums, 0, (acc, x) -> acc + x)
  println("  list_fold([1,2,3,4,5], 0, +): " + to_string(sum))

  let product = list_fold(nums, 1, (acc, x) -> acc * x)
  println("  list_fold([1,2,3,4,5], 1, *): " + to_string(product))
  println()

  // ==========================================
  // 列表搜索
  // ==========================================
  println("7. 列表搜索")

  println("  list_contains([1,2,3], 2): " + to_string(list_contains(nums, 2)))
  println("  list_contains([1,2,3], 99): " + to_string(list_contains(nums, 99)))

  let found = list_find(nums, (x) -> x > 3)
  println("  list_find([1,2,3,4,5], (x)->x>3): " + to_string(found))

  let pos = list_position(nums, (x) -> x == 3)
  println("  list_position([1,2,3,4,5], (x)->x==3): " + to_string(pos))

  let all_pos = list_all(nums, (x) -> x > 0)
  println("  list_all([1,2,3,4,5], (x)->x>0): " + to_string(all_pos))

  let any_gt3 = list_any(nums, (x) -> x > 3)
  println("  list_any([1,2,3,4,5], (x)->x>3): " + to_string(any_gt3))

  let count_even = list_count(nums, (x) -> x % 2 == 0)
  println("  list_count([1,2,3,4,5], even): " + to_string(count_even))
  println()

  // ==========================================
  // 列表排序和反转
  // ==========================================
  println("8. 列表排序和反转")

  let to_reverse = [1, 2, 3, 4, 5]
  let reversed = list_reverse(to_reverse)
  println("  list_reverse([1,2,3,4,5]): " + to_string(reversed))

  let to_sort = [3, 1, 4, 1, 5, 9, 2, 6]
  let sorted = list_sort_int(to_sort)
  println("  list_sort_int([3,1,4,1,5,9,2,6]): " + to_string(sorted))
  println()

  // ==========================================
  // 列表数值操作
  // ==========================================
  println("9. 列表数值操作")

  let ints = [1, 2, 3, 4, 5]
  println("  list_sum([1,2,3,4,5]): " + to_string(list_sum(ints)))
  println("  list_min_int([3,1,4,1,5]): " + to_string(list_min_int([3, 1, 4, 1, 5])))
  println("  list_max_int([3,1,4,1,5]): " + to_string(list_max_int([3, 1, 4, 1, 5])))
  println()

  // ==========================================
  // 列表切片
  // ==========================================
  println("10. 列表切片")

  let long_list = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
  let slice = list_slice(long_list, 2, 7)
  println("  list_slice([0..9], 2, 7): " + to_string(slice))
  println("  list_take([0..9], 5): " + to_string(list_take(long_list, 5)))
  println("  list_drop([0..9], 5): " + to_string(list_drop(long_list, 5)))
  println()

  // ==========================================
  // 映射操作
  // ==========================================
  println("11. 映射操作")

  let mut map = map_new()
  map = map_insert(map, "a", 1)
  map = map_insert(map, "b", 2)
  map = map_insert(map, "c", 3)

  println("  映射: " + to_string(map))
  println("  map_len: " + to_string(map_len(map)))
  println("  map_is_empty: " + to_string(map_is_empty(map)))
  println("  map_get(\"b\"): " + to_string(map_get(map, "b")))
  println("  map_contains_key(\"a\"): " + to_string(map_contains_key(map, "a")))
  println("  map_contains_key(\"z\"): " + to_string(map_contains_key(map, "z")))
  println("  map_keys: " + to_string(map_keys(map)))
  println("  map_values: " + to_string(map_values(map)))

  let (removed_val, map2) = map_remove(map, "b")
  println("  map_remove(\"b\"): " + to_string(removed_val) + ", " + to_string(map2))
  println()

  // ==========================================
  // 集合操作
  // ==========================================
  println("12. 集合操作")

  let set1 = set_of([1, 2, 3, 4])
  let set2 = set_of([3, 4, 5, 6])

  println("  set1: " + to_string(set1))
  println("  set2: " + to_string(set2))
  println("  set_contains(set1, 2): " + to_string(set_contains(set1, 2)))
  println("  set_contains(set1, 5): " + to_string(set_contains(set1, 5)))

  let union = set_union(set1, set2)
  println("  set_union: " + to_string(union))

  let intersection = set_intersection(set1, set2)
  println("  set_intersection: " + to_string(intersection))

  let difference = set_difference(set1, set2)
  println("  set_difference(set1, set2): " + to_string(difference))
  println()

  println("=== 集合操作演示完成 ===")
}
