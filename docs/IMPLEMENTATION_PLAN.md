# X 语言编译器特性实现计划

## 概述

本文档规划 X 语言编译器缺失特性的逐步实现路径，基于现有代码库架构分析制定。

## 实现优先级排序

| 优先级 | 特性 | 依赖关系 | 预估工作量 |
|--------|------|----------|------------|
| P0 | 带值模式匹配完善 | 无 | 中 |
| P1 | Option/Result 方法 | P0 | 中 |
| P2 | 泛型约束检查 | P1 | 高 |
| P3 | 模块系统基础 | 无 | 高 |
| P4 | 效果系统基础 | P2 | 高 |

---

## Phase 1: 带值模式匹配完善 (P0)

### 1.1 现状分析

- **解析器**: 已支持 `Some(v)`, `Ok(v)` 语法 (`x-parser/src/parser.rs:1114`)
- **AST**: `Pattern::EnumConstructor` 已定义 (`x-parser/src/ast.rs:694-709`)
- **解释器**: 已支持运行时模式匹配 (`x-interpreter/src/lib.rs:1228-1335`)
- **缺失**: 类型检查器的穷尽性检查和泛型实例化

### 1.2 实现步骤

#### Step 1.2.1: 集成穷尽性检查
**文件**: `compiler/x-typechecker/src/exhaustiveness.rs`

```rust
// 需要完善的功能：
// 1. 枚举穷尽性检查
// 2. 嵌套模式穷尽性
// 3. 通配符模式处理

pub fn check_exhaustiveness(
    patterns: &[Pattern],
    scrutinee_type: &Type,
    env: &TypeEnv
) -> Result<(), Vec<Pattern>> {
    // 返回缺失的模式
}
```

**修改点**: `x-typechecker/src/lib.rs` 中的 `check_match` 函数

#### Step 1.2.2: 泛型枚举模式类型推断
**文件**: `compiler/x-typechecker/src/lib.rs:3939`

```rust
// Pattern::EnumConstructor 分支需要完善：
// 当匹配 Some(v) 时，从 Option<T> 推断 v: T

Pattern::EnumConstructor(enum_name, variant_name, patterns) => {
    // 1. 查找枚举定义
    // 2. 获取变体参数类型
    // 3. 实例化泛型参数
    // 4. 递归检查子模式
}
```

### 1.3 测试用例

```x
// tests/patterns/enum_pattern_with_value.toml
let opt: Option<Integer> = Some(42)
match opt {
    Some(v) => println(v)  // v: Integer
    None => println(0)
}

let res: Result<Integer, String> = Ok(100)
match res {
    Ok(n) => println(n)    // n: Integer
    Err(e) => println(e)   // e: String
}
```

### 1.4 验收标准

- [ ] `Some(v)` 模式正确推断 `v` 的类型
- [ ] `Ok(v)` 和 `Err(e)` 正确推断类型
- [ ] 穷尽性检查警告缺失的模式
- [ ] 所有现有测试继续通过

---

## Phase 2: Option/Result 方法实现 (P1)

### 2.1 现状分析

- **标准库**: 已定义方法签名 (`library/stdlib/types.x`)
- **解释器**: 可执行方法调用
- **缺失**: 方法调用的类型检查和代码生成

### 2.2 实现步骤

#### Step 2.2.1: 方法调用类型检查
**文件**: `compiler/x-typechecker/src/lib.rs`

```rust
// 在 infer_expression_type 中添加方法调用检查
Expression::Member(obj, method_name) => {
    let obj_type = self.infer_expression_type(obj)?;
    match obj_type {
        Type::Option(inner) => {
            // 检查 Option<T> 的方法
            self.check_option_method(method_name, inner)
        }
        Type::Result(ok, err) => {
            // 检查 Result<T, E> 的方法
            self.check_result_method(method_name, ok, err)
        }
        _ => self.check_class_method(obj_type, method_name)
    }
}
```

#### Step 2.2.2: Option 方法实现

**文件**: `library/stdlib/types.x`

```x
// Option<T> 方法
public function is_some<T>(self: Option<T>) -> boolean {
    match self {
        Some(_) => true
        None => false
    }
}

public function is_none<T>(self: Option<T>) -> boolean {
    match self {
        None => true
        Some(_) => false
    }
}

public function unwrap_or<T>(self: Option<T>, default: T) -> T {
    match self {
        Some(v) => v
        None => default
    }
}

public function map<T, U>(self: Option<T>, f: function(T) -> U) -> Option<U> {
    match self {
        Some(v) => Some(f(v))
        None => None
    }
}

public function and_then<T, U>(self: Option<T>, f: function(T) -> Option<U>) -> Option<U> {
    match self {
        Some(v) => f(v)
        None => None
    }
}
```

#### Step 2.2.3: Result 方法实现

```x
// Result<T, E> 方法
public function is_ok<T, E>(self: Result<T, E>) -> boolean {
    match self {
        Ok(_) => true
        Err(_) => false
    }
}

public function is_err<T, E>(self: Result<T, E>) -> boolean {
    match self {
        Err(_) => true
        Ok(_) => false
    }
}

public function unwrap_or<T, E>(self: Result<T, E>, default: T) -> T {
    match self {
        Ok(v) => v
        Err(_) => default
    }
}

public function map<T, E, U>(self: Result<T, E>, f: function(T) -> U) -> Result<U, E> {
    match self {
        Ok(v) => Ok(f(v))
        Err(e) => Err(e)
    }
}

public function map_err<T, E, F>(self: Result<T, E>, f: function(E) -> F) -> Result<T, F> {
    match self {
        Ok(v) => Ok(v)
        Err(e) => Err(f(e))
    }
}
```

### 2.3 测试用例

```x
// tests/error_handling/option_methods.toml
let opt = Some(42)
println(opt.is_some())       // true
println(opt.unwrap_or(0))    // 42

let none: Option<Integer> = None
println(none.is_none())      // true
println(none.unwrap_or(100)) // 100

// map 测试
let doubled = Some(10).map(function(x) { x * 2 })
match doubled {
    Some(v) => println(v)    // 20
    None => println(0)
}
```

### 2.4 验收标准

- [ ] `is_some()`, `is_none()` 正确工作
- [ ] `unwrap_or()` 返回正确值
- [ ] `map()` 正确转换值
- [ ] `and_then()` 链式操作正确
- [ ] Result 方法类似正确工作

---

## Phase 3: 泛型约束检查 (P2)

### 3.1 现状分析

- **AST**: `TypeParameter.constraints` 已定义
- **AST**: `TraitDecl`, `ImplementDecl` 已定义
- **缺失**: 约束检查和 Trait 方法解析

### 3.2 实现步骤

#### Step 3.2.1: Trait 约束收集
**文件**: `compiler/x-typechecker/src/lib.rs`

```rust
impl TypeEnv {
    // 收集泛型参数的约束
    fn collect_constraints(&mut self, type_params: &[TypeParameter]) {
        for param in type_params {
            for constraint in &param.constraints {
                // 验证 trait 存在
                // 记录约束关系
            }
        }
    }
}
```

#### Step 3.2.2: 约束满足检查

```rust
impl TypeEnv {
    // 检查类型是否满足约束
    fn check_constraint_satisfaction(
        &self,
        ty: &Type,
        constraint: &TypeConstraint
    ) -> bool {
        // 1. 检查是否有 impl Trait for Type
        // 2. 检查内建 trait (Add, Compare, etc.)
        // 3. 检查约束传递
    }
}
```

#### Step 3.2.3: 方法解析

```rust
impl TypeEnv {
    // 解析 trait 方法调用
    fn resolve_trait_method(
        &self,
        trait_name: &str,
        method_name: &str,
        type_args: &[Type]
    ) -> Option<FunctionInfo> {
        // 1. 查找 trait 定义
        // 2. 查找 impl 块
        // 3. 实例化方法类型
    }
}
```

### 3.3 测试用例

```x
// tests/metaprogramming/trait_constraints.toml
trait Comparable {
    function compare(self, other: Self) -> integer
}

function max<T: Comparable>(a: T, b: T) -> T {
    if a.compare(b) > 0 { a } else { b }
}

// 实现 trait
implement Comparable for Integer {
    function compare(self, other: Integer) -> integer {
        self - other
    }
}

// 使用
println(max(10, 20))  // 20
```

### 3.4 验收标准

- [ ] 泛型函数约束检查
- [ ] Trait 方法调用解析
- [ ] `impl` 块类型检查
- [ ] 约束错误报告

---

## Phase 4: 模块系统基础 (P3)

### 4.1 现状分析

- **解析器**: 支持 `module`, `import`, `export` 语法
- **HIR/MIR/LIR**: 有 `Import` 结构
- **缺失**: 模块解析、符号导入、多文件编译

### 4.2 实现步骤

#### Step 4.2.1: 创建模块解析器

**新建文件**: `compiler/x-resolver/src/lib.rs`

```rust
pub struct ModuleResolver {
    // 模块路径 -> 文件路径
    module_paths: HashMap<String, PathBuf>,
    // 模块导出符号
    exports: HashMap<String, Vec<Export>>,
    // 已解析模块缓存
    resolved: HashMap<String, Module>,
}

impl ModuleResolver {
    // 解析模块路径
    pub fn resolve_import(&mut self, import: &Import) -> Result<Module, ResolverError> {
        // 1. 查找模块文件
        // 2. 解析模块
        // 3. 提取导出符号
    }

    // 解析符号
    pub fn resolve_symbol(&self, module: &str, name: &str) -> Option<Symbol> {
        // 查找导出符号
    }
}
```

#### Step 4.2.2: 类型环境扩展

**文件**: `compiler/x-typechecker/src/lib.rs`

```rust
impl TypeEnv {
    // 注册模块
    pub fn register_module(&mut self, name: &str, exports: Vec<Export>) {
        self.modules.insert(name.to_string(), ModuleInfo { exports });
    }

    // 导入符号
    pub fn import_symbol(&mut self, module: &str, name: &str, alias: Option<&str>) -> Result<(), TypeError> {
        // 1. 查找模块
        // 2. 查找符号
        // 3. 添加到当前作用域
    }
}
```

#### Step 4.2.3: CLI 多文件支持

**文件**: `tools/x-cli/src/main.rs`

```rust
// 支持多文件编译
fn compile_project(entry: &Path) -> Result<(), CliError> {
    // 1. 解析入口文件
    // 2. 收集所有 import
    // 3. 解析依赖模块
    // 4. 构建完整模块图
    // 5. 类型检查
    // 6. 代码生成
}
```

### 4.3 测试用例

```
// 项目结构
myapp/
├── x.toml
└── src/
    ├── main.x
    └── utils.x

// src/utils.x
module myapp.utils

export function greet(name: string) -> string {
    "Hello, " + name
}

// src/main.x
module myapp

import myapp.utils.greet

println(greet("World"))
```

### 4.4 验收标准

- [ ] 解析 `module` 声明
- [ ] 解析 `import` 语句
- [ ] 解析 `export` 语句
- [ ] 符号正确导入
- [ ] 多文件编译工作

---

## Phase 5: 效果系统基础 (P4)

### 5.1 现状分析

- **规范**: 完整定义 (`spec/docs/07-effects.md`)
- **解析器**: 部分语法支持
- **缺失**: 类型检查和语义实现

### 5.2 实现步骤

#### Step 5.2.1: 效果类型定义

**文件**: `compiler/x-parser/src/ast.rs`

```rust
pub enum Effect {
    IO,
    Async,
    State(Box<Type>),
    Throws(Box<Type>),
    Custom(String),
}

pub struct EffectList(pub Vec<Effect>);

// 函数签名扩展
pub struct FunctionSignature {
    pub params: Vec<Parameter>,
    pub return_type: Type,
    pub effects: Option<EffectList>,  // with IO, Async, ...
}
```

#### Step 5.2.2: 效果传播检查

**文件**: `compiler/x-typechecker/src/lib.rs`

```rust
impl TypeEnv {
    // 检查效果传播
    fn check_effect_propagation(
        &self,
        called_effects: &[Effect],
        caller_effects: &[Effect],
    ) -> Result<(), EffectError> {
        // 被调用函数的效果必须是调用者效果的子集
        for effect in called_effects {
            if !caller_effects.contains(effect) {
                return Err(EffectError::UnhandledEffect(effect.clone()));
            }
        }
        Ok(())
    }
}
```

#### Step 5.2.3: IO 效果基础实现

```rust
// 标记 IO 效果的函数
fn check_io_effect(&mut self, expr: &Expression) -> Result<Vec<Effect>, TypeError> {
    match expr {
        Expression::Call(name, _) if is_io_function(name) => {
            Ok(vec![Effect::IO])
        }
        // ...
    }
}
```

### 5.3 测试用例

```x
// tests/effects/io_effect_check.toml
// IO 效果必须声明
function greet(name: string) -> string with IO {
    println("Hello, " + name)
    "greeted"
}

// 纯函数不需要效果声明
function add(a: integer, b: integer) -> integer {
    a + b
}

// 测试
greet("World")
```

### 5.4 验收标准

- [ ] 解析 `with` 效果列表
- [ ] IO 效果传播检查
- [ ] 效果子类型检查
- [ ] 错误报告

---

## 时间线

| 阶段 | 特性 | 预计时间 |
|------|------|----------|
| Week 1-2 | Phase 1: 带值模式匹配 | 2 周 |
| Week 3-4 | Phase 2: Option/Result 方法 | 2 周 |
| Week 5-7 | Phase 3: 泛型约束 | 3 周 |
| Week 8-10 | Phase 4: 模块系统 | 3 周 |
| Week 11-14 | Phase 5: 效果系统 | 4 周 |

---

## 风险和依赖

### 技术风险
1. **泛型实例化**: 类型参数替换可能影响多个阶段
2. **模块循环依赖**: 需要仔细处理模块图
3. **效果多态**: 高级效果特性可能需要更多设计

### 缓解策略
1. 每个阶段完成后运行完整测试套件
2. 增量实现，先支持基础用例
3. 保持与 SPEC 规范一致

---

## 参考

- SPEC 规范: `spec/docs/`
- 现有测试: `tests/`
- 标准库: `library/stdlib/`
