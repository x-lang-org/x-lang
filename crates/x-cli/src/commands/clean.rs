use crate::project::Project;
use crate::utils;

pub fn exec(doc: bool, release: bool) -> Result<(), String> {
    let project = Project::find()?;
    let target = project.target_dir();

    if doc {
        let doc_dir = target.join("doc");
        if doc_dir.exists() {
            std::fs::remove_dir_all(&doc_dir)
                .map_err(|e| format!("无法删除文档目录: {}", e))?;
            utils::status("Removed", &format!("{}", doc_dir.display()));
        }
        return Ok(());
    }

    if release {
        let release_dir = target.join("release");
        if release_dir.exists() {
            std::fs::remove_dir_all(&release_dir)
                .map_err(|e| format!("无法删除 release 目录: {}", e))?;
            utils::status("Removed", &format!("{}", release_dir.display()));
        }
        return Ok(());
    }

    if target.exists() {
        std::fs::remove_dir_all(&target)
            .map_err(|e| format!("无法删除目标目录: {}", e))?;
        utils::status("Removed", &format!("{}", target.display()));
    } else {
        utils::note("目标目录不存在，无需清理");
    }

    Ok(())
}
