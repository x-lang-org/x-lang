// @test IO operations
// @stdout: Hello, World!
// @stdout: Enter a number: 42
// @stdout: File content: test data
// @stdin: 42
// @stdout: Read line: 42

println("Hello, World!")

let input = "42"
println("Enter a number: " + input)

let fileContent = "test data"
println("File content: " + fileContent)

let data = "line1\nline2\nline3"
let lines = data.split("\n")
println(lines[0])
println(lines[1])
println(lines[2])

let formatted = "Name: X, Version: 1.0"
println(formatted)

import std.io

let read = read_line_or_empty()
println("Read line: " + read)
