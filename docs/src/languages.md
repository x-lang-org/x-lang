### 说明

本页按照 **[TIOBE Index 2026‑02 排名](https://www.tiobe.com/tiobe-index/)** 的 **前 50 名编程语言** 进行整理。

对每门语言，统一给出：

- **1. 简介**：定位、典型应用场景与主要特性
- **2. 关键字列表及示例**：几类代表性关键字 / 语句，说明其作用，并给出简单示例

此外，大多数语言还共享一套 **常见运算符**（算术、比较、逻辑、赋值等），下面先给出一个跨语言的运算符速查表，后续在各小节的代码中也会自然出现这些运算符的用法。

#### 通用运算符速查（跨语言）

- **算术运算符**
  - `+`：加法（如 `a + b`）
  - `-`：减法或一元取负（如 `a - b`、`-a`）
  - `*`：乘法（如 `a * b`）
  - `/`：除法（如 `a / b`）
  - `%`：取余（模运算，如 `a % b`）
- **比较运算符**
  - `==` / `=`：相等（部分语言使用 `==`，部分 SQL 方言使用 `=`）
  - `!=` / `<>`：不等（C 系一族是 `!=`，SQL 常见 `<>`）
  - `<`、`>`、`<=`、`>=`：大小比较
- **逻辑运算符**
  - C/Java/JavaScript 等：`&&`（与）、`||`（或）、`!`（非）
  - Python / SQL / X 语言等更偏向可读性的语言：`and`、`or`、`not`
- **自增 / 自减与复合赋值（部分语言支持）**
  - `++a`、`a++`、`--a`、`a--`：自增 / 自减（C、C++、Java、C#、JavaScript 等）
  - `+=`、`-=`、`*=`、`/=`、`%=`：在原值基础上进行运算并赋值回去
- **位运算（主要出现在 C 家族、Java、C#、Rust 等）**
  - `&`（按位与）、`|`（按位或）、`^`（按位异或）、`~`（按位取反）
  - `<<`、`>>`：左移 / 右移

典型对比示例：

- **Python（关键字风格逻辑运算，适合阅读）**

  ```python
  x = 10 + 2 * 3      # 算术
  if x > 10 and x < 20:
      print("in range")
  ```

- **C（符号风格逻辑运算，接近硬件）**

  ```c
  int x = 10 + 2 * 3;
  if (x > 10 && x < 20) {
      printf("in range\n");
  }
  ```

- **JavaScript（符号逻辑 + 复合赋值）**

  ```javascript
  let x = 10;
  x += 2;           // x = x + 2
  if (x >= 10 && x <= 20) {
    console.log("in range");
  }
  ```

代码示例偏向“入门级直观理解”，不追求覆盖全部语法细节。

---

### 1. Python

#### 1. 简介

Python 是目前最流行的通用高级语言之一，语法简洁、生态庞大，广泛用于数据科学、机器学习、Web 开发、自动化脚本、运维等。

#### 2. 关键字列表及示例

- **`def` / `return`**：定义函数和返回值。

```python
def add(a, b):
    return a + b

print(add(1, 2))
```

- **`if` / `elif` / `else`**：条件分支。

```python
x = 10

if x > 10:
    print("big")
elif x == 10:
    print("equal")
else:
    print("small")
```

- **`for` / `while` / `break` / `continue`**：循环控制。

```python
for i in range(3):
    if i == 1:
        continue
    print(i)
```

- **`try` / `except` / `finally` / `raise`**：异常处理。

```python
def safe_div(a, b):
    try:
        return a / b
    except ZeroDivisionError:
        return None
    finally:
        print("done")

if safe_div(1, 0) is None:
    raise ValueError("division failed")
```

- **`import` / `from` / `as`**：模块导入。

```python
import math
from collections import Counter as C

print(math.sqrt(4))
print(C("abca"))
```

---

### 2. C

#### 1. 简介

C 是经典的过程式系统编程语言，广泛用于操作系统、嵌入式、编译器和高性能库等底层场景。

#### 2. 关键字列表及示例

- **`int` / `char` / `double` 等**：基本类型。

```c
int x = 10;
double y = 3.14;
char c = 'A';
```

- **`if` / `else` / `switch`**：条件控制。

```c
int n = 2;

switch (n) {
case 1:
    printf("one\n");
    break;
case 2:
    printf("two\n");
    break;
default:
    printf("other\n");
}
```

- **`for` / `while` / `do`**：循环。

```c
for (int i = 0; i < 3; i++) {
    printf("%d\n", i);
}
```

- **`struct` / `typedef`**：结构体与类型别名。

```c
typedef struct {
    int x;
    int y;
} Point;

Point p = { .x = 1, .y = 2 };
```

---

### 3. C++

#### 1. 简介

C++ 在 C 的基础上加入面向对象、泛型和现代抽象机制，是系统软件、游戏引擎、高性能库等领域的主力语言。

#### 2. 关键字列表及示例

- **`class` / `public` / `private`**：类与访问控制。

```cpp
class Person {
public:
    explicit Person(std::string name) : name_(std::move(name)) {}
    void greet() const { std::cout << "Hello, " << name_ << "\n"; }
private:
    std::string name_;
};
```

- **`template` / `typename`**：模板与泛型。

```cpp
template <typename T>
T add(T a, T b) {
    return a + b;
}
```

- **`auto` / `constexpr`**：类型推断与编译期常量。

```cpp
constexpr int square(int x) { return x * x; }
auto v = square(4);
```

---

### 4. Java

#### 1. 简介

Java 是面向对象的通用语言，运行在 JVM 上，适合大型企业应用、Android 开发和后端服务。

#### 2. 关键字列表及示例

- **`class` / `interface`**：类与接口。

```java
public interface Greet {
    void greet();
}

public class Person implements Greet {
    private final String name;
    public Person(String name) { this.name = name; }
    @Override
    public void greet() {
        System.out.println("Hello, " + name);
    }
}
```

- **`public` / `private` / `static` / `final`**：访问控制与修饰符。

```java
public final class MathUtil {
    private MathUtil() {}
    public static int add(int a, int b) {
        return a + b;
    }
}
```

- **`try` / `catch` / `finally` / `throw` / `throws`**：异常处理。

```java
public int parse(String s) throws NumberFormatException {
    try {
        return Integer.parseInt(s);
    } catch (NumberFormatException e) {
        throw e;
    } finally {
        System.out.println("done");
    }
}
```

---

### 5. C#

#### 1. 简介

C# 是 .NET 平台上的现代面向对象语言，支持泛型、LINQ、async/await 等特性，广泛用于桌面、Web、云服务和游戏（Unity）。

#### 2. 关键字列表及示例

- **`class` / `interface` / `struct`**：引用类型与值类型。

```csharp
public interface IGreet {
    void Greet();
}

public class Person : IGreet {
    public string Name { get; }
    public Person(string name) => Name = name;
    public void Greet() => Console.WriteLine($"Hello, {Name}");
}
```

- **`var` / `async` / `await`**：类型推断与异步。

```csharp
public async Task<string> FetchAsync(HttpClient client) {
    var res = await client.GetStringAsync("https://example.com");
    return res;
}
```

- **`using`**：资源自动释放（`IDisposable`）。

```csharp
using var file = File.OpenText("data.txt");
Console.WriteLine(file.ReadLine());
```

---

### 6. JavaScript

#### 1. 简介

JavaScript 最初是浏览器脚本语言，如今已发展为可在浏览器、Node.js、Deno 等环境中运行的全栈语言。

#### 2. 关键字列表及示例

- **`let` / `const` / `var`**：变量和常量声明。

```javascript
let x = 1;      // 块级作用域，可重新赋值
const y = 2;    // 块级作用域，不可重新赋值
var z = 3;      // 函数作用域
```

- **`function` / `=>`**：函数声明与箭头函数。

```javascript
function add(a, b) {
  return a + b;
}

const mul = (a, b) => a * b;
```

- **`async` / `await`**：异步编程。

```javascript
async function fetchData() {
  const res = await fetch("/api/data");
  const json = await res.json();
  console.log(json);
}
```

---

### 7. Visual Basic

#### 1. 简介

这里的 Visual Basic 主要指 Visual Basic .NET，是 .NET 平台上的基于 Basic 语法的语言，常用于 Windows 桌面和业务系统开发。

#### 2. 关键字列表及示例

- **`Module` / `Sub` / `Function`**：模块与过程。

```vbnet
Module Program
  Sub Main()
    Console.WriteLine(Add(1, 2))
  End Sub

  Function Add(a As Integer, b As Integer) As Integer
    Return a + b
  End Function
End Module
```

- **`If` / `Then` / `Else` / `End If`**：条件控制。

```vbnet
If x > 10 Then
  Console.WriteLine("big")
Else
  Console.WriteLine("small")
End If
```

---

### 8. R

#### 1. 简介

R 是面向统计分析和数据可视化的语言，拥有丰富的统计模型和绘图库，在学术研究和数据科学中广泛使用。

#### 2. 关键字列表及示例

- **`function`**：函数定义。

```r
add <- function(a, b) {
  a + b
}

add(1, 2)
```

- **`if` / `else` / `for` / `while`**：控制流。

```r
for (i in 1:3) {
  print(i)
}
```

---

### 9. SQL

#### 1. 简介

SQL（Structured Query Language）是关系型数据库的标准查询语言，用于定义表结构和操作数据。

#### 2. 常见语句及示例（类比关键字）

- **`SELECT` / `FROM` / `WHERE`**：查询。

```sql
SELECT id, name
FROM users
WHERE active = 1;
```

- **`INSERT` / `UPDATE` / `DELETE`**：写入与修改。

```sql
INSERT INTO users (name) VALUES ('Alice');
```

- **`CREATE TABLE` / `ALTER TABLE` / `DROP TABLE`**：表结构。

```sql
CREATE TABLE users (
  id   INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
```

---

### 10. Delphi / Object Pascal

#### 1. 简介

Delphi 使用 Object Pascal 语言，侧重于快速构建 Windows 桌面和数据库应用，也支持跨平台开发。

#### 2. 关键字列表及示例

- **`program` / `begin` / `end`**：程序入口。

```pascal
program Hello;
begin
  Writeln('Hello, world');
end.
```

- **`type` / `class`**：类型与类。

```pascal
type
  TPerson = class
  private
    FName: string;
  public
    constructor Create(const AName: string);
    procedure Greet;
  end;
```

---

### 11. Perl

#### 1. 简介

Perl 以强大的文本处理和正则表达式能力著称，早期常用于 CGI 脚本、系统管理和日志分析。

#### 2. 关键字列表及示例

- **`my` / `sub`**：变量与子例程。

```perl
use strict;
use warnings;

sub add {
  my ($a, $b) = @_;
  return $a + $b;
}

print add(1, 2);
```

- **`if` / `elsif` / `else` / `for` / `foreach`**：控制流。

```perl
for my $x (1..3) {
  print "$x\n";
}
```

---

### 12. Fortran

#### 1. 简介

Fortran 是最早的高级语言之一，仍在数值计算、工程仿真和科学计算中广泛使用。

#### 2. 关键字列表及示例

- **`program` / `end program`**：程序入口。

```fortran
program hello
  print *, "Hello, world"
end program hello
```

- **`do` / `if`**：循环与条件。

```fortran
do i = 1, 3
  print *, i
end do
```

---

### 13. PHP

#### 1. 简介

PHP 是主要用于服务端 Web 开发的脚本语言，可嵌入 HTML，生态中有 Laravel、Symfony 等框架。

#### 2. 关键字列表及示例

- **`function`**：定义函数。

```php
<?php
function add(int $a, int $b): int {
    return $a + $b;
}
echo add(1, 2);
```

- **`class` / `interface`**：类与接口。

```php
<?php
interface Greet {
    public function greet(): void;
}

class Person implements Greet {
    public function __construct(private string $name) {}
    public function greet(): void {
        echo "Hello, {$this->name}";
    }
}
```

---

### 14. Rust

#### 1. 简介

Rust 是强调内存安全与并发安全的系统编程语言，通过所有权和借用系统在无 GC 的情况下避免常见内存错误。

#### 2. 关键字列表及示例

- **`fn` / `let` / `mut`**：函数与绑定。

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

let mut x = 10;
x += 1;
```

- **`struct` / `enum` / `impl`**：数据类型与方法。

```rust
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn len2(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}
```

---

### 15. Scratch

#### 1. 简介

Scratch 是面向儿童和初学者的图形化编程语言，通过拖拽积木块构建程序，常用于编程启蒙教育。

#### 2. 常见积木（类比关键字）

- **控制类积木**：`重复执行`、`如果…那么`、`等待` 等。
- **事件类积木**：如“当绿旗被点击时”作为程序入口。

（Scratch 是图形化环境，这里不以文本代码展示示例。）

---

### 16. Go

#### 1. 简介

Go（Golang）是 Google 设计的静态类型语言，内置 goroutine 与 channel 并发模型，适合云原生服务和工具开发。

#### 2. 关键字列表及示例

- **`func`**：函数。

```go
func Add(a, b int) int {
    return a + b
}
```

- **`go` / `chan` / `select`**：并发与通道。

```go
func worker(ch chan int) {
    ch <- 42
}

func main() {
    ch := make(chan int)
    go worker(ch)
    v := <-ch
    println(v)
}
```

---

### 17. Ada

#### 1. 简介

Ada 是为高可靠性和安全关键系统（航空航天、军工等）设计的强类型语言，强调可读性和并发。

#### 2. 关键字列表及示例

- **`procedure` / `is` / `begin` / `end`**：过程定义。

```ada
procedure Hello is
begin
   Put_Line("Hello, world");
end Hello;
```

- **`type` / `record`**：自定义类型。

```ada
type Point is record
   X : Integer;
   Y : Integer;
end record;
```

---

### 18. MATLAB

#### 1. 简介

MATLAB 是面向矩阵运算和数值分析的商业语言与环境，在工程、控制、信号处理等领域广泛使用。

#### 2. 关键字列表及示例

- **`function`**：函数定义。

```matlab
function y = add(a, b)
  y = a + b;
end
```

- **`for` / `if`**：控制流。

```matlab
for i = 1:3
  disp(i);
end
```

---

### 19. Assembly language

#### 1. 简介

汇编语言是紧贴硬件指令集的低级语言，不同 CPU 架构有不同语法，常用于性能关键或直接硬件控制场景。

#### 2. 常见指令及示例（x86 伪代码）

- **`MOV` / `ADD` / `JMP`**：数据移动、算术和跳转。

```asm
MOV AX, 1
ADD AX, 2
JMP done
done:
```

---

### 20. Kotlin

#### 1. 简介

Kotlin 是 JetBrains 设计的现代静态类型语言，可编译到 JVM、Android、Native 和 JavaScript，是 Android 官方首选语言之一。

#### 2. 关键字列表及示例

- **`val` / `var`**：不可变 / 可变变量。

```kotlin
val name = "Alice"  // 只读
var age = 18        // 可变
age += 1
```

- **`fun`**：函数。

```kotlin
fun add(a: Int, b: Int): Int = a + b

fun main() {
    println(add(1, 2))
}
```

- **`data class` / `object`**：数据类和单例。

```kotlin
data class User(val id: Int, val name: String)

object Config {
    const val Version = "1.0"
}
```

---

### 21. Swift

#### 1. 简介

Swift 是 Apple 推出的现代静态类型语言，用于 iOS、macOS、watchOS、tvOS 等平台开发。

#### 2. 关键字列表及示例

- **`let` / `var`**：常量与变量。

```swift
let name = "Alice"
var age = 18
age += 1
```

- **`struct` / `class` / `enum`**：类型定义。

```swift
struct Point {
    var x: Int
    var y: Int
}
```

- **`if` / `guard` / `switch`**：控制流。

```swift
func greet(_ name: String?) {
    guard let name else { return }
    print("Hello, \(name)")
}
```

---

### 22. COBOL

#### 1. 简介

COBOL 是为商业数据处理设计的老牌语言，在金融、保险等大型机系统中仍有大量遗留代码。

#### 2. 关键字列表及示例

- **`IDENTIFICATION DIVISION` / `PROCEDURE DIVISION`**：程序结构。

```cobol
IDENTIFICATION DIVISION.
PROGRAM-ID. HELLO.
PROCEDURE DIVISION.
    DISPLAY "Hello, world".
    STOP RUN.
```

---

### 23. Classic Visual Basic

#### 1. 简介

Classic Visual Basic 通常指 VB6 及更早版本，主要用于早期 Windows 桌面应用开发。

#### 2. 关键字列表及示例

- **`Sub` / `Function`**：过程与函数。

```vb
Sub Hello()
  MsgBox "Hello"
End Sub
```

- **`If` / `Then` / `Else`**：条件控制。

```vb
If x > 10 Then
  MsgBox "big"
Else
  MsgBox "small"
End If
```

---

### 24. Prolog

#### 1. 简介

Prolog 是逻辑编程语言，基于事实和规则，通过“查询”让系统推理出答案，常用于人工智能和知识表示。

#### 2. 关键字与结构示例

- **事实与规则**：`:-` 表示蕴含。

```prolog
parent(alice, bob).
parent(bob, carol).

grandparent(X, Z) :- parent(X, Y), parent(Y, Z).
```

---

### 25. Ruby

#### 1. 简介

Ruby 是语法优雅、强调开发者愉悦度的动态语言，Ruby on Rails 框架在 Web 开发领域影响深远。

#### 2. 关键字列表及示例

- **`def` / `end`**：定义方法。

```ruby
def add(a, b)
  a + b
end

puts add(1, 2)
```

- **`class` / `module`**：类与模块。

```ruby
module Greeting
  def greet
    puts "Hello, #{@name}"
  end
end

class Person
  include Greeting
  def initialize(name)
    @name = name
  end
end
```

---

### 26. Dart

#### 1. 简介

Dart 是 Google 推出的语言，常与 Flutter 一起用于跨平台移动、Web 和桌面应用开发。

#### 2. 关键字列表及示例

- **`var` / `final` / `const`**：变量与常量。

```dart
var x = 1;        // 可变
final y = 2;      // 运行期常量
const z = 3;      // 编译期常量
```

- **`class` / `extends` / `implements`**：OOP。

```dart
class Person {
  final String name;
  Person(this.name);

  void greet() => print('Hello, $name');
}
```

---

### 27. Lua

#### 1. 简介

Lua 是轻量级脚本语言，常嵌入游戏引擎和应用程序，用作配置和扩展语言。

#### 2. 关键字列表及示例

- **`function` / `local`**：函数与局部变量。

```lua
local function add(a, b)
  return a + b
end

print(add(1, 2))
```

- **`if` / `elseif` / `else` / `end`**：条件。

```lua
local x = 10
if x > 10 then
  print("big")
elseif x == 10 then
  print("equal")
else
  print("small")
end
```

---

### 28. SAS

#### 1. 简介

SAS 是商业统计分析系统和语言，常用于数据仓库、商业智能和医疗统计。

#### 2. 关键字与示例

- **`DATA` / `SET` / `RUN`**：数据步。

```sas
DATA work.sample;
  SET work.source;
RUN;
```

- **`PROC`**：过程分析，如 `PROC MEANS`。

```sas
PROC MEANS DATA=work.sample;
RUN;
```

---

### 29. Julia

#### 1. 简介

Julia 是为数值和科学计算设计的高性能动态语言，兼具易用性和接近 C 的速度。

#### 2. 关键字列表及示例

- **`function` / `end`**：函数。

```julia
function add(a, b)
    a + b
end

println(add(1, 2))
```

- **`struct` / `mutable struct`**：结构体。

```julia
struct Point
    x::Int
    y::Int
end
```

---

### 30. Lisp

#### 1. 简介

Lisp 是历史悠久的函数式语言家族，特点是 S 表达式、宏系统和强大的元编程能力。

#### 2. 关键字列表及示例（以 Common Lisp 风格为例）

- **`defun` / `let`**：定义函数和局部绑定。

```lisp
(defun add (a b)
  (+ a b))

(let ((x 1) (y 2))
  (print (add x y)))
```

---

### 31. Objective-C

#### 1. 简介

Objective-C 在 C 的基础上加入 Smalltalk 风格消息机制，曾是 macOS / iOS 开发的主要语言。

#### 2. 关键字列表及示例

- **`@interface` / `@implementation`**：类声明与实现。

```objectivec
@interface Person : NSObject
@property (nonatomic, copy) NSString *name;
- (void)greet;
@end

@implementation Person
- (void)greet {
    NSLog(@"Hello, %@", self.name);
}
@end
```

- **`@autoreleasepool`**：自动释放池。

```objectivec
@autoreleasepool {
    Person *p = [Person new];
    p.name = @"Alice";
    [p greet];
}
```

---

### 32. TypeScript

#### 1. 简介

TypeScript 是 JavaScript 的超集，引入静态类型和语言扩展，最终编译为普通 JavaScript。

#### 2. 关键字列表及示例

- **`type` / `interface`**：类型别名与接口。

```typescript
type ID = number | string;

interface User {
  id: ID;
  name: string;
}
```

- **`enum` / `implements` / `extends`**：枚举与继承。

```typescript
enum Direction { Up, Down, Left, Right }

interface Greet { greet(): void; }

class Person implements Greet {
  constructor(private name: string) {}
  greet() {
    console.log(`Hello, ${this.name}`);
  }
}
```

---

### 33. PL/SQL

#### 1. 简介

PL/SQL 是 Oracle 数据库的过程化扩展 SQL，用于在数据库内部编写存储过程、函数和触发器。

#### 2. 关键字与示例

- **`DECLARE` / `BEGIN` / `END`**：块结构。

```sql
DECLARE
  v_sum NUMBER;
BEGIN
  v_sum := 1 + 2;
  DBMS_OUTPUT.PUT_LINE(v_sum);
END;
```

---

### 34. VBScript

#### 1. 简介

VBScript 是基于 Visual Basic 语法的脚本语言，曾常用于 Windows 脚本和早期 IE 浏览器脚本。

#### 2. 关键字列表及示例

- **`Dim` / `Sub` / `Function`**。

```vbscript
Dim x
x = 1

Sub Hello()
  MsgBox "Hello"
End Sub
```

---

### 35. Haskell

#### 1. 简介

Haskell 是纯函数式语言，具有惰性求值和强类型系统，适合抽象表达和形式化推理。

#### 2. 关键字列表及示例

- **`data` / `type`**：代数数据类型。

```haskell
data Direction = Up | Down | Left | Right

type Name = String
```

- **`let` / `where`**：局部绑定。

```haskell
area r = pi * r2
  where r2 = r * r
```

---

### 36. Erlang

#### 1. 简介

Erlang 是为电信和高并发系统设计的函数式语言，提供轻量级进程和消息传递模型。

#### 2. 关键字列表及示例

- **`receive` / `fun` / `case`**。

```erlang
loop() ->
    receive
        {From, Msg} ->
            From ! {ok, Msg},
            loop()
    end.
```

---

### 37. Ladder Logic

#### 1. 简介

梯形图（Ladder Logic）是一种用于 PLC（可编程逻辑控制器）的图形化编程语言，外观类似电气继电器电路。

#### 2. 常见结构（概念性说明）

- **触点（常开 / 常闭）**：表示输入条件。
- **线圈**：表示输出动作。

（梯形图主要以图形形式编辑，一般不使用文本关键字表示。）

---

### 38. (Visual) FoxPro

#### 1. 简介

Visual FoxPro 是微软推出的面向数据的编程语言和数据库系统，曾在桌面数据库应用中流行。

#### 2. 关键字与示例

- **`SELECT` / `FROM` / `WHERE`**：内建数据库查询语句。

```foxpro
SELECT name FROM users WHERE active = .T.
```

---

### 39. Scala

#### 1. 简介

Scala 结合面向对象与函数式编程，运行在 JVM 上，常用于数据处理（如 Spark）、分布式系统和后端服务。

#### 2. 关键字列表及示例

- **`object` / `class` / `trait`**。

```scala
trait Greet {
  def greet(): Unit
}

class Person(name: String) extends Greet {
  def greet(): Unit = println(s"Hello, $name")
}

object Main {
  def main(args: Array[String]): Unit = {
    new Person("Alice").greet()
  }
}
```

- **`val` / `var`**：不可变 / 可变变量。

```scala
val x = 1
var y = 2
y += 1
```

---

### 40. LabVIEW

#### 1. 简介

LabVIEW 是图形化编程环境和语言，主要用于测试测量、数据采集和工业控制领域。

#### 2. 常见结构

- **虚拟仪器（VI）**：图形化函数单元。
- **数据流连线**：表示数据依赖关系。

（LabVIEW 程序以图形方式构建，不以文本关键字为主。）

---

### 41. PowerShell

#### 1. 简介

PowerShell 是基于 .NET 的命令行外壳与脚本语言，使用对象管道，擅长系统管理与自动化。

#### 2. 关键字与示例

- **`function`**：定义函数。

```powershell
function Add($a, $b) {
  $a + $b
}

Add 1 2
```

- **`if` / `elseif` / `else`**：条件。

```powershell
if ($x -gt 10) {
  "big"
} elseif ($x -eq 10) {
  "equal"
} else {
  "small"
}
```

---

### 42. Transact-SQL

#### 1. 简介

Transact-SQL（T‑SQL）是 Microsoft SQL Server 等使用的 SQL 扩展，加入了过程化控制结构。

#### 2. 关键字与示例

- **`DECLARE` / `BEGIN` / `END` / `IF`**。

```sql
DECLARE @x INT = 10;

IF @x > 5
BEGIN
  PRINT 'big';
END;
```

---

### 43. X++

#### 1. 简介

X++ 是 Microsoft Dynamics 365 Finance and Operations 等产品使用的语言，语法类似 C#，用于业务逻辑与数据访问。

#### 2. 关键字与示例（概念性）

- **`class` / `static` / `void`**。

```x++
class Hello {
    public static void main(Args _args) {
        info("Hello, world");
    }
}
```

---

### 44. ABAP

#### 1. 简介

ABAP 是 SAP 系统中的主要编程语言，用于编写业务逻辑、报表和扩展。

#### 2. 关键字与示例

- **`REPORT` / `WRITE`**。

```abap
REPORT zhello.
WRITE 'Hello, world'.
```

- **`SELECT`**：数据库访问。

```abap
SELECT * FROM mara INTO TABLE @DATA(lt_mara) UP TO 10 ROWS.
```

---

### 45. Elixir

#### 1. 简介

Elixir 构建在 Erlang VM（BEAM）之上，是一门函数式语言，适合高并发 Web 服务和实时系统。

#### 2. 关键字列表及示例

- **`defmodule` / `def` / `defp`**。

```elixir
defmodule Greeter do
  def greet(name) do
    IO.puts("Hello, #{name}")
  end
end
```

- **`case` / 模式匹配**。

```elixir
case {:ok, 42} do
  {:ok, value} -> IO.puts(value)
  :error -> :noop
end
```

---

### 46. Zig

#### 1. 简介

Zig 是新兴的系统编程语言，主打可预测性能、简洁语义和手动内存管理，常与 C 互操作。

#### 2. 关键字列表及示例

- **`fn` / `var` / `const`**。

```zig
const std = @import("std");

fn add(a: i32, b: i32) i32 {
    return a + b;
}

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("{}\n", .{add(1, 2)});
}
```

---

### 47. ActionScript

#### 1. 简介

ActionScript 是基于 ECMAScript 的语言，主要用于 Adobe Flash 平台上的交互内容和动画。

#### 2. 关键字列表及示例

- **`function` / `var` / `class`**。

```actionscript
package {
  public class Main {
    public function Main() {
      trace(add(1, 2));
    }
  }
}

function add(a:int, b:int):int {
  return a + b;
}
```

---

### 48. D

#### 1. 简介

D 语言结合了 C++ 的性能和更现代的语法特性，支持系统编程和高层抽象。

#### 2. 关键字列表及示例

- **`module` / `import` / `auto`**。

```d
import std.stdio;

auto add(int a, int b) {
  return a + b;
}

void main() {
  writeln(add(1, 2));
}
```

---

### 49. Logo

#### 1. 简介

Logo 是面向教育的编程语言，以“海龟绘图”著称，常用于儿童编程启蒙。

#### 2. 常见命令（类比关键字）

- **`FORWARD` / `BACK` / `LEFT` / `RIGHT`**：控制海龟移动。

```logo
FORWARD 100
RIGHT 90
FORWARD 100
```

---

### 50. PL/I

#### 1. 简介

PL/I 是面向科学计算和商业应用的多用途语言，在大型机环境中使用。

#### 2. 关键字与示例

- **`DECLARE` / `PROC` / `END`**。

```pli
HELLO: PROC OPTIONS(MAIN);
  PUT SKIP LIST('Hello, world');
END HELLO;
```

---

以上即 TIOBE Index 2026‑02 前 50 名编程语言的简介与关键字示例，后续如排名变化，可以按相同结构更新本页面中的顺序与内容。

---
layout: page
title: 编程语言列表（示例与关键字）
---

### 说明

本文档基于 Languish 的语言列表，数据来源参考：

- [Languish - Programming Language Trends](https://tjpalmer.github.io/languish/)

总共约有 **544** 种编程语言。由于完整覆盖所有语言会非常庞大，本页面：

- **为主流语言提供较完整的内容示例**
- **给出统一的章节结构与模板**
- 方便后续按需增补其他语言

建议为每种语言使用如下结构：

- **1. 简介**
- **2. 关键字列表**
  - 关键字名称
  - 功能说明
  - 简短代码示例

下面先给出若干主流语言的完整示例。

---

### Python

#### 1. 简介

Python 是一种强调可读性和快速开发的高级通用编程语言，广泛用于数据分析、机器学习、Web 开发、脚本自动化等领域。它拥有丰富的标准库和第三方生态。

#### 2. 关键字列表及示例

- **`def`**：定义函数。

```python
def add(a, b):
    return a + b

print(add(1, 2))
```

- **`class`**：定义类。

```python
class Person:
    def __init__(self, name):
        self.name = name

    def greet(self):
        print(f"Hello, {self.name}")

Person("Alice").greet()
```

- **`if` / `elif` / `else`**：条件分支。

```python
x = 10

if x > 10:
    print("large")
elif x == 10:
    print("equal")
else:
    print("small")
```

- **`for` / `while`**：循环。

```python
for i in range(3):
    print(i)

count = 0
while count < 3:
    print(count)
    count += 1
```

- **`try` / `except` / `finally` / `raise`**：异常处理。

```python
def safe_div(a, b):
    try:
        return a / b
    except ZeroDivisionError:
        return None
    finally:
        print("done")

if safe_div(1, 0) is None:
    raise ValueError("division failed")
```

- **`with`**：上下文管理（资源自动管理）。

```python
with open("data.txt", "w", encoding="utf-8") as f:
    f.write("hello")
```

- **`import` / `from` / `as`**：模块导入。

```python
import math
from collections import Counter as C

print(math.sqrt(4))
print(C("abca"))
```

---

### JavaScript

#### 1. 简介

JavaScript 最初是浏览器脚本语言，现在已经发展为全栈通用语言，可运行在浏览器、Node.js、Deno 等多种环境中。

#### 2. 关键字列表及示例

- **`let` / `const` / `var`**：变量和常量声明。

```javascript
let x = 1;      // 块级作用域，可重新赋值
const y = 2;    // 块级作用域，不可重新赋值
var z = 3;      // 函数作用域
```

- **`function`**：函数声明。

```javascript
function add(a, b) {
  return a + b;
}

console.log(add(1, 2));
```

- **`if` / `else if` / `else`**：条件分支。

```javascript
const score = 85;

if (score >= 90) {
  console.log("A");
} else if (score >= 80) {
  console.log("B");
} else {
  console.log("C");
}
```

- **`for` / `while` / `do...while`**：循环。

```javascript
for (let i = 0; i < 3; i++) {
  console.log(i);
}

let n = 0;
while (n < 3) {
  console.log(n);
  n++;
}
```

- **`switch`**：多分支选择。

```javascript
const day = 2;

switch (day) {
  case 1:
    console.log("Mon");
    break;
  case 2:
    console.log("Tue");
    break;
  default:
    console.log("Other");
}
```

- **`try` / `catch` / `finally` / `throw`**：异常处理。

```javascript
function safeParse(json) {
  try {
    return JSON.parse(json);
  } catch (e) {
    return null;
  } finally {
    console.log("parsed");
  }
}

if (!safeParse("not json")) {
  throw new Error("invalid json");
}
```

- **`async` / `await`**：异步函数与等待 Promise。

```javascript
async function fetchData() {
  const res = await fetch("/api/data");
  const json = await res.json();
  console.log(json);
}

fetchData();
```

---

### TypeScript

#### 1. 简介

TypeScript 是 JavaScript 的超集，在 JS 基础上增加了静态类型系统和一些语言扩展特性，最终编译为普通 JavaScript 运行。

#### 2. 关键字列表及示例

- **`type` / `interface`**：类型别名与接口。

```typescript
type ID = number | string;

interface User {
  id: ID;
  name: string;
}

const u: User = { id: 1, name: "Alice" };
```

- **`enum`**：枚举类型。

```typescript
enum Direction {
  Up,
  Down,
  Left,
  Right,
}

function move(dir: Direction) {
  console.log("move", dir);
}

move(Direction.Up);
```

- **`implements` / `extends`**：接口实现与类继承。

```typescript
interface Greet {
  greet(): void;
}

class Person implements Greet {
  constructor(private name: string) {}
  greet() {
    console.log(`Hello, ${this.name}`);
  }
}

class Student extends Person {
  constructor(name: string, public grade: number) {
    super(name);
  }
}
```

- **`public` / `private` / `protected` / `readonly`**：成员可见性修饰符。

```typescript
class Counter {
  private count = 0;
  readonly name = "counter";

  public inc() {
    this.count++;
  }
}
```

---

### Java

#### 1. 简介

Java 是面向对象的通用编程语言，运行在 JVM 上，广泛用于企业级应用、Android 开发和大规模后端系统。

#### 2. 关键字列表及示例

- **`class` / `interface`**：类与接口。

```java
public class Person implements Greet {
    private String name;

    public Person(String name) {
        this.name = name;
    }

    @Override
    public void greet() {
        System.out.println("Hello, " + name);
    }
}
```

- **`public` / `private` / `protected` / `static` / `final`**：访问控制和修饰符。

```java
public final class MathUtil {
    private MathUtil() {}

    public static int add(int a, int b) {
        return a + b;
    }
}
```

- **`if` / `else` / `switch`**：条件与分支。

```java
int score = 85;

if (score >= 90) {
    System.out.println("A");
} else if (score >= 80) {
    System.out.println("B");
} else {
    System.out.println("C");
}
```

- **`try` / `catch` / `finally` / `throw` / `throws`**：异常处理。

```java
public int parse(String s) throws NumberFormatException {
    try {
        return Integer.parseInt(s);
    } catch (NumberFormatException e) {
        throw e;
    } finally {
        System.out.println("done");
    }
}
```

- **`for` / `while` / `do`**：循环。

```java
for (int i = 0; i < 3; i++) {
    System.out.println(i);
}
```

---

### C#

#### 1. 简介

C# 是由微软主导设计的现代面向对象语言，运行在 .NET 平台上，适用于桌面、Web、移动、游戏（Unity）等多种场景。

#### 2. 关键字列表及示例

- **`class` / `interface` / `struct`**：类型定义。

```csharp
public interface IGreet {
    void Greet();
}

public class Person : IGreet {
    public string Name { get; }
    public Person(string name) => Name = name;
    public void Greet() => Console.WriteLine($"Hello, {Name}");
}
```

- **`var` / `dynamic`**：类型推断与动态类型。

```csharp
var x = 10;        // 编译期推断类型为 int
dynamic y = "hi";  // 运行期决定成员解析
```

- **`async` / `await`**：异步编程。

```csharp
public async Task<string> FetchAsync(HttpClient client) {
    var res = await client.GetStringAsync("https://example.com");
    return res;
}
```

- **`using`**：资源自动释放（`IDisposable`）。

```csharp
using var file = File.OpenText("data.txt");
Console.WriteLine(file.ReadLine());
```

---

### C

#### 1. 简介

C 是一种过程式系统级编程语言，广泛用于操作系统、嵌入式、编译器和高性能系统软件。

#### 2. 关键字列表及示例

- **`int` / `char` / `float` / `double`**：基本类型关键字。

```c
int x = 10;
double y = 3.14;
```

- **`if` / `else` / `switch`**：条件控制。

```c
int n = 2;
switch (n) {
case 1:
    printf("one\n");
    break;
case 2:
    printf("two\n");
    break;
default:
    printf("other\n");
}
```

- **`for` / `while` / `do`**：循环。

```c
for (int i = 0; i < 3; i++) {
    printf("%d\n", i);
}
```

- **`struct` / `typedef`**：结构体与类型别名。

```c
typedef struct {
    int x;
    int y;
} Point;

Point p = { .x = 1, .y = 2 };
```

---

### C++

#### 1. 简介

C++ 在 C 基础上加入了面向对象、泛型与现代抽象机制，同时保留了对底层资源的精细控制能力。

#### 2. 关键字列表及示例

- **`class` / `struct` / `public` / `private`**：面向对象与访问控制。

```cpp
class Person {
public:
    explicit Person(std::string name) : name_(std::move(name)) {}
    void greet() const { std::cout << "Hello, " << name_ << "\n"; }
private:
    std::string name_;
};
```

- **`template` / `typename`**：模板与泛型。

```cpp
template <typename T>
T add(T a, T b) {
    return a + b;
}
```

- **`auto` / `constexpr` / `inline`**：类型推断与编译期计算。

```cpp
constexpr int square(int x) { return x * x; }

auto v = square(4);
```

---

### Go

#### 1. 简介

Go（Golang）是 Google 设计的静态类型、编译型语言，强调简单、高并发和快速编译，内置 goroutine 和 channel 并发模型。

#### 2. 关键字列表及示例

- **`func`**：函数或方法定义。

```go
func Add(a, b int) int {
    return a + b
}
```

- **`go` / `chan` / `select`**：轻量级并发与通道。

```go
func worker(ch chan int) {
    ch <- 42
}

func main() {
    ch := make(chan int)
    go worker(ch)
    v := <-ch
    println(v)
}
```

- **`defer`**：延迟执行（通常用于资源释放）。

```go
func readFile() {
    f, _ := os.Open("data.txt")
    defer f.Close()
}
```

---

### Rust

#### 1. 简介

Rust 是一门强调内存安全和并发安全的系统编程语言，通过所有权和借用系统在无需垃圾回收的前提下防止数据竞争和悬垂指针。

#### 2. 关键字列表及示例

- **`fn` / `let` / `mut`**：函数与变量绑定。

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

let mut x = 10;
x += 1;
```

- **`struct` / `enum` / `impl`**：数据类型与方法实现。

```rust
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn len2(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}
```

- **`match`**：模式匹配。

```rust
let value = Some(10);

match value {
    Some(v) if v > 5 => println!("big: {v}"),
    Some(v) => println!("small: {v}"),
    None => println!("none"),
}
```

- **`async` / `await`**：异步。

```rust
async fn fetch() -> Result<(), reqwest::Error> {
    let body = reqwest::get("https://example.com").await?.text().await?;
    println!("{body}");
    Ok(())
}
```

---

### HTML（超文本标记语言）

> 严格来说 HTML 不是“编程语言”，但在许多统计与趋势网站（包括 Languish）中常被列入语言列表。

#### 1. 简介

HTML 是用于描述 Web 页面结构的标记语言，由标签（tag）和属性（attribute）组成，配合 CSS 和 JavaScript 构建完整的 Web 应用。

#### 2. 常见标签及作用示例（类比关键字）

- **`<html>` / `<head>` / `<body>`**：文档结构。

```html
<!DOCTYPE html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <title>示例</title>
  </head>
  <body>
    <h1>你好</h1>
  </body>
  </html>
```

- **`<div>` / `<span>`**：块级 / 行内通用容器。

```html
<div class="card">
  <span class="title">标题</span>
</div>
```

- **`<a>` / `<img>` / `<button>`**：超链接、图片、按钮。

```html
<a href="https://example.com">链接</a>
<img src="logo.png" alt="Logo" />
<button type="button">点击</button>
```

---

### PHP

#### 1. 简介

PHP 是一种主要用于服务端 Web 开发的脚本语言，嵌入 HTML 使用，生态中有 Laravel、Symfony 等主流框架。

#### 2. 关键字列表及示例

- **`function`**：定义函数。

```php
<?php
function add(int $a, int $b): int {
    return $a + $b;
}
echo add(1, 2);
```

- **`class` / `interface`**：定义类与接口。

```php
<?php
interface Greet {
    public function greet(): void;
}

class Person implements Greet {
    public function __construct(private string $name) {}
    public function greet(): void {
        echo "Hello, {$this->name}";
    }
}
```

- **`if` / `elseif` / `else` / `foreach`**：条件与循环。

```php
<?php
$nums = [1, 2, 3];
foreach ($nums as $n) {
    if ($n % 2 === 0) {
        echo "even\n";
    } else {
        echo "odd\n";
    }
}
```

---

### Ruby

#### 1. 简介

Ruby 是一门强调开发者愉悦度、语法优雅的动态语言，常用于 Web 开发（如 Ruby on Rails）、脚本和自动化。

#### 2. 关键字列表及示例

- **`def` / `end`**：定义方法。

```ruby
def add(a, b)
  a + b
end

puts add(1, 2)
```

- **`class` / `module`**：类与模块。

```ruby
module Greeting
  def greet
    puts "Hello, #{@name}"
  end
end

class Person
  include Greeting
  def initialize(name)
    @name = name
  end
end
```

- **`if` / `elsif` / `else` / `unless`**：条件控制。

```ruby
x = 10

if x > 10
  puts "big"
elsif x == 10
  puts "equal"
else
  puts "small"
end

puts "ok" unless x < 0
```

---

### Kotlin

#### 1. 简介

Kotlin 是 JetBrains 设计的静态类型语言，可编译到 JVM、Android、Native 和 JavaScript，是现代 Android 开发的官方首选语言之一。

#### 2. 关键字列表及示例

- **`val` / `var`**：不可变 / 可变变量。

```kotlin
val name = "Alice"  // 只读
var age = 18        // 可变
age += 1
```

- **`fun`**：函数与方法定义。

```kotlin
fun add(a: Int, b: Int): Int = a + b

fun main() {
    println(add(1, 2))
}
```

- **`data class` / `object` / `companion object`**：数据类与单例。

```kotlin
data class User(val id: Int, val name: String)

object Config {
    const val Version = "1.0"
}
```

---

### Swift

#### 1. 简介

Swift 是 Apple 推出的现代静态类型语言，用于 iOS、macOS、watchOS、tvOS 等平台的应用开发。

#### 2. 关键字列表及示例

- **`let` / `var`**：常量与变量。

```swift
let name = "Alice"
var age = 18
age += 1
```

- **`struct` / `class` / `enum`**：值类型、引用类型和枚举。

```swift
struct Point {
    var x: Int
    var y: Int
}

enum Direction {
    case up, down, left, right
}
```

- **`if` / `guard` / `switch`**：控制流。

```swift
func greet(_ name: String?) {
    guard let name else { return }
    print("Hello, \(name)")
}
```

---

### Dart

#### 1. 简介

Dart 是 Google 推出的语言，常与 Flutter 框架一起用于跨平台移动、Web 与桌面应用开发。

#### 2. 关键字列表及示例

- **`var` / `final` / `const`**：变量与常量声明。

```dart
var x = 1;        // 可变
final y = 2;      // 运行期常量
const z = 3;      // 编译期常量
```

- **`class` / `extends` / `implements`**：面向对象。

```dart
class Person {
  final String name;
  Person(this.name);

  void greet() => print('Hello, $name');
}
```

---

### Scala

#### 1. 简介

Scala 结合了面向对象与函数式编程特性，运行在 JVM 上，常用于数据处理、分布式系统和后端服务。

#### 2. 关键字列表及示例

- **`object` / `class` / `trait`**：单例对象、类与特质。

```scala
trait Greet {
  def greet(): Unit
}

class Person(name: String) extends Greet {
  def greet(): Unit = println(s"Hello, $name")
}

object Main {
  def main(args: Array[String]): Unit = {
    new Person("Alice").greet()
  }
}
```

- **`val` / `var`**：不可变与可变变量。

```scala
val x = 1
var y = 2
y += 1
```

---

### Haskell

#### 1. 简介

Haskell 是纯函数式编程语言，具有惰性求值和强大的类型系统，适合抽象表达和形式化推理。

#### 2. 关键字列表及示例

- **`data` / `type`**：代数数据类型与类型别名。

```haskell
data Direction = Up | Down | Left | Right

type Name = String
```

- **`let` / `where`**：局部绑定。

```haskell
area r = pi * r2
  where r2 = r * r
```

---

### Elixir

#### 1. 简介

Elixir 是一种构建在 Erlang VM（BEAM）上的函数式语言，强调并发、容错和分布式，常用于 Web 服务与实时系统。

#### 2. 关键字列表及示例

- **`def` / `defp` / `defmodule`**：定义函数与模块。

```elixir
defmodule Greeter do
  def greet(name) do
    IO.puts("Hello, #{name}")
  end
end
```

- **`case` / `with`**：模式匹配控制流。

```elixir
case {:ok, 42} do
  {:ok, value} -> IO.puts(value)
  :error -> :noop
end
```

---

### Erlang

#### 1. 简介

Erlang 是为高并发、电信系统设计的函数式语言，提供轻量级进程和消息传递模型。

#### 2. 关键字列表及示例

- **`fun` / `case` / `receive`**：匿名函数、匹配与消息收发。

```erlang
loop() ->
    receive
        {From, Msg} ->
            From ! {ok, Msg},
            loop()
    end.
```

---

### R

#### 1. 简介

R 是面向统计计算和数据可视化的语言，广泛用于数据分析、统计建模和科研。

#### 2. 关键字列表及示例

- **`function`**：定义函数。

```r
add <- function(a, b) {
  a + b
}

add(1, 2)
```

- **`if` / `else` / `for` / `while`**：控制流。

```r
for (i in 1:3) {
  print(i)
}
```

---

### Julia

#### 1. 简介

Julia 是为数值计算和科学计算设计的高性能动态语言，兼具易用语法和接近 C 的速度。

#### 2. 关键字列表及示例

- **`function` / `end`**：函数。

```julia
function add(a, b)
    a + b
end

println(add(1, 2))
```

- **`struct` / `mutable struct`**：不可变 / 可变结构体。

```julia
struct Point
    x::Int
    y::Int
end
```

---

### SQL（结构化查询语言）

#### 1. 简介

SQL 是用于关系型数据库的查询与数据操作语言，几乎所有主流关系数据库（MySQL、PostgreSQL、SQLite 等）都支持。

#### 2. 常见语句及示例（类比关键字）

- **`SELECT` / `FROM` / `WHERE`**：查询数据。

```sql
SELECT id, name
FROM users
WHERE active = 1;
```

- **`INSERT` / `UPDATE` / `DELETE`**：修改数据。

```sql
INSERT INTO users (name) VALUES ('Alice');
```

- **`CREATE TABLE` / `ALTER TABLE` / `DROP TABLE`**：定义表结构。

```sql
CREATE TABLE users (
  id   INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
```

---

### Bash（Shell 脚本）

#### 1. 简介

Bash 是类 Unix 系统最常用的 Shell 之一，同时也是脚本语言，用于系统管理、自动化任务等。

#### 2. 关键字与常用语法示例

- **`if` / `then` / `elif` / `else` / `fi`**：条件。

```bash
if [ "$1" -gt 10 ]; then
  echo "big"
else
  echo "small"
fi
```

- **`for` / `while` / `do` / `done`**：循环。

```bash
for f in *.txt; do
  echo "$f"
done
```

---

### PowerShell

#### 1. 简介

PowerShell 是基于 .NET 的命令行外壳与脚本语言，使用管道传递对象而非纯文本，擅长系统管理与自动化。

#### 2. 关键字与常用语法示例

- **`function`**：定义函数。

```powershell
function Add($a, $b) {
  $a + $b
}

Add 1 2
```

- **`if` / `elseif` / `else`**：条件。

```powershell
if ($x -gt 10) {
  "big"
} elseif ($x -eq 10) {
  "equal"
} else {
  "small"
}
```

---

### Lua

#### 1. 简介

Lua 是一门轻量级脚本语言，常嵌入到游戏引擎和应用中作为配置与扩展语言。

#### 2. 关键字列表及示例

- **`function` / `local`**：函数与局部变量。

```lua
local function add(a, b)
  return a + b
end

print(add(1, 2))
```

- **`if` / `elseif` / `else` / `end`**：条件。

```lua
local x = 10
if x > 10 then
  print("big")
elseif x == 10 then
  print("equal")
else
  print("small")
end
```

---

### Perl

#### 1. 简介

Perl 以强大的文本处理能力著称，早期常用于 CGI、系统管理和日志分析等任务。

#### 2. 关键字列表及示例

- **`my` / `sub`**：变量与子例程。

```perl
use strict;
use warnings;

sub add {
  my ($a, $b) = @_;
  return $a + $b;
}

print add(1, 2);
```

- **`if` / `elsif` / `else` / `for` / `foreach`**：控制流。

```perl
for my $x (1..3) {
  print "$x\n";
}
```

---

### Objective-C

#### 1. 简介

Objective-C 是在 C 语言基础上加入 Smalltalk 风格消息发送的面向对象语言，主要用于早期的 macOS 和 iOS 开发。

#### 2. 关键字列表及示例

- **`@interface` / `@implementation`**：类声明与实现。

```objectivec
@interface Person : NSObject
@property (nonatomic, copy) NSString *name;
- (void)greet;
@end

@implementation Person
- (void)greet {
    NSLog(@"Hello, %@", self.name);
}
@end
```

- **`@autoreleasepool`**：自动释放池块。

```objectivec
@autoreleasepool {
    Person *p = [Person new];
    p.name = @"Alice";
    [p greet];
}
```

---

### F#

#### 1. 简介

F# 是 .NET 平台上的函数式优先语言，支持不可变数据、代数数据类型和模式匹配，也可与 C# 等语言互操作。

#### 2. 关键字列表及示例

- **`let`**：绑定值或函数。

```fsharp
let add a b = a + b
printfn "%d" (add 1 2)
```

- **`type` / 模式匹配 `match`**：定义类型与匹配。

```fsharp
type Direction = Up | Down | Left | Right

let describe dir =
  match dir with
  | Up -> "up"
  | Down -> "down"
  | Left -> "left"
  | Right -> "right"
```

---

### OCaml

#### 1. 简介

OCaml 是一门静态类型的函数式语言，同时支持面向对象特性，常用于编译器、形式化验证和系统工具开发。

#### 2. 关键字列表及示例

- **`let` / `let rec`**：绑定与递归函数。

```ocaml
let rec fib n =
  if n <= 1 then n
  else fib (n - 1) + fib (n - 2)
```

- **`type` / `match`**：代数数据类型与模式匹配。

```ocaml
type direction = Up | Down | Left | Right

let show = function
  | Up -> "up"
  | Down -> "down"
  | Left -> "left"
  | Right -> "right"
```

---

### Clojure

#### 1. 简介

Clojure 是运行在 JVM 和 JavaScript 平台上的现代 Lisp 方言，强调不可变数据结构和并发编程。

#### 2. 关键字列表及示例

- **`def` / `defn`**：定义变量与函数。

```clojure
(def x 1)

(defn add [a b]
  (+ a b))

(println (add 1 2))
```

- **`let` / `if` / `cond`**：局部绑定与条件。

```clojure
(let [n 10]
  (cond
    (> n 10) "big"
    (= n 10) "equal"
    :else "small"))
```

---

### Scheme

#### 1. 简介

Scheme 是 Lisp 家族的一员，语法极简、核心概念精炼，常用于教学、研究和实验性语言设计。

#### 2. 关键字列表及示例

- **`define` / `lambda`**：定义变量与匿名函数。

```scheme
(define (add a b)
  (+ a b))

(display (add 1 2))
```

- **`if` / `cond` / `let`**：控制流与局部绑定。

```scheme
(let ((x 10))
  (if (> x 10)
      (display "big")
      (display "not-big")))
```

---

### Common Lisp

#### 1. 简介

Common Lisp 是一种多范式 Lisp 语言，支持面向对象（CLOS）、宏系统和动态元编程。

#### 2. 关键字列表及示例

- **`defun` / `let`**：定义函数与局部绑定。

```lisp
(defun add (a b)
  (+ a b))

(let ((x 1) (y 2))
  (print (add x y)))
```

- **`defclass` / `defmethod`**：定义类与方法。

```lisp
(defclass person ()
  ((name :initarg :name :accessor person-name)))
```

---

### Elm

#### 1. 简介

Elm 是一门针对前端 Web 应用的函数式语言，具有强类型系统和“无运行时异常”的设计目标。

#### 2. 关键字列表及示例

- **`type` / `type alias`**：自定义类型与别名。

```elm
type Direction = Up | Down | Left | Right

type alias User =
    { id : Int
    , name : String
    }
```

- **`case .. of`**：模式匹配。

```elm
describe dir =
    case dir of
        Up -> "up"
        Down -> "down"
        Left -> "left"
        Right -> "right"
```

---

### Solidity

#### 1. 简介

Solidity 是在以太坊等区块链平台上编写智能合约的主流语言，语法类似于 JavaScript / C++。

#### 2. 关键字列表及示例

- **`contract` / `function`**：定义合约与函数。

```solidity
pragma solidity ^0.8.0;

contract Counter {
    uint256 public value;

    function inc() public {
        value += 1;
    }
}
```

- **`mapping` / `address`**：映射与地址类型。

```solidity
mapping(address => uint256) public balances;
```

---

### Groovy

#### 1. 简介

Groovy 是 JVM 上的动态语言，语法类似 Java 但更简洁，常用于脚本、构建工具（如 Gradle）和 Web 开发。

#### 2. 关键字列表及示例

- **`def` / 闭包 `{}`**：动态变量与闭包。

```groovy
def add = { a, b -> a + b }
println add(1, 2)
```

- **`class` / `implements` / `extends`**：面向对象。

```groovy
class Person {
  String name
}
```

---

### Crystal

#### 1. 简介

Crystal 是一门语法类似 Ruby 的静态编译语言，目标是提供接近 C 的性能和友好的开发体验。

#### 2. 关键字列表及示例

- **`def` / `class`**：函数与类。

```crystal
def add(a, b)
  a + b
end

puts add(1, 2)
```

- **`if` / `else` / `elsif`**：条件语句。

```crystal
x = 10
if x > 10
  puts "big"
elsif x == 10
  puts "equal"
else
  puts "small"
end
```

---

### Nim

#### 1. 简介

Nim 是一门静态类型、编译型语言，语法简洁，支持宏和元编程，能编译为 C、C++ 或 JavaScript。

#### 2. 关键字列表及示例

- **`proc`**：过程（函数）定义。

```nim
proc add(a, b: int): int =
  a + b

echo add(1, 2)
```

- **`var` / `let`**：可变与不可变绑定。

```nim
var x = 1
let y = 2
```

---

### Fortran

#### 1. 简介

Fortran 是最早的高级编程语言之一，主要用于科学计算和工程模拟领域。

#### 2. 关键字列表及示例

- **`program` / `end program`**：程序入口。

```fortran
program hello
  print *, "Hello, world"
end program hello
```

- **`do` / `if`**：循环与条件。

```fortran
do i = 1, 3
  print *, i
end do
```

---

### COBOL

#### 1. 简介

COBOL 是为商业数据处理设计的老牌语言，仍在许多大型机和金融系统中使用。

#### 2. 关键字列表及示例

- **`IDENTIFICATION DIVISION` / `PROCEDURE DIVISION`**：程序结构。

```cobol
IDENTIFICATION DIVISION.
PROGRAM-ID. HELLO.
PROCEDURE DIVISION.
    DISPLAY "Hello, world".
    STOP RUN.
```

---

### MATLAB

#### 1. 简介

MATLAB 是面向矩阵运算和数值分析的商业语言与环境，广泛用于工程、控制和信号处理。

#### 2. 关键字列表及示例

- **`function`**：函数定义。

```matlab
function y = add(a, b)
  y = a + b;
end
```

- **`for` / `if`**：控制流。

```matlab
for i = 1:3
  disp(i);
end
```

---

### VBA（Visual Basic for Applications）

#### 1. 简介

VBA 是嵌入在 Office 等应用中的脚本语言，用于编写宏和自动化任务。

#### 2. 关键字列表及示例

- **`Sub` / `Function`**：过程与函数。

```vbnet
Sub Hello()
  MsgBox "Hello"
End Sub
```

- **`If` / `Then` / `Else` / `End If`**：条件。

```vbnet
If x > 10 Then
  MsgBox "big"
Else
  MsgBox "small"
End If
```

---

### 其他语言与扩展方式

上文已经为一批主流语言给出了**完整的「简介 + 关键字与示例」**。由于 Languish 数据集中包含约 544 种语言，剩余较少使用或较小众的语言可以按以下方式扩展：

- **继续沿用上述结构，为感兴趣的语言补充内容**
- 或者使用脚本从 Languish 的数据源解析出语言名称列表，再批量生成类似的 Markdown 小节作为起点

一个简单的扩展模板（复制后将 `YourLang` 和内容替换为目标语言）：

```markdown
### YourLang

#### 1. 简介

（用 2–4 句话介绍该语言的定位、主要场景和特点。）

#### 2. 关键字列表及示例

- **`keyword1`**：说明关键字的作用。

```yourlang
// 示例代码
```

- **`keyword2`**：说明关键字的作用。

```yourlang
// 示例代码
```
```

---

### TIOBE 下一批 50 种语言（按字母排序，仅列名称）

以下为 TIOBE Index 中 #51–#100 的编程语言名称列表，来自“Next 50 Programming Languages” 段落，按字母顺序列出，便于后续按本页模板继续补充「简介 + 关键字」内容：

- **Algol**
- **Alice**
- **Apex**
- **Awk**
- **Bash**
- **C shell**
- **Caml**
- **CL (OS/400)**
- **Clojure**
- **Common Lisp**
- **F#**
- **Forth**
- **GAMS**
- **GML**
- **Groovy**
- **Hack**
- **Icon**
- **Inform**
- **Io**
- **J**
- **J#**
- **JScript**
- **JScript.NET**
- **Korn shell**
- **ML**
- **Modula-2**
- **Mojo**
- **MQL5**
- **MS-DOS batch**
- **NATURAL**
- **Nim**
- **OCaml**
- **OpenCL**
- **Q**
- **REXX**
- **RPG**
- **S**
- **Scheme**
- **Small Basic**
- **Smalltalk**
- **Solidity**
- **SPARK**
- **Structured Text**
- **Tcl**
- **V**
- **Vala/Genie**
- **VHDL**
- **WebAssembly**
- **Wolfram**
- **Xojo**


