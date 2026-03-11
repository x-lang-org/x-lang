# x-codegen-wasm

## 项目概述

x-codegen-wasm 是 X 语言编译器的 WebAssembly 绑定，允许在浏览器或 Node.js 环境中直接使用 X 语言编译器。它提供了与 JavaScript 的互操作性，支持将 X 语言代码编译为 TypeScript 或 JavaScript。

## 功能定位

- **Web 平台集成**：使 X 语言编译器能够在浏览器中运行
- **跨语言绑定**：提供 JavaScript/TypeScript API
- **编译服务**：在 Web 环境中提供 X 语言编译功能
- **教育工具**：支持在线代码编辑器和学习平台

## 架构与依赖

```
x-codegen-wasm/
├── Cargo.toml       # 项目元数据和依赖
├── src/
│   └── lib.rs       # 核心实现（WebAssembly 绑定）
└── pkg/             # 编译后的 npm 包输出
```

### 核心依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| wasm-bindgen | 0.2.92 | WebAssembly 与 JavaScript 绑定 |
| x-codegen | 工作区 | 代码生成基础设施 |
| x-lexer | 工作区 | 词法分析器 |
| x-parser | 工作区 | 语法分析器 |
| x-typechecker | 工作区 | 类型检查器 |

### 主要组件

1. **XLangCompiler**：X 语言编译器的 WebAssembly 包装类
2. **compile_x_to_ts**：编译 X 语言代码到 TypeScript 的函数
3. **compile_x_to_js**：编译 X 语言代码到 JavaScript 的函数

## 实现状态

**当前状态**：早期阶段（功能不全）

- 已实现基本的 WebAssembly 绑定
- 支持 X 语言到 TypeScript 的编译
- 类型检查功能被注释掉（TODO）
- 编译到 JavaScript 的功能尚未完整实现

## 使用方法

### 在 JavaScript 中使用

```javascript
import { XLangCompiler } from 'x-codegen-wasm';

// 初始化编译器
const compiler = new XLangCompiler();

try {
    // 编译 X 语言代码到 TypeScript
    const tsCode = compiler.compileXToTs('fn main() { print("Hello") }');
    console.log('Generated TypeScript:', tsCode);

    // 编译 X 语言代码到 JavaScript
    const jsCode = compiler.compileXToJs('fn main() { print("Hello") }');
    console.log('Generated JavaScript:', jsCode);
} catch (error) {
    console.error('Compilation error:', error);
}
```

### 在浏览器中使用

```html
<script type="module">
    import { XLangCompiler } from 'https://cdn.example.com/x-codegen-wasm';

    const compiler = new XLangCompiler();

    async function compileCode() {
        const code = document.getElementById('x-code').value;
        try {
            const tsCode = await compiler.compileXToTs(code);
            document.getElementById('ts-output').textContent = tsCode;
        } catch (error) {
            document.getElementById('error').textContent = error.message;
        }
    }
</script>
```

## 构建与部署

### 构建 WebAssembly 包

```bash
cd compiler/x-codegen-wasm
wasm-pack build --target web
```

### 发布到 npm

```bash
cd compiler/x-codegen-wasm
wasm-pack publish
```

## 代码风格与规范

- 使用标准 Rust 风格，执行 `cargo fmt` 格式化
- WebAssembly 绑定使用 wasm-bindgen 规范
- 函数命名遵循 JavaScript 惯例（驼峰式）
- 文档字符串使用中文，符合项目整体风格

## 相关资源

- **主项目文档**：[../..//CLAUDE.md](../../CLAUDE.md)
- **代码生成基础设施**：[../x-codegen/CLAUDE.md](../x-codegen/CLAUDE.md)
- **编译器架构**：[../../ARCHITECTURE.md](../../ARCHITECTURE.md)
- **wasm-bindgen 文档**：https://rustwasm.github.io/docs/wasm-bindgen/

## Testing & Verification

### 前置依赖（wasm-pack）

本 crate 的 Web 构建与发布依赖 `wasm-pack`。若只跑 Rust 单测/覆盖率，则不要求安装。

### 最小验证（只验证本 crate）

```bash
cd compiler
cargo test -p x-codegen-wasm
```

### 覆盖率与分支覆盖率（目标：行覆盖率 100%，分支覆盖率 100%）

```bash
cd compiler
cargo llvm-cov -p x-codegen-wasm --tests --lcov --output-path target/coverage/x-codegen-wasm.lcov
```

### 绑定可用性验证（生成 wasm 包）

```bash
cd compiler/x-codegen-wasm
wasm-pack build --target web
```
