# 第8章 模块系统

## 8.1 模块声明

X 的模块系统使用 `module` 关键字声明模块，以点号分隔的路径表示层级关系。每个源文件隐式构成一个模块，也可通过显式声明指定模块名。

### 语法

```
ModuleDeclaration ::= 'module' ModulePath ';'
                    | 'module' ModulePath '{' ModuleItem* '}'

ModulePath ::= Identifier ('.' Identifier)*

ModuleItem ::= Declaration
             | ImportDeclaration
             | ExportDeclaration
             | SubModuleDeclaration

SubModuleDeclaration ::= ModuleDeclaration
```

### 模块声明示例

```x
module myapp.utils

public function formatDate(date: Date) -> String {
    // ...
}

public function sanitize(input: String) -> String {
    // ...
}
```

嵌套模块：

```x
module myapp {
    module models {
        public class User {
            let name: String
            let email: String
        }
    }

    module services {
        import myapp.models.User

        public function findUser(id: Integer) -> Option<User> {
            // ...
        }
    }
}
```

### 模块规则

```
每个源文件隐式定义一个模块：
  file "src/utils/format.x"  ⟹  module myapp.utils.format

显式声明优先于文件路径推断：
  若 file 中含 module M，则该文件的模块名为 M
```

---

## 8.2 导入与导出

### Import 语法

```
ImportDeclaration ::= 'import' ImportPath ';'
                    | 'import' ImportPath 'as' Identifier ';'
                    | 'import' ImportPath '.{' ImportItem (',' ImportItem)* '}' ';'
                    | 'import' ImportPath '.*' ';'

ImportPath ::= Identifier ('.' Identifier)*

ImportItem ::= Identifier
             | Identifier 'as' Identifier
```

导入有四种形式：

| 形式 | 语法 | 说明 |
|------|------|------|
| 单一导入 | `import std.collections.HashMap` | 导入单个符号 |
| 选择导入 | `import std.collections.{HashMap, TreeMap}` | 导入多个符号 |
| 通配导入 | `import std.collections.*` | 导入模块全部公开符号 |
| 重命名导入 | `import std.collections.HashMap as Map` | 导入并重命名 |

```x
import std.io.File
import std.collections.{HashMap, LinkedList}
import json.Parser as JsonParser
```

### Export 语法

```
ExportDeclaration ::= 'export' Declaration
                    | 'export' '{' ExportItem (',' ExportItem)* '}' ';'
                    | 'export' '*' 'from' ImportPath ';'

ExportItem ::= Identifier
             | Identifier 'as' Identifier
```

导出有三种形式：

| 形式 | 语法 | 说明 |
|------|------|------|
| 内联导出 | `export function f() -> Integer { ... }` | 声明时即导出 |
| 批量导出 | `export { formatDate, sanitize }` | 导出已声明的符号 |
| 重新导出 | `export * from myapp.utils` | 转发其他模块的导出 |

```x
export function greet(name: String) -> String {
    "Hello, ${name}!"
}

export { formatDate, sanitize }
```

### 导入导出类型规则

```
Module M exports symbol x : T
──────────────────────────────
⊢ import M.x : T

Module M contains N as submodule
────────────────────────────────
⊢ M.N is a valid module path

Module M exports x    Module N re-exports x from M
────────────────────────────────────────────────────
⊢ import N.x : T
```

---

## 8.3 模块路径与解析

### 模块路径语法

```
ModulePathRef ::= AbsolutePath | RelativePath

AbsolutePath ::= Identifier ('.' Identifier)*

RelativePath ::= 'self' ('.' Identifier)*
               | 'super' ('.' Identifier)*
```

**路径解析策略：**

1. **绝对路径**：从包根开始解析，例如 `std.collections.HashMap`
2. **self 路径**：从当前模块开始，例如 `self.helpers.format`
3. **super 路径**：从父模块开始，例如 `super.models.User`

### 模块解析规则

```
resolve(Package, AbsolutePath) = Module
  where Package is the current package or a dependency
        AbsolutePath is walked from the package root

resolve(Current, self.Path) = resolve(Current, Path)

resolve(Current, super.Path) = resolve(parent(Current), Path)

lookup(Module, Name) = Declaration
  where Declaration is exported by Module
        and visibility permits access from the call site
```

### 文件系统映射

模块路径与文件系统目录结构对应：

```
myapp/
├── x.toml
└── src/
    ├── main.x              → module myapp (entry point)
    ├── models/
    │   ├── user.x          → module myapp.models.user
    │   └── post.x          → module myapp.models.post
    └── services/
        └── auth.x          → module myapp.services.auth
```

---

## 8.4 包管理

X 的包管理系统对标 Rust 的 Cargo，使用 `x.toml` 作为项目清单文件，`x.lock` 作为锁文件。

### x.toml 格式

```toml
[package]
name = "myapp"
version = "1.0.0"
edition = "2026"
authors = ["Author Name <author@example.com>"]
description = "A sample X application"
license = "MIT"

[dependencies]
std = "^2.0"
json = "1.0.0"
http = { version = "0.5", features = ["tls"] }

[dev-dependencies]
testing = "^1.0"

[build]
target = "native"
```

### 版本约束

| 语法 | 含义 | 示例 |
|------|------|------|
| `^1.2.3` | 兼容版本（SemVer major 不变） | `>=1.2.3, <2.0.0` |
| `~1.2.3` | 补丁版本（minor 不变） | `>=1.2.3, <1.3.0` |
| `=1.2.3` | 精确版本 | 仅 `1.2.3` |
| `>=1.0, <2.0` | 范围约束 | 自定义范围 |

### x.lock 文件

`x.lock` 由工具链自动生成和维护，记录依赖树中每个包的精确版本，保证构建可重现。该文件应纳入版本控制。

### 工作空间（Workspace）

多包项目可用工作空间统一管理：

```toml
[workspace]
members = [
    "core",
    "cli",
    "stdlib",
]
```

### 工具链命令

| 命令 | 功能 |
|------|------|
| `x new myapp` | 创建新项目，生成 `x.toml` 与目录结构 |
| `x build` | 构建项目 |
| `x run` | 构建并运行 |
| `x test` | 运行测试 |
| `x add json` | 添加依赖到 `x.toml` |
| `x publish` | 打包并发布到已配置的本地文件型注册表；远程仓库发布仍在建设中 |

---

## 8.5 可见性控制

X 使用完整英文单词作为可见性修饰符：`public`、`private`、`internal`。

### 可见性修饰符语法

```
Visibility ::= 'public' | 'private' | 'internal'

VisibleDeclaration ::= Visibility? Declaration
```

默认可见性为 `private`（模块内可见）。

### 可见性范围

| 修饰符 | 同模块 | 同包（其他模块） | 外部包 |
|--------|--------|-----------------|--------|
| `public` | ✓ | ✓ | ✓ |
| `internal` | ✓ | ✓ | ✗ |
| `private` | ✓ | ✗ | ✗ |

### 可见性形式化规则

```
Declaration D in module M has visibility V
Access from location L is permitted iff:
  V = public
  ∨ (V = internal ∧ same_package(M, L))
  ∨ (V = private  ∧ same_module(M, L))
```

### 可见性示例

```x
module myapp.services

import myapp.models.User

public function getUser(id: Integer) -> Option<User> {
    let user = queryDatabase(id)
    user
}

internal function queryDatabase(id: Integer) -> Option<User> {
    // 仅包内可见
}

private function buildQuery(id: Integer) -> String {
    // 仅本模块可见
    "SELECT * FROM users WHERE id = ${id}"
}
```

### 可见性与导出的关系

- `export` 控制模块的**对外接口**——哪些符号可被导入
- `public`/`internal`/`private` 控制符号的**访问权限**——哪些代码可引用该符号
- 只有 `public` 或 `internal` 的符号才可被 `export`；`private` 符号不可导出

```
export(D) requires visibility(D) ∈ {public, internal}
```

---

**本章定义了 X 语言的模块系统，包括模块声明、导入导出、路径解析、包管理（x.toml/x.lock）以及可见性控制。模块系统的设计对标 Rust/Cargo 的成熟实践，使用完整英文关键字以保持可读性。**
