# 抽象类

抽象类是不能直接实例化的类——相反，它们旨在被子类化。抽象类可以包含抽象方法——声明但没有实现的方法。子类必须为所有抽象方法提供实现，或者它们自己也必须是抽象的。

## 定义抽象类

要在 X 语言中声明一个抽象类，我们使用 `abstract` 关键字：

```x
abstract class Shape {
  let color: String

  constructor(color: String) {
    this.color = color
  }

  // 抽象方法：没有实现
  abstract function area(self: &Self) -> Float

  // 具体方法：有实现
  function get_color(self: &Self) -> String {
    self.color.clone()
  }

  function set_color(self: &mut Self, color: String) {
    self.color = color
  }
}
```

抽象类可以包含：
- 字段（与普通类相同）
- 构造函数（与普通类相同）
- 抽象方法（使用 `abstract` 关键字，没有实现）
- 具体方法（有实现）

注意，我们不能直接实例化抽象类：

```x
// let shape = Shape::new(String::from("red"))  // 错误！无法实例化抽象类
```

## 实现抽象类

要使用抽象类，我们必须创建一个继承自它的具体子类，并为所有抽象方法提供实现：

```x
class Circle extends Shape {
  let radius: Float

  constructor(color: String, radius: Float) {
    super(color)
    this.radius = radius
  }

  // 必须实现抽象方法 area()
  override function area(self: &Self) -> Float {
    3.14159 * self.radius * self.radius
  }

  function get_radius(self: &Self) -> Float {
    self.radius
  }
}

class Rectangle extends Shape {
  let width: Float
  let height: Float

  constructor(color: String, width: Float, height: Float) {
    super(color)
    this.width = width
    this.height = height
  }

  // 必须实现抽象方法 area()
  override function area(self: &Self) -> Float {
    self.width * self.height
  }

  function get_width(self: &Self) -> Float {
    self.width
  }

  function get_height(self: &Self) -> Float {
    self.height
  }
}
```

现在我们可以实例化具体的子类：

```x
let circle = Circle::new(String::from("red"), 5.0)
let rectangle = Rectangle::new(String::from("blue"), 4.0, 6.0)

println("圆的颜色: ", circle.get_color())     // red
println("圆的面积: ", circle.area())            // ~78.53975

println("矩形的颜色: ", rectangle.get_color())  // blue
println("矩形的面积: ", rectangle.area())       // 24.0
```

## 多态与抽象类

抽象类对于多态特别有用——我们可以将子类的实例视为抽象类的实例：

```x
function print_shape_info(shape: &Shape) {
  println("形状颜色: ", shape.get_color())
  println("形状面积: ", shape.area())
}

let circle = Circle::new(String::from("red"), 5.0)
let rectangle = Rectangle::new(String::from("blue"), 4.0, 6.0)

print_shape_info(&circle)
print_shape_info(&rectangle)
```

## 抽象类与 Trait

你可能想知道什么时候应该使用抽象类，什么时候应该使用 trait。这里有一些指导原则：

### 使用抽象类的情况：
- 你想要在相关类之间共享代码
- 你需要在方法之间共享状态（字段）
- 你想要定义一个类层次结构的基础
- 你需要构造函数

### 使用 trait 的情况：
- 你想要将行为附加到不相关的类
- 你想要指定可以由任何类实现的契约
- 你想要支持多重继承的行为（一个类可以实现多个 trait）
- 你不需要共享状态

## 另一个抽象类示例：游戏角色

让我们看一个更具体的例子——游戏中的角色：

```x
abstract class GameCharacter {
  let name: String
  let mutable health: integer
  let max_health: integer

  constructor(name: String, max_health: integer) {
    this.name = name
    this.max_health = max_health
    this.health = max_health
  }

  // 所有角色都可以攻击，但攻击方式不同
  abstract function attack(self: &Self, target: &mut GameCharacter)

  // 所有角色都有相同的受伤方式
  function take_damage(self: &mut Self, amount: integer) {
    if self.health > amount {
      self.health = self.health - amount
      println(this.name, " 受到了 ", amount, " 点伤害！")
    } else {
      self.health = 0
      println(this.name, " 被击败了！")
    }
  }

  function is_alive(self: &Self) -> boolean {
    self.health > 0
  }

  function get_name(self: &Self) -> String {
    self.name.clone()
  }

  function get_health(self: &Self) -> integer {
    self.health
  }
}

class Warrior extends GameCharacter {
  let weapon_damage: integer

  constructor(name: String) {
    super(name, 100)
    this.weapon_damage = 25
  }

  override function attack(self: &Self, target: &mut GameCharacter) {
    println(this.name, " 用剑攻击！")
    target.take_damage(this.weapon_damage)
  }
}

class Mage extends GameCharacter {
  let spell_power: integer
  let mutable mana: integer

  constructor(name: String) {
    super(name, 70)
    this.spell_power = 30
    this.mana = 50
  }

  override function attack(self: &Self, target: &mut GameCharacter) {
    if this.mana >= 10 {
      println(this.name, " 施放火球术！")
      target.take_damage(this.spell_power)
      this.mana = this.mana - 10
    } else {
      println(this.name, " 法力不足！")
    }
  }
}
```

现在我们可以有不同类型的角色进行战斗：

```x
let mutable warrior = Warrior::new(String::from("Conan"))
let mutable mage = Mage::new(String::from("Gandalf"))

warrior.attack(&mut mage)
mage.attack(&mut warrior)

println(warrior.get_name(), " 的生命值: ", warrior.get_health())
println(mage.get_name(), " 的生命值: ", mage.get_health())
```

## 总结

抽象类在 X 语言中：
- 使用 `abstract class` 声明
- 不能直接实例化
- 可以包含抽象方法（使用 `abstract function`）
- 可以包含具体方法（有实现）
- 可以有字段和构造函数
- 必须被子类化，并且子类必须实现所有抽象方法
- 对于多态和定义类层次结构非常有用

抽象类与 trait 类似，但它们可以共享状态，并且旨在用于相关类的层次结构。

现在我们已经介绍了 X 语言中的面向对象特性，让我们继续讨论函数式编程特性！

