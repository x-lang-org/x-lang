# 第11章 元编程

## 11.1 概述

### 定义
```
Metaprogramming: Programs that operate on types or run at compile time.
```

**执行说明：**

元编程允许在编译期进行计算与决策，从而：
- 减少运行时开销（常量折叠、类型特化）
- 根据类型或常量生成不同代码
- 在编译期捕获错误（静态断言、约束检查）

本章涵盖：编译期常量、常量表达式、泛型与类型参数、静态断言，以及（可选）宏与代码生成的设计方向。

---

## 11.2 编译期常量

### 常量声明语法
```
ConstantDeclaration ::= 'const' Identifier ':' Type '=' ConstExpression ';'

ConstExpression ::= ConstLiteral
                  | ConstIdentifier
                  | ConstUnaryOp ConstExpression
                  | ConstExpression ConstBinaryOp ConstExpression
                  | ConstCall
```

**执行说明：**

1. **常量**：
   - 在编译期求值，结果嵌入到程序中
   - 必须有显式类型注解
   - 只能使用常量表达式初始化
   - 命名约定：推荐 `UPPER_SNAKE_CASE`

2. **常量表达式**：
   - 仅包含字面量、已定义常量、纯算术/逻辑运算
   - 不包含运行时输入、I/O、随机数或未限定为 const 的函数调用

### 常量求值规则
```
⟦const x: T = e⟧ = embed(⟦e⟧ᶜᵒⁿˢᵗ)

⟦e⟧ᶜᵒⁿˢᵗ  denotes compile-time evaluation of e
embed(v)   embeds value v into the binary
```

---

## 11.3 常量函数（const fn）

### 常量函数语法
```
ConstFunctionDeclaration ::= 'const' 'fun' Identifier Parameters (':' Type)? ConstFunctionBody

ConstFunctionBody ::= '=' ConstExpression ';'
                    | Block  // block may only contain const-evaluable statements
```

**执行说明：**

1. **const fun**：
   - 可在编译期调用的函数
   - 函数体仅允许常量表达式、其他 const fun 调用、以及受限控制流（如常量条件分支）
   - 不允许：I/O、分配、非 const 的全局状态、无限循环

2. **用途**：
   - 在 const 初始化式中复用逻辑
   - 在类型或数组长度等编译期上下文中使用

### 常量函数规则
```
Γ ⊢ e : T    e is const-evaluable
────────────────────────────────────
Γ ⊢ const fun f(...) = e : const (T)

const-evaluable(e):
  - literals, const identifiers
  - const fun calls
  - pure arithmetic, comparison, short-circuit logic
  - conditional with const condition
```

---

## 11.4 泛型与类型参数

### 泛型声明语法
```
GenericDeclaration ::= Identifier '<' TypeParameter (',' TypeParameter)* '>' (':' Constraint)*

TypeParameter ::= Identifier
                | Identifier ':' Type  // bounded
                | Identifier ':' 'const' Type  // const generic

Constraint ::= Type ':' Trait
             | Type '==' Type  // type equality (for const generics)
```

**执行说明：**

1. **类型参数**：
   - 在函数、类、类型别名上参数化类型
   - 未绑定时可受使用处推断或显式实参约束
   - 绑定形式 `T: Trait` 表示 T 必须实现 Trait

2. **常量泛型（const generic）**：
   - 用编译期常量（如整数、布尔、某类常量）参数化类型或函数
   - 典型用途：固定长度数组、维度、编译期配置

### 泛型实例化规则
```
Γ ⊢ f: ∀α. T → U    Γ ⊢ τ type    Γ ⊢ τ instantiates α
────────────────────────────────────────────────────
Γ ⊢ f(τ): T[α↦τ] → U[α↦τ]

monomorphise(f, concrete_types) = f'  // one specialised copy per instantiation
```

---

## 11.5 静态断言

### 静态断言语法
```
StaticAssert ::= 'static_assert' '(' ConstExpression ',' StringLiteral? ')' ';'
               | 'static_assert' ConstExpression (',' StringLiteral)? ';'
```

**执行说明：**

1. **static_assert**：
   - 条件必须是常量表达式，在编译期求值
   - 若求值结果为 false，编译失败；可选字符串作为错误消息
   - 用于不变量、配置一致性、类型/常量约束的检查

### 静态断言规则
```
⟦static_assert(c, msg?)⟧ =
  if ⟦c⟧ᶜᵒⁿˢᵗ == true then ()
  else compile_error(msg ?: "static assertion failed")
```

### 示例
```
const MAX: Int = 1024;
static_assert(MAX > 0, "MAX must be positive");

const N: Int = 8;
static_assert(N.is_power_of_two());  // 若存在 const fn
```

---

## 11.6 编译期条件

### 编译期条件语法（设计）
```
CompileTimeCondition ::= '#if' ConstExpression Block
                       | '#if' ConstExpression Block '#else' Block
                       | '#if' ConstExpression Block ('#elseif' ConstExpression Block)* '#else' Block?
```

**执行说明：**

1. **#if / #else / #elseif**：
   - 条件为常量表达式，在编译期求值
   - 仅保留满足条件的分支参与编译，未选中分支不生成代码
   - 用于跨平台、特性开关、根据常量选择实现

2. **与运行时 when 的区别**：
   - `#if` 在编译期消除分支，不产生运行时开销
   - `when` 为运行时多分支

### 编译期条件规则
```
⟦#if c then b₁ else b₂⟧ =
  if ⟦c⟧ᶜᵒⁿˢᵗ then ⟦b₁⟧ else ⟦b₂⟧

Only one branch is type-checked and compiled.
```

---

## 11.7 类型级编程（设计方向）

### 类型作为值
```
TypeExpression ::= 'type' '(' Type ')'       // type as term
                 | 'sizeof' '(' Type ')'     // compile-time constant
                 | 'alignof' '(' Type ')'    // compile-time constant
```

**执行说明：**

- `type(T)`：在需要“类型作为值”的上下文中使用类型 T（如传给泛型或类型级函数）
- `sizeof(T)` / `alignof(T)`：产生编译期整数常量，用于布局、缓冲区大小、静态断言

### 类型级约束
```
where Clause ::= 'where' (TypeConstraint (',' TypeConstraint)*)?

TypeConstraint ::= Type ':' Trait
                 | Type '==' Type
                 | Identifier ':' Type      // const generic bound
```

**执行说明：**

- `where` 子句对类型参数或常量泛型施加约束
- 编译器在实例化时检查约束满足，并在需要时用于类型推断与重载解析

---

## 11.8 宏与代码生成（设计方向）

### 宏的定位
```
MacroInvocation ::= Identifier '!' MacroInput?

MacroInput ::= '(' TokenTree* ')'
             | '[' TokenTree* ']'
             | '{' TokenTree* '}'
```

**执行说明：**

1. **宏**（若引入）：
   - 在语法/词法层面进行代码变换，在编译早期执行
   - 可生成重复结构、领域语法、样板代码
   - 设计原则：卫生性（hygiene）、可预测的展开、与类型系统交互清晰

2. **替代与补充**：
   - 优先使用泛型、const fn、静态断言满足需求
   - 宏作为补充，用于语法扩展或无法用类型/常量表达的模式

### 代码生成
```
CodeGeneration ::= 'generate' GeneratorConfig Block
                 |  // 或通过注解/属性驱动
```

**执行说明：**

- 代码生成可在编译管线中由注解、属性或外部工具触发
- 与类型检查的先后顺序、错误报告、增量编译需在实现中明确

---

## 11.9 与其它章节的关系

| 章节       | 关系说明 |
|------------|----------|
| 第2章 类型 | 泛型基于类型系统；const generic 扩展类型与值的边界 |
| 第5章 函数 | const fun 是函数的编译期子集；泛型函数多态 |
| 第10章 内存 | 编译期常量无运行时分配；const 不涉及 Perceus 操作 |
| 第8章 模块 | 常量与泛型在模块内可见性遵循模块规则 |

---

## 11.10 小结

- **编译期常量**：`const` 声明与常量表达式，在编译期求值并嵌入。
- **常量函数**：`const fun` 可在编译期调用，用于复用常量逻辑。
- **泛型**：类型参数与（可选）常量泛型，支持多态与类型/常量级抽象。
- **静态断言**：`static_assert` 在编译期检查不变量与配置。
- **编译期条件**：`#if` 等在编译期选择代码分支。
- **类型级编程**：`sizeof`/`alignof`、`where` 约束等，支撑高级泛型与布局需求。
- **宏与代码生成**：作为可选扩展，在语法与生成层面提供元编程能力。
