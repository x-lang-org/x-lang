// 测试所有后端的基本功能

use std::path::PathBuf;
use x_codegen::{get_code_generator, CodeGenConfig, Target};
use x_parser::parse_program;

fn test_backend(target: Target, test_name: &str) {
    println!("\n=== 测试 {} 后端 ===", test_name);

    // 简单的测试代码
    let test_code = r#"
    function main() {
        print("Hello, World!");
    }
    "#;

    // 解析程序
    let program = parse_program(test_code).expect("解析失败");

    // 创建代码生成器
    let config = CodeGenConfig {
        target,
        output_dir: Some(PathBuf::from("./test_output")),
        optimize: false,
        debug_info: true,
    };

    let mut generator = get_code_generator(target, config).expect("创建代码生成器失败");

    // 生成代码
    let output = generator.generate_from_ast(&program).expect("生成代码失败");

    // 打印生成的文件
    println!("生成的文件:");
    for file in &output.files {
        println!("- {:?}", file.path);
        let content = String::from_utf8_lossy(&file.content);
        println!("内容:");
        println!("{}", content);
        println!("---");
    }

    println!("{} 后端测试通过！", test_name);
}

fn main() {
    println!("开始测试所有后端...");

    // 测试 Zig 后端 (Native)
    test_backend(Target::Native, "Zig (Native)");

    // 测试 Zig 后端 (Wasm)
    test_backend(Target::Wasm, "Zig (Wasm)");

    // 测试 Python 后端
    test_backend(Target::Python, "Python");

    // 测试 Java 后端
    test_backend(Target::Jvm, "Java");

    // 测试 C# 后端
    test_backend(Target::DotNet, "C#");

    // 测试 TypeScript 后端
    test_backend(Target::TypeScript, "TypeScript");

    println!("\n所有后端测试完成！");
}
