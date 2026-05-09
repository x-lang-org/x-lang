// @test math operations
// @stdout: 3.14159
// @stdout: 2.71828
// @stdout: -4
// @stdout: 2
// @stdout: 8
// @stdout: 3
// @stdout: 3
// @stdout: 5
// @stdout: 10

let pi = 3.14159
let e = 2.71828

println(pi)
println(e)

let abs_val = -4.abs()
println(abs_val)

let sqrt_val = 4.sqrt()
println(sqrt_val)

let pow_val = 2.pow(3)
println(pow_val)

let floor_val = 3.7.floor()
println(floor_val)

let ceil_val = 2.1.ceil()
println(ceil_val)

let min_val = min(5, 10)
let max_val = max(5, 10)
println(min_val)
println(max_val)
