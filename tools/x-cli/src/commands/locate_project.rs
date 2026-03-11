use crate::manifest::Manifest;
use serde::Serialize;

#[derive(Serialize)]
struct ProjectLocation {
    root: String,
}

#[allow(unused_variables)]
pub fn exec(workspace: bool) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| format!("无法获取当前目录: {}", e))?;

    let manifest_path = Manifest::find_manifest_path(&cwd)
        .ok_or_else(|| "在当前目录及其父目录中找不到 x.toml".to_string())?;

    let location = ProjectLocation {
        root: manifest_path.display().to_string(),
    };

    let json = serde_json::to_string(&location).map_err(|e| format!("JSON 序列化失败: {}", e))?;
    println!("{}", json);

    Ok(())
}
