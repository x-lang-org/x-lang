use crate::project::Project;
use crate::resolver;
use crate::utils;

#[allow(unused_variables)]
pub fn exec(packages: &[String], aggressive: bool, dry_run: bool) -> Result<(), String> {
    let project = Project::find()?;

    utils::status(
        "Updating",
        &format!("{} v{}", project.name(), project.version()),
    );

    let lockfile = resolver::generate_lockfile(&project.manifest);
    let lock_path = project.root.join("x.lock");

    if dry_run {
        utils::note("试运行模式，不会修改 x.lock");
        for pkg in &lockfile.packages {
            println!("  {} v{}", pkg.name, pkg.version);
        }
        return Ok(());
    }

    lockfile.save(&lock_path)?;

    utils::status(
        "Updated",
        &format!("x.lock ({} packages)", lockfile.packages.len()),
    );

    Ok(())
}
