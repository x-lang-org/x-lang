// 测试函数定义简写
function add(a, b) = a + b

// 测试const绑定
const PI = 3.14159

// 测试管道操作符
function double(x) = x * 2
function square(x) = x * x

// 测试when表达式
function abs(x) = when x < 0 then -x else x

// 测试for循环和范围表达式
function sum_range(start, end) {
    let sum = 0
    for i in start..end {
        sum = sum + i
    }
    return sum
}

// 测试Option和Result类型
function test_option(x) {
    if x > 0 {
        return Some(x)
    } else {
        return None
    }
}

function test_result(x) {
    if x > 0 {
        return Ok(x)
    } else {
        return Err("Negative number")
    }
}

// 主函数
function main() {
    print("Hello, X Language!")
    print("2 + 3 = " + add(2, 3))
    print("PI = " + PI)
    print("4 doubled and squared = " + (4 |> double |> square))
    print("abs(-5) = " + abs(-5))
    print("sum(1..5) = " + sum_range(1, 5))
    print("test_option(5) = " + type_of(test_option(5)))
    print("test_option(-5) = " + type_of(test_option(-5)))
    print("test_result(5) = " + type_of(test_result(5)))
    print("test_result(-5) = " + type_of(test_result(-5)))
}
