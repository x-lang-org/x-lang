use crate::project::Project;

pub fn exec(spec: Option<&str>) -> Result<(), String> {
    let project = Project::find()?;

    if let Some(spec) = spec {
        if let Some(dep) = project.manifest.dependencies.get(spec) {
            let version = dep.version().unwrap_or("*");
            println!("{}@{}", spec, version);
        } else {
            return Err(format!("未找到包: {}", spec));
        }
    } else {
        println!(
            "{}@{} ({})",
            project.name(),
            project.version(),
            project.manifest_path.display()
        );
    }

    Ok(())
}
