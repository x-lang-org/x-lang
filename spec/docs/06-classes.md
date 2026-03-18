# 第6章 面向对象

## 6.1 类声明

### 形式语法

```
ClassDeclaration ::= AccessModifier? ClassModifier? 'class' Identifier TypeParameters?
                     ('extends' Type)?
                     ('implement' Type (',' Type)*)?
                     ClassBody

ClassModifier ::= 'abstract' | 'final'

ClassBody ::= '{' ClassMember* '}'

ClassMember ::= FieldDeclaration
              | MethodDeclaration
              | ConstructorDeclaration
              | StaticDeclaration
```

### 说明

类是 X 语言中面向对象编程的核心构造。一个类定义了一个新类型，封装数据（字段）和行为（方法）。

1. **类声明**使用 `class` 关键字，后跟类名和可选的类型参数。
2. **继承**通过 `extends` 关键字指定唯一父类（单继承）。
3. **Trait 实现**通过 `implement` 关键字列出该类实现的 trait（可多个）。
4. **类修饰符**：
   - `abstract`：抽象类不能直接实例化，可包含抽象方法
   - `final`：终止类不能被继承

```x
class Point {
    let x: Float
    let y: Float

    new(x: Float, y: Float) {
        this.x = x
        this.y = y
    }

    function distance(other: Point) -> Float {
        let dx = this.x - other.x
        let dy = this.y - other.y
        (dx * dx + dy * dy).sqrt()
    }
}

class ColorPoint extends Point implement Printable {
    let color: String

    new(x: Float, y: Float, color: String) {
        super(x, y)
        this.color = color
    }

    function show() -> String = "({x}, {y}, {color})"
}
```

---

## 6.2 字段声明

### 形式语法

```
FieldDeclaration ::= AccessModifier? 'let' 'mutable'? Identifier (':' Type)? ('=' Expression)?

AccessModifier ::= 'public' | 'private' | 'protected'
```

### 说明

字段定义对象的状态。

1. **不可变字段** `let`：初始化后不能修改（默认）。
2. **可变字段** `let mutable`：可以在对象生命周期内重新赋值。
3. **可见性**默认为 `private`（见 §6.8）。

```x
class User {
    let name: String                          // 不可变，private
    let mutable email: String                 // 可变，private
    public let id: Integer                    // 不可变，public
    protected let mutable loginCount: Integer = 0  // 可变，protected，有默认值
}
```

### 类型规则

$$
\frac{C \;\text{has field}\; f : T \;\text{with visibility}\; V \qquad V \;\text{allows access from}\; \text{ctx}}
     {\Gamma \;\vdash\; \text{obj} : C \implies \Gamma \;\vdash\; \text{obj}.f : T}
$$

---

## 6.3 方法声明

### 形式语法

```
MethodDeclaration ::= AccessModifier? MethodModifier? 'function' Identifier
                      Parameters ('->' Type)? ('with' EffectList)? FunctionBody

MethodModifier ::= 'virtual' | 'override' | 'abstract' | 'final' | 'static'
```

### 说明

方法定义对象的行为。方法内部隐式绑定 `this`，指向当前实例。

1. **普通方法**：默认不可被子类重写。
2. **`virtual` 方法**：声明为可被子类重写的方法。
3. **`override` 方法**：显式标记正在重写父类的虚方法。编译器检查父类中确实存在该虚方法。
4. **`abstract` 方法**：仅有签名无实现，所在类必须为 `abstract` 类，子类必须提供实现。
5. **`final` 方法**：禁止子类进一步重写。
6. **`static` 方法**：属于类而非实例，没有 `this`。

```x
class Shape {
    virtual function area() -> Float = 0.0

    function describe() -> String = "I am a shape"

    static function unit() -> Shape = Shape {}
}

class Circle extends Shape {
    let radius: Float

    new(radius: Float) {
        this.radius = radius
    }

    override function area() -> Float = 3.14159 * radius * radius
}

abstract class Animal {
    abstract function speak() -> String

    function greet() -> String = "I say: {speak()}"
}

class Dog extends Animal {
    override function speak() -> String = "Woof!"
}
```

### 类型规则

$$
\frac{\Gamma,\; \texttt{this}: C,\; x_1: T_1,\;\ldots,\; x_n: T_n \;\vdash\; e : R}
     {\Gamma \;\vdash\; C\!::\!m(x_1: T_1,\;\ldots,\; x_n: T_n) \;:\; (T_1,\;\ldots,\; T_n) \to R}
$$

$$
\frac{\text{obj} : C \qquad C \;\text{has method}\; m : (T_1,\;\ldots,\; T_n) \to R}
     {\Gamma \;\vdash\; \text{obj}.m(e_1,\;\ldots,\; e_n) : R}
$$

---

## 6.4 构造函数

### 形式语法

```
ConstructorDeclaration ::= AccessModifier? 'new' Parameters ConstructorBody

ConstructorBody ::= Block

SuperCall ::= 'super' '(' (Expression (',' Expression)*)? ')'
```

### 说明

构造函数使用 `new` 关键字声明，负责初始化新创建的对象。

1. **实例创建**：`ClassName(args)` 分配内存并调用构造函数。
2. **字段初始化**：构造函数体内通过 `this.field = value` 初始化字段。具有默认值的字段可以不在构造函数中显式赋值。
3. **父类构造调用**：子类构造函数中通过 `super(args)` 调用父类构造函数，必须作为构造函数体的第一条语句。

```x
class Rectangle {
    let width: Float
    let height: Float

    new(width: Float, height: Float) {
        this.width = width
        this.height = height
    }

    // 便捷构造函数
    new(side: Float) {
        this.width = side
        this.height = side
    }
}

let rect = Rectangle(10.0, 20.0)
let square = Rectangle(5.0)
```

### 求值规则

$$
\llbracket \text{ClassName}(e_1,\;\ldots,\; e_n) \rrbracket^g = \text{obj}
\quad\text{where}\quad
\begin{cases}
  v_i = \llbracket e_i \rrbracket^g \quad (1 \le i \le n) \\
  \text{obj} = \text{allocate}(C) \\
  \text{obj} = \text{init\_fields}(\text{obj},\; v_1,\;\ldots,\; v_n) \\
  \text{obj} = \text{run\_constructor}(\text{obj},\; v_1,\;\ldots,\; v_n)
\end{cases}
$$

---

## 6.5 继承

### 形式语法

```
ClassDeclaration ::= 'class' Identifier 'extends' Type ClassBody
```

### 说明

X 支持单继承——每个类最多继承一个父类。子类继承父类所有非 `private` 的字段和方法。

1. **单继承**：`class Child extends Parent { ... }`
2. **方法重写**：子类可以用 `override` 重写父类的 `virtual` 方法。签名必须兼容——参数类型逆变（contravariant），返回类型协变（covariant）。
3. **`super` 调用**：子类中通过 `super.method()` 调用父类方法的原始实现。

```x
class Vehicle {
    let mutable speed: Float = 0.0

    virtual function accelerate(amount: Float) {
        speed = speed + amount
    }

    function describe() -> String = "Vehicle at speed {speed}"
}

class Car extends Vehicle {
    let brand: String

    new(brand: String) {
        this.brand = brand
    }

    override function accelerate(amount: Float) {
        super.accelerate(amount * 1.5)
    }
}
```

### 类型规则

子类型关系：

$$
\frac{\vdash\; C \;\texttt{extends}\; D}
     {\vdash\; C \;<:\; D}
$$

向上转型：

$$
\frac{\Gamma \;\vdash\; \text{obj} : C \qquad C \;<:\; D}
     {\Gamma \;\vdash\; \text{obj} : D}
$$

重写合法性：

$$
\frac{m \;\text{in}\; C \;\text{overrides}\; m \;\text{in}\; D \qquad
      \text{param\_types}(C\!::\!m) \;<:\; \text{param\_types}(D\!::\!m) \qquad
      \text{return\_type}(D\!::\!m) \;<:\; \text{return\_type}(C\!::\!m)}
     {\text{override valid}}
$$

---

## 6.6 Trait（接口）

### 形式语法

```
TraitDeclaration ::= 'trait' Identifier TypeParameters? ('extends' TraitList)? TraitBody

TraitList ::= Type (',' Type)*

TraitBody ::= '{' TraitMember* '}'

TraitMember ::= MethodSignature
              | DefaultMethod

MethodSignature ::= 'function' Identifier Parameters ('->' Type)? ('with' EffectList)?

DefaultMethod ::= 'function' Identifier Parameters ('->' Type)? ('with' EffectList)? FunctionBody

TraitImplementation ::= 'class' Identifier ('implement' TraitList) ClassBody
```

### 说明

Trait 定义一组行为契约（方法签名），类通过 `implement` 关键字实现 trait。

1. **Trait 定义**：声明方法签名和可选的默认实现。
2. **Trait 继承**：trait 可以通过 `extends` 继承其他 trait。
3. **实现**：类使用 `implement` 列出所有实现的 trait，必须提供所有未给出默认实现的方法。

```x
trait Printable {
    function show() -> String
}

trait Comparable<T> {
    function compareTo(other: T) -> Integer

    function lessThan(other: T) -> Boolean = compareTo(other) < 0
    function greaterThan(other: T) -> Boolean = compareTo(other) > 0
}

trait Serializable extends Printable {
    function serialize() -> String
}

class User implement Printable, Comparable<User> {
    let name: String
    let age: Integer

    new(name: String, age: Integer) {
        this.name = name
        this.age = age
    }

    function show() -> String = "User({name}, {age})"

    function compareTo(other: User) -> Integer = this.age - other.age
}
```

**Trait 作为类型约束**用于泛型函数：

```x
function printAll<T: Printable>(items: List<T>) -> () with IO {
    for item in items {
        print(item.show())
    }
}
```

### 类型规则

$$
\frac{C \;\texttt{implement}\; T \qquad C \;\text{provides all methods required by}\; T}
     {\vdash\; C \;<:\; T}
$$

$$
\frac{\texttt{trait}\; T \;\{ \;\texttt{function}\; m(x: A) \to B; \quad \texttt{function}\; n(x: C) \to D = e \;\}}
     {C \;\texttt{implement}\; T \implies C \;\text{must define}\; m, \;\text{may override}\; n}
$$

---

## 6.7 多态与动态分发

### 说明

方法调用在运行时根据对象的**动态类型**（实际类型）选择实现——这是动态分发（dynamic dispatch）。X 使用单分派（基于 `this` 的类型），遵循最具体实现优先原则。

```x
let shapes: List<Shape> = [Circle(5.0), Rectangle(3.0, 4.0)]
for shape in shapes {
    print(shape.area())  // 运行时分发到 Circle.area() 或 Rectangle.area()
}
```

### 分发函数

$$
\text{dispatch} : \text{Type} \times \text{MethodName} \to \text{MethodImplementation}
$$

$$
\text{dispatch}(\text{obj},\; m) = \text{most\_specific\_implementation}(\text{type\_of}(\text{obj}),\; m)
$$

### 求值规则

$$
\llbracket \text{obj}.m(e_1,\;\ldots,\; e_n) \rrbracket^g = v
\quad\text{where}\quad
\begin{cases}
  C = \text{dynamic\_type}(\text{obj}) \\
  m_{\text{impl}} = \text{lookup\_method}(C,\; m) \\
  v_i = \llbracket e_i \rrbracket^g \quad (1 \le i \le n) \\
  v = \text{apply}(m_{\text{impl}},\; \text{obj},\; v_1,\;\ldots,\; v_n)
\end{cases}
$$

---

## 6.8 访问控制

### 形式定义

```
AccessModifier ::= 'public' | 'private' | 'protected' | 'internal'
```

### 说明

X 使用全称关键字进行访问控制（不使用缩写如 `pub`），默认可见性为 `private`。

| 修饰符 | 同类 | 子类 | 同模块 | 外部 |
|-------------|------|------|--------|------|
| `public` | ✓ | ✓ | ✓ | ✓ |
| `protected` | ✓ | ✓ | ✓ | ✗ |
| `internal` | ✓ | ✓ | ✓ | ✗ |
| `private` | ✓ | ✗ | ✗ | ✗ |

```x
class Account {
    private let id: Integer
    protected let mutable balance: Float
    public let owner: String
    internal let createdAt: String

    public function deposit(amount: Float) {
        balance = balance + amount
    }

    private function validate() -> Boolean {
        balance >= 0.0
    }
}
```

### 形式化

$$
\text{accessible}(\text{from},\; \text{member}) =
\begin{cases}
  \text{true} & \text{if}\; \text{member.visibility} = \texttt{public} \\
  \text{from} = \text{member.declaring\_class} & \text{if}\; \text{member.visibility} = \texttt{private} \\
  \text{from} \in \text{member.declaring\_class} \cup \text{subclasses} & \text{if}\; \text{member.visibility} = \texttt{protected} \\
  \text{from.module} = \text{member.module} & \text{if}\; \text{member.visibility} = \texttt{internal}
\end{cases}
$$

---

**本章规范使用 `function` 全称关键字定义方法、`implement` 关键字实现 trait、`let mutable` 声明可变字段，结合数学形式化与 X 代码示例定义面向对象语义。**
