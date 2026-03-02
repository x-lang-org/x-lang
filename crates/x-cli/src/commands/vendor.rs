use crate::project::Project;
use crate::resolver::{self, DepSource};
use crate::utils;

pub fn exec(path: Option<&str>, no_delete: bool, versioned_dirs: bool) -> Result<(), String> {
    let project = Project::find()?;
    let vendor_dir = match path {
        Some(p) => project.root.join(p),
        None => project.root.join("vendor"),
    };

    utils::status(
        "Vendoring",
        &format!("dependencies to {}", vendor_dir.display()),
    );

    if !no_delete && vendor_dir.exists() {
        std::fs::remove_dir_all(&vendor_dir)
            .map_err(|e| format!("无法清除 vendor 目录: {}", e))?;
    }

    std::fs::create_dir_all(&vendor_dir)
        .map_err(|e| format!("无法创建 vendor 目录: {}", e))?;

    let resolver = resolver::Resolver::new(project.manifest.clone(), None);
    let deps = resolver.resolve()?;

    let mut vendored = 0;
    for dep in &deps {
        let dest = if versioned_dirs {
            vendor_dir.join(format!("{}-{}", dep.name, dep.version))
        } else {
            vendor_dir.join(&dep.name)
        };

        match &dep.source {
            DepSource::Path(p) => {
                let src = std::path::Path::new(p);
                if src.exists() {
                    copy_dir_recursive(src, &dest)?;
                    vendored += 1;
                } else {
                    utils::warning(&format!("路径不存在: {}", p));
                }
            }
            _ => {
                std::fs::create_dir_all(&dest)
                    .map_err(|e| format!("无法创建目录 {}: {}", dest.display(), e))?;
                std::fs::write(
                    dest.join("x.toml"),
                    format!(
                        "[package]\nname = \"{}\"\nversion = \"{}\"\n",
                        dep.name, dep.version
                    ),
                )
                .map_err(|e| format!("写入失败: {}", e))?;
                vendored += 1;
            }
        }
    }

    println!(
        "\n# 将以下内容添加到 .x/config.toml 以使用 vendor 目录:\n\
         [source.vendor]\n\
         directory = \"{}\"",
        vendor_dir.display()
    );

    utils::status("Vendored", &format!("{} dependencies", vendored));
    Ok(())
}

fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("无法创建 {}: {}", dest.display(), e))?;

    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry.map_err(|e| format!("遍历目录失败: {}", e))?;
        let rel = entry.path().strip_prefix(src).unwrap();
        let target = dest.join(rel);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| format!("无法创建 {}: {}", target.display(), e))?;
        } else {
            std::fs::copy(entry.path(), &target)
                .map_err(|e| format!("无法复制 {}: {}", entry.path().display(), e))?;
        }
    }
    Ok(())
}
