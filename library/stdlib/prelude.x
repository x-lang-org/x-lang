// X 语言 Prelude
// 自动导入到每个 X 程序中的核心类型和函数

// 重新导出 Option 和 Result 类型
export use ./option.{Option, Some, None}
export use ./result.{Result, Ok, Err}

// 基本打印函数（由后端运行时实现）
extern function println(message: string)
extern function print(message: string)

// 程序退出
extern function exit(code: integer)

// 恐慌：不可恢复错误
function panic(message: string) {
    println("panic: " + message)
    exit(1)
}

// 断言
function assert(condition: boolean, message: string = "assertion failed") {
    if !condition {
        panic(message)
    }
}

function assert_eq<T>(a: T, b: T, message: string = "values not equal") {
    if a != b {
        panic(message)
    }
}

function assert_ne<T>(a: T, b: T, message: string = "values are equal") {
    if a == b {
        panic(message)
    }
}

// 未实现标记
function todo(message: string = "not implemented") {
    panic("todo: " + message)
}

function unimplemented(message: string = "unimplemented") {
    panic("unimplemented: " + message)
}

function unreachable(message: string = "unreachable code reached") {
    panic("unreachable: " + message)
}
