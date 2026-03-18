# x-lexer

X语言词法分析器，负责将源代码转换为令牌流。

## 功能定位

- 词法分析阶段的核心组件
- 将源代码字符串转换为编译器后续阶段可处理的令牌序列
- 处理注释、空白字符和各种字面量的解析
- 提供错误恢复和位置信息（Span）

## 依赖关系

**外部依赖**：
- thiserror = "1.0" - 错误处理
- log = "0.4" - 日志记录

**内部依赖**：
无（独立组件）

## 主要结构

### 核心类型

1. **Token** - 定义所有词法单元类型
   - 关键字：`let`, `mut`, `val`, `var`, `const`, `function`, `async`, `class` 等
   - 标识符：`Ident(String)`
   - 数字字面量：`DecimalInt`, `Float`, `HexInt`, `OctInt`, `BinInt`
   - 字符串字面量：`StringQuote`, `MultilineStringQuote`, `StringContent`
   - 字符字面量：`CharQuote`, `CharContent`
   - 运算符：各种二元和一元运算符
   - 结束标记：`Eof`

2. **Lexer** - 词法分析器
   - 状态管理：`LexerState`（Normal, String, MultilineString, Char）
   - 核心方法：`next_token()` 产生下一个令牌
   - 辅助方法：解析标识符、数字、字符串、字符、注释等

3. **TokenIterator** - 令牌迭代器
   - 包装 Lexer 提供迭代接口
   - 保留 last_span 供解析错误使用
   - 支持 peek() 操作

4. **Span** - 源代码位置信息
   - 字节偏移范围：`start` 和 `end`
   - 提供行号/列号转换：`line_col()`
   - 提供源代码片段：`snippet()`

5. **LexError** - 词法分析错误
   - `InvalidToken(char, usize)` - 无效标记（含字符和位置）
   - `UnclosedString` - 字符串未闭合
   - `UnclosedChar` - 字符未闭合
   - `InvalidNumber(usize, String)` - 无效数字格式（含位置和内容）
   - `InvalidUnicodeEscape(String)` - 无效的 Unicode 转义
   - `InvalidCharEscape(String)` - 无效的字符转义

## 使用方法

```rust
use x_lexer::new_lexer;

let source = "let x = 42;";
let mut iter = new_lexer(source);

while let Some(token_result) = iter.next() {
    match token_result {
        Ok((token, span)) => println!("Token: {:?}, Span: {:?}", token, span),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## 实现状态

**已实现功能**：
- 完整的关键字解析
- 标识符、数字、字符串、字符字面量解析
- 单行（`//`）和多行（`/* */`）注释
- 大部分运算符解析
- 错误处理和位置信息
- 字符串转义：`\n` `\t` `\r` `\"` `\'` `\\` `\0` 及单行未闭合时返回 `UnclosedString`
- 字符字面量：`'x'` 及转义 `\n` `\t` `\r` `\'` `\\` `\0`，未闭合返回 `UnclosedChar`
- 数字：十进制、浮点、科学计数法、数字分隔符 `_`；前缀 `0x`/`0X`（十六进制）、`0o`/`0O`（八进制）、`0b`/`0B`（二进制），无效前缀返回 `InvalidNumber`
- Span 行号列号：`line_col()` 使用 `split('\n')` 与列偏移，正确反映行首
- **多行字符串**：`"""..."""` 语法，支持跨行
- **Unicode 转义**：`\u{...}` 语法，支持任意 Unicode 码点（如 `\u{4E2D}` = "中"）
- **Shebang 支持**：识别并跳过 `#!` 开头的脚本头
- **BOM 处理**：自动跳过 UTF-8 BOM (0xEF 0xBB 0xBF)
- **改进的错误信息**：包含位置和上下文信息

**待实现功能**：
- 字符串插值（`${expr}`）
- 错误恢复机制
- 多错误报告
- 处理 Unicode 标识符
- 增量词法分析
- 性能基准测试

## 测试覆盖

本 crate 含 39 个 `#[cfg(test)]` 单元测试，覆盖：关键字、标识符、整数/浮点/十六进制/八进制/二进制、字符串与转义（含多行字符串）、字符与转义（含 Unicode）、未闭合字符串/字符、非法数字、运算符、标点、Span、空输入、空白与注释、peek 与 last_span、shebang、BOM。可通过 `cargo test -p x-lexer` 运行；覆盖率可用 `cargo llvm-cov -p x-lexer --tests` 查看。

## Testing & Verification

### 最小验证（只验证本 crate）

```bash
cd compiler
cargo test -p x-lexer
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

推荐使用 `cargo llvm-cov` 生成 **line/branch** 覆盖率报告（Windows 兼容性更好）。

```bash
cd compiler
cargo llvm-cov -p x-lexer --tests --lcov --output-path target/coverage/x-lexer.lcov
```

如果需要在本地查看详细报告，可改用 html：

```bash
cd compiler
cargo llvm-cov -p x-lexer --tests --html
```

### 集成验证（通过上游使用方间接验证）

```bash
cd compiler
cargo test -p x-parser
```

## 代码生成后端支持

所有后端均依赖此 lexer。
