use crate::project::Project;
use crate::registry::RegistryClient;
use crate::utils;

pub fn exec(
    dry_run: bool,
    no_verify: bool,
    allow_dirty: bool,
    registry: Option<&str>,
    _token: Option<&str>,
) -> Result<(), String> {
    let project = Project::find()?;

    utils::status(
        "Publishing",
        &format!(
            "{} v{} to {}",
            project.name(),
            project.version(),
            registry.unwrap_or("default registry")
        ),
    );

    if !allow_dirty {
        let git_status = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&project.root)
            .output();
        if let Ok(output) = git_status {
            let status = String::from_utf8_lossy(&output.stdout);
            if !status.is_empty() {
                return Err("工作目录有未提交的更改，使用 --allow-dirty 跳过检查".to_string());
            }
        }
    }

    if !no_verify {
        super::package::exec(false, true, allow_dirty, None)?;
    }

    if dry_run {
        utils::note("试运行模式，不会实际发布");
        utils::status("Finished", "dry run complete");
        return Ok(());
    }

    let client = RegistryClient::new(registry);
    client.publish(&[])?;

    utils::status(
        "Published",
        &format!("{} v{}", project.name(), project.version()),
    );
    Ok(())
}
