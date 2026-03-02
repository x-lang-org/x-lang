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
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("无法创建目标目录: {}", e))?;

    if let Some(main_path) = &main_file {
        build_source(main_path, &project, &target_dir)?;
    }

    if let Some(lib_path) = &lib_file {
        build_source(lib_path, &project, &target_dir)?;
    }

    let elapsed = start.elapsed();
    utils::status(
        "Finished",
        &format!(
            "`{}` profile [{}] target(s) in {}",
            profile,
            if release { "optimized" } else { "unoptimized + debuginfo" },
            utils::elapsed_str(elapsed)
        ),
    );

    Ok(())
}

fn build_source(
    path: &std::path::Path,
    _project: &Project,
    _target_dir: &std::path::Path,
) -> Result<(), String> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;

    let parser = x_parser::parser::XParser::new();
    let program = parser.parse(&source).map_err(|e| {
        pipeline::format_parse_error(&path.display().to_string(), &source, &e)
    })?;

    x_typechecker::type_check(&program).map_err(|e| format!("类型检查错误: {}", e))?;

    #[cfg(feature = "codegen")]
    {
        let _hir = x_hir::ast_to_hir(&program)
            .map_err(|e| format!("HIR 转换错误: {}", e))?;
        let _pir = x_perceus::analyze_hir(&_hir)
            .map_err(|e| format!("Perceus 分析错误: {}", e))?;

        let config = x_codegen::CodeGenConfig {
            target: x_codegen::Target::Native,
            ..Default::default()
        };
        let bytes = x_codegen::generate_code(&program, &config)
            .map_err(|e| format!("代码生成失败: {}", e))?;

        let obj_name = format!(
            "{}.{}",
            project.name(),
            if cfg!(windows) { "obj" } else { "o" }
        );
        let obj_path = target_dir.join(&obj_name);
        std::fs::write(&obj_path, &bytes)
            .map_err(|e| format!("无法写入 {}: {}", obj_path.display(), e))?;

        let exe_name = if cfg!(windows) {
            format!("{}.exe", project.name())
        } else {
            project.name().to_string()
        };
        let exe_path = target_dir.join(&exe_name);
        pipeline::try_link(
            obj_path.to_str().unwrap(),
            exe_path.to_str().unwrap(),
        );
    }

    Ok(())
}
