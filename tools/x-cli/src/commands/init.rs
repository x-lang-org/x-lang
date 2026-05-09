use crate::manifest::Manifest;
use crate::utils;

pub fn exec(
    path: Option<&str>,
    lib: bool,
    vcs: &str,
    _edition: Option<&str>,
) -> Result<(), String> {
    let dir = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir().map_err(|e| format!("无法获取当前目录: {}", e))?,
    };

    if dir.join("x.toml").exists() {
        return Err(format!("`x.toml` 已存在于 {}", dir.display()));
    }

    let pkg_name = utils::sanitize_package_name(
        dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project"),
    );
    let pkg_name = if pkg_name.chars().next().is_some_and(|c| c.is_alphanumeric()) {
        pkg_name
    } else {
        format!("x_{}", pkg_name.trim_start_matches('_'))
    };
    utils::validate_package_name(&pkg_name)?;

    let manifest = if lib {
        Manifest::default_lib(&pkg_name)
    } else {
        Manifest::default_bin(&pkg_name)
    };

    let manifest_str = manifest.to_string_pretty()?;
    std::fs::write(dir.join("x.toml"), manifest_str)
        .map_err(|e| format!("无法写入 x.toml: {}", e))?;

    let src = dir.join("src");
    if !src.exists() {
        std::fs::create_dir_all(&src).map_err(|e| format!("无法创建 src/: {}", e))?;
    }

    if lib {
        let lib_file = src.join("lib.x");
        if !lib_file.exists() {
            std::fs::write(&lib_file, super::new::LIB_TEMPLATE)
                .map_err(|e| format!("无法写入 src/lib.x: {}", e))?;
        }
    } else {
        let main_file = src.join("main.x");
        if !main_file.exists() {
            std::fs::write(&main_file, super::new::MAIN_TEMPLATE)
                .map_err(|e| format!("无法写入 src/main.x: {}", e))?;
        }
    }

    if !dir.join(".gitignore").exists() {
        std::fs::write(dir.join(".gitignore"), super::new::GITIGNORE_TEMPLATE)
            .map_err(|e| format!("无法写入 .gitignore: {}", e))?;
    }

    if vcs != "none" && !dir.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&dir)
            .output();
    }

    utils::status(
        "Created",
        &format!(
            "{} `{}` package",
            if lib {
                "library"
            } else {
                "binary (application)"
            },
            pkg_name
        ),
    );
    Ok(())
}
