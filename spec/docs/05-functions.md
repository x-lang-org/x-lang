# 第5章 函数

## 5.1 函数声明

### 形式语法

```
FunctionDeclaration ::= 'function' Identifier Parameters ('->' Type)? ('with' EffectList)? FunctionBody

Parameters ::= '(' (Parameter (',' Parameter)*)? ')'

Parameter ::= Identifier (':' Type)? ('=' Expression)?

FunctionBody ::= '=' Expression
               | Block
```

### 说明

函数是 X 语言中最基本的抽象单元。使用 `function` 关键字声明（全称，不缩写）。

1. **函数名**遵循标识符命名规则，绑定在当前作用域中。
2. **参数列表**定义函数的输入。每个参数有名称、可选的类型注解和可选的默认值。有默认值的参数必须位于参数列表末尾。
3. **返回类型**可以显式声明（`-> Type`），也可由编译器通过 Hindley-Milner 推断自动推导。公共 API 建议写明返回类型以提升可读性。
4. **效果注解**用 `with` 分隔符声明函数可能产生的效果（见 §5.7）。
5. **函数体**有两种形式：
   - 表达式形式：`= expression`，直接返回表达式的值
   - 块形式：`{ statements }`，执行块内语句，最后一个表达式为返回值，或通过 `return` 显式返回

```x
function add(a: Integer, b: Integer) -> Integer = a + b

function greet(name: String) -> String {
    let message = "Hello, {name}!"
    message
}

function increment(x: Integer, step: Integer = 1) -> Integer = x + step
```

### 类型规则

$$
\frac{\Gamma,\; x_1: T_1,\;\ldots,\; x_n: T_n \;\vdash\; e : R}
     {\Gamma \;\vdash\; \texttt{function}\; f(x_1: T_1,\;\ldots,\; x_n: T_n) = e \;:\; (T_1,\;\ldots,\; T_n) \to R}
$$

当省略返回类型注解时，编译器对函数体 $e$ 执行 HM 类型推断以确定 $R$：

$$
\frac{\Gamma,\; x_1: T_1,\;\ldots,\; x_n: T_n \;\vdash\; e : R \quad (\text{inferred})}
     {\Gamma \;\vdash\; \texttt{function}\; f(x_1: T_1,\;\ldots,\; x_n: T_n) = e \;:\; (T_1,\;\ldots,\; T_n) \to R}
$$

---

## 5.2 匿名函数（Lambda）

### 形式语法

```
Lambda ::= '(' (Identifier (',' Identifier)*)? ')' '->' Expression
         | '(' (Identifier (',' Identifier)*)? ')' '->' Block
         | '.' Identifier                            // 点缩写
```

### 说明

Lambda 创建匿名函数值，可捕获周围作用域的变量（闭包）。使用 `->` 箭头语法。

```x
let double = (x) -> x * 2

let add = (x, y) -> x + y

let process = (x) -> {
    let result = x * 2
    result + 1
}
```

**点缩写（dot shorthand）** 提供简洁的字段/方法访问语法，等价于以对象为参数的单参数 lambda：

```x
let names = users |> map(.name)             // 等价于 map((u) -> u.name)
let adults = users |> filter(.age >= 18)    // 等价于 filter((u) -> u.age >= 18)
```

### 求值规则

$$
\llbracket (x_1,\;\ldots,\; x_n) \to e \rrbracket^g = \text{closure}((x_1,\;\ldots,\; x_n),\; e,\; g)
$$

$$
\text{apply}(\text{closure}(\text{params},\; \text{body},\; \text{env}),\; v_1,\;\ldots,\; v_n) = \llbracket \text{body} \rrbracket^{\text{env}[x_1 \mapsto v_1,\;\ldots,\; x_n \mapsto v_n]}
$$

---

## 5.3 函数调用

### 形式语法

```
FunctionCall ::= Expression '(' (Argument (',' Argument)*)? ')'

Argument ::= Expression
           | Identifier '=' Expression
```

### 说明

1. **位置参数**：按顺序与函数参数匹配，数量和类型必须兼容。
2. **命名参数**：通过参数名指定，可以任意顺序。命名参数必须出现在位置参数之后。
3. **默认参数**：调用时可省略有默认值的参数，省略时使用声明处的默认值。

```x
function formatDate(year: Integer, month: Integer, day: Integer,
                    separator: String = "-") -> String {
    "{year}{separator}{month}{separator}{day}"
}

formatDate(2026, 3, 3)                       // 位置参数
formatDate(2026, 3, 3, separator = "/")      // 命名参数
formatDate(year = 2026, month = 3, day = 3)  // 全部命名
```

**管道运算符** `|>` 将左侧表达式作为第一个参数传入右侧函数，支持函数链式调用：

```x
let result = data
    |> filter(is_valid)
    |> map(transform)
    |> take(10)
```

### 求值规则

$$
\llbracket f(e_1,\;\ldots,\; e_n) \rrbracket^g = v
\quad\text{where}\quad
\begin{cases}
  f_{\text{val}} = \llbracket f \rrbracket^g \\
  v_i = \llbracket e_i \rrbracket^g \quad (1 \le i \le n) \\
  v = \text{apply}(f_{\text{val}},\; v_1,\;\ldots,\; v_n)
\end{cases}
$$

---

## 5.4 高阶函数

### 定义

> 高阶函数（higher-order function）是以函数为参数或以函数为返回值的函数。

函数在 X 中是一等公民——可以作为参数传递、作为返回值返回、赋值给变量。

```x
function apply_twice(f: (Integer) -> Integer, x: Integer) -> Integer {
    f(f(x))
}

function make_adder(n: Integer) -> (Integer) -> Integer {
    (x) -> x + n
}

let add5 = make_adder(5)
add5(10)  // 15
```

**柯里化（Currying）与部分应用**：

```x
function multiply(a: Integer, b: Integer) -> Integer = a * b

let double = multiply(2, _)   // 部分应用
double(5)  // 10
```

### 形式化

$$
\llbracket \text{map}(f,\; []) \rrbracket^g = []
$$

$$
\llbracket \text{map}(f,\; x :: xs) \rrbracket^g = \llbracket f(x) \rrbracket^g :: \llbracket \text{map}(f,\; xs) \rrbracket^g
$$

$$
\llbracket \text{compose}(f,\; g)(x) \rrbracket^g = \llbracket f(g(x)) \rrbracket^g
$$

---

## 5.5 递归函数

### 形式语法

```
RecursiveFunction ::= 'function' 'rec'? Identifier Parameters ('->' Type)? FunctionBody
```

### 说明

函数可以在其体内引用自身实现递归。`rec` 关键字是可选的，主要用于明确表示递归意图或辅助编译器推断。递归函数必须有终止条件，否则导致无限递归。

```x
function factorial(n: Integer) -> Integer {
    match n {
        0 => 1
        _ => n * factorial(n - 1)
    }
}

function rec fibonacci(n: Integer) -> Integer {
    match n {
        0 => 0
        1 => 1
        _ => fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```

**分段定义（Piecewise）** 允许将多个 case 写成独立的函数子句：

```x
function fibonacci(0) -> Integer = 0
function fibonacci(1) -> Integer = 1
function fibonacci(n) -> Integer = fibonacci(n - 1) + fibonacci(n - 2)
```

### 求值规则

$$
\llbracket \texttt{let rec}\; f = \lambda x.\; e \rrbracket^g = ((),\; g')
\quad\text{where}\quad
g' = g[f \mapsto \text{fix}(\lambda f.\; \lambda x.\; e)]
$$

$$
\text{fix}(F) = F(\text{fix}(F)) \qquad \text{(Y combinator)}
$$

---

## 5.6 多态函数

### 形式语法

```
TypeParameters ::= '<' TypeParameter (',' TypeParameter)* '>'

TypeParameter ::= Identifier (':' TypeConstraint)?

TypeConstraint ::= TraitName ('+' TraitName)*
```

### 说明

函数可以有类型参数（泛型），使用 `<>` 语法。类型参数在调用时由编译器根据实参类型推断，也可以显式指定。类型约束限定类型参数必须满足的 trait 条件。

```x
function identity<T>(x: T) -> T = x

function max<T: Comparable>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

function swap<A, B>(pair: (A, B)) -> (B, A) = (pair.1, pair.0)

// 调用：类型由推断确定
let x = identity(42)         // T = Integer
let m = max(3, 7)            // T = Integer
let s = swap(("hello", 42))  // A = String, B = Integer
```

### 类型规则

$$
\frac{\Gamma,\; \alpha_1,\;\ldots,\;\alpha_n,\; x_1: T_1,\;\ldots,\; x_m: T_m \;\vdash\; e : R}
     {\Gamma \;\vdash\; \texttt{function}\; f\langle\alpha_1,\;\ldots,\;\alpha_n\rangle(x_1: T_1,\;\ldots,\; x_m: T_m) = e \;:\;
      \forall\alpha_1\cdots\alpha_n.\;(T_1,\;\ldots,\; T_m) \to R}
$$

带约束的情形：

$$
\frac{\Gamma,\; \alpha: C_1 + \cdots + C_k,\; x: \alpha \;\vdash\; e : R}
     {\Gamma \;\vdash\; \texttt{function}\; f\langle\alpha: C_1 + \cdots + C_k\rangle(x: \alpha) = e \;:\;
      \forall(\alpha: C_1 + \cdots + C_k).\; (\alpha) \to R}
$$

---

## 5.7 效果注解

### 形式语法

```
EffectAnnotation ::= 'with' EffectList

EffectList ::= Effect (',' Effect)*

Effect ::= 'IO'
         | 'Async'
         | 'Throws' '<' Type '>'
         | 'State' '<' Type '>'
         | 'NonDet'
         | Identifier                    // 用户自定义效果
```

### 说明

X 使用 `with` 关键字在函数签名中显式声明效果。纯函数（无效果）不需要效果注解。效果注解使副作用在类型层面可见，由编译器静态检查。

```x
// 纯函数——无效果
function add(a: Integer, b: Integer) -> Integer = a + b

// IO 效果
function printMessage(message: String) -> () with IO {
    print(message)
}

// 可能失败——Throws 效果表示返回 Result<T, E>
function readFile(path: String) -> String with IO, Throws<FileNotFound> {
    let file = open(path)?
    file.readAll()
}

// 多效果
function processData(url: String) -> Data with Async, IO, Throws<NetworkError> {
    let response = await fetch(url)?
    parse(response.body)?
}
```

**`needs`/`given` 依赖注入** 也通过效果系统实现（详见 §7.4）：

```x
function getUser(id: Integer) -> User with Throws<NotFound>
    needs Database, Logger {
    let rows = Database.query("SELECT * FROM users WHERE id = {id}")?
    Logger.info("Queried user {id}")
    rows.first() ?? Err(NotFound { id })
}
```

### 效果类型规则

$$
\frac{\Gamma \;\vdash\; f : (T_1,\;\ldots,\; T_n) \to R \;\texttt{with}\; \{E_1,\;\ldots,\; E_k\}
      \qquad
      \Gamma \;\vdash\; e_i : T_i \;\;(1 \le i \le n)}
     {\Gamma \;\vdash\; f(e_1,\;\ldots,\; e_n) : R \;\texttt{with}\; \{E_1,\;\ldots,\; E_k\}}
$$

效果集合的子类型关系：

$$
\frac{\Delta_1 \subseteq \Delta_2}
     {(T_1,\;\ldots,\; T_n) \to R \;\texttt{with}\; \Delta_1 \;<:\; (T_1,\;\ldots,\; T_n) \to R \;\texttt{with}\; \Delta_2}
$$

---

## 5.8 闭包与环境捕获

### 形式定义

$$
\text{Closure} = \text{Parameters} \times \text{Body} \times \text{Environment}
$$

$$
\text{Environment} = \text{Identifier} \to \text{Value}
$$

### 说明

闭包在定义时捕获周围作用域的变量。被捕获的变量保持与外部作用域的关联；若外部变量为 `let mutable`，闭包内外的修改相互可见。

```x
function make_counter() -> () -> Integer {
    let mutable count = 0
    () -> {
        count = count + 1
        count
    }
}

let counter = make_counter()
counter()  // 1
counter()  // 2
counter()  // 3
```

### 捕获规则

自由变量集合：

$$
\text{FV}(\lambda x.\; e) = \text{FV}(e) \setminus \{x\}
$$

环境捕获：

$$
\text{capture}(g,\; \text{vars}) = \{ x \mapsto g(x) \mid x \in \text{vars} \}
$$

闭包构造：

$$
\llbracket \lambda x.\; e \rrbracket^g = \text{closure}((x),\; e,\; \text{capture}(g,\; \text{FV}(\lambda x.\; e)))
$$

---

**本章规范采用 `function` 全称关键字、`with` 效果注解语法和 Hindley-Milner 类型推断，结合数学形式化与 X 代码示例定义函数语义。**
