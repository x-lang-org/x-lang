use crate::registry::RegistryClient;
use crate::utils;

pub fn exec(
    package: &str,
    add: &[String],
    remove: &[String],
    list: bool,
    registry: Option<&str>,
) -> Result<(), String> {
    let client = RegistryClient::from_registry_name(registry)?;

    if list || (add.is_empty() && remove.is_empty()) {
        let owners = client.list_owners(package)?;
        println!("Owners of {}:", package);
        for owner in &owners {
            println!("  {}", owner);
        }
        return Ok(());
    }

    for owner in add {
        client.add_owner(package, owner)?;
        utils::status("Owner", &format!("added {} to {}", owner, package));
    }

    for owner in remove {
        client.remove_owner(package, owner)?;
        utils::status("Owner", &format!("removed {} from {}", owner, package));
    }

    Ok(())
}
