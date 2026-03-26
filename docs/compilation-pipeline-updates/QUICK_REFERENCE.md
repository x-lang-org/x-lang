# X 语言编译流水线快速参考

## 快速开始

### 运行程序（解释执行）
```bash
cd tools/x-cli && cargo run -- run ../../examples/hello.x
```

### 完整编译（生成可执行文件）
```bash
cd tools/x-cli && cargo run -- compile ../../examples/hello.x -o hello
```

### 查看编译各阶段的输出
```bash
# 词法分析
cargo run -- compile hello.x --emit tokens

# 语法分析
cargo run -- compile hello.x --emit ast

# 高层中间表示（HIR）
cargo run -- compile hello.x --emit hir

# 中层中间表示（MIR）
cargo run -- compile hello.x --emit mir

# 低层中间表示（LIR）← 后端统一输入
cargo run -- compile hello.x --emit lir

# Zig 代码生成
cargo run -- compile hello.x --emit zig

# 仅生成代码，不链接
cargo run -- compile hello.x --no-link
```

---

## 编译流水线架构

```
源代码 (.x 文件)
    ↓ Lexer (词法分析)
令牌流
    ↓ Parser (语法分析)
AST (抽象语法树)
    ↓ TypeChecker (类型检查)
类型化 AST
    ↓ HIR (高层中间表示)
HIR
    ↓ MIR (中层中间表示，含 Perceus 分析)
MIR
    ↓ LIR (低层中间表示，后端统一输入) ⭐
LIR
    ↓ CodeGenerator (代码生成)
┌──────────────────────────────┐
│ Zig 代码 → Zig 编译器        │
│ JS 代码  → Node.js           │
│ JVM 代码 → Java 虚拟机       │
│ .NET 代码 → .NET 运行时      │
└──────────────────────────────┘
    ↓
可执行文件或目标代码
```

---

## 关键概念

### LIR (Low-level Intermediate Representation)
- **用途**: 所有后端的统一输入
- **特点**: 类似 C 的简化中间表示
- **包含**: 函数、变量、语句、表达式、类型定义
- **优势**: 
  - 后端实现简单（只需 LIR → 目标语言）
  - 可进行平台无关的优化
  - Perceus 分析结果可被所有后端利用

### CodeGenerator Trait
```rust
pub trait CodeGenerator {
    fn generate_from_ast(&mut self, program: &AstProgram) -> Result<CodegenOutput>;
    fn generate_from_hir(&mut self, hir: &x_hir::Hir) -> Result<CodegenOutput>;
    fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput>; // ⭐ 正式输入
}
```

---

## 已实现的后端

| 后端 | 状态 | 文件位置 | 成熟度 |
|------|------|---------|--------|
| Zig | ✅ LIR 支持 | `compiler/x-codegen/src/zig_backend.rs` | 成熟 |
| JavaScript | ✅ LIR 支持 | `compiler/x-codegen-js/src/lib.rs` | 初期 |
| JVM | 🚧 框架就绪 | `compiler/x-codegen-jvm/src/lib.rs` | 早期 |
| .NET | 🚧 框架就绪 | `compiler/x-codegen-dotnet/src/lib.rs` | 早期 |
| Rust | 🔄 待迁移 | `compiler/x-codegen/src/rust_backend.rs` | 早期 |
| C | 🔄 待迁移 | `compiler/x-codegen/src/c_backend.rs` | 早期 |

---

## 常见任务

### 添加新后端

1. **创建 crate**（如果是独立后端）
   ```bash
   cargo new --lib compiler/x-codegen-newlang
   ```

2. **实现 CodeGenerator trait**
   ```rust
   impl CodeGenerator for NewLangBackend {
       type Config = NewLangConfig;
       type Error = NewLangError;
       
       fn generate_from_lir(&mut self, lir: &x_lir::Program) -> Result<CodegenOutput> {
           // 1. 遍历 lir.declarations
           // 2. 对每个声明调用相应的 emit_* 方法
           // 3. 返回 CodegenOutput
       }
   }
   ```

3. **实现 LIR 处理方法**
   ```rust
   fn emit_lir_function(&mut self, func: &x_lir::Function) -> Result<()> { }
   fn emit_lir_statement(&mut self, stmt: &x_lir::Statement) -> Result<()> { }
   fn emit_lir_expression(&mut self, expr: &x_lir::Expression) -> Result<String> { }
   // 等等...
   ```

4. **参考现有实现**
   - Zig 后端: `compiler/x-codegen/src/zig_backend.rs` (完整、成熟)
   - JavaScript 后端: `compiler/x-codegen-js/src/lib.rs` (简单、易参考)

### 调试编译问题

1. **查看词法分析结果**
   ```bash
   cargo run -- compile test.x --emit tokens
   ```
   检查: 令牌序列是否正确？有无非法字符？

2. **查看语法分析结果**
   ```bash
   cargo run -- compile test.x --emit ast
   ```
   检查: AST 结构是否正确？优先级对吗？

3. **查看类型检查结果**
   ```bash
   cargo run -- check test.x
   ```
   检查: 有无类型错误？变量是否定义？

4. **查看 HIR**
   ```bash
   cargo run -- compile test.x --emit hir
   ```
   检查: 符号表对吗？宏展开对吗？

5. **查看 MIR**
   ```bash
   cargo run -- compile test.x --emit mir
   ```
   检查: 控制流简化对吗？Perceus 分析结果？

6. **查看 LIR**
   ```bash
   cargo run -- compile test.x --emit lir
   ```
   检查: 是否已准备好被后端处理？

7. **查看生成的代码**
   ```bash
   cargo run -- compile test.x --emit zig
   ```
   检查: 生成的代码可读吗？逻辑对吗？

8. **仅生成代码，不链接**
   ```bash
   cargo run -- compile test.x --no-link
   cargo run -- compile test.x --emit zig
   ```
   检查: Zig 编译器是否接受生成的代码？

### 运行编译器测试

```bash
# 单元测试
cd compiler && cargo test

# 特定 crate 的测试
cd compiler && cargo test -p x-codegen

# 特定函数的测试
cd compiler && cargo test -p x-parser parse_function

# 规格测试
cargo run -p x-spec

# 所有测试
./test.sh
```

---

## LIR 数据结构参考

### 基本结构
```rust
pub struct Program {
    pub declarations: Vec<Declaration>,
}

pub enum Declaration {
    Function(Function),
    Global(GlobalVar),
    Struct(Struct),
    Enum(Enum),
    // ... 等等
}

pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<Parameter>,
    pub body: Block,
}

pub struct Block {
    pub statements: Vec<Statement>,
}
```

### 语句类型
```rust
pub enum Statement {
    Expression(Expression),
    Variable(Variable),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    Return(Option<Expression>),
    Break,
    Continue,
    // ... 等等
}
```

### 表达式类型
```rust
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Call(Box<Expression>, Vec<Expression>),
    // ... 等等
}

pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Char(char),
    NullPointer,
    // ... 等等
}
```

### 类型
```rust
pub enum Type {
    Void,
    Bool,
    Int,
    Float,
    // ... 基本类型
    Pointer(Box<Type>),
    Array(Box<Type>, Option<u64>),
    Named(String),
    // ... 等等
}
```

---

## 性能优化建议

### 编译速度优化
1. **使用 release 模式编译编译器**
   ```bash
   cargo build --release
   ```

2. **启用 LTO（Link Time Optimization）**
   在 `Cargo.toml` 中:
   ```toml
   [profile.release]
   lto = true
   ```

3. **缓存流水线产物**（未来特性）
   ```bash
   cargo run -- compile test.x --use-cache
   ```

### 生成代码质量优化
1. **启用优化编译**
   ```bash
   cargo run -- compile test.x --release
   ```

2. **查看 Perceus 分析结果**
   在 MIR 中查看内存管理信息

3. **使用 Zig 的优化选项**
   ```bash
   cargo run -- compile test.x -o output --release
   ```

---

## 故障排除

### 问题：编译器崩溃
**解决**:
1. 运行类型检查: `cargo run -- check test.x`
2. 查看 AST: `cargo run -- compile test.x --emit ast`
3. 查看错误消息中的行号

### 问题：生成的代码无法编译
**解决**:
1. 查看生成的 Zig 代码: `cargo run -- compile test.x --emit zig`
2. 手动编译 Zig 代码: `zig build-exe output.zig`
3. 查看 Zig 编译器的错误消息

### 问题：运行时错误
**解决**:
1. 使用解释执行: `cargo run -- run test.x`
2. 添加调试输出（如 `println!`）
3. 查看 HIR/MIR 的控制流

### 问题：性能差
**解决**:
1. 测量编译时间各阶段: 添加 `--verbose` 标志（待实现）
2. 查看 LIR 大小和复杂度
3. 检查 Perceus 分析是否正确

---

## 相关文档

- **详细报告**: [COMPLETION_REPORT.md](COMPLETION_REPORT.md)
- **实现规范**: [DESIGN_GOALS.md](../../DESIGN_GOALS.md)
- **架构审计**: [COMPILER_PIPELINE_AUDIT.md](../../COMPILER_PIPELINE_AUDIT.md)
- **TODO 清单**: [TODO_COMPILER_PIPELINE.md](../../TODO_COMPILER_PIPELINE.md)
- **开发指南**: [CLAUDE.md](../../CLAUDE.md)

---

## 快速命令速记

```bash
# 构建
cd compiler && cargo build
cd tools/x-cli && cargo build

# 运行
cargo run -- run hello.x
cargo run -- compile hello.x -o output

# 调试
cargo run -- compile hello.x --emit lir
cargo run -- compile hello.x --emit zig

# 测试
cargo test
cargo test -p x-codegen
cargo run -p x-spec

# 清理
cargo clean
```

---

## 关键文件速查

| 功能 | 文件 | 行数 |
|------|------|------|
| CodeGenerator trait | `compiler/x-codegen/src/lib.rs` | ~70 |
| Zig 后端 LIR 生成 | `compiler/x-codegen/src/zig_backend.rs` | 260-330 |
| JS 后端 LIR 生成 | `compiler/x-codegen-js/src/lib.rs` | 666-950 |
| 编译命令 | `tools/x-cli/src/commands/compile.rs` | 20-100 |
| 流水线 | `tools/x-cli/src/pipeline.rs` | 365-385 |
| LIR 定义 | `compiler/x-lir/src/lib.rs` | ~100-500 |

---

**最后更新**: 2024年  
**文档版本**: 1.0  
**状态**: 完成 Phase 1 & Phase 2 部分