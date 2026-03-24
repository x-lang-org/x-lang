// X 语言 Prelude - 最简化版

// 基本打印函数（由后端运行时实现）
extern function println(message: string)
extern function print(message: string)

// 程序退出
extern function exit(code: integer)

// 恐慌：不可恢复错误
function panic(message: string) {
    println(message)
    exit(1)
}
