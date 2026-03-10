use std::path::PathBuf;
use x_parser::parser::XParser;
use x_codegen::c_backend::{CBackend, CBackendConfig};

fn main() {
    // 读取benchmark.x文件
    let file = "examples/benchmark.x";
    let content = std::fs::read_to_string(file).expect("无法读取文件");
    
    // 解析X语言代码
    let parser = XParser::new();
    let program = parser.parse(&content).expect("解析失败");
    
    // 创建C后端
    let config = CBackendConfig::default();
    let mut backend = CBackend::new(config);
    
    // 生成C代码
    let c_code = backend.generate_from_ast(&program).expect("C代码生成失败");
    
    // 设置输出文件路径
    let output_file = PathBuf::from("benchmark.exe");
    
    // 设置MSVC编译器路径
    std::env::set_var("CC", "C:\\Program Files (x86)\\Microsoft Visual Studio\\18\\BuildTools\\VC\\Tools\\MSVC\\14.50.35717\\bin\\Hostx64\\x64\\cl.exe");
    
    // 编译C代码
    backend.compile_c_code(&c_code, &output_file).expect("C编译失败");
    
    println!("编译成功！可执行文件: benchmark.exe");
}