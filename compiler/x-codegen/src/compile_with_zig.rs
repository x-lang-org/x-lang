use std::path::PathBuf;
use std::fs::read_to_string;
use x_parser::parse_program;
use x_codegen::{get_code_generator, CodeGenConfig, Target};

fn main() {
    // 解析X语言文件
    let file_path = PathBuf::from("../../test_zig_stdlib.x");
    let code = read_to_string(&file_path).expect("读取文件失败");
    let program = parse_program(&code).expect("解析失败");
    
    // 创建Zig后端的代码生成器
    let config = CodeGenConfig {
        target: Target::Zig,
        output_dir: Some(PathBuf::from("../../../")),
        optimize: false,
        debug_info: true,
    };
    
    let mut generator = get_code_generator(Target::Zig, config).expect("创建代码生成器失败");
    
    // 生成Zig代码
    let output = generator.generate_from_ast(&program).expect("生成代码失败");
    
    // 保存生成的Zig代码
    for file in output.files {
        println!("生成文件: {:?}", file.path);
        std::fs::write(&file.path, file.content).expect("写入文件失败");
    }
    
    println!("编译完成！");
}
