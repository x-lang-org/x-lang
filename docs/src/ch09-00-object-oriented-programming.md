# 面向对象编程

X 语言支持面向对象编程（OOP）范式，通过类、对象、继承等概念实现。本章将详细介绍 X 语言中的面向对象编程特性。

## 9.1 类定义

类是面向对象编程的核心构造，用于封装数据和行为。在 X 语言中，使用 `class` 关键字定义类。

### 基本语法

```x
class ClassName {
    // 字段声明
    let field1: Type
    let mutable field2: Type = defaultValue
    
    // 构造函数
    new(parameters) {
        // 初始化代码
    }
    
    // 方法
    function methodName(parameters) -> ReturnType {
        // 方法实现
    }
}
```

### 字段声明

字段是类中存储数据的成员，分为不可变和可变两种：

- **不可变字段**：使用 `let` 关键字声明，初始化后不能修改
- **可变字段**：使用 `let mutable` 关键字声明，可以在对象生命周期内修改
- **默认值**：字段可以在声明时指定默认值

```x
class Person {
    let name: String
    let mutable age: Integer = 0
    let isAdult: Boolean = age >= 18
}
```

### 构造函数

构造函数使用 `new` 关键字声明，用于初始化新创建的对象：

- 构造函数负责初始化对象的字段
- 可以有多个构造函数（重载）
- 子类构造函数必须调用父类构造函数

```x
class Person {
    let name: String
    let age: Integer
    
    new(name: String, age: Integer) {
        this.name = name
        this.age = age
    }
    
    // 便捷构造函数
    new(name: String) {
        this.name = name
        this.age = 0
    }
}
```

### 创建对象

要创建类的实例，使用类名加构造函数参数：

```x
let person = Person("Alice", 30)
let baby = Person("Bob")
```

## 9.2 继承

X 语言支持单继承，即每个类最多只能有一个父类。子类继承父类的非私有字段和方法。

### 基本语法

```x
class ChildClass extends ParentClass {
    // 子类特有的字段和方法
}
```

### 方法重写

子类可以重写父类的虚方法，使用 `override` 关键字标记：

- 只有标记为 `virtual` 的方法才能被重写
- 重写方法的签名必须与父类方法兼容
- 使用 `super` 关键字调用父类方法

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
    
    override function describe() -> String = "{brand} car at speed {speed}"
}
```

### 抽象类和抽象方法

抽象类使用 `abstract` 关键字标记，不能直接实例化：

- 抽象类可以包含抽象方法
- 抽象方法只有签名，没有实现
- 子类必须实现所有抽象方法

```x
abstract class Animal {
    abstract function speak() -> String
    
    function greet() -> String = "I say: {speak()}"
}

class Dog extends Animal {
    override function speak() -> String = "Woof!"
}

class Cat extends Animal {
    override function speak() -> String = "Meow!"
}
```

## 9.3 方法

方法是类中定义的函数，用于实现对象的行为。

### 方法类型

X 语言支持多种类型的方法：

1. **普通方法**：默认不可被子类重写
2. **虚方法**：使用 `virtual` 关键字标记，可被子类重写
3. **抽象方法**：使用 `abstract` 关键字标记，无实现
4. **静态方法**：使用 `static` 关键字标记，属于类而非实例
5. **最终方法**：使用 `final` 关键字标记，禁止子类重写

```x
class Calculator {
    // 普通方法
    function add(a: Integer, b: Integer) -> Integer {
        a + b
    }
    
    // 虚方法
    virtual function multiply(a: Integer, b: Integer) -> Integer {
        a * b
    }
    
    // 静态方法
    static function square(x: Integer) -> Integer {
        x * x
    }
    
    // 最终方法
    final function divide(a: Integer, b: Integer) -> Float {
        a.toFloat() / b.toFloat()
    }
}
```

### 方法调用

方法通过对象实例调用，或者对于静态方法，通过类名调用：

```x
let calc = Calculator()
let sum = calc.add(5, 3)  // 8
let product = calc.multiply(4, 6)  // 24
let squared = Calculator.square(7)  // 49
```

## 9.4 访问修饰符

X 语言提供了四种访问修饰符，用于控制类成员的可见性：

| 修饰符 | 同类 | 子类 | 同模块 | 外部 |
|--------|------|------|--------|------|
| `public` | ✓ | ✓ | ✓ | ✓ |
| `protected` | ✓ | ✓ | ✓ | ✗ |
| `internal` | ✓ | ✓ | ✓ | ✗ |
| `private` | ✓ | ✗ | ✗ | ✗ |

默认情况下，类成员的访问修饰符为 `private`。

### 示例

```x
class Account {
    private let id: Integer
    protected let mutable balance: Float
    public let owner: String
    internal let createdAt: String
    
    public new(id: Integer, owner: String, initialBalance: Float) {
        this.id = id
        this.owner = owner
        this.balance = initialBalance
        this.createdAt = "2023-01-01"
    }
    
    public function deposit(amount: Float) {
        balance = balance + amount
    }
    
    public function withdraw(amount: Float) -> Boolean {
        if amount > 0 && balance >= amount {
            balance = balance - amount
            true
        } else {
            false
        }
    }
    
    private function validate() -> Boolean {
        balance >= 0.0
    }
}
```

## 9.5 接口（Trait）

X 语言使用 trait 实现接口功能，定义一组行为契约：

- Trait 可以包含方法签名和默认实现
- 类通过 `implement` 关键字实现 trait
- 一个类可以实现多个 trait

```x
trait Printable {
    function show() -> String
}

trait Comparable<T> {
    function compareTo(other: T) -> Integer
    
    function lessThan(other: T) -> Boolean = compareTo(other) < 0
    function greaterThan(other: T) -> Boolean = compareTo(other) > 0
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

## 9.6 多态与动态分发

X 语言支持运行时多态，通过动态分发实现：

- 方法调用根据对象的实际类型选择实现
- 支持向上转型（子类对象赋值给父类类型）
- 遵循最具体实现优先原则

```x
class Shape {
    virtual function area() -> Float = 0.0
}

class Circle extends Shape {
    let radius: Float
    
    new(radius: Float) {
        this.radius = radius
    }
    
    override function area() -> Float = 3.14159 * radius * radius
}

class Rectangle extends Shape {
    let width: Float
    let height: Float
    
    new(width: Float, height: Float) {
        this.width = width
        this.height = height
    }
    
    override function area() -> Float = width * height
}

// 多态示例
let shapes: List<Shape> = [Circle(5.0), Rectangle(3.0, 4.0)]
for shape in shapes {
    println(shape.area())  // 运行时分发到对应子类的实现
}
```

## 9.7 最佳实践

### 类设计原则

1. **单一职责原则**：一个类应该只负责一项职责
2. **封装原则**：隐藏内部实现细节，只暴露必要的接口
3. **继承原则**：只有当子类真正是父类的一种特殊类型时才使用继承
4. **组合优先**：优先使用组合而不是继承来实现功能

### 代码风格

- 使用 PascalCase 命名类名
- 使用 camelCase 命名方法和字段
- 保持方法简洁，每个方法只做一件事
- 使用访问修饰符明确控制可见性

## 9.8 总结

X 语言的面向对象编程特性包括：

- **类定义**：使用 `class` 关键字定义类，包含字段和方法
- **继承**：支持单继承，通过 `extends` 关键字实现
- **方法**：支持普通方法、虚方法、抽象方法、静态方法和最终方法
- **访问控制**：提供 `public`、`protected`、`internal` 和 `private` 四种访问修饰符
- **接口**：通过 trait 实现接口功能
- **多态**：支持运行时多态和动态分发

这些特性使 X 语言能够构建复杂的面向对象系统，同时保持代码的清晰性和可维护性。
