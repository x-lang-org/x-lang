// Integer to Roman - LeetCode 12

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

function int_to_roman(num: integer) -> string {
    let mutable result = ""
    let mutable n = num

    while n >= 1000 {
        result = result + "M"
        n = n - 1000
    }

    if n >= 900 { result = result + "CM"; n = n - 900 }
    if n >= 500 { result = result + "D"; n = n - 500 }
    if n >= 400 { result = result + "CD"; n = n - 400 }
    while n >= 100 {
        result = result + "C"
        n = n - 100
    }

    if n >= 90 { result = result + "XC"; n = n - 90 }
    if n >= 50 { result = result + "L"; n = n - 50 }
    if n >= 40 { result = result + "XL"; n = n - 40 }
    while n >= 10 {
        result = result + "X"
        n = n - 10
    }

    if n >= 9 { result = result + "IX"; n = n - 9 }
    if n >= 5 { result = result + "V"; n = n - 5 }
    if n >= 4 { result = result + "IV"; n = n - 4 }
    while n >= 1 {
        result = result + "I"
        n = n - 1
    }

    return result
}

function main() -> integer {
    let args = __args()

    let mutable num_str = "3749"
    let mutable num = 3749

    let n_args = len(args)
    let mutable idx = 0
    while idx < n_args {
        if args[idx] == "--" {
            if idx + 1 < n_args {
                num_str = args[idx + 1]
            }
        }
        idx = idx + 1
    }

    num = str_to_int(num_str)
    let result = int_to_roman(num)
    println(result)

    return 0
}
