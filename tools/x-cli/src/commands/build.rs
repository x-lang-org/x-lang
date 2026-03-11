use crate::pipeline;
use crate::project::Project;
use crate::utils;
use std::time::Instant;

#[allow(unused_variables)]
pub fn exec(
    release: bool,
    target: Option<&str>,
    jobs: Option<u32>,
    features: &[String],
    all_features: bool,
    no_default_features: bool,
    verbose: bool,
) -> Result<(), String> {
    let project = Project::find()?;
    let start = Instant::now();

    let profile = if release { "release" } else { "dev" };
    utils::status(
        "Compiling",
        &format!(
            "{} v{} ({})",
            project.name(),
            project.version(),
            project.root.display()
        ),
    );

    let main_file = project.main_file();
    let lib_file = project.lib_file();

    if main_file.is_none() && lib_file.is_none() {
        return Err("未找到 src/main.x 或 src/lib.x".to_string());
    }

    let target_dir = project.target_dir().join(profile);
    std::fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建目标目录: {}", e))?;

    if let Some(main_path) = &main_file {
        build_source(main_path, &project, &target_dir, release)?;
    }

    if let Some(lib_path) = &lib_file {
        build_source(lib_path, &project, &target_dir, release)?;
    }

    let elapsed = start.elapsed();
    utils::status(
        "Finished",
        &format!(
            "`{}` profile [{}] target(s) in {}",
            profile,
            if release {
                "optimized"
            } else {
                "unoptimized + debuginfo"
            },
            utils::elapsed_str(elapsed)
        ),
    );

    Ok(())
}

fn build_source(
    path: &std::path::Path,
    project: &Project,
    target_dir: &std::path::Path,
    release: bool,
) -> Result<(), String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;

    let parser = x_parser::parser::XParser::new();
    let program = parser
        .parse(&source)
        .map_err(|e| pipeline::format_parse_error(&path.display().to_string(), &source, &e))?;

    x_typechecker::type_check(&program).map_err(|e| format!("类型检查错误: {}", e))?;

    // Use Zig backend directly for now, since it's the most mature
    let mut backend = x_codegen::zig_backend::ZigBackend::new(x_codegen::zig_backend::ZigBackendConfig {
        output_dir: Some(target_dir.to_path_buf()),
        optimize: release,
        debug_info: !release,
    });

    let codegen_output = backend
        .generate_from_ast(&program)
        .map_err(|e| format!("代码生成失败: {}", e))?;

    let zig_code = String::from_utf8_lossy(&codegen_output.files[0].content);

    // Get output name: use file stem for binaries, project name for lib
    let output_name = if path.file_name().unwrap_or_default() == "main.x" {
        project.name().to_string()
    } else {
        path.file_stem().unwrap().to_string_lossy().to_string()
    };

    let output_path = target_dir.join(&output_name);

    backend
        .compile_zig_code(&zig_code, &output_path)
        .map_err(|e| format!("Zig编译失败: {}", e))?;

    utils::status("Built", &format!("{} -> {}", path.display(), output_path.display()));

    Ok(())
}
