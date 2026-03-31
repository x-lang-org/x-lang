// Test basic string interpolation
let name = "World";
let greeting = "Hello, ${name}!";
println(greeting);

// Test multiple interpolations
let a = 42;
let b = 1337;
let message = "a is ${a}, b is ${b}";
println(message);

// Test interpolation with expression
let x = 10;
let y = 20;
let sum_str = "Sum: ${x + y}";
println(sum_str);

// Test empty string with interpolation
let empty_interp = "${x + y}";
println(empty_interp);

// Test adjacent interpolations
let first = "Hello";
let last = "World";
let both = "${first} ${last}";
println(both);

// Test multiline string with interpolation
let multi = """
Hello
This is a multi-line
string with ${name}
""";
println(multi);
