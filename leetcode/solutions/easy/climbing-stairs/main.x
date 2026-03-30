// Climbing Stairs - Read case.json and compute

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

function climb(n: integer) -> integer {
    if n == 1 { return 1 }
    if n == 2 { return 2 }

    let a = 1
    let b = 2
    let mutable i = 3

    while i <= n {
        let t = a + b
        a = b
        b = t
        i = i + 1
    }

    return b
}

function main() -> integer {
    let args = __args()

    let mutable n_str = "2"
    let mutable n = 2

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

    n = str_to_int(n_str)
    let result = climb(n)
    println(result)

    return 0
}
