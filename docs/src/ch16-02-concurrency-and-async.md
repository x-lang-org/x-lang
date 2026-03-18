# 并发与异步

在现代软件开发中，并发和异步编程变得越来越重要。X语言通过其强大的效果系统和结构化并发原语，为开发者提供了一种安全、高效的方式来处理并发和异步操作。本章将介绍X语言中的Goroutine、结构化并发、Async/Await和并发安全等特性。

## Goroutine：轻量级线程

Goroutine是X语言中实现并发的基本单位，它是一种轻量级的线程实现，由X语言运行时管理。与操作系统线程相比，Goroutine的创建和切换成本更低，使得开发者可以创建大量的Goroutine而不会导致系统资源耗尽。

### 创建Goroutine

使用`go`关键字可以创建一个新的Goroutine：

```x
function main() {
    // 创建一个新的Goroutine
    go function() {
        for i in 1..5 {
            println("Goroutine: {i}")
            time::sleep(100.ms)
        }
    }
    
    // 主线程继续执行
    for i in 1..3 {
        println("Main: {i}")
        time::sleep(150.ms)
    }
}
```

### Goroutine的生命周期

- **创建**：使用`go`关键字创建新的Goroutine
- **执行**：Goroutine在独立的执行上下文中运行
- **调度**：由X语言运行时调度器管理，实现M:N调度模型
- **终止**：当Goroutine执行完毕或发生panic时终止

### 等待Goroutine完成

使用`join`函数可以等待Goroutine完成：

```x
import std::thread

function main() {
    let handle = thread::spawn(function() {
        for i in 1..10 {
            println("Goroutine: {i}")
            thread::sleep(100.ms)
        }
    })
    
    // 等待Goroutine完成
    handle.join().unwrap()
    
    println("All done!")
}
```

## 结构化并发

X语言提供了丰富的结构化并发原语，使得并发编程更加安全和可预测。这些原语包括`together`、`race`和`timeout`等。

### Together：并行执行多个任务

`together`表达式允许并行执行多个异步任务，并等待所有任务完成：

```x
async function fetchUser(id: Integer) -> User with IO, Throws<NetworkError> {
    let response = await http.get("/api/users/{id}")?
    parseUser(response.body)?
}

async function fetchPosts(userId: Integer) -> List<Post> with IO, Throws<NetworkError> {
    let response = await http.get("/api/users/{userId}/posts")?
    parsePosts(response.body)?
}

async function loadProfile(id: Integer) -> Profile with IO, Throws<NetworkError> {
    let (user, posts) = await together {
        fetchUser(id),
        fetchPosts(id)
    }
    Profile { user, posts }
}
```

### Race：竞争执行多个任务

`race`表达式允许并行执行多个异步任务，并等待第一个完成的任务，同时取消其余任务：

```x
async function fetchFastest(urls: List<String>) -> Response with IO, Throws<NetworkError> {
    await race {
        http.get(urls[0]),
        http.get(urls[1]),
        http.get(urls[2])
    }
}
```

### Timeout：设置任务超时

`timeout`表达式允许为异步任务设置超时时间：

```x
async function fetchWithTimeout(url: String) -> Option<Response> with IO {
    timeout (await http.get(url)) after 5.seconds
}
```

## Async/Await

X语言的`Async`效果和`async/await`语法提供了一种优雅的方式来处理异步操作。

### 异步函数声明

使用`async function`声明异步函数：

```x
async function fetchData(url: String) -> String with IO, Throws<NetworkError> {
    let response = await http.get(url)?
    response.body
}
```

### Await表达式

使用`await`关键字等待异步操作完成：

```x
async function processData() -> String with IO, Throws<NetworkError> {
    let data = await fetchData("https://api.example.com/data")
    process(data)
}
```

### 异步函数的类型

异步函数的类型包含`Async`效果：

```x
// 类型为: (String) -> String with Async, IO, Throws<NetworkError>
async function fetchData(url: String) -> String with IO, Throws<NetworkError> {
    // 实现...
}
```

## 并发安全

X语言提供了多种机制来确保并发安全，包括软件事务内存（STM）、原子操作和锁等。

### 软件事务内存（STM）

使用`atomic`块实现软件事务内存，确保状态操作的原子性：

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
```

### 原子操作

X语言提供了原子操作来处理基本类型的并发访问：

```x
import std::sync::atomic

function counter() {
    let count = atomic::AtomicInteger::new(0)
    
    for i in 1..100 {
        go function() {
            count.fetch_add(1, atomic::Ordering::SeqCst)
        }
    }
    
    // 等待所有Goroutine完成
    thread::sleep(100.ms)
    println("Count: {count.load(atomic::Ordering::SeqCst)}")
}
```

### 锁和互斥量

使用锁来保护共享资源：

```x
import std::sync::Mutex

function safeCounter() {
    let mutex = Mutex::new(0)
    
    for i in 1..100 {
        go function() {
            let mut guard = mutex.lock().unwrap()
            *guard = *guard + 1
        }
    }
    
    // 等待所有Goroutine完成
    thread::sleep(100.ms)
    let guard = mutex.lock().unwrap()
    println("Count: {*guard}")
}
```

## 总结

X语言通过其强大的效果系统和结构化并发原语，为开发者提供了一种安全、高效的并发和异步编程模型。主要特性包括：

- **Goroutine**：轻量级线程，创建和切换成本低
- **结构化并发**：通过`together`、`race`和`timeout`等原语实现安全的并发操作
- **Async/Await**：优雅的异步编程语法，基于效果系统
- **并发安全**：通过软件事务内存、原子操作和锁等机制确保并发安全

这些特性使得X语言在处理并发和异步任务时更加安全、可靠，同时保持了代码的清晰性和可维护性。
