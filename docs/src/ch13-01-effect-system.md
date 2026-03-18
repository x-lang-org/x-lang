# 第13章 效果系统

效果系统是 X 语言的核心特性之一，它使函数的副作用在类型签名中显式可见，确保所有副作用都被正确追踪和处理。通过效果系统，X 语言实现了类型安全的副作用管理，同时保持了代码的清晰性和可维护性。

## 13.1 效果声明

效果声明是效果系统的基础，它定义了函数可能产生的副作用类型。在 X 语言中，效果通过函数签名中的 `with` 关键字进行声明。

### 内置效果

X 语言提供了以下内置效果：

| 效果 | 含义 | 语义 |
|------|------|------|
| `IO` | 输入输出操作 | 文件系统、网络、控制台交互 |
| `Async` | 异步执行 | 函数可能挂起并稍后恢复 |
| `State<S>` | 可变状态 | 读写类型为 `S` 的状态 |
| `Throws<E>` | 可能失败 | 返回 `Result<T, E>`，用 `?` 传播错误 |
| `NonDet` | 非确定性 | 可能产生多个结果 |

### 函数效果注解

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

### 用户自定义效果

除了内置效果外，X 语言还允许用户定义自己的效果，以捕获特定领域的副作用。

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

### 效果类型规则

效果系统遵循以下类型规则：

1. **函数调用传播效果**：调用具有效果的函数会将这些效果传播到调用点。
2. **效果集合的并**：多个效果的组合会形成一个效果集合。
3. **效果子类型**：效果集越小的函数越纯，可以替代效果集更大的函数。

## 13.2 效果处理

效果处理是 X 语言中处理副作用的核心机制，它允许开发者拦截、转换和处理效果。通过效果处理器，开发者可以为效果提供具体实现，从而消除函数的效果依赖。

### 效果处理器语法

效果处理器使用 `handle` 表达式来定义：

```x
handle {
    // 可能产生效果的代码块
} with {
    // 效果处理规则
}
```

### 效果处理示例

#### 基本效果处理

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

#### 用处理器实现纯测试

效果处理器的一个重要应用是实现纯测试，通过模拟依赖来避免实际的副作用：

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

#### 效果多态

函数可以对效果进行参数化，实现效果多态：

```x
function map<A, B, E>(list: List<A>, f: (A) -> B with E) -> List<B> with E {
    match list {
        [] => []
        [head, ...tail] => [f(head), ...map(tail, f)]
    }
}
```

### 效果处理的类型规则

效果处理遵循以下类型规则：

1. **效果消除**：处理效果后，被处理的效果会从函数的效果集合中移除。
2. **类型兼容性**：效果处理器必须为效果中的所有操作提供实现。

## 13.3 依赖注入

X 语言的依赖注入机制是通过效果系统实现的，使用 `needs` 和 `given` 关键字来声明和提供依赖。

### 需求声明（needs）

在函数签名中使用 `needs` 关键字声明函数所需的依赖：

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
```

### 给定依赖（given）

在调用函数时，使用 `given` 块提供依赖的具体实现：

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

### 依赖作用域

`given` 块内提供的依赖对所有嵌套调用可见。例如，`deleteUser` 内部调用 `getUser` 时，会自动继承外层的 `Database` 和 `Logger` 实例：

```x
function deleteUser(id: Integer) -> () with Throws<NotFound>
    needs Database, Logger {
    let user = getUser(id)?
    Database.execute("DELETE FROM users WHERE id = {id}")?
    Logger.info("Deleted user {user.name}")
}
```

### 依赖注入的类型规则

依赖注入遵循以下类型规则：

1. **依赖传播**：函数声明的依赖会成为其效果集合的一部分。
2. **依赖消除**：提供依赖后，依赖会从函数的效果集合中移除。
3. **类型匹配**：提供的依赖必须与声明的依赖类型匹配。

## 13.4 最佳实践

### 效果使用建议

1. **保持函数纯性**：尽量编写纯函数，只在必要时使用效果。
2. **效果最小化**：函数应只声明其实际需要的效果，避免过度声明。
3. **效果组合**：使用效果多态来编写可以处理多种效果的通用函数。

### 依赖注入建议

1. **依赖抽象**：通过接口（trait）声明依赖，而不是具体实现。
2. **依赖隔离**：每个函数只声明其直接需要的依赖。
3. **测试友好**：使用效果处理器来模拟依赖，实现纯测试。

### 错误处理建议

1. **使用 Throws 效果**：对于可能失败的操作，使用 `Throws<E>` 效果而不是异常。
2. **明确错误类型**：为不同类型的错误定义明确的错误类型。
3. **错误传播**：使用 `?` 运算符来简洁地传播错误。

## 13.5 总结

X 语言的效果系统是一个强大的工具，它通过以下方式提升代码质量：

1. **副作用显式化**：所有副作用在类型签名中明确可见，提高代码的可读性和可维护性。
2. **类型安全**：效果系统在编译时确保所有副作用都被正确处理。
3. **依赖管理**：通过 `needs` 和 `given` 实现类型安全的依赖注入。
4. **测试友好**：通过效果处理器可以轻松模拟依赖，实现纯测试。
5. **无异常设计**：使用 `Result<T, E>` 和 `?` 运算符替代传统的异常机制，使错误处理更加显式和可控。

效果系统是 X 语言的核心特性之一，它为开发者提供了一种优雅的方式来管理副作用，同时保持代码的清晰性和类型安全性。通过合理使用效果系统，开发者可以编写更加可靠、可测试和可维护的代码。
