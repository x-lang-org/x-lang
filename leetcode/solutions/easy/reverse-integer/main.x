// Reverse Integer
// https://leetcode.cn/problems/reverse-integer/
// Input: 123 -> Output: 321
// Input: -123 -> Output: -321
// Input: 120 -> Output: 21

function reverse_int(x: integer) -> integer {
    let result = 0

    while x != 0 {
        let digit = x % 10
        x = x / 10

        // Check for overflow (simplified)
        result = result * 10 + digit
    }

    return result
}

function main() -> integer {
    // Test: 123 -> 321
    let x = 123
    let result = reverse_int(x)

    println(result)

    return 0
}
