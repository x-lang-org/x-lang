use crate::pipeline;
use crate::project::Project;
use crate::utils;
use std::time::Instant;
use x_codegen_zig::{ZigBackend, ZigBackendConfig, ZigTarget};

#[allow(unused_variables, clippy::too_many_arguments)]
pub fn exec(
    release: bool,
    target: Option<&str>,
    jobs: Option<u32>,
    features: &[String],
    all_features: bool,
    no_default_features: bool,
    verbose: bool,
    examples: bool,
) -> Result<(), String> {
    let project = Project::find()?;
    let start = Instant::now();

    // Parse target
    let zig_target = match target {
        None | Some("native") => ZigTarget::Native,
        Some("wasm" | "wasm32-wasi") => ZigTarget::Wasm32Wasi,
        Some("wasm32-freestanding") => ZigTarget::Wasm32Freestanding,
        Some(t) => {
            return Err(format!(
                "未知目标平台: {}（支持: native, wasm, wasm32-wasi, wasm32-freestanding）",
                t
            ))
        }
    };

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

    if zig_target != ZigTarget::Native {
        utils::status("Target", zig_target.as_zig_target());
    }

    let main_file = project.main_file();
    let lib_file = project.lib_file();

    if main_file.is_none() && lib_file.is_none() {
        return Err("未找到 src/main.x 或 src/lib.x".to_string());
    }

    let target_dir = project.target_dir().join(profile);
    std::fs::create_dir_all(&target_dir).map_err(|e| format!("无法创建目标目录: {}", e))?;

    let mut built_count = 0;

    if let Some(main_path) = &main_file {
        build_source(main_path, &project, &target_dir, release, zig_target)?;
        built_count += 1;
    }

    if let Some(lib_path) = &lib_file {
        build_source(lib_path, &project, &target_dir, release, zig_target)?;
        built_count += 1;
    }

    // Build examples if requested
    if examples {
        for example_path in project.example_files() {
            build_source(&example_path, &project, &target_dir, release, zig_target)?;
            built_count += 1;
        }
    }

    let elapsed = start.elapsed();
    utils::status(
        "Finished",
        &format!(
            "`{}` profile [{}] {} target(s) in {}",
            profile,
            if release {
                "optimized"
            } else {
                "unoptimized + debuginfo"
            },
            built_count,
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
    zig_target: ZigTarget,
) -> Result<(), String> {
    let source =
        std::fs::read_to_string(path).map_err(|e| format!("无法读取 {}: {}", path.display(), e))?;

    // Run the full compiler pipeline: source → AST → HIR → MIR → LIR
    let pipeline_output = pipeline::run_pipeline(&source)?;

    // Use Zig backend directly for now, since it's the most mature
    let mut backend = ZigBackend::new(ZigBackendConfig {
        output_dir: Some(target_dir.to_path_buf()),
        optimize: release,
        debug_info: !release,
        target: zig_target,
    });

    let codegen_output = backend
        .generate_from_lir(&pipeline_output.lir)
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

    utils::status(
        "Built",
        &format!("{} -> {}", path.display(), output_path.display()),
    );

    Ok(())
}
