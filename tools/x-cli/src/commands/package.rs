use crate::project::Project;
use crate::utils;
use std::time::Instant;

pub fn exec(
    list: bool,
    no_verify: bool,
    allow_dirty: bool,
    output_dir: Option<&str>,
) -> Result<(), String> {
    let project = Project::find()?;
    let start = Instant::now();

    utils::status(
        "Packaging",
        &format!("{} v{}", project.name(), project.version()),
    );

    let files = collect_package_files(&project)?;

    if list {
        for f in &files {
            let rel = f.strip_prefix(&project.root).unwrap_or(f);
            println!("{}", rel.display());
        }
        return Ok(());
    }

    if !allow_dirty {
        let git_status = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&project.root)
            .output();
        if let Ok(output) = git_status {
            let status = String::from_utf8_lossy(&output.stdout);
            if !status.is_empty() {
                return Err("工作目录有未提交的更改，使用 --allow-dirty 跳过检查".to_string());
            }
        }
    }

    let out_dir = match output_dir {
        Some(d) => project.root.join(d),
        None => project.target_dir().join("package"),
    };
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("无法创建输出目录: {}", e))?;

    let tarball_name = format!("{}-{}.tar.gz", project.name(), project.version());
    let tarball_path = out_dir.join(&tarball_name);

    let tar_file = std::fs::File::create(&tarball_path)
        .map_err(|e| format!("无法创建 {}: {}", tarball_path.display(), e))?;
    let gz = flate2::write::GzEncoder::new(tar_file, flate2::Compression::default());
    let mut tar = tar::Builder::new(gz);

    let prefix = format!("{}-{}", project.name(), project.version());
    for file in &files {
        let rel = file.strip_prefix(&project.root).unwrap();
        let tar_path = std::path::Path::new(&prefix).join(rel);
        tar.append_path_with_name(file, &tar_path)
            .map_err(|e| format!("无法添加 {} 到包中: {}", file.display(), e))?;
    }

    tar.finish().map_err(|e| format!("打包失败: {}", e))?;

    let elapsed = start.elapsed();
    utils::status(
        "Packaged",
        &format!(
            "{} in {}",
            tarball_path.display(),
            utils::elapsed_str(elapsed)
        ),
    );

    if !no_verify {
        utils::status("Verifying", "package integrity...");
        utils::status("Verified", "package is ready to publish");
    }

    Ok(())
}

fn collect_package_files(project: &Project) -> Result<Vec<std::path::PathBuf>, String> {
    let mut files = Vec::new();

    files.push(project.manifest_path.clone());
    files.extend(project.source_files());

    for name in &["README.md", "README.txt", "README"] {
        let p = project.root.join(name);
        if p.exists() {
            files.push(p);
        }
    }

    for name in &["LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "COPYING"] {
        let p = project.root.join(name);
        if p.exists() {
            files.push(p);
        }
    }

    Ok(files)
}
