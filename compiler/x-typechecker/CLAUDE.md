# x-typechecker

X语言类型检查器。

## 功能定位

- 语义分析阶段的核心组件
- 对AST进行类型检查和验证
- 确保程序的类型安全性
- 提供类型信息供后续编译阶段使用

## 依赖关系

**外部依赖**：
- im = "15.1" - 不可变数据结构
- thiserror = "1.0" - 错误处理
- log = "0.4" - 日志记录

**内部依赖**：
- x-parser = { workspace = true } - 语法分析器（获取AST）

## 主要结构

### 核心类型

1. **TypeCheckError** - 类型检查错误
   - 类型不匹配：`TypeMismatch`
   - 未定义变量：`UndefinedVariable`
   - 重复声明：`DuplicateDeclaration`
   - 未定义类型：`UndefinedType`
   - 参数数量不匹配：`ParameterCountMismatch`
   - 参数类型不匹配：`ParameterTypeMismatch`
   - 无法推断类型：`CannotInferType`
   - 类型参数数量不匹配：`TypeParameterCountMismatch`
   - 类型参数约束违规：`TypeParameterConstraintViolated`
   - 递归类型定义：`RecursiveType`
   - 无效类型注解：`InvalidTypeAnnotation`
   - 类型不兼容：`TypeIncompatible`

2. **TypeEnv** - 类型环境
   - 变量类型映射：作用域栈（block/function/branch）内逐层查找
   - 函数类型映射：`functions: HashMap<String, Type>`
   - 支持添加和查找变量、函数类型

### 核心函数

1. **type_check()** - 主类型检查函数
   - 入口点：接受完整程序AST
   - 初始化类型环境
   - 调用 check_program() 进行检查

2. **check_program()** - 程序检查
   - 先检查所有声明
   - 再检查所有语句
   - 递归处理各个部分

3. **check_declaration()** - 声明检查
   - 处理变量声明：`check_variable_decl()`
   - 处理函数声明：`check_function_decl()`
   - 其他声明暂未实现

4. **check_statement()** - 语句检查
   - 处理表达式语句
   - 处理变量声明
   - 处理返回语句
   - 处理if语句
   - 处理while语句
   - 其他语句暂未实现

5. **infer_expression_type()** - 类型推断
   - 处理字面量类型
   - 处理变量和函数引用
   - 处理成员访问和函数调用
   - 处理二元和一元运算
   - 处理赋值和条件表达式
   - 其他表达式暂未实现

6. **types_equal()** - 类型比较
   - 基础类型比较
   - 数组和字典类型比较
   - 函数类型比较
   - 其他复合类型比较

## 使用方法

```rust
use x_parser::parse_program;
use x_typechecker::type_check;

let source = "
function add(x: Int, y: Int) -> Int {
    return x + y;
}

let result = add(2, 3);
";

match parse_program(source) {
    Ok(ast) => {
        if let Err(e) = type_check(&ast) {
            eprintln!("Type error: {}", e);
        } else {
            println!("Type check passed");
        }
    }
    Err(e) => eprintln!("Parse error: {}", e),
}
```

## 实现状态

**已实现功能**：
- 基本类型系统定义
- 类型检查框架
- 变量和函数声明检查
- 基本表达式类型推断
- 错误类型定义
- 作用域处理：函数参数/if/while/for/match/try/catch/finally 引入新作用域（避免变量泄漏）
- 语义检查：未定义变量、同一作用域重复声明、函数调用参数数量/类型校验

**待实现功能**：
- 完善类型系统（支持所有Type变体）
- 完善类型推断（处理更复杂的表达式）
- 实现类型参数化和泛型
- 完善模式匹配的类型检查
- 完善异常处理的类型检查
- 实现类型约束系统
- 支持类型别名和类型定义
- 完善类型错误信息

## 架构特点

1. **类型系统**：基于Type枚举，支持基本类型、复合类型、泛型等
2. **类型环境**：使用HashMap管理类型信息
3. **递归处理**：对AST进行深度优先遍历
4. **错误报告**：提供位置信息和详细错误消息

## 代码生成后端支持

所有后端均依赖此类型检查器提供类型信息。

## Testing & Verification

### 最小验证（只验证本 crate）

```bash
cd compiler
cargo test -p x-typechecker
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

```bash
cd compiler
cargo llvm-cov -p x-typechecker --tests --lcov --output-path target/coverage/x-typechecker.lcov
```

### 集成验证（与 parser/cli 联动）

```bash
cd tools/x-cli
cargo test -p x-cli
```

## 未来规划

1. 完善类型系统和类型推断
2. 实现完整的语义分析
3. 集成Perceus分析
4. 优化类型检查性能
