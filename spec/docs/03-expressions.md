# 第3章 表达式

## 3.1 语法

### 表达式语法定义

```
Expression → Literal
           | Variable
           | MemberAccess
           | FunctionCall
           | BinaryOperation
           | UnaryOperation
           | IfExpression
           | MatchExpression
           | Lambda
           | Array
           | Dictionary
           | Record
           | Range
           | Pipe
           | ErrorPropagation
           | OptionalChain
           | DefaultValue
           | TypeCheck
           | TypeCast
           | Parenthesized
```

### 优先级与结合性

从高到低排列：

```
优先级   运算符                        结合性    说明
─────────────────────────────────────────────────────────────
 1       . () []                       左        成员访问、函数调用、索引
 2       - not ~                       右（前缀） 一元取负、逻辑非、位取反
 3       * / %                         左        乘法、除法、取模
 4       + -                           左        加法、减法
 5       << >>                         左        位左移、位右移
 6       &                             左        位与
 7       ^                             左        位异或
 8       |                             左        位或
 9       .. ..=                        无        范围（左闭右开、左闭右闭）
10       < > <= >= is as               左        比较、类型检查、类型转换
11       == !=                         左        相等、不等
12       and                           左        逻辑与（短路求值）
13       or                            左        逻辑或（短路求值）
14       ?? ?.                         左        默认值、可选链
15       |>                            左        管道
16       = += -= *= /= %= ^=          右        赋值
```

---

## 3.2 字面量

### 整数字面量

```
Literal ::= IntegerLiteral

IntegerLiteral ::= DecimalLiteral | HexLiteral | OctalLiteral | BinaryLiteral
DecimalLiteral ::= [0-9] ([0-9_])*
HexLiteral     ::= '0x' [0-9a-fA-F_]+
OctalLiteral   ::= '0o' [0-7_]+
BinaryLiteral  ::= '0b' [01_]+
```

类型推断为 `Integer`。

```
⟦n⟧ = n  where n ∈ ℤ
```

```x
let decimal = 42
let hex = 0xFF
let octal = 0o77
let binary = 0b1010_0110
let with_separator = 1_000_000
```

### 浮点数字面量

```
Literal ::= FloatLiteral
FloatLiteral ::= [0-9]+ '.' [0-9]+ ([eE] [+-]? [0-9]+)?
```

类型推断为 `Float`。

```
⟦f⟧ = f  where f ∈ ℝ
```

```x
let pi = 3.14159
let scientific = 1.0e-10
let negative_exp = 2.5E+3
```

### 布尔字面量

```
Literal ::= 'true' | 'false'
```

类型推断为 `Boolean`。

```
⟦true⟧ = true
⟦false⟧ = false
```

```x
let flag = true
let check = false
```

### 字符串字面量

```
Literal ::= '"' StringContent* '"'
StringContent ::= EscapeSequence | InterpolationExpr | Character
InterpolationExpr ::= '${' Expression '}'
EscapeSequence ::= '\\' ('n' | 'r' | 't' | '\\' | '"' | '0' | 'u{' HexDigit+ '}')
```

类型推断为 `String`。

```
⟦"s"⟧ = s  where s ∈ Σ*
```

```x
let greeting = "Hello, World!"
let interpolated = "1 + 2 = ${1 + 2}"
let multiline = "line1\nline2"
let escaped = "quote: \" backslash: \\"
```

### 字符字面量

```
Literal ::= '\'' Character '\''
```

类型推断为 `Character`。

```x
let ch = 'A'
let newline = '\n'
let unicode = '\u{1F389}'
```

### 字面量求值规则

```
⟦l⟧ᵍ = v  where v is the value of literal l

Γ ⊢ n : Integer         (整数字面量)
Γ ⊢ f : Float           (浮点数字面量)
Γ ⊢ true : Boolean      (布尔字面量)
Γ ⊢ false : Boolean     (布尔字面量)
Γ ⊢ "s" : String        (字符串字面量)
Γ ⊢ 'c' : Character     (字符字面量)
```

> **注意**：X 语言没有 `null` 字面量。使用 `None` 表示"无值"，`None` 是 `Option<T>` 的构造器而非字面量。

---

## 3.3 变量

### 变量引用

```
Expression ::= Identifier
```

### 变量求值

```
⟦x⟧ᵍ = g(x)  where g is the environment

g : Identifier → Value
g(x) = v  if x is bound to v in g
```

### 自由变量

```
FV(x) = {x}
FV(e₁ op e₂) = FV(e₁) ∪ FV(e₂)
FV(function(x) => e) = FV(e) \ {x}
```

### 类型规则

```
x : T ∈ Γ
──────────
Γ ⊢ x : T
```

```x
let name = "Alice"
let length = name.length()    // 变量引用
```

---

## 3.4 函数调用

### 函数调用语法

```
Expression ::= Expression '(' (Expression (',' Expression)*)? ')'
```

支持尾随逗号。

### 函数调用求值

```
⟦f(e₁, ..., eₙ)⟧ᵍ = v
  where
    f' = ⟦f⟧ᵍ
    v₁ = ⟦e₁⟧ᵍ
    ...
    vₙ = ⟦eₙ⟧ᵍ
    v = apply(f', v₁, ..., vₙ)
```

### 函数应用

```
apply : (Value × Value*) → Value
apply(closure, v₁, ..., vₙ) = ⟦body⟧ᵍ[x₁ ↦ v₁, ..., xₙ ↦ vₙ]
  where closure = (x₁, ..., xₙ) → body, g
```

### 类型规则

```
Γ ⊢ f : (T₁, ..., Tₙ) -> R with Δ
Γ ⊢ e₁ : T₁  ...  Γ ⊢ eₙ : Tₙ
────────────────────────────────────
Γ ⊢ f(e₁, ..., eₙ) : R, Δ
```

```x
let result = add(1, 2)
let formatted = format("Hello, {}!", name)
let mapped = list.map(function(x) => x * 2)
```

---

## 3.5 二元运算

### 二元运算语法

```
Expression ::= Expression BinaryOp Expression

BinaryOp ::= ArithmeticOp | ComparisonOp | LogicalOp | BitwiseOp

ArithmeticOp  ::= '+' | '-' | '*' | '/' | '%'
ComparisonOp  ::= '==' | '!=' | '<' | '>' | '<=' | '>='
LogicalOp     ::= 'and' | 'or'
BitwiseOp     ::= '&' | '|' | '^' | '<<' | '>>'
```

### 算术运算

```
⟦e₁ + e₂⟧ᵍ = ⟦e₁⟧ᵍ + ⟦e₂⟧ᵍ
⟦e₁ - e₂⟧ᵍ = ⟦e₁⟧ᵍ - ⟦e₂⟧ᵍ
⟦e₁ * e₂⟧ᵍ = ⟦e₁⟧ᵍ × ⟦e₂⟧ᵍ
⟦e₁ / e₂⟧ᵍ = ⟦e₁⟧ᵍ ÷ ⟦e₂⟧ᵍ      (e₂ ≠ 0)
⟦e₁ % e₂⟧ᵍ = ⟦e₁⟧ᵍ mod ⟦e₂⟧ᵍ    (e₂ ≠ 0)
```

类型规则：

```
Γ ⊢ e₁ : Integer    Γ ⊢ e₂ : Integer
──────────────────────────────────────
Γ ⊢ e₁ + e₂ : Integer

Γ ⊢ e₁ : Float    Γ ⊢ e₂ : Float
──────────────────────────────────
Γ ⊢ e₁ + e₂ : Float
```

```x
let sum = 10 + 20           // Integer
let product = 3.14 * 2.0    // Float
let remainder = 17 % 5      // Integer
```

### 比较运算

```
⟦e₁ == e₂⟧ᵍ = (⟦e₁⟧ᵍ = ⟦e₂⟧ᵍ)
⟦e₁ != e₂⟧ᵍ = (⟦e₁⟧ᵍ ≠ ⟦e₂⟧ᵍ)
⟦e₁ < e₂⟧ᵍ  = (⟦e₁⟧ᵍ < ⟦e₂⟧ᵍ)
⟦e₁ > e₂⟧ᵍ  = (⟦e₁⟧ᵍ > ⟦e₂⟧ᵍ)
⟦e₁ <= e₂⟧ᵍ = (⟦e₁⟧ᵍ ≤ ⟦e₂⟧ᵍ)
⟦e₁ >= e₂⟧ᵍ = (⟦e₁⟧ᵍ ≥ ⟦e₂⟧ᵍ)
```

类型规则：

```
Γ ⊢ e₁ : T    Γ ⊢ e₂ : T    T : Comparable
─────────────────────────────────────────────
Γ ⊢ e₁ < e₂ : Boolean
```

```x
let equal = (x == y)        // Boolean
let greater = (a > b)       // Boolean
let in_range = (0 <= x and x < 100)
```

### 逻辑运算

X 使用英文关键字 `and`、`or` 作为逻辑运算符，两者均**短路求值**。

```
⟦e₁ and e₂⟧ᵍ = if ⟦e₁⟧ᵍ = false then false else ⟦e₂⟧ᵍ
⟦e₁ or e₂⟧ᵍ  = if ⟦e₁⟧ᵍ = true  then true  else ⟦e₂⟧ᵍ
```

类型规则：

```
Γ ⊢ e₁ : Boolean    Γ ⊢ e₂ : Boolean
──────────────────────────────────────
Γ ⊢ e₁ and e₂ : Boolean

Γ ⊢ e₁ : Boolean    Γ ⊢ e₂ : Boolean
──────────────────────────────────────
Γ ⊢ e₁ or e₂ : Boolean
```

```x
let both = is_valid and is_active
let either = is_admin or has_permission
let complex = (age >= 18) and (has_id or has_passport)
```

### 位运算

```
⟦e₁ & e₂⟧ᵍ  = ⟦e₁⟧ᵍ AND ⟦e₂⟧ᵍ    (位与)
⟦e₁ | e₂⟧ᵍ  = ⟦e₁⟧ᵍ OR  ⟦e₂⟧ᵍ    (位或)
⟦e₁ ^ e₂⟧ᵍ  = ⟦e₁⟧ᵍ XOR ⟦e₂⟧ᵍ    (位异或)
⟦e₁ << e₂⟧ᵍ = ⟦e₁⟧ᵍ SHL ⟦e₂⟧ᵍ    (左移)
⟦e₁ >> e₂⟧ᵍ = ⟦e₁⟧ᵍ SHR ⟦e₂⟧ᵍ    (右移)
```

```x
let flags = 0b1100 & 0b1010    // 0b1000
let combined = flag_a | flag_b
let shifted = value << 4
```

---

## 3.6 一元运算

### 一元运算语法

```
Expression ::= UnaryOp Expression
UnaryOp ::= '-' | 'not' | '~'
```

### 一元运算求值

```
⟦-e⟧ᵍ   = -⟦e⟧ᵍ           (算术取负)
⟦not e⟧ᵍ = ¬⟦e⟧ᵍ           (逻辑非)
⟦~e⟧ᵍ   = BNOT(⟦e⟧ᵍ)      (位取反)
```

类型规则：

```
Γ ⊢ e : Integer               Γ ⊢ e : Boolean
────────────────               ────────────────
Γ ⊢ -e : Integer              Γ ⊢ not e : Boolean

Γ ⊢ e : Integer
────────────────
Γ ⊢ ~e : Integer
```

```x
let negative = -42
let inverted = not is_valid
let complement = ~0xFF
```

---

## 3.7 条件表达式

### 条件表达式语法

`if` 在 X 中是表达式，具有值。

```
Expression ::= 'if' Expression Block ('else' (Block | IfExpression))?
```

### 条件表达式求值

```
⟦if e₁ { e₂ } else { e₃ }⟧ᵍ =
  if ⟦e₁⟧ᵍ = true then ⟦e₂⟧ᵍ else ⟦e₃⟧ᵍ
```

类型规则：

```
Γ ⊢ e₁ : Boolean    Γ ⊢ e₂ : T    Γ ⊢ e₃ : T
─────────────────────────────────────────────────
Γ ⊢ if e₁ { e₂ } else { e₃ } : T
```

当用作表达式时，`if` 和 `else` 分支必须返回相同类型。

```x
let max = if a > b { a } else { b }

let category = if age < 13 {
    "child"
} else if age < 18 {
    "teen"
} else {
    "adult"
}
```

### Match 表达式

`match` 是模式匹配表达式，对 scrutinee 进行穷尽匹配。

```
Expression ::= 'match' Expression '{' MatchArm* '}'
MatchArm   ::= Pattern ('if' Expression)? '=>' Expression
```

求值规则：

```
⟦match e { p₁ => e₁, p₂ => e₂, ..., pₙ => eₙ }⟧ᵍ =
  let v = ⟦e⟧ᵍ in
  if matches(p₁, v) then ⟦e₁⟧ᵍ₁ where g₁ = g + bindings(p₁, v)
  else if matches(p₂, v) then ⟦e₂⟧ᵍ₂ where g₂ = g + bindings(p₂, v)
  ...
  else if matches(pₙ, v) then ⟦eₙ⟧ᵍₙ where gₙ = g + bindings(pₙ, v)
```

类型规则：

```
Γ ⊢ e : S
Γ + bindings(pᵢ) ⊢ eᵢ : T    for all i
patterns p₁, ..., pₙ are exhaustive for S
──────────────────────────────────────────
Γ ⊢ match e { p₁ => e₁, ..., pₙ => eₙ } : T
```

```x
let description = match shape {
    Circle { radius } => "Circle with radius ${radius}"
    Rect { width, height } => "Rectangle ${width}x${height}"
    Point => "A point"
}

let result = match option_value {
    Some(x) => x * 2
    None => 0
}

let message = match status_code {
    200 => "OK"
    404 => "Not Found"
    500 => "Internal Server Error"
    code if code >= 400 => "Client Error"
    _ => "Unknown"
}
```

---

## 3.8 数组

### 数组语法

```
Expression ::= '[' (Expression (',' Expression)*)? ']'
```

支持尾随逗号。

### 数组求值

```
⟦[e₁, e₂, ..., eₙ]⟧ᵍ = [⟦e₁⟧ᵍ, ⟦e₂⟧ᵍ, ..., ⟦eₙ⟧ᵍ]
```

类型规则：

```
Γ ⊢ e₁ : T    Γ ⊢ e₂ : T    ...    Γ ⊢ eₙ : T
─────────────────────────────────────────────────
Γ ⊢ [e₁, e₂, ..., eₙ] : [T]
```

```x
let numbers: [Integer] = [1, 2, 3, 4, 5]
let names = ["Alice", "Bob", "Charlie"]    // [String]
let empty: [Float] = []
let nested = [[1, 2], [3, 4]]              // [[Integer]]
```

### 索引访问

```
Expression ::= Expression '[' Expression ']'
```

```
⟦e₁[e₂]⟧ᵍ = index(⟦e₁⟧ᵍ, ⟦e₂⟧ᵍ)
```

```x
let first = numbers[0]
let last = names[names.length() - 1]
```

### 范围表达式

```
Expression ::= Expression '..' Expression       // 左闭右开
             | Expression '..=' Expression      // 左闭右闭
```

```
⟦e₁..e₂⟧ᵍ = range(⟦e₁⟧ᵍ, ⟦e₂⟧ᵍ)          // [e₁, e₂)
⟦e₁..=e₂⟧ᵍ = range_inclusive(⟦e₁⟧ᵍ, ⟦e₂⟧ᵍ)  // [e₁, e₂]
```

```x
let exclusive = 0..10       // 0, 1, 2, ..., 9
let inclusive = 1..=100     // 1, 2, 3, ..., 100
let slice = items[2..5]     // 索引 2, 3, 4
```

---

## 3.9 成员访问

### 成员访问语法

```
Expression ::= Expression '.' Identifier
```

### 成员访问求值

```
⟦e.l⟧ᵍ = get_member(⟦e⟧ᵍ, l)

get_member : Record × Identifier → Value
get_member({l₁: v₁, ..., lₙ: vₙ}, lᵢ) = vᵢ
```

### 方法调用

```
Expression ::= Expression '.' Identifier '(' (Expression (',' Expression)*)? ')'
```

```
⟦e.m(e₁, ..., eₙ)⟧ᵍ = apply(method(⟦e⟧ᵍ, m), ⟦e⟧ᵍ, ⟦e₁⟧ᵍ, ..., ⟦eₙ⟧ᵍ)
```

```x
let x = point.x
let len = name.length()
let upper = text.to_uppercase()
```

### 类型检查（`is`）

```
Expression ::= Expression 'is' Type
```

运行时检查值是否属于指定类型，返回 `Boolean`。

```
Γ ⊢ e : S
──────────────────
Γ ⊢ e is T : Boolean
```

```x
let is_integer = value is Integer      // Boolean
let is_string = x is String

if value is Some {
    // value 确实是 Some 变体
}
```

### 类型转换（`as`）

```
Expression ::= Expression 'as' Type
```

显式类型转换。只允许安全的转换（如 `Integer` → `Float`）。不安全转换需在 `unsafe` 块中。

```
Γ ⊢ e : S    S convertible_to T
──────────────────────────────
Γ ⊢ e as T : T
```

```x
let f = 42 as Float              // 42.0
let i = 3.14 as Integer          // 3（截断）
```

---

## 3.10 管道

### 管道语法

```
Expression ::= Expression '|>' Expression
```

管道运算符 `|>` 将左侧表达式的结果作为右侧函数的第一个参数传入。

### 管道求值

```
⟦e₁ |> e₂⟧ᵍ = ⟦e₂(e₁)⟧ᵍ
⟦e₁ |> e₂ |> e₃⟧ᵍ = ⟦(e₁ |> e₂) |> e₃⟧ᵍ
```

管道左结合：`a |> f |> g` 等价于 `g(f(a))`。

类型规则：

```
Γ ⊢ e₁ : T₁    Γ ⊢ e₂ : (T₁, ...) -> R with Δ
──────────────────────────────────────────────
Γ ⊢ e₁ |> e₂ : R, Δ
```

```x
let result = [1, 2, 3, 4, 5]
    |> filter(function(n) => n % 2 == 0)
    |> map(function(n) => n * n)
    |> sum

let processed = raw_data
    |> parse
    |> validate
    |> transform
    |> serialize
```

---

## 3.11 错误传播（`?` / `?.` / `??`）

X 通过三个运算符提供简洁的错误处理语法，替代异常机制。

### `?` 运算符（错误传播）

```
Expression ::= Expression '?'
```

对 `Result<T, E>` 或 `Option<T>` 值使用。若为 `Err(e)` 或 `None`，立即从当前函数返回错误；否则展开内部值。

```
⟦e?⟧ᵍ =
  let v = ⟦e⟧ᵍ in
  match v with
  | Ok(x)  → x
  | Err(e) → return Err(e)

  // 或对 Option：
  | Some(x) → x
  | None    → return None
```

类型规则：

```
Γ ⊢ e : Result<T, E>    当前函数返回类型为 Result<_, E>
──────────────────────────────────────────────────────
Γ ⊢ e? : T

Γ ⊢ e : Option<T>    当前函数返回类型为 Option<_>
──────────────────────────────────────────────────
Γ ⊢ e? : T
```

```x
function load_config() -> Result<Config, IoError> {
    let content = read_file("config.toml")?
    let parsed = parse_toml(content)?
    Ok(parsed)
}
```

### `?.` 运算符（可选链）

```
Expression ::= Expression '?.' Identifier
             | Expression '?.' Identifier '(' (Expression (',' Expression)*)? ')'
```

安全地访问 `Option` 值的成员或方法。若值为 `None`，整个表达式求值为 `None`，不会引发错误。

```
⟦e?.l⟧ᵍ =
  let v = ⟦e⟧ᵍ in
  match v with
  | Some(x) → Some(get_member(x, l))
  | None    → None
```

类型规则：

```
Γ ⊢ e : Option<S>    S has member l : T
────────────────────────────────────────
Γ ⊢ e?.l : Option<T>
```

```x
let name = user?.name                  // Option<String>
let city = user?.address?.city         // Option<String>（链式）
let upper = user?.name?.to_uppercase() // Option<String>
```

### `??` 运算符（默认值）

```
Expression ::= Expression '??' Expression
```

若左侧为 `None`，返回右侧的默认值。

```
⟦e₁ ?? e₂⟧ᵍ =
  let v = ⟦e₁⟧ᵍ in
  match v with
  | Some(x) → x
  | None    → ⟦e₂⟧ᵍ
```

类型规则：

```
Γ ⊢ e₁ : Option<T>    Γ ⊢ e₂ : T
──────────────────────────────────
Γ ⊢ e₁ ?? e₂ : T
```

```x
let name = user?.name ?? "anonymous"
let port = config.get("port")?.parse_integer() ?? 8080
let display = title ?? description ?? "untitled"
```

### 组合使用

三个运算符可以自由组合，形成简洁的错误处理链：

```x
function get_user_email(id: Integer) -> Result<String, AppError> {
    let user = find_user(id)?                          // ? 传播错误
    let email = user.email ?? "no-reply@example.com"   // ?? 提供默认值
    Ok(email)
}

let display_name = user?.profile?.display_name ?? user?.name ?? "Guest"
```

### 括号表达式

```
Expression ::= '(' Expression ')'
⟦(e)⟧ᵍ = ⟦e⟧ᵍ
```

括号用于改变运算优先级，不影响求值结果。

---

**本章定义了 X 语言的表达式语法、求值语义与类型规则。核心设计：`and`/`or`/`not` 逻辑运算符、`match` 模式匹配、`?`/`?.`/`??` 错误处理运算符、`|>` 管道。**
