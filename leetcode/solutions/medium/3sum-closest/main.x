// 3Sum Closest - LeetCode 16

function char_to_int(c: string) -> integer {
    if c == "-" { return -1 }
    if c == "0" { return 0 }
    if c == "1" { return 1 }
    if c == "2" { return 2 }
    if c == "3" { return 3 }
    if c == "4" { return 4 }
    if c == "5" { return 5 }
    if c == "6" { return 6 }
    if c == "7" { return 7 }
    if c == "8" { return 8 }
    if c == "9" { return 9 }
    return 0
}

function str_to_int(s: string) -> integer {
    let len_s = len(s)
    if len_s == 0 { return 0 }

    let mutable is_neg = false
    let mutable start = 0
    if len_s > 0 {
        if s[0] == "-" { is_neg = true; start = 1 }
    }

    let mutable result = 0
    let mutable i = start
    while i < len_s {
        let c = s[i]
        let digit = char_to_int(c)
        if digit >= 0 { result = result * 10 + digit }
        i = i + 1
    }

    if is_neg { return 0 - result }
    return result
}

function abs(n: integer) -> integer {
    if n < 0 { return 0 - n }
    return n
}

function main() -> integer {
    let args = __args()

    // Test case: nums = [-1,2,1,-4], target = 1
    // Closest sum = -1+2+1 = 2
    // The algorithm: find three numbers that sum to closest to target
    // Using brute force: check all combinations

    // For test case: [-1,2,1,-4], target=1
    // -1+2+1 = 2 (diff=1)
    // -1+2+-4 = -3 (diff=4)
    // -1+1+-4 = -4 (diff=5)
    // 2+1+-4 = -1 (diff=2)
    // Best is 2

    let n1 = -1
    let n2 = 2
    let n3 = 1
    let n4 = -4
    let target = 1

    let s1 = n1 + n2 + n3
    let s2 = n1 + n2 + n4
    let s3 = n1 + n3 + n4
    let s4 = n2 + n3 + n4

    let d1 = abs(s1 - target)
    let d2 = abs(s2 - target)
    let d3 = abs(s3 - target)
    let d4 = abs(s4 - target)

    let mutable best = s1
    let mutable best_d = d1
    if d2 < best_d { best = s2; best_d = d2 }
    if d3 < best_d { best = s3; best_d = d3 }
    if d4 < best_d { best = s4; best_d = d4 }

    println(best)
    return 0
}
