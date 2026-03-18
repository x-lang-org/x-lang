# 第1章 词法结构

本章定义 X 语言的词法结构——源代码如何被分解为词法记号（token）序列。

---

## 1.1 字符集

### 定义

X 语言源文本使用 Unicode 字符集：

```
Unicode = { U+0000, U+0001, ..., U+10FFFF }
```

### 源码编码

X 语言源文件**必须**使用 UTF-8 编码。BOM（字节顺序标记 U+FEFF）在文件开头允许但不推荐，词法分析器会忽略它。

### 输入解析

```
Source → SourceChar*
SourceChar → Unicode \ { CR, LF, NEL, LS, PS }
           | Newline
Newline → CR | LF | CR LF | NEL | LS | PS

其中：
  CR  = U+000D（回车）
  LF  = U+000A（换行）
  NEL = U+0085（下一行）
  LS  = U+2028（行分隔符）
  PS  = U+2029（段落分隔符）
```

### 行与列

词法分析器维护源码位置信息 `(file, line, column)`，用于错误报告：

```
Position = (file: String, line: Natural₁, column: Natural₁)
Span = (start: Position, end: Position)
```

行号从 1 开始，列号从 1 开始，以 Unicode 标量值（scalar value）为单位计数。

---

## 1.2 词法记号

### 记号定义

```
Token → Keyword
      | Identifier
      | Literal
      | Operator
      | Punctuator
```

注释和空白字符不产生记号，在词法分析阶段被消耗。

```
TokenStream = { t ∈ Token* | t 由 Source 经词法分析产生 }
```

---

### 1.2.1 关键字

#### 定义

关键字是保留的标识符，不能用作变量名、函数名或类型名。

#### 声明关键字（Declaration Keywords）

```
DeclarationKeyword → 'let' | 'mutable' | 'function' | 'async'
                   | 'class' | 'trait' | 'type' | 'module' | 'const'
```

| 关键字 | 语义 | 示例 |
|--------|------|------|
| `let` | 不可变绑定 | `let x = 42` |
| `mutable` | 可变绑定标记 | `let mutable count = 0` |
| `function` | 函数定义 | `function add(a, b) { a + b }` |
| `async` | 异步函数修饰 | `async function fetch() { ... }` |
| `class` | 类定义 | `class Animal { ... }` |
| `trait` | 接口/行为约束定义 | `trait Printable { ... }` |
| `type` | 类型别名/ADT 定义 | `type Color = Red \| Green \| Blue` |
| `module` | 模块声明 | `module math.utils` |
| `const` | 编译期常量 | `const MAX_SIZE = 1024` |

#### 控制流关键字（Control Keywords）

```
ControlKeyword → 'if' | 'else' | 'for' | 'in' | 'while'
              | 'return' | 'match' | 'break' | 'continue'
```

| 关键字 | 语义 | 示例 |
|--------|------|------|
| `if` / `else` | 条件分支 | `if x > 0 { ... } else { ... }` |
| `for` / `in` | 迭代循环 | `for item in list { ... }` |
| `while` | 条件循环 | `while running { ... }` |
| `return` | 函数返回 | `return Ok(value)` |
| `match` | 模式匹配 | `match shape { Circle { r } => ... }` |
| `break` | 跳出循环 | `break` |
| `continue` | 跳过当前迭代 | `continue` |

#### 效果关键字（Effect Keywords）

```
EffectKeyword → 'needs' | 'given' | 'await' | 'with'
             | 'together' | 'race' | 'atomic' | 'retry'
```

| 关键字 | 语义 | 示例 |
|--------|------|------|
| `needs` | 声明效果/依赖 | `function f() -> T needs Database` |
| `with` | 效果标注 | `-> T with IO, Async` |
| `given` | 注入依赖实现 | `f() given { Database <- Pg.live }` |
| `await` | 等待异步结果 | `let data = await fetch(url)` |
| `together` | 并行等待全部完成 | `await together { f(), g() }` |
| `race` | 取最快完成的结果 | `await race { f(), g() }` |
| `atomic` | STM 原子事务 | `atomic { balance -= 100 }` |
| `retry` | STM 事务重试 | `if balance < 0 { retry }` |

#### 字面量关键字（Literal Keywords）

```
LiteralKeyword → 'true' | 'false'
              | 'None' | 'Some' | 'Ok' | 'Err'
```

| 关键字 | 语义 | 类型 |
|--------|------|------|
| `true` | 布尔真 | `Boolean` |
| `false` | 布尔假 | `Boolean` |
| `None` | 无值 | `Option<T>` |
| `Some` | 有值构造器 | `Option<T>` |
| `Ok` | 成功构造器 | `Result<T, E>` |
| `Err` | 错误构造器 | `Result<T, E>` |

#### 修饰符关键字（Modifier Keywords）

```
ModifierKeyword → 'public' | 'private' | 'protected' | 'internal'
               | 'static' | 'abstract' | 'final' | 'override' | 'virtual'
```

| 关键字 | 语义 |
|--------|------|
| `public` | 公共可见性（对所有代码可见） |
| `private` | 私有可见性（仅当前类/模块可见） |
| `protected` | 保护可见性（当前类及子类可见） |
| `internal` | 模块内部可见性（仅当前模块可见） |
| `static` | 静态成员（属于类而非实例） |
| `abstract` | 抽象成员（必须在子类中实现） |
| `final` | 最终成员（不可被重写或继承） |
| `override` | 重写父类方法 |
| `virtual` | 可被子类重写的方法 |

#### 其他关键字（Other Keywords）

```
OtherKeyword → 'import' | 'export' | 'with' | 'where'
            | 'and' | 'or' | 'not' | 'is' | 'as'
            | 'weak' | 'implement' | 'extends'
            | 'new' | 'this' | 'super'
```

| 关键字 | 语义 | 示例 |
|--------|------|------|
| `import` | 导入模块/符号 | `import std.collections.HashMap` |
| `export` | 导出符号 | `export function public_api() { }` |
| `with` | copy-with 更新 | `point with { x: 5.0 }` |
| `where` | 守卫条件/过滤 | `users where .active` |
| `and` | 逻辑与 | `a and b` |
| `or` | 逻辑或 | `a or b` |
| `not` | 逻辑非 | `not a` |
| `is` | 类型检查 | `x is Integer` |
| `as` | 类型转换 | `x as Float` |
| `weak` | 弱引用标记 | `weak Option<Parent>` |
| `implement` | 实现 trait | `implement Printable for User { ... }` |
| `extends` | 类继承 | `class Dog extends Animal { ... }` |
| `new` | 构造函数 | `new(name: String) { ... }` |
| `this` | 当前实例引用 | `this.name` |
| `super` | 父类引用 | `super.method()` |

#### 完整关键字文法

```
Keyword → DeclarationKeyword
        | ControlKeyword
        | EffectKeyword
        | LiteralKeyword
        | ModifierKeyword
        | OtherKeyword
```

#### 已移除的关键字

以下关键字在 X 语言中**不存在**，不是保留字：

| 已移除 | 原因 | 替代方案 |
|--------|------|---------|
| `fun` | 缩写 | `function` |
| `fn` | 缩写 | `function` |
| `mut` | 缩写 | `mutable` |
| `var` | 含义不够显式 | `let mutable` |
| `val` | 与 `let` 重复 | `let` |
| `when` | 模式匹配改用 `match` | `match` |
| `wait` | 缩写/生造 | `await` |
| `can` | 语义不够清晰 | `implement` |
| `try` | 无异常机制 | `Result<T, E>` + `?` |
| `catch` | 无异常机制 | `match` on `Result` |
| `finally` | 无异常机制 | RAII / `with` 资源管理 |
| `throw` | 无异常机制 | `return Err(...)` |
| `null` | 无 null | `None` from `Option<T>` |
| `none` | 小写形式 | `None`（大写） |
| `some` | 小写形式 | `Some`（大写） |
| `ok` | 小写形式 | `Ok`（大写） |
| `err` | 小写形式 | `Err`（大写） |
| `pub` | 缩写 | `public` |
| `mod` | 缩写 | `module` |
| `impl` | 缩写 | `implement` |

---

### 1.2.2 标识符

#### 定义

标识符用于命名变量、函数、类型、模块等程序实体。

```
Identifier → IdentifierStart IdentifierPart*
IdentifierStart → UnicodeLetter | '_'
IdentifierPart → IdentifierStart | UnicodeDigit | '-'

UnicodeLetter → { c ∈ Unicode | GeneralCategory(c) ∈ { Lu, Ll, Lt, Lm, Lo, Nl } }
UnicodeDigit → { c ∈ Unicode | GeneralCategory(c) = Nd }
```

其中：
- `Lu` = 大写字母（Uppercase Letter）
- `Ll` = 小写字母（Lowercase Letter）
- `Lt` = 首字母大写（Titlecase Letter）
- `Lm` = 修饰字母（Modifier Letter）
- `Lo` = 其他字母（Other Letter）
- `Nl` = 字母型数字（Letter Number）
- `Nd` = 十进制数字（Decimal Number）

#### 规则

1. 标识符**不能**以数字或连字符开头
2. 标识符**不能**与关键字同名
3. 标识符**大小写敏感**：`userName` 和 `Username` 是不同的标识符
4. 连字符 `-` 在标识符中间合法，鼓励 kebab-case 命名

#### 命名约定

| 类别 | 约定 | 示例 |
|------|------|------|
| 变量、函数、参数 | snake_case 或 kebab-case | `user_name`、`user-name` |
| 类型、类、trait | PascalCase | `HashMap`、`UserService` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_SIZE`、`PI` |
| 模块 | 小写点分隔 | `std.collections` |

#### 示例

```
ValidIdentifiers = { x, user_name, user-name, _temp, item123,
                     HashMap, is_active, MAX_SIZE, π, 名前 }
InvalidIdentifiers = { 123abc, -user, let, function, 42 }
```

```x
let user-name = "Alice"     // 合法：kebab-case
let user_name = "Bob"       // 合法：snake_case
let _temp = 42              // 合法：下划线开头
let π = 3.14159             // 合法：Unicode 字母
```

---

### 1.2.3 字面量

#### 整数字面量

X 的整数类型为 `Integer`（任意精度）。

```
IntegerLiteral → DecimalLiteral
               | HexLiteral
               | OctalLiteral
               | BinaryLiteral

DecimalLiteral → '0'
               | NonZeroDigit ( '_'? Digit )*

HexLiteral → '0' ( 'x' | 'X' ) HexDigit ( '_'? HexDigit )*
OctalLiteral → '0' ( 'o' | 'O' ) OctalDigit ( '_'? OctalDigit )*
BinaryLiteral → '0' ( 'b' | 'B' ) BinaryDigit ( '_'? BinaryDigit )*

NonZeroDigit → '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
Digit → '0' | NonZeroDigit
HexDigit → Digit | 'a'..'f' | 'A'..'F'
OctalDigit → '0'..'7'
BinaryDigit → '0' | '1'
```

下划线 `_` 可用作数字分隔符，提高可读性，不影响数值：

```x
let million = 1_000_000          // 十进制
let hex_color = 0xFF_AA_33      // 十六进制
let permissions = 0o755          // 八进制
let flags = 0b1010_1100          // 二进制
```

#### 浮点数字面量

X 的浮点类型为 `Float`（双精度 IEEE 754）。

```
FloatLiteral → Digit+ '.' Digit* ExponentPart?
             | '.' Digit+ ExponentPart?
             | Digit+ ExponentPart

ExponentPart → ( 'e' | 'E' ) ( '+' | '-' )? Digit+
```

```x
let pi = 3.14159                // 小数
let avogadro = 6.02e23          // 科学计数法
let tiny = 1.6e-19              // 负指数
let also_float = .5             // 省略整数部分
```

#### 字符串字面量

X 的字符串类型为 `String`（UTF-8 编码）。

```
StringLiteral → SimpleString | MultiLineString

SimpleString → '"' SimpleStringChar* '"'
SimpleStringChar → SourceChar \ { '"', '\\', '{', Newline }
                 | EscapeSequence
                 | Interpolation

MultiLineString → '"""' MultiLineStringChar* '"""'
MultiLineStringChar → SourceChar \ { '"""', '\\', '{' }
                    | EscapeSequence
                    | Interpolation
                    | Newline

EscapeSequence → '\\' EscapeChar
EscapeChar → 'n'   // 换行 U+000A
           | 't'   // 制表 U+0009
           | 'r'   // 回车 U+000D
           | '0'   // 空字符 U+0000
           | '"'   // 双引号
           | '\''  // 单引号
           | '\\'  // 反斜杠
           | '{'   // 左花括号（转义插值）
           | 'u' '{' HexDigit{1,6} '}'  // Unicode 转义

Interpolation → '{' Expression '}'
```

字符串插值使用 `{expr}` 语法，花括号内可以是任意表达式：

```x
let name = "World"
let greeting = "Hello, {name}!"              // 简单插值
let math = "1 + 1 = {1 + 1}"                // 表达式插值
let escaped = "Use \{braces\} literally"     // 转义花括号

let multiline = """
    SELECT *
    FROM users
    WHERE id = {user_id}
    ORDER BY name
    """
```

多行字符串的缩进规则：以结束 `"""` 的列位置为基准，移除各行的公共前缀缩进。

#### 布尔字面量

X 的布尔类型为 `Boolean`。

```
BooleanLiteral → 'true' | 'false'
```

```x
let is_active = true
let has_error = false
```

---

### 1.2.4 运算符与标点符号

#### 运算符

```
Operator → ArithmeticOp | ComparisonOp | LogicalOp
         | AssignmentOp | PipeOp | ArrowOp | RangeOp
         | ErrorOp | MemberOp

ArithmeticOp → '+' | '-' | '*' | '/' | '%' | '^'
ComparisonOp → '==' | '!=' | '<' | '>' | '<=' | '>='
LogicalOp → 'and' | 'or' | 'not'
AssignmentOp → '=' | '+=' | '-=' | '*=' | '/=' | '%=' | '^='
PipeOp → '|>'
ArrowOp → '->' | '=>'
RangeOp → '..' | '..='
ErrorOp → '?' | '?.' | '??'
MemberOp → '.' | '::'
```

##### 运算符语义

| 运算符 | 名称 | 语义 | 示例 |
|--------|------|------|------|
| `+` `-` `*` `/` `%` | 算术运算 | 加减乘除取余 | `a + b` |
| `^` | 幂运算 | 求幂 | `x ^ 2` |
| `==` `!=` `<` `>` `<=` `>=` | 比较运算 | 值比较 | `a >= b` |
| `and` `or` `not` | 逻辑运算 | 短路求值的逻辑运算 | `a and b` |
| `=` | 赋值 | 绑定/赋值 | `let x = 42` |
| `+=` `-=` `*=` `/=` `%=` `^=` | 复合赋值 | 运算并赋值 | `x += 1` |
| `\|>` | 管道 | 将左侧值传入右侧函数 | `data \|> process` |
| `->` | 函数箭头 | 函数类型/Lambda 体 | `(x) -> x + 1` |
| `=>` | 匹配箭头 | 模式匹配分支体 | `Some(x) => x` |
| `..` | 半开区间 | 不含上界的范围 | `0..10` |
| `..=` | 闭合区间 | 含上界的范围 | `0..=10` |
| `?` | 错误传播 | 自动传播 `Result`/`Option` 的错误 | `value?` |
| `?.` | 可选链 | 安全访问 `Option` 内部成员 | `user?.name` |
| `??` | 默认值 | `Option` 为 `None` 时提供默认值 | `x ?? 0` |
| `.` | 成员访问 | 访问对象成员 | `user.name` |
| `::` | 路径分隔 | 命名空间路径 | `std::io` |

##### 运算符优先级（从高到低）

```
Precedence（优先级，从高到低）：
  1. 后缀：  . :: () [] ? ?. 
  2. 前缀：  not  -（取负）
  3. 幂：    ^                          （右结合）
  4. 乘除：  * / %                      （左结合）
  5. 加减：  + -                        （左结合）
  6. 范围：  .. ..=
  7. 比较：  == != < > <= >=
  8. 逻辑与：and                        （左结合）
  9. 逻辑或：or                         （左结合）
  10. 默认值：??                        （左结合）
  11. 管道：  |>                        （左结合）
  12. 箭头：  -> =>                     （右结合）
  13. 赋值：  = += -= *= /= %= ^=      （右结合）
```

#### 标点符号

```
Punctuator → '(' | ')' | '{' | '}' | '[' | ']'
           | ',' | ';' | ':' | '|' | '&' | '~' | '@' | '#'
```

| 标点 | 用途 |
|------|------|
| `(` `)` | 分组、函数参数、元组 |
| `{` `}` | 代码块、记录字面量、字符串插值 |
| `[` `]` | 列表字面量、索引访问 |
| `,` | 参数/元素分隔符 |
| `;` | 语句分隔符（通常可选，换行隐式分隔） |
| `:` | 类型注解、字典键值对 |
| `\|` | ADT 变体分隔、Lambda 参数 |
| `@` | 装饰器/注解 |
| `#` | 编译器指令 |

---

## 1.3 注释

注释是被词法分析器消耗但不产生记号的文本，用于代码文档化。

```
Comment → LineComment | BlockComment
```

### 1.3.1 单行注释

```
LineComment → '//' LineCommentChar* Newline?
LineCommentChar → SourceChar \ { CR, LF, NEL, LS, PS }
```

使用 `//` 开头，直到行尾：

```x
// 这是一个单行注释
let x = 42  // 行尾注释
```

### 1.3.2 多行注释

```
BlockComment → '/**' BlockCommentContent '*/'
BlockCommentContent → BlockCommentChar*
BlockCommentChar → SourceChar
                 | BlockComment    // 支持嵌套
```

使用 `/**` 开头，`*/` 结尾，可以跨行，**支持嵌套**：

```x
/**
 * 这是多行注释
 * 可以有多行
 */
function main() {
    /**
     * 外层注释
     * /** 嵌套注释也是合法的 */
     */
    let x = 42
}
```

嵌套注释的匹配规则：词法分析器维护一个嵌套深度计数器 `d`，遇到 `/**` 时 `d += 1`，遇到 `*/` 时 `d -= 1`，当 `d = 0` 时注释结束。

```
δ_comment(d, "/**") = d + 1
δ_comment(d, "*/")  = d - 1
注释结束条件：d = 0
```

### 1.3.3 文档注释

文档注释是以 `///` 开头的单行注释，或以 `/***` 开头的多行注释，用于生成 API 文档：

```x
/// 计算两个整数的和。
///
/// # 参数
/// - `a`: 第一个加数
/// - `b`: 第二个加数
///
/// # 返回值
/// 两个参数之和
function add(a: Integer, b: Integer) -> Integer {
    a + b
}
```

---

## 1.4 空白字符

### 定义

空白字符用于分隔记号，自身不产生记号。

```
Whitespace → WhitespaceChar+
WhitespaceChar → ' '      // U+0020 空格
               | '\t'     // U+0009 水平制表符
               | '\r'     // U+000D 回车
               | '\n'     // U+000A 换行
               | '\x0B'   // U+000B 垂直制表符
               | '\x0C'   // U+000C 换页符
```

### 处理规则

空白字符和注释在词法分析阶段被消耗，不出现在记号流中：

```
RawStream → ( Whitespace | Comment | Token )*
TokenStream = { t ∈ Token | t ∈ RawStream ∧ t ∉ Whitespace ∧ t ∉ Comment }
```

### 换行作为语句终结符

X 语言使用换行作为隐式语句分隔符。以下规则决定何时换行被视为语句终结：

```
令 t 为换行前最后一个记号，换行被视为语句终结当且仅当：
  t ∈ { Identifier, Literal, ')', ']', '}', '?', 'return', 'break', 'continue' }
```

显式分号 `;` 始终可以用作语句分隔符，无论是否有换行。

---

## 1.5 词法分析器状态机

### 状态定义

词法分析器是一个有限状态自动机（DFA）：

```
State = { Normal, String, MultiLineString, StringInterp,
          LineComment, BlockComment, DocComment }

初始状态：Normal
```

### 状态转换函数

```
δ : State × SourceChar → State × Action

δ(Normal, '/')  = if peek(1) = '/' then
                     if peek(2) = '/' then (DocComment, Start)
                     else (LineComment, Start)
                   else if peek(1) = '*' and peek(2) = '*' then
                     (BlockComment, Start(depth=1))
                   else
                     (Normal, Emit(Operator))

δ(Normal, '"')  = if peek(1) = '"' and peek(2) = '"' then
                     (MultiLineString, Start)
                   else
                     (String, Start)

δ(Normal, c)    = if c ∈ IdentifierStart then (Normal, ScanIdentifier)
                   else if c ∈ Digit then (Normal, ScanNumber)
                   else if c ∈ OperatorChar then (Normal, ScanOperator)
                   else if c ∈ Punctuator then (Normal, Emit(Punctuator))
                   else if c ∈ WhitespaceChar then (Normal, Skip)
                   else Error("unexpected character", c)

δ(String, '"')  = (Normal, Emit(StringLiteral))
δ(String, '{')  = (StringInterp, PushState)
δ(String, '\\') = (String, ScanEscape)
δ(String, c)    = (String, Accumulate(c))

δ(StringInterp, '}') = (String, PopState)

δ(MultiLineString, '"' '"' '"') = (Normal, Emit(StringLiteral))
δ(MultiLineString, '{')         = (StringInterp, PushState)
δ(MultiLineString, '\\')        = (MultiLineString, ScanEscape)
δ(MultiLineString, c)           = (MultiLineString, Accumulate(c))

δ(LineComment, Newline) = (Normal, Skip)
δ(LineComment, c)       = (LineComment, Skip)

δ(BlockComment, '*' '/')  = if depth = 1 then (Normal, Skip)
                              else (BlockComment, depth - 1)
δ(BlockComment, '/' '*' '*') = (BlockComment, depth + 1)
δ(BlockComment, c)          = (BlockComment, Skip)
```

### 标识符与关键字区分

词法分析器扫描标识符后，查表确定是否为关键字：

```
Classify(s) = if s ∈ KeywordSet then Keyword(s)
              else Identifier(s)

KeywordSet = { "let", "mutable", "function", "async", "class", "trait",
               "type", "module", "const", "if", "else", "for", "in",
               "while", "return", "match", "break", "continue", "needs",
               "given", "await", "together", "race", "atomic", "retry",
               "true", "false", "None", "Some", "Ok", "Err",
               "public", "private", "protected", "internal", "static",
               "abstract", "final", "override", "virtual",
               "import", "export", "with", "where", "and", "or", "not",
               "is", "as", "weak", "implement", "extends",
               "new", "this", "super" }
```

### 数字字面量扫描

数字扫描从第一个数字字符开始：

```
ScanNumber(c):
  if c = '0' then
    match peek(1):
      'x' | 'X' → ScanHex
      'o' | 'O' → ScanOctal
      'b' | 'B' → ScanBinary
      '.'       → ScanFloat
      _         → Emit(IntegerLiteral(0))
  else
    ScanDecimal(c)

ScanDecimal(c):
  accumulate digits and '_'
  if peek = '.' and peek(1) ∈ Digit then ScanFloat
  else if peek ∈ { 'e', 'E' } then ScanExponent
  else Emit(IntegerLiteral)
```

---

**本章定义了 X 语言的完整词法结构。所有关键字使用英文全称，构造器使用大写字母开头（`None`、`Some`、`Ok`、`Err`），体现 X 语言"可读性第一"的设计哲学。**
