// 测试 Option/Result 函数调用
let opt = Some(42)
println(is_some(opt))

let none: Option<Int> = None
println(is_none(none))

let ok_res = Ok(100)
println(is_ok(ok_res))

let err_res = Err("error")
println(is_err(err_res))