// @test stdin preserves boundary whitespace
// @stdin:   padded input  
// @stdout: Read line: [  padded input  ]

import std.io

let read = read_line_or_empty()
println("Read line: [" + read + "]")
