use crate::project::Project;
use crate::resolver;
use crate::utils;

pub fn exec() -> Result<(), String> {
    let project = Project::find()?;

    utils::status("Generating", "x.lock");

    let lockfile = resolver::generate_lockfile(&project.manifest);
    let lock_path = project.root.join("x.lock");

    lockfile.save(&lock_path)?;

    utils::status(
        "Generated",
        &format!("x.lock ({} packages)", lockfile.packages.len()),
    );
    Ok(())
}
