use crate::manifest::Dependency;
use crate::project::Project;

#[allow(unused_variables)]
pub fn exec(
    depth: Option<usize>,
    invert: bool,
    no_dedupe: bool,
    prefix: &str,
) -> Result<(), String> {
    let project = Project::find()?;

    println!(
        "{} v{} ({})",
        project.name(),
        project.version(),
        project.root.display()
    );

    let deps: Vec<_> = project.manifest.dependencies.iter().collect();
    let dev_deps: Vec<_> = project.manifest.dev_dependencies.iter().collect();

    let max_depth = depth.unwrap_or(usize::MAX);

    print_deps(&deps, "", max_depth, 1);

    if !dev_deps.is_empty() {
        println!("[dev-dependencies]");
        print_deps(&dev_deps, "", max_depth, 1);
    }

    Ok(())
}

fn print_deps(
    deps: &[(&String, &Dependency)],
    prefix: &str,
    max_depth: usize,
    current_depth: usize,
) {
    if current_depth > max_depth {
        return;
    }

    for (i, (name, dep)) in deps.iter().enumerate() {
        let is_last = i == deps.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let version = dep.version().unwrap_or("*");

        let source_info = match dep {
            Dependency::Detailed(d) => {
                if let Some(p) = &d.path {
                    format!(" ({})", p)
                } else if let Some(g) = &d.git {
                    format!(" ({})", g)
                } else {
                    String::new()
                }
            }
            _ => String::new(),
        };

        println!("{}{}{} v{}{}", prefix, connector, name, version, source_info);
    }
}
