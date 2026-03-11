use crate::project::Project;
use crate::utils;

#[allow(clippy::too_many_arguments)]
pub fn exec(
    packages: &[String],
    dev: bool,
    build: bool,
    optional: bool,
    rename: Option<&str>,
    path: Option<&str>,
    git: Option<&str>,
    branch: Option<&str>,
    tag: Option<&str>,
    rev: Option<&str>,
    features: &[String],
    no_default_features: bool,
) -> Result<(), String> {
    let project = Project::find()?;

    if packages.is_empty() {
        return Err("请指定要添加的包名".to_string());
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

    if doc.get(section).is_none() {
        doc[section] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    for pkg in packages {
        let (name, version) = parse_package_spec(pkg);
        let dep_name = rename.unwrap_or(&name);

        let needs_table = path.is_some()
            || git.is_some()
            || optional
            || !features.is_empty()
            || no_default_features
            || rename.is_some();

        if needs_table {
            let mut table = toml_edit::InlineTable::new();

            if let Some(v) = &version {
                table.insert("version", v.as_str().into());
            }
            if let Some(p) = path {
                table.insert("path", p.into());
            }
            if let Some(g) = git {
                table.insert("git", g.into());
            }
            if let Some(b) = branch {
                table.insert("branch", b.into());
            }
            if let Some(t) = tag {
                table.insert("tag", t.into());
            }
            if let Some(r) = rev {
                table.insert("rev", r.into());
            }
            if optional {
                table.insert("optional", true.into());
            }
            if !features.is_empty() {
                let mut arr = toml_edit::Array::new();
                for f in features {
                    arr.push(f.as_str());
                }
                table.insert("features", toml_edit::Value::Array(arr));
            }
            if no_default_features {
                table.insert("default-features", false.into());
            }
            if rename.is_some() {
                table.insert("package", name.as_str().into());
            }

            doc[section][dep_name] = toml_edit::value(toml_edit::Value::InlineTable(table));
        } else {
            let ver = version.unwrap_or_else(|| "*".to_string());
            doc[section][&name] = toml_edit::value(&ver);
        }

        utils::status("Adding", &format!("{} to {}", pkg, section));
    }

    std::fs::write(&project.manifest_path, doc.to_string())
        .map_err(|e| format!("无法写入 x.toml: {}", e))?;

    Ok(())
}

fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if let Some(at) = spec.rfind('@') {
        let name = spec[..at].to_string();
        let version = spec[at + 1..].to_string();
        (name, Some(version))
    } else {
        (spec.to_string(), None)
    }
}
