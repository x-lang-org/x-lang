# X语言规格说明书

## 设计哲学

### 核心原则

X语言是一门现代编程语言，设计遵循以下核心原则：

- **自然语言语法**：使用`needs`、`given`、`wait`、`when/is`、`can`、`atomic`等自然语言关键字，代码读起来像英文句子
- **数学直觉**：支持数学风格的函数定义和集合表示法，语法遵循数学书写习惯
- **副作用可见**：函数类型签名包含R·E·A，依赖、错误、返回值一目了然，无隐藏副作用
- **Perceus内存管理**：编译期精确插入dup/drop，垃圾即时回收，重用分析让函数式代码原地更新
- **无null·无异常**：Option代替null，Result代替异常，编译器强制处理
- **不可变优先**：绑定默认不可变，Perceus的重用分析依赖不可变性——唯一引用才能原地更新

## 四种编程范式

X语言支持四种编程范式，开发者可以根据场景选择最适合的方式：

### 函数式（数学 + 管道）
```x
val topUsers = users |> filter(.active) |> sortBy(.score) |> take(10)
```

### 声明式（自然语言 where/sort by）
```x
val topUsers = users
  where   .active and .score > 80
  sort by .score descending
  take    10
```

### 面向对象（方法链）
```x
val topUsers = users.filter(.active).sortBy(.score).take(10)
```

### 过程式（var + for）
```x
fun getTopUsers() {
  var result = []
  for u in users {
    if u.active and u.score > 80 { result.add(u) }
  }
  result |> sortBy(.score) |> take(10)
}
```

---

## 1. 词法结构

### 1.1 标识符
- **命名规则**：标识符由字母、数字、下划线和连字符组成，但不能以数字或连字符开头
- **大小写敏感**：`userName` 和 `Username` 是不同的标识符
- **关键字**：不能使用关键字作为标识符
- **示例**：
  ```x
  validName      // 合法
  user_name      // 合法
  user-name      // 合法（推荐）
  123name        // 非法（以数字开头）
  -user          // 非法（以连字符开头）
  ```

### 1.2 数字字面量
- **整数**：支持十进制、十六进制（`0x`前缀）、八进制（`0o`前缀）、二进制（`0b`前缀）
- **浮点数**：支持小数点、科学计数法（`e`或`E`）
- **示例**：
  ```x
  42              // 十进制
  0x2A            // 十六进制
  0o52            // 八进制
  0b101010        // 二进制
  3.14            // 浮点数
  6.02e23         // 科学计数法
  1_000_000       // 数字分隔符（提高可读性）
  ```

### 1.3 字符串字面量
- **普通字符串**：使用双引号 `"`，支持转义字符
- **多行字符串**：使用三个双引号 `"""`，保留换行和缩进
- **插值字符串**：使用 `{}` 嵌入表达式
- **示例**：
  ```x
  "Hello, World!"             // 普通字符串
  """
  多行字符串
  保留格式
  """
  "Hello, {name}!"            // 字符串插值
  ```

### 1.4 布尔值
- `true` 和 `false` 是布尔类型的字面量
- **示例**：
  ```x
  val isActive = true
  val hasError = false
  ```

### 1.5 特殊字符
- **注释**：
  - 单行注释：`-- 注释内容`
  - 多行注释：`{- 多行注释 -}`
- **分隔符**：逗号 `,` 用于分隔列表项，分号 `;` 用于分隔语句（可选）

---

## 2. 类型系统

### 2.1 基本类型
```x
Int         // 整数类型（任意精度）
Float       // 浮点数类型（双精度）
Bool        // 布尔类型
Char        // 字符类型
String      // 字符串类型
Unit        // 单元类型（无返回值）
Never       // 永不存在的类型（永不返回）
```

### 2.2 复合类型

#### 记录类型
```x
type Point = {
  x: Float,
  y: Float
}

type User  = { id: Int, name: String, email: String }
```

#### 联合类型
```x
type Shape =
  | Circle  { radius: Float }
  | Rect    { width: Float, height: Float }
  | Point

type Color = Red | Green | Blue | Custom { r: Int, g: Int, b: Int }
```

#### 列表和字典
```x
[T]            // 列表（Perceus 下大多数操作可原地执行）
{K: V}         // 字典
```

### 2.3 高级类型

#### 选项类型（代替null）
```x
Option<T>    // Some(v) | None
```

#### 结果类型（代替异常）
```x
Result<T, E>   // Ok(v) | Err(e)
```

#### 函数类型
```x
(T1, T2) -> T3 // 接受T1和T2，返回T3的函数
```

#### 异步类型
```x
async<T>     // 异步计算结果
```

### 2.4 类型操作

#### copy-with 更新（不可变，产生新值）
```x
val p2 = point with { x: 5.0 }
```

#### Trait 接口
```x
trait Printable { show(): String }
trait Comparable<T> { compare(other: T): Int }
```

---

## 3. 名字与作用域

### 3.1 作用域类型
- **全局作用域**：整个程序可见
- **模块作用域**：模块内可见
- **函数作用域**：函数内可见
- **块作用域**：代码块内可见（如if/for/while块）

### 3.2 作用域规则
- **词法作用域**：变量的可见性由代码的物理位置决定
- **遮蔽规则**：内层作用域的变量会遮蔽外层作用域的同名变量
- **示例**：
  ```x
  val x = 10  // 全局作用域

  fun foo() {
    val x = 20  // 函数作用域，遮蔽全局变量
    {
      val x = 30  // 块作用域，遮蔽函数变量
    }
  }
  ```

### 3.3 名字解析
- 按照从内到外的顺序查找变量
- 支持`this`关键字访问当前对象的成员
- 支持`super`关键字访问父类成员

---

## 4. 变量

### 4.1 变量声明
```x
// 不可变绑定（值）
val name = "Alice"
val age = 30

// 显式类型标注
val name: String = "Alice"
val age: Int = 30

// 可变变量
var count = 0
var isActive: Bool = true
```

### 4.2 变量解构
```x
// 元组解构
val (x, y) = (10, 20)

// 记录解构
val { name, age } = user

// 列表解构
val [first, ...rest] = [1, 2, 3, 4]
```

### 4.3 变量作用域
```x
fun example() {
  val x = 10  // 函数作用域

  if true {
    val y = 20  // 块作用域
  }

  // 错误：y在此处不可见
  // print(y)
}
```

---

## 5. 修饰符

### 5.1 访问修饰符
```x
// 公共成员（默认）
public val name = "Alice"

// 私有成员（仅当前类可见）
private val secret = "password"

// 保护成员（当前类和子类可见）
protected val familyName = "Smith"

// 模块私有（仅当前模块可见）
module val internal = 42
```

### 5.2 其他修饰符
```x
// 静态成员
static count = 0

// 抽象成员（必须在子类中实现）
abstract fun calculate(): Float

// 最终成员（不可被重写）
final fun toString() = "Object"

// 可重载
overloadable fun add(a: Int, b: Int) = a + b
overloadable fun add(a: Float, b: Float) = a + b
```

---

## 6. 表达式

### 6.1 基本表达式
```x
// 字面量表达式
42
3.14
true
"Hello"

// 变量引用
x
name

// 成员访问
user.name
point.x
```

### 6.2 算术表达式
```x
// 基本运算
a + b   // 加法
a - b   // 减法
a * b   // 乘法
a / b   // 除法
a % b   // 取余

// 幂运算
x ^ 2   // x的平方

// 复合赋值
x += 5  // x = x + 5
x -= 3  // x = x - 3
```

### 6.3 逻辑表达式
```x
// 逻辑运算
a and b   // 逻辑与
a or b    // 逻辑或
not a     // 逻辑非

// 短路评估
x > 0 and y < 10  // 如果x<=0，y<10不会计算
```

### 6.4 比较表达式
```x
a == b    // 相等
a != b    // 不等
a < b     // 小于
a > b     // 大于
a <= b    // 小于等于
a >= b    // 大于等于
```

### 6.5 类型检查表达式
```x
// 类型检查
x is Int
y is String

// 类型转换
x as Float
```

### 6.6 函数调用表达式
```x
// 普通函数调用
val result = add(2, 3)

// 方法调用
user.toString()

// 管道操作
data |> process |> filter |> sort
```

---

## 7. 函数

### 7.1 函数定义

#### 数学风格函数
```x
add(a, b) = a + b
square(x) = x ^ 2

// 显式类型标注
add(a: Int, b: Int): Int = a + b
square(x: Float): Float = x ^ 2
```

#### 多行函数
```x
fun factorial(n: Int): Int {
  if n <= 1 {
    1
  } else {
    n * factorial(n - 1)
  }
}
```

#### 异步函数
```x
async fun fetchData(): String {
  wait delay(1000)
  "Data"
}
```

#### 分段函数（数学分段符号）
```x
fib(0) = 0
fib(1) = 1
fib(n) = fib(n-1) + fib(n-2)

sign(0) = 0
sign(n) = when n > 0 then 1 else -1
```

### 7.2 函数参数
```x
// 默认参数
fun greet(name: String = "World") = "Hello, {name}!"

// 可变参数
fun sum(...numbers: Int[]): Int {
  numbers |> reduce(0, (acc, x) -> acc + x)
}

// 具名参数
greet(name: "Alice")
sum(1, 2, 3)
```

### 7.3 函数返回
```x
// 隐式返回（最后一个表达式的值）
fun add(a, b) = a + b

// 显式返回
fun findUser(id: Int): Option<User> {
  val user = database.query(id)
  if user != null {
    return Some(user)
  }
  None
}
```

### 7.4 Lambda函数
```x
// 简单lambda
(x) -> x * 2

// 多行lambda
(x, y) -> {
  val z = x + y
  z * z
}

// 点语法简写
users |> map(.name)    // 等价于 map((u) -> u.name)
```

---

## 8. 类和接口

### 8.1 类定义
```x
class Animal {
  name: String
  age: Int

  // 主构造函数
  new(name: String, age: Int) {
    this.name = name
    this.age = age
  }

  // 方法
  greet() = "I'm {name}"

  // 可重写方法
  virtual birthday() = this with { age: age + 1 }
}

// 创建实例
val animal = Animal("Bob", 3)
```

### 8.2 继承
```x
class Dog extends Animal {
  breed: String

  // 子构造函数
  new(name: String, age: Int, breed: String) {
    super(name, age)
    this.breed = breed
  }

  // 重写方法
  override greet() = "Woof! I'm {name}, a {breed}"
}
```

### 8.3 接口
```x
trait Printable {
  fun show(): String
}

trait Comparable<T> {
  fun compare(other: T): Int
}

// 实现接口
class User implements Printable, Comparable<User> {
  name: String
  age: Int

  fun show(): String = "User: {name}"

  fun compare(other: User): Int {
    if age < other.age { -1 }
    else if age > other.age { 1 }
    else { 0 }
  }
}
```

### 8.4 抽象类
```x
abstract class Shape {
  abstract fun area(): Float
  abstract fun perimeter(): Float

  fun description() = "A shape"
}

class Circle extends Shape {
  radius: Float

  fun area(): Float = pi * radius ^ 2
  fun perimeter(): Float = 2 * pi * radius
}
```

---

## 9. 属性

### 9.1 简单属性
```x
class Person {
  // 只读属性
  get fullName(): String = "{firstName} {lastName}"

  // 读写属性
  var _age: Int = 0
  get age(): Int = _age
  set age(value: Int) {
    if value >= 0 {
      _age = value
    }
  }

  // 计算属性
  get isAdult(): Bool = age >= 18
}
```

### 9.2 属性访问器
```x
val person = Person()
person.age = 25  // 调用setter
print(person.age)  // 调用getter
print(person.fullName)  // 调用getter
```

---

## 10. 扩展

### 10.1 扩展函数
```x
// 为现有类型添加方法
extension String {
  fun toInt(): Option<Int> {
    // 解析字符串为整数
  }

  fun isPalindrome(): Bool {
    this == reverse(this)
  }
}

// 使用扩展方法
"123".toInt()       // 结果：Some(123)
"abcba".isPalindrome()  // 结果：true
```

### 10.2 扩展属性
```x
// 为现有类型添加属性
extension Int {
  get isEven(): Bool = this % 2 == 0
  get isOdd(): Bool = this % 2 == 1
}

// 使用扩展属性
4.isEven()  // 结果：true
5.isOdd()   // 结果：true
```

---

## 11. 泛型

### 11.1 泛型类型
```x
// 泛型类
class Stack<T> {
  private items: [T] = []

  fun push(item: T) {
    items.append(item)
  }

  fun pop(): Option<T> {
    if items.isEmpty() {
      None
    } else {
      Some(items.removeLast())
    }
  }
}

// 使用泛型类
val intStack = Stack<Int>()
val stringStack = Stack<String>()
```

### 11.2 泛型函数
```x
// 泛型函数
fun identity<T>(value: T): T = value

fun map<T, R>(list: [T], transform: (T) -> R): [R] {
  [transform(x) for x in list]
}

// 使用泛型函数
val result = map([1, 2, 3], (x) -> x * 2)  // 结果：[2, 4, 6]
```

### 11.3 泛型接口
```x
// 泛型接口
trait Comparable<T> {
  fun compare(other: T): Int
}

class Person implements Comparable<Person> {
  fun compare(other: Person): Int {
    // 比较逻辑
  }
}
```

### 11.4 类型约束
```x
// 带约束的泛型
fun printAll<T: Printable>(items: [T]) {
  for item in items {
    print(item.show())
  }
}

// 使用带约束的泛型
printAll([Person(), Animal()])
```

---

## 12. 重载

### 12.1 函数重载
```x
// 同名函数，不同参数类型
fun add(a: Int, b: Int): Int = a + b
fun add(a: Float, b: Float): Float = a + b
fun add(a: String, b: String): String = a + b

// 调用时根据参数类型选择
val result1 = add(1, 2)        // 调用Int版本
val result2 = add(1.5, 2.5)    // 调用Float版本
val result3 = add("a", "b")    // 调用String版本
```

### 12.2 操作符重载
```x
// 重载加法操作符
class Vector {
  x: Float
  y: Float

  operator +(other: Vector): Vector {
    Vector(x + other.x, y + other.y)
  }

  operator -(other: Vector): Vector {
    Vector(x - other.x, y - other.y)
  }
}

// 使用重载的操作符
val v1 = Vector(1, 2)
val v2 = Vector(3, 4)
val v3 = v1 + v2  // 结果：Vector(4, 6)
```

### 12.3 方法重载
```x
class MathUtil {
  fun compute(x: Int): Int = x * 2
  fun compute(x: Float): Float = x * 3.0
}

// 使用重载的方法
val util = MathUtil()
val result1 = util.compute(5)    // 调用Int版本：10
val result2 = util.compute(5.0)  // 调用Float版本：15.0
```

---

## 13. 模式匹配

### 类型联合匹配（编译器保证穷举）
```x
fun area(shape: Shape) =
  when shape is
    Circle { radius }        -> pi * radius ^ 2
    Rect   { width, height } -> width * height
    Point                    -> 0.0
```

### 守卫条件（where）
```x
fun grade(score: Int) =
  when score is
    s where s >= 90 -> "A"
    s where s >= 75 -> "B"
    s where s >= 60 -> "C"
    _               -> "F"
```

### 内联三元（when/then/else）
```x
val label = when x > 0 then "pos" else "non-pos"
```

---

## 14. 集合与推导式

### 列表推导（数学集合表示法）
```x
val evens      = [x       | x in 1..100, x mod 2 == 0]
val squares    = [x^2     | x in 1..10]
val names      = [u.name  | u in users, u.active]
val pairs      = [(x,y)   | x in 1..3, y in 1..3, x != y]
```

### 字典推导
```x
val scoreMap = {u.id: u.score | u in users}
```

### 范围
```x
1..10     // [1..9]  不含末尾
1...10    // [1..10] 含末尾
```

### 常用操作
```x
users |> map(.name)
users |> filter(.active)
users |> sortBy(.score)
users |> groupBy(.department)
users |> reduce(0, (acc, u) -> acc + u.score)
```

---

## 15. 管道与声明式

### 函数管道
```x
val result = users |> filter(.active) |> sortBy(.score) |> take(10)
```

### SQL 风格（最接近自然语言）
```x
val result = users
  where   .active and .score > 80
  sort by .score descending
  take    10
  select  .name
```

### 效果管道（|>> 有副作用的链式操作）
```x
fetchUser(id)
  |>> tap((u) -> log("Got: {u.name}"))
  |>> map(.name)
```

---

## 16. Effect系统

### 声明依赖（needs 关键字）
```x
fun getUser(id: Int): User needs Database, Logger {
  val rows = Database.query("SELECT * FROM users WHERE id = {id}")?
  log("Fetched user {id}")
  rows.first() or fail(NotFound { id })
}
```

### 注入实现（given 关键字）
```x
getUser(42) given {
  Database <- PostgresDatabase.live
  Logger   <- ConsoleLogger.live
}
```

### 测试：换成 mock
```x
test "getUser works" {
  getUser(1) given {
    Database <- MockDatabase.with([testUser])
    Logger   <- SilentLogger
  } should equal(testUser)
}
```

---

## 17. 异步与并发

### 异步函数
```x
async fun loadDashboard(uid: Int): Dashboard needs Http, Database {
  val (user, orders, notices) = wait together {
    fetchUser(uid),
    fetchOrders(uid),
    fetchNotices(uid)
  }?
  Dashboard { user, orders, notices }
}
```

### 并发操作
```x
val result = wait race { fetchPrimary(), fetchReplica() }

val data = wait heavyTask() timeout 5.seconds or fail(Timeout)
```

---

## 18. 原子事务

### STM（软件事务内存）
```x
val balance = TVar(1000.0)

transfer(amount: Float) = atomic {
  if balance.read() < amount { retry }
  balance.update((b) -> b - amount)
}
```

---

## 19. Perceus内存模型（核心章节）

### 什么是 Perceus

Perceus 是由 Microsoft Research 的 Daan Leijen 团队在 PLDI 2021 发表的精确引用计数算法。X语言将 Perceus 原生集成为唯一的内存管理策略，彻底取代 GC 和 ORC。

#### Perceus 的两个核心定理：
1. **垃圾自由（Garbage Free）**：对象在不可能再被引用的那一刻立即释放。不存在"悬而未决"的垃圾。
2. **精确性（Precise）**：所有中间状态都不持有多余引用。通过线性资源演算在编译期严格证明。

### Perceus 四大机制

#### 1. 精确 RC 插入
编译器通过线性资源演算分析每个变量的使用情况，在最晚时刻插入 dup（引用+1），在最早时刻插入 drop（引用-1 直至释放）。

#### 2. 重用分析 (Reuse)
当对象引用计数恰好为 1（唯一引用）时，下一次同形状分配可以直接复用该内存块，零 malloc / free 开销。

#### 3. FBIP 范式
Functional But In-Place。纯函数式代码在对象唯一时自动原地执行，表现得像可变更新但语义上仍是纯函数。

#### 4. 特化 (Specialization)
针对已知唯一的对象生成特化代码路径，省去运行时 RC 判断开销，直接原地操作。

### 精确 dup/drop 插入示例

**X语言源码：**
```x
map(f, [])     = []
map(f, x::xs)  = f(x) :: map(f, xs)
```

**Perceus 生成的 IR（概念示意）：**
```x
fun map(f, xs):
  when xs is
    []    -> drop(f); []
    x::xs ->
      dup(f)             // f 被用两次，延迟到此处 dup
      val head = f(x)        // x 在这里消费，无需额外操作
      val tail = map(f, xs)  // xs 在这里消费
                         // 注意：原来的 cons 节点（x::xs）已被析构
                         // 若 rc == 1，可以重用该内存！（见重用分析）
      head :: tail
```

### 重用分析示例

**传统 RC 执行（每个节点都要 free + malloc）：**
```x
[1, 2, 3]  →  free(node1)  →  malloc()  →  [2, 3, 4]
               free(node2)     malloc()
               free(node3)     malloc()
3 次 free + 3 次 malloc = 6 次堆操作
```

**Perceus 重用分析（rc=1 → 直接复用节点内存）：**
```x
[1, 2, 3]  →  node1.rc=1  →  原地写入 2  →  [2, 3, 4]
               node2.rc=1     原地写入 3
               node3.rc=1     原地写入 4
0 次 free + 0 次 malloc = 0 次堆操作 ✓
```

### FBIP：函数式但原地执行

**X语言源码（完全相同！）：**
```x
insert(Leaf, v)    = Node(Leaf,v,Leaf)
insert(Node(l,x,r), v) =
  when v < x then
    Node(insert(l,v), x, r)
  else
    Node(l, x, insert(r,v))
```

**Perceus 优化：**
- 当树是唯一引用时：0 次 malloc，原地改写
- 当树是共享引用：自动 COW（写时复制）

### 循环引用处理

Perceus 原论文针对纯函数式程序（不可变 ADT 不会形成循环）。X语言支持 class（可能有循环），通过 `weak` 引用在类型系统层面打破循环，不需要运行时循环检测器。

```x
// 父子引用（经典循环场景）
class TreeNode {
  value:    Int
  children: [TreeNode]
  parent:   weak Option<TreeNode>  // weak：不参与 RC，不会形成强引用循环
}

// 双向链表
class ListNode<T> {
  value: T
  next:  Option<ListNode<T>>
  prev:  weak Option<ListNode<T>>  // 反向链接用 weak
}

// weak 引用语义
// weak 读取返回 Option：如果原对象已释放，返回 None
when node.parent.upgrade() is
  Some(p) -> doSomething(p)   // 对象还在
  None    -> handleOrphan()   // 父节点已被释放
```

---

## 20. 错误处理

### ? 传播，catch 捕获，or 默认值
```x
fun processOrder(id: Int): Receipt needs Database {
  user  = fetchUser(id)?
  cart  = fetchCart(user.id)?
  order = placeOrder(cart)?
  Receipt.from(order)
}

name = findUser(42) or "unknown"

result = fetchUser(42)
  |>> catch(NotFound, (_) -> User.guest)
```

---

## 21. 包和模块管理

### 包定义
```x
// package.luma文件（包描述文件）
name = "com.example.utils"
version = "1.0.0"
description = "示例工具库"

dependencies {
  "com.example.core": "2.0.0"
  "org.json": "1.0.0"
}

exports {
  "com.example.utils.string"
  "com.example.utils.math"
}
```

### 模块声明
```x
// 模块声明（文件顶部）
module com.example.utils.string

// 导出符号
export fun toCamelCase(s: String): String
export fun toSnakeCase(s: String): String
```

### 导入模块
```x
// 导入整个模块
import com.example.utils.string

// 导入特定符号
import com.example.utils.string.toCamelCase
import com.example.utils.string.toSnakeCase as snakeCase

// 导入所有符号
import com.example.utils.string.*
```

### 模块结构
```
src/
└── com/
    └── example/
        └── utils/
            ├── string/
            │   ├── package.luma
            │   └── operations.luma
            ├── math/
            │   ├── package.luma
            │   └── calculations.luma
            └── package.luma
```

### 模块访问控制
```x
// 导出的成员（公共API）
export fun publicAPI() { }

// 模块内可见的成员
internal fun internalHelper() { }

// 私有成员
private fun privateHelper() { }
```

---

## 22. 异常处理

### 异常类型
```x
// 异常基类
class Exception {
  message: String
  stackTrace: [String]

  new(message: String) {
    this.message = message
  }
}

// 自定义异常
class FileNotFoundException extends Exception {
  path: String

  new(path: String) {
    super("File not found: {path}")
    this.path = path
  }
}

class IllegalArgumentException extends Exception {
  paramName: String

  new(paramName: String, message: String) {
    super("Illegal argument: {paramName}: {message}")
    this.paramName = paramName
  }
}
```

### 抛出异常
```x
fun readFile(path: String): String {
  if !fileExists(path) {
    throw FileNotFoundException(path)
  }

  if !isReadable(path) {
    throw Exception("File is not readable")
  }

  // 读取文件内容
}
```

### 捕获异常
```x
// 基本异常捕获
try {
  content = readFile("nonexistent.txt")
} catch e: FileNotFoundException {
  print("Error: File not found: {e.path}")
} catch e: Exception {
  print("Error: {e.message}")
}

// 捕获所有异常
try {
  riskyOperation()
} catch e {
  print("Unexpected error: {e}")
}
```

### 资源管理
```x
// 自动资源释放（try-with-resources）
try {
  file = File.open("data.txt")
  content = file.read()
} catch e: IOException {
  print("Error reading file: {e}")
} finally {
  file.close()
}

// 使用use关键字（语法糖）
File.open("data.txt") use file {
  content = file.read()
}  // 自动调用close()
```

### 异常传播
```x
// 声明可能抛出异常的函数
fun riskyOperation(): String throws IOException, SQLException {
  // 可能抛出异常的代码
}

// 调用时必须处理或重新抛出
try {
  result = riskyOperation()
} catch e {
  // 处理异常
}
```

---

## 23. 关键字速查

| 关键字 | 类别 | 含义 | 示例 |
|--------|------|------|------|
| fun / async fun | 结构 | 函数 / 异步函数 | `fun f(x): T { ... }` |
| class / extends | 结构 | 类 / 继承 | `class Dog extends Animal` |
| trait / type | 结构 | 接口 / 类型定义 | `trait Printable { show() }` |
| var | 结构 | 可变变量 | `var count = 0` |
| for / in / if / else | 结构 | 控制流 | `for x in list { ... }` |
| spawn / weak | 结构 | 并发 Fiber / 弱引用 | `weak Option<Parent>` |
| needs | 自然 | 声明 Effect 环境依赖 | `fun f(): T needs Database` |
| given | 自然 | 注入依赖实现 | `f() given { Db <- Pg.live }` |
| wait | 自然 | 等待异步（替代 await） | `user = wait fetchUser(id)` |
| wait together | 自然 | 并行等待全部完成 | `(a,b) = wait together { f(), g() }` |
| wait race | 自然 | 取最快完成的结果 | `r = wait race { f(), g() }` |
| when / is | 自然 | 模式匹配 | `when x is Circle { r } -> ...` |
| where | 自然 | 守卫条件 / SQL 式过滤 | `users where .active` |
| can | 自然 | 实现 trait | `User can Printable { ... }` |
| atomic / retry | 自然 | STM 原子事务 | `atomic { balance -= 100 }` |
| use | 自然 | 资源自动释放 | `Db.open() use conn { ... }` |
| with | 自然 | copy-with 更新 | `user with { age: 31 }` |
| or | 自然 | 默认值 / 逻辑或 | `find(id) or "unknown"` |
| dup / drop | Perceus IR | 编译器自动生成，不可手写 | IR 内部使用 |
| drop_reuse / reuse | Perceus IR | 重用分析生成的内存复用指令 | IR 内部使用 |

---

## 24. 内存方案对比

| 特性 | Perceus（X语言） | ORC（Nim） | GC（C#/Go） | ARC（Swift） | Ownership（Rust） |
|------|-----------------|------------|------------|-------------|------------------|
| GC 暂停 | ❌ 无 | ❌ 无 | ⚠️ 有 | ❌ 无 | ❌ 无 |
| 内存精确释放 | ✅ 精确 | ⚠️ 近似 | ❌ 延迟 | ⚠️ 近似 | ✅ 精确 |
| 重用分析（零分配） | ✅ FBIP | ❌ | ❌ | ❌ | ❌ |
| 循环引用 | weak 引用 | 周期扫描 | ✅ 自动 | weak 引用 | ✅ 编译期 |
| 学习曲线 | 低（自动） | 低 | 低 | 低 | 高（借用检查器） |
| 理论基础 | 线性资源演算 | 经验主义 | 三色标记等 | 无正式证明 | 仿射类型 |
| 函数式代码性能 | 极高（FBIP） | 中 | 中 | 中 | 高 |
| 已有语言实现 | Koka · Lean 4 | Nim | C# · Go · JVM | Swift | Rust |

---

## 25. 完整示例

### 综合示例

#### 数学函数 + 集合推导（Perceus: 零分配）
```x
square(x)    = x ^ 2
norm({x, y}) = (x^2 + y^2) ^ 0.5

primes       = [n | n in 2..1000, isPrime(n)]
pythagorean  = [(a,b,c) | a in 1..20, b in a..20, c in b..20,
                           a^2 + b^2 == c^2]

-- FBIP：对列表做变换，若是唯一引用则零 malloc
doubled = [x * 2 | x in bigList]
```

#### 完整 Web 服务
```x
use http, db, log

type Item  = { id: Int, price: Float }
type Order = { id: Int, userId: Int, items: [Item] }
type OrderError =
  | UserNotFound       { id: Int }
  | InsufficientFunds  { needed: Float }
  | OutOfStock         { itemId: Int }

-- 纯函数（Perceus FBIP 下原地执行）
total(items: [Item]): Float = [i.price | i in items] |> sum

-- 业务逻辑（过程式 + Effect）
async fun placeOrder(userId: Int, items: [Item]): Order
    needs Database, Http, Logger {

  (user, stockOk) = wait together {
    Database.findUser(userId),
    Http.checkStock(items)
  }?

  log("Order for {user.name}: {items.count} items")
  if not stockOk { fail(OutOfStock { itemId: items[0].id }) }

  order = atomic {
    deductBalance(userId, total(items))
    reserveStock(items)
    createOrder(userId, items)
  }?

  Database.save(order)?
  order
}

-- 路由（声明式）
async fun main() needs Nothing {
  app = HttpServer.new()
  app.post("/orders", (req) ->
    req.json<OrderRequest>()?
      |>> (r) -> placeOrder(r.userId, r.items)
      |>> map(Response.ok)
      |>> catch((e) -> when e is
        UserNotFound { id }          -> Response.notFound("User {id}")
        InsufficientFunds { needed } -> Response.badRequest("需要 {needed}")
        OutOfStock { itemId }        -> Response.badRequest("{itemId} 缺货")
      )
  )
  wait app.listen(8080) given {
    Database <- PostgresDatabase.live
    Http     <- HttpClient.live
    Logger   <- ConsoleLogger.live
  }
}
```

---

## 26. 实现路线图

### Phase 1 · Bootstrap (3-6个月)
- 词法分析器（Lexer）
- 语法分析器（Parser）
- 抽象语法树（AST）表示
- 简单的树遍历解释器

### Phase 2 · Type System (2-4个月)
- Hindley-Milner 类型推断
- 类型检查器
- 多态类型支持
- needs/given 语法糖展开

### Phase 3 · Perceus RC (3-5个月)
- 实现线性资源演算分析器
- 插入基本 dup/drop
- 实现 drop_reuse/reuse 指令对
- 逃逸分析

### Phase 4 · FBIP (2-3个月)
- 重用分析——配对 drop_reuse 与 reuse 位置
- 特化代码生成
- 性能优化

### Phase 5 · LLVM IR (3-4个月)
- 带 Perceus 标注的 LLVM IR 生成器
- 首个原生二进制
- O2/O3 优化

### Phase 6 · Concurrency (2-3个月)
- Fiber 运行时
- wait together/race/timeout
- Channel 实现
- atomic / STM 支持

### Phase 7 · Stdlib + DX (3-6个月)
- 标准库核心模块
- 包管理器
- LSP 服务器（编辑器支持）
- REPL 环境

---

## 总结

X语言是一门现代编程语言，具有以下特色：

### 语言特色
- **自然语言语法**：代码读起来像英文句子，降低学习曲线
- **数学直觉**：支持数学风格的函数定义和集合表示法
- **四种范式**：函数式、面向对象、过程式、声明式自由混用
- **Perceus内存管理**：编译期精确RC，零开销重用分析，FBIP原地更新
- **Effect系统**：显式的依赖声明和注入，提高代码的可测试性和可维护性
- **异步编程**：wait together、wait race、timeout等自然语法
- **无null·无异常**：Option代替null，Result代替异常，编译器强制处理
- **LLVM后端**：x86-64 · ARM64 · WASM，原生二进制

### 适用场景
- **系统编程**：LLVM原生编译，高性能
- **Web开发**：异步、并发支持，清晰的架构
- **数据处理**：集合推导、函数式编程，代码简洁
- **教育编程**：自然语法，降低学习难度

X语言旨在成为一门通用编程语言，既适合初学者学习，也适合专业开发人员构建复杂系统。
