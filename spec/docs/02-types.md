# 第2章 类型系统

## 2.1 类型定义

X 拥有完整的、健全的类型系统，基于 Hindley-Milner 类型推断与代数数据类型。所有值都有确定的类型，所有错误路径在类型签名中显式可见。

### 类型集合

```
Type = PrimitiveType
     | CompositeType
     | FunctionType
     | EffectType
     | TypeVariable
     | TypeConstructor
```

### 类型环境

```
TypeEnv = Identifier → Type
Δ ∈ TypeEnv
```

### 子类型关系

```
⊢ T <: U    （T 是 U 的子类型）
```

---

## 2.2 基本类型

### 定义

X 的所有内置基础类型都使用**英文全称的小写单词**表示；整数和浮点数带有可选的**位宽后缀**，写成自然语言形式：`integer 32`、`unsigned integer 64`、`float 32` 等。

```
PrimitiveType = SignedIntegerType
              | UnsignedIntegerType
              | FloatType
              | BooleanType
              | StringType
              | CharacterType
              | UnitType
              | NeverType

SignedIntegerType   = "integer"
                    | "integer" IntegerWidth

UnsignedIntegerType = "unsigned" "integer" IntegerWidth

IntegerWidth        = 8 | 16 | 32 | 64 | 128

FloatType           = "float"
                    | "float" FloatWidth

FloatWidth          = 32 | 64

BooleanType         = "boolean"
StringType          = "string"
CharacterType       = "character"
UnitType            = "unit"
NeverType           = "never"
```

示例：

```x
let a: integer 32 = 42
let b: unsigned integer 32 = 1_000
let c: float 64 = 3.14159
let ok: boolean = true
let name: string = "Alice"
let ch: character = '中'
```

> 约定：
> - 未带位宽的 `integer`/`float` 表示“默认机型友好”的数值类型（例如 64 位平台上的 64 位整数），具体映射由实现定义但必须在标准库文档中说明。
> - 位宽后缀写成自然数：`integer 32`（而非 `i32`）、`unsigned integer 64`（而非 `u64`）。

### 详细定义

#### integer / integer *n*（有符号整数）

数学语义：

```
integer           : ℤ
integer n (n ∈ {8,16,32,64,128}) : 有界整数子集
Value(integer n) : { k ∈ ℤ | minₙ ≤ k ≤ maxₙ }
```

其中 `minₙ`、`maxₙ` 为 n 位二进制补码整数的最小/最大值（与 Rust 等语言一致）。

```x
let x: integer 32 = 42
let y: integer 64 = 1_000_000_000_000
let hex: integer 32 = 0xFF
let bin: integer 8 = 0b1010
```

#### unsigned integer *n*（无符号整数）

```
unsigned integer n (n ∈ {8,16,32,64,128})
Value(unsigned integer n) : { k ∈ ℤ | 0 ≤ k ≤ maxₙ }
```

```x
let size: unsigned integer 32 = 4_096
let mask: unsigned integer 8 = 0b1111_0000
```

#### float / float *n*（浮点数类型）

```
float 32 : IEEE 754 单精度浮点数
float 64 : IEEE 754 双精度浮点数
float    : 默认浮点类型（通常等同于 float 64）
```

```x
let pi: float 64 = 3.14159
let probability: float 32 = 0.125
let scientific: float 64 = 1.0e-10
```

#### boolean（布尔类型）

```
boolean : 𝔹
Value  : { true, false }
```

```x
let flag: boolean = true
let check = 1 > 0    // 推断为 boolean
```

#### string（字符串类型）

```
string : Σ*
Value  : { c₁c₂...cₙ | n ≥ 0, cᵢ ∈ Unicode }
```

UTF-8 编码的不可变字符串。

```x
let greeting: string = "Hello, X!"
let interpolated = "result = {1 + 2}"
```

#### character（字符类型）

```
character : Σ
Value     : Unicode 码点
```

```x
let ch: Character = 'A'
let emoji: Character = '🎉'
```

#### Unit（单位类型）

```
Unit : ()
Value : { () }
```

只有一个值 `()` 的类型，表示"无有意义的返回值"。等价于其他语言中的 `void`，但 `Unit` 是真正的类型，可参与类型运算。

```x
function greet(name: String) -> Unit {
    println("Hello, ${name}!")
}
```

#### Never（永无类型）

```
Never : ⊥
Value : ∅
```

没有任何值的类型，表示计算永远不会正常返回。`Never` 是所有类型的子类型：

```
∀T. Never <: T
```

```x
function panic(message: String) -> Never {
    // 程序终止，永不返回
}
```

### 基本类型关系

```
Never <: Integer
Never <: Float
Never <: Boolean
Never <: String
Never <: Character
Never <: Unit
```

---

## 2.3 复合类型

### 定义

```
CompositeType = ListType(Type)
              | DictionaryType(Type, Type)
              | TupleType(Type*)
              | RecordType(Identifier × Type*)
              | UnionType(Variant*)
              | OptionType(Type)
              | ResultType(Type, Type)
              | AsyncType(Type)
```

### 详细定义

#### List（列表类型）

```
ListType(T) : [T]
Value : [v₁, v₂, ..., vₙ | n ≥ 0, vᵢ ∈ T]
```

使用 `[T]` 表示元素类型为 `T` 的列表。

```x
let numbers: [Integer] = [1, 2, 3, 4, 5]
let names: [String] = ["Alice", "Bob"]
let empty: [Float] = []
```

#### Dictionary（字典类型）

```
DictionaryType(K, V) : {K: V}
Value : { k₁: v₁, k₂: v₂, ..., kₙ: vₙ | kᵢ ∈ K, vᵢ ∈ V }
```

使用 `{K: V}` 表示键类型为 `K`、值类型为 `V` 的字典。

```x
let ages: {String: Integer} = {"Alice": 30, "Bob": 25}
let config: {String: String} = {"host": "localhost", "port": "8080"}
```

#### Tuple（元组类型）

```
TupleType(T₁, T₂, ..., Tₙ) : (T₁, T₂, ..., Tₙ)
Value : (v₁, v₂, ..., vₙ)  where vᵢ ∈ Tᵢ
```

固定长度、异构类型的有序集合。

```x
let pair: (Integer, String) = (42, "answer")
let triple: (Float, Float, Float) = (1.0, 2.0, 3.0)
```

#### Record（记录类型）

```
RecordType(l₁: T₁, l₂: T₂, ..., lₙ: Tₙ)
Value : { l₁: v₁, l₂: v₂, ..., lₙ: vₙ }
        where vᵢ ∈ Tᵢ, lᵢ are distinct labels
```

记录是具名字段的积类型（product type）。使用 `type` 关键字定义：

```x
type Point = {
    x: Float,
    y: Float
}

type Person = {
    name: String,
    age: Integer,
    email: String
}

let origin: Point = { x: 0.0, y: 0.0 }
let alice = Person { name: "Alice", age: 30, email: "alice@example.com" }
```

**Copy-with 语法**：使用 `with` 从现有记录创建新记录，仅修改指定字段：

```x
let p1 = Point { x: 1.0, y: 2.0 }
let p2 = p1 with { x: 5.0 }           // p2 = { x: 5.0, y: 2.0 }

let updated = alice with { age: 31 }   // 仅修改 age，其余不变
```

语义规则：

```
⟦e₁ with { l₁: e'₁, ..., lₖ: e'ₖ }⟧ᵍ = r'
  where
    r = ⟦e₁⟧ᵍ                    // 原始记录
    v'ᵢ = ⟦e'ᵢ⟧ᵍ                 // 新字段值
    r' = r[l₁ ↦ v'₁, ..., lₖ ↦ v'ₖ]  // 更新指定字段
```

#### Union（联合类型 / 代数数据类型）

```
UnionType = Variant₁ | Variant₂ | ... | Variantₙ
Variant = Tag                             // 无数据变体
        | Tag(Type)                       // 单数据变体
        | Tag { l₁: T₁, ..., lₙ: Tₙ }   // 记录变体
```

联合类型是和类型（sum type），使用 `type` 关键字和 `|` 定义：

```x
type Shape =
    | Circle { radius: Float }
    | Rect { width: Float, height: Float }
    | Point

type Color =
    | Red
    | Green
    | Blue
    | Custom(Integer, Integer, Integer)
```

通过模式匹配使用联合类型：

```x
function area(shape: Shape) -> Float {
    match shape {
        Circle { radius } => 3.14159 * radius * radius
        Rect { width, height } => width * height
        Point => 0.0
    }
}
```

#### Option（可选类型）

```
OptionType(T) : Option<T>
Value : Some(v) where v ∈ T
      | None
```

X 语言**没有 null**。所有"可能缺失"的值用 `Option<T>` 表示。编译器强制处理 `None` 的情况，从而在编译期消除空指针错误。

```
Option<T> = Some(T) | None
```

```x
function find(users: [User], id: Integer) -> Option<User> {
    users |> filter(function(u) => u.id == id) |> first
}

let user = find(users, 42)
match user {
    Some(u) => println("Found: ${u.name}")
    None    => println("Not found")
}

// 便捷运算符
let name = user?.name ?? "anonymous"
```

类型规则：

```
Γ ⊢ e : Option<T>
────────────────────
Γ ⊢ e? : T           （在 Result/Option 返回上下文中展开）

Γ ⊢ e₁ : Option<T>    Γ ⊢ e₂ : T
──────────────────────────────────
Γ ⊢ e₁ ?? e₂ : T      （默认值运算符）
```

#### Result（结果类型）

```
ResultType(T, E) : Result<T, E>
Value : Ok(v)  where v ∈ T
      | Err(e) where e ∈ E
```

X 语言**没有异常机制**。所有可能失败的操作返回 `Result<T, E>`，错误路径在类型签名中显式可见。

```
Result<T, E> = Ok(T) | Err(E)
```

```x
function read_file(path: String) -> Result<String, IoError> {
    if not exists(path) {
        return Err(IoError.NotFound(path))
    }
    Ok(read_bytes(path).decode())
}

match read_file("config.toml") {
    Ok(content) => parse_config(content)
    Err(e)      => use_default()
}

// ? 运算符：自动传播错误
function load_config() -> Result<Config, IoError> {
    let content = read_file("config.toml")?
    let config = parse(content)?
    Ok(config)
}
```

类型规则：

```
Γ ⊢ e : Result<T, E>
────────────────────────────
Γ ⊢ e? : T     （错误自动传播，函数返回类型须为 Result<_, E>）
```

#### Async（异步类型）

```
AsyncType(T) : Async<T>
Value : 一个将来会产生 T 类型值的异步计算
```

```x
function fetch_data(url: String) -> Async<Data> {
    let response = await fetch(url)
    parse(response.body)
}
```

### Trait（特征 / 接口）

Trait 定义一组行为约束，类型可以实现 trait 以满足这些约束：

```x
trait Printable {
    function show(): String
}

trait Numeric {
    function add(other: Self) -> Self
    function multiply(other: Self) -> Self
}
```

实现 trait：

```x
implement Printable for Point {
    function show() -> String {
        "(${this.x}, ${this.y})"
    }
}
```

Trait 约束用于泛型参数：

```x
function print_all<T: Printable>(items: [T]) -> Unit {
    for item in items {
        println(item.show())
    }
}
```

---

## 2.4 函数类型

### 定义

```
FunctionType = (T₁, T₂, ..., Tₙ) -> R
             | (T₁, T₂, ..., Tₙ) -> R with Effects
```

函数类型由参数类型、返回类型和可选的效果集组成。纯函数无效果注解；有副作用的函数用 `with` 标注效果。

### 纯函数类型

```
(T₁, T₂, ..., Tₙ) -> R
```

```x
let add: (Integer, Integer) -> Integer = function(a, b) => a + b
let predicate: (String) -> Boolean = function(s) => s.length() > 0
```

### 带效果的函数类型

```
(T₁, T₂, ..., Tₙ) -> R with Effects
Effects = Effect₁, Effect₂, ..., Effectₖ
```

```x
let reader: (String) -> String with IO = read_file
let fetcher: (String) -> Data with Async, IO = fetch_data
```

### 函数类型规则

```
Γ, x₁: T₁, ..., xₙ: Tₙ ⊢ e : R, Δ
─────────────────────────────────────────────────────
Γ ⊢ (x₁: T₁, ..., xₙ: Tₙ) -> e : (T₁, ..., Tₙ) -> R with Δ
```

当 Δ = ∅ 时，简写为 `(T₁, ..., Tₙ) -> R`（纯函数）。

### 函数子类型（逆变参数，协变返回）

```
T'₁ <: T₁  ...  T'ₙ <: Tₙ    R <: R'    Δ ⊆ Δ'
────────────────────────────────────────────────────
(T₁, ..., Tₙ) -> R with Δ  <:  (T'₁, ..., T'ₙ) -> R' with Δ'
```

---

## 2.5 类型操作

### 类型等价

```
T ≡ U    （T 和 U 等价）
```

结构等价规则：两个类型在结构上相同时等价。

### 类型组合

```
T & U    （类型交集 / intersection）
T | U    （类型联合 / union）
```

### 泛型（参数多态）

```
∀α. T(α)
```

泛型类型通过类型参数实现参数多态：

```x
type Pair<A, B> = {
    first: A,
    second: B
}

function identity<T>(x: T) -> T {
    x
}

function map<A, B>(list: [A], f: (A) -> B) -> [B] {
    // ...
}
```

### 类型别名

```x
type Name = String
type UserMap = {String: User}
type Callback<T> = (T) -> Unit
```

---

## 2.6 类型推理（Hindley-Milner）

X 的类型推理基于 **Hindley-Milner（HM）算法**及其扩展（参见设计目标 §6）。绝大多数类型注解可省略，编译器自动推断。

### 推理规则

```
Γ ⊢ e : T    （在环境 Γ 中，表达式 e 的类型为 T）
```

### 变量查找（Var）

```
x : T ∈ Γ
──────────
Γ ⊢ x : T
```

### 函数应用（App）

```
Γ ⊢ f : (T₁, ..., Tₙ) -> R with Δ
Γ ⊢ e₁ : T₁  ...  Γ ⊢ eₙ : Tₙ
────────────────────────────────────
Γ ⊢ f(e₁, ..., eₙ) : R, Δ
```

### 函数抽象（Abs）

```
Γ, x₁: T₁, ..., xₙ: Tₙ ⊢ e : R
──────────────────────────────────────────────
Γ ⊢ function(x₁, ..., xₙ) => e : (T₁, ..., Tₙ) -> R
```

### 条件表达式（If）

```
Γ ⊢ e₁ : Boolean
Γ ⊢ e₂ : T
Γ ⊢ e₃ : T
────────────────────────────────
Γ ⊢ if e₁ then e₂ else e₃ : T
```

### Let 绑定（Let）

```
Γ ⊢ e₁ : S    Γ, x : Gen(S, Γ) ⊢ e₂ : T
──────────────────────────────────────────────
Γ ⊢ let x = e₁ in e₂ : T
```

其中 `Gen(S, Γ)` 将 `S` 中不在 `Γ` 自由变量中出现的类型变量泛化。

### 统一（Unification）

```
unify(T, U) = σ    （σ 是最通用统一替换 MGU）

unify(α, T) = [α ↦ T]           if α ∉ FTV(T)
unify(T, α) = [α ↦ T]           if α ∉ FTV(T)
unify(C<T₁,...,Tₙ>, C<U₁,...,Uₙ>) = σₙ ∘ ... ∘ σ₁
  where σᵢ = unify(σᵢ₋₁(Tᵢ), σᵢ₋₁(Uᵢ))
```

### 推断示例

```x
// 编译器自动推断所有类型
let x = 42                              // x : Integer
let f = function(a, b) => a + b         // f : (Integer, Integer) -> Integer
let xs = [1, 2, 3]                      // xs : [Integer]
let result = xs |> map(function(n) => n * 2)   // result : [Integer]
let pair = (true, "hello")              // pair : (Boolean, String)
```

### 局部推断与顶层签名

- **局部类型推断**：函数体内几乎不需要任何类型注解
- **顶层签名推荐但可选**：公共 API 建议写类型签名以提升可读性
- **双向类型检查**：结合自上而下（期望类型）与自下而上（推断类型）两个方向

---

## 2.7 类型约束

### 类型变量约束

```
∀α. C(α) ⇒ T(α)
```

其中 `C(α)` 是对类型变量 `α` 的约束集合。

```x
function sum<T: Numeric>(list: [T]) -> T {
    list |> reduce(function(a, b) => a.add(b))
}
```

### Trait 约束

```
⊢ T : Trait              （T 实现了 Trait）
⊢ T : Trait₁ + Trait₂    （T 同时实现了 Trait₁ 和 Trait₂）
```

```x
function display_and_compare<T: Printable + Comparable>(a: T, b: T) -> String {
    if a > b { a.show() } else { b.show() }
}
```

### 子类型约束

```
⊢ T <: U
```

### 效果约束

```
⊢ Δ₁ ⊆ Δ₂    （效果集 Δ₁ 是 Δ₂ 的子集）
```

带效果约束的函数可以调用效果更小的函数，但不能反过来：

```x
function pure_compute(x: Integer) -> Integer {
    x * 2
}

function io_compute(x: Integer) -> Integer with IO {
    println("computing...")
    pure_compute(x)    // 纯函数可在有效果的上下文中调用
}
```

---

**本章定义了 X 语言的类型系统，采用数学形式化与代码示例结合的方式。核心设计原则：无 null、无异常、类型安全、HM 推断。**
