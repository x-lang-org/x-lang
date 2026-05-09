// @test recursive functions
// @stdout: 55
// @stdout: 120
// @stdout: 55
// @stdout: 5050

function fib(n: integer) -> integer {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

let fib10 = fib(10)
println(fib10)

function factorial(n: integer) -> integer {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

let fact5 = factorial(5)
println(fact5)

function sumTo(n: integer) -> integer {
    if n <= 0 {
        0
    } else {
        n + sumTo(n - 1)
    }
}

let sum100 = sumTo(10)
println(sum100)

let mutable total = 0
for i in 1..=100 {
    total = total + i
}
println(total)
