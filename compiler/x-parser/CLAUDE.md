# x-parser

X语言语法分析器，负责将令牌流转换为抽象语法树（AST）。

## 功能定位

- 语法分析阶段的核心组件
- 将词法分析器产生的令牌流转换为结构化的抽象语法树（AST）
- 提供语法错误报告和错误恢复
- 实现递归下降解析算法

## 依赖关系

**外部依赖**：
- lalrpop = "0.20" - 语法分析生成器（虽已配置但当前使用递归下降解析）
- lalrpop-util = "0.20" - LALRPOP 工具
- thiserror = "1.0" - 错误处理
- log = "0.4" - 日志记录

**内部依赖**：
- x-lexer = { workspace = true } - 词法分析器

## 主要结构

### 核心类型

1. **Program** - 程序根节点
   - 包含声明（Declarations）和语句（Statements）

2. **Declaration** - 声明类型
   - 变量声明：`Variable(VariableDecl)`
   - 函数声明：`Function(FunctionDecl)`
   - 类声明：`Class(ClassDecl)`
   - 接口声明：`Trait(TraitDecl)`
   - 类型别名：`TypeAlias(TypeAlias)`
   - 模块声明：`Module(ModuleDecl)`
   - 导入声明：`Import(ImportDecl)`
   - 导出声明：`Export(ExportDecl)`

3. **Statement** - 语句类型
   - 表达式语句：`Expression(Expression)`
   - 变量声明：`Variable(VariableDecl)`
   - 返回语句：`Return(Option<Expression>)`
   - If语句：`If(IfStatement)`
   - For循环：`For(ForStatement)`
   - While循环：`While(WhileStatement)`
   - Match语句（模式匹配）：`Match(MatchStatement)`
   - Try语句（异常处理）：`Try(TryStatement)`

4. **Expression** - 表达式类型
   - 字面量：`Literal(Literal)`
   - 变量引用：`Variable(String)`
   - 成员访问：`Member(Box<Expression>, String)`
   - 函数调用：`Call(Box<Expression>, Vec<Expression>)`
   - 二元运算：`Binary(BinaryOp, Box<Expression>, Box<Expression>)`
   - 一元运算：`Unary(UnaryOp, Box<Expression>)`
   - 赋值：`Assign(Box<Expression>, Box<Expression>)`
   - 三元条件：`If(Box<Expression>, Box<Expression>, Box<Expression>)`
   - Lambda函数：`Lambda(Vec<Parameter>, Block)`
   - 数组：`Array(Vec<Expression>)`
   - 字典：`Dictionary(Vec<(Expression, Expression)>)`
   - 记录：`Record(String, Vec<(String, Expression)>)`
   - 范围：`Range(Box<Expression>, Box<Expression>, bool)`
   - 管道操作：`Pipe(Box<Expression>, Vec<Box<Expression>>)`
   - Wait操作（异步）：`Wait(WaitType, Vec<Expression>)`
   - Effect相关：`Needs(String)`, `Given(String, Box<Expression>)`
   - 括号表达式：`Parenthesized(Box<Expression>)`

5. **Type** - 类型系统
   - 基本类型：Int, Float, Bool, String, Char, Unit, Never
   - 复合类型：Array, Dictionary, Record, Union, Tuple
   - 高级类型：Option, Result, Function, Async
   - 泛型类型：Generic, TypeParam
   - 类型变量：Var

6. **ParseError** - 解析错误
   - 语法错误：包含位置信息和消息

## 使用方法

```rust
use x_parser::parse_program;

let source = "
function add(x: Int, y: Int) -> Int {
    return x + y;
}

let result = add(2, 3);
";

match parse_program(source) {
    Ok(ast) => println!("AST: {:?}", ast),
    Err(e) => eprintln!("Error: {}", e),
}
```

## 实现状态

**已实现功能**：
- 程序解析
- 变量和函数声明解析
- 基本语句解析（if, while, for）
- 表达式解析（使用优先级攀爬）
- 类型注解解析
- 导入声明解析
- 模块/导出声明解析（`module <name>;`、`export <symbol>;`）
- `match` 语句解析（statement 级：支持 `when` guard 与 `a | b` or-pattern）
- `try/catch/finally` 语句解析（支持 `catch (Type var)`）
- 错误报告和位置信息

**待实现功能**：
- 完善类和接口声明解析
- 完善异步操作（async, wait）解析
- 实现增量解析
- 优化错误恢复

## 测试覆盖

本 crate 含 `#[cfg(test)]` 单元测试（位于 `src/lib.rs`），覆盖 `module/import/export`、`match`、`try/catch/finally` 等关键分支；可通过 `cargo test -p x-parser` 运行。

## Testing & Verification

### 最小验证（只验证本 crate）

```bash
cd compiler
cargo test -p x-parser
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

推荐使用 `cargo llvm-cov` 生成 **line/branch** 覆盖率报告。

```bash
cd compiler
cargo llvm-cov -p x-parser --tests --lcov --output-path target/coverage/x-parser.lcov
```

### 集成验证（上游/下游联动）

```bash
cd tools/x-cli
cargo test -p x-cli
```

## 架构特点

1. **递归下降解析**：不使用 LALRPOP 生成，直接手写递归下降
2. **优先级攀爬**：用于表达式解析，处理运算符优先级
3. **错误恢复**：保留 last_span 供错误报告
4. **位置追踪**：通过 Span 提供精确的错误位置

## 代码生成后端支持

所有后端均依赖此 parser 产生的 AST。
