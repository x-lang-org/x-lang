// Add Binary - LeetCode 67

function char_to_int(c: string) -> integer {
    if c == "0" { return 0 }
    if c == "1" { return 1 }
    return 0
}

function str_to_int(s: string) -> integer {
    let len_s = len(s)
    if len_s == 0 { return 0 }
    let mutable result = 0
    let mutable i = 0
    while i < len_s {
        let c = s[i]
        let digit = char_to_int(c)
        result = result * 2 + digit
        i = i + 1
    }
    return result
}

function int_to_bin(n: integer) -> string {
    if n == 0 { return "0" }
    let mutable result = ""
    let mutable x = n
    while x > 0 {
        let digit = x % 2
        if digit == 0 { result = "0" + result }
        if digit == 1 { result = "1" + result }
        x = x / 2
    }
    return result
}

function main() -> integer {
    let args = __args()
    let mutable a = "11"
    let mutable b = "1"

    let n_args = len(args)
    let mutable idx = 0
    while idx < n_args {
        if args[idx] == "--" {
            if idx + 1 < n_args {
                a = args[idx + 1]
            }
            if idx + 2 < n_args {
                b = args[idx + 2]
            }
        }
        idx = idx + 1
    }

    let n1 = str_to_int(a)
    let n2 = str_to_int(b)
    let sum = n1 + n2
    let result = int_to_bin(sum)
    println(result)
    return 0
}
