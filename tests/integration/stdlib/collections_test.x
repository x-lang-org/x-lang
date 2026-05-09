// @test collection operations
// @stdout: 1
// @stdout: 2
// @stdout: 3
// @stdout: 5
// @stdout: 1
// @stdout: 5
// @stdout: 15
// @stdout: 1
// @stdout: 4
// @stdout: 3
// @stdout: 0

let numbers = [1, 2, 3, 4, 5]

println(numbers[0])
println(numbers[1])
println(numbers[2])

let len = numbers.length()
println(len)

let first = numbers[0]
let last = numbers[numbers.length() - 1]
println(first)
println(last)

let mutable sum = 0
for n in numbers {
    sum = sum + n
}
println(sum)

let nested = [[1, 2], [3, 4]]
println(nested[0][0])
println(nested[1][1])

let mutable arr = [1, 2]
arr = arr + [3]
println(arr[2])

let empty = []
println(empty.length())
