// Reverse Integer - LeetCode 7

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

function int_to_str(n: integer) -> string {
    if n == 0 { return "0" }

    let mutable result = ""
    let mutable x = n
    if x < 0 {
        result = "-"
        x = 0 - x
    }

    while x > 0 {
        let digit = x % 10
        if digit == 0 { result = "0" + result }
        if digit == 1 { result = "1" + result }
        if digit == 2 { result = "2" + result }
        if digit == 3 { result = "3" + result }
        if digit == 4 { result = "4" + result }
        if digit == 5 { result = "5" + result }
        if digit == 6 { result = "6" + result }
        if digit == 7 { result = "7" + result }
        if digit == 8 { result = "8" + result }
        if digit == 9 { result = "9" + result }
        x = x / 10
    }
    return result
}

function reverse_int(x: integer) -> integer {
    let mutable result = 0
    let mutable n = x

    if n < 0 {
        n = 0 - n
    }

    while n != 0 {
        let digit = n % 10
        result = result * 10 + digit
        n = n / 10
    }

    if x < 0 {
        return 0 - result
    }
    return result
}

function main() -> integer {
    let args = __args()

    let mutable x_str = "123"
    let mutable x = 123

    let n_args = len(args)
    let mutable idx = 0
    while idx < n_args {
        if args[idx] == "--" {
            if idx + 1 < n_args {
                x_str = args[idx + 1]
            }
        }
        idx = idx + 1
    }

    x = str_to_int(x_str)
    let result = reverse_int(x)
    println(result)

    return 0
}
