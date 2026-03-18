# 第4章 语句

## 4.1 语句分类

### 语句语法定义

```
Statement → Declaration
          | ExpressionStatement
          | Assignment
          | IfStatement
          | WhileStatement
          | ForStatement
          | ReturnStatement
          | Block
          | MatchStatement
```

X 语言**没有 `try`/`catch`/`finally`/`throw` 语句**。错误处理通过 `Result<T, E>` 和 `Option<T>` 类型在表达式层面完成（参见第3章 §3.11）。

### 表达式语句

```
ExpressionStatement ::= Expression
```

**执行说明：**
计算表达式的值，然后丢弃结果。仅用于表达式的副作用。

```
⟦e;⟧ᵍ = let _ = ⟦e⟧ᵍ in (), g
```

```x
println("Hello, World!")
list.push(42)
send_notification(user, message)
```

---

## 4.2 声明语句

### 变量声明语法

```
Declaration ::= 'let' Identifier (':' Type)? ('=' Expression)?
              | 'let' 'mutable' Identifier (':' Type)? ('=' Expression)?
```

X 使用 `let` 声明不可变绑定，`let mutable` 声明可变绑定。关键字使用英文全称 `mutable` 而非缩写（参见设计目标 §14）。

### 不可变绑定（`let`）

在当前作用域中创建一个新的不可变绑定：

- 如果提供了初始化表达式，计算并将值绑定到标识符
- 后续不能重新赋值给该标识符
- 类型可显式注解，也可由编译器通过 HM 推断自动推导

```x
let name = "Alice"                    // 类型推断为 String
let age: Integer = 30                 // 显式类型注解
let pi = 3.14159                      // 类型推断为 Float
let is_valid = true                   // 类型推断为 Boolean
let numbers = [1, 2, 3]              // 类型推断为 [Integer]
```

### 可变绑定（`let mutable`）

在当前作用域中创建一个新的可变绑定：

- 如果提供了初始化表达式，计算并将值绑定到标识符
- 后续可以通过赋值语句修改该绑定的值
- 可变性是显式选择，遵循"默认不可变"原则（参见设计目标 §7）

```x
let mutable count = 0                 // 可变 Integer
let mutable name: String = "Bob"      // 可变 String，显式注解
let mutable items: [Integer] = []     // 可变列表
```

### 类型注解规则

- 如果提供了类型注解，初始化表达式的类型必须与注解类型兼容
- 如果没有提供类型注解，通过初始化表达式推断类型（HM 推断）
- 公共 API 建议写类型注解以提升可读性

### 声明求值规则

```
⟦let x: T = e⟧ᵍ = (), g'
  where
    v = ⟦e⟧ᵍ
    g' = g[x ↦ v]

⟦let mutable x: T = e⟧ᵍ = (), g'
  where
    v = ⟦e⟧ᵍ
    g' = g[x ↦ mutable v]
```

类型规则：

```
Γ ⊢ e : T
──────────────────────
Γ, x : T ⊢ let x = e

Γ ⊢ e : T
──────────────────────────────
Γ, x : mutable T ⊢ let mutable x = e
```

### 解构绑定

`let` 绑定支持模式解构：

```x
let (x, y) = get_position()              // 元组解构
let { name, age } = get_person()          // 记录解构
let [first, second, ..rest] = numbers     // 列表解构

let mutable (a, b) = (1, 2)              // 可变元组解构
```

---

## 4.3 赋值语句

### 赋值语法

```
Assignment ::= LValue '=' Expression
             | LValue AssignmentOp Expression

AssignmentOp ::= '+=' | '-=' | '*=' | '/=' | '%=' | '^='
               | '&=' | '|=' | '<<=' | '>>='

LValue ::= Identifier
          | LValue '.' Identifier
          | LValue '[' Expression ']'
```

### 简单赋值（`=`）

- 计算右侧表达式的值
- 将值存储到左侧的左值位置
- **左侧必须是 `let mutable` 声明的可变绑定**

```x
let mutable x = 10
x = 20                    // OK：x 是可变的

let y = 10
// y = 20                 // 编译错误：y 是不可变的
```

### 复合赋值（`op=`）

- 读取左值的当前值
- 与右侧表达式的值进行指定运算
- 将结果存回左值位置
- 等价于 `lvalue = lvalue op expression`

```x
let mutable count = 0
count += 1                // count = count + 1
count *= 2                // count = count * 2

let mutable flags = 0xFF
flags &= 0x0F             // flags = flags & 0x0F
flags |= 0x80             // flags = flags | 0x80
```

### 赋值求值规则

```
⟦l = e⟧ᵍ = (), g'
  where
    v = ⟦e⟧ᵍ
    g' = g[l ↦ v]

⟦l += e⟧ᵍ = ⟦l = l + e⟧ᵍ
⟦l -= e⟧ᵍ = ⟦l = l - e⟧ᵍ
⟦l *= e⟧ᵍ = ⟦l = l * e⟧ᵍ
⟦l /= e⟧ᵍ = ⟦l = l / e⟧ᵍ
⟦l %= e⟧ᵍ = ⟦l = l % e⟧ᵍ
```

类型规则：

```
Γ ⊢ l : mutable T    Γ ⊢ e : T
────────────────────────────────
Γ ⊢ l = e : Unit
```

---

## 4.4 块语句

### 块语法

```
Block ::= '{' Statement* '}'
```

**执行说明：**
- 创建一个新的词法作用域
- 按顺序执行块内的所有语句
- 块内声明的变量在块结束后超出作用域
- 块的值是最后一个表达式的值（如果最后一条语句是表达式），否则是 `Unit`

### 块求值规则

```
⟦{ s₁; s₂; ...; sₙ }⟧ᵍ = v, gₙ
  where
    (_, g₁) = ⟦s₁⟧ᵍ
    (_, g₂) = ⟦s₂⟧ᵍ₁
    ...
    (v, gₙ) = ⟦sₙ⟧ᵍₙ₋₁
```

块作为表达式使用时，最后一个表达式即为块的值：

```x
let result = {
    let a = compute_x()
    let b = compute_y()
    a + b                   // 块的值
}

let message = {
    let name = get_name()
    "Hello, ${name}!"       // 块的值为 String
}
```

---

## 4.5 条件语句

### If 语句语法

```
IfStatement ::= 'if' Expression Block ('else' (Block | IfStatement))?
```

**执行说明：**
- 计算条件表达式的值（必须是 `Boolean` 类型）
- 如果为 `true`，执行 then 分支的块
- 如果为 `false`，执行 else 分支（如果存在）
- 可以链式使用 `else if` 进行多分支判断

### If 语句求值规则

```
⟦if e { b₁ } else { b₂ }⟧ᵍ =
  if ⟦e⟧ᵍ = true then ⟦b₁⟧ᵍ else ⟦b₂⟧ᵍ

⟦if e { b }⟧ᵍ =
  if ⟦e⟧ᵍ = true then ⟦b⟧ᵍ else (), g
```

类型规则：

```
Γ ⊢ e : Boolean    Γ ⊢ b₁ : T    Γ ⊢ b₂ : T
───────────────────────────────────────────────
Γ ⊢ if e { b₁ } else { b₂ } : T

Γ ⊢ e : Boolean    Γ ⊢ b : Unit
────────────────────────────────
Γ ⊢ if e { b } : Unit
```

```x
if is_valid {
    process(data)
}

if age >= 18 {
    println("Adult")
} else {
    println("Minor")
}

if score >= 90 {
    println("A")
} else if score >= 80 {
    println("B")
} else if score >= 70 {
    println("C")
} else {
    println("F")
}
```

---

## 4.6 循环语句

### While 语句

```
WhileStatement ::= 'while' Expression Block
```

**执行说明：**
- 重复执行：
  1. 计算条件表达式（必须是 `Boolean` 类型）
  2. 如果为 `true`，执行循环体，然后回到步骤 1
  3. 如果为 `false`，退出循环
- 条件在每次迭代前检查

```
⟦while e { b }⟧ᵍ = loop(e, b, g)
  where
    loop(e, b, g) =
      if ⟦e⟧ᵍ = true
      then let (_, g') = ⟦b⟧ᵍ in loop(e, b, g')
      else (), g
```

```x
let mutable i = 0
while i < 10 {
    println("i = ${i}")
    i += 1
}

let mutable sum = 0
let mutable n = 100
while n > 0 {
    sum += n
    n -= 1
}
```

### For 语句

```
ForStatement ::= 'for' Pattern 'in' Expression Block
```

**执行说明：**
- 遍历可迭代对象（如列表、范围）
- 对于每个元素：
  - 将元素与模式绑定
  - 执行循环体
- 循环变量在每次迭代结束后重新绑定

```
⟦for p in e { b }⟧ᵍ = loop(xs, p, b, g)
  where
    xs = ⟦e⟧ᵍ
    loop([], p, b, g) = (), g
    loop(v:vs, p, b, g) =
      let g' = g + bindings(p, v) in
      let (_, g'') = ⟦b⟧ᵍ' in
      loop(vs, p, b, g'')
```

```x
for item in items {
    println(item)
}

for i in 0..10 {
    println("index: ${i}")
}

for (key, value) in dictionary {
    println("${key} = ${value}")
}

for name in ["Alice", "Bob", "Charlie"] {
    println("Hello, ${name}!")
}
```

### 循环控制

```
BreakStatement    ::= 'break'
ContinueStatement ::= 'continue'
```

- `break`：立即退出最内层循环
- `continue`：跳过当前迭代的剩余部分，进入下一次迭代

```x
for i in 0..100 {
    if i % 2 == 0 {
        continue
    }
    if i > 50 {
        break
    }
    println(i)
}
```

---

## 4.7 返回语句

### Return 语句语法

```
ReturnStatement ::= 'return' Expression?
```

**执行说明：**
- 立即从当前函数返回
- 如果提供了表达式，计算其值作为返回值
- 如果没有表达式，返回 `Unit`（即 `()`）
- 跳过后续所有语句

### Return 语句求值规则

```
⟦return e⟧ᵍ = Return(v)
  where v = ⟦e⟧ᵍ

⟦return⟧ᵍ = Return(())
```

类型规则：

```
Γ ⊢ e : T    函数返回类型为 T
────────────────────────────
Γ ⊢ return e : Never

Γ ⊢ return : Never        （函数返回类型为 Unit）
```

`return` 的类型是 `Never`，因为控制流不会继续到下一条语句。

```x
function find_index(items: [String], target: String) -> Option<Integer> {
    for i in 0..items.length() {
        if items[i] == target {
            return Some(i)
        }
    }
    None
}

function greet(name: String) {
    if name.is_empty() {
        return
    }
    println("Hello, ${name}!")
}
```

---

## 4.8 Match 语句（模式匹配）

### Match 语句语法

```
MatchStatement ::= 'match' Expression '{' MatchBranch* '}'

MatchBranch ::= Pattern ('if' Expression)? '=>' (Expression | Block)
```

X 使用 `match` 关键字进行模式匹配（而非其他语言中的 `switch` 或 `when`）。

**执行说明：**
- 计算 scrutinee 表达式的值
- 按顺序尝试将值与每个分支的模式匹配
- 第一个匹配成功的分支被执行
- 如果分支有 `if` 守卫（guard），守卫条件也必须为 `true` 才算匹配
- 编译器检查分支的穷尽性——必须覆盖所有可能的情况

### Match 语句求值规则

```
⟦match e { p₁ => b₁, p₂ => b₂, ..., pₙ => bₙ }⟧ᵍ =
  let v = ⟦e⟧ᵍ in
  if matches(p₁, v) ∧ guard₁(g₁)
    then ⟦b₁⟧ᵍ₁ where g₁ = g + bindings(p₁, v)
  else if matches(p₂, v) ∧ guard₂(g₂)
    then ⟦b₂⟧ᵍ₂ where g₂ = g + bindings(p₂, v)
  ...
```

### 支持的模式类型

```
Pattern ::= LiteralPattern          // 字面量匹配
           | IdentifierPattern       // 绑定变量
           | WildcardPattern         // 通配符 _
           | TuplePattern            // 元组解构
           | RecordPattern           // 记录解构
           | VariantPattern          // 联合类型变体
           | GuardedPattern          // 带守卫的模式

LiteralPattern    ::= IntegerLiteral | FloatLiteral | StringLiteral | 'true' | 'false'
IdentifierPattern ::= Identifier
WildcardPattern   ::= '_'
TuplePattern      ::= '(' Pattern (',' Pattern)* ')'
RecordPattern     ::= '{' Identifier (':' Pattern)? (',' Identifier (':' Pattern)?)* '}'
VariantPattern    ::= Identifier ('(' Pattern ')')? ('{' ... '}')?
```

### 使用示例

#### 基本模式匹配

```x
match command {
    "quit" => {
        println("Goodbye!")
        exit(0)
    }
    "help" => show_help()
    "version" => println("v1.0.0")
    _ => println("Unknown command: ${command}")
}
```

#### Option 匹配

```x
let result = find_user(42)
match result {
    Some(user) => println("Found: ${user.name}")
    None => println("User not found")
}
```

#### Result 匹配

```x
match read_file("data.txt") {
    Ok(content) => process(content)
    Err(IoError.NotFound(path)) => println("File not found: ${path}")
    Err(IoError.Permission(path)) => println("Permission denied: ${path}")
    Err(e) => println("Error: ${e}")
}
```

#### 联合类型匹配

```x
match shape {
    Circle { radius } => {
        let area = 3.14159 * radius * radius
        println("Circle area: ${area}")
    }
    Rect { width, height } => {
        let area = width * height
        println("Rectangle area: ${area}")
    }
    Point => println("Point has no area")
}
```

#### 带守卫的匹配

```x
match score {
    n if n >= 90 => "A"
    n if n >= 80 => "B"
    n if n >= 70 => "C"
    n if n >= 60 => "D"
    _ => "F"
}
```

#### 嵌套模式

```x
match data {
    (Some(x), Some(y)) => println("Both present: ${x}, ${y}")
    (Some(x), None) => println("Only first: ${x}")
    (None, Some(y)) => println("Only second: ${y}")
    (None, None) => println("Neither present")
}
```

---

## 4.9 作用域与生命周期

### 作用域规则

```
Scope ::= GlobalScope | LocalScope(parent: Scope)

lookup : Scope × Identifier → Value
lookup(LocalScope(parent), x) =
  if x defined in LocalScope
  then get(LocalScope, x)
  else lookup(parent, x)

lookup(GlobalScope, x) =
  if x defined in GlobalScope
  then get(GlobalScope, x)
  else Error("undefined variable: x")
```

### 词法作用域

变量绑定遵循词法作用域（lexical scoping）——变量的可见性由其在源码中的位置决定，而非运行时的调用关系。

```x
let x = 10

function foo() -> Integer {
    x               // 可以访问外层的 x
}

{
    let y = 20      // y 仅在此块内可见
    println(x + y)  // 可以访问外层的 x
}
// println(y)       // 编译错误：y 不在作用域内
```

### 遮蔽（Shadowing）

内部作用域可以声明与外部作用域同名的变量，遮蔽外部变量：

```x
let x = 10
{
    let x = 20      // 遮蔽外层的 x
    println(x)      // 输出 20
}
println(x)          // 输出 10（外层 x 未被修改）

let value = "hello"
let value = value.length()    // 允许：用新类型遮蔽旧绑定
```

### 变量生命周期

- 变量在声明点创建，在作用域结束时销毁
- 声明点之前不可访问（无变量提升）
- Perceus 算法在编译期插入 `dup`/`drop` 操作管理内存

```
lifetime(x) = [declaration_point, scope_end]

// 编译器自动插入：
⟦let x = e⟧ → let x = e; dup(x) if referenced later
⟦scope_end⟧ → drop(x) for each x leaving scope
```

```x
function example() -> String {
    let a = "hello"             // a 创建
    let b = a + " world"       // a 被 dup（仍需使用）
    // a 在此作用域结束时 drop
    b                           // b 的所有权转移给调用者
}
```

---

**本章定义了 X 语言的语句语法与执行语义。核心设计：`let`/`let mutable` 声明、`match` 模式匹配、无异常（无 try/catch/throw）、词法作用域、Perceus 生命周期管理。**
