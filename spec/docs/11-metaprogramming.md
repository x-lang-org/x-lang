# 第11章 元编程

## 11.1 概述

元编程允许程序在编译期进行计算与决策，从而减少运行时开销并在编译期捕获错误。

```
Metaprogramming: Programs that operate on types, constants, or code at compile time.
```

X 的元编程能力包括：

| 能力 | 机制 | 用途 |
|------|------|------|
| 编译期常量 | `const` 声明 | 嵌入编译期已知的值 |
| 常量函数 | `const function` | 编译期执行的函数逻辑 |
| 泛型 | 类型参数 `<T>` | 参数多态，类型安全的代码复用 |
| 静态断言 | `static_assert` | 编译期不变量检查 |
| 编译期条件 | `#if` / `#else` | 条件编译，平台特化 |
| 类型级编程 | `sizeof`、`where` 约束 | 高级泛型与布局控制 |
| 宏 | `name!()` | 语法层面的代码生成 |

---

## 11.2 编译期常量

### 语法

```
ConstantDeclaration ::= 'const' Identifier ':' Type '=' ConstExpression ';'

ConstExpression ::= ConstLiteral
                  | ConstIdentifier
                  | ConstUnaryOp ConstExpression
                  | ConstExpression ConstBinaryOp ConstExpression
                  | ConstFunctionCall
```

常量在编译期求值，结果直接嵌入到生成的代码中。常量必须有显式类型注解，且只能用常量表达式初始化。

### 常量求值规则

```
⟦const x: T = e⟧ = embed(⟦e⟧ᶜᵒⁿˢᵗ)

⟦e⟧ᶜᵒⁿˢᵗ  denotes compile-time evaluation of e
embed(v)   embeds the computed value v into the binary
```

### 常量表达式限制

常量表达式只能包含：
- 字面量（整数、浮点、布尔、字符串、字符）
- 已定义的其他常量
- 纯算术、比较、逻辑运算
- `const function` 调用

不允许：运行时输入、I/O、分配、随机数、非 `const` 函数调用。

### 示例

```x
const MAX_BUFFER_SIZE: Integer = 1024
const PI: Float = 3.14159265358979
const APP_NAME: String = "MyApp"
const DEBUG: Boolean = false
const MASK: Integer = 0xFF00 | 0x00FF
```

---

## 11.3 常量函数

### 语法

```
ConstFunctionDeclaration ::= 'const' 'function' Identifier TypeParameters? Parameters '->' Type FunctionBody

FunctionBody ::= '=' ConstExpression ';'
               | Block   // block may only contain const-evaluable statements
```

`const function` 可在编译期被调用。函数体仅允许常量表达式、其他 `const function` 调用和受限控制流（常量条件分支、常量循环）。

### 常量函数规则

```
Γ ⊢ e : T    e is const-evaluable
────────────────────────────────────
Γ ⊢ const function f(...) -> T { e } : const (... -> T)

const-evaluable(e):
  - literals, const identifiers
  - const function calls
  - pure arithmetic, comparison, short-circuit logic
  - if/else with const condition
  - match with const scrutinee
```

### 示例

```x
const function factorial(n: Integer) -> Integer {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

const function max(a: Integer, b: Integer) -> Integer {
    if a > b { a } else { b }
}

const function is_power_of_two(n: Integer) -> Boolean {
    n > 0 && (n & (n - 1)) == 0
}

const FACT_10: Integer = factorial(10)
const BUFFER: Integer = max(256, MAX_BUFFER_SIZE)
```

---

## 11.4 泛型与类型参数

### 语法

```
GenericDeclaration ::= Identifier '<' TypeParameter (',' TypeParameter)* '>'

TypeParameter ::= Identifier
                | Identifier ':' TraitBound ('+' TraitBound)*
                | 'const' Identifier ':' Type

TraitBound ::= Identifier
             | Identifier '<' Type (',' Type)* '>'
```

### 泛型函数

类型参数使函数可以对任意类型工作，同时保持类型安全。

```x
function identity<T>(value: T) -> T {
    value
}

function swap<T>(pair: (T, T)) -> (T, T) {
    let (a, b) = pair
    (b, a)
}

function first<A, B>(pair: (A, B)) -> A {
    let (a, _) = pair
    a
}
```

### 约束泛型

通过 trait bound 约束类型参数：

```x
function largest<T: Comparable>(list: List<T>) -> Option<T> {
    match list {
        []       => None
        [single] => Some(single)
        [head, ..tail] => {
            match largest(tail) {
                Some(t) if t > head => Some(t)
                _                   => Some(head)
            }
        }
    }
}

function printAll<T: Display>(items: List<T>) -> () with IO {
    for item in items {
        println("${item}")
    }
}
```

### 常量泛型

用编译期常量参数化类型或函数：

```x
class FixedArray<T, const N: Integer> {
    private let data: Array<T>

    public function length() -> Integer {
        N
    }
}

function dot<const N: Integer>(a: FixedArray<Float, N>, b: FixedArray<Float, N>) -> Float {
    let mutable sum = 0.0
    for i in 0..N {
        sum = sum + a[i] * b[i]
    }
    sum
}
```

### 泛型实例化规则

```
Γ ⊢ f : ∀α. (T → U)    Γ ⊢ τ : Type    Γ ⊢ τ satisfies bounds on α
────────────────────────────────────────────────────────────────────────
Γ ⊢ f<τ> : T[α ↦ τ] → U[α ↦ τ]

monomorphize(f, concrete_types) = f'
  where f' is a specialized copy of f with all type parameters resolved
```

---

## 11.5 静态断言

### 语法

```
StaticAssert ::= 'static_assert' '(' ConstExpression (',' StringLiteral)? ')' ';'
```

`static_assert` 在编译期求值条件表达式。若为 `false`，编译失败并输出可选的错误消息。

### 静态断言规则

```
⟦static_assert(c, msg?)⟧ =
  if ⟦c⟧ᶜᵒⁿˢᵗ == true then ()
  else compile_error(msg ?: "static assertion failed")
```

### 示例

```x
const MAX: Integer = 1024
static_assert(MAX > 0, "MAX must be positive");
static_assert(is_power_of_two(MAX), "MAX must be a power of two");

const CACHE_LINE: Integer = 64
static_assert(CACHE_LINE >= 32, "cache line too small");
```

---

## 11.6 编译期条件

### 语法

```
CompileTimeCondition ::= '#if' ConstExpression Block
                       | '#if' ConstExpression Block '#else' Block
                       | '#if' ConstExpression Block ('#elseif' ConstExpression Block)* ('#else' Block)?
```

`#if` 在编译期求值条件表达式，仅保留满足条件的分支参与编译。未选中的分支不进行类型检查，也不生成代码。

### 编译期条件规则

```
⟦#if c then b₁ else b₂⟧ =
  if ⟦c⟧ᶜᵒⁿˢᵗ == true then ⟦b₁⟧ else ⟦b₂⟧

Only the selected branch is type-checked and compiled.
The other branch is discarded entirely.
```

### 与运行时 match 的区别

| 特性 | `#if` (编译期) | `match` (运行时) |
|------|---------------|-----------------|
| 求值时机 | 编译期 | 运行时 |
| 条件类型 | 常量表达式 | 任意表达式 |
| 未选中分支 | 不编译、不类型检查 | 编译但不执行 |
| 用途 | 平台特化、特性开关 | 运行时分支 |

### 示例

```x
const PLATFORM: String = "windows"

#if PLATFORM == "windows" {
    function lineEnding() -> String { "\r\n" }
} #elseif PLATFORM == "unix" {
    function lineEnding() -> String { "\n" }
} #else {
    function lineEnding() -> String { "\n" }
}

#if DEBUG {
    function log(msg: String) -> () with IO {
        println("[DEBUG] ${msg}")
    }
} #else {
    function log(msg: String) -> () {
        // no-op in release
    }
}
```

---

## 11.7 类型级编程

### 类型作为值

```
TypeExpression ::= 'sizeof' '(' Type ')'     // compile-time Integer constant
                 | 'alignof' '(' Type ')'    // compile-time Integer constant
```

`sizeof(T)` 和 `alignof(T)` 产生编译期整数常量，用于内存布局计算、缓冲区大小和静态断言。

```x
const INT_SIZE: Integer = sizeof(Integer)
const PTR_ALIGN: Integer = alignof(Pointer<Integer>)

static_assert(sizeof(MyStruct) <= 64, "MyStruct too large for cache line");
```

### where 约束

```
WhereClause ::= 'where' TypeConstraint (',' TypeConstraint)*

TypeConstraint ::= Type ':' TraitBound ('+' TraitBound)*
                 | Type '==' Type
                 | 'const' Identifier ':' Type
```

`where` 子句对类型参数施加约束，编译器在实例化时验证约束满足。

```x
function serialize<T>(value: T) -> String
    where T: Serialize
{
    value.to_string()
}

function zip_map<A, B, C>(
    list_a: List<A>,
    list_b: List<B>,
    f: (A, B) -> C
) -> List<C>
    where A: Clone, B: Clone
{
    // ...
}
```

---

## 11.8 宏与代码生成

### 宏定位

```
MacroInvocation ::= Identifier '!' MacroInput?

MacroInput ::= '(' TokenTree* ')'
             | '[' TokenTree* ']'
             | '{' TokenTree* '}'
```

宏在编译管线的语法/词法层面进行代码变换。宏是泛型和 `const function` 的补充——仅在类型系统和常量计算无法表达的场景下使用。

### 设计原则

1. **卫生性 (Hygiene)**：宏引入的标识符不与调用处冲突
2. **可预测展开**：宏展开结果是合法的 X 代码，可正常类型检查
3. **错误定位**：宏展开错误能追溯到宏定义或调用处

### 示例

```x
// 宏调用（假设 vec! 宏已定义）
let numbers = vec![1, 2, 3, 4, 5]

// derive 宏（通过属性触发）
@derive(Debug, Clone, Serialize)
class Point {
    let x: Float
    let y: Float
}
```

### 代码生成

```
CodeGeneration ::= Annotation-driven generation
                 | Build-script generation
```

代码生成可由注解（如 `@derive`）或构建脚本触发。生成的代码在类型检查之前插入到编译管线中。

---

## 11.9 与其他章节的关系

| 章节 | 关系说明 |
|------|----------|
| 第2章 类型系统 | 泛型基于类型系统；常量泛型扩展类型与值的边界 |
| 第5章 函数 | `const function` 是函数的编译期子集；泛型函数实现参数多态 |
| 第8章 模块 | 常量与泛型的可见性遵循模块系统的 `public`/`private`/`internal` 规则 |
| 第9章 模式匹配 | 泛型类型的模式匹配需要编译器单态化后检查穷尽性 |
| 第10章 内存管理 | 编译期常量无运行时分配；`const` 值不涉及 Perceus dup/drop 操作 |

---

## 11.10 小结

| 能力 | 关键字 / 语法 | 说明 |
|------|-------------|------|
| 编译期常量 | `const MAX: Integer = 1024` | 编译期求值并嵌入 |
| 常量函数 | `const function f(...) -> T` | 可在编译期调用的函数 |
| 泛型 | `function f<T>(x: T) -> T` | 参数多态，类型安全的代码复用 |
| 常量泛型 | `class Arr<T, const N: Integer>` | 编译期常量参数化类型 |
| 静态断言 | `static_assert(expr, msg)` | 编译期不变量检查 |
| 编译期条件 | `#if` / `#elseif` / `#else` | 条件编译，未选中分支不编译 |
| 类型查询 | `sizeof(T)` / `alignof(T)` | 编译期类型布局信息 |
| where 约束 | `where T: Trait` | 泛型类型参数约束 |
| 宏 | `name!(...)` | 语法层面代码变换 |
| 代码生成 | `@derive(...)` | 注解驱动的代码生成 |

---

**本章定义了 X 语言的元编程能力。编译期常量与常量函数提供编译期计算；泛型（含常量泛型）提供参数多态；静态断言和编译期条件在编译期检查不变量并选择代码路径；宏与代码生成作为补充手段，用于类型系统无法表达的场景。所有元编程能力的设计目标是：在编译期完成尽可能多的工作，减少运行时开销。**
