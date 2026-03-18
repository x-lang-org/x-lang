# 第7章 效果系统

## 7.1 效果定义

### 形式语法

```
Effect ::= 'IO'
         | 'Async'
         | 'State' '<' Type '>'
         | 'Throws' '<' Type '>'
         | 'NonDet'
         | Identifier                    // 用户自定义效果

EffectDeclaration ::= 'effect' Identifier TypeParameters? '{' EffectOperation* '}'

EffectOperation ::= 'function' Identifier Parameters '->' Type
```

### 说明

X 的效果系统使函数的副作用在类型签名中**显式可见**。效果不是运行时异常——它们是编译时类型信息，确保所有副作用都被正确追踪和处理。

**内置效果**：

| 效果 | 含义 | 语义 |
|------|------|------|
| `IO` | 输入输出操作 | 文件系统、网络、控制台交互 |
| `Async` | 异步执行 | 函数可能挂起并稍后恢复 |
| `State<S>` | 可变状态 | 读写类型为 `S` 的状态 |
| `Throws<E>` | 可能失败 | 返回 `Result<T, E>`，用 `?` 传播错误 |
| `NonDet` | 非确定性 | 可能产生多个结果 |

**用户自定义效果**：

```x
effect Logger {
    function log(level: String, message: String) -> ()
    function getLevel() -> String
}

effect Random {
    function nextInteger(bound: Integer) -> Integer
    function nextFloat() -> Float
}
```

> **核心设计原则**：X 语言**没有异常**。`Throws<E>` 不是传统的 throw/catch 异常机制——它是一个类型级标记，表示函数返回 `Result<T, E>`。错误通过 `?` 运算符显式传播，通过模式匹配显式处理。

---

## 7.2 效果注解

### 形式语法

```
FunctionType ::= '(' (Type (',' Type)*)? ')' '->' Type ('with' EffectList)?

EffectList ::= Effect (',' Effect)*
```

### 说明

函数签名中使用 `with` 关键字分隔返回类型和效果列表。无效果的函数是**纯函数**，不需要效果注解。

```x
// 纯函数——无副作用
function add(a: Integer, b: Integer) -> Integer = a + b

// 单效果
function readLine() -> String with IO {
    Console.readLine()
}

// 多效果
function fetchUser(id: Integer) -> User with Async, IO, Throws<NetworkError> {
    let response = await http.get("/users/{id}")?
    parseUser(response.body)?
}

// 效果推断——编译器可自动推断效果集，签名中可省略
function helper(x: Integer) {   // 编译器推断效果
    print(x)                     // 推断出 IO
}
```

### 效果类型规则

函数调用传播效果：

$$
\frac{\Gamma \;\vdash\; f : (T_1,\;\ldots,\; T_n) \to R \;\texttt{with}\; \Delta
      \qquad
      \Gamma \;\vdash\; e_i : T_i \;\;(1 \le i \le n)}
     {\Gamma \;\vdash\; f(e_1,\;\ldots,\; e_n) : R \;\texttt{with}\; \Delta}
$$

效果集合的并：

$$
\frac{\Gamma \;\vdash\; e_1 : T \;\texttt{with}\; \Delta_1 \qquad \Gamma \;\vdash\; e_2 : U \;\texttt{with}\; \Delta_2}
     {\Gamma \;\vdash\; (e_1;\; e_2) : U \;\texttt{with}\; \Delta_1 \cup \Delta_2}
$$

效果子类型（效果集越小越纯）：

$$
\frac{\Delta_1 \subseteq \Delta_2}
     {(T) \to R \;\texttt{with}\; \Delta_1 \;\;<:\;\; (T) \to R \;\texttt{with}\; \Delta_2}
$$

---

## 7.3 Throws 效果（基于 Result，无异常）

### 说明

`Throws<E>` 效果声明函数可能以错误类型 `E` 失败。**这不是异常——没有 `throw`、`try`、`catch`、`finally`**。语义上 `Throws<E>` 等价于函数返回 `Result<T, E>`，错误通过 `?` 运算符向上传播。

```x
// Throws<E> 表示返回 Result<T, E>
function parseInt(s: String) -> Integer with Throws<ParseError> {
    if s.isEmpty() {
        return Err(ParseError.EmptyInput)
    }
    // ... 解析逻辑 ...
    Ok(result)
}

// ? 运算符：失败时自动向上传播
function parseConfig(path: String) -> Config with IO, Throws<ConfigError> {
    let content = readFile(path)?          // IO + Throws<FileNotFound>
    let value = parseJson(content)?        // Throws<ParseError>
    validate(value)?                       // Throws<ValidationError>
    Config.from(value)
}

// 模式匹配处理错误
function main() with IO {
    match parseConfig("app.toml") {
        Ok(config) => startApp(config)
        Err(ConfigError.FileNotFound(path)) => {
            print("Config file not found: {path}")
            startApp(Config.default())
        }
        Err(e) => print("Config error: {e}")
    }
}
```

**便捷运算符**：

```x
let name = user?.name ?? "anonymous"   // Optional 链式访问 + 默认值
let config = loadConfig() ?? Config.default()  // Result 默认值
```

### `?` 运算符的类型规则

$$
\frac{\Gamma \;\vdash\; e : \text{Result}\langle T,\; E \rangle \;\texttt{with}\; \Delta}
     {\Gamma \;\vdash\; e\texttt{?} : T \;\texttt{with}\; \Delta \cup \{\text{Throws}\langle E \rangle\}}
$$

语义展开——`e?` 等价于：

```x
match e {
    Ok(v) => v
    Err(err) => return Err(err)
}
```

### 错误类型组合

多个 `Throws` 效果可以统一为联合错误类型：

$$
\text{Throws}\langle E_1 \rangle,\; \text{Throws}\langle E_2 \rangle \;\equiv\; \text{Throws}\langle E_1 \;|\; E_2 \rangle
$$

---

## 7.4 需求与给定（needs/given）

### 形式语法

```
NeedsClause ::= 'needs' Dependency (',' Dependency)*

Dependency ::= Type

GivenBlock ::= Expression 'given' '{' (Identifier '=' Expression)* '}'
```

### 说明

`needs`/`given` 是 X 的依赖注入机制，通过效果系统实现。`needs` 在函数签名中声明所需依赖，`given` 在调用处提供依赖的具体实现。

```x
trait Database {
    function query(sql: String) -> List<Row> with Throws<DbError>
    function execute(sql: String) -> Integer with Throws<DbError>
}

trait Logger {
    function info(message: String) -> () with IO
    function error(message: String) -> () with IO
}

function getUser(id: Integer) -> User with Throws<NotFound>
    needs Database, Logger {
    Logger.info("Fetching user {id}")
    let rows = Database.query("SELECT * FROM users WHERE id = {id}")?
    match rows.first() {
        Some(row) => User.fromRow(row)
        None => Err(NotFound { entity = "User", id })
    }
}

function deleteUser(id: Integer) -> () with Throws<NotFound>
    needs Database, Logger {
    let user = getUser(id)?
    Database.execute("DELETE FROM users WHERE id = {id}")?
    Logger.info("Deleted user {user.name}")
}
```

**给定（providing dependencies）**：

```x
function main() with IO {
    let result = getUser(42) given {
        Database = PostgresDatabase.connect("localhost:5432")
        Logger = ConsoleLogger.new()
    }

    match result {
        Ok(user) => print("Found: {user.name}")
        Err(NotFound { id, .. }) => print("User {id} not found")
    }
}
```

**`given` 的作用域传递**：`given` 块内提供的依赖对所有嵌套调用可见——`deleteUser` 内部调用 `getUser` 自动继承外层的 `Database` 和 `Logger` 实例。

### 类型规则

$$
\frac{\Gamma,\; \texttt{needs}\; D_1,\;\ldots,\; D_n \;\vdash\; e : R \;\texttt{with}\; \Delta}
     {\Gamma \;\vdash\; \texttt{function}\; f()\; \texttt{needs}\; D_1,\;\ldots,\; D_n = e \;:\;
      () \to R \;\texttt{with}\; \Delta \cup \{D_1,\;\ldots,\; D_n\}}
$$

给定消除依赖：

$$
\frac{\Gamma \;\vdash\; e : R \;\texttt{with}\; \Delta \cup \{D_i\}
      \qquad
      \Gamma \;\vdash\; v_i : D_i}
     {\Gamma \;\vdash\; e \;\texttt{given}\; \{ D_i = v_i \} : R \;\texttt{with}\; \Delta}
$$

---

## 7.5 异步效果（async/await/together/race）

### 形式语法

```
AsyncFunction ::= 'async' 'function' Identifier Parameters ('->' Type)? ('with' EffectList)? FunctionBody

AwaitExpression ::= 'await' Expression

TogetherExpression ::= 'together' '{' (Expression ',')* '}'

RaceExpression ::= 'race' '{' (Expression ',')* '}'

TimeoutExpression ::= 'timeout' Expression 'after' Duration
```

### 说明

`Async` 效果标记函数可能挂起执行。`async function` 声明异步函数，`await` 等待异步结果。

```x
async function fetchUser(id: Integer) -> User with IO, Throws<NetworkError> {
    let response = await http.get("/api/users/{id}")?
    parseUser(response.body)?
}

async function fetchPosts(userId: Integer) -> List<Post> with IO, Throws<NetworkError> {
    let response = await http.get("/api/users/{userId}/posts")?
    parsePosts(response.body)?
}
```

**结构化并发原语**：

```x
// together：并行等待所有任务完成
async function loadProfile(id: Integer) -> Profile with IO, Throws<NetworkError> {
    let (user, posts) = await together {
        fetchUser(id),
        fetchPosts(id)
    }
    Profile { user, posts }
}

// race：等待任一任务完成，取消其余
async function fetchFastest(urls: List<String>) -> Response with IO, Throws<NetworkError> {
    await race {
        http.get(urls[0]),
        http.get(urls[1]),
        http.get(urls[2])
    }
}

// timeout：设置超时
async function fetchWithTimeout(url: String) -> Option<Response> with IO {
    timeout (await http.get(url)) after 5.seconds
}
```

### 类型规则

$$
\frac{\Gamma \;\vdash\; e : T \;\texttt{with}\; \{\text{Async}\} \cup \Delta}
     {\Gamma \;\vdash\; \texttt{await}\; e : T \;\texttt{with}\; \Delta}
$$

$$
\frac{\Gamma \;\vdash\; e_i : T_i \;\texttt{with}\; \{\text{Async}\} \cup \Delta_i \quad (1 \le i \le n)}
     {\Gamma \;\vdash\; \texttt{together}\;\{e_1,\;\ldots,\; e_n\} : (T_1,\;\ldots,\; T_n) \;\texttt{with}\; \{\text{Async}\} \cup \bigcup_i \Delta_i}
$$

$$
\frac{\Gamma \;\vdash\; e_i : T \;\texttt{with}\; \{\text{Async}\} \cup \Delta_i \quad (1 \le i \le n)}
     {\Gamma \;\vdash\; \texttt{race}\;\{e_1,\;\ldots,\; e_n\} : T \;\texttt{with}\; \{\text{Async}\} \cup \bigcup_i \Delta_i}
$$

$$
\frac{\Gamma \;\vdash\; e : T \;\texttt{with}\; \{\text{Async}\} \cup \Delta
      \qquad
      \Gamma \;\vdash\; d : \text{Duration}}
     {\Gamma \;\vdash\; \texttt{timeout}\; e \;\texttt{after}\; d : \text{Option}\langle T \rangle \;\texttt{with}\; \{\text{Async}\} \cup \Delta}
$$

---

## 7.6 原子效果（atomic/retry/STM）

### 形式语法

```
AtomicBlock ::= 'atomic' Block

RetryStatement ::= 'retry'
```

### 说明

`atomic` 块实现**软件事务内存（STM）**：块内所有状态操作作为原子事务执行，要么全部成功提交，要么全部回滚。

1. **原子性**：块内的状态读写被事务保护。
2. **隔离性**：并发事务之间互不可见中间状态。
3. **`retry`**：如果条件不满足，回滚并等待相关状态变化后自动重试。
4. **无死锁**：STM 基于乐观并发控制，不使用锁，从根本上避免死锁。

```x
class BankAccount {
    let mutable balance: Float

    new(initial: Float) {
        this.balance = initial
    }
}

function transfer(from: BankAccount, to: BankAccount, amount: Float) {
    atomic {
        if from.balance < amount {
            retry
        }
        from.balance = from.balance - amount
        to.balance = to.balance + amount
    }
}

// 组合事务——atomic 块可嵌套组合
function transferAll(accounts: List<BankAccount>, target: BankAccount) {
    atomic {
        for account in accounts {
            let amount = account.balance
            account.balance = 0.0
            target.balance = target.balance + amount
        }
    }
}
```

### 类型规则

$$
\frac{\Gamma \;\vdash\; b : T \;\texttt{with}\; \{\text{State}\langle S \rangle\} \cup \Delta}
     {\Gamma \;\vdash\; \texttt{atomic}\; b : T \;\texttt{with}\; \Delta}
$$

`retry` 仅在 `atomic` 块内合法：

$$
\frac{\Gamma,\; \text{in\_atomic} \;\vdash\; \cdot}
     {\Gamma \;\vdash\; \texttt{retry} : \text{Never} \;\texttt{with}\; \{\text{State}\langle S \rangle\}}
$$

---

## 7.7 效果处理（handlers）

### 形式语法

```
HandleExpression ::= 'handle' Block 'with' '{' HandlerCase* '}'

HandlerCase ::= EffectOperation '=>' Expression
```

### 说明

效果处理器可以拦截、转换和处理效果。处理器定义了当效果操作被触发时的具体行为——类似于依赖注入，但在类型系统层面有保障。

```x
effect Ask<T> {
    function ask() -> T
}

// 使用效果
function greet() -> String with Ask<String> {
    let name = Ask.ask()
    "Hello, {name}!"
}

// 处理效果——提供具体实现
let result = handle {
    greet()
} with {
    Ask.ask() => "World"
}
// result = "Hello, World!"
```

**用处理器实现纯测试**：

```x
function testGetUser() {
    let result = handle {
        getUser(42)
    } with {
        Database.query(sql) => [Row { id = 42, name = "Alice" }]
        Logger.info(msg) => ()
    }

    assert(result == Ok(User { id = 42, name = "Alice" }))
}
```

**效果多态**——函数可以对效果进行参数化：

```x
function map<A, B, E>(list: List<A>, f: (A) -> B with E) -> List<B> with E {
    match list {
        [] => []
        [head, ...tail] => [f(head), ...map(tail, f)]
    }
}
```

### 类型规则

$$
\frac{\text{handler} : E \to T
      \qquad
      \Gamma,\; \texttt{handle}\; E \;\texttt{with}\; \text{handler} \;\vdash\; b : R \;\texttt{with}\; \Delta}
     {\Gamma \;\vdash\; \texttt{handle}\; b \;\texttt{with}\; \text{handler} : R \;\texttt{with}\; \Delta \setminus \{E\}}
$$

效果处理消除被处理的效果：

$$
\frac{\Gamma \;\vdash\; e : T \;\texttt{with}\; \{E\} \cup \Delta
      \qquad
      h : \text{Handler}(E)}
     {\Gamma \;\vdash\; \texttt{handle}\; e \;\texttt{with}\; h : T \;\texttt{with}\; \Delta}
$$

---

**本章规范定义 X 的效果系统。核心设计原则：无异常——所有错误通过 `Result<T, E>` + `?` 运算符处理；所有副作用通过 `with` 效果注解在类型签名中显式声明。数学形式化与 X 代码示例结合定义效果语义。**
