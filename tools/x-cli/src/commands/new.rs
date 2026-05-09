use crate::manifest::Manifest;
use crate::utils;
use std::path::Path;

pub const MAIN_TEMPLATE: &str = r#"function main() -> Unit {
    println("Hello, X!")
}
"#;

pub const LIB_TEMPLATE: &str = r#"function add(a: integer, b: integer) -> integer {
    a + b
}
"#;

pub const GITIGNORE_TEMPLATE: &str = "/target
*.o
*.obj
*.exe
";

pub fn exec(name: &str, lib: bool, vcs: &str, _edition: Option<&str>) -> Result<(), String> {
    let path = Path::new(name);
    if path.exists() {
        return Err(format!("目录 `{}` 已存在", name));
    }

    let pkg_name =
        utils::sanitize_package_name(path.file_name().and_then(|n| n.to_str()).unwrap_or(name));
    utils::validate_package_name(&pkg_name)?;

    std::fs::create_dir_all(path).map_err(|e| format!("无法创建目录: {}", e))?;

    let manifest = if lib {
        Manifest::default_lib(&pkg_name)
    } else {
        Manifest::default_bin(&pkg_name)
    };

    let manifest_str = manifest.to_string_pretty()?;
    std::fs::write(path.join("x.toml"), manifest_str)
        .map_err(|e| format!("无法写入 x.toml: {}", e))?;

    let src = path.join("src");
    std::fs::create_dir_all(&src).map_err(|e| format!("无法创建 src/: {}", e))?;

    if lib {
        std::fs::write(src.join("lib.x"), LIB_TEMPLATE)
            .map_err(|e| format!("无法写入 src/lib.x: {}", e))?;
    } else {
        std::fs::write(src.join("main.x"), MAIN_TEMPLATE)
            .map_err(|e| format!("无法写入 src/main.x: {}", e))?;
    }

    std::fs::write(path.join(".gitignore"), GITIGNORE_TEMPLATE)
        .map_err(|e| format!("无法写入 .gitignore: {}", e))?;

    if vcs != "none" {
        init_vcs(path, vcs)?;
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

fn init_vcs(path: &Path, vcs: &str) -> Result<(), String> {
    match vcs {
        "git" => {
            match std::process::Command::new("git")
                .args(["init"])
                .current_dir(path)
                .output()
            {
                Ok(o) if o.status.success() => Ok(()),
                Ok(o) => Err(format!(
                    "git init 失败: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(_) => {
                    utils::warning("未找到 git，跳过仓库初始化");
                    Ok(())
                }
            }
        }
        other => {
            utils::warning(&format!("不支持的版本控制系统: {}", other));
            Ok(())
        }
    }
}
