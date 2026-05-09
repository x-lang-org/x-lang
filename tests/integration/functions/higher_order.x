// @test higher-order functions
// @stdout: 12
// @stdout: 7
// @stdout: 21
// @stdout: 2
// @stdout: 4
// @stdout: 6

function applyTwice(f, x) {
    f(f(x))
}

function double(x) = x * 2
function increment(x) = x + 1

let result1 = applyTwice(double, 3)
println(result1)

let result2 = applyTwice(increment, 5)
println(result2)

function compose(f, g) {
    function(x) -> f(g(x))
}

let doubleThenInc = compose(increment, double)
println(doubleThenInc(10))

function mapManual(list, f) {
    let result = []
    for item in list {
        result = result + [f(item)]
    }
    result
}

let numbers = [1, 2, 3]
let doubled = mapManual(numbers, double)
println(doubled[0])
println(doubled[1])
println(doubled[2])
