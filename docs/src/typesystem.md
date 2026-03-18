# X 编程语言类型系统

X 语言拥有完整的、健全的类型系统，基于 Hindley-Milner 类型推断与代数数据类型。所有值都有确定的类型，所有错误路径在类型签名中显式可见。本文档详细介绍 X 语言的类型系统核心特性。

## 1. 类型推断

X 的类型推断基于 **Hindley-Milner（HM）算法**及其扩展，使得绝大多数类型注解可省略，编译器能够自动推断出表达式的类型。

### 1.1 推断原理

类型推断的核心是通过以下规则推导出表达式的类型：

- **变量查找**：从类型环境中查找变量的类型
- **函数应用**：根据函数的参数类型和返回类型，推断函数调用的类型
- **函数抽象**：根据函数体的类型，推断函数的类型
- **条件表达式**：要求条件分支的类型一致
- **Let 绑定**：支持类型泛化，使得同一变量在不同上下文中可以有不同的具体类型

### 1.2 统一算法

类型推断的关键是**统一（Unification）**算法，它通过找到最通用的类型替换，使得不同的类型表达式能够匹配：

```
unify(α, T) = [α ↦ T]           if α ∉ FTV(T)
unify(T, α) = [α ↦ T]           if α ∉ FTV(T)
unify(C<T₁,...,Tₙ>, C<U₁,...,Uₙ>) = σₙ ∘ ... ∘ σ₁
  where σᵢ = unify(σᵢ₋₁(Tᵢ), σᵢ₋₁(Uᵢ))
```

### 1.3 推断示例

```x
// 编译器自动推断所有类型
let x = 42                              // x : integer
let f = function(a, b) => a + b         // f : (integer, integer) -> integer
let xs = [1, 2, 3]                      // xs : [integer]
let result = xs |> map(function(n) => n * 2)   // result : [integer]
let pair = (true, "hello")              // pair : (boolean, string)
```

### 1.4 局部推断与顶层签名

- **局部类型推断**：函数体内几乎不需要任何类型注解
- **顶层签名**：公共 API 建议写类型签名以提升可读性
- **双向类型检查**：结合自上而下（期望类型）与自下而上（推断类型）两个方向

## 2. 代数数据类型

X 语言支持代数数据类型（Algebraic Data Types, ADTs），包括积类型（product types）和和类型（sum types）。

### 2.1 积类型

积类型表示多个值的组合，包括：

- **元组（Tuple）**：固定长度、异构类型的有序集合
  ```x
  let pair: (integer, string) = (42, "answer")
  ```

- **记录（Record）**：具名字段的积类型
  ```x
  type Point = {
      x: float,
      y: float
  }
  
  let origin: Point = { x: 0.0, y: 0.0 }
  ```

### 2.2 和类型

和类型表示多个可能类型的选择，主要通过**联合类型（Union）**实现：

```x
type Shape =
    | Circle { radius: float }
    | Rect { width: float, height: float }
    | Point

type Color =
    | Red
    | Green
    | Blue
    | Custom(integer, integer, integer)
```

### 2.3 常用代数数据类型

X 语言内置了几个常用的代数数据类型：

- **Option 类型**：表示可能缺失的值
  ```x
  Option<T> = Some(T) | None
  ```

- **Result 类型**：表示可能失败的操作
  ```x
  Result<T, E> = Ok(T) | Err(E)
  ```

### 2.4 模式匹配

代数数据类型通常通过模式匹配来使用：

```x
function area(shape: Shape) -> float {
    match shape {
        Circle { radius } => 3.14159 * radius * radius
        Rect { width, height } => width * height
        Point => 0.0
    }
}
```

## 3. 泛型

X 语言支持泛型（参数多态），允许定义与具体类型无关的代码。

### 3.1 泛型类型

```x
type Pair<A, B> = {
    first: A,
    second: B
}

let pair: Pair<integer, string> = { first: 42, second: "answer" }
```

### 3.2 泛型函数

```x
function identity<T>(x: T) -> T {
    x
}

function map<A, B>(list: [A], f: (A) -> B) -> [B] {
    // 实现细节
}
```

### 3.3 类型约束

泛型参数可以有类型约束，限制它们必须实现特定的 Trait：

```x
function sum<T: Numeric>(list: [T]) -> T {
    list |> reduce(function(a, b) => a.add(b))
}
```

### 3.4 类型别名

可以为复杂类型创建别名，包括泛型类型：

```x
type Name = string
type UserMap = {string: User}
type Callback<T> = (T) -> unit
```

## 4. Trait 系统

Trait 定义一组行为约束，类型可以实现 trait 以满足这些约束。

### 4.1 Trait 定义

```x
trait Printable {
    function show(): string
}

trait Numeric {
    function add(other: Self) -> Self
    function multiply(other: Self) -> Self
}
```

### 4.2 Trait 实现

```x
implement Printable for Point {
    function show() -> string {
        "(${this.x}, ${this.y})"
    }
}
```

### 4.3 Trait 约束

Trait 约束用于泛型参数，确保类型参数实现了特定的行为：

```x
function print_all<T: Printable>(items: [T]) -> unit {
    for item in items {
        println(item.show())
    }
}

function display_and_compare<T: Printable + Comparable>(a: T, b: T) -> string {
    if a > b { a.show() } else { b.show() }
}
```

### 4.4 子类型关系

X 语言支持子类型关系，其中 Never 类型是所有类型的子类型：

```
Never <: T  // 对所有类型 T
```

函数类型的子类型关系遵循**逆变参数，协变返回**的原则：

```
T'₁ <: T₁  ...  T'ₙ <: Tₙ    R <: R'    Δ ⊆ Δ'
────────────────────────────────────────────────────
(T₁, ..., Tₙ) -> R with Δ  <:  (T'₁, ..., T'ₙ) -> R' with Δ'
```

## 5. 类型转换

X 语言的类型转换系统设计注重类型安全，避免运行时类型错误。

### 5.1 隐式类型转换

X 语言在某些情况下会进行隐式类型转换：

- **数字类型**：从小范围类型到大范围类型的转换（如 integer 32 到 integer 64）
- **子类型**：子类型可以隐式转换为父类型
- **Never 类型**：Never 类型可以转换为任何类型

### 5.2 显式类型转换

对于需要显式转换的情况，X 语言提供了类型转换操作：

```x
let x: integer = 42
let y: float = float(x)  // 显式转换为浮点数

let s: string = "123"
let n: integer = integer(s)  // 字符串转整数，可能失败
```

### 5.3 类型安全

X 语言的类型转换设计确保了类型安全：

- 可能失败的转换返回 Result 类型
- 编译器在编译时检查类型转换的有效性
- 不存在运行时类型错误

### 5.4 类型组合

X 语言支持类型组合操作：

- **类型交集**：`T & U` 表示同时具有 T 和 U 的所有成员
- **类型联合**：`T | U` 表示具有 T 或 U 的成员

## 总结

X 语言的类型系统具有以下特点：

- **类型安全**：编译时检查类型错误，无运行时类型异常
- **类型推断**：基于 Hindley-Milner 算法的强大类型推断能力
- **代数数据类型**：支持积类型和和类型，表达能力强
- **泛型**：支持参数多态，代码复用性高
- **Trait 系统**：提供行为约束和接口抽象
- **无 null**：使用 Option 类型替代 null，消除空指针错误
- **无异常**：使用 Result 类型替代异常，错误处理显式可见

这些特性使得 X 语言在保证类型安全的同时，保持了代码的简洁性和表达能力。
