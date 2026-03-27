# X 语言规范文档

> 本文档是 X 语言的正式语法与语义定义。所有语言行为、语法规则和类型系统规则以本文档为准。

---

## 设计哲学

**可读性第一**：X 语言的语法设计遵循"代码即文档"原则，让代码读起来像自然语言一样流畅。

### 核心原则

| 原则 | 示例 |
|------|------|
| **全称关键字** | `function` 而非 `fn`，`let` 而非 `val` |
| **类型小写** | `integer` 而非 `Integer`，`string` 而非 `String` |
| **英文连接词** | `when x is 0 then "zero"`、`for each item in list` |
| **动词开头** | `let x = 1`、`export function foo()` |
| **从句风格** | `when x is 0 then "zero"` |

---

## ASCII 符号用法

X 语言使用英文关键字提高可读性，同时保留必要的符号用于表达常见操作。以下是 ASCII 可打印符号（共 32 个）在 X 语言中的用法：

### 32 个 ASCII 符号一览

ASCII 可打印符号（不含字母和数字）共 32 个，在 X 语言中均有明确用途：

| 序号 | 符号 | 名称 | X 语言用途 |
|:----:|:----:|------|------------|
| 1 | `!` | 感叹号 | 前置：逻辑非 `!flag`；后置：宏调用 `println!`、`assert!` |
| 2 | `"` | 双引号 | 字符串字面量 `"hello"` |
| 3 | `#` | 井号 | 编译属性 `#[inline]`、`#[test]`、`#[derive]` |
| 4 | `$` | 美元符号 | 字符串插值 `"Hello, $name!"`、`"Result: ${x + y}"` |
| 5 | `%` | 百分号 | 取模运算 `a % b` |
| 6 | `&` | 和号 | 位与运算 `a & b`；`&&` 逻辑与 `a && b` |
| 7 | `'` | 单引号 | 字符字面量 `'A'`、`'中'` |
| 8 | `(` | 左圆括号 | 函数参数、元组、分组、模式开始 |
| 9 | `)` | 右圆括号 | 函数参数、元组、分组、模式结束 |
| 10 | `*` | 星号 | 乘法运算 `a * b` |
| 11 | `+` | 加号 | 加法运算 `a + b` |
| 12 | `,` | 逗号 | 分隔符（参数、列表元素等） |
| 13 | `-` | 连字符 | 减法运算、Lambda 箭头一部分 |
| 14 | `.` | 句点 | 成员访问、方法调用 |
| 15 | `/` | 斜杠 | 除法运算 `a / b` |
| 16 | `:` | 冒号 | 类型注解、命名参数、字典键值对 |
| 17 | `;` | 分号 | 语句分隔符（可选） |
| 18 | `<` | 小于号 | 比较运算、泛型参数开始 |
| 19 | `=` | 等号 | 赋值、默认值 |
| 20 | `>` | 大于号 | 比较运算、泛型参数结束 |
| 21 | `?` | 问号 | 错误传播 `?`、可选链 `?.`、空合并 `??` |
| 22 | `@` | at 符号 | 注解 `@Deprecated`、`@Override`、`@Inject` |
| 23 | `[` | 左方括号 | 列表字面量、列表模式、索引访问开始 |
| 24 | `\` | 反斜杠 | 转义字符 `\n`、`\t`、`\\` |
| 25 | `]` | 右方括号 | 列表字面量、列表模式、索引访问结束 |
| 26 | `^` | 脱字符 | 按位异或 `a ^ b` |
| 27 | `_` | 下划线 | 通配符模式、数字分隔符 `1_000_000` |
| 28 | `` ` `` | 反引号 | 原始字符串 `` `C:\path\to\file` ``、模板字符串、关键字标识符 `` `type` `` |
| 29 | `{` | 左花括号 | 代码块、字典字面量、when 表达式开始 |
| 30 | `|` | 竖线 | 位或运算 `a | b`、或模式 `1 | 2 | 3`、`||` 逻辑或 |
| 31 | `}` | 右花括号 | 代码块、字典字面量、when 表达式结束 |
| 32 | `~` | 波浪号 | 位取反 `~bits`、模式守卫 `when x is _ if ~x.is_empty()` |

### 符号组合

X 语言支持以下符号组合：

| 组合 | 用途 | 示例 |
|------|------|------|
| `->` | Lambda 箭头 | `x -> x * 2`、`(a, b) -> a + b` |
| `=>` | when 分支箭头 | `pattern => value` |
| `==` | 相等比较（或 `eq`） | `a == b` |
| `!=` | 不等比较（或 `ne`） | `a != b` |
| `<=` | 小于等于 | `a <= b` |
| `>=` | 大于等于 | `a >= b` |
| `&&` | 逻辑与（或 `and`） | `a && b` |
| `||` | 逻辑或（或 `or`） | `a || b` |
| `+=` | 加法赋值 | `x += 1` |
| `-=` | 减法赋值 | `x -= 1` |
| `*=` | 乘法赋值 | `x *= 2` |
| `/=` | 除法赋值 | `x /= 2` |
| `%=` | 取模赋值 | `x %= 3` |
| `|>` | 管道运算符 | `data |> process() |> output()` |
| `..` | 范围表达式 | `1..10`、`90..100` |
| `??` | 空合并运算符 | `x ?? default` |
| `?.` | 可选链访问 | `user?.name` |
| `//` | 单行注释 | `// 这是注释` |
| `/*` `*/` | 多行注释 | `/* 注释 */` |
| `#[` `]` | 编译属性 | `#[inline]`、`#[test]` |
| `${` `}` | 复杂插值表达式 | `"Sum: ${a + b}"` |

### 符号与关键字分工

X 语言支持符号与关键字两种写法，用户可根据偏好选择：

| 场景 | 符号写法 | 关键字写法 |
|------|----------|------------|
| **逻辑非** | `!flag` | `not flag` |
| **逻辑与** | `a && b` | `a and b` |
| **逻辑或** | `a || b` | `a or b` |
| **相等比较** | `a == b` | `a eq b` |
| **不等比较** | `a != b` | `a ne b` |
| **位运算** | `&`、`|`、`^`、`~` | — |
| **算术运算** | `+`、`-`、`*`、`/`、`%` | — |
| **比较运算** | `<`、`>`、`<=`、`>=` | — |
| **函数返回类型** | `->` | — |
| **Lambda** | `->` | — |
| **分支匹配** | `=>` | `when`、`is` |
| **泛型参数** | `<` `>` | — |
| **宏调用** | 后置 `!` | — |
| **注解** | `@` 前置 | — |

**推荐风格**：优先使用关键字写法以提高可读性，符号写法适用于简洁场景。

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
number = digit { digit } ;  (* 用于任意位宽整数类型 *)

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
let name = "X"
let snake_case = 1
let camelCase = 2
let _private = 3
let 变量名 = 42  // 支持 Unicode，但推荐英文
```

### 1.4 关键字

```ebnf
keyword = "let" | "mutable" | "constant"
        | "function" | "->" | "async" | "await" | "return" | "yield"
        | "if" | "then" | "else" | "when" | "is" | "as"
        | "for" | "each" | "in" | "while" | "loop" | "break" | "continue"
        | "type" | "class" | "trait" | "implement" | "enum" | "record" | "effect"
        | "module" | "import" | "export"
        | "public" | "private" | "static"
        | "try" | "catch" | "finally" | "throw" | "defer"
        | "with" | "perform" | "handle" | "operation" | "given" | "needs"
        | "concurrently" | "race" | "atomic" | "retry"
        | "and" | "or" | "not" | "extends" | "super" | "where"
        | "true" | "false" | "self" | "Self" | "constructor" | "unsafe" ;
```

| 类别 | 关键字 | 自然语言含义 |
|------|--------|-------------|
| 声明 | `let`, `mutable`, `constant` | 绑定、可变、常量 |
| 函数 | `function`, `async`, `await`, `return`, `yield` | 函数、异步、等待、返回、生成 |
| 控制流 | `if`, `then`, `else`, `when`, `is` | 如果、则、否则、当、是 |
| 模式匹配 | `when`, `is`, `as` | 当、是、类型转换 |
| 循环 | `for`, `each`, `in`, `while`, `loop`, `break`, `continue` | 对于、每个、在...中、当...时、循环、中断、继续 |
| 类型 | `type`, `class`, `trait`, `implement`, `enum`, `record`, `effect` | 类型、类、特质、实现、枚举、记录、效果 |
| 模块 | `module`, `import`, `export` | 模块、导入、导出 |
| 异常 | `try`, `catch`, `finally`, `throw`, `defer` | 尝试、捕获、最终、抛出、延迟执行 |
| 效果 | `with`, `perform`, `handle`, `operation`, `given`, `needs` | 声明效果、执行效果、处理效果、定义操作、提供上下文、使用效果 |
| 并发 | `concurrently`, `race`, `atomic`, `retry` | 并发地、竞争、原子、重试 |
| 逻辑运算 | `and`, `or`, `not` | 逻辑与、逻辑或、逻辑非 |
| 继承 | `extends`, `super` | 继承父类、父类引用 |
| 泛型约束 | `where` | 泛型类型约束 |
| 访问控制 | `public`, `private`, `static` | 公共成员、私有成员、静态成员 |
| 安全 | `unsafe` | 不安全代码块（FFI） |
| 字面量 | `true`, `false`, `self`, `Self`, `constructor` | 真、假、自身实例、自身类型、构造器 |

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
let age = 25
let hex = 0xFF
let octal = 0o755
let binary = 0b1010

// 浮点数
let pi = 3.14159
let scientific = 1.5e-10

// 布尔值
let active = true
let disabled = false

// 字符串
let greeting = "Hello, World!"
let multiline = "Line1\nLine2"
let escaped = "Tab:\tQuote:\""

// 字符
let grade = 'A'
let chinese = '中'

// 列表 - 方括号
let numbers = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]

// 字典 - 大括号，键值对用冒号
let scores = { Alice: 95, Bob: 87, Charlie: 92 }
let config = { host: "localhost", port: 8080 }

// 元组 - 圆括号，至少两个元素
let point = (10, 20)
let person = ("Alice", 30, true)

// 单元值 - 空圆括号
let nothing = ()
```

---

## 2. 类型系统

### 2.1 基本类型

X 语言提供丰富的内置基本类型，均为**值类型**，使用**小写**命名。类型系统涵盖整数、浮点数、布尔、字符、字符串等。

#### 2.1.1 整数类型

```ebnf
integer_type = "integer"                          (* 平台相关，默认 32 位 *)
             | signed_integer_type
             | unsigned_integer_type ;

signed_integer_type = "signed 8-bit integer"      (*  8 位有符号整数 *)
                     | "signed 16-bit integer"     (* 16 位有符号整数 *)
                     | "signed 32-bit integer"     (* 32 位有符号整数 *)
                     | "signed 64-bit integer"     (* 64 位有符号整数 *)
                     | "signed 128-bit integer"    (* 128 位有符号整数 *)
                     | "signed" number "-bit integer" ;  (* N 位有符号整数 *)

unsigned_integer_type = "unsigned 8-bit integer"    (*  8 位无符号整数 *)
                       | "unsigned 16-bit integer"   (* 16 位无符号整数 *)
                       | "unsigned 32-bit integer"   (* 32 位无符号整数 *)
                       | "unsigned 64-bit integer"   (* 64 位无符号整数 *)
                       | "unsigned 128-bit integer"  (* 128 位无符号整数 *)
                       | "unsigned" number "-bit integer" ;  (* N 位无符号整数 *)
```

| 类型 | 位宽 | 范围 | 示例 |
|------|------|------|------|
| `integer` | 32 位（默认） | -2,147,483,648 ~ 2,147,483,647 | `42` |
| `signed 8-bit integer` | 8 位 | -128 ~ 127 | `127i8` |
| `signed 16-bit integer` | 16 位 | -32,768 ~ 32,767 | `-1000i16` |
| `signed 32-bit integer` | 32 位 | ±2.1×10⁹ | `100_000i32` |
| `signed 64-bit integer` | 64 位 | ±9.2×10¹⁸ | `999_999_999_999i64` |
| `signed 128-bit integer` | 128 位 | ±1.7×10³⁸ | `170141183460469231731687303715884105727i128` |
| `signed N-bit integer` | N 位 | -2^(N-1) ~ 2^(N-1)-1 | `signed 7-bit integer` |
| `unsigned 8-bit integer` | 8 位 | 0 ~ 255 | `255u8` |
| `unsigned 16-bit integer` | 16 位 | 0 ~ 65,535 | `65535u16` |
| `unsigned 32-bit integer` | 32 位 | 0 ~ 4.3×10⁹ | `4_000_000_000u32` |
| `unsigned 64-bit integer` | 64 位 | 0 ~ 1.8×10¹⁹ | `18_446_744_073_709_551_615u64` |
| `unsigned 128-bit integer` | 128 位 | 0 ~ 3.4×10³⁸ | `340282366920938463463374607431768211455u128` |
| `unsigned N-bit integer` | N 位 | 0 ~ 2^N-1 | `unsigned 10-bit integer` |

```x
// 基本整数
let age: integer = 25
let count: signed 32-bit integer = 1_000_000

// 有符号整数
let small: signed 8-bit integer = -128
let medium: signed 16-bit integer = 32767
let large: signed 64-bit integer = 9_223_372_036_854_775_807i64

// 无符号整数
let byte: unsigned 8-bit integer = 255u8
let word: unsigned 16-bit integer = 65535u16
let dword: unsigned 32-bit integer = 4_000_000_000u32
let qword: unsigned 64-bit integer = 18_446_744_073_709_551_615u64

// 任意位宽整数
let bits7: signed 7-bit integer = -64
let bits10: unsigned 10-bit integer = 1023
```

#### 2.1.2 浮点类型

```ebnf
float_type = "float"                    (* 平台相关，默认 64 位 *)
           | "16-bit float"             (* 16 位浮点（半精度） *)
           | "32-bit float"             (* 32 位浮点（单精度） *)
           | "64-bit float"             (* 64 位浮点（双精度） *)
           | "128-bit float"            (* 128 位浮点（四精度） *)
           | "long float"               (* 扩展精度 *)
           | decimal_type ;

decimal_type = "32-bit decimal"         (* 32 位十进制浮点 *)
             | "64-bit decimal"         (* 64 位十进制浮点 *)
             | "128-bit decimal" ;      (* 128 位十进制浮点 *)
```

| 类型 | 位宽 | 精度 | 示例 |
|------|------|------|------|
| `float` | 64 位（默认） | 约 15-17 位有效数字 | `3.14159` |
| `16-bit float` | 16 位 | 约 3 位有效数字（半精度） | `1.5f16` |
| `32-bit float` | 32 位 | 约 6-9 位有效数字（单精度） | `3.14f32` |
| `64-bit float` | 64 位 | 约 15-17 位有效数字（双精度） | `3.141592653589793f64` |
| `128-bit float` | 128 位 | 约 33-36 位有效数字（四精度） | `3.14159265358979323846f128` |
| `long float` | 平台相关 | 扩展精度 | `3.14159L` |
| `32-bit decimal` | 32 位 | 7 位有效十进制数字 | `123.45d32` |
| `64-bit decimal` | 64 位 | 16 位有效十进制数字 | `123456789.12345678d64` |
| `128-bit decimal` | 128 位 | 34 位有效十进制数字 | `123456789012345678901234567890.1234d128` |

```x
// 基本浮点
let pi: float = 3.141592653589793
let e: 64-bit float = 2.718281828459045f64

// 半精度浮点（GPU 计算）
let half: 16-bit float = 1.5f16

// 单精度浮点
let single: 32-bit float = 3.14f32

// 四精度浮点（高精度计算）
let quad: 128-bit float = 3.14159265358979323846264338327950288f128

// 十进制浮点（金融计算）
let money: 64-bit decimal = 123456789.12d64
let scientific: 128-bit decimal = 1.234567890123456789012345678901234e100d128
```

#### 2.1.3 复数与虚数类型

```ebnf
complex_type = "complex float"              (* 默认双精度复数 *)
             | "complex 32-bit float"       (* 单精度复数 *)
             | "complex 64-bit float"       (* 双精度复数 *)
             | "complex 128-bit float" ;    (* 四精度复数 *)

imaginary_type = "imaginary float"           (* 默认双精度虚数 *)
               | "imaginary 32-bit float"    (* 单精度虚数 *)
               | "imaginary 64-bit float" ;  (* 双精度虚数 *)
```

| 类型 | 描述 | 示例 |
|------|------|------|
| `complex float` | 双精度复数 | `1.0 + 2.0i` |
| `complex 32-bit float` | 单精度复数 | `(1.0f32, 2.0f32)` |
| `complex 64-bit float` | 双精度复数 | `(1.0, 2.0)` |
| `complex 128-bit float` | 四精度复数 | `(1.0f128, 2.0f128)` |
| `imaginary float` | 双精度纯虚数 | `2.0i` |
| `imaginary 32-bit float` | 单精度纯虚数 | `2.0fi32` |
| `imaginary 64-bit float` | 双精度纯虚数 | `2.0fi64` |

```x
// 复数
let z: complex float = 1.0 + 2.0i
let z32: complex 32-bit float = (1.0f32, 2.0f32)

// 纯虚数
let im: imaginary float = 2.0i
let im64: imaginary 64-bit float = 3.0fi64

// 复数运算
let sum: complex float = z + (3.0 + 4.0i)
let product: complex float = z * (2.0 + 1.0i)
```

#### 2.1.4 布尔类型

```ebnf
boolean_type = "boolean" ;
```

| 类型 | 描述 | 示例 |
|------|------|------|
| `boolean` | 布尔值 | `true` / `false` |

```x
let active: boolean = true
let disabled: boolean = false
let result: boolean = 5 > 3  // true
```

#### 2.1.5 字符类型

```ebnf
character_type = "character"           (* Unicode 码点，默认 UTF-32 *)
               | "utf-8 character"     (* UTF-8 编码字符 *)
               | "utf-16 character"    (* UTF-16 编码字符 *)
               | "utf-32 character" ;  (* UTF-32 编码字符 *)
```

| 类型 | 描述 | 大小 | 示例 |
|------|------|------|------|
| `character` | Unicode 码点 | 32 位 | `'A'`、`'中'`、`'😀'` |
| `utf-8 character` | UTF-8 编码 | 1-4 字节 | `'A'` |
| `utf-16 character` | UTF-16 编码 | 2 或 4 字节 | `'A'` |
| `utf-32 character` | UTF-32 编码 | 4 字节 | `'A'` |

```x
let letter: character = 'A'
let chinese: character = '中'
let emoji: character = '😀'

// 特定编码
let utf8_char: utf-8 character = 'A'
let utf16_char: utf-16 character = '中'
```

#### 2.1.6 字符串类型

```ebnf
string_type = "string"           (* UTF-8 字符串 *)
            | "utf-8 string"     (* 明确 UTF-8 编码 *)
            | "utf-16 string"    (* UTF-16 字符串 *)
            | "utf-32 string" ;  (* UTF-32 字符串 *)
```

| 类型 | 描述 | 编码 | 示例 |
|------|------|------|------|
| `string` | 字符串 | UTF-8 | `"Hello"` |
| `utf-8 string` | UTF-8 字符串 | UTF-8 | `"你好"` |
| `utf-16 string` | UTF-16 字符串 | UTF-16 | `"世界"` |
| `utf-32 string` | UTF-32 字符串 | UTF-32 | `"🌍"` |

```x
let name: string = "X Language"
let greeting: string = "Hello, 世界! 🌍"

// 字符串插值
let message: string = "Hello, $name!"

// 多行字符串
let multiline: string = """
    This is a
    multi-line string.
    """

// 原始字符串（不转义）
let path: string = `C:\Users\Documents\file.txt`
```

#### 2.1.7 特殊类型

```ebnf
special_type = "unit"       (* 空值，无返回值 *)
             | "nothing"    (* 永不返回，底类型 *)
             | "void" ;     (* C FFI 兼容的无类型 *)
```

| 类型 | 描述 | 示例 |
|------|------|------|
| `unit` | 空值，表示无有意义的返回值 | `()` |
| `nothing` | 永不返回（底类型），用于 `panic()`、无限循环等 | `panic("error")` |
| `void` | C FFI 兼容的无类型 | `foreign function foo() -> void` |

```x
// unit 类型
let result: unit = println("Hello")

// nothing 类型（永不返回）
function panic(message: string) -> nothing {
    // 永不返回
}

// 底类型用于类型推断
function fail() -> nothing = panic("error")

function compute(x: integer) -> integer {
    when x is {
        0 => 0
        _ => fail()  // nothing 是所有类型的子类型，这里返回 integer
    }
}
```

#### 2.1.8 类型别名速查表

为方便使用，X 语言提供常用类型的简写别名：

| 完整类型 | 简写别名 | 描述 |
|----------|----------|------|
| `integer` | `int` | 默认整数 |
| `signed 8-bit integer` | `i8` | 8 位有符号整数 |
| `signed 16-bit integer` | `i16` | 16 位有符号整数 |
| `signed 32-bit integer` | `i32` | 32 位有符号整数 |
| `signed 64-bit integer` | `i64` | 64 位有符号整数 |
| `signed 128-bit integer` | `i128` | 128 位有符号整数 |
| `unsigned 8-bit integer` | `u8`, `byte` | 8 位无符号整数 |
| `unsigned 16-bit integer` | `u16` | 16 位无符号整数 |
| `unsigned 32-bit integer` | `u32` | 32 位无符号整数 |
| `unsigned 64-bit integer` | `u64` | 64 位无符号整数 |
| `unsigned 128-bit integer` | `u128` | 128 位无符号整数 |
| `float` | `f64` | 双精度浮点 |
| `16-bit float` | `f16` | 半精度浮点 |
| `32-bit float` | `f32` | 单精度浮点 |
| `128-bit float` | `f128` | 四精度浮点 |
| `boolean` | `bool` | 布尔类型 |
| `character` | `char` | 字符类型 |

```x
// 使用完整类型名
let a: signed 32-bit integer = 42
let b: unsigned 64-bit integer = 100u64
let c: 32-bit float = 3.14f32

// 使用简写别名
let a: i32 = 42
let b: u64 = 100u64
let c: f32 = 3.14f32

// 使用默认类型（类型推断）
let x = 42          // 推断为 integer (i32)
let y = 3.14        // 推断为 float (f64)
let z = true        // 推断为 boolean
let s = "hello"     // 推断为 string
let ch = 'A'        // 推断为 character
```

### 2.2 复合类型

```ebnf
type = type_expr [ "?" | "!" ] ;

type_expr = simple_type | compound_type | function_type | type_variable ;

simple_type = type_name [ type_arguments ] ;
type_name = identifier ;

(* 泛型参数使用尖括号 *)
type_arguments = "<" type { "," type } ">" ;

compound_type = tuple_type | list_type | map_type | optional_type | result_type ;
tuple_type = "(" [ type { "," type } ] ")" ;
list_type = "List" "<" type ">" ;
map_type = "Map" "<" type "," type ">" ;
optional_type = "Optional" "<" type ">" ;
result_type = "Result" "<" type "," type ">" ;
```

```x
// 列表类型
let numbers: List<integer> = [1, 2, 3]
let names: List<string> = ["Alice", "Bob"]

// 字典类型
let scores: Map<string, integer> = { Alice: 95, Bob: 87 }

// 元组类型
let point: (float, float) = (10.5, 20.5)
let person: (string, integer, boolean) = ("Alice", 30, true)

// 嵌套类型
let matrix: List<List<integer>> = [[1, 2], [3, 4]]

// Optional 和 Result 类型
let maybe: Optional<integer> = Some(42)
let outcome: Result<string, IoError> = Success("ok")
```

### 2.3 函数类型

```ebnf
function_type = "function" "(" [ param_type_list ] ")" "->" type ;
param_type_list = type { "," type } ;
```

```x
// 函数类型
let add: function (integer, integer) -> integer = (a, b) -> a + b
let greet: function (string) -> string = name -> "Hello, " + name

// 高阶函数
let apply: function (function (integer) -> integer, integer) -> integer = (f, x) -> f(x)
```

### 2.4 Optional 和 Result

```ebnf
optional_type = "Optional" "<" type ">" ;
result_type = "Result" "<" type "," type ">" ;
```

```x
// Optional<T> - 表示"可能有值"
type Optional<T> = Some(T) | None

// Result<T, E> - 表示"成功或失败"
type Result<T, E> = Success(T) | Failure(E)

// 使用示例
let maybe_number: Optional<integer> = Some(42)
let no_value: Optional<integer> = None

let success: Result<integer, string> = Success(100)
let failure: Result<integer, string> = Failure("error occurred")
```

### 2.5 代数数据类型

```ebnf
type_definition = enum_definition | record_definition | alias_definition ;

(* 枚举 *)
enum_definition = "enum" identifier [ type_parameters ] "{" { enum_variant } "}" ;
enum_variant = identifier
             | identifier "(" type_list ")"
             | identifier "{" field_list "}" ;

(* 记录 *)
record_definition = "record" identifier [ type_parameters ] "{" field_list "}" ;
field_list = field { "," field } ;
field = identifier ":" type [ default_value ] ;
default_value = "=" expression ;

(* 别名 *)
alias_definition = "type" identifier [ type_parameters ] "=" type ;
type_parameters = "<" identifier { "," identifier } ">" ;
```

```x
// 枚举（sum type）- "或"的关系
enum Color {
    Red
    Green
    Blue
    RGB(integer, integer, integer)
}

// 带泛型的枚举
enum Optional<T> {
    Some(T)
    None
}

enum Result<T, E> {
    Success(T)
    Failure(E)
}

// 记录（product type）- "积"的关系
record Person {
    name: string
    age: integer
    email: string = ""  // 默认值
}

// 类型别名
type UserId = integer
type Point = (float, float)
type Name = string
```

### 2.6 泛型

```ebnf
type_reference = type_name [ type_arguments ] ;
type_arguments = "<" type { "," type } ">" ;
```

```x
function first<T>(list: List<T>) -> Optional<T> {
    when list is {
        [] => None
        [x, ...] => Some(x)
    }
}

function identity<T>(value: T) -> T = value

function pair<A, B>(a: A, b: B) -> (A, B) = (a, b)
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
let sum = a + b
let diff = a - b
let product = a * b
let quotient = a / b
let remainder = a % b

// 逻辑运算 - 使用英文单词
let both_true = a and b
let either_true = a or b
let negated = not a

// 比较
let is_equal = a == b
let is_different = a != b
let is_less = a < b
let is_greater = a > b
let is_less_or_equal = a <= b
let is_greater_or_equal = a >= b

// 复合表达式
let complex = (a + b) * c and not (d > e)
```

### 3.3 管道运算符

```x
// 管道让操作顺序从左到右，更自然
let result = numbers
    |> filter(is_even)
    |> map(square)
    |> take(10)
    |> sum

// 等价于嵌套调用（从内到外，难以阅读）
let result_equivalent = sum(take(map(filter(numbers, is_even), square), 10))
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
let result = calculate(a, b, c)

// 命名参数，更清晰
let greeting = greet(name: "Alice", title: "Dr.")

// 方法调用
let upper_name = name.to_upper()
let sorted = numbers.sort()

// 链式调用
let processed = text.trim().to_lower().split(" ")

// 成员访问
let person_name = person.name
let deep_value = config.server.host

// 索引访问
let first = numbers[0]
let value = scores["Alice"]

// 可选链
let email = user?.profile?.email
```

### 3.5 Lambda 表达式

```ebnf
lambda_expr = lambda_params "->" ( expression | block ) ;
lambda_params = "(" [ lambda_param_list ] ")" | identifier ;
lambda_param_list = lambda_param { "," lambda_param } ;
lambda_param = identifier [ ":" type ] ;
```

```x
// 单参数简写
let square = x -> x * x

// 多参数
let add = (a, b) -> a + b

// 带类型注解
let multiply = (a: integer, b: integer) -> a * b

// 块体 lambda
let process = (x, y) -> {
    let sum = x + y
    let diff = x - y
    sum * diff
}

// 在高阶函数中使用
let doubled = numbers.map(x -> x * 2)
let evens = numbers.filter(x -> x % 2 == 0)
let sum = numbers.reduce((acc, x) -> acc + x, 0)
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
let grade = if score >= 90 then "A"
            else if score >= 80 then "B"
            else if score >= 70 then "C"
            else "F"

// 单行形式
let max = if a > b then a else b

// 嵌套
let description = if x > 0 then {
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
let description = when score is {
    100 => "perfect"
    n if n >= 90 => "excellent"
    n if n >= 60 => "passed"
    _ => "failed"
}

// 解构匹配
let location = when point is {
    (0, 0) => "origin"
    (x, 0) => "on x-axis at " + x
    (0, y) => "on y-axis at " + y
    (x, y) => "at (" + x + ", " + y + ")"
}

// 列表匹配
let head = when list is {
    [] => None
    [first, ...] => Some(first)
}

// 类型匹配
let info = when value is {
    s as string => "string: " + s
    n as integer => "number: " + n
    b as boolean => "boolean: " + b
    _ => "unknown type"
}
```

### 3.8 块表达式

```ebnf
block_expr = block ;
block = "{" { statement } [ expression ] "}" ;
```

```x
// 块作为表达式
let result = {
    let x = 10
    let y = 20
    x + y  // 最后一个表达式作为返回值
}

// 带副作用
let processed = {
    let temp = calculate()
    log("calculated: " + temp)
    transform(temp)
}

// 用于控制流
let value = if condition then {
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
let some_value = Some(42)
let none_value = None
let success = Success("data")
let failure = Failure("error message")

// 构造记录
let person = Person(name: "Alice", age: 30)

// 构造带位置参数的变体
let color = Color.RGB(255, 128, 0)
```

---

## 4. 语句与声明

### 4.1 变量绑定

```ebnf
let_statement = "let" [ "mutable" | "constant" ] identifier [ ":" type ] "=" expression ;
(* 不带修饰符: 不可变绑定，编译器推断是编译期常量还是运行期值 *)
(* 带 mutable: 可变变量 *)
(* 带 constant: 编译期常量，必须能在编译期求值 *)
```

```x
// 不可变绑定（推荐，编译器自动推断编译期/运行期）
let name = "X Language"
let age: integer = 25
let MAX_SIZE = 1024        // 编译期推断为常量
let PI = 3.14159           // 编译期推断为常量

// 可变变量
let mutable counter = 0
counter = counter + 1

// 编译期常量（显式声明）
let constant BUFFER_SIZE = 4096
let constant VERSION = "1.0.0"
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
let action = if danger_level > 5 then "evacuate" else "stay calm"
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
    let input = read_input()
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
function_decl = "function" identifier [ type_parameters ]
              "(" [ param_list ] ")" [ "->" type ] [ "requires" effect_list ]
              ( block | "=" expression ) ;

param_list = param { "," param } ;
param = identifier ":" type [ default_value ] ;
default_value = "=" expression ;
effect_list = identifier { "," identifier } ;
type_parameters = "<" identifier { "," identifier } ">" ;
```

```x
// 基本函数
function greet(name: string) -> string {
    return "Hello, " + name + "!"
}

// 单表达式函数
function square(x: integer) -> integer = x * x

// 数学风格（简洁）
function f(x) -> integer = x * x

// 多返回值
function divide_and_remainder(a: integer, b: integer) -> (integer, integer) {
    return (a / b, a % b)
}

// 默认参数
function greet_person(name: string, greeting: string = "Hello") -> string {
    greeting + ", " + name + "!"
}

// 调用时使用默认参数
let message = greet_person("Alice")  // "Hello, Alice!"
let custom = greet_person("Bob", "Hi")  // "Hi, Bob!"

// 泛型函数
function first<T>(list: List<T>) -> Optional<T> {
    when list is {
        [] => None
        [x, ...] => Some(x)
    }
}

// 带效果要求
function read_config() -> string requires Io {
    needs Io.read_file("config.toml")
}
```

### 5.2 return 语句

```ebnf
return_expr = "return" [ expression ] ;
```

```x
function find_first_negative(numbers: List<integer>) -> Optional<integer> {
    for each n in numbers {
        if n < 0 then return Some(n)
    }
    None  // 最后一个表达式自动返回
}

// 单表达式返回
function absolute(x: integer) -> integer {
    if x < 0 then return -x
    x
}
```

### 5.3 yield 生成器

`yield` 关键字用于创建生成器函数，可以逐个产出值而不需要一次性生成所有结果。生成器使用 `Generator<T>` 类型表示。

```ebnf
yield_stmt = "yield" expression ;
generator_type = "Generator" "<" type ">" ;
```

```x
// 基本生成器
function count_up(max: integer) -> Generator<integer> {
    let mutable i = 0
    while i < max {
        yield i
        i = i + 1
    }
}

// 使用生成器
for each n in count_up(5) {
    println(n)  // 输出: 0, 1, 2, 3, 4
}

// 斐波那契数列生成器
function fibonacci() -> Generator<integer> {
    let mutable a = 0
    let mutable b = 1
    loop {
        yield a
        let temp = a
        a = b
        b = temp + b
    }
}

// 取前 10 个斐波那契数
let fibs = fibonacci().take(10).to_list()
// fibs = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]

// 无限序列生成器
function naturals() -> Generator<integer> {
    let mutable n = 0
    loop {
        yield n
        n = n + 1
    }
}

// 筛选偶数
function even_numbers() -> Generator<integer> {
    for each n in naturals() {
        if n % 2 == 0 then yield n
    }
}

// 生成器组合
function squares(gen: Generator<integer>) -> Generator<integer> {
    for each n in gen {
        yield n * n
    }
}

// 链式调用
let result = naturals()
    |> filter(x -> x % 3 == 0)  // 筛选 3 的倍数
    |> map(x -> x * x)           // 平方
    |> take(5)                   // 取前 5 个
    |> to_list()
// result = [0, 9, 36, 81, 144]
```

**设计说明**：生成器是一种惰性求值的数据结构，只在需要时计算下一个值。`yield` 关键字被 Python、JavaScript、Scala、C# 等语言广泛采用，是处理无限序列和大数据集的重要工具。

### 5.4 方法定义

```ebnf
method_decl = identifier [ type_parameters ] "(" [ param_list ] ")"
            [ "->" type ] ( block | "=" expression ) ;
```

```x
class Calculator {
    mutable value: integer = 0

    add(n: integer) -> integer {
        self.value = self.value + n
        self.value
    }

    reset() -> unit {
        self.value = 0
    }
}
```

---

## 6. 类与接口

### 6.1 类定义

```ebnf
class_decl = "class" identifier [ type_parameters ] "{" { class_member } "}" ;

class_member = property_decl | function_decl | constructor_decl ;
property_decl = [ "mutable" ] identifier ":" type [ default_value ] ;
function_decl = identifier [ type_parameters ] "(" [ param_list ] ")"
              [ "->" type ] ( block | "=" expression ) ;
constructor_decl = "constructor" "(" [ param_list ] ")" block ;
```

```x
class Person {
    name: string
    age: integer
    mutable score: integer = 0

    constructor(name: string, age: integer) {
        Self(name, age, 0)  // 调用主构造器初始化字段
    }

    greet() -> string {
        "Hello, I'm " + self.name
    }

    is_adult() -> boolean = self.age >= 18

    have_birthday() -> unit {
        self.age = self.age + 1
        println("Happy birthday! Now " + self.age)
    }
}

// 使用
let alice = Person("Alice", 30)
let greeting = alice.greet()
alice.have_birthday()
```

### 6.2 trait 定义

```ebnf
trait_decl = "trait" identifier [ type_parameters ]
           [ "extends" trait_list ] "{" { trait_method } "}" ;

trait_list = type_name { "," type_name } ;

trait_method = identifier "(" [ param_list ] ")" [ "->" type ] ;
```

```x
// 基础 trait
trait Printable {
    to_string() -> string
}

// 泛型 trait
trait Comparable<T> {
    compare(other: T) -> integer
    is_less_than(other: T) -> boolean = self.compare(other) < 0
    is_equal_to(other: T) -> boolean = self.compare(other) == 0
}

// 多方法 trait
trait Iterator<Item> {
    next() -> Optional<Item>
    has_next() -> boolean
}

// trait 继承
trait Showable extends Printable {
    show() -> string = self.to_string()
}

// 多继承
trait ComparableHashable extends Comparable, Hashable {
    compare_hash(other: Self) -> integer
}
```

### 6.3 implement 定义

```ebnf
implement_decl = "implement" [ type_parameters ] type_name "for" type_name
               [ "where" where_clause ] "{" { implement_method } "}" ;

implement_method = identifier "(" [ param_list ] ")" [ "->" type ]
                 ( block | "=" expression ) ;

where_clause = where_constraint { "," where_constraint } ;
where_constraint = identifier ":" type_name ;
```

```x
implement Printable for Person {
    to_string() -> string {
        "Person(name: " + self.name + ", age: " + self.age + ")"
    }
}

implement Comparable<integer> for integer {
    compare(other: integer) -> integer = self - other
}

// 泛型实现（带约束）
implement<T> Printable for List<T> where T: Printable {
    to_string() -> string {
        "[" + self.map(x -> x.to_string()).join(", ") + "]"
    }
}

// 多 trait 实现
implement Printable for integer {
    to_string() -> string = integer.to_string(self)
}

implement Comparable<integer> for integer {
    compare(other: integer) -> integer = self - other
}
```

---

## 7. 模式匹配

### 7.1 when 表达式

`when` 表达式的语法已在 [3.7 when 表达式](#37-when-表达式模式匹配)中介绍。本节重点介绍更复杂的匹配模式。

```x
// 或模式 - 匹配多个值
let description = when score is {
    100 => "perfect"
    90 | 91 | 92 | 93 | 94 | 95 | 96 | 97 | 98 | 99 => "excellent"
    n if n >= 80 => "good"
    n if n >= 60 => "passed"
    _ => "failed"
}

// 带块体的分支
when status is {
    "success" => {
        let data = fetch_data()
        process(data)
        save(data)
    }
    "error" => {
        log_error()
        notify_admin()
    }
    _ => println("unknown status")
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
wildcard_pattern = "_" ;
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
let describe = when number is {
    0 => "zero"
    1 => "one"
    2 => "two"
    _ => "many"
}

// 元组解构
let locate = when point is {
    (0, 0) => "at origin"
    (x, 0) => "on x-axis at x=" + x
    (0, y) => "on y-axis at y=" + y
    (x, y) => "at (" + x + ", " + y + ")"
}

// 列表模式
let analyze = when list is {
    [] => "empty list"
    [x] => "single element: " + x
    [first, second] => "two elements: " + first + " and " + second
    [first, ...rest] => "first is " + first + ", rest has " + rest.length()
}

// 构造器模式
let handle_result = when result is {
    Success(value) => "got: " + value
    Failure(error) => "error: " + error
}

// 范围模式
let grade_level = when score is {
    90..100 => "A"
    80..89 => "B"
    70..79 => "C"
    60..69 => "D"
    _ => "F"
}

// 或模式
let classify = when character is {
    'a' | 'e' | 'i' | 'o' | 'u' => "vowel"
    'A' | 'E' | 'I' | 'O' | 'U' => "capital vowel"
    _ => "consonant"
}

// 类型模式 - 使用 "as" 关键字
let describe_value = when value is {
    s as string => "a string: " + s
    n as integer => "an integer: " + n
    b as boolean => "a boolean: " + b
    _ => "something else"
}

// 嵌套模式
let nested = when container is {
    Some((x, y)) => "contains pair (" + x + ", " + y + ")"
    Some(single) => "contains single value"
    None => "is empty"
}
```

### 7.3 穷尽检查

编译器确保 `when` 表达式覆盖所有可能的情况。

```x
enum Direction { North, South, East, West }

// 编译器会检查所有变体
function to_string(dir: Direction) -> string {
    when dir is {
        Direction.North => "N"
        Direction.South => "S"
        Direction.East => "E"
        // 编译错误：缺少 Direction.West，或使用 _
    }
}

// 正确写法
function to_string_complete(dir: Direction) -> string {
    when dir is {
        Direction.North => "N"
        Direction.South => "S"
        Direction.East => "E"
        Direction.West => "W"
    }
}

// 或使用通配符 _
function to_string_safe(dir: Direction) -> string {
    when dir is {
        Direction.North => "North"
        _ => "Not North"
    }
}
```

---

## 8. 效果系统

### 8.1 效果声明

```ebnf
effect_decl = "effect" identifier [ type_parameters ] "{" { effect_operation } "}" ;
effect_operation = "operation" identifier "(" [ param_list ] ")" "->" type ;
```

```x
// 基础效果
effect Io {
    operation read_file(path: string) -> string
    operation write_file(path: string, content: string) -> Unit
    operation delete_file(path: string) -> Unit
}

// 泛型效果
effect State<S> {
    operation get() -> S
    operation set(value: S) -> Unit
    operation update(f: function (S) -> S) -> Unit
}

// 多操作效果
effect Database {
    operation query(sql: string) -> List<Row>
    operation execute(sql: string) -> integer
    operation begin_transaction() -> Unit
    operation commit() -> Unit
    operation rollback() -> Unit
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
function read_config() -> string requires Io {
    needs Io.read_file("config.toml")
}

// 多效果
function fetch_and_save(url: string, path: string) -> unit requires Io, Network {
    let data = needs Network.fetch(url)
    needs Io.write_file(path, data)
}

// 效果传播
function process_file(path: string) -> Result<Data, ParseError> requires Io, Parse {
    let content = needs Io.read_file(path)
    needs Parse.parse(content)
}
```

### 8.3 given 处理器

```ebnf
effect_handler = "given" effect_list "{" { effect_impl } "}" ;
effect_impl = "operation" identifier "(" [ param_list ] ")" "->" type block ;
```

```x
// 提供效果实现
function main() -> unit {
    given Io {
        operation read_file(path: string) -> string {
            std.fs.read_text_file(path)
        }
        operation write_file(path: string, content: string) -> unit {
            std.fs.write_text_file(path, content)
        }
        operation delete_file(path: string) -> unit {
            std.fs.delete_file(path)
        }
    }

    // 在此作用域内，Io 效果可用
    let config = read_config()
    println(config)
}

// 多效果处理
function run_server() -> unit {
    given Io, Network, Logger {
        // 所有效果的具体实现
        operation read_file(path: string) -> string { ... }
        operation send_request(url: string) -> string { ... }
        operation log(message: string) -> unit { ... }
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

export let constant PI = 3.14159
export function add(a: integer, b: integer) -> integer = a + b
```

### 9.2 导入导出

```ebnf
import_decl = "import" module_path [ import_list ] ;
import_list = "{" identifier { "," identifier } "}" ;

export_decl = "export" declaration ;
```

```x
// 完整模块导入
import std.collections

// 选择性导入
import std.collections { HashMap, HashSet, LinkedList }

// 重命名导入
import std.collections.HashMap as HM

// 导出
export function add(a: integer, b: integer) -> integer = a + b
export let constant PI = 3.14159
export type Point = (float, float)
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
let a = [1, 2, 3]  // 引用计数 = 1
let b = a          // dup: 引用计数 = 2
// b 离开作用域: drop: 引用计数 = 1
// a 离开作用域: drop: 引用计数 = 0, 释放内存

// FBIP: 函数式但原地更新
let mutable list = [1, 2, 3]
list = list.push(4)  // 如果引用计数为 1，原地追加，无需分配
```

### 10.2 弱引用

```ebnf
weak_type = "weak" type ;
```

```x
class Node {
    value: integer
    next: Optional<Node>
    parent: weak Optional<Node>  // 不参与引用计数
}

// 使用弱引用需要升级
function get_parent(node: Node) -> Optional<Node> {
    when node.parent.upgrade() is {
        Some(parent) => Some(parent)
        None => None  // 父节点已被释放
    }
}
```

### 10.3 FBIP（Functionally But In Place）

```x
// 函数式风格代码，编译器优化为原地操作
function append<T>(list: List<T>, item: T) -> List<T> {
    list.push(item)  // 如果是唯一引用，原地追加
}

// 编译器自动分析
function process() -> List<integer> {
    let mutable data = [1, 2, 3]
    data = data.push(4)   // 原地更新
    data = data.push(5)   // 原地更新
    data                  // 最终 [1, 2, 3, 4, 5]，无额外内存分配
}
```

---

## 11. 错误处理

### 11.1 Optional 用法

```x
function find_user(users: List<User>, id: integer) -> Optional<User> {
    users.filter(u -> u.id == id).first()
}

// 模式匹配处理
when find_user(users, 42) is {
    Some(user) => println("Found: " + user.name)
    None => println("Not found")
}

// 可选链
let email = user?.profile?.email

// 空合并运算符
let name = user?.name ?? "anonymous"
let timeout = config?.timeout ?? 30
```

### 11.2 Result 用法

```x
function read_file(path: string) -> Result<String, IoError> {
    // 尝试读取文件
}

// ? 运算符传播错误
function load_config() -> Result<Config, IoError> {
    let content = read_file("config.toml")?
    parse_config(content)?
}

// 链式处理
function process_file(path: string) -> Result<Data, Error> {
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
function risky_operation() -> Result<integer, Error> {
    let result = try {
        might_fail()
    } catch (e) {
        return Failure(e)
    }
    Success(result)
}

// with finally
function with_resource() -> unit {
    let resource = acquire_resource()
    try {
        use_resource(resource)
    } finally {
        release_resource(resource)
    }
}

// throw 用于效果系统
function validate(input: string) -> string requires Error {
    if input == "" then throw EmptyInputError
    if input.length() > 100 then throw InputTooLongError
    input
}
```

### 11.4 defer 延迟执行

`defer` 关键字用于声明延迟执行的代码块，确保资源被正确释放。延迟执行的代码会在当前作用域结束时（函数返回、循环结束、或块结束时）按照"后进先出"（LIFO）顺序执行。

```ebnf
defer_stmt = "defer" ( expression | block ) ;
```

```x
// 基本用法：确保文件关闭
function read_config(path: string) -> Result<string, IoError> {
    let file = open_file(path)?
    defer file.close()  // 函数返回时自动执行

    let content = file.read_all()?
    Success(content)
}  // file.close() 在这里执行

// 多个 defer：后进先出
function process_data() -> unit {
    defer println("cleanup 3")  // 第3个注册，最后执行
    defer println("cleanup 2")  // 第2个注册，第2个执行
    defer println("cleanup 3")  // 第1个注册，最先执行
    println("processing...")
}
// 输出:
// processing...
// cleanup 1
// cleanup 2
// cleanup 3

// 使用 defer 管理锁
function update_shared_data(data: Data) -> unit {
    lock.acquire()
    defer lock.release()  // 确保锁被释放

    data.modify()
    // 即使 modify() 抛出异常，lock.release() 也会执行
}

// defer 与 try 配合
function safe_operation() -> Result<integer, Error> {
    let resource = acquire_resource()
    defer release_resource(resource)

    try {
        let result = risky_compute()?
        Success(result)
    } catch (e) {
        Failure(e)
    }
    // release_resource(resource) 在这里执行
}
```

**设计说明**：`defer` 借鉴了 Go 语言的设计，被 Swift、Zig、V、Odin 等现代语言广泛采纳。它提供了一种简洁的资源管理方式，比 `try-finally` 更直观，比 RAII（构造/析构）更灵活。

---

## 12. 并发

### 12.1 async/await

```ebnf
async_function = "async" "function" identifier "(" [ param_list ] ")" [ "->" type ]
               ( block | "=" expression ) ;
await_expr = "await" expression ;
```

```x
// 异步函数
async function fetch_data(url: string) -> Result<String, NetworkError> {
    let response = await http_get(url)
    Success(response.body)
}

// 多个异步操作
async function fetch_all(urls: List<string>) -> List<string> {
    let results = await Promise.all(urls.map(fetch_data))
    results
}

// async 块
let future = async {
    let a = await fetch("url_a")
    let b = await fetch("url_b")
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
let results = concurrently {
    fetch_from_server_a()
    fetch_from_server_b()
    fetch_from_server_c()
}
// results = (result_a, result_b, result_c)

// race: 竞态，返回最先完成的
let winner = race {
    fetch_from_cache()
    fetch_from_database()
    fetch_from_remote()
}
// winner = 最先返回的结果

// 实际应用
async function fetch_with_fallback(url: string) -> string {
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
let mutable counter = atomic 0

// 原子操作块
atomic {
    counter = counter + 1
}

// 比较并交换
function increment_if_positive(c: Atomic<integer>) -> boolean {
    atomic {
        let current = c.load()
        if current > 0 then {
            c.store(current + 1)
            true
        } else {
            false
        }
    }
}

// 重试机制
function with_retry<T>(operation: function () -> T) -> T {
    retry 3 times {
        let result = operation()
        if result.is_success() then return result
    }
}
```

---

## 13. 运算符完整列表

```ebnf
operator = arithmetic_op | comparison_op | logical_op | other_op ;
logical_op = "&&" | "||" | "!" | "and" | "or" | "not" ;
comparison_op = "==" | "!=" | "<" | ">" | "<=" | ">=" ;
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

## 14. 关键字

### 14.1 关键字列表

以下标识符是语言关键字，不能用作标识符：

```ebnf
keyword = "let" | "mutable" | "constant"          (* 变量声明 *)
        | "function" | "async" | "await" | "return" | "yield" (* 函数与异步返回 *)
        | "if" | "then" | "else"                  (* 条件 *)
        | "when" | "is" | "as"                    (* 模式匹配与类型转换 *)
        | "for" | "each" | "in" | "while" | "loop" | "break" | "continue"  (* 循环 *)
        | "enum" | "record" | "type"              (* 类型定义 *)
        | "class" | "trait" | "effect"            (* 结构定义 *)
        | "implement" | "extends"                 (* 实现与继承 *)
        | "constructor"                           (* 类构造器 *)
        | "module" | "import" | "export"          (* 模块 *)
        | "public" | "private" | "static"         (* 访问控制与静态成员 *)
        | "try" | "catch" | "finally" | "throw" | "defer" (* 异常与延迟执行 *)
        | "concurrently" | "race" | "atomic" | "retry"  (* 并发 *)
        | "true" | "false"                        (* 布尔字面量 *)
        | "and" | "or" | "not"                    (* 逻辑运算 *)
        | "with" | "perform" | "handle" | "operation" | "given" | "needs" (* 效果系统 *)
        | "where"                                 (* 泛型约束 *)
        | "super"                                 (* 父类引用 *)
        | "self" | "Self" | "unsafe"              (* 自身引用与不安全代码 *)
        ;
```

---

## 15. 语法速查表

```x
// 变量
let x = 42
let mutable counter = 0
let MAX = 100

// 函数
function add(a: integer, b: integer) -> integer = a + b
let add = (a, b) -> a + b

// 控制流
if condition then { } else { }
for each item in items { }
while condition { }
loop { if done then break }
when value is { pattern => result }

// 生成器
function count_up(n: integer) -> Generator<integer> {
    let mutable i = 0
    while i < n { yield i; i = i + 1 }
}

// 延迟执行
function process_file(path: string) -> unit {
    let file = open(path)
    defer file.close()
    process(file)
}

// 类型
type Point = (float, float)
enum Optional<T> { Some(T), None }
record Person { name: string, age: integer }

// 类
class Point { x: integer, y: integer }
trait Printable { to_string() -> string }

// 效果
function foo() -> T requires Io { needs Io.read_file("...") }

// 并发
async function fetch() -> string { await http_get(url) }
let results = concurrently { task_a(), task_b() }
let winner = race { fast(), slow() }
```

---

## 16. 与其他语言对比

| 特性 | X | Rust | Python | TypeScript |
|------|---|------|--------|------------|
| 变量声明 | `let x = 1` | `let x = 1` | `x = 1` | `const x = 1` |
| 可变变量 | `let mutable x = 1` | `let mut x = 1` | `x = 1` | `let x = 1` |
| 函数声明 | `function f() -> T` | `fn f() -> T` | `def f() -> T` | `function f(): T` |
| 泛型语法 | `List<integer>` | `List<integer>` | `list[int]` | `List<number>` |
| 模式匹配 | `when x is { }` | `match x { }` | `match x:` | - |
| 错误类型 | `Result<T, E>` | `Result<T, E>` | 异常 | 异常 |
| 空安全 | `Optional<T>` | `Option<T>` | `None` | `undefined` |
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
function process<T>(list: List<T>) -> Optional<T> where T: Clone {
    when list.first() is {
        Some(x) => Some(x)
        None => None
    }
}
```

### 17.2 可读性优先

```x
// 定义一个用户服务
class UserService {
    database: Database
    cache: Cache

    constructor(database: Database, cache: Cache) {
        Self(database, cache)
    }

    // 方法名和参数清晰表达意图
    find_user_by_id(id: integer) -> Optional<User> {
        // 先查缓存
        when self.cache.get(id) is {
            Some(user) => Some(user)
            None => {
                // 缓存未命中，查询数据库
                let user = self.database.find_user(id)
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
    user_exists(id: integer) -> boolean =
        self.find_user_by_id(id).is_some()
}
```

---

## 参考

- 完整规范：[spec/](spec/)
- 设计目标：[DESIGN_GOALS.md](DESIGN_GOALS.md)
- 示例程序：[examples/](examples/)

---

*最后更新：2026-03-27*
