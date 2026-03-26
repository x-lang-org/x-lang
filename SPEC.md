# X 语言规范文档

> 本文档是 X 语言的正式语法与语义定义。所有语言行为、语法规则和类型系统规则以本文档为准。

---

## 设计哲学

**可读性第一**：X 语言的语法设计遵循"代码即文档"原则，让代码读起来像自然语言一样流畅。

### 核心原则

| 原则 | 示例 |
|------|------|
| **全称关键字** | `function` 而非 `fn`，`define` 而非 `let` |
| **自然语序** | `function foo() returns Integer` 而非 `function foo() -> Integer` |
| **英文连接词** | `List of Integer` 而非 `List<Integer>` |
| **动词开头** | `define x = 1`、`export function foo()` |
| **从句风格** | `when x is 0 then "zero"` |

---

## 语言概述

X 是一门**现代的、通用的编程语言**，适用于从底层系统编程到上层应用开发的全栈场景。

### 核心设计特征

| 特性 | 描述 |
|------|------|
| **自然语言语法** | 关键字全称、自然语序、英文连接词 |
| **类型安全** | Hindley-Milner 类型推断、代数数据类型、穷尽模式匹配 |
| **无 null、无异常** | 用 `Optional<T>` 代替 null，用 `Result<T, E>` 代替异常 |
| **内存安全** | Perceus 编译时引用计数——无 GC 停顿、无手动管理 |
| **多范式** | 函数式 + 面向对象 + 过程式 + 声明式 |
| **效果系统** | 函数副作用在类型签名中显式声明（`requires Effects`） |
| **结构化并发** | `async`/`await`、`concurrently`、`race`、`atomic`/`retry` |
| **多后端** | Zig → 原生/Wasm、JVM 字节码、.NET CIL、JavaScript |

---

## EBNF 约定

| 符号 | 含义 |
|------|------|
| `=` | 定义 |
| `,` | 连接 |
| `\|` | 选择 |
| `[ ]` | 可选（0 或 1 次） |
| `{ }` | 重复（0 或多次） |
| `{ }+` | 重复（1 或多次） |
| `( )` | 分组 |
| `"..."` | 终结符（字面量） |
| `'...'` | 终结符（字面量） |
| `(* ... *)` | 注释 |

---

## 1. 词法结构

### 1.1 字符集

```ebnf
(* 基本字符 *)
letter = lowercase | uppercase | unicode_letter ;
lowercase = "a" | "b" | "c" | ... | "z" ;
uppercase = "A" | "B" | "C" | ... | "Z" ;
digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" ;
hex_digit = digit | "a" | "b" | "c" | "d" | "e" | "f" | "A" | "B" | "C" | "D" | "E" | "F" ;

(* 空白字符 *)
whitespace = " " | "\t" | "\n" | "\r" ;
```

- 源代码使用 **UTF-8** 编码
- 标识符支持 Unicode 字母和数字

### 1.2 注释

```ebnf
comment = line_comment | block_comment ;
line_comment = "//" { (* 除换行符外的任何字符 *) } ;
block_comment = "/*" { block_comment_content } "*/" ;
block_comment_content = (* 任何字符 *) | block_comment ; (* 支持嵌套 *)
```

```x
// 这是单行注释，用于简短说明

/* 这是
   多行注释，
   用于较长说明 */

/* 嵌套注释 /* 也是支持的 */
   用于注释掉包含注释的代码段 */
```

### 1.3 标识符

```ebnf
identifier = identifier_start { identifier_continue } ;
identifier_start = letter | "_" | unicode_letter ;
identifier_continue = letter | digit | "_" | unicode_letter ;
```

```x
define name = "X"
define snake_case = 1
define camelCase = 2
define _private = 3
define 变量名 = 42  // 支持 Unicode，但推荐英文
```

### 1.4 关键字

```ebnf
keyword = "define" | "mutable" | "constant"
        | "function" | "returns" | "async" | "await"
        | "if" | "then" | "else" | "when" | "is" | "otherwise"
        | "for" | "each" | "in" | "while" | "loop" | "break" | "continue"
        | "type" | "class" | "trait" | "implement" | "enum" | "record" | "effect"
        | "module" | "import" | "from" | "export" | "public"
        | "try" | "catch" | "finally" | "throw"
        | "needs" | "given" | "requires"
        | "concurrently" | "race" | "atomic" | "retry"
        | "of" | "as" | "and" | "or" | "not" | "to" | "extends" | "where"
        | "true" | "false" | "self" | "Self" | "constructor" | "method" | "field" ;
```

| 类别 | 关键字 | 自然语言含义 |
|------|--------|-------------|
| 声明 | `define`, `mutable`, `constant` | 定义、可变的、常量 |
| 函数 | `function`, `returns`, `async`, `await` | 函数、返回、异步、等待 |
| 控制流 | `if`, `then`, `else`, `when`, `is`, `otherwise` | 如果、则、否则、当、是、其他情况 |
| 循环 | `for`, `each`, `in`, `while`, `loop` | 对于、每个、在...中、当...时、循环 |
| 类型 | `type`, `class`, `trait`, `implement`, `enum`, `record` | 类型、类、特质、实现、枚举、记录 |
| 模块 | `module`, `import`, `from`, `export`, `public` | 模块、导入、从...、导出、公开 |
| 效果 | `needs`, `given`, `requires` | 需要、给定、要求 |
| 并发 | `concurrently`, `race`, `atomic`, `retry` | 并发地、竞争、原子、重试 |
| 类型连接 | `of`, `to`, `or`, `extends`, `where` | ...的、到、或、继承、约束 |

### 1.5 字面量

```ebnf
literal = integer_literal | float_literal | boolean_literal | string_literal
        | char_literal | list_literal | dict_literal | tuple_literal | unit_literal ;

(* 整数 *)
integer_literal = decimal_literal | hex_literal | octal_literal | binary_literal ;
decimal_literal = digit { digit } ;
hex_literal = "0" ("x" | "X") hex_digit { hex_digit } ;
octal_literal = "0" ("o" | "O") octal_digit { octal_digit } ;
binary_literal = "0" ("b" | "B") binary_digit { binary_digit } ;

(* 浮点数 *)
float_literal = decimal_literal "." decimal_literal [ exponent ] | decimal_literal exponent ;
exponent = ("e" | "E") ["+" | "-"] decimal_literal ;

(* 布尔 *)
boolean_literal = "true" | "false" ;

(* 字符串 *)
string_literal = `"` { string_char | escape_sequence } `"` ;
escape_sequence = `\` ("n" | "t" | "r" | `\` | `"` | `'` | "0" | unicode_escape) ;

(* 字符 *)
char_literal = `'` (char_char | escape_sequence) `'` ;

(* 列表 - 使用方括号 *)
list_literal = "[" [ expression { "," expression } ] "]" ;

(* 字典 - 使用大括号加冒号键值对 *)
dict_literal = "{" [ dict_entry { "," dict_entry } ] "}" ;
dict_entry = identifier ":" expression | string_literal ":" expression ;

(* 元组 - 使用圆括号，至少两个元素 *)
tuple_literal = "(" expression "," expression { "," expression } [ "," ] ")" ;

(* 单元值 - 空圆括号 *)
unit_literal = "()" ;
```

```x
// 整数
define age = 25
define hex = 0xFF
define octal = 0o755
define binary = 0b1010

// 浮点数
define pi = 3.14159
define scientific = 1.5e-10

// 布尔值
define active = true
define disabled = false

// 字符串
define greeting = "Hello, World!"
define multiline = "Line1\nLine2"
define escaped = "Tab:\tQuote:\""

// 字符
define grade = 'A'
define chinese = '中'

// 列表 - 方括号
define numbers = [1, 2, 3, 4, 5]
define names = ["Alice", "Bob", "Charlie"]

// 字典 - 大括号，键值对用冒号
define scores = { Alice: 95, Bob: 87, Charlie: 92 }
define config = { host: "localhost", port: 8080 }

// 元组 - 圆括号，至少两个元素
define point = (10, 20)
define person = ("Alice", 30, true)

// 单元值 - 空圆括号
define nothing = ()
```

---

## 2. 类型系统

### 2.1 基本类型

```ebnf
primitive_type = "Integer" | "Float" | "Boolean" | "String" | "Character" | "Unit" | "Nothing" ;
```

| 类型 | 描述 | 示例 |
|------|------|------|
| `Integer` | 整数 | `42` |
| `Float` | 浮点数 | `3.14` |
| `Boolean` | 布尔值 | `true` / `false` |
| `String` | 字符串 | `"Hello"` |
| `Character` | 单个字符 | `'A'` |
| `Unit` | 空值 | `()` |
| `Nothing` | 永不返回 | `panic()` 的返回类型 |

```x
define age: Integer = 25
define pi: Float = 3.14159
define active: Boolean = true
define grade: Character = 'A'
define name: String = "X Language"
define result: Unit = println("Hello")
```

### 2.2 复合类型

```ebnf
type = type_expr [ "?" | "!" ] ;

type_expr = simple_type | compound_type | function_type | type_variable ;

simple_type = type_name [ type_arguments ] ;
type_name = identifier ;

(* 使用 "of" 连接泛型参数，更接近自然语言 *)
type_arguments = "of" type { "," type } ;

compound_type = tuple_type | list_type | map_type | optional_type | result_type ;
tuple_type = "(" [ type { "," type } ] ")" ;
list_type = "List" "of" type ;
map_type = "Map" "of" type "to" type ;
optional_type = "Optional" "of" type ;
result_type = "Result" "of" type "with_error" type ;
```

```x
// 列表类型
define numbers: List of Integer = [1, 2, 3]
define names: List of String = ["Alice", "Bob"]

// 字典类型
define scores: Map of String to Integer = { Alice: 95, Bob: 87 }

// 元组类型
define point: (Float, Float) = (10.5, 20.5)
define person: (String, Integer, Boolean) = ("Alice", 30, true)

// 嵌套类型
define matrix: List of List of Integer = [[1, 2], [3, 4]]

// Optional 和 Result 类型
define maybe: Optional of Integer = Some(42)
define outcome: Result of String with_error IoError = Success("ok")
```

### 2.3 函数类型

```ebnf
function_type = "function" "from" "(" [ param_type_list ] ")" "returns" type ;
param_type_list = type { "," type } ;
```

```x
// 函数类型使用自然语言风格
define add: function from (Integer, Integer) returns Integer = (a, b) => a + b
define greet: function from (String) returns String = name => "Hello, " + name

// 高阶函数
define apply: function from (function from (Integer) returns Integer, Integer) returns Integer = (f, x) => f(x)
```

### 2.4 Optional 和 Result

```ebnf
optional_type = "Optional" "of" type ;
result_type = "Result" "of" type "with_error" type ;
```

> **注意**：使用 `with_error` 而非 `or` 作为 Result 类型的分隔符，避免与逻辑运算符 `or` 产生歧义。

```x
// Optional<T> - 表示"可能有值"
type Optional<T> = Some(T) | None

// Result<T, E> - 表示"成功或失败"
type Result<T, E> = Success(T) | Failure(E)

// 使用示例
define maybe_number: Optional of Integer = Some(42)
define no_value: Optional of Integer = None

define success: Result of Integer with_error String = Success(100)
define failure: Result of Integer with_error String = Failure("error occurred")
```

### 2.5 代数数据类型

```ebnf
type_definition = enum_definition | record_definition | alias_definition ;

(* 枚举 *)
enum_definition = "define" "enum" identifier [ type_parameters ] "{" { enum_variant } "}" ;
enum_variant = identifier
             | identifier "(" type_list ")"
             | identifier "{" field_list "}" ;

(* 记录 *)
record_definition = "define" "record" identifier [ type_parameters ] "{" field_list "}" ;
field_list = field { "," field } ;
field = identifier ":" type [ default_value ] ;
default_value = "=" expression ;

(* 别名 *)
alias_definition = "define" "type" identifier [ type_parameters ] "=" type ;
type_parameters = "of" identifier { "," identifier } ;
```

```x
// 枚举（sum type）- "或"的关系
define enum Color {
    Red
    Green
    Blue
    RGB(Integer, Integer, Integer)
}

// 带泛型的枚举
define enum Optional of T {
    Some(T)
    None
}

define enum Result of T, E {
    Success(T)
    Failure(E)
}

// 记录（product type）- "和"的关系
define record Person {
    name: String
    age: Integer
    email: String = ""  // 默认值
}

// 类型别名
define type UserId = Integer
define type Point = (Float, Float)
define type Name = String
```

### 2.6 泛型

```ebnf
type_reference = type_name [ type_arguments ] ;
type_arguments = "of" type { "," type } ;
```

```x
function first of T(list: List of T) returns Optional of T {
    when list is {
        [] => None
        [x, ...] => Some(x)
    }
}

function identity of T(value: T) returns T = value

function pair of A, B(a: A, b: B) returns (A, B) = (a, b)
```

---

## 3. 表达式

### 3.1 运算符优先级

| 优先级 | 运算符 | 结合性 | 描述 |
|--------|--------|--------|------|
| 1 (最高) | `.` `(` `[` `?` | 左 | 成员访问、调用、索引、可选链 |
| 2 | `not` `-` (一元) | 右 | 逻辑非、负号 |
| 3 | `*` `/` `%` | 左 | 乘除、取模 |
| 4 | `+` `-` | 左 | 加减 |
| 5 | `<` `>` `<=` `>=` `==` `!=` | 无 | 比较 |
| 6 | `and` | 左 | 逻辑与 |
| 7 | `or` | 左 | 逻辑或 |
| 8 | `?` `??` | 右 | 错误传播、空合并 |
| 9 | `|>` | 左 | 管道 |
| 10 (最低) | `=` `+=` `-=` `*=` `/=` | 右 | 赋值 |

### 3.2 算术与逻辑

```ebnf
expression = assignment_expr ;

assignment_expr = pipeline_expr [ assignment_op expression ] ;
assignment_op = "=" | "+=" | "-=" | "*=" | "/=" | "%=" ;

pipeline_expr = coalesce_expr { "|>" coalesce_expr } ;

coalesce_expr = or_expr [ ("?" | "??") expression ] ;

or_expr = and_expr { "or" and_expr } ;
and_expr = not_expr { "and" not_expr } ;
not_expr = "not" not_expr | comparison_expr ;

comparison_expr = add_expr [ comparison_op add_expr ] ;
comparison_op = "==" | "!=" | "<" | ">" | "<=" | ">=" ;

add_expr = mul_expr { ("+" | "-") mul_expr } ;
mul_expr = unary_expr { ("*" | "/" | "%") unary_expr } ;
unary_expr = ("-" | "+") unary_expr | postfix_expr ;
```

```x
// 算术运算
define sum = a + b
define diff = a - b
define product = a * b
define quotient = a / b
define remainder = a % b

// 逻辑运算 - 使用英文单词
define both_true = a and b
define either_true = a or b
define negated = not a

// 比较
define is_equal = a == b
define is_different = a != b
define is_less = a < b
define is_greater = a > b
define is_less_or_equal = a <= b
define is_greater_or_equal = a >= b

// 复合表达式
define complex = (a + b) * c and not (d > e)
```

### 3.3 管道运算符

```x
// 管道让操作顺序从左到右，更自然
define result = numbers
    |> filter(is_even)
    |> map(square)
    |> take(10)
    |> sum

// 等价于嵌套调用（从内到外，难以阅读）
define result_equivalent = sum(take(map(filter(numbers, is_even), square), 10))
```

### 3.4 函数调用与成员访问

```ebnf
postfix_expr = primary_expr { postfix } ;
postfix = field_access | method_call | index_access | call_arguments | "?" ;

field_access = "." identifier ;
method_call = "." identifier call_arguments ;
index_access = "[" expression "]" ;
call_arguments = "(" [ argument_list ] ")" ;
argument_list = expression { "," expression } | named_argument { "," named_argument } ;
named_argument = identifier ":" expression ;
```

```x
// 函数调用
define result = calculate(a, b, c)

// 命名参数，更清晰
define greeting = greet(name: "Alice", title: "Dr.")

// 方法调用
define upper_name = name.to_upper()
define sorted = numbers.sort()

// 链式调用
define processed = text.trim().to_lower().split(" ")

// 成员访问
define person_name = person.name
define deep_value = config.server.host

// 索引访问
define first = numbers[0]
define value = scores["Alice"]

// 可选链
define email = user?.profile?.email
```

### 3.5 Lambda 表达式

```ebnf
lambda_expr = lambda_params "=>" ( expression | block ) ;
lambda_params = "(" [ lambda_param_list ] ")" | identifier ;
lambda_param_list = lambda_param { "," lambda_param } ;
lambda_param = identifier [ ":" type ] ;
```

```x
// 单参数简写
define square = x => x * x

// 多参数
define add = (a, b) => a + b

// 带类型注解
define multiply = (a: Integer, b: Integer) => a * b

// 块体 lambda
define process = (x, y) => {
    define sum = x + y
    define diff = x - y
    sum * diff
}

// 在高阶函数中使用
define doubled = numbers.map(x => x * 2)
define evens = numbers.filter(x => x % 2 == 0)
define sum = numbers.reduce((acc, x) => acc + x, 0)
```

### 3.6 if 表达式

```ebnf
if_expr = "if" expression "then" ( expression | block )
       [ "else" ( expression | block ) ] ;
```

```x
// if-then-else 语句
if score >= 60 then {
    println("Passed")
} else {
    println("Failed")
}

// if 表达式（返回值）
define grade = if score >= 90 then "A"
            else if score >= 80 then "B"
            else if score >= 70 then "C"
            else "F"

// 单行形式
define max = if a > b then a else b

// 嵌套
define description = if x > 0 then {
    if x > 100 then "very large"
    else "positive"
} else if x < 0 then {
    "negative"
} else {
    "zero"
}
```

### 3.7 when 表达式（模式匹配）

```ebnf
when_expr = "when" expression "is" "{" { when_arm } "}" ;
when_arm = pattern [ guard ] "=>" ( expression | block ) [ "," ] ;
guard = "if" expression ;
```

```x
// 基本模式匹配
define description = when score is {
    100 => "perfect"
    n if n >= 90 => "excellent"
    n if n >= 60 => "passed"
    otherwise => "failed"
}

// 解构匹配
define location = when point is {
    (0, 0) => "origin"
    (x, 0) => "on x-axis at " + x
    (0, y) => "on y-axis at " + y
    (x, y) => "at (" + x + ", " + y + ")"
}

// 列表匹配
define head = when list is {
    [] => None
    [first, ...] => Some(first)
}

// 类型匹配
define info = when value is {
    s as String => "string: " + s
    n as Integer => "number: " + n
    b as Boolean => "boolean: " + b
    otherwise => "unknown type"
}
```

### 3.8 块表达式

```ebnf
block_expr = block ;
block = "{" { statement } [ expression ] "}" ;
```

```x
// 块作为表达式
define result = {
    define x = 10
    define y = 20
    x + y  // 最后一个表达式作为返回值
}

// 带副作用
define processed = {
    define temp = calculate()
    log("calculated: " + temp)
    transform(temp)
}

// 用于控制流
define value = if condition then {
    prepare()
    compute()
} else {
    default_value()
}
```

### 3.9 构造表达式

```ebnf
(* 类型构造器调用 - 使用类型名后跟圆括号 *)
constructor_expr = type_name "(" [ constructor_args ] ")" ;
constructor_args = expression { "," expression } | named_arg { "," named_arg } ;
named_arg = identifier ":" expression ;
```

```x
// 构造枚举变体
define some_value = Some(42)
define none_value = None
define success = Success("data")
define failure = Failure("error message")

// 构造记录
define person = Person(name: "Alice", age: 30)

// 构造带位置参数的变体
define color = Color.RGB(255, 128, 0)
```

---

## 4. 语句与声明

### 4.1 变量绑定

```ebnf
let_statement = "define" [ "mutable" ] identifier [ ":" type ] "=" expression ;
const_statement = "constant" identifier [ ":" type ] "=" expression ;
```

```x
// 不可变绑定（推荐）
define name = "X Language"
define age: Integer = 25

// 可变绑定
define mutable counter = 0
counter = counter + 1

// 常量（编译时确定）
constant MAX_SIZE = 1024
constant GREETING: String = "Hello, World!"
constant PI = 3.14159
```

### 4.2 控制流

```ebnf
if_statement = "if" expression "then" ( statement | block )
             [ "else" ( statement | block ) ] ;
```

```x
// if-then-else
if temperature > 30 then {
    println("It's hot!")
    turn_on_ac()
} else if temperature < 10 then {
    println("It's cold!")
    turn_on_heater()
} else {
    println("Comfortable!")
}

// 表达式形式
define action = if danger_level > 5 then "evacuate" else "stay calm"
```

### 4.3 循环

```ebnf
while_statement = "while" expression block ;
for_each_statement = "for" "each" identifier "in" expression block ;
loop_statement = "loop" block ;
break_statement = "break" [ expression ] ;
continue_statement = "continue" ;
```

> **设计说明**：X 语言只提供 `for each` 循环，不支持传统的 C 风格 `for(init; cond; update)` 循环。范围迭代使用 `1..10` 语法。

```x
// for each 循环 - 自然语言风格
for each item in collection {
    println(item)
}

for each number in 1..10 {
    println("Number: " + number)
}

// while 循环
while has_more_data() {
    process(next_data())
}

// 无限循环
loop {
    define input = read_input()
    if input == "quit" then break
    handle(input)
}

// 循环控制
for each item in items {
    if should_skip(item) then continue
    if should_stop(item) then break
    process(item)
}
```

---

## 5. 函数

### 5.1 函数定义

```ebnf
function_decl = [ "public" ] "function" identifier [ type_parameters ]
              "(" [ param_list ] ")" [ "returns" type ] [ "requires" effect_list ]
              ( block | "=" expression ) ;

param_list = param { "," param } ;
param = identifier ":" type [ default_value ] ;
default_value = "=" expression ;
effect_list = identifier { "," identifier } ;
type_parameters = "of" identifier { "," identifier } ;
```

```x
// 基本函数
function greet(name: String) returns String {
    return "Hello, " + name + "!"
}

// 单表达式函数
function square(x: Integer) returns Integer = x * x

// 数学风格（简洁）
function f(x) returns Integer = x * x

// 多返回值
function divide_and_remainder(a: Integer, b: Integer) returns (Integer, Integer) {
    return (a / b, a % b)
}

// 默认参数
function greet_person(name: String, greeting: String = "Hello") returns String {
    greeting + ", " + name + "!"
}

// 调用时使用默认参数
define message = greet_person("Alice")  // "Hello, Alice!"
define custom = greet_person("Bob", "Hi")  // "Hi, Bob!"

// 泛型函数
function first of T(list: List of T) returns Optional of T {
    when list is {
        [] => None
        [x, ...] => Some(x)
    }
}

// 带效果要求
function read_config() returns String requires Io {
    needs Io.read_file("config.toml")
}
```

### 5.2 return 语句

```ebnf
return_expr = "return" [ expression ] ;
```

```x
function find_first_negative(numbers: List of Integer) returns Optional of Integer {
    for each n in numbers {
        if n < 0 then return Some(n)
    }
    None  // 最后一个表达式自动返回
}

// 单表达式返回
function absolute(x: Integer) returns Integer {
    if x < 0 then return -x
    x
}
```

### 5.3 方法定义

```ebnf
method_decl = "method" identifier [ type_parameters ] "(" [ param_list ] ")"
            [ "returns" type ] ( block | "=" expression ) ;
```

```x
define class Calculator {
    field mutable value: Integer = 0

    method add(n: Integer) returns Integer {
        self.value = self.value + n
        self.value
    }

    method reset() returns Unit {
        self.value = 0
    }
}
```

---

## 6. 类与接口

### 6.1 类定义

```ebnf
class_decl = [ "public" ] "define" "class" identifier [ type_parameters ] "{" { class_member } "}" ;

class_member = field_decl | method_decl | constructor_decl ;
field_decl = [ "public" ] "field" [ "mutable" ] identifier ":" type [ default_value ] ;
method_decl = "method" identifier [ type_parameters ] "(" [ param_list ] ")"
            [ "returns" type ] ( block | "=" expression ) ;
constructor_decl = "constructor" "(" [ param_list ] ")" block ;
```

```x
define class Person {
    field name: String
    field age: Integer
    field mutable score: Integer = 0

    constructor(name: String, age: Integer) {
        Self(name, age, 0)  // 调用主构造器初始化字段
    }

    method greet() returns String {
        "Hello, I'm " + self.name
    }

    method is_adult() returns Boolean = self.age >= 18

    method have_birthday() returns Unit {
        self.age = self.age + 1
        println("Happy birthday! Now " + self.age)
    }
}

// 使用
define alice = Person("Alice", 30)
define greeting = alice.greet()
alice.have_birthday()
```

### 6.2 trait 定义

```ebnf
trait_decl = [ "public" ] "define" "trait" identifier [ type_parameters ]
           [ "extends" trait_list ] "{" { trait_method } "}" ;

trait_list = type_name { "," type_name } ;

trait_method = "method" identifier "(" [ param_list ] ")" [ "returns" type ] ;
```

```x
// 基础 trait
define trait Printable {
    method to_string() returns String
}

// 泛型 trait
define trait Comparable of T {
    method compare(other: T) returns Integer
    method is_less_than(other: T) returns Boolean = self.compare(other) < 0
    method is_equal_to(other: T) returns Boolean = self.compare(other) == 0
}

// 多方法 trait
define trait Iterator of Item {
    method next() returns Optional of Item
    method has_next() returns Boolean
}

// trait 继承
define trait Showable extends Printable {
    method show() returns String = self.to_string()
}

// 多继承
define trait ComparableHashable extends Comparable, Hashable {
    method compare_hash(other: Self) returns Integer
}
```

### 6.3 implement 定义

```ebnf
implement_decl = "implement" [ type_parameters ] type_name "for" type_name
               [ "where" where_clause ] "{" { implement_method } "}" ;

implement_method = "method" identifier "(" [ param_list ] ")" [ "returns" type ]
                 ( block | "=" expression ) ;

where_clause = where_constraint { "," where_constraint } ;
where_constraint = identifier ":" type_name ;
```

```x
implement Printable for Person {
    method to_string() returns String {
        "Person(name: " + self.name + ", age: " + self.age + ")"
    }
}

implement Comparable of Integer for Integer {
    method compare(other: Integer) returns Integer = self - other
}

// 泛型实现（带约束）
implement of T Printable for List of T where T: Printable {
    method to_string() returns String {
        "[" + self.map(x => x.to_string()).join(", ") + "]"
    }
}

// 多 trait 实现
implement Printable for Integer {
    method to_string() returns String = Integer.to_string(self)
}

implement Comparable of Integer for Integer {
    method compare(other: Integer) returns Integer = self - other
}
```

---

## 7. 模式匹配

### 7.1 when 表达式

```ebnf
when_expr = "when" expression "is" "{" { when_arm } "}" ;
when_arm = pattern [ guard ] "=>" ( expression | block ) [ "," ] ;
guard = "if" expression ;
```

```x
// 基本匹配
define description = when score is {
    100 => "perfect"
    90 | 91 | 92 | 93 | 94 | 95 | 96 | 97 | 98 | 99 => "excellent"
    n if n >= 80 => "good"
    n if n >= 60 => "passed"
    otherwise => "failed"
}

// 带块体
when status is {
    "success" => {
        define data = fetch_data()
        process(data)
        save(data)
    }
    "error" => {
        log_error()
        notify_admin()
    }
    otherwise => println("unknown status")
}
```

### 7.2 模式语法

```ebnf
pattern = literal_pattern
        | identifier_pattern
        | wildcard_pattern
        | tuple_pattern
        | list_pattern
        | constructor_pattern
        | range_pattern
        | or_pattern
        | type_pattern ;

literal_pattern = integer_literal | float_literal | boolean_literal | string_literal | char_literal ;
identifier_pattern = identifier [ "@" pattern ] ;
wildcard_pattern = "_" | "otherwise" ;
tuple_pattern = "(" [ pattern { "," pattern } ] ")" ;
list_pattern = "[" [ pattern_list ] "]" ;
pattern_list = pattern { "," pattern } [ "," spread_pattern ] | spread_pattern ;
spread_pattern = "..." identifier ;
constructor_pattern = type_name "(" [ pattern_list ] ")" ;
range_pattern = literal_pattern ".." literal_pattern ;
or_pattern = pattern "|" pattern ;
type_pattern = identifier "as" type_name ;  (* 使用 "as" 关键字避免与字典字面量混淆 *)
```

> **设计说明**：类型模式使用 `identifier as TypeName` 而非 `identifier: TypeName`，因为冒号在字典字面量和命名参数中已有其他含义，会产生解析歧义。

```x
// 字面量模式
define describe = when number is {
    0 => "zero"
    1 => "one"
    2 => "two"
    otherwise => "many"
}

// 元组解构
define locate = when point is {
    (0, 0) => "at origin"
    (x, 0) => "on x-axis at x=" + x
    (0, y) => "on y-axis at y=" + y
    (x, y) => "at (" + x + ", " + y + ")"
}

// 列表模式
define analyze = when list is {
    [] => "empty list"
    [x] => "single element: " + x
    [first, second] => "two elements: " + first + " and " + second
    [first, ...rest] => "first is " + first + ", rest has " + rest.length()
}

// 构造器模式
define handle_result = when result is {
    Success(value) => "got: " + value
    Failure(error) => "error: " + error
}

// 范围模式
define grade_level = when score is {
    90..100 => "A"
    80..89 => "B"
    70..79 => "C"
    60..69 => "D"
    otherwise => "F"
}

// 或模式
define classify = when character is {
    'a' | 'e' | 'i' | 'o' | 'u' => "vowel"
    'A' | 'E' | 'I' | 'O' | 'U' => "capital vowel"
    otherwise => "consonant"
}

// 类型模式 - 使用 "as" 关键字
define describe_value = when value is {
    s as String => "a string: " + s
    n as Integer => "an integer: " + n
    b as Boolean => "a boolean: " + b
    otherwise => "something else"
}

// 嵌套模式
define nested = when container is {
    Some((x, y)) => "contains pair (" + x + ", " + y + ")"
    Some(single) => "contains single value"
    None => "is empty"
}
```

### 7.3 穷尽检查

编译器确保 `when` 表达式覆盖所有可能的情况。

```x
define enum Direction { North, South, East, West }

// 编译器会检查所有变体
function to_string(dir: Direction) returns String {
    when dir is {
        Direction.North => "N"
        Direction.South => "S"
        Direction.East => "E"
        // 编译错误：缺少 Direction.West，或使用 otherwise
    }
}

// 正确写法
function to_string_complete(dir: Direction) returns String {
    when dir is {
        Direction.North => "N"
        Direction.South => "S"
        Direction.East => "E"
        Direction.West => "W"
    }
}

// 或使用 otherwise
function to_string_safe(dir: Direction) returns String {
    when dir is {
        Direction.North => "North"
        otherwise => "Not North"
    }
}
```

---

## 8. 效果系统

### 8.1 效果声明

```ebnf
effect_decl = "define" "effect" identifier [ type_parameters ] "{" { effect_operation } "}" ;
effect_operation = "operation" identifier "(" [ param_list ] ")" "returns" type ;
```

```x
// 基础效果
define effect Io {
    operation read_file(path: String) returns String
    operation write_file(path: String, content: String) returns Unit
    operation delete_file(path: String) returns Unit
}

// 泛型效果
define effect State of S {
    operation get() returns S
    operation set(value: S) returns Unit
    operation update(f: function from (S) returns S) returns Unit
}

// 多操作效果
define effect Database {
    operation query(sql: String) returns List of Row
    operation execute(sql: String) returns Integer
    operation begin_transaction() returns Unit
    operation commit() returns Unit
    operation rollback() returns Unit
}
```

### 8.2 效果使用

```ebnf
effect_constraint = "requires" effect_list ;
effect_statement = "needs" effect_call ;
effect_call = identifier "." identifier "(" [ argument_list ] ")" ;
```

```x
// 声明函数需要的效果
function read_config() returns String requires Io {
    needs Io.read_file("config.toml")
}

// 多效果
function fetch_and_save(url: String, path: String) returns Unit requires Io, Network {
    define data = needs Network.fetch(url)
    needs Io.write_file(path, data)
}

// 效果传播
function process_file(path: String) returns Result of Data with_error ParseError requires Io, Parse {
    define content = needs Io.read_file(path)
    needs Parse.parse(content)
}
```

### 8.3 given 处理器

```ebnf
effect_handler = "given" effect_list "{" { effect_impl } "}" ;
effect_impl = "operation" identifier "(" [ param_list ] ")" "returns" type block ;
```

```x
// 提供效果实现
function main() returns Unit {
    given Io {
        operation read_file(path: String) returns String {
            std.fs.read_text_file(path)
        }
        operation write_file(path: String, content: String) returns Unit {
            std.fs.write_text_file(path, content)
        }
        operation delete_file(path: String) returns Unit {
            std.fs.delete_file(path)
        }
    }

    // 在此作用域内，Io 效果可用
    define config = read_config()
    println(config)
}

// 多效果处理
function run_server() returns Unit {
    given Io, Network, Logger {
        // 所有效果的具体实现
        operation read_file(path: String) returns String { ... }
        operation send_request(url: String) returns String { ... }
        operation log(message: String) returns Unit { ... }
    }

    start_server()
}
```

---

## 9. 模块系统

### 9.1 模块声明

```ebnf
module_decl = "module" module_path ;
module_path = identifier { "." identifier } ;
```

```x
module math.utils

export constant PI = 3.14159
export function add(a: Integer, b: Integer) returns Integer = a + b
```

### 9.2 导入导出

```ebnf
import_decl = "use" module_path [ import_list ]
            | "from" module_path "use" import_list ;
import_list = "{" identifier { "," identifier } "}" ;

export_decl = "export" declaration ;
public_decl = "public" declaration ;
```

```x
// 完整模块导入
use std.collections

// 选择性导入
use std.collections { HashMap, HashSet, LinkedList }

// from-use 风格
from std.io use { read_file, write_file }

// 重命名导入
use std.collections.HashMap as HM

// 导出
export function add(a: Integer, b: Integer) returns Integer = a + b
export constant PI = 3.14159
export define type Point = (Float, Float)

// public 声明（同时导出）
public function api_endpoint() returns String {
    "Hello from API"
}

public define class Service {
    // ...
}
```

---

## 10. 内存模型

### 10.1 Perceus 引用计数

```ebnf
(* Perceus 操作由编译器自动插入，不是语法结构 *)
(* dup: 复制引用，增加引用计数 *)
(* drop: 释放引用，减少引用计数 *)
(* reuse: 当引用计数为 1 时，原地更新 *)
```

- **dup**：复制引用，增加引用计数
- **drop**：释放引用，减少引用计数
- **reuse**：当引用计数为 1 时，原地更新

```x
// 编译器自动管理内存
define a = [1, 2, 3]  // 引用计数 = 1
define b = a          // dup: 引用计数 = 2
// b 离开作用域: drop: 引用计数 = 1
// a 离开作用域: drop: 引用计数 = 0, 释放内存

// FBIP: 函数式但原地更新
define mutable list = [1, 2, 3]
list = list.push(4)  // 如果引用计数为 1，原地追加，无需分配
```

### 10.2 弱引用

```ebnf
weak_type = "weak" type ;
```

```x
define class Node {
    field value: Integer
    field next: Optional of Node
    field parent: weak Optional of Node  // 不参与引用计数
}

// 使用弱引用需要升级
function get_parent(node: Node) returns Optional of Node {
    when node.parent.upgrade() is {
        Some(parent) => Some(parent)
        None => None  // 父节点已被释放
    }
}
```

### 10.3 FBIP（Functionally But In Place）

```x
// 函数式风格代码，编译器优化为原地操作
function append of T(list: List of T, item: T) returns List of T {
    list.push(item)  // 如果是唯一引用，原地追加
}

// 编译器自动分析
function process() returns List of Integer {
    define mutable data = [1, 2, 3]
    data = data.push(4)   // 原地更新
    data = data.push(5)   // 原地更新
    data                  // 最终 [1, 2, 3, 4, 5]，无额外内存分配
}
```

---

## 11. 错误处理

### 11.1 Optional 用法

```x
function find_user(users: List of User, id: Integer) returns Optional of User {
    users.filter(u => u.id == id).first()
}

// 模式匹配处理
when find_user(users, 42) is {
    Some(user) => println("Found: " + user.name)
    None => println("Not found")
}

// 可选链
define email = user?.profile?.email

// 空合并运算符
define name = user?.name ?? "anonymous"
define timeout = config?.timeout ?? 30
```

### 11.2 Result 用法

```x
function read_file(path: String) returns Result of String with_error IoError {
    // 尝试读取文件
}

// ? 运算符传播错误
function load_config() returns Result of Config with_error IoError {
    define content = read_file("config.toml")?
    parse_config(content)?
}

// 链式处理
function process_file(path: String) returns Result of Data with_error Error {
    read_file(path)?
    |> parse?
    |> validate?
}

// 模式匹配处理
when read_file("data.txt") is {
    Success(content) => {
        println("Read " + content.length() + " bytes")
        process(content)
    }
    Failure(error) => {
        log_error(error)
        use_default()
    }
}
```

### 11.3 try-catch

```ebnf
try_expr = "try" block [ "catch" "(" identifier ")" block ] [ "finally" block ] ;
throw_expr = "throw" expression ;
```

```x
// try-catch 作为控制流（非异常机制）
function risky_operation() returns Result of Integer with_error Error {
    define result = try {
        might_fail()
    } catch (e) {
        return Failure(e)
    }
    Success(result)
}

// with finally
function with_resource() returns Unit {
    define resource = acquire_resource()
    try {
        use_resource(resource)
    } finally {
        release_resource(resource)
    }
}

// throw 用于效果系统
function validate(input: String) returns String requires Error {
    if input == "" then throw EmptyInputError
    if input.length() > 100 then throw InputTooLongError
    input
}
```

---

## 12. 并发

### 12.1 async/await

```ebnf
async_function = "async" "function" identifier "(" [ param_list ] ")" [ "returns" type ]
               ( block | "=" expression ) ;
await_expr = "await" expression ;
```

```x
// 异步函数
async function fetch_data(url: String) returns Result of String with_error NetworkError {
    define response = await http_get(url)
    Success(response.body)
}

// 多个异步操作
async function fetch_all(urls: List of String) returns List of String {
    define results = await Promise.all(urls.map(fetch_data))
    results
}

// async 块
define future = async {
    define a = await fetch("url_a")
    define b = await fetch("url_b")
    a + b
}
```

### 12.2 并发组合

```ebnf
concurrently_expr = "concurrently" block ;
race_expr = "race" block ;
```

```x
// concurrently: 并发执行所有任务，等待全部完成
define results = concurrently {
    fetch_from_server_a()
    fetch_from_server_b()
    fetch_from_server_c()
}
// results = (result_a, result_b, result_c)

// race: 竞态，返回最先完成的
define winner = race {
    fetch_from_cache()
    fetch_from_database()
    fetch_from_remote()
}
// winner = 最先返回的结果

// 实际应用
async function fetch_with_fallback(url: String) returns String {
    race {
        fetch_from_primary(url)
        fetch_from_backup(url)
    }
}
```

### 12.3 原子操作

```ebnf
atomic_expr = "atomic" expression ;
atomic_block = "atomic" block ;
retry_statement = "retry" [ integer_literal ] "times" block ;
```

```x
// 原子变量
define mutable counter = atomic 0

// 原子操作块
atomic {
    counter = counter + 1
}

// 比较并交换
function increment_if_positive(c: Atomic of Integer) returns Boolean {
    atomic {
        define current = c.load()
        if current > 0 then {
            c.store(current + 1)
            true
        } else {
            false
        }
    }
}

// 重试机制
function with_retry of T(operation: function from () returns T) returns T {
    retry 3 times {
        define result = operation()
        if result.is_success() then return result
    }
}
```

---

## 13. 运算符完整列表

```ebnf
operator = arithmetic_op | comparison_op | logical_op | other_op ;

arithmetic_op = "+" | "-" | "*" | "/" | "%" ;
comparison_op = "==" | "!=" | "<" | ">" | "<=" | ">=" ;
logical_op = "and" | "or" | "not" ;
other_op = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "?" | "??" | "|>" | ".." ;
```

| 类别 | 运算符 | 描述 | 自然语言替代 |
|------|--------|------|-------------|
| 算术 | `+ - * / %` | 加减乘除取模 | - |
| 比较 | `== != < > <= >=` | 相等和大小比较 | - |
| 逻辑 | `and or not` | 与或非 | 英文单词 |
| 赋值 | `= += -= *= /= %=` | 赋值和复合赋值 | - |
| 错误 | `? ??` | 错误传播、空合并 | - |
| 管道 | `\|>` | 管道 | - |
| 范围 | `..` | 范围 | - |

---

## 14. 保留字

```ebnf
reserved = "abstract" | "assert" | "become" | "box"
         | "do" | "dyn" | "final" | "macro" | "move"
         | "override" | "priv" | "pure" | "ref" | "sealed"
         | "sizeof" | "static" | "super" | "typeof" | "unsafe"
         | "virtual" | "yield" ;
```

以下标识符保留供未来使用：

`abstract`, `assert`, `become`, `box`, `do`, `dyn`, `final`, `macro`, `move`, `override`, `priv`, `pure`, `ref`, `sealed`, `sizeof`, `static`, `super`, `typeof`, `unsafe`, `virtual`, `yield`

---

## 15. 语法速查表

```x
// 变量
define x = 42
define mutable counter = 0
constant MAX = 100

// 函数
function add(a: Integer, b: Integer) returns Integer = a + b
define add = (a, b) => a + b

// 控制流
if condition then { } else { }
for each item in items { }
while condition { }
loop { if done then break }
when value is { pattern => result }

// 类型
define type Point = (Float, Float)
define enum Optional of T { Some(T), None }
define record Person { name: String, age: Integer }

// 类
define class Point { field x: Integer, field y: Integer }
define trait Printable { method to_string() returns String }

// 效果
function foo() returns T requires Io { needs Io.read_file("...") }

// 并发
async function fetch() returns String { await http_get(url) }
define results = concurrently { task_a(), task_b() }
define winner = race { fast(), slow() }
```

---

## 16. 与其他语言对比

| 特性 | X | Rust | Python | TypeScript |
|------|---|------|--------|------------|
| 变量声明 | `define x = 1` | `let x = 1` | `x = 1` | `const x = 1` |
| 函数声明 | `function f() returns T` | `fn f() -> T` | `def f() -> T` | `function f(): T` |
| 泛型语法 | `List of Integer` | `List<Integer>` | `list[int]` | `List<number>` |
| 模式匹配 | `when x is { }` | `match x { }` | `match x:` | - |
| 错误类型 | `Result of T with_error E` | `Result<T, E>` | 异常 | 异常 |
| 空安全 | `Optional of T` | `Option<T>` | `None` | `undefined` |
| 效果系统 | 有 | 无 | 无 | 无 |
| 内存管理 | Perceus | 所有权 | GC | GC |

---

## 17. 设计理念示例

### 17.1 自然语言风格对比

**传统语法**：
```
fn process<T: Clone>(list: Vec<T>) -> Option<T> {
    match list.first() {
        Some(x) => Some(x.clone()),
        None => None,
    }
}
```

**X 语言**：
```x
function process of T(list: List of T) returns Optional of T where T: Clone {
    when list.first() is {
        Some(x) => Some(x)
        None => None
    }
}
```

### 17.2 可读性优先

```x
// 定义一个用户服务
define class UserService {
    field database: Database
    field cache: Cache

    constructor(database: Database, cache: Cache) {
        Self(database, cache)
    }

    // 方法名和参数清晰表达意图
    method find_user_by_id(id: Integer) returns Optional of User {
        // 先查缓存
        when self.cache.get(id) is {
            Some(user) => Some(user)
            None => {
                // 缓存未命中，查询数据库
                define user = self.database.find_user(id)
                when user is {
                    Some(u) => {
                        self.cache.set(id, u)
                        Some(u)
                    }
                    None => None
                }
            }
        }
    }

    // 单表达式方法
    method user_exists(id: Integer) returns Boolean =
        self.find_user_by_id(id).is_some()
}
```

---

## 参考

- 完整规范：[spec/](spec/)
- 设计目标：[DESIGN_GOALS.md](DESIGN_GOALS.md)
- 示例程序：[examples/](examples/)

---

*最后更新：2026-03-26*
