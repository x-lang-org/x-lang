use crate::project::Project;
use crate::pipeline;
use crate::utils;

#[allow(unused_variables)]
pub fn exec(
    file: Option<&str>,
    release: bool,
    example: Option<&str>,
    bin: Option<&str>,
    args: &[String],
    quiet: bool,
) -> Result<bool, String> {
    if let Some(f) = file {
        return run_file(f, quiet);
    }

    let project = Project::find()?;

    let source_path = if let Some(example_name) = example {
        let path = project.examples_dir().join(format!("{}.x", example_name));
        if !path.exists() {
            return Err(format!("未找到示例: {}", example_name));
        }
        path
    } else if let Some(bin_name) = bin {
        let found = project
            .manifest
            .bins
            .iter()
            .find(|b| b.name == bin_name)
            .and_then(|b| b.path.as_ref())
            .map(|p| project.root.join(p));
        found.ok_or_else(|| format!("未找到二进制目标: {}", bin_name))?
    } else {
        project
            .main_file()
            .ok_or_else(|| "未找到 src/main.x，且未指定 --bin 或 --example".to_string())?
    };

    if !quiet {
        utils::status("Running", &format!("`{}`", source_path.display()));
    }

    run_file(source_path.to_str().unwrap(), quiet)
}

fn run_file(file: &str, quiet: bool) -> Result<bool, String> {
    let content =
        std::fs::read_to_string(file).map_err(|e| format!("无法读取文件 {}: {}", file, e))?;

    let parser = x_parser::parser::XParser::new();
    let program = parser
        .parse(&content)
        .map_err(|e| format!("解析错误: {}", e))?;

    pipeline::type_check_with_big_stack(&program)?;

    let mut interpreter = x_interpreter::Interpreter::new();
    interpreter
        .run(&program)
        .map_err(|e| format!("运行失败: {}", e))?;

    if !quiet {
        utils::status("Finished", "运行成功");
    }

    Ok(true)
}
