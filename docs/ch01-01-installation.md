# 安装 X

在我们开始编写 X 语言代码之前，首先需要安装 X 语言工具链。

## 前置要求

X 语言编译器是用 Rust 编写的，因此你需要先安装 Rust。你可以通过以下命令安装 Rust：

### 在 Linux 或 macOS 上

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 在 Windows 上

访问 [https://rustup.rs](https://rustup.rs) 并下载安装程序。

安装完成后，你可以通过以下命令验证 Rust 是否安装成功：

```bash
rustc --version
cargo --version
```

## 安装 X 语言

### 从源码安装

首先，克隆 X 语言的仓库：

```bash
git clone https://github.com/your-username/x-lang.git
cd x-lang
```

然后构建并安装：

```bash
cargo build --release
```

构建完成后，你可以在 `target/release/` 目录下找到 `x` 可执行文件。建议将其添加到你的 PATH 环境变量中。

### 验证安装

安装完成后，运行以下命令验证 X 语言是否安装成功：

```bash
x --version
```

如果看到版本号输出，说明安装成功！

## 常见问题

### 在 macOS 上遇到 "command not found"

确保你已经将 `x` 可执行文件所在的目录添加到 PATH 中。你可以在 `~/.bash_profile` 或 `~/.zshrc` 中添加：

```bash
export PATH="$PATH:/path/to/x-lang/target/release"
```

### 在 Windows 上遇到权限问题

以管理员身份运行命令提示符或 PowerShell。

## 下一步

现在你已经成功安装了 X 语言，让我们在下一节中编写你的第一个 "Hello, World!" 程序！
