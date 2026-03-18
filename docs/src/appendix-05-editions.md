# 附录 E - 版本说明

X 语言使用 editions（版本）系统来管理语言随时间的演变。版本允许 X 语言以向后兼容的方式发展，同时仍然引入新功能和改进。

在本附录中，我们将讨论 X 语言的 editions 系统、它们是什么、它们如何工作，以及每个 edition 中包含什么。

## 什么是 Editions？

edition 是 X 语言语言的一组功能的版本。新的 edition 可以引入不兼容的更改，例如：

- 新的语法
- 新的关键字
- 新的警告
- 默认行为的更改

但是，edition 与版本号不同。一个编译器可以支持多个 edition，你可以为每个包选择使用哪个 edition。

### 为什么使用 Editions？

Editions 很重要有几个原因：

1. **演进** - 允许语言演进而不破坏现有代码
2. **采用** - 允许项目可以按照自己的节奏采用新功能
3. **兼容性** - 旧代码继续工作而无需修改
4. **清晰度** - 清楚哪些功能属于一起

### 指定 Edition

你在包的 `x.toml` 中指定 edition：

```toml
[package]
name = "my_package"
version = "0.1.0"
edition = "2024"
```

## Edition 列表

让我们看看假设的 X 语言 editions。

### 2024 Edition

2024 edition 是 X 语言语言的第一个稳定 edition。

#### 包含的内容

- 基本语法和类型
- 所有权和借用
- 结构体和枚举
- 模式匹配
- 错误处理（Option 和 Result）
- 泛型和 traits
- 迭代器和闭包
- 异步/等待
- 等等。

#### 示例

```x
// 2024 edition 代码
function main() {
    let x = 5;
    println("x 是 {}", x);
}
```

### 未来 Editions

未来的 editions 可能包括：

- 新语法糖
- 改进的类型推断
- 新的标准库功能
- 性能改进
- 等等。

## 过渡指南

当新 edition 发布时，你可以按照自己的节奏过渡你的项目。

### 检查兼容性

大多数时候，编译器可以帮助你检查与新 edition 的兼容性：

```bash
# 检查你的代码与新 edition 的兼容性
x fix --edition 2027
```

### 自动修复

许多更改可以自动应用：

```bash
# 自动修复代码以使用新 edition
x fix --edition 2027
```

### 预览功能

在新 edition 发布之前，你可以使用预览标志预览新功能：

```toml
[package]
name = "my_package"
edition = "2024"

[features]
preview = ["async_fn_traits", "let_chains"]
```

然后在你的代码中：

```x
// 这启用预览功能
#![feature(async_fn_traits)]

async function example() {
    // ...
}
```

## 最佳实践

关于 editions 的一些最佳实践：

1. **使用最新 edition** - 对于新项目，使用最新的稳定 edition
2. **逐渐过渡** - 对于现有项目，当你准备好时过渡
3. **使用预览功能** - 小心使用，用于测试新功能
4. **阅读发布说明** - 每个 edition 都有包含更改的发布说明

## 编译器支持

X 语言编译器支持多个 editions。你可以使用 `--edition` 标志为单个文件指定 edition：

```bash
# 使用特定 edition 编译
x compile --edition 2024 file.x
```

## 总结

X 语言的 editions 系统：

- 允许语言以向后兼容的方式演进
- 你可以为每个包选择 edition
- 新 editions 可能引入不兼容的更改
- 你可以使用预览功能尝试新功能
- 过渡工具可以帮助更新代码

Editions 是保持 X 语言语言现代化同时保持与现有代码兼容性的重要部分！

