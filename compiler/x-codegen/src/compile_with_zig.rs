use std::fs::read_to_string;
use std::path::PathBuf;
use x_codegen::{get_code_generator, CodeGenConfig, Target};
use x_parser::parse_program;

fn default_input_path() -> PathBuf {
    PathBuf::from("../../test_zig_stdlib.x")
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("../../../")
}

fn run(input_path: PathBuf, output_dir: PathBuf) -> Result<Vec<PathBuf>, String> {
    let code = read_to_string(&input_path).map_err(|e| format!("读取文件失败: {e}"))?;
    let program = parse_program(&code).map_err(|e| format!("解析失败: {e}"))?;

    let config = CodeGenConfig {
        target: Target::Native,
        output_dir: Some(output_dir.clone()),
        optimize: false,
        debug_info: true,
    };
    let mut generator =
        get_code_generator(Target::Native, config).map_err(|e| format!("创建代码生成器失败: {e}"))?;
    let output = generator
        .generate_from_ast(&program)
        .map_err(|e| format!("生成代码失败: {e}"))?;

    let mut written = Vec::new();
    for file in output.files {
        let dest = if file.path.is_absolute() {
            file.path
        } else {
            output_dir.join(file.path)
        };
        std::fs::write(&dest, file.content).map_err(|e| format!("写入文件失败: {e}"))?;
        written.push(dest);
    }
    Ok(written)
}

fn main() {
    let input_path = default_input_path();
    let output_dir = default_output_dir();
    let written = run(input_path, output_dir).expect("编译失败");
    for path in written {
        println!("生成文件: {:?}", path);
    }
    println!("编译完成！");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_writes_files_to_output_dir() {
        let temp = std::env::temp_dir().join(format!(
            "x_codegen_compile_with_zig_test_{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&temp);
        let input = temp.join("input.x");
        std::fs::write(&input, "function main() { print(\"hi\"); }").unwrap();

        let out_dir = temp.join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let written = run(input, out_dir.clone()).expect("run ok");
        assert!(!written.is_empty());
        for p in written {
            assert!(p.to_string_lossy().contains(out_dir.to_string_lossy().as_ref()));
            let bytes = std::fs::read(&p).unwrap();
            assert!(!bytes.is_empty());
        }
    }
}
