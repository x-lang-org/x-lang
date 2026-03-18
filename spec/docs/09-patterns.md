# 第9章 模式匹配

X 使用 `match` 关键字进行模式匹配，编译器强制要求穷尽性检查。模式匹配广泛用于 `match` 表达式、`let` 绑定解构、`for` 循环解构和函数参数解构。

## 9.1 模式语法

### 模式分类

```
Pattern ::= LiteralPattern
          | VariablePattern
          | WildcardPattern
          | ConstructorPattern
          | TuplePattern
          | RecordPattern
          | OrPattern
          | GuardedPattern
          | AsPattern
          | TypeTestPattern
```

### match 表达式语法

```
MatchExpression ::= 'match' Expression '{' MatchArm+ '}'

MatchArm ::= Pattern ('=>' Expression)+
           | Pattern 'if' Expression '=>' Expression
```

模式匹配出现的上下文：

| 上下文 | 示例 |
|--------|------|
| `match` 表达式 | `match value { pattern => body }` |
| `let` 绑定 | `let (a, b) = pair` |
| `for` 循环 | `for (key, value) in map { ... }` |
| 函数参数 | `function first((a, _): (Integer, Integer)) -> Integer { a }` |

---

## 9.2 字面量模式

### 语法

```
LiteralPattern ::= IntegerLiteral
                 | FloatLiteral
                 | BooleanLiteral
                 | StringLiteral
                 | CharacterLiteral
```

字面量模式匹配与该字面量值相等的值。不存在 `null` 或 `none` 字面量——缺失值通过 `Option<T>` 的 `None` 构造函数表达。

### 匹配规则

```
matches(v, lit) = (v == ⟦lit⟧)
  where ⟦lit⟧ is the compile-time value of the literal
  and typeof(v) = typeof(⟦lit⟧)
```

### 示例

```x
match statusCode {
    200 => "OK"
    404 => "Not Found"
    500 => "Internal Server Error"
    _   => "Unknown"
}

match flag {
    true  => "enabled"
    false => "disabled"
}
```

---

## 9.3 变量模式

### 语法

```
VariablePattern ::= Identifier
                  | 'mutable' Identifier
```

变量模式匹配任何值，并将该值绑定到标识符。`mutable` 修饰符表示绑定为可变。

### 匹配规则

```
matches(v, x) = true
bindings(v, x) = {x ↦ v}         (immutable binding)

matches(v, mutable x) = true
bindings(v, mutable x) = {x ↦ mutable v}  (mutable binding)
```

### 示例

```x
match value {
    x => println("got ${x}")
}

match pair {
    (mutable x, y) => {
        x = x + 1
        x + y
    }
}
```

---

## 9.4 通配符模式

### 语法

```
WildcardPattern ::= '_'
```

通配符模式匹配任何值，但不创建绑定。用于表示不关心的位置。

### 匹配规则

```
matches(v, _) = true
bindings(v, _) = ∅
```

### 示例

```x
match triple {
    (first, _, _) => first
}
```

---

## 9.5 构造函数模式

### 语法

```
ConstructorPattern ::= Identifier '(' (Pattern (',' Pattern)*)? ')'
                     | Identifier
```

构造函数模式匹配代数数据类型（ADT）的构造函数。`Option<T>` 和 `Result<T, E>` 作为核心类型，其构造函数 `Some`、`None`、`Ok`、`Err` 在模式中广泛使用。

### Option 模式

```x
match optionalValue {
    Some(x) => println("has value: ${x}")
    None    => println("no value")
}
```

### Result 模式

```x
match result {
    Ok(data)  => process(data)
    Err(error) => handleError(error)
}
```

### 自定义 ADT 构造函数模式

```x
enum Shape {
    Circle(Float)
    Rect(Float, Float)
    Point
}

match shape {
    Circle(radius)       => 3.14159 * radius * radius
    Rect(width, height)  => width * height
    Point                => 0.0
}
```

### 匹配规则

```
v = C(v₁, ..., vₙ)
matches(v₁, p₁) ∧ ... ∧ matches(vₙ, pₙ)
──────────────────────────────────────────
matches(v, C(p₁, ..., pₙ)) = true

v = Some(v₁)    matches(v₁, p)
────────────────────────────────
matches(v, Some(p)) = true

matches(None, None) = true

v = Ok(v₁)    matches(v₁, p)
──────────────────────────────
matches(v, Ok(p)) = true

v = Err(e₁)    matches(e₁, p)
───────────────────────────────
matches(v, Err(p)) = true
```

---

## 9.6 元组与记录模式

### 元组模式语法

```
TuplePattern ::= '(' Pattern (',' Pattern)+ ')'
```

元组模式按位置匹配元组的每个元素。

### 记录模式语法

```
RecordPattern ::= Identifier? '{' FieldPattern (',' FieldPattern)* (',' '..')? '}'

FieldPattern ::= Identifier (':' Pattern)?
```

记录模式按字段名匹配，字段顺序无关。使用 `..` 忽略未列出的字段。简写形式 `{ name }` 等价于 `{ name: name }`。

### 匹配规则

```
v = (v₁, ..., vₙ)
matches(v₁, p₁) ∧ ... ∧ matches(vₙ, pₙ)
──────────────────────────────────────────
matches(v, (p₁, ..., pₙ)) = true

v = { l₁: v₁, ..., lₙ: vₙ }
matches(vᵢ, pᵢ) for each (lᵢ: pᵢ) in pattern
────────────────────────────────────────────────
matches(v, { l₁: p₁, ..., lₖ: pₖ, .. }) = true    (k ≤ n)
```

### 示例

```x
let (x, y, z) = (1, 2, 3)

match point {
    { x: 0, y: 0 } => "origin"
    { x, y, .. }    => "point at (${x}, ${y})"
}

match user {
    User { name, age } if age >= 18 => "adult: ${name}"
    User { name, .. }               => "minor: ${name}"
}
```

---

## 9.7 组合模式

### Or 模式

```
OrPattern ::= Pattern '|' Pattern
```

匹配任一子模式。两个子模式绑定的变量集合必须相同，且类型一致。

```
matches(v, p₁) ∨ matches(v, p₂)
FV(p₁) = FV(p₂)
────────────────────────────────
matches(v, p₁ | p₂) = true
```

```x
match direction {
    North | South => "vertical"
    East | West   => "horizontal"
}
```

### Guarded 模式（守卫）

```
GuardedPattern ::= Pattern 'if' Expression
```

先匹配模式，再求值守卫条件（必须为 `Boolean` 类型）。仅当模式匹配且守卫为 `true` 时才选中该分支。

```
matches(v, p)
⟦guard⟧^(env ∪ bindings(v, p)) = true
──────────────────────────────────────
matches(v, p if guard) = true
```

```x
match number {
    x if x > 0 => "positive"
    x if x < 0 => "negative"
    _           => "zero"
}
```

### As 模式

```
AsPattern ::= Pattern 'as' Identifier
```

匹配子模式，同时将整个值绑定到标识符。

```
matches(v, p)
──────────────────────────────────────
matches(v, p as x) = true
bindings(v, p as x) = bindings(v, p) ∪ {x ↦ v}
```

```x
match list {
    [first, ..rest] as whole => {
        println("first: ${first}, total: ${whole.length()}")
    }
    [] => println("empty")
}
```

---

## 9.8 类型测试模式

### 语法

```
TypeTestPattern ::= Identifier 'is' Type
                  | '_' 'is' Type
```

测试值的运行时类型。若类型匹配成功，值在该分支中自动具有更具体的类型（smart cast）。

### 匹配规则

```
typeof(v) <: T
──────────────────────
matches(v, x is T) = true
bindings(v, x is T) = {x ↦ v : T}
```

### 示例

```x
match animal {
    d is Dog => d.bark()
    c is Cat => c.meow()
    _        => println("unknown animal")
}
```

---

## 9.9 穷尽性检查

X 编译器强制要求 `match` 表达式覆盖所有可能的模式。若无法证明穷尽性，编译器报错。

### 形式化定义

```
exhaustive(patterns, T) : Boolean =
  ∀ v : T, ∃ p ∈ patterns, matches(v, p)
```

### 穷尽性检查规则

| 类型 | 穷尽条件 |
|------|----------|
| `Boolean` | 必须覆盖 `true` 和 `false` |
| `enum` | 必须覆盖所有变体，或使用 `_` 通配符 |
| `Option<T>` | 必须覆盖 `Some(_)` 和 `None` |
| `Result<T, E>` | 必须覆盖 `Ok(_)` 和 `Err(_)` |
| `Integer`、`String` 等 | 必须包含 `_` 通配符兜底 |

### 示例

```x
// 编译通过：穷尽覆盖
match option_value {
    Some(x) if x > 0 => x
    Some(_)           => 0
    None              => -1
}

// 编译错误：未覆盖 None
match option_value {
    Some(x) => x     // error: non-exhaustive patterns, `None` not covered
}

// 编译通过：通配符兜底
match code {
    200 => "OK"
    404 => "Not Found"
    _   => "Other"
}
```

### 完整示例

```x
enum Shape {
    Circle { radius: Float }
    Rect { width: Float, height: Float }
    Point
}

function area(shape: Shape) -> Float {
    match shape {
        Circle { radius }       => 3.14159 * radius * radius
        Rect { width, height }  => width * height
        Point                   => 0.0
    }
}

function describe(value: Result<Integer, String>) -> String {
    match value {
        Ok(n) if n > 100 => "large: ${n}"
        Ok(n)            => "small: ${n}"
        Err(msg)         => "error: ${msg}"
    }
}
```

---

**本章定义了 X 语言的模式匹配系统。`match` 表达式使用 `=>` 分隔模式与分支体，支持字面量、变量、通配符、构造函数、元组、记录、Or、守卫、As 以及类型测试模式。编译器强制穷尽性检查，确保所有可能的值都被处理。**
