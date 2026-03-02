use crate::project::Project;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct Metadata {
    version: u32,
    workspace_root: String,
    target_directory: String,
    packages: Vec<PackageMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolve: Option<ResolveMeta>,
}

#[derive(Serialize)]
struct PackageMeta {
    name: String,
    version: String,
    id: String,
    source: Option<String>,
    dependencies: Vec<DepMeta>,
    targets: Vec<TargetMeta>,
    features: BTreeMap<String, Vec<String>>,
    manifest_path: String,
    edition: String,
}

#[derive(Serialize)]
struct DepMeta {
    name: String,
    req: String,
    kind: Option<String>,
    optional: bool,
}

#[derive(Serialize)]
struct TargetMeta {
    name: String,
    kind: Vec<String>,
    src_path: String,
}

#[derive(Serialize)]
struct ResolveMeta {
    nodes: Vec<ResolveNode>,
}

#[derive(Serialize)]
struct ResolveNode {
    id: String,
    dependencies: Vec<String>,
}

pub fn exec(no_deps: bool, format_version: Option<u32>) -> Result<(), String> {
    let project = Project::find()?;

    let mut targets = Vec::new();
    if let Some(main) = project.main_file() {
        targets.push(TargetMeta {
            name: project.name().to_string(),
            kind: vec!["bin".to_string()],
            src_path: main.display().to_string(),
        });
    }
    if let Some(lib) = project.lib_file() {
        targets.push(TargetMeta {
            name: project.name().to_string(),
            kind: vec!["lib".to_string()],
            src_path: lib.display().to_string(),
        });
    }

    let mut dependencies = Vec::new();
    for (name, dep) in &project.manifest.dependencies {
        dependencies.push(DepMeta {
            name: name.clone(),
            req: dep.version().unwrap_or("*").to_string(),
            kind: None,
            optional: false,
        });
    }
    for (name, dep) in &project.manifest.dev_dependencies {
        dependencies.push(DepMeta {
            name: name.clone(),
            req: dep.version().unwrap_or("*").to_string(),
            kind: Some("dev".to_string()),
            optional: false,
        });
    }

    let pkg_id = format!(
        "{} {} ({})",
        project.name(),
        project.version(),
        project.manifest_path.display()
    );

    let pkg = PackageMeta {
        name: project.name().to_string(),
        version: project.version().to_string(),
        id: pkg_id.clone(),
        source: None,
        dependencies,
        targets,
        features: project.manifest.features.clone(),
        manifest_path: project.manifest_path.display().to_string(),
        edition: project
            .manifest
            .package
            .as_ref()
            .map(|p| p.edition.clone())
            .unwrap_or_else(|| "2025".to_string()),
    };

    let metadata = Metadata {
        version: format_version.unwrap_or(1),
        workspace_root: project.root.display().to_string(),
        target_directory: project.target_dir().display().to_string(),
        packages: vec![pkg],
        resolve: if no_deps {
            None
        } else {
            Some(ResolveMeta {
                nodes: vec![ResolveNode {
                    id: pkg_id,
                    dependencies: Vec::new(),
                }],
            })
        },
    };

    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("JSON 序列化失败: {}", e))?;
    println!("{}", json);

    Ok(())
}
