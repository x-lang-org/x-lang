// 测试 enum 和 match
enum Color {
    Red,
    Green,
    Blue
}

function get_name(c: Color): string {
    match c {
        Red => "red",
        Green => "green",
        Blue => "blue"
    }
}

let c = Red;
println(get_name(c));

let c2 = Green;
println(get_name(c2));

println("Test completed!");
