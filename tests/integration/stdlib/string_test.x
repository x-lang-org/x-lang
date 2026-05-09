// @test string operations
// @stdout: Hello
// @stdout: World
// @stdout: HelloWorld
// @stdout: 5
// @stdout: true
// @stdout: Hello
// @stdout: H
// @stdout: e
// @stdout: l
// @stdout: l
// @stdout: o
// @stdout: HELLO
// @stdout: hello
// @stdout: hello
// @stdout: a
// @stdout: b
// @stdout: c

let s1 = "Hello"
let s2 = "World"

println(s1)
println(s2)

let combined = s1 + s2
println(combined)

let len = s1.length()
println(len)

let greeting = "Hello, X Language!"
let contains = greeting.contains("X")
println(contains)

let substr = greeting.substring(0, 5)
println(substr)

for c in "Hello" {
    println(c)
}

let upper = "hello".toUpperCase()
let lower = "HELLO".toLowerCase()
println(upper)
println(lower)

let trimmed = "  hello  ".trim()
println(trimmed)

let parts = "a,b,c".split(",")
println(parts[0])
println(parts[1])
println(parts[2])
