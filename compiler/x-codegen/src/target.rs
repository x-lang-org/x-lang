// 目标平台和文件类型定义

/// 支持的目标平台
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Target {
    /// 本地机器码（Native）- Zig后端
    Native,
    /// Java虚拟机（JVM）- Java字节码
    Jvm,
    /// .NET平台 - CIL字节码
    DotNet,
    /// JavaScript - 浏览器或Node.js
    JavaScript,
    /// TypeScript - 类型安全的JavaScript超集
    TypeScript,
    /// WebAssembly - 浏览器或Wasm运行时
    Wasm,
    /// Python 字节码 - .pyc 文件
    Pyc,
    /// Python 源代码 - .py 文件
    Python,
}

impl Target {
    /// 获取目标平台的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            Target::Native => "native",
            Target::Jvm => "jvm",
            Target::DotNet => "dotnet",
            Target::JavaScript => "javascript",
            Target::TypeScript => "typescript",
            Target::Wasm => "wasm",
            Target::Pyc => "pyc",
            Target::Python => "python",
        }
    }

    /// 从字符串解析目标平台
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "native" => Some(Target::Native),
            "jvm" | "java" => Some(Target::Jvm),
            "dotnet" | "net" | "cil" => Some(Target::DotNet),
            "js" | "javascript" => Some(Target::JavaScript),
            "ts" | "typescript" => Some(Target::TypeScript),
            "wasm" => Some(Target::Wasm),
            "pyc" | "pyo3" => Some(Target::Pyc),
            "python" | "py" => Some(Target::Python),
            _ => None,
        }
    }

    /// 获取目标平台的默认文件扩展名
    pub fn default_extension(&self) -> &'static str {
        match self {
            Target::Native => "exe",
            Target::Jvm => "jar",
            Target::DotNet => "dll",
            Target::JavaScript => "js",
            Target::TypeScript => "ts",
            Target::Wasm => "wasm",
            Target::Pyc => "pyc",
            Target::Python => "py",
        }
    }

    /// 检查目标平台是否需要链接器
    pub fn requires_linker(&self) -> bool {
        matches!(self, Target::Native)
    }

    /// 检查目标平台是否需要运行时
    pub fn requires_runtime(&self) -> bool {
        matches!(
            self,
            Target::Jvm | Target::DotNet | Target::JavaScript | Target::TypeScript | Target::Wasm | Target::Pyc | Target::Python
        )
    }

    /// 检查目标平台是否有 Python 虚拟机
    pub fn is_python(&self) -> bool {
        matches!(self, Target::Pyc | Target::Python)
    }
}

/// 输出文件类型
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    /// 目标文件（.o/.obj）
    ObjectFile,
    /// 可执行文件（.exe）
    Executable,
    /// JVM字节码（.class）
    JvmBytecode,
    /// JAR文件（.jar）
    JarFile,
    /// .NET程序集（.dll/.exe）
    DotNetAssembly,
    /// .NET模块（.netmodule）
    DotNetModule,
    /// JavaScript文件（.js）
    JavaScript,
    /// TypeScript文件（.ts）
    TypeScript,
    /// WebAssembly文件（.wasm）
    Wasm,
    /// WebAssembly文本（.wat）
    Wat,
    /// Python 字节码（.pyc）
    Pyc,
    /// Python 源代码（.py）
    Python,
    /// Zig 源代码（.zig）
    Zig,
}

impl FileType {
    /// 获取文件类型的默认扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            FileType::ObjectFile => "o",
            FileType::Executable => "exe",
            FileType::JvmBytecode => "class",
            FileType::JarFile => "jar",
            FileType::DotNetAssembly => "dll",
            FileType::DotNetModule => "netmodule",
            FileType::JavaScript => "js",
            FileType::TypeScript => "ts",
            FileType::Wasm => "wasm",
            FileType::Wat => "wat",
            FileType::Pyc => "pyc",
            FileType::Python => "py",
            FileType::Zig => "zig",
        }
    }

    /// 获取文件类型的描述
    pub fn description(&self) -> &'static str {
        match self {
            FileType::ObjectFile => "Object file",
            FileType::Executable => "Executable",
            FileType::JvmBytecode => "JVM bytecode",
            FileType::JarFile => "JAR file",
            FileType::DotNetAssembly => ".NET assembly",
            FileType::DotNetModule => ".NET module",
            FileType::JavaScript => "JavaScript",
            FileType::TypeScript => "TypeScript",
            FileType::Wasm => "WebAssembly",
            FileType::Wat => "WebAssembly text",
            FileType::Pyc => "Python bytecode",
            FileType::Python => "Python source",
            FileType::Zig => "Zig source",
        }
    }
}
