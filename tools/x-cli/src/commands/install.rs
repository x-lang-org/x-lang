use crate::config;
use crate::registry::RegistryClient;
use crate::utils;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::process::Command;

#[allow(unused_variables)]
pub fn exec(
    package: Option<&str>,
    path: Option<&str>,
    git: Option<&str>,
    version: Option<&str>,
    force: bool,
    root: Option<&str>,
    list: bool,
) -> Result<(), String> {
    let install_dir = match root {
        Some(r) => std::path::PathBuf::from(r),
        None => config::install_root(),
    };

    if list {
        return list_installed(&install_dir);
    }

    std::fs::create_dir_all(&install_dir).map_err(|e| format!("无法创建安装目录: {}", e))?;

    if let Some(p) = path {
        return install_from_path(p, &install_dir, force);
    }

    if let Some(g) = git {
        utils::status("Installing", &format!("from git: {}", g));
        if version.is_some() {
            return Err("--version 暂不支持与 --git 一起使用".to_string());
        }
        return install_from_git(g, &install_dir, force);
    }

    if let Some(pkg) = package {
        utils::status("Installing", pkg);
        return install_from_registry(pkg, version, &install_dir, force);
    }

    install_from_path(".", &install_dir, force)
}

fn list_installed(install_dir: &std::path::Path) -> Result<(), String> {
    if !install_dir.exists() {
        println!("没有已安装的包");
        return Ok(());
    }

    let mut found = false;
    if let Ok(entries) = std::fs::read_dir(install_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            if !name.is_empty() {
                println!("    {} ({})", name, path.display());
                found = true;
            }
        }
    }

    if !found {
        println!("没有已安装的包");
    }

    Ok(())
}

fn install_from_path(path: &str, install_dir: &Path, force: bool) -> Result<(), String> {
    let abs = Path::new(path)
        .canonicalize()
        .map_err(|e| format!("无法解析路径 {}: {}", path, e))?;
    install_from_project_root(&abs, install_dir, force)
}

fn install_from_project_root(project_root: &Path, install_dir: &Path, force: bool) -> Result<(), String> {
    ensure_managed_stdlib_available()?;
    let project = crate::project::Project::find_from(project_root)?;

    let main_file = project
        .main_file()
        .ok_or("项目没有可执行目标（未找到 src/main.x）")?;

    utils::status(
        "Installing",
        &format!("{} v{}", project.name(), project.version()),
    );

    let exe_name = project.name();

    #[cfg(windows)]
    let script_path = install_dir.join(format!("{}.cmd", exe_name));
    #[cfg(not(windows))]
    let script_path = install_dir.join(exe_name);

    if script_path.exists() && !force {
        return Err(format!(
            "{} 已存在，使用 --force 覆盖",
            script_path.display()
        ));
    }

    let main_path = main_file
        .canonicalize()
        .map_err(|e| format!("无法获取绝对路径: {}", e))?;
    let x_cli_path = std::env::current_exe().map_err(|e| format!("无法定位当前 x 可执行文件: {}", e))?;

    #[cfg(windows)]
    {
        let script = format!(
            "@echo off\n\"{}\" run \"{}\" -- %*\n",
            x_cli_path.display(),
            main_path.display()
        );
        std::fs::write(&script_path, script).map_err(|e| format!("无法写入安装脚本: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        let script = format!(
            "#!/bin/sh\n\"{}\" run \"{}\" -- \"$@\"\n",
            x_cli_path.display(),
            main_path.display()
        );
        std::fs::write(&script_path, &script).map_err(|e| format!("无法写入安装脚本: {}", e))?;
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("无法设置执行权限: {}", e))?;
    }

    utils::status(
        "Installed",
        &format!("{} -> {}", exe_name, script_path.display()),
    );

    let install_dir_str = install_dir.display().to_string();
    if let Ok(path_var) = std::env::var("PATH") {
        if !path_var.contains(&install_dir_str) {
            utils::warning(&format!(
                "{} 不在 PATH 中，请将其添加到 PATH 环境变量",
                install_dir_str
            ));
        }
    }

    Ok(())
}

fn ensure_managed_stdlib_available() -> Result<(), String> {
    let managed_stdlib = config::managed_stdlib_root();
    if managed_stdlib_is_valid(&managed_stdlib) {
        return Ok(());
    }

    if managed_stdlib.exists() {
        std::fs::remove_dir_all(&managed_stdlib)
            .map_err(|e| format!("无法清理损坏的托管标准库 {}: {}", managed_stdlib.display(), e))?;
    }

    let source_stdlib = crate::pipeline::find_trusted_stdlib_source()?;
    copy_dir_recursive(&source_stdlib, &managed_stdlib)
}

fn managed_stdlib_is_valid(path: &Path) -> bool {
    path.join("prelude.x").exists() && path.join("types.x").exists() && path.join("io.x").exists()
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("无法创建 {}: {}", dest.display(), e))?;

    for entry in walkdir::WalkDir::new(src).min_depth(1) {
        let entry = entry.map_err(|e| format!("无法遍历 {}: {}", src.display(), e))?;
        let relative = entry
            .path()
            .strip_prefix(src)
            .map_err(|e| format!("无法计算相对路径: {}", e))?;
        let target = dest.join(relative);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|e| format!("无法创建 {}: {}", target.display(), e))?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("无法创建 {}: {}", parent.display(), e))?;
            }
            std::fs::copy(entry.path(), &target).map_err(|e| {
                format!(
                    "无法复制 {} 到 {}: {}",
                    entry.path().display(),
                    target.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

fn install_from_git(repo: &str, install_dir: &Path, force: bool) -> Result<(), String> {
    ensure_git_available()?;

    let managed_root = config::git_install_root();
    std::fs::create_dir_all(&managed_root)
        .map_err(|e| format!("无法创建 Git 安装目录 {}: {}", managed_root.display(), e))?;

    let mut hasher = DefaultHasher::new();
    repo.hash(&mut hasher);
    let temp_checkout = managed_root.join(format!(".tmp-install-{:x}", hasher.finish()));

    if temp_checkout.exists() {
        std::fs::remove_dir_all(&temp_checkout)
            .map_err(|e| format!("无法清理旧的临时 Git 目录 {}: {}", temp_checkout.display(), e))?;
    }

    let clone_output = Command::new("git")
        .arg("clone")
        .arg(repo)
        .arg(&temp_checkout)
        .output()
        .map_err(|e| format!("无法执行 git clone: {}", e))?;

    if !clone_output.status.success() {
        let stderr = String::from_utf8_lossy(&clone_output.stderr).trim().to_string();
        let _ = std::fs::remove_dir_all(&temp_checkout);
        return Err(if stderr.is_empty() {
            format!("git clone 失败: {}", repo)
        } else {
            format!("git clone 失败: {}", stderr)
        });
    }

    let project = match crate::project::Project::find_from(&temp_checkout) {
        Ok(project) => project,
        Err(err) => {
            let _ = std::fs::remove_dir_all(&temp_checkout);
            return Err(err);
        }
    };

    let final_checkout = managed_root.join(project.name());
    if final_checkout.exists() {
        if !force {
            let _ = std::fs::remove_dir_all(&temp_checkout);
            return Err(format!(
                "{} 已存在，使用 --force 覆盖",
                final_checkout.display()
            ));
        }
        std::fs::remove_dir_all(&final_checkout)
            .map_err(|e| format!("无法删除现有 Git 安装目录 {}: {}", final_checkout.display(), e))?;
    }

    std::fs::rename(&temp_checkout, &final_checkout).map_err(|e| {
        format!(
            "无法移动 Git 项目到 {}: {}",
            final_checkout.display(),
            e
        )
    })?;

    install_from_project_root(&final_checkout, install_dir, force).inspect_err(|_| {
        let _ = std::fs::remove_dir_all(&final_checkout);
    })
}

fn install_from_registry(
    package: &str,
    version: Option<&str>,
    install_dir: &Path,
    force: bool,
) -> Result<(), String> {
    let client = RegistryClient::from_registry_name(None)?;
    let pkg = client.get_package(package)?;
    let selected = if let Some(requested) = version {
        pkg.versions
            .iter()
            .find(|v| v.version == requested)
            .ok_or_else(|| format!("本地注册表中未找到 {}@{}", package, requested))?
    } else {
        pkg.versions
            .iter()
            .find(|v| v.version == pkg.max_version)
            .ok_or_else(|| format!("本地注册表中未找到 {} 的可安装版本", package))?
    };

    if selected.yanked {
        return Err(format!("{}@{} 已被撤回，无法安装", package, selected.version));
    }

    let tarball = client.read_package_tarball(package, &selected.version)?;
    let managed_root = config::x_home().join("registry").join(package).join(&selected.version);
    let project_root = managed_root.join(format!("{}-{}", package, selected.version));

    if project_root.exists() {
        if !force {
            return Err(format!("{} 已存在，使用 --force 覆盖", project_root.display()));
        }
        std::fs::remove_dir_all(&managed_root)
            .map_err(|e| format!("无法清理现有注册表安装目录 {}: {}", managed_root.display(), e))?;
    }

    std::fs::create_dir_all(&managed_root)
        .map_err(|e| format!("无法创建注册表安装目录 {}: {}", managed_root.display(), e))?;
    let cursor = std::io::Cursor::new(tarball);
    let gz = flate2::read::GzDecoder::new(cursor);
    let mut archive = tar::Archive::new(gz);
    archive
        .unpack(&managed_root)
        .map_err(|e| format!("无法解压本地注册表包: {}", e))?;

    install_from_project_root(&project_root, install_dir, force).inspect_err(|_| {
        let _ = std::fs::remove_dir_all(&managed_root);
    })
}

fn ensure_git_available() -> Result<(), String> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .map_err(|e| format!("未找到 git，请先安装 Git: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err("未找到可用的 git，请先安装 Git".to_string())
    }
}
