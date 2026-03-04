# 继承

继承是面向对象编程中的一个机制，其中一个类基于另一个类，从它继承属性和方法。继承的类称为子类或派生类，被继承的类称为超类或基类。

继承促进了代码重用，并允许创建分层的类分类法。让我们看看继承在 X 语言中是如何工作的。

## 基本继承

要在 X 语言中声明一个继承自另一个类的类，我们在子类名后使用 `extends` 关键字，后跟超类名。

```x
class Animal {
  let name: String
  let age: integer

  constructor(name: String, age: integer) {
    this.name = name
    this.age = age
  }

  function make_sound(self: &Self) {
    println("动物发出声音！")
  }

  function get_name(self: &Self) -> String {
    self.name.clone()
  }

  function get_age(self: &Self) -> integer {
    self.age
  }
}

class Dog extends Animal {
  let breed: String

  constructor(name: String, age: integer, breed: String) {
    super(name, age)  // 调用超类构造函数
    this.breed = breed
  }

  override function make_sound(self: &Self) {
    println("汪汪！")
  }

  function get_breed(self: &Self) -> String {
    self.breed.clone()
  }

  function fetch(self: &Self) {
    println(self.name, " 正在去拿球！")
  }
}
```

让我们看看这里发生了什么：

1. **`extends Animal`**：声明 `Dog` 继承自 `Animal`
2. **`super(name, age)`**：调用超类的构造函数来初始化继承的字段
3. **`override`**：标记 `make_sound` 方法为覆盖超类中的方法
4. **新字段和方法**：`Dog` 添加了 `breed` 字段、`get_breed` 方法和 `fetch` 方法

## 使用子类

现在我们可以创建 `Dog` 的实例并使用继承和新的方法：

```x
let dog = Dog::new(String::from("Fido"), 3, String::from("金毛寻回犬"))

// 继承自 Animal 的方法
println("名字: ", dog.get_name())      // Fido
println("年龄: ", dog.get_age())        // 3

// 覆盖的方法
dog.make_sound()                      // 汪汪！

// Dog 特有的方法
println("品种: ", dog.get_breed())     // 金毛寻回犬
dog.fetch()                           // Fido 正在去拿球！
```

## 多态

继承的一个强大特性是多态——子类的实例可以被视为超类的实例。

```x
function make_animal_sound(animal: &Animal) {
  animal.make_sound()
}

let animal = Animal::new(String::from("Generic"), 5)
let dog = Dog::new(String::from("Fido"), 3, String::from("金毛寻回犬"))

make_animal_sound(&animal)  // 动物发出声音！
make_animal_sound(&dog)     // 汪汪！
```

尽管 `make_animal_sound` 函数接受 `&Animal`，我们可以传递 `&Dog`，因为 `Dog` 继承自 `Animal`。并且调用的 `make_sound` 方法是实际类型的方法——这就是多态！

## 覆盖方法

正如我们看到的，子类可以覆盖超类的方法，以提供特定于子类的行为。`override` 关键字是必需的，以明确我们有意覆盖一个方法。

```x
class Cat extends Animal {
  constructor(name: String, age: integer) {
    super(name, age)
  }

  override function make_sound(self: &Self) {
    println("喵喵！")
  }

  function purr(self: &Self) {
    println(self.name, " 正在发出咕噜声！")
  }
}

let cat = Cat::new(String::from("Whiskers"), 2)
cat.make_sound()  // 喵喵！
cat.purr()        // Whiskers 正在发出咕噜声！
```

## 调用超类方法

有时你可能想在覆盖的方法中调用超类的实现。你可以使用 `super` 关键字来做到这一点：

```x
class LoudDog extends Dog {
  constructor(name: String, age: integer, breed: String) {
    super(name, age, breed)
  }

  override function make_sound(self: &Self) {
    super.make_sound()  // 调用 Dog.make_sound()
    println("汪汪汪！！！")
  }
}

let loud_dog = LoudDog::new(String::from("Buddy"), 4, String::from("比格犬"))
loud_dog.make_sound()
// 输出:
// 汪汪！
// 汪汪汪！！！
```

## 保护成员

有时你希望子类可以访问成员，但不能从类外部公开访问。对于这种情况，X 语言有 `protected` 可见性：

```x
class Vehicle {
  protected let mutable speed: integer
  let max_speed: integer

  constructor(max_speed: integer) {
    this.speed = 0
    this.max_speed = max_speed
  }

  public function accelerate(self: &mut Self) {
    if this.speed < this.max_speed {
      this.speed = this.speed + 10
    }
  }

  public function get_speed(self: &Self) -> integer {
    this.speed
  }
}

class Car extends Vehicle {
  let model: String

  constructor(max_speed: integer, model: String) {
    super(max_speed)
    this.model = model
  }

  public function honk(self: &Self) {
    println("嘟嘟！这是一辆 ", this.model)
  }

  public function emergency_stop(self: &mut Self) {
    this.speed = 0  // 可以访问受保护的字段
  }
}

let car = Car::new(120, String::from("轿车"))
car.accelerate()
println("速度: ", car.get_speed())  // 10

// car.speed  // 错误！无法从外部访问受保护的字段

car.emergency_stop()
println("紧急停车后: ", car.get_speed())  // 0
```

## 继承层级

你可以创建多级继承的继承层级：

```x
class Animal { /* ... */ }
class Mammal extends Animal { /* ... */ }
class Dog extends Mammal { /* ... */ }
class GoldenRetriever extends Dog { /* ... */ }
```

但要小心——深层继承层级可能变得难以理解和维护。通常更喜欢组合而不是继承。

## 继承与组合

虽然继承很强大，但通常最好使用组合——在类中包含其他类的实例，而不是继承它们：

```x
// 使用继承
class Vehicle {
  let engine: Engine
  // ...
}

// 使用组合（通常更好）
class Vehicle {
  let engine: Engine
  let wheels: List<Wheel>
  // ...
}
```

经验法则是：当存在"is-a"关系时使用继承，当存在"has-a"关系时使用组合。

## 总结

X 语言中的继承提供：
- 使用 `extends` 进行类继承
- 使用 `super` 调用超类构造函数和方法
- 使用 `override` 覆盖方法
- 多态（子类可以用作超类）
- 用于子类访问的 `protected` 可见性
- 继承层级

但要记住，继承并不总是最好的工具——通常更喜欢组合而不是继承！

在下一章中，我们将讨论抽象类，这是一种不能直接实例化但可以被子类化的特殊类。
