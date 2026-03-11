use crate::project::Project;
use crate::resolver::{self, DepSource};
use crate::utils;

pub fn exec() -> Result<(), String> {
    let project = Project::find()?;

    utils::status(
        "Fetching",
        &format!("dependencies for {} v{}", project.name(), project.version()),
    );

    let resolver = resolver::Resolver::new(project.manifest.clone(), None);
    let deps = resolver.resolve()?;

    if deps.is_empty() {
        utils::note("没有需要获取的依赖");
        return Ok(());
    }

    let mut fetched = 0;
    for dep in &deps {
        match &dep.source {
            DepSource::Registry(r) => {
                utils::status(
                    "Downloading",
                    &format!("{} v{} ({})", dep.name, dep.version, r),
                );
                fetched += 1;
            }
            DepSource::Git { url, .. } => {
                utils::status("Fetching", &format!("{} ({})", dep.name, url));
                fetched += 1;
            }
            DepSource::Path(p) => {
                utils::status("Using", &format!("{} (path: {})", dep.name, p));
            }
        }
    }

    utils::status("Finished", &format!("fetched {} dependencies", fetched));
    Ok(())
}
