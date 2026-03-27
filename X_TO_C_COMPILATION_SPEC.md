# X 语言到 C23 编译规范

本文档定义了将 X 语言代码编译为 C23 代码的规范，确保生成的 C 代码高效、可读且符合 C23 标准。

## 目录

1. [基本类型映射](#1-基本类型映射)
2. [变量与作用域](#2-变量与作用域)
3. [函数](#3-函数)
4. [类与对象](#4-类与对象)
5. [继承与多态](#5-继承与多态)
6. [接口与 Trait](#6-接口与-trait)
7. [泛型](#7-泛型)
8. [模式匹配](#8-模式匹配)
9. [错误处理](#9-错误处理)
10. [内存管理](#10-内存管理)
11. [模块与导入](#11-模块与导入)
12. [并发与异步](#12-并发与异步)
13. [Unsafe 与 FFI](#13-unsafe-与-ffi)

---

## 1. 基本类型映射

### 1.1 整数类型

| X 类型 | C23 类型 |
|--------|----------|
| `integer` | `int32_t` |
| `signed 8-bit integer` | `int8_t` |
| `signed 16-bit integer` | `int16_t` |
| `signed 32-bit integer` | `int32_t` |
| `signed 64-bit integer` | `int64_t` |
| `signed 128-bit integer` | `_BitInt(128)` |
| `signed N-bit integer` | `_BitInt(N)` |
| `unsigned 8-bit integer` | `uint8_t` |
| `unsigned 16-bit integer` | `uint16_t` |
| `unsigned 32-bit integer` | `uint32_t` |
| `unsigned 64-bit integer` | `uint64_t` |
| `unsigned 128-bit integer` | `unsigned _BitInt(128)` |
| `unsigned N-bit integer` | `unsigned _BitInt(N)` |

### 1.2 浮点类型

| X 类型 | C23 类型 |
|--------|----------|
| `float` | `double` |
| `16-bit float` | `_Float16` |
| `32-bit float` | `float` |
| `64-bit float` | `double` |
| `128-bit float` | `_Float128` |
| `long float` | `long double` |
| `32-bit decimal` | `_Decimal32` |
| `64-bit decimal` | `_Decimal64` |
| `128-bit decimal` | `_Decimal128` |

### 1.3 其他基本类型

| X 类型 | C23 类型 |
|--------|----------|
| `boolean` | `bool` |
| `character` | `char32_t` |
| `utf-8 character` | `char8_t` |
| `utf-16 character` | `char16_t` |
| `utf-32 character` | `char32_t` |
| `string` | `const char*`（UTF-8 编码） |
| `utf-8 string` | `const char8_t*` |
| `Unit` | `void` |

### 1.4 复数与虚数类型

| X 类型 | C23 类型 |
|--------|----------|
| `complex float` | `double _Complex` |
| `complex 32-bit float` | `float _Complex` |
| `complex 64-bit float` | `double _Complex` |
| `complex 128-bit float` | `_Float128 _Complex` |
| `imaginary float` | `double _Imaginary` |
| `imaginary 32-bit float` | `float _Imaginary` |

### 1.5 复合类型

| X 类型 | C23 类型 |
|--------|----------|
| `(T1, T2, ...)` | 匿名结构体 |
| `List<T>` | 运行时结构（见第 10 章） |
| `{K: V}` | 运行时结构（见第 10 章） |
| `Option<T>` | 联合体（见第 9 章） |
| `Result<T, E>` | 联合体（见第 9 章） |

---

## 2. 变量与作用域

### 2.1 不可变变量

X 语言的 `let` 绑定默认不可变，映射为 C23 的 `const`：

```x
// X 语言
let x: integer = 42
let name: string = "hello"
```

```c
// C23
const int32_t x = 42;
const char* const name = "hello";
```

### 2.2 可变变量

X 语言的 `let mutable` 映射为普通变量：

```x
// X 语言
let mutable counter: integer = 0
counter = counter + 1
```

```c
// C23
int32_t counter = 0;
counter = counter + 1;
```

### 2.3 类型推断

X 语言支持类型推断，编译器需在编译期确定类型：

```x
// X 语言
let x = 42           // 推断为 integer (int32_t)
let y = 3.14         // 推断为 float (double)
let z = true         // 推断为 boolean (bool)
```

```c
// C23 - 类型已由编译器确定
const int32_t x = 42;
const double y = 3.14;
const bool z = true;
```

### 2.4 块作用域

X 语言的块作用域直接映射为 C 的块作用域：

```x
// X 语言
function example() -> Unit {
    let a = 1
    {
        let b = 2
        // a 和 b 都可见
    }
    // 仅 a 可见
}
```

```c
// C23
void example(void) {
    const int32_t a = 1;
    {
        const int32_t b = 2;
        // a 和 b 都可见
    }
    // 仅 a 可见
}
```

---

## 3. 函数

### 3.1 顶层函数

X 语言的顶层函数映射为 C 的全局函数：

```x
// X 语言
function add(a: integer, b: integer) -> integer {
    a + b
}

function greet(name: string) -> Unit {
    print("Hello, {name}!")
}
```

```c
// C23
int32_t add(int32_t a, int32_t b) {
    return a + b;
}

void greet(const char* const name) {
    printf("Hello, %s!\n", name);
}
```

### 3.2 默认参数值

X 语言支持默认参数，C 不支持。通过生成重载函数实现：

```x
// X 语言
function greet(name: string, greeting: string = "Hello") -> Unit {
    print("{greeting}, {name}!")
}
```

```c
// C23 - 生成两个函数
void greet_full(const char* const name, const char* const greeting) {
    printf("%s, %s!\n", greeting, name);
}

void greet(const char* const name) {
    greet_full(name, "Hello");
}
```

### 3.3 命名参数

X 语言的命名参数通过编译器重新排序：

```x
// X 语言
function create_point(x: float, y: float, z: float = 0.0) -> Point
let p = create_point(y: 2.0, x: 1.0)
```

```c
// C23 - 编译器按正确顺序传递参数
Point p = create_point_full(1.0, 2.0, 0.0);
```

### 3.4 高阶函数与闭包

闭包需要捕获环境变量，映射为结构体 + 函数指针：

```x
// X 语言
function make_counter() -> (() -> integer) {
    let mutable count = 0
    function() => {
        count = count + 1
        count
    }
}

let counter = make_counter()
print(counter())  // 1
print(counter())  // 2
```

```c
// C23 - 闭包结构体
typedef struct {
    int32_t* count;  // 捕获的变量（堆分配）
} counter_closure_t;

static int32_t counter_call(counter_closure_t* self) {
    (*self->count) = (*self->count) + 1;
    return *self->count;
}

typedef int32_t (*counter_fn_t)(counter_closure_t*);

typedef struct {
    counter_fn_t fn;
    counter_closure_t env;
} counter_closure;

static counter_closure make_counter(void) {
    counter_closure closure;
    closure.env.count = malloc(sizeof(int32_t));
    *closure.env.count = 0;
    closure.fn = counter_call;
    return closure;
}

// 使用
counter_closure counter = make_counter();
printf("%d\n", counter.fn(&counter.env));  // 1
printf("%d\n", counter.fn(&counter.env));  // 2
```

---

## 4. 类与对象

### 4.1 简单类

类映射为结构体 + 相关函数：

```x
// X 语言
class Point {
    let x: float
    let y: float

    public function new(x: float, y: float) -> Point {
        Point { x: x, y: y }
    }

    public function distance(self, other: Point) -> float {
        let dx = self.x - other.x
        let dy = self.y - other.y
        (dx * dx + dy * dy) ^ 0.5
    }

    public function translate(self, dx: float, dy: float) -> Point {
        Point.new(self.x + dx, self.y + dy)
    }
}
```

```c
// C23
// 结构体定义
typedef struct {
    double x;
    double y;
} Point;

// 构造函数
Point Point_new(double x, double y) {
    Point self;
    self.x = x;
    self.y = y;
    return self;
}

// 方法：self 作为第一个参数
double Point_distance(const Point* const self, const Point* const other) {
    double dx = self->x - other->x;
    double dy = self->y - other->y;
    return sqrt(dx * dx + dy * dy);
}

// 返回新实例的方法
Point Point_translate(const Point* const self, double dx, double dy) {
    return Point_new(self->x + dx, self->y + dy);
}

// 使用
Point p1 = Point_new(0.0, 0.0);
Point p2 = Point_new(3.0, 4.0);
double d = Point_distance(&p1, &p2);
Point p3 = Point_translate(&p1, 1.0, 2.0);
```

### 4.2 可变方法

修改 `self` 的方法使用指针：

```x
// X 语言
class Counter {
    let mutable value: integer

    public function new() -> Counter {
        Counter { value: 0 }
    }

    public function increment(self) -> Unit {
        self.value = self.value + 1
    }

    public function get(self) -> integer {
        self.value
    }
}
```

```c
// C23
typedef struct {
    int32_t value;
} Counter;

Counter Counter_new(void) {
    Counter self;
    self.value = 0;
    return self;
}

void Counter_increment(Counter* const self) {
    self->value = self->value + 1;
}

int32_t Counter_get(const Counter* const self) {
    return self->value;
}

// 使用
Counter c = Counter_new();
Counter_increment(&c);
int32_t v = Counter_get(&c);
```

### 4.3 访问控制

X 语言的访问控制通过命名约定和头文件控制：

```x
// X 语言
class BankAccount {
    private let balance: float

    public function new(initial: float) -> BankAccount {
        BankAccount { balance: initial }
    }

    public function deposit(self, amount: float) -> Unit {
        self.balance = self.balance + amount
    }

    public function get_balance(self) -> float {
        self.balance
    }
}
```

```c
// C23 - 头文件 (bank_account.h)
typedef struct BankAccount BankAccount;  // 不透明类型

BankAccount* BankAccount_new(double initial);
void BankAccount_delete(BankAccount* self);
void BankAccount_deposit(BankAccount* self, double amount);
double BankAccount_get_balance(const BankAccount* self);

// C23 - 实现文件 (bank_account.c)
struct BankAccount {
    double balance;  // 私有字段，外部不可直接访问
};

BankAccount* BankAccount_new(double initial) {
    BankAccount* self = malloc(sizeof(BankAccount));
    self->balance = initial;
    return self;
}

void BankAccount_delete(BankAccount* self) {
    free(self);
}

void BankAccount_deposit(BankAccount* self, double amount) {
    self->balance = self->balance + amount;
}

double BankAccount_get_balance(const BankAccount* self) {
    return self->balance;
}
```

### 4.4 静态成员

类的静态成员映射为全局变量和函数：

```x
// X 语言
class Math {
    public static let PI: float = 3.14159265359

    public static function square(x: float) -> float {
        x * x
    }
}

let area = Math.PI * Math.square(radius)
```

```c
// C23
// 静态常量
static const double Math_PI = 3.14159265359;

// 静态方法
static double Math_square(double x) {
    return x * x;
}

// 使用
double area = Math_PI * Math_square(radius);
```

### 4.5 析构函数

X 语言的析构函数映射为 `delete` 函数：

```x
// X 语言
class File {
    private let handle: *Void

    public function new(path: string) -> Result<File, IoError> {
        // 打开文件
    }

    public function close(self) -> Unit {
        // 关闭文件
    }
}
```

```c
// C23
typedef struct {
    void* handle;
} File;

File* File_new(const char* path) {
    File* self = malloc(sizeof(File));
    // 打开文件...
    return self;
}

void File_close(File* self) {
    // 关闭文件...
}

void File_delete(File* self) {
    if (self != nullptr) {
        File_close(self);
        free(self);
    }
}
```

---

## 5. 继承与多态

### 5.1 单继承

X 语言的单继承通过结构体嵌套实现：

```x
// X 语言
class Animal {
    let name: string

    public function new(name: string) -> Animal {
        Animal { name: name }
    }

    public function speak(self) -> string {
        "..."
    }
}

class Dog inherits Animal {
    let breed: string

    public function new(name: string, breed: string) -> Dog {
        Dog { name: name, breed: breed }
    }

    public override function speak(self) -> string {
        "Woof!"
    }
}
```

```c
// C23
// 基类
typedef struct {
    const char* name;
} Animal;

Animal Animal_new(const char* name) {
    Animal self;
    self.name = name;
    return self;
}

const char* Animal_speak(const Animal* const self) {
    return "...";
}

// 派生类：嵌套基类
typedef struct {
    Animal base;  // 基类作为第一个成员
    const char* breed;
} Dog;

Dog Dog_new(const char* name, const char* breed) {
    Dog self;
    self.base = Animal_new(name);
    self.breed = breed;
    return self;
}

const char* Dog_speak(const Dog* const self) {
    return "Woof!";
}
```

### 5.2 向上转型

派生类指针可安全转换为基类指针：

```x
// X 语言
let dog = Dog.new("Buddy", "Golden Retriever")
let animal: Animal = dog  // 向上转型
```

```c
// C23
Dog dog = Dog_new("Buddy", "Golden Retriever");
Animal* animal = (Animal*)&dog;  // 安全转换
```

### 5.3 多态（虚函数）

需要多态时，使用虚函数表（vtable）：

```x
// X 语言
class Animal {
    let name: string

    public function new(name: string) -> Animal {
        Animal { name: name }
    }

    public virtual function speak(self) -> string {
        "..."
    }

    public function greet(self) -> string {
        "I am {self.name} and I say {self.speak()}"
    }
}

class Dog inherits Animal {
    public override function speak(self) -> string {
        "Woof!"
    }
}

class Cat inherits Animal {
    public override function speak(self) -> string {
        "Meow!"
    }
}

// 多态调用
let animals: [Animal] = [Dog.new("Buddy"), Cat.new("Whiskers")]
for animal in animals {
    print(animal.speak())  // 多态调用
}
```

```c
// C23
// 虚函数表
typedef struct Animal_vtable Animal_vtable;

typedef struct {
    const Animal_vtable* vtable;
    const char* name;
} Animal;

struct Animal_vtable {
    const char* (*speak)(const Animal* const);
};

// Animal 的虚函数实现
static const char* Animal_speak_impl(const Animal* const self) {
    return "...";
}

static const Animal_vtable Animal_vtable_instance = {
    .speak = Animal_speak_impl,
};

Animal Animal_new(const char* name) {
    Animal self;
    self.vtable = &Animal_vtable_instance;
    self.name = name;
    return self;
}

// 虚函数调用
const char* Animal_speak(const Animal* const self) {
    return self->vtable->speak(self);
}

// 非虚函数
const char* Animal_greet(const Animal* const self) {
    static char buffer[256];
    snprintf(buffer, sizeof(buffer), "I am %s and I say %s",
             self->name, Animal_speak(self));
    return buffer;
}

// Dog 派生类
typedef struct {
    Animal base;
} Dog;

static const char* Dog_speak_impl(const Animal* const self) {
    return "Woof!";
}

static const Animal_vtable Dog_vtable_instance = {
    .speak = Dog_speak_impl,
};

Dog Dog_new(const char* name) {
    Dog self;
    self.base.vtable = &Dog_vtable_instance;
    self.base.name = name;
    return self;
}

// Cat 派生类
typedef struct {
    Animal base;
} Cat;

static const char* Cat_speak_impl(const Animal* const self) {
    return "Meow!";
}

static const Animal_vtable Cat_vtable_instance = {
    .speak = Cat_speak_impl,
};

Cat Cat_new(const char* name) {
    Cat self;
    self.base.vtable = &Cat_vtable_instance;
    self.base.name = name;
    return self;
}

// 多态使用
Animal animals[2];
animals[0] = Dog_new("Buddy").base;
animals[1] = Cat_new("Whiskers").base;

for (size_t i = 0; i < 2; i++) {
    printf("%s\n", Animal_speak(&animals[i]));
}
```

---

## 6. 接口与 Trait

### 6.1 接口定义

接口通过虚函数表实现：

```x
// X 语言
interface Printable {
    function to_string(self) -> string
}

interface Comparable {
    function compare(self, other: Self) -> integer
}

class Point implements Printable, Comparable {
    let x: float
    let y: float

    public function to_string(self) -> string {
        "({self.x}, {self.y})"
    }

    public function compare(self, other: Point) -> integer {
        if self.x < other.x { -1 }
        else if self.x > other.x { 1 }
        else if self.y < other.y { -1 }
        else if self.y > other.y { 1 }
        else { 0 }
    }
}
```

```c
// C23
// Printable 接口的虚函数表
typedef struct {
    const char* (*to_string)(const void* self);
} Printable_vtable;

// Comparable 接口的虚函数表
typedef struct {
    int32_t (*compare)(const void* self, const void* other);
} Comparable_vtable;

// Point 类
typedef struct {
    double x;
    double y;
    const Printable_vtable* printable;
    const Comparable_vtable* comparable;
} Point;

// Printable 实现
static const char* Point_to_string(const void* self) {
    const Point* p = (const Point*)self;
    static char buffer[64];
    snprintf(buffer, sizeof(buffer), "(%g, %g)", p->x, p->y);
    return buffer;
}

static const Printable_vtable Point_Printable = {
    .to_string = Point_to_string,
};

// Comparable 实现
static int32_t Point_compare(const void* self, const void* other) {
    const Point* a = (const Point*)self;
    const Point* b = (const Point*)other;
    if (a->x < b->x) return -1;
    if (a->x > b->x) return 1;
    if (a->y < b->y) return -1;
    if (a->y > b->y) return 1;
    return 0;
}

static const Comparable_vtable Point_Comparable = {
    .compare = Point_compare,
};

// 构造函数
Point Point_new(double x, double y) {
    Point self;
    self.x = x;
    self.y = y;
    self.printable = &Point_Printable;
    self.comparable = &Point_Comparable;
    return self;
}

// 使用
Point p = Point_new(1.0, 2.0);
const char* s = p.printable->to_string(&p);
```

### 6.2 Trait 约束

泛型函数使用 Trait 约束时，通过函数参数传递 vtable：

```x
// X 语言
function print_all<T: Printable>(items: [T]) -> Unit {
    for item in items {
        print(item.to_string())
    }
}
```

```c
// C23
void print_all(
    const void* items,
    size_t count,
    size_t item_size,
    const Printable_vtable* vtable
) {
    const char* ptr = (const char*)items;
    for (size_t i = 0; i < count; i++) {
        const void* item = ptr + i * item_size;
        printf("%s\n", vtable->to_string(item));
    }
}

// 使用
Point points[3] = { Point_new(1, 2), Point_new(3, 4), Point_new(5, 6) };
print_all(points, 3, sizeof(Point), &Point_Printable);
```

---

## 7. 泛型

### 7.1 单态化

X 语言的泛型通过单态化（monomorphization）实现——为每个具体类型生成专门的代码：

```x
// X 语言
function identity<T>(x: T) -> T {
    x
}

let a = identity(42)        // T = integer
let b = identity(3.14)      // T = float
let c = identity("hello")   // T = string
```

```c
// C23 - 为每个类型生成专门函数
int32_t identity_int32_t(int32_t x) {
    return x;
}

double identity_double(double x) {
    return x;
}

const char* identity_string(const char* x) {
    return x;
}

// 使用
int32_t a = identity_int32_t(42);
double b = identity_double(3.14);
const char* c = identity_string("hello");
```

### 7.2 泛型类

泛型类同样通过单态化：

```x
// X 语言
class Pair<T, U> {
    let first: T
    let second: U

    public function new(first: T, second: U) -> Pair<T, U> {
        Pair { first: first, second: second }
    }
}

let p1 = Pair.new(1, "one")
let p2 = Pair.new(3.14, 42)
```

```c
// C23 - 生成具体类型
typedef struct {
    int32_t first;
    const char* second;
} Pair_int32_t_string;

Pair_int32_t_string Pair_int32_t_string_new(int32_t first, const char* second) {
    Pair_int32_t_string self;
    self.first = first;
    self.second = second;
    return self;
}

typedef struct {
    double first;
    int32_t second;
} Pair_double_int32_t;

Pair_double_int32_t Pair_double_int32_t_new(double first, int32_t second) {
    Pair_double_int32_t self;
    self.first = first;
    self.second = second;
    return self;
}

// 使用
Pair_int32_t_string p1 = Pair_int32_t_string_new(1, "one");
Pair_double_int32_t p2 = Pair_double_int32_t_new(3.14, 42);
```

### 7.3 泛型约束

有约束的泛型需要传递 vtable：

```x
// X 语言
function max<T: Comparable>(a: T, b: T) -> T {
    if a.compare(b) > 0 { a } else { b }
}
```

```c
// C23
// 使用宏简化泛型调用
#define max(type, a, b, vtable) \
    (vtable.compare(&(a), &(b)) > 0 ? (a) : (b))

// 或生成专门函数
int32_t max_int32_t(int32_t a, int32_t b) {
    return a > b ? a : b;
}
```

### 7.4 常量泛型

常量泛型参数映射为编译期常量：

```x
// X 语言
class FixedArray<T, const N: size> {
    private let data: [T; N]

    public function length(self) -> size {
        N
    }
}

let arr: FixedArray<integer, 10> = FixedArray.new()
```

```c
// C23
typedef struct {
    int32_t data[10];
} FixedArray_int32_t_10;

size_t FixedArray_int32_t_10_length(const FixedArray_int32_t_10* const self) {
    return 10;
}
```

---

## 8. 模式匹配

### 8.1 枚举匹配

枚举类型映射为 tagged union：

```x
// X 语言
type Option<T> = Some(T) | None

let x: Option<integer> = Some(42)

match x {
    Some(value) => print("Value: {value}")
    None => print("No value")
}
```

```c
// C23
typedef enum {
    Option_int32_t_Tag_None,
    Option_int32_t_Tag_Some,
} Option_int32_t_Tag;

typedef struct {
    Option_int32_t_Tag tag;
    union {
        int32_t some;
    } data;
} Option_int32_t;

// 使用
Option_int32_t x;
x.tag = Option_int32_t_Tag_Some;
x.data.some = 42;

// 模式匹配
switch (x.tag) {
    case Option_int32_t_Tag_Some:
        printf("Value: %d\n", x.data.some);
        break;
    case Option_int32_t_Tag_None:
        printf("No value\n");
        break;
}
```

### 8.2 带数据的变体

```x
// X 语言
type Shape =
    | Circle { radius: float }
    | Rectangle { width: float, height: float }
    | Point

function area(shape: Shape) -> float {
    match shape {
        Circle { radius } => 3.14159 * radius * radius
        Rectangle { width, height } => width * height
        Point => 0.0
    }
}
```

```c
// C23
typedef enum {
    Shape_Tag_Circle,
    Shape_Tag_Rectangle,
    Shape_Tag_Point,
} Shape_Tag;

typedef struct {
    double radius;
} Shape_Circle;

typedef struct {
    double width;
    double height;
} Shape_Rectangle;

typedef struct {
    Shape_Tag tag;
    union {
        Shape_Circle circle;
        Shape_Rectangle rectangle;
    } data;
} Shape;

double Shape_area(const Shape* const shape) {
    switch (shape->tag) {
        case Shape_Tag_Circle:
            return 3.14159 * shape->data.circle.radius * shape->data.circle.radius;
        case Shape_Tag_Rectangle:
            return shape->data.rectangle.width * shape->data.rectangle.height;
        case Shape_Tag_Point:
            return 0.0;
    }
}
```

### 8.3 守卫条件

```x
// X 语言
match x {
    Some(value) if value > 0 => print("Positive: {value}")
    Some(value) if value < 0 => print("Negative: {value}")
    Some(0) => print("Zero")
    None => print("No value")
}
```

```c
// C23
switch (x.tag) {
    case Option_int32_t_Tag_Some:
        if (x.data.some > 0) {
            printf("Positive: %d\n", x.data.some);
        } else if (x.data.some < 0) {
            printf("Negative: %d\n", x.data.some);
        } else {
            printf("Zero\n");
        }
        break;
    case Option_int32_t_Tag_None:
        printf("No value\n");
        break;
}
```

---

## 9. 错误处理

### 9.1 Option 类型

```x
// X 语言
function find<T>(list: [T], predicate: (T) -> boolean) -> Option<T> {
    for item in list {
        if predicate(item) {
            return Some(item)
        }
    }
    None
}

let result = find([1, 2, 3], function(x) => x > 2)
match result {
    Some(value) => print("Found: {value}")
    None => print("Not found")
}
```

```c
// C23 - 使用 tagged union
typedef struct {
    bool is_some;
    int32_t value;
} Option_int32_t;

Option_int32_t find_int32_t(
    const int32_t* list,
    size_t len,
    bool (*predicate)(int32_t)
) {
    for (size_t i = 0; i < len; i++) {
        if (predicate(list[i])) {
            return (Option_int32_t){ .is_some = true, .value = list[i] };
        }
    }
    return (Option_int32_t){ .is_some = false };
}

// 使用
Option_int32_t result = find_int32_t(list, 3, predicate);
if (result.is_some) {
    printf("Found: %d\n", result.value);
} else {
    printf("Not found\n");
}
```

### 9.2 Result 类型

```x
// X 语言
function divide(a: float, b: float) -> Result<float, string> {
    if b == 0.0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

let result = divide(10.0, 2.0)
match result {
    Ok(value) => print("Result: {value}")
    Err(error) => print("Error: {error}")
}
```

```c
// C23
typedef enum {
    Result_double_string_Tag_Ok,
    Result_double_string_Tag_Err,
} Result_double_string_Tag;

typedef struct {
    Result_double_string_Tag tag;
    union {
        double ok;
        const char* err;
    } data;
} Result_double_string;

Result_double_string divide(double a, double b) {
    if (b == 0.0) {
        return (Result_double_string){
            .tag = Result_double_string_Tag_Err,
            .data.err = "Division by zero"
        };
    }
    return (Result_double_string){
        .tag = Result_double_string_Tag_Ok,
        .data.ok = a / b
    };
}

// 使用
Result_double_string result = divide(10.0, 2.0);
if (result.tag == Result_double_string_Tag_Ok) {
    printf("Result: %g\n", result.data.ok);
} else {
    printf("Error: %s\n", result.data.err);
}
```

### 9.3 错误传播 (? 运算符)

```x
// X 语言
function read_config(path: string) -> Result<Config, IoError> {
    let content = read_file(path)?
    let config = parse_config(content)?
    Ok(config)
}
```

```c
// C23 - 使用 goto 或提前返回
Result_Config_IoError read_config(const char* path) {
    Result_String_IoError content_result = read_file(path);
    if (content_result.tag == Result_Tag_Err) {
        return (Result_Config_IoError){
            .tag = Result_Tag_Err,
            .data.err = content_result.data.err
        };
    }

    Result_Config_ParseError config_result = parse_config(content_result.data.ok);
    if (config_result.tag == Result_Tag_Err) {
        return (Result_Config_IoError){
            .tag = Result_Tag_Err,
            .data.err = IoError_from_parse(config_result.data.err)
        };
    }

    return (Result_Config_IoError){
        .tag = Result_Tag_Ok,
        .data.ok = config_result.data.ok
    };
}
```

---

## 10. 内存管理

### 10.1 栈分配

小对象默认在栈上分配：

```x
// X 语言
let p = Point.new(1.0, 2.0)
```

```c
// C23
Point p = Point_new(1.0, 2.0);  // 栈分配
```

### 10.2 堆分配

大对象或需要跨作用域的对象使用堆分配：

```x
// X 语言
let p = new Point(1.0, 2.0)  // 显式堆分配
```

```c
// C23
Point* p = malloc(sizeof(Point));
*p = Point_new(1.0, 2.0);
// 使用完毕后释放
free(p);
```

### 10.3 Perceus 引用计数

X 语言使用 Perceus 自动内存管理，编译为 C 时可生成引用计数代码：

```x
// X 语言
let s = "hello"
let t = s  // 引用计数 +1
// 离开作用域时自动释放
```

```c
// C23 - 生成引用计数结构
typedef struct {
    char* data;
    size_t ref_count;
} RC_String;

RC_String* RC_String_new(const char* data) {
    RC_String* self = malloc(sizeof(RC_String));
    self->data = strdup(data);
    self->ref_count = 1;
    return self;
}

RC_String* RC_String_retain(RC_String* self) {
    if (self != nullptr) {
        self->ref_count++;
    }
    return self;
}

void RC_String_release(RC_String* self) {
    if (self != nullptr && --self->ref_count == 0) {
        free(self->data);
        free(self);
    }
}

// 使用
RC_String* s = RC_String_new("hello");
RC_String* t = RC_String_retain(s);
RC_String_release(t);
RC_String_release(s);
```

### 10.4 列表类型

```x
// X 语言
let list: [integer] = [1, 2, 3, 4, 5]
list.append(6)
```

```c
// C23 - 动态数组
typedef struct {
    int32_t* data;
    size_t length;
    size_t capacity;
} List_int32_t;

List_int32_t List_int32_t_new(void) {
    return (List_int32_t){ .data = nullptr, .length = 0, .capacity = 0 };
}

void List_int32_t_append(List_int32_t* self, int32_t value) {
    if (self->length >= self->capacity) {
        size_t new_capacity = self->capacity == 0 ? 4 : self->capacity * 2;
        self->data = realloc(self->data, new_capacity * sizeof(int32_t));
        self->capacity = new_capacity;
    }
    self->data[self->length++] = value;
}

void List_int32_t_delete(List_int32_t* self) {
    free(self->data);
    self->data = nullptr;
    self->length = 0;
    self->capacity = 0;
}
```

---

## 11. 模块与导入

### 11.1 模块映射

X 语言的模块映射为 C 的头文件和源文件：

```x
// X 语言: math/utils.x
module math.utils

public function add(a: integer, b: integer) -> integer {
    a + b
}

private function helper() -> Unit {
    // 内部实现
}
```

```c
// C23: math_utils.h
#ifndef MATH_UTILS_H
#define MATH_UTILS_H

#include <stdint.h>

int32_t math_utils_add(int32_t a, int32_t b);

#endif

// C23: math_utils.c
#include "math_utils.h"

int32_t math_utils_add(int32_t a, int32_t b) {
    return a + b;
}

// helper 函数不导出（static）
static void helper(void) {
    // 内部实现
}
```

### 11.2 导入

```x
// X 语言
import math.utils

let result = utils.add(1, 2)
```

```c
// C23
#include "math_utils.h"

int32_t result = math_utils_add(1, 2);
```

### 11.3 选择性导入

```x
// X 语言
from math.utils import add, subtract

let x = add(1, 2)
```

```c
// C23
#include "math_utils.h"

// 可选择性创建别名
static inline int32_t add(int32_t a, int32_t b) {
    return math_utils_add(a, b);
}

int32_t x = add(1, 2);
```

---

## 12. 并发与异步

### 12.1 原子类型

```x
// X 语言
let counter: atomic integer = 0
atomic fetch_add(counter, 1)
```

```c
// C23
#include <stdatomic.h>

atomic_int counter = ATOMIC_VAR_INIT(0);
atomic_fetch_add(&counter, 1);
```

### 12.2 异步函数

X 语言的异步函数可编译为状态机或回调：

```x
// X 语言
async function fetch_data(url: string) -> string {
    let response = await http_get(url)
    response.body
}
```

```c
// C23 - 回调方式
typedef void (*fetch_data_callback_t)(const char* result, void* user_data);

void fetch_data(const char* url, fetch_data_callback_t callback, void* user_data) {
    http_get(url, [](const HttpResponse* response, void* data) {
        fetch_data_callback_t cb = (fetch_data_callback_t)data;
        cb(response->body, user_data);
    }, callback);
}
```

---

## 13. Unsafe 与 FFI

### 13.1 外部函数声明

```x
// X 语言
foreign function printf(format: *const character, ...) -> signed 32-bit integer
foreign function malloc(size: size) -> *Void
foreign function free(ptr: *Void) -> Unit
```

```c
// C23
#include <stdio.h>
#include <stdlib.h>

// 直接使用标准库函数
```

### 13.2 Unsafe 块

```x
// X 语言
function use_c_functions() -> Unit {
    unsafe {
        let ptr: *Void = malloc(1024)
        printf("Allocated %zu bytes\n", 1024)
        free(ptr)
    }
}
```

```c
// C23
void use_c_functions(void) {
    void* ptr = malloc(1024);
    printf("Allocated %zu bytes\n", 1024);
    free(ptr);
}
```

### 13.3 指针操作

```x
// X 语言
function pointer_example() -> Unit {
    unsafe {
        let arr: [integer; 3] = [1, 2, 3]
        let ptr: *integer = &arr[0]
        let value = *ptr
        *ptr = 100
    }
}
```

```c
// C23
void pointer_example(void) {
    int32_t arr[3] = {1, 2, 3};
    int32_t* ptr = &arr[0];
    int32_t value = *ptr;
    *ptr = 100;
}
```

### 13.4 volatile 和 restrict

```x
// X 语言
let hw_reg: volatile *unsigned 32-bit integer = 0x1000

function copy(dst: *character restrict, src: *const character restrict, n: size) -> Unit
```

```c
// C23
volatile uint32_t* const hw_reg = (volatile uint32_t*)0x1000;

void copy(char* restrict dst, const char* restrict src, size_t n) {
    memcpy(dst, src, n);
}
```

---

## 附录：命名约定

| X 语言 | C23 | 说明 |
|--------|-----|------|
| `Point` | `Point` | 类型名保持不变 |
| `Point.new` | `Point_new` | 构造函数 |
| `Point.distance` | `Point_distance` | 方法 |
| `Math.PI` | `Math_PI` | 静态常量 |
| `math.utils` | `math_utils` | 模块名 |
| `Some(T)` | `Option_T_Tag_Some` | 枚举变体 |
| `Result<T, E>` | `Result_T_E` | 泛型类型 |

---

## 附录：代码生成示例

### 完整示例：复数类

```x
// X 语言
class Complex {
    let real: float
    let imag: float

    public function new(real: float, imag: float) -> Complex {
        Complex { real: real, imag: imag }
    }

    public function add(self, other: Complex) -> Complex {
        Complex.new(self.real + other.real, self.imag + other.imag)
    }

    public function multiply(self, other: Complex) -> Complex {
        Complex.new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real
        )
    }

    public function magnitude(self) -> float {
        (self.real * self.real + self.imag * self.imag) ^ 0.5
    }

    public function to_string(self) -> string {
        "{self.real} + {self.imag}i"
    }
}

let c1 = Complex.new(1.0, 2.0)
let c2 = Complex.new(3.0, 4.0)
let sum = c1.add(c2)
print(sum.to_string())
```

```c
// C23 - 生成的代码
#include <stdio.h>
#include <math.h>

typedef struct {
    double real;
    double imag;
} Complex;

Complex Complex_new(double real, double imag) {
    Complex self;
    self.real = real;
    self.imag = imag;
    return self;
}

Complex Complex_add(const Complex* const self, const Complex* const other) {
    return Complex_new(self->real + other->real, self->imag + other->imag);
}

Complex Complex_multiply(const Complex* const self, const Complex* const other) {
    return Complex_new(
        self->real * other->real - self->imag * other->imag,
        self->real * other->imag + self->imag * other->real
    );
}

double Complex_magnitude(const Complex* const self) {
    return sqrt(self->real * self->real + self->imag * self->imag);
}

const char* Complex_to_string(const Complex* const self) {
    static char buffer[64];
    snprintf(buffer, sizeof(buffer), "%g + %gi", self->real, self->imag);
    return buffer;
}

int main(void) {
    Complex c1 = Complex_new(1.0, 2.0);
    Complex c2 = Complex_new(3.0, 4.0);
    Complex sum = Complex_add(&c1, &c2);
    printf("%s\n", Complex_to_string(&sum));
    return 0;
}
```

---

**本文档定义了 X 语言到 C23 的完整编译映射规范，确保生成的 C 代码高效、可读且符合标准。**
