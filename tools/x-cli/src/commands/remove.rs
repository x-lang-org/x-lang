use crate::project::Project;
use crate::utils;

pub fn exec(packages: &[String], dev: bool, build: bool) -> Result<(), String> {
    let project = Project::find()?;

    if packages.is_empty() {
        return Err("请指定要移除的包名".to_string());
    }

    let manifest_content = std::fs::read_to_string(&project.manifest_path)
        .map_err(|e| format!("无法读取 x.toml: {}", e))?;

    let mut doc: toml_edit::DocumentMut = manifest_content
        .parse()
        .map_err(|e| format!("解析 x.toml 失败: {}", e))?;

    let section = if dev {
        "dev-dependencies"
    } else if build {
        "build-dependencies"
    } else {
        "dependencies"
    };

    for pkg in packages {
        if let Some(table) = doc.get_mut(section).and_then(|t| t.as_table_mut()) {
            if table.remove(pkg).is_some() {
                utils::status("Removing", &format!("{} from {}", pkg, section));
            } else {
                utils::warning(&format!("未在 {} 中找到依赖 `{}`", section, pkg));
            }
        } else {
            utils::warning(&format!("x.toml 中没有 [{}] 节", section));
        }
    }

    std::fs::write(&project.manifest_path, doc.to_string())
        .map_err(|e| format!("无法写入 x.toml: {}", e))?;

    Ok(())
}
