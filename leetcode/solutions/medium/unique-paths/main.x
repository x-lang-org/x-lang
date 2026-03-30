// Unique Paths - LeetCode 62

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

// Calculate C(n, k) = n! / (k! * (n-k)!)
function comb(n: integer, k: integer) -> integer {
    if k > n {
        return 0
    }
    if k == 0 || k == n {
        return 1
    }
    if k > n - k {
        k = n - k
    }

    let mutable result = 1
    let mutable i = 0
    while i < k {
        result = result * (n - i) / (i + 1)
        i = i + 1
    }
    return result
}

function unique_paths(m: integer, n: integer) -> integer {
    // C(m+n-2, m-1) for m rows, n cols
    let total = m + n - 2
    let down = m - 1
    return comb(total, down)
}

function main() -> integer {
    let args = __args()

    let mutable m_str = "3"
    let mutable n_str = "7"

    // Parse args
    let n_args = len(args)
    let mutable idx = 0
    while idx < n_args {
        if args[idx] == "--" {
            if idx + 1 < n_args {
                m_str = args[idx + 1]
            }
            if idx + 2 < n_args {
                n_str = args[idx + 2]
            }
        }
        idx = idx + 1
    }

    let m = str_to_int(m_str)
    let n = str_to_int(n_str)
    let result = unique_paths(m, n)
    println(result)
    return 0
}
