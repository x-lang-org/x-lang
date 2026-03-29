// 两数之和
// https://leetcode.cn/problems/two-sum/

needs stdio

can two_sum(nums: []int, target: int) -> []int {
    // Hash map to store value -> index
    given map: [int]int = [:]

    for i in 0..nums.length - 1 {
        complement = target - nums[i]
        if map contains complement {
            return [map[complement], i]
        }
        map[nums[i]] = i
    }

    // Problem states exactly one solution exists
    return []
}

// Read input and output result
when is main {
    // For simplicity, read from stdin:
    // First line: n, then n numbers, then target
    n = 0
    _ = scanf("%d", &n)

    given nums: []int = [] with cap n
    for i in 0..n-1 {
        x = 0
        _ = scanf("%d", &x)
        nums push x
    }

    target = 0
    _ = scanf("%d", &target)

    result = two_sum(nums, target)
    printf("[%d, %d]\n", result[0], result[1])
}
