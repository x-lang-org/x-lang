// @test Unicode stdin read_line_or_empty
// @stdin: 你好
// @stdout: Read line: 你好

import std.io

let read = read_line_or_empty()
println("Read line: " + read)
