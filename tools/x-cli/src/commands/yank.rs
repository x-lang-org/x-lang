use crate::registry::RegistryClient;
use crate::utils;

pub fn exec(
    package: &str,
    version: &str,
    undo: bool,
    registry: Option<&str>,
) -> Result<(), String> {
    let client = RegistryClient::from_registry_name(registry)?;

    if undo {
        client.unyank(package, version)?;
        utils::status("Unyank", &format!("{} v{}", package, version));
    } else {
        client.yank(package, version)?;
        utils::status("Yank", &format!("{} v{}", package, version));
    }

    Ok(())
}
