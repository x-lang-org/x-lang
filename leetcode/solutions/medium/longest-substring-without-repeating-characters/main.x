// 无重复字符的最长子串
// https://leetcode.cn/problems/longest-substring-without-repeating-characters/

needs stdio

can length_of_longest_substring(s: string) -> int {
    // Sliding window with hash map
    given map: [char]int = [:]
    var left = 0
    var max_len = 0

    for right in 0..s.length - 1 {
        c = s[right]
        if map contains c {
            // If character already in window, move left to past position
            left = max(left, map[c] + 1)
        }
        map[c] = right
        current_len = right - left + 1
        if current_len > max_len {
            max_len = current_len
        }
    }

    return max_len
}

when is main {
    // Read input string
    // Note: reads until newline
    buf = [1000]byte
    _ = scanf("%s", buf)

    s = buf as string
    result = length_of_longest_substring(s)
    printf("%d\n", result)
}
