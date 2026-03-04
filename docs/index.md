---
layout: default
title: X 编程语言
---

# The X Programming Language - 目录

- [前言](foreword.md)
- [介绍](introduction.md)

## 开始使用

- [安装 X](ch01-01-installation.md)
- [Hello, World!](ch01-02-hello-world.md)
- [编写一个简单的程序](ch01-03-guessing-game.md)

## 常见编程概念

- [常见编程概念](ch02-00-common-programming-concepts.md)
  - [变量与可变性](ch02-01-variables-and-mutability.md)
  - [数据类型](ch02-02-data-types.md)
  - [函数](ch02-03-functions.md)
  - [注释](ch02-04-comments.md)
  - [控制流](ch02-05-control-flow.md)

## 理解 Perceus 内存管理

- [理解 Perceus](ch03-00-understanding-perceus.md)
  - [Perceus 基础](ch03-01-perceus-basics.md)
  - [Perceus 高级特性](ch03-02-perceus-advanced.md)

## 使用结构体组织相关数据

- [结构体与记录](ch04-01-structs.md)
  - [定义和实例化结构体](ch04-01-structs.md#定义和实例化结构体)
  - [记录类型](ch04-03-records.md)
  - [使用结构体的示例程序](ch04-01-structs.md#使用结构体的示例程序)
  - [方法语法](ch04-01-structs.md#方法语法)

## 枚举与模式匹配

- [枚举与模式匹配](ch04-02-enums-and-pattern-matching.md)
  - [定义枚举](ch04-02-enums-and-pattern-matching.md#定义枚举)
  - [when/is 控制流结构](ch04-02-enums-and-pattern-matching.md#whenis-控制流结构)
  - [简洁控制流与 if let](ch04-02-enums-and-pattern-matching.md#简洁控制流与-if-let)

## 使用包和模块管理增长的项目

- [包和 Crate](ch05-02-packages.md)
- [定义模块来控制作用域和隐私](ch05-01-modules.md)
- [引用模块树中项目的路径](ch05-03-scope-and-imports.md)
- [使用 use 关键字将路径引入作用域](ch05-03-scope-and-imports.md#使用-use-关键字将路径引入作用域)
- [将模块分离到不同的文件中](ch05-01-modules.md#将模块分离到不同的文件中)

## 常见集合

- [使用列表存储多个值](ch06-01-lists.md)
- [使用字符串存储 UTF-8 编码的文本](ch06-04-strings.md)
- [使用 Dictionary 存储键值对](ch06-02-dictionaries.md)
- [使用 Set 存储唯一值](ch06-03-sets.md)

## 错误处理

- [使用 panic! 处理不可恢复的错误](ch07-03-panic.md)
- [使用 Result 处理可恢复的错误](ch07-02-result.md)
- [使用 Option 处理可选值](ch07-01-option.md)
- [要 panic! 还是不要 panic!](ch07-00-error-handling.md#要-panic-还是不要-panic)

## 泛型、Trait 和生命周期

- [泛型数据类型](ch08-01-generic-data-types.md)
- [Trait：定义共享行为](ch08-02-traits.md)
- [使用生命周期验证引用](ch08-03-lifetimes.md)

## 类与面向对象编程

- [类与对象](ch09-01-classes-and-objects.md)
- [继承](ch09-02-inheritance.md)
- [抽象类与接口](ch09-03-abstract-classes.md)

## 函数式语言特性：闭包、迭代器与管道

- [闭包：捕获其环境的匿名函数](ch10-01-closures.md)
- [使用迭代器处理一系列项目](ch10-02-iterators.md)
- [管道操作符](ch10-03-pipeline.md)
- [改进我们的 I/O 项目](ch10-02-iterators.md#改进我们的-io-项目)
- [比较性能：循环 vs 迭代器](ch10-02-iterators.md#比较性能循环-vs-迭代器)

## 编写自动化测试

- [如何编写测试](ch11-01-how-to-write-tests.md)
- [控制测试如何运行](ch11-01-how-to-write-tests.md#控制测试如何运行)
- [测试组织](ch11-02-test-organization.md)

## 标准库概览

- [标准库概览](ch12-01-stdlib-overview.md)
- [Prelude](ch12-02-prelude.md)
- [常用模块](ch12-03-common-modules.md)

## I/O 项目：构建命令行程序

- [接受命令行参数](ch12-04-io-project.md)
- [读取文件](ch12-04-io-project.md#读取文件)
- [重构以改进模块化和错误处理](ch12-04-io-project.md#重构以改进模块化和错误处理)
- [使用测试驱动开发开发库功能](ch12-04-io-project.md#使用测试驱动开发开发库功能)
- [处理环境变量](ch12-04-io-project.md#处理环境变量)
- [将错误消息写入标准错误而不是标准输出](ch12-04-io-project.md#将错误消息写入标准错误而不是标准输出)

## 高级特性：效果系统与异步

- [效果系统](ch13-01-effect-system.md)
- [异步编程](ch13-02-async.md)
- [元编程](ch13-03-metaprogramming.md)

## 关于 X 工具链的更多内容

- [使用发布配置文件自定义构建](ch14-01-more-cargo.md)
- [将包发布到注册表](ch14-01-more-cargo.md#将包发布到注册表)
- [X 工作空间](ch14-01-more-cargo.md#x-工作空间)
- [从注册表安装二进制文件](ch14-01-more-cargo.md#从注册表安装二进制文件)
- [使用自定义命令扩展 X 工具](ch14-01-more-cargo.md#使用自定义命令扩展-x-工具)

## 智能指针

- [使用 Box<T> 指向堆上的数据](ch15-01-smart-pointers.md)
- [使用 Deref trait 将智能指针像常规引用一样处理](ch15-01-smart-pointers.md#使用-deref-trait-将智能指针像常规引用一样处理)
- [使用 Drop trait 在清理时运行代码](ch15-01-smart-pointers.md#使用-drop-trait-在清理时运行代码)
- [Rc<T>，引用计数智能指针](ch15-01-smart-pointers.md#rct引用计数智能指针)
- [RefCell<T> 和内部可变性模式](ch15-01-smart-pointers.md#refcellt-和内部可变性模式)
- [引用循环可能会泄漏内存](ch15-01-smart-pointers.md#引用循环可能会泄漏内存)

## 无畏并发

- [使用线程同时运行代码](ch16-01-fearless-concurrency.md)
- [使用消息传递在线程之间传输数据](ch16-01-fearless-concurrency.md#使用消息传递在线程之间传输数据)
- [共享状态并发](ch16-01-fearless-concurrency.md#共享状态并发)
- [使用 Sync 和 Send trait 扩展并发](ch16-01-fearless-concurrency.md#使用-sync-和-send-trait-扩展并发)

## 模式和模式匹配

- [所有可能的模式](ch17-01-patterns.md)
- [模式的可反驳性：匹配可能失败的地方](ch17-01-patterns.md#模式的可反驳性匹配可能失败的地方)
- [模式语法](ch17-01-patterns.md#模式语法)

## 高级特性

- [不安全 X](ch18-01-unsafe-x.md)
- [高级 trait](ch18-02-advanced-traits.md)
- [高级类型](ch18-03-advanced-types.md)
- [高级函数和闭包](ch18-04-advanced-functions.md)
- [宏](ch18-05-macros.md)

## 附录

- [A - 关键字](appendix-01-keywords.md)
- [B - 操作符与符号](appendix-02-operators.md)
- [C - 可派生的 Trait](appendix-03-derivable-traits.md)
- [D - 有用的开发工具](appendix-04-useful-development-tools.md)
- [E - 版本说明](appendix-05-editions.md)
- [F - 翻译](appendix-06-translations.md)
- [G - X 是如何开发的？](appendix-07-how-x-is-made.md)
