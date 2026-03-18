use crate::config;
use crate::utils;

pub fn exec(packages: &[String], root: Option<&str>) -> Result<(), String> {
    let install_dir = match root {
        Some(r) => std::path::PathBuf::from(r),
        None => config::install_root(),
    };

    if packages.is_empty() {
        return Err("请指定要卸载的包名".to_string());
    }

    for name in packages {
        let candidates = if cfg!(windows) {
            vec![
                install_dir.join(format!("{}.cmd", name)),
                install_dir.join(format!("{}.exe", name)),
                install_dir.join(name),
            ]
        } else {
            vec![install_dir.join(name)]
        };

        let mut found = false;
        for path in &candidates {
            if path.exists() {
                std::fs::remove_file(path)
                    .map_err(|e| format!("无法删除 {}: {}", path.display(), e))?;
                utils::status("Removed", &format!("{}", path.display()));
                found = true;
            }
        }

        if !found {
            utils::warning(&format!(
                "未找到已安装的包 `{}`（在 {} 中）",
                name,
                install_dir.display()
            ));
        }
    }

    Ok(())
}
