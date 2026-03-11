use crate::config;
use crate::utils;

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
        return Err("从 Git 安装尚未实现".to_string());
    }

    if let Some(pkg) = package {
        utils::status("Installing", pkg);
        return Err("从注册表安装尚未实现。请使用 --path 从本地路径安装".to_string());
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

fn install_from_path(path: &str, install_dir: &std::path::Path, force: bool) -> Result<(), String> {
    let abs = std::path::Path::new(path)
        .canonicalize()
        .map_err(|e| format!("无法解析路径 {}: {}", path, e))?;
    let project = crate::project::Project::find_from(&abs)?;

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

    #[cfg(windows)]
    {
        let script = format!("@echo off\nx run \"{}\" -- %*\n", main_path.display());
        std::fs::write(&script_path, script).map_err(|e| format!("无法写入安装脚本: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        let script = format!("#!/bin/sh\nx run \"{}\" -- \"$@\"\n", main_path.display());
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
