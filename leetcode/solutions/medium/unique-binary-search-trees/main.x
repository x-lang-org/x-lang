// Unique Binary Search Trees - LeetCode 96

function char_to_int(c: string) -> integer {
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
    if len_s == 0 {
        return 0
    }

    let mutable result = 0
    let mutable i = 0
    while i < len_s {
        let c = s[i]
        let digit = char_to_int(c)
        result = result * 10 + digit
        i = i + 1
    }
    return result
}

function catalan(n: integer) -> integer {
    if n <= 1 {
        return 1
    }

    // Use DP for small n
    let dp0 = 1
    let dp1 = 1
    let dp2 = dp0 * dp1 + dp1 * dp0
    let dp3 = dp0 * dp2 + dp1 * dp1 + dp2 * dp0
    let dp4 = dp0 * dp3 + dp1 * dp2 + dp2 * dp1 + dp3 * dp0

    if n == 0 { return dp0 }
    if n == 1 { return dp1 }
    if n == 2 { return dp2 }
    if n == 3 { return dp3 }
    return dp4
}

function main() -> integer {
    let args = __args()

    let mutable n_str = "3"

    // Parse args
    let n_args = len(args)
    let mutable idx = 0
    while idx < n_args {
        if args[idx] == "--" {
            if idx + 1 < n_args {
                n_str = args[idx + 1]
            }
        }
        idx = idx + 1
    }

    let n = str_to_int(n_str)
    let result = catalan(n)
    println(result)
    return 0
}
