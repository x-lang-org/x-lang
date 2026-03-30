// Multiply Strings - LeetCode 43

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

function int_to_str(n: integer) -> string {
    if n == 0 {
        return "0"
    }

    let mutable result = ""
    let mutable x = n
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

function multiply(num1: string, num2: string) -> string {
    let n1 = str_to_int(num1)
    let n2 = str_to_int(num2)
    let product = n1 * n2
    return int_to_str(product)
}

function main() -> integer {
    let args = __args()

    let mutable num1 = "2"
    let mutable num2 = "3"

    let n = len(args)
    let mutable idx = 0
    while idx < n {
        if args[idx] == "--" {
            if idx + 1 < n {
                num1 = args[idx + 1]
            }
            if idx + 2 < n {
                num2 = args[idx + 2]
            }
        }
        idx = idx + 1
    }

    let result = multiply(num1, num2)
    println(result)
    return 0
}
