// 测试 Option 方法调用
let opt = Some(42)

// 方法调用语法
println(opt.is_some())

let none: Option<Int> = None
println(none.is_none())

// 函数调用语法
println(is_some(opt))
println(is_none(none))