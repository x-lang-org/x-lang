function factorial(n: integer) -> integer {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

function main() {
    let result = factorial(5)
    println(result)
}